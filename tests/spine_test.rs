mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn epx() -> Command {
    Command::cargo_bin("epx").unwrap()
}

#[test]
fn test_spine_list() {
    let fixture = common::fixture_path("minimal-v3.epub");
    epx()
        .args(["spine", "list", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn test_spine_list_json() {
    let fixture = common::fixture_path("minimal-v3.epub");
    epx()
        .args(["spine", "list", fixture.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}

#[test]
fn test_spine_set_from_yaml() {
    let (_tmp, copy) = common::temp_copy("basic-v3plus2.epub");
    let tmp = TempDir::new().unwrap();

    // Get current spine order
    let output = epx()
        .args(["spine", "list", copy.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let spine_json: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    let idrefs: Vec<String> = spine_json.iter()
        .map(|v| v["idref"].as_str().unwrap().to_string())
        .collect();

    // Reverse the order and write as YAML
    let reversed: Vec<String> = idrefs.iter().rev().cloned().collect();
    let yaml_path = tmp.path().join("spine.yml");
    std::fs::write(&yaml_path, serde_yaml_ng::to_string(&reversed).unwrap()).unwrap();

    // Apply the reversed spine
    epx()
        .args(["spine", "set", copy.to_str().unwrap(), yaml_path.to_str().unwrap()])
        .assert()
        .success();

    // Verify the new order
    let output2 = epx()
        .args(["spine", "list", copy.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let spine_json2: Vec<serde_json::Value> = serde_json::from_slice(&output2.stdout).unwrap();
    let new_idrefs: Vec<String> = spine_json2.iter()
        .map(|v| v["idref"].as_str().unwrap().to_string())
        .collect();

    assert_eq!(new_idrefs, reversed);
}

#[test]
fn test_spine_reorder() {
    let (_tmp, copy) = common::temp_copy("basic-v3plus2.epub");

    // Get the original spine order (basic-v3plus2.epub has 3 items)
    let output = epx()
        .args(["spine", "list", copy.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let spine_json: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    let original_idrefs: Vec<String> = spine_json
        .iter()
        .map(|v| v["idref"].as_str().unwrap().to_string())
        .collect();

    assert!(
        original_idrefs.len() >= 2,
        "fixture must have at least 2 spine items for reorder test"
    );

    // Move item from index 0 to index 1
    epx()
        .args([
            "spine", "reorder", copy.to_str().unwrap(),
            "0", "1",
        ])
        .assert()
        .success();

    // Get the new spine order
    let output2 = epx()
        .args(["spine", "list", copy.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let spine_json2: Vec<serde_json::Value> = serde_json::from_slice(&output2.stdout).unwrap();
    let new_idrefs: Vec<String> = spine_json2
        .iter()
        .map(|v| v["idref"].as_str().unwrap().to_string())
        .collect();

    // After moving index 0 to index 1, the first two items should be swapped
    assert_eq!(
        new_idrefs[0], original_idrefs[1],
        "item previously at index 1 should now be at index 0"
    );
    assert_eq!(
        new_idrefs[1], original_idrefs[0],
        "item previously at index 0 should now be at index 1"
    );

    // Total count should be unchanged
    assert_eq!(
        new_idrefs.len(),
        original_idrefs.len(),
        "spine length should not change after reorder"
    );

    // Verify the EPUB is still valid after modification
    common::assert_valid_epub(&copy);
}
