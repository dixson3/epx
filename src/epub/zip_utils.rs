use crate::error::{EpxError, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

pub fn open_epub(path: &Path) -> Result<ZipArchive<File>> {
    let file = File::open(path)?;
    let archive = ZipArchive::new(file)?;
    Ok(archive)
}

pub fn validate_mimetype(archive: &mut ZipArchive<File>) -> Result<()> {
    let mut mimetype = archive.by_index(0).map_err(|_| {
        EpxError::InvalidEpub("missing mimetype entry".into())
    })?;

    if mimetype.name() != "mimetype" {
        return Err(EpxError::InvalidEpub(
            "first entry must be 'mimetype'".into(),
        ));
    }

    let mut content = String::new();
    mimetype.read_to_string(&mut content)?;

    if content.trim() != "application/epub+zip" {
        return Err(EpxError::InvalidEpub(format!(
            "invalid mimetype: {content}"
        )));
    }

    Ok(())
}

pub fn read_entry(archive: &mut ZipArchive<File>, name: &str) -> Result<Vec<u8>> {
    let mut entry = archive.by_name(name).map_err(|_| {
        EpxError::InvalidEpub(format!("missing entry: {name}"))
    })?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn read_entry_string(archive: &mut ZipArchive<File>, name: &str) -> Result<String> {
    let bytes = read_entry(archive, name)?;
    String::from_utf8(bytes).map_err(|e| {
        EpxError::InvalidEpub(format!("invalid UTF-8 in {name}: {e}"))
    })
}

pub fn list_entries(archive: &ZipArchive<File>) -> Vec<String> {
    (0..archive.len())
        .filter_map(|i| archive.name_for_index(i).map(|s| s.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> std::path::PathBuf {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("tests/fixtures");
        p.push(name);
        p
    }

    #[test]
    fn open_epub_valid() {
        let path = fixture("minimal-v3.epub");
        assert!(open_epub(&path).is_ok());
    }

    #[test]
    fn validate_mimetype_valid() {
        let path = fixture("minimal-v3.epub");
        let mut archive = open_epub(&path).unwrap();
        assert!(validate_mimetype(&mut archive).is_ok());
    }

    #[test]
    fn read_entry_string_valid() {
        let path = fixture("minimal-v3.epub");
        let mut archive = open_epub(&path).unwrap();
        let container = read_entry_string(&mut archive, "META-INF/container.xml").unwrap();
        assert!(container.contains("rootfile"));
    }

    #[test]
    fn read_entry_missing() {
        let path = fixture("minimal-v3.epub");
        let mut archive = open_epub(&path).unwrap();
        assert!(read_entry(&mut archive, "nonexistent.txt").is_err());
    }
}
