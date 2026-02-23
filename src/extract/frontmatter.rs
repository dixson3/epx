use crate::epub::EpubMetadata;
use crate::util::format_iso8601_date;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata YAML for the extracted book
#[derive(Debug, Serialize, Deserialize)]
pub struct BookMetadataYaml {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub creators: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub identifiers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub languages: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub publishers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dates: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub subjects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rights: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub custom: HashMap<String, String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub epx: HashMap<String, String>,
}

impl BookMetadataYaml {
    pub fn from_epub_metadata(meta: &EpubMetadata, epub_version: &str) -> Self {
        let mut epx = HashMap::new();
        epx.insert("source_format".to_string(), "epub".to_string());
        epx.insert("epub_version".to_string(), epub_version.to_string());
        epx.insert("extracted_date".to_string(), format_iso8601_date());

        Self {
            title: meta.titles.first().cloned(),
            creators: meta.creators.clone(),
            identifiers: meta.identifiers.clone(),
            languages: meta.languages.clone(),
            publishers: meta.publishers.clone(),
            dates: meta.dates.clone(),
            description: meta.description.clone(),
            subjects: meta.subjects.clone(),
            rights: meta.rights.clone(),
            custom: meta.custom.clone(),
            epx,
        }
    }

    pub fn to_yaml(&self) -> anyhow::Result<String> {
        Ok(serde_yaml_ng::to_string(self)?)
    }
}

/// Per-chapter frontmatter
#[derive(Debug, Serialize, Deserialize)]
pub struct ChapterFrontmatter {
    pub original_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_id: Option<String>,
    pub spine_index: usize,
}

impl ChapterFrontmatter {
    pub fn to_yaml_header(&self) -> anyhow::Result<String> {
        let yaml = serde_yaml_ng::to_string(self)?;
        Ok(format!("---\n{yaml}---\n\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epub::EpubMetadata;

    #[test]
    fn test_from_epub_metadata_full() {
        let meta = EpubMetadata {
            titles: vec!["My Book".to_string()],
            creators: vec!["Author".to_string()],
            identifiers: vec!["urn:uuid:test".to_string()],
            languages: vec!["en".to_string()],
            publishers: vec!["Publisher".to_string()],
            description: Some("A description".to_string()),
            subjects: vec!["Fiction".to_string()],
            rights: Some("CC-BY".to_string()),
            ..Default::default()
        };
        let yaml = BookMetadataYaml::from_epub_metadata(&meta, "3.0");
        assert_eq!(yaml.title, Some("My Book".to_string()));
        assert_eq!(yaml.creators, vec!["Author"]);
        assert!(yaml.epx.contains_key("epub_version"));
    }

    #[test]
    fn test_to_yaml_output() {
        let meta = EpubMetadata {
            titles: vec!["My Book".to_string()],
            creators: vec!["Author".to_string()],
            ..Default::default()
        };
        let yaml_obj = BookMetadataYaml::from_epub_metadata(&meta, "3.0");
        let yaml = yaml_obj.to_yaml().unwrap();
        assert!(yaml.contains("title:"), "yaml: {yaml}");
        assert!(yaml.contains("creators:"), "yaml: {yaml}");
    }

    #[test]
    fn test_chapter_frontmatter_to_yaml_header() {
        let fm = ChapterFrontmatter {
            original_file: "ch1.xhtml".to_string(),
            original_id: Some("ch1".to_string()),
            spine_index: 0,
        };
        let header = fm.to_yaml_header().unwrap();
        assert!(header.starts_with("---\n"));
        assert!(header.ends_with("---\n\n"));
        assert!(header.contains("original_file:"));
    }

    #[test]
    fn test_from_epub_metadata_minimal() {
        let meta = EpubMetadata::default();
        let yaml = BookMetadataYaml::from_epub_metadata(&meta, "3.0");
        assert_eq!(yaml.title, None);
        assert!(yaml.creators.is_empty());
    }
}
