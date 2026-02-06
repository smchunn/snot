use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::error::Result;

/// Scan a vault directory for all markdown files.
pub fn scan_vault(vault_path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(vault_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        // Skip .snot directory
        if path.components().any(|c| c.as_os_str() == ".snot") {
            continue;
        }
        if path.is_file() && is_markdown(path) {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

/// Calculate SHA-256 checksum of a file.
pub fn calculate_checksum(path: &Path) -> Result<String> {
    let content = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "md")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_calculate_checksum() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("test.md");
        fs::write(&file, "hello world").unwrap();

        let checksum = calculate_checksum(&file).unwrap();
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA-256 hex

        // Same content = same checksum
        let checksum2 = calculate_checksum(&file).unwrap();
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_scan_vault() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("note1.md"), "# Note 1").unwrap();
        fs::write(tmp.path().join("note2.md"), "# Note 2").unwrap();
        fs::write(tmp.path().join("readme.txt"), "not markdown").unwrap();

        // Create .snot dir with a file that should be skipped
        let snot_dir = tmp.path().join(".snot");
        fs::create_dir_all(&snot_dir).unwrap();
        fs::write(snot_dir.join("db.bin"), "data").unwrap();

        let files = scan_vault(tmp.path()).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.extension().unwrap() == "md"));
    }
}
