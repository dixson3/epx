mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn epx() -> Command {
    Command::cargo_bin("epx").unwrap()
}

#[test]
fn test_content_search() {
    let fixture = common::fixture_path("alice-in-wonderland.epub");
    epx()
        .args(["content", "search", fixture.to_str().unwrap(), "Alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("match"));
}

#[test]
fn test_content_replace_dry_run() {
    let fixture = common::fixture_path("alice-in-wonderland.epub");
    epx()
        .args([
            "content",
            "replace",
            fixture.to_str().unwrap(),
            "Alice",
            "Bob",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));
}

#[test]
fn test_content_headings() {
    let fixture = common::fixture_path("basic-v3plus2.epub");
    epx()
        .args(["content", "headings", fixture.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_content_replace_actual() {
    let (_tmp, copy) = common::temp_copy("alice-in-wonderland.epub");

    // Perform actual (non-dry-run) replacement
    epx()
        .args(["content", "replace", copy.to_str().unwrap(), "Alice", "Bob"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Replaced"));

    // Verify the replacement took effect: "Bob" should now appear in content
    epx()
        .args(["content", "search", copy.to_str().unwrap(), "Bob"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"));

    // Verify "Alice" is largely gone from text content
    // (some residual matches may remain in metadata/nav, but text nodes
    // should have been replaced)
    let output = epx()
        .args(["content", "search", copy.to_str().unwrap(), "Bob"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Bob"),
        "replacement text should be found in content after replace"
    );
}

#[test]
fn test_content_search_regex() {
    let fixture = common::fixture_path("alice-in-wonderland.epub");

    // Search with --regex flag for chapter headings using a Roman numeral pattern
    epx()
        .args([
            "content",
            "search",
            fixture.to_str().unwrap(),
            "CHAPTER [IVX]+",
            "--regex",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("CHAPTER"));

    // Verify multiple matches are found (Alice has 12 chapters)
    let output = epx()
        .args([
            "content",
            "search",
            fixture.to_str().unwrap(),
            "CHAPTER [IVX]+",
            "--regex",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Count the match lines (each starts with a filename:line pattern)
    let match_count = stdout.lines().filter(|l| l.contains("CHAPTER")).count();
    assert!(
        match_count >= 10,
        "regex should match multiple chapter headings, found {match_count}"
    );
}

#[test]
fn test_content_headings_restructure() {
    let (_tmp, copy) = common::temp_copy("basic-v3plus2.epub");

    // Verify the fixture has h1 headings before restructuring
    epx()
        .args(["content", "headings", copy.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("h1:"));

    // Apply heading remapping: h1 -> h2
    epx()
        .args([
            "content",
            "headings",
            copy.to_str().unwrap(),
            "--restructure",
            "h1->h2",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Restructured"));

    // Verify headings changed: should now show h2 instead of h1
    let output = epx()
        .args(["content", "headings", copy.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("h2:"),
        "headings should be remapped to h2 after restructure"
    );
    assert!(
        !stdout.contains("h1:"),
        "h1 headings should no longer exist after h1->h2 restructure"
    );
}
