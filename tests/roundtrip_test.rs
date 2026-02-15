mod common;

use assert_cmd::Command;
use epx::epub::reader::read_epub;
use tempfile::TempDir;

fn epx() -> Command {
    Command::cargo_bin("epx").unwrap()
}

#[test]
fn test_roundtrip_preserves_custom_metadata() {
    // Copy fixture so we can mutate it without affecting other tests
    let (_tmp_fixture, epub_copy) = common::temp_copy("minimal-v3.epub");

    // Set a custom metadata field on the EPUB
    epx()
        .args([
            "metadata",
            "set",
            epub_copy.to_str().unwrap(),
            "--field",
            "rendition:layout",
            "--value",
            "pre-paginated",
        ])
        .assert()
        .success();

    // Extract to a temp directory
    let tmp = TempDir::new().unwrap();
    let extract_dir = tmp.path().join("extracted");
    let assembled = tmp.path().join("reassembled.epub");

    epx()
        .args([
            "book",
            "extract",
            epub_copy.to_str().unwrap(),
            "-o",
            extract_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Verify the custom field appears in the extracted metadata.yml
    let metadata_yml = std::fs::read_to_string(extract_dir.join("metadata.yml"))
        .expect("metadata.yml should exist");
    assert!(
        metadata_yml.contains("rendition:layout"),
        "metadata.yml should contain the custom key 'rendition:layout', got:\n{metadata_yml}"
    );
    assert!(
        metadata_yml.contains("pre-paginated"),
        "metadata.yml should contain the custom value 'pre-paginated', got:\n{metadata_yml}"
    );

    // Assemble back from the extracted directory
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

    // Verify the assembled EPUB is structurally valid
    common::assert_valid_epub(&assembled);

    // Read the assembled EPUB and verify the custom metadata field survived the round-trip
    let book = epx::epub::reader::read_epub(&assembled).expect("read assembled epub");
    assert_eq!(
        book.metadata.custom.get("rendition:layout"),
        Some(&"pre-paginated".to_string()),
        "custom metadata 'rendition:layout' should survive extract-assemble round-trip"
    );
}

#[test]
fn test_roundtrip_minimal_v3() {
    let fixture = common::fixture_path("minimal-v3.epub");
    let tmp = TempDir::new().unwrap();
    let extract_dir = tmp.path().join("extracted");
    let assembled = tmp.path().join("reassembled.epub");

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
fn test_roundtrip_minimal_v2() {
    let fixture = common::fixture_path("minimal-v2.epub");
    let tmp = TempDir::new().unwrap();
    let extract_dir = tmp.path().join("extracted");
    let assembled = tmp.path().join("reassembled.epub");

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
fn test_roundtrip_basic_v3plus2() {
    let fixture = common::fixture_path("basic-v3plus2.epub");
    let tmp = TempDir::new().unwrap();
    let extract_dir = tmp.path().join("extracted");
    let assembled = tmp.path().join("reassembled.epub");

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
fn test_roundtrip_childrens_literature() {
    let fixture = common::fixture_path("childrens-literature.epub");
    let tmp = TempDir::new().unwrap();
    let extract_dir = tmp.path().join("extracted");
    let assembled = tmp.path().join("reassembled.epub");

    // Read original metadata before round-trip
    let original = read_epub(&fixture).expect("read original epub");

    // Extract to markdown
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

    // Assemble back to EPUB
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

    // Validate the resulting EPUB structure
    common::assert_valid_epub(&assembled);

    // Verify key metadata survived the round-trip
    let reassembled = read_epub(&assembled).expect("read reassembled epub");
    // The primary title should survive (subtitle may be stored in custom metadata)
    assert!(
        !reassembled.metadata.titles.is_empty(),
        "at least one title should survive round-trip"
    );
    assert!(
        reassembled.metadata.titles[0].contains("Children's Literature"),
        "primary title should contain 'Children's Literature', got: {:?}",
        reassembled.metadata.titles
    );
    assert_eq!(
        reassembled.metadata.creators, original.metadata.creators,
        "creators should survive round-trip"
    );
    assert_eq!(
        reassembled.metadata.languages, original.metadata.languages,
        "languages should survive round-trip"
    );
    assert!(
        !reassembled.metadata.identifiers.is_empty(),
        "identifiers should survive round-trip"
    );
    assert_eq!(
        reassembled.metadata.subjects, original.metadata.subjects,
        "subjects should survive round-trip"
    );
    // Spine should have entries
    assert!(
        !reassembled.spine.is_empty(),
        "spine should not be empty after round-trip"
    );
}

#[test]
fn test_roundtrip_accessible_epub3() {
    let fixture = common::fixture_path("accessible_epub_3.epub");
    let tmp = TempDir::new().unwrap();
    let extract_dir = tmp.path().join("extracted");
    let assembled = tmp.path().join("reassembled.epub");

    // Read original metadata before round-trip
    let original = read_epub(&fixture).expect("read original epub");

    // Extract to markdown
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

    // Assemble back to EPUB
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

    // Validate the resulting EPUB structure
    common::assert_valid_epub(&assembled);

    // Verify key metadata survived the round-trip
    let reassembled = read_epub(&assembled).expect("read reassembled epub");
    assert_eq!(
        reassembled.metadata.titles, original.metadata.titles,
        "titles should survive round-trip"
    );
    assert_eq!(
        reassembled.metadata.creators, original.metadata.creators,
        "creators should survive round-trip"
    );
    assert_eq!(
        reassembled.metadata.languages, original.metadata.languages,
        "languages should survive round-trip"
    );
    assert!(
        !reassembled.metadata.identifiers.is_empty(),
        "identifiers should survive round-trip"
    );
    // Publisher should survive for this fixture
    assert_eq!(
        reassembled.metadata.publishers, original.metadata.publishers,
        "publishers should survive round-trip"
    );
    // Spine should have entries (exact count may change through content extraction)
    assert!(
        !reassembled.spine.is_empty(),
        "spine should not be empty after round-trip"
    );
}

#[test]
fn test_roundtrip_alice_in_wonderland() {
    let fixture = common::fixture_path("alice-in-wonderland.epub");
    let tmp = TempDir::new().unwrap();
    let extract_dir = tmp.path().join("extracted");
    let assembled = tmp.path().join("reassembled.epub");

    // Read original metadata before round-trip
    let original = read_epub(&fixture).expect("read original epub");

    // Extract to markdown
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

    // Assemble back to EPUB
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

    // Validate the resulting EPUB structure
    common::assert_valid_epub(&assembled);

    // Verify key metadata survived the round-trip
    let reassembled = read_epub(&assembled).expect("read reassembled epub");
    assert_eq!(
        reassembled.metadata.titles, original.metadata.titles,
        "titles should survive round-trip"
    );
    assert_eq!(
        reassembled.metadata.creators, original.metadata.creators,
        "creators should survive round-trip"
    );
    assert_eq!(
        reassembled.metadata.languages, original.metadata.languages,
        "languages should survive round-trip"
    );
    assert!(
        !reassembled.metadata.identifiers.is_empty(),
        "identifiers should survive round-trip"
    );
    // Spine should have at least as many items as the original
    assert!(
        reassembled.spine.len() >= original.spine.len(),
        "spine should have at least {} items, got {}",
        original.spine.len(),
        reassembled.spine.len()
    );
    // Subjects should survive (fantasy fiction, children's stories, etc.)
    assert_eq!(
        reassembled.metadata.subjects, original.metadata.subjects,
        "subjects should survive round-trip"
    );
}

#[test]
fn test_roundtrip_minimal_v3plus2() {
    let fixture = common::fixture_path("minimal-v3plus2.epub");
    let tmp = TempDir::new().unwrap();
    let extract_dir = tmp.path().join("extracted");
    let assembled = tmp.path().join("reassembled.epub");

    // Read original metadata before round-trip
    let original = read_epub(&fixture).expect("read original epub");

    // Extract to markdown
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

    // Assemble back to EPUB
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

    // Validate the resulting EPUB structure
    common::assert_valid_epub(&assembled);

    // Verify key metadata survived the round-trip
    let reassembled = read_epub(&assembled).expect("read reassembled epub");
    assert_eq!(
        reassembled.metadata.titles, original.metadata.titles,
        "titles should survive round-trip"
    );
    assert_eq!(
        reassembled.metadata.languages, original.metadata.languages,
        "languages should survive round-trip"
    );
    assert!(
        !reassembled.metadata.identifiers.is_empty(),
        "identifiers should survive round-trip"
    );
    assert_eq!(
        reassembled.spine.len(),
        original.spine.len(),
        "spine item count should survive round-trip"
    );
}
