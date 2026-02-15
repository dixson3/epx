mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn epx() -> Command {
    Command::cargo_bin("epx").unwrap()
}

#[test]
fn test_chapter_list() {
    let fixture = common::fixture_path("minimal-v3.epub");
    epx()
        .args(["chapter", "list", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn test_chapter_extract() {
    let fixture = common::fixture_path("minimal-v3.epub");
    epx()
        .args(["chapter", "extract", fixture.to_str().unwrap(), "0"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_chapter_add_remove() {
    let (_tmp, copy) = common::temp_copy("minimal-v3.epub");

    // Create a markdown file to add
    let md_dir = tempfile::TempDir::new().unwrap();
    let md_path = md_dir.path().join("new-chapter.md");
    std::fs::write(&md_path, "# New Chapter\n\nNew content here.").unwrap();

    // Add chapter
    epx()
        .args([
            "chapter",
            "add",
            copy.to_str().unwrap(),
            md_path.to_str().unwrap(),
            "--title",
            "New Chapter",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added chapter"));

    // Remove by index 0
    epx()
        .args(["chapter", "remove", copy.to_str().unwrap(), "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed chapter"));
}

#[test]
fn test_chapter_reorder() {
    let (_tmp, copy) = common::temp_copy("basic-v3plus2.epub");
    epx()
        .args(["chapter", "reorder", copy.to_str().unwrap(), "0", "1"])
        .assert()
        .success();
}
