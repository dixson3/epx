use crate::epub::{reader, EpubBook, EpubMetadata};
use crate::epub::writer;
use std::path::Path;

/// Set a metadata field on an EPUB
pub fn set_field(book: &mut EpubBook, field: &str, value: &str) -> anyhow::Result<()> {
    match field {
        "title" => {
            if book.metadata.titles.is_empty() {
                book.metadata.titles.push(value.to_string());
            } else {
                book.metadata.titles[0] = value.to_string();
            }
        }
        "creator" | "author" => {
            book.metadata.creators = vec![value.to_string()];
        }
        "language" => {
            book.metadata.languages = vec![value.to_string()];
        }
        "publisher" => {
            book.metadata.publishers = vec![value.to_string()];
        }
        "description" => {
            book.metadata.description = Some(value.to_string());
        }
        "rights" => {
            book.metadata.rights = Some(value.to_string());
        }
        "identifier" => {
            if book.metadata.identifiers.is_empty() {
                book.metadata.identifiers.push(value.to_string());
            } else {
                book.metadata.identifiers[0] = value.to_string();
            }
        }
        "date" => {
            book.metadata.dates = vec![value.to_string()];
        }
        "subject" => {
            book.metadata.subjects.push(value.to_string());
        }
        other => {
            book.metadata.custom.insert(other.to_string(), value.to_string());
        }
    }
    Ok(())
}

/// Remove a metadata field from an EPUB
pub fn remove_field(book: &mut EpubBook, field: &str) -> anyhow::Result<()> {
    match field {
        "title" => book.metadata.titles.clear(),
        "creator" | "author" => book.metadata.creators.clear(),
        "language" => book.metadata.languages.clear(),
        "publisher" => book.metadata.publishers.clear(),
        "description" => book.metadata.description = None,
        "rights" => book.metadata.rights = None,
        "identifier" => book.metadata.identifiers.clear(),
        "date" => book.metadata.dates.clear(),
        "subject" => book.metadata.subjects.clear(),
        other => {
            book.metadata.custom.remove(other);
        }
    }
    Ok(())
}

/// Import metadata from a YAML file
pub fn import_metadata(book: &mut EpubBook, yaml_path: &Path) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(yaml_path)?;
    let yaml: crate::extract::frontmatter::BookMetadataYaml = serde_yaml_ng::from_str(&content)?;

    book.metadata = EpubMetadata {
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
        custom: Default::default(),
    };

    Ok(())
}

/// Export metadata to a YAML file
pub fn export_metadata(book: &EpubBook, yaml_path: &Path) -> anyhow::Result<()> {
    let yaml = crate::extract::frontmatter::BookMetadataYaml::from_epub_metadata(
        &book.metadata,
        &book.navigation.epub_version.to_string(),
    );
    std::fs::write(yaml_path, yaml.to_yaml()?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_book() -> EpubBook {
        EpubBook {
            metadata: EpubMetadata {
                titles: vec!["Original".to_string()],
                creators: vec!["Author".to_string()],
                identifiers: vec!["urn:uuid:test".to_string()],
                languages: vec!["en".to_string()],
                publishers: vec!["Publisher".to_string()],
                description: Some("A description".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_set_field_title() {
        let mut book = test_book();
        set_field(&mut book, "title", "New Title").unwrap();
        assert_eq!(book.metadata.titles[0], "New Title");
    }

    #[test]
    fn test_set_field_creator() {
        let mut book = test_book();
        set_field(&mut book, "creator", "New Author").unwrap();
        assert_eq!(book.metadata.creators, vec!["New Author"]);
    }

    #[test]
    fn test_set_field_language() {
        let mut book = test_book();
        set_field(&mut book, "language", "fr").unwrap();
        assert_eq!(book.metadata.languages, vec!["fr"]);
    }

    #[test]
    fn test_set_field_description() {
        let mut book = test_book();
        set_field(&mut book, "description", "New desc").unwrap();
        assert_eq!(book.metadata.description, Some("New desc".to_string()));
    }

    #[test]
    fn test_set_field_custom() {
        let mut book = test_book();
        set_field(&mut book, "my-custom", "value").unwrap();
        assert_eq!(book.metadata.custom.get("my-custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_remove_field_title() {
        let mut book = test_book();
        remove_field(&mut book, "title").unwrap();
        assert!(book.metadata.titles.is_empty());
    }

    #[test]
    fn test_remove_field_description() {
        let mut book = test_book();
        remove_field(&mut book, "description").unwrap();
        assert!(book.metadata.description.is_none());
    }

    #[test]
    fn test_export_import_roundtrip() {
        let book = test_book();
        let tmp = tempfile::TempDir::new().unwrap();
        let yaml_path = tmp.path().join("meta.yml");

        export_metadata(&book, &yaml_path).unwrap();
        assert!(yaml_path.exists());

        let mut book2 = EpubBook::default();
        import_metadata(&mut book2, &yaml_path).unwrap();
        assert_eq!(book2.metadata.titles, vec!["Original"]);
        assert_eq!(book2.metadata.creators, vec!["Author"]);
    }
}

/// Read EPUB, modify, write back atomically
pub fn modify_epub(path: &Path, modify: impl FnOnce(&mut EpubBook) -> anyhow::Result<()>) -> anyhow::Result<()> {
    let mut book = reader::read_epub(path)?;
    modify(&mut book)?;
    writer::write_epub(&book, path)?;
    Ok(())
}
