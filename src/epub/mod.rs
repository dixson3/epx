pub mod container;
pub mod navigation;
pub mod opf;
pub mod reader;
pub mod writer;
pub mod zip_utils;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a parsed EPUB book
#[derive(Debug, Default)]
pub struct EpubBook {
    pub metadata: EpubMetadata,
    pub manifest: Vec<ManifestItem>,
    pub spine: Vec<SpineItem>,
    pub navigation: Navigation,
    pub resources: HashMap<String, Vec<u8>>,
}

/// Dublin Core metadata fields
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EpubMetadata {
    pub identifiers: Vec<String>,
    pub titles: Vec<String>,
    pub languages: Vec<String>,
    pub creators: Vec<String>,
    pub publishers: Vec<String>,
    pub dates: Vec<String>,
    pub description: Option<String>,
    pub subjects: Vec<String>,
    pub rights: Option<String>,
    pub modified: Option<String>,
    pub cover_id: Option<String>,
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

/// An item in the EPUB manifest
#[derive(Debug, Clone)]
pub struct ManifestItem {
    pub id: String,
    pub href: String,
    pub media_type: String,
    pub properties: Option<String>,
}

/// A spine item reference
#[derive(Debug, Clone)]
pub struct SpineItem {
    pub idref: String,
    pub linear: bool,
    pub properties: Option<String>,
}

/// Navigation structure
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Navigation {
    pub toc: Vec<NavPoint>,
    pub landmarks: Vec<NavPoint>,
    pub page_list: Vec<NavPoint>,
    pub epub_version: EpubVersion,
}

/// A navigation point in the TOC tree
#[derive(Debug, Clone)]
pub struct NavPoint {
    pub label: String,
    pub href: String,
    pub children: Vec<NavPoint>,
}

/// EPUB version
#[derive(Debug, Default, Clone, Copy)]
pub enum EpubVersion {
    V2,
    #[default]
    V3,
}

impl EpubBook {
    /// Detect the OPF directory prefix from loaded resources.
    ///
    /// Checks for a `.opf` file first, then falls back to common prefixes.
    pub fn detect_opf_dir(&self) -> String {
        for key in self.resources.keys() {
            if key.ends_with(".opf") {
                if let Some(idx) = key.rfind('/') {
                    return format!("{}/", &key[..idx]);
                }
                // OPF at root level — no directory prefix
                return String::new();
            }
        }
        for prefix in &["OEBPS/", "OPS/", "EPUB/", "content/"] {
            if self.resources.keys().any(|k| k.starts_with(prefix)) {
                return prefix.to_string();
            }
        }
        String::new()
    }
}

impl std::fmt::Display for EpubVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpubVersion::V2 => write!(f, "2.0"),
            EpubVersion::V3 => write!(f, "3.0"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_opf_dir_from_opf_path() {
        let mut book = EpubBook::default();
        book.resources
            .insert("OEBPS/content.opf".to_string(), vec![]);
        assert_eq!(book.detect_opf_dir(), "OEBPS/");
    }

    #[test]
    fn detect_opf_dir_fallback_prefix() {
        let mut book = EpubBook::default();
        book.resources
            .insert("OPS/chapter1.xhtml".to_string(), vec![]);
        assert_eq!(book.detect_opf_dir(), "OPS/");
    }

    #[test]
    fn detect_opf_dir_root_level() {
        let mut book = EpubBook::default();
        book.resources.insert("chapter1.xhtml".to_string(), vec![]);
        assert_eq!(book.detect_opf_dir(), "");
    }
}
