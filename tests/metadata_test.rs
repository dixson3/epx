mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[allow(unused_imports)]
use tempfile::TempDir;

fn epx() -> Command {
    Command::cargo_bin("epx").unwrap()
}

#[test]
fn test_metadata_show() {
    let fixture = common::fixture_path("minimal-v3.epub");
    epx()
        .args(["metadata", "show", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Title:").or(predicate::str::contains("Language:")));
}

#[test]
fn test_metadata_set() {
    let (_tmp, copy) = common::temp_copy("minimal-v3.epub");

    epx()
        .args([
            "metadata",
            "set",
            copy.to_str().unwrap(),
            "--field",
            "title",
            "--value",
            "New Title",
        ])
        .assert()
        .success();

    // Verify the change
    epx()
        .args(["metadata", "show", copy.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("New Title"));
}

#[test]
fn test_metadata_remove() {
    let (_tmp, copy) = common::temp_copy("minimal-v3.epub");

    epx()
        .args([
            "metadata",
            "remove",
            copy.to_str().unwrap(),
            "--field",
            "description",
        ])
        .assert()
        .success();
}

#[test]
fn test_metadata_show_json() {
    let fixture = common::fixture_path("minimal-v3.epub");
    epx()
        .args(["metadata", "show", fixture.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

#[test]
fn test_metadata_custom_roundtrip() {
    let (_tmp, copy) = common::temp_copy("minimal-v3.epub");

    // Set a custom metadata field
    epx()
        .args([
            "metadata", "set", copy.to_str().unwrap(),
            "--field", "rendition:layout",
            "--value", "pre-paginated",
        ])
        .assert()
        .success();

    // Read back and verify
    epx()
        .args(["metadata", "show", copy.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rendition:layout"));
}

#[test]
fn test_metadata_date_roundtrip() {
    let (_tmp, copy) = common::temp_copy("minimal-v3.epub");

    // Set a date
    epx()
        .args([
            "metadata", "set", copy.to_str().unwrap(),
            "--field", "date",
            "--value", "2024-06-15",
        ])
        .assert()
        .success();

    // Read back and verify
    epx()
        .args(["metadata", "show", copy.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2024-06-15"));
}

#[test]
fn test_metadata_export() {
    let (_tmp, copy) = common::temp_copy("minimal-v3.epub");
    let out_dir = tempfile::TempDir::new().unwrap();
    let yaml_path = out_dir.path().join("metadata.yml");

    // Set some known metadata first so we have predictable values
    epx()
        .args([
            "metadata", "set", copy.to_str().unwrap(),
            "--field", "title",
            "--value", "Export Test Book",
        ])
        .assert()
        .success();

    // Export metadata to YAML
    epx()
        .args([
            "metadata", "export", copy.to_str().unwrap(),
            "-o", yaml_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Verify the exported file exists and contains expected fields
    let content = std::fs::read_to_string(&yaml_path).expect("read exported YAML");
    assert!(content.contains("title:"), "exported YAML should contain 'title:' field");
    assert!(content.contains("Export Test Book"), "exported YAML should contain the title value");
    assert!(content.contains("languages:"), "exported YAML should contain 'languages:' field");
    assert!(content.contains("identifiers:"), "exported YAML should contain 'identifiers:' field");
}

#[test]
fn test_metadata_import() {
    let (_tmp_src, src_copy) = common::temp_copy("minimal-v3.epub");
    let (_tmp_dst, dst_copy) = common::temp_copy("minimal-v3.epub");
    let out_dir = tempfile::TempDir::new().unwrap();
    let yaml_path = out_dir.path().join("metadata.yml");

    // Set distinctive metadata on the source copy
    epx()
        .args([
            "metadata", "set", src_copy.to_str().unwrap(),
            "--field", "title",
            "--value", "Imported Title",
        ])
        .assert()
        .success();

    epx()
        .args([
            "metadata", "set", src_copy.to_str().unwrap(),
            "--field", "creator",
            "--value", "Import Author",
        ])
        .assert()
        .success();

    // Export metadata from source
    epx()
        .args([
            "metadata", "export", src_copy.to_str().unwrap(),
            "-o", yaml_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Import metadata into destination
    epx()
        .args([
            "metadata", "import", dst_copy.to_str().unwrap(),
            yaml_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Verify the destination now has the imported metadata
    epx()
        .args(["metadata", "show", dst_copy.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Imported Title")
                .and(predicate::str::contains("Import Author")),
        );
}
