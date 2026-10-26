//! Marcas de tiempo ISO 8601 en UTC, sin dependencias externas.

use std::time::{SystemTime, UNIX_EPOCH};

/// Fecha y hora actuales como `YYYY-MM-DDTHH:MM:SSZ`.
#[must_use]
pub fn utc_now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    format_unix_seconds(secs)
}

/// Formatea segundos desde la época Unix como `YYYY-MM-DDTHH:MM:SSZ`.
#[must_use]
pub fn format_unix_seconds(secs: u64) -> String {
    let days = i64::try_from(secs / 86_400).unwrap_or(i64::MAX / 2);
    let rem = secs % 86_400;
    let (hour, minute, second) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    // Días desde 1970-01-01 a fecha civil (algoritmo de Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_known_instants() {
        assert_eq!(format_unix_seconds(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_unix_seconds(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(format_unix_seconds(1_000_000_000), "2001-09-09T01:46:40Z");
        assert_eq!(format_unix_seconds(1_700_000_000), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn now_has_the_expected_shape() {
        let now = utc_now_iso8601();
        assert_eq!(now.len(), 20);
        assert!(now.ends_with('Z'));
        assert_eq!(&now[10..11], "T");
    }
}
