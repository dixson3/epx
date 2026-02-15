pub mod asset_embed;
pub mod md_to_xhtml;
pub mod metadata_build;
pub mod package;
pub mod spine_build;

use crate::epub::{EpubBook, ManifestItem, SpineItem};
use anyhow::Context;
use std::path::Path;

/// Assemble a Markdown directory into an EpubBook
pub fn assemble_book(dir: &Path) -> anyhow::Result<EpubBook> {
    // Read metadata
    let metadata = metadata_build::read_metadata(dir)
        .with_context(|| format!("reading metadata.yml from {}", dir.display()))?;

    // Parse SUMMARY.md for chapter order and navigation
    let (chapter_order, navigation) = spine_build::parse_summary(dir)
        .with_context(|| format!("reading SUMMARY.md from {}", dir.display()))?;

    let chapters_dir = dir.join("chapters");

    let mut manifest: Vec<ManifestItem> = Vec::new();
    let mut spine: Vec<SpineItem> = Vec::new();
    let mut resources: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();

    // Determine stylesheet (check styles/ directory)
    let styles_dir = dir.join("styles");
    let stylesheet_href = if styles_dir.is_dir() {
        let mut css_file = None;
        if let Ok(entries) = std::fs::read_dir(&styles_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "css") {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    let href = format!("styles/{filename}");
                    let data = std::fs::read(&path)
                        .with_context(|| format!("reading {}", path.display()))?;
                    resources.insert(href.clone(), data);
                    manifest.push(ManifestItem {
                        id: format!("style-{}", slug::slugify(&filename)),
                        href: href.clone(),
                        media_type: "text/css".to_string(),
                        properties: None,
                    });
                    if css_file.is_none() {
                        css_file = Some(href);
                    }
                }
            }
        }
        css_file
    } else {
        None
    };

    // Convert chapters
    for (index, chapter_file) in chapter_order.iter().enumerate() {
        let chapter_path = chapters_dir.join(chapter_file);
        if !chapter_path.exists() {
            anyhow::bail!("chapter file not found: {}", chapter_path.display());
        }

        let md_content = std::fs::read_to_string(&chapter_path)
            .with_context(|| format!("reading {}", chapter_path.display()))?;

        // Strip YAML frontmatter if present
        let md_body = strip_frontmatter(&md_content);

        // Derive title from first heading or filename
        let title = extract_title(md_body, chapter_file);

        // Convert to XHTML
        let css_rel = stylesheet_href.as_deref();
        let xhtml = md_to_xhtml::markdown_to_xhtml(md_body, &title, css_rel);

        // Create XHTML filename
        let xhtml_name = chapter_file
            .strip_suffix(".md")
            .unwrap_or(chapter_file);
        let xhtml_href = format!("{xhtml_name}.xhtml");
        let item_id = format!("chapter-{index:02}");

        resources.insert(xhtml_href.clone(), xhtml.into_bytes());

        manifest.push(ManifestItem {
            id: item_id.clone(),
            href: xhtml_href,
            media_type: "application/xhtml+xml".to_string(),
            properties: None,
        });

        spine.push(SpineItem {
            idref: item_id,
            linear: true,
            properties: None,
        });
    }

    // Add assets from assets/ directory
    let assets_dir = dir.join("assets");
    if assets_dir.is_dir() {
        add_assets_recursive(&assets_dir, "assets", &mut manifest, &mut resources)?;
    }

    Ok(EpubBook {
        metadata,
        manifest,
        spine,
        navigation,
        resources,
    })
}

/// Strip YAML frontmatter (--- ... ---) from markdown content
fn strip_frontmatter(content: &str) -> &str {
    if !content.starts_with("---") {
        return content;
    }
    // Find the closing ---
    if let Some(end) = content[3..].find("\n---") {
        let after = end + 3 + 4; // skip past \n---
        if after < content.len() {
            return content[after..].trim_start_matches('\n');
        }
    }
    content
}

/// Extract title from markdown heading or filename
fn extract_title(md: &str, filename: &str) -> String {
    for line in md.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            return heading.trim().to_string();
        }
    }
    // Fallback: clean up filename
    filename
        .strip_suffix(".md")
        .unwrap_or(filename)
        .replace('-', " ")
        .trim()
        .to_string()
}

/// Recursively add assets from a directory
fn add_assets_recursive(
    dir: &Path,
    prefix: &str,
    manifest: &mut Vec<ManifestItem>,
    resources: &mut std::collections::HashMap<String, Vec<u8>>,
) -> anyhow::Result<()> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy();
            let sub_prefix = format!("{prefix}/{dir_name}");
            add_assets_recursive(&path, &sub_prefix, manifest, resources)?;
        } else {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            let href = format!("{prefix}/{filename}");
            let media_type = asset_embed::infer_media_type(&path);
            let data = std::fs::read(&path)
                .with_context(|| format!("reading {}", path.display()))?;

            let id = format!("asset-{}", slug::slugify(&href));
            resources.insert(href.clone(), data);
            manifest.push(ManifestItem {
                id,
                href,
                media_type: media_type.to_string(),
                properties: None,
            });
        }
    }

    Ok(())
}
