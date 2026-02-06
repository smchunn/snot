use crate::error::{Result, SnotError};

/// Magic bytes identifying a SNOT database file.
pub const MAGIC: [u8; 4] = *b"SNOT";

/// Current schema version.
pub const VERSION: u32 = 1;

/// Size of the header in bytes (4 magic + 4 version).
pub const HEADER_SIZE: usize = 8;

/// Write the schema header (magic + version) into a buffer.
pub fn write_header(buf: &mut Vec<u8>) {
    buf.extend_from_slice(&MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
}

/// Validate and strip the schema header from a buffer.
/// Returns the payload bytes after the header.
pub fn read_header(data: &[u8]) -> Result<&[u8]> {
    if data.len() < HEADER_SIZE {
        return Err(SnotError::InvalidMagic);
    }

    if data[..4] != MAGIC {
        return Err(SnotError::InvalidMagic);
    }

    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version != VERSION {
        return Err(SnotError::SchemaVersionMismatch {
            expected: VERSION,
            found: version,
        });
    }

    Ok(&data[HEADER_SIZE..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_header() {
        let mut buf = Vec::new();
        write_header(&mut buf);
        buf.extend_from_slice(b"payload");

        let payload = read_header(&buf).unwrap();
        assert_eq!(payload, b"payload");
    }

    #[test]
    fn test_bad_magic() {
        let data = b"BADMxxxxxxxx";
        assert!(matches!(read_header(data), Err(SnotError::InvalidMagic)));
    }

    #[test]
    fn test_version_mismatch() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&MAGIC);
        buf.extend_from_slice(&99u32.to_le_bytes());
        buf.extend_from_slice(b"payload");

        let err = read_header(&buf).unwrap_err();
        assert!(matches!(
            err,
            SnotError::SchemaVersionMismatch {
                expected: 1,
                found: 99
            }
        ));
    }

    #[test]
    fn test_too_short() {
        let data = b"SNO";
        assert!(matches!(read_header(data), Err(SnotError::InvalidMagic)));
    }
}
