#[allow(unused_imports)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Resolve a fixture EPUB by name from tests/fixtures/
pub fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    assert!(path.exists(), "fixture not found: {}", path.display());
    path
}

/// Copy a fixture EPUB into a temp directory for mutation tests.
/// Returns (TempDir, path_to_copy). TempDir must be kept alive.
#[allow(dead_code)]
pub fn temp_copy(fixture_name: &str) -> (tempfile::TempDir, PathBuf) {
    let src = fixture_path(fixture_name);
    let tmp = tempfile::TempDir::new().expect("create temp dir");
    let dest = tmp.path().join(fixture_name);
    std::fs::copy(&src, &dest).expect("copy fixture");
    (tmp, dest)
}

/// Shorthand: read a fixture EPUB into an EpubBook
#[allow(dead_code)]
pub fn read_epub_fixture(name: &str) -> epx::epub::EpubBook {
    let path = fixture_path(name);
    epx::epub::reader::read_epub(&path).expect("read fixture epub")
}

/// Create a minimal in-memory EpubBook for unit tests
#[allow(dead_code)]
pub fn create_minimal_book() -> epx::epub::EpubBook {
    use epx::epub::*;

    let xhtml = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Chapter 1</title></head>
<body><h1>Chapter 1</h1><p>Hello world.</p></body>
</html>"#;

    let mut resources = HashMap::new();
    resources.insert(
        "OEBPS/chapter1.xhtml".to_string(),
        xhtml.to_vec(),
    );

    EpubBook {
        metadata: EpubMetadata {
            titles: vec!["Test Book".to_string()],
            creators: vec!["Test Author".to_string()],
            identifiers: vec!["urn:uuid:12345678-1234-1234-1234-123456789abc".to_string()],
            languages: vec!["en".to_string()],
            ..Default::default()
        },
        manifest: vec![ManifestItem {
            id: "chapter1".to_string(),
            href: "chapter1.xhtml".to_string(),
            media_type: "application/xhtml+xml".to_string(),
            properties: None,
        }],
        spine: vec![SpineItem {
            idref: "chapter1".to_string(),
            linear: true,
            properties: None,
        }],
        navigation: Navigation {
            toc: vec![NavPoint {
                label: "Chapter 1".to_string(),
                href: "chapter1.xhtml".to_string(),
                children: Vec::new(),
            }],
            ..Default::default()
        },
        resources,
    }
}

/// Basic structural validation of an EPUB file
#[allow(dead_code)]
pub fn assert_valid_epub(path: &Path) {
    use std::io::Read;

    let file = std::fs::File::open(path).expect("open epub");
    let mut archive = zip::ZipArchive::new(file).expect("open zip");

    // Check mimetype is first entry and stored
    let mimetype = archive.by_index(0).expect("first entry");
    assert_eq!(mimetype.name(), "mimetype");
    assert_eq!(mimetype.compression(), zip::CompressionMethod::Stored);
    drop(mimetype);

    // Read mimetype content
    let mut mimetype = archive.by_name("mimetype").expect("mimetype entry");
    let mut content = String::new();
    mimetype.read_to_string(&mut content).expect("read mimetype");
    assert_eq!(content.trim(), "application/epub+zip");
    drop(mimetype);

    // Check container.xml exists
    archive
        .by_name("META-INF/container.xml")
        .expect("container.xml");
}
