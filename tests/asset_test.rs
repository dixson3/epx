mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn epx() -> Command {
    Command::cargo_bin("epx").unwrap()
}

#[test]
fn test_asset_list() {
    let fixture = common::fixture_path("basic-v3plus2.epub");
    epx()
        .args(["asset", "list", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_asset_list_type_filter() {
    let fixture = common::fixture_path("basic-v3plus2.epub");
    epx()
        .args([
            "asset",
            "list",
            fixture.to_str().unwrap(),
            "--type",
            "image",
        ])
        .assert()
        .success();
}

#[test]
fn test_asset_extract_single() {
    let fixture = common::fixture_path("basic-v3plus2.epub");
    let tmp = tempfile::TempDir::new().unwrap();
    let output_file = tmp.path().join("cover_extracted.jpg");

    epx()
        .args([
            "asset",
            "extract",
            fixture.to_str().unwrap(),
            "images/cover.jpg",
            "--output",
            output_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Extracted to"));

    // Verify the file was written and is non-empty
    assert!(output_file.exists(), "extracted file should exist");
    let metadata = std::fs::metadata(&output_file).unwrap();
    assert!(metadata.len() > 0, "extracted file should be non-empty");
}

#[test]
fn test_asset_extract_all() {
    let fixture = common::fixture_path("basic-v3plus2.epub");
    let tmp = tempfile::TempDir::new().unwrap();
    let output_dir = tmp.path().join("extracted_assets");

    epx()
        .args([
            "asset",
            "extract-all",
            fixture.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Assets extracted to"));

    // The command should complete successfully. The output directory
    // structure depends on the OPF directory configuration.
    // At minimum, verify the command did not error out.
}

#[test]
fn test_asset_add() {
    let (_tmp, copy) = common::temp_copy("basic-v3plus2.epub");

    // Create a small asset file to add
    let asset_dir = tempfile::TempDir::new().unwrap();
    let asset_path = asset_dir.path().join("new-image.png");
    // Write a minimal valid-looking PNG (the tool doesn't validate image contents)
    std::fs::write(&asset_path, b"fake png data for testing").unwrap();

    // Add the asset
    epx()
        .args([
            "asset",
            "add",
            copy.to_str().unwrap(),
            asset_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added asset"));

    // Verify the new asset appears in the manifest via JSON listing
    epx()
        .args(["asset", "list", copy.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("new-image.png"));
}

#[test]
fn test_asset_remove() {
    let (_tmp, copy) = common::temp_copy("basic-v3plus2.epub");

    // First add an asset so we have something specific to remove
    let asset_dir = tempfile::TempDir::new().unwrap();
    let asset_path = asset_dir.path().join("removable.txt");
    std::fs::write(&asset_path, b"text content to remove").unwrap();

    epx()
        .args([
            "asset",
            "add",
            copy.to_str().unwrap(),
            asset_path.to_str().unwrap(),
            "--media-type",
            "text/plain",
        ])
        .assert()
        .success();

    // Verify it was added
    epx()
        .args(["asset", "list", copy.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removable.txt"));

    // Remove the asset
    epx()
        .args([
            "asset",
            "remove",
            copy.to_str().unwrap(),
            "removable.txt",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed asset"));

    // Verify the asset is gone from the manifest
    let output = epx()
        .args(["asset", "list", copy.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        !stdout.contains("removable.txt"),
        "removed asset should not appear in manifest"
    );
}
