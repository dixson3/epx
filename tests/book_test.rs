mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn epx() -> Command {
    Command::cargo_bin("epx").unwrap()
}

#[test]
fn test_book_info() {
    let fixture = common::fixture_path("minimal-v3.epub");
    epx()
        .args(["book", "info", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Title:"))
        .stdout(predicate::str::contains("Chapters:"));
}

#[test]
fn test_book_info_json() {
    let fixture = common::fixture_path("minimal-v3.epub");
    epx()
        .args(["book", "info", fixture.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\""));
}

#[test]
fn test_book_info_nonexistent() {
    epx()
        .args(["book", "info", "nonexistent.epub"])
        .assert()
        .failure();
}

#[test]
fn test_book_extract() {
    let fixture = common::fixture_path("minimal-v3.epub");
    let tmp = TempDir::new().unwrap();
    let out_dir = tmp.path().join("extracted");

    epx()
        .args([
            "book",
            "extract",
            fixture.to_str().unwrap(),
            "-o",
            out_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out_dir.join("chapters").exists());
    assert!(out_dir.join("metadata.yml").exists());
    assert!(out_dir.join("SUMMARY.md").exists());
}

#[test]
fn test_book_assemble() {
    let fixture = common::fixture_path("minimal-v3.epub");
    let tmp = TempDir::new().unwrap();
    let extract_dir = tmp.path().join("extracted");
    let assembled = tmp.path().join("output.epub");

    // Extract first
    epx()
        .args([
            "book",
            "extract",
            fixture.to_str().unwrap(),
            "-o",
            extract_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Then assemble
    epx()
        .args([
            "book",
            "assemble",
            extract_dir.to_str().unwrap(),
            "-o",
            assembled.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(assembled.exists());
}

#[test]
fn test_book_validate_valid() {
    let fixture = common::fixture_path("minimal-v3.epub");
    epx()
        .args(["book", "validate", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn test_book_extract_assemble_roundtrip() {
    let fixture = common::fixture_path("minimal-v3.epub");
    let tmp = TempDir::new().unwrap();
    let extract_dir = tmp.path().join("extracted");
    let assembled = tmp.path().join("roundtrip.epub");

    epx()
        .args([
            "book",
            "extract",
            fixture.to_str().unwrap(),
            "-o",
            extract_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    epx()
        .args([
            "book",
            "assemble",
            extract_dir.to_str().unwrap(),
            "-o",
            assembled.to_str().unwrap(),
        ])
        .assert()
        .success();

    common::assert_valid_epub(&assembled);
}

#[test]
fn test_book_validate_missing_title() {
    let (_tmp, copy) = common::temp_copy("minimal-v3.epub");

    // Remove the title
    epx()
        .args([
            "metadata",
            "remove",
            copy.to_str().unwrap(),
            "--field",
            "title",
        ])
        .assert()
        .success();

    epx()
        .args(["book", "validate", copy.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("missing"));
}

#[test]
fn test_book_validate_missing_language() {
    // The writer auto-generates dc:language="en" so we must construct a
    // broken EPUB manually with an OPF that omits dc:language.
    use std::io::Write;
    let tmp = TempDir::new().unwrap();
    let bad_epub = tmp.path().join("no-lang.epub");
    let file = std::fs::File::create(&bad_epub).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let stored =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let deflate = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("mimetype", stored).unwrap();
    zip.write_all(b"application/epub+zip").unwrap();
    zip.start_file("META-INF/container.xml", deflate).unwrap();
    zip.write_all(br#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#).unwrap();
    zip.start_file("OEBPS/content.opf", deflate).unwrap();
    zip.write_all(br#"<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="uid">urn:uuid:test</dc:identifier><dc:title>Test</dc:title></metadata><manifest><item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="ch1"/></spine></package>"#).unwrap();
    zip.start_file("OEBPS/ch1.xhtml", deflate).unwrap();
    zip.write_all(b"<html><body><p>Hello</p></body></html>")
        .unwrap();
    zip.finish().unwrap();

    epx()
        .args(["book", "validate", bad_epub.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("missing"));
}

#[test]
fn test_book_validate_json_output() {
    let fixture = common::fixture_path("minimal-v3.epub");
    epx()
        .args(["book", "validate", fixture.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"valid\""));
}

#[test]
fn test_book_info_corrupt_file() {
    let tmp = TempDir::new().unwrap();
    let corrupt = tmp.path().join("corrupt.epub");
    std::fs::write(&corrupt, b"not a real epub file").unwrap();

    epx()
        .args(["book", "info", corrupt.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn test_book_info_epub2() {
    let fixture = common::fixture_path("minimal-v2.epub");
    epx()
        .args(["book", "info", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Title:"));
}
