use crate::epub::EpubMetadata;
use crate::extract::frontmatter::BookMetadataYaml;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_metadata_full() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("metadata.yml"),
            r#"
title: Test Book
creators:
  - Author Name
identifiers:
  - "urn:uuid:test-id"
languages:
  - en
publishers:
  - Publisher
description: A description
subjects:
  - Fiction
rights: CC-BY
"#,
        )
        .unwrap();

        let meta = read_metadata(tmp.path()).unwrap();
        assert_eq!(meta.titles, vec!["Test Book"]);
        assert_eq!(meta.creators, vec!["Author Name"]);
        assert_eq!(meta.languages, vec!["en"]);
    }

    #[test]
    fn test_read_metadata_missing_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(read_metadata(tmp.path()).is_err());
    }
}

/// Read metadata.yml and convert to EpubMetadata
pub fn read_metadata(dir: &Path) -> anyhow::Result<EpubMetadata> {
    let meta_path = dir.join("metadata.yml");
    let content = std::fs::read_to_string(&meta_path)?;
    let yaml: BookMetadataYaml = serde_yaml_ng::from_str(&content)?;

    Ok(EpubMetadata {
        titles: yaml.title.into_iter().collect(),
        creators: yaml.creators,
        identifiers: yaml.identifiers,
        languages: yaml.languages,
        publishers: yaml.publishers,
        dates: yaml.dates,
        description: yaml.description,
        subjects: yaml.subjects,
        rights: yaml.rights,
        modified: None,
        cover_id: None,
        custom: yaml.custom,
    })
}
