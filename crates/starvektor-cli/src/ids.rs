//! Identificadores de ejecución.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// Identificador único con forma de UUID v4, generado sin dependencias externas.
///
/// No es criptográficamente seguro (no hace falta): solo evita colisiones
/// entre ejecuciones. `RandomState` aporta una semilla aleatoria por proceso.
#[must_use]
pub fn new_run_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let pid = std::process::id();

    let mut bytes = [0_u8; 16];
    for (i, chunk) in bytes.as_chunks_mut::<8>().0.iter_mut().enumerate() {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u128(nanos);
        hasher.write_u32(pid);
        hasher.write_usize(i);
        *chunk = hasher.finish().to_le_bytes();
    }
    bytes[6] = (bytes[6] & 0x0F) | 0x40; // versión 4
    bytes[8] = (bytes[8] & 0x3F) | 0x80; // variante RFC 4122

    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_uuid_v4_shape() {
        let id = new_run_id();
        assert_eq!(id.len(), 36, "{id}");
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(
            parts.iter().map(|p| p.len()).collect::<Vec<_>>(),
            [8, 4, 4, 4, 12]
        );
        assert!(parts[2].starts_with('4'), "versión: {id}");
        assert!(
            matches!(parts[3].as_bytes()[0], b'8' | b'9' | b'a' | b'b'),
            "variante: {id}"
        );
        assert!(id.chars().all(|c| c == '-' || c.is_ascii_hexdigit()));
    }

    #[test]
    fn consecutive_ids_differ() {
        let ids: Vec<String> = (0..100).map(|_| new_run_id()).collect();
        let mut unique = ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), ids.len());
    }
}
