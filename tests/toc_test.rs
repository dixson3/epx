mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn epx() -> Command {
    Command::cargo_bin("epx").unwrap()
}

#[test]
fn test_toc_show() {
    let fixture = common::fixture_path("basic-v3plus2.epub");
    epx()
        .args(["toc", "show", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_toc_show_json() {
    let fixture = common::fixture_path("basic-v3plus2.epub");
    epx()
        .args(["toc", "show", fixture.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}

#[test]
fn test_toc_generate() {
    let (_tmp, copy) = common::temp_copy("basic-v3plus2.epub");

    // Capture the original TOC
    let original = epx()
        .args(["toc", "show", copy.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let _original_toc: serde_json::Value =
        serde_json::from_slice(&original.stdout).expect("parse original TOC JSON");

    // Generate a new TOC from headings
    epx()
        .args(["toc", "generate", copy.to_str().unwrap()])
        .assert()
        .success();

    // Read back the TOC and verify it changed
    let updated = epx()
        .args(["toc", "show", copy.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let updated_toc: serde_json::Value =
        serde_json::from_slice(&updated.stdout).expect("parse updated TOC JSON");

    // The generated TOC should be a non-empty array
    assert!(updated_toc.is_array(), "TOC should be a JSON array");
    assert!(
        !updated_toc.as_array().unwrap().is_empty(),
        "generated TOC should not be empty"
    );

    // Verify the generated TOC contains labels derived from headings
    // basic-v3plus2.epub has h1 headings "Ladle Rat Rotten Hut" and
    // "Guilty Looks Enter Tree Beers"
    let toc_str = serde_json::to_string(&updated_toc).unwrap();
    assert!(
        toc_str.contains("Ladle Rat Rotten Hut") || toc_str.contains("Guilty Looks"),
        "generated TOC should contain heading text from the EPUB chapters"
    );

    // Verify the EPUB is still valid after modification
    common::assert_valid_epub(&copy);
}

#[test]
fn test_toc_set() {
    let (_tmp, copy) = common::temp_copy("basic-v3plus2.epub");
    let toc_dir = tempfile::TempDir::new().unwrap();
    let toc_path = toc_dir.path().join("toc.md");

    // Write a custom Markdown TOC file
    // The hrefs must reference existing chapters in the EPUB
    let toc_content = "- [Custom Chapter One](xhtml/section0001.xhtml)\n- [Custom Chapter Two](xhtml/section0002.xhtml)\n";
    std::fs::write(&toc_path, toc_content).expect("write TOC markdown file");

    // Apply the custom TOC
    epx()
        .args([
            "toc", "set", copy.to_str().unwrap(),
            toc_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Read back the TOC and verify the custom labels were applied
    let output = epx()
        .args(["toc", "show", copy.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let toc: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse updated TOC JSON");

    let toc_str = serde_json::to_string(&toc).unwrap();
    assert!(
        toc_str.contains("Custom Chapter One"),
        "TOC should contain the custom label 'Custom Chapter One'"
    );
    assert!(
        toc_str.contains("Custom Chapter Two"),
        "TOC should contain the custom label 'Custom Chapter Two'"
    );

    // Verify the EPUB is still valid after modification
    common::assert_valid_epub(&copy);
}
