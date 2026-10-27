//! Asynchronous JSONL log writer.
//!
//! The frame thread only enqueues the record (`try_send`, without blocking);
//! another thread serializes and writes it. If the queue fills up, the record is
//! dropped and counted: the log must never slow the game down (see the time
//! budget in docs/design.md).

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{self, JoinHandle};

use crate::schema::{Line, Record, RunRecord};

/// Result of closing the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriteStats {
    /// Lines written, including the `run` one.
    pub written: u64,
    /// Records dropped because the queue was full or the writer was down.
    pub dropped: u64,
}

/// Log writer for one run.
pub struct LogWriter {
    tx: Option<SyncSender<Record>>,
    handle: Option<JoinHandle<io::Result<u64>>>,
    dropped: Arc<AtomicU64>,
}

impl LogWriter {
    /// Creates the file and writes the `run` record before returning, so that
    /// it is always the first line. `capacity` is the queue size.
    ///
    /// # Errors
    /// If the directory or the file cannot be created, or the header cannot be written.
    pub fn create(path: &Path, run: RunRecord, capacity: usize) -> io::Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let mut out = BufWriter::new(File::create(path)?);
        write_line(&mut out, &Line::new(Record::Run(Box::new(run))))?;
        out.flush()?;

        let (tx, rx) = sync_channel::<Record>(capacity.max(1));
        let handle = thread::Builder::new()
            .name("sv-log".into())
            .spawn(move || {
                let mut written = 1_u64;
                for record in rx {
                    write_line(&mut out, &Line::new(record))?;
                    written += 1;
                }
                out.flush()?;
                Ok(written)
            })?;

        Ok(Self {
            tx: Some(tx),
            handle: Some(handle),
            dropped: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Enqueues a record without blocking. `false` if it was dropped.
    pub fn log(&self, record: Record) -> bool {
        let sent = self
            .tx
            .as_ref()
            .is_some_and(|tx| tx.try_send(record).is_ok());
        if !sent {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
        sent
    }

    /// Records dropped so far.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// Drains the queue, closes the file and returns the statistics.
    ///
    /// # Errors
    /// If the writer thread failed while writing or panicked.
    pub fn finish(mut self) -> io::Result<WriteStats> {
        self.tx.take();
        let written = match self.handle.take() {
            Some(handle) => handle
                .join()
                .map_err(|_| io::Error::other("the log thread panicked"))??,
            None => 0,
        };
        Ok(WriteStats {
            written,
            dropped: self.dropped(),
        })
    }
}

impl Drop for LogWriter {
    fn drop(&mut self) {
        // If `finish` was not called, the queue is drained anyway (best effort).
        self.tx.take();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn write_line<W: Write>(out: &mut W, line: &Line) -> io::Result<()> {
    serde_json::to_writer(&mut *out, line).map_err(io::Error::from)?;
    out.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::AtomicU32;

    use super::*;
    use crate::fixtures;
    use crate::validate::validate;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_path(name: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sv-telemetry-{}-{id}-{name}", std::process::id()))
    }

    #[test]
    fn writes_run_first_and_every_queued_record() {
        let path = temp_path("full.jsonl");
        let writer = LogWriter::create(&path, fixtures::run(), 1 << 14).unwrap();
        for i in 0..5000 {
            assert!(writer.log(Record::Frame(fixtures::frame(i, i as f64 * 16.0))));
        }
        let stats = writer.finish().unwrap();
        assert_eq!(
            stats,
            WriteStats {
                written: 5001,
                dropped: 0
            }
        );

        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().count(), 5001);
        assert!(text.lines().next().unwrap().contains("\"type\":\"run\""));
        let report = validate(text.as_bytes()).unwrap();
        assert_eq!(report.frames, 5000);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn creates_missing_parent_directories() {
        let dir = temp_path("nested");
        let path = dir.join("a").join("b").join("run.jsonl");
        let writer = LogWriter::create(&path, fixtures::run(), 8).unwrap();
        writer.finish().unwrap();
        assert!(path.exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn dropping_without_finish_still_flushes() {
        let path = temp_path("dropped.jsonl");
        {
            let writer = LogWriter::create(&path, fixtures::run(), 64).unwrap();
            writer.log(Record::Frame(fixtures::frame(0, 0.0)));
        }
        assert_eq!(fs::read_to_string(&path).unwrap().lines().count(), 2);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn fails_cleanly_when_the_path_is_a_directory() {
        let dir = temp_path("isdir");
        fs::create_dir_all(&dir).unwrap();
        assert!(LogWriter::create(&dir, fixtures::run(), 8).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
