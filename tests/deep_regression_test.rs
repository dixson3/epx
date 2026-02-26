mod common;

use assert_cmd::Command;
use std::path::PathBuf;

fn epx() -> Command {
    Command::cargo_bin("epx").unwrap()
}

/// Return the `_resources/` directory path.
fn resources_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("_resources");
    path
}

/// Return the `_books/` output root, creating it if needed.
fn books_output_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("_books");
    std::fs::create_dir_all(&path).expect("create _books/ directory");
    path
}

/// Return `_books/<stem>/`, removing any prior content for a fresh run.
fn book_output_dir(stem: &str) -> PathBuf {
    let dir = books_output_root().join(stem);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clean prior output");
    }
    std::fs::create_dir_all(&dir).expect("create output directory");
    dir
}

/// Discover all `.epub` files in `_resources/`, returning (stem, full_path) pairs.
fn discover_epubs() -> Vec<(String, PathBuf)> {
    let dir = resources_dir();
    assert!(
        dir.exists(),
        "Missing _resources/ directory — deep regression tests require real EPUBs"
    );

    let mut epubs: Vec<(String, PathBuf)> = std::fs::read_dir(&dir)
        .expect("read _resources/")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "epub"))
        .map(|e| {
            let path = e.path();
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            (stem, path)
        })
        .collect();

    epubs.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(!epubs.is_empty(), "No .epub files found in _resources/");
    epubs
}

/// Assert that an extraction directory has the expected structure:
/// - `chapters/` with at least one `.md` file
/// - `metadata.yml` exists and is non-empty
/// - `SUMMARY.md` exists
fn assert_extraction_structure(dir: &std::path::Path) {
    let chapters = dir.join("chapters");
    assert!(chapters.exists(), "chapters/ directory missing");

    let md_count = std::fs::read_dir(&chapters)
        .expect("read chapters/")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        .count();
    assert!(
        md_count >= 1,
        "chapters/ should have >= 1 .md file, found {md_count}"
    );

    let metadata = dir.join("metadata.yml");
    assert!(metadata.exists(), "metadata.yml missing");
    let meta_len = std::fs::metadata(&metadata)
        .expect("stat metadata.yml")
        .len();
    assert!(meta_len > 0, "metadata.yml is empty");

    let summary = dir.join("SUMMARY.md");
    assert!(summary.exists(), "SUMMARY.md missing");
}

/// Assert extraction quality: no SVG base64 blobs, no empty alt text,
/// no raw HTML anchors, metadata contains genre classification.
fn assert_extraction_quality(dir: &std::path::Path, stem: &str) {
    let chapters = dir.join("chapters");
    let mut svg_base64_count = 0;
    let mut empty_alt_count = 0;
    let mut raw_html_anchor_count = 0;

    for entry in std::fs::read_dir(&chapters)
        .expect("read chapters/")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let filename = entry.file_name().to_string_lossy().to_string();

        // No SVG base64 blobs in markdown
        if content.contains("data:image/svg+xml;base64") {
            svg_base64_count += 1;
            eprintln!("  SVG base64 blob found in {stem}/{filename}");
        }

        // No empty alt text: ![](path)
        // Exclude ![]{#id} which is a pandoc span preceded by punctuation `!`
        for (i, line) in content.lines().enumerate() {
            if line.contains("![](") {
                empty_alt_count += 1;
                eprintln!("  Empty alt text in {stem}/{filename}:{}", i + 1);
            }
        }

        // No raw HTML anchors — all anchors should use pandoc syntax
        for (i, line) in content.lines().enumerate() {
            if line.contains("<a id=") {
                raw_html_anchor_count += 1;
                eprintln!("  Raw HTML anchor in {stem}/{filename}:{}", i + 1);
            }
        }
    }

    assert_eq!(
        svg_base64_count, 0,
        "{stem}: found {svg_base64_count} chapters with SVG base64 blobs"
    );
    assert_eq!(
        empty_alt_count, 0,
        "{stem}: found {empty_alt_count} empty alt texts (![])"
    );
    assert_eq!(
        raw_html_anchor_count, 0,
        "{stem}: found {raw_html_anchor_count} raw HTML anchors (<a id=) — should use pandoc {{#id}} syntax"
    );

    // Metadata must contain genre under epx key
    let metadata = std::fs::read_to_string(dir.join("metadata.yml")).unwrap_or_default();
    assert!(
        metadata.contains("genre:"),
        "{stem}: metadata.yml missing genre classification"
    );
}

// ─── Extraction: all EPUBs in _resources/ ────────────────────────

#[test]
#[ignore]
fn deep_extract_all() {
    for (stem, epub_path) in discover_epubs() {
        let out = book_output_dir(&stem);
        eprintln!("Extracting: {stem}");

        epx()
            .args([
                "book",
                "extract",
                epub_path.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
            ])
            .assert()
            .success();

        assert_extraction_structure(&out);
        assert_extraction_quality(&out, &stem);
    }
}

// ─── Roundtrip: all EPUBs in _resources/ ─────────────────────────

#[test]
#[ignore]
fn deep_roundtrip_all() {
    for (stem, epub_path) in discover_epubs() {
        let out = book_output_dir(&format!("{stem}-rt"));
        let assembled = books_output_root().join(format!("{stem}-roundtrip.epub"));
        eprintln!("Roundtrip: {stem}");

        epx()
            .args([
                "book",
                "extract",
                epub_path.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
            ])
            .assert()
            .success();

        epx()
            .args([
                "book",
                "assemble",
                out.to_str().unwrap(),
                "-o",
                assembled.to_str().unwrap(),
            ])
            .assert()
            .success();

        common::assert_valid_epub(&assembled);
    }
}
