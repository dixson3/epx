pub mod asset_extract;
pub mod chapter_org;
pub mod frontmatter;
pub mod html_to_md;
pub mod summary;

use crate::epub::{self, EpubBook};
use crate::extract::frontmatter::ChapterFrontmatter;
use anyhow::Context;
use std::path::Path;

/// Extract a full EPUB to the opinionated directory structure
pub fn extract_book(book: &EpubBook, output_dir: &Path) -> anyhow::Result<()> {
    let opf_dir = book.detect_opf_dir();

    // Create directory structure
    let chapters_dir = output_dir.join("chapters");
    std::fs::create_dir_all(&chapters_dir)?;

    // Pass 1: pre-compute chapter href → markdown filename mapping
    let mut chapter_files: Vec<(String, String)> = Vec::new();
    for (index, spine_item) in book.spine.iter().enumerate() {
        let Some(manifest_item) = book.manifest.iter().find(|m| m.id == spine_item.idref) else {
            continue;
        };
        if !manifest_item.media_type.contains("html") && !manifest_item.media_type.contains("xml") {
            continue;
        }
        let chapter_filename = chapter_org::chapter_filename(index, book, &manifest_item.href);
        chapter_files.push((manifest_item.href.clone(), chapter_filename));
    }

    // Build path map for asset + chapter cross-reference rewriting
    let path_map = asset_extract::build_path_map(book, &opf_dir, &chapter_files);

    // Pass 2: extract chapters using the complete path map
    let mut written_chapters: Vec<(String, String)> = Vec::new();

    for (index, spine_item) in book.spine.iter().enumerate() {
        let Some(manifest_item) = book.manifest.iter().find(|m| m.id == spine_item.idref) else {
            continue;
        };
        if !manifest_item.media_type.contains("html") && !manifest_item.media_type.contains("xml") {
            continue;
        }

        let full_path = if opf_dir.is_empty() {
            manifest_item.href.clone()
        } else {
            format!("{opf_dir}{}", manifest_item.href)
        };

        let xhtml = book
            .resources
            .get(&full_path)
            .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
            .unwrap_or_default();

        if xhtml.is_empty() {
            continue;
        }

        let chapter_filename = chapter_org::chapter_filename(index, book, &manifest_item.href);

        // Convert XHTML to Markdown
        let md = html_to_md::xhtml_to_markdown(&xhtml, &path_map);

        // Generate frontmatter
        let fm = ChapterFrontmatter {
            original_file: manifest_item.href.clone(),
            original_id: Some(manifest_item.id.clone()),
            spine_index: index,
        };
        let header = fm.to_yaml_header()?;

        // Write chapter file
        let chapter_path = chapters_dir.join(&chapter_filename);
        std::fs::write(&chapter_path, format!("{header}{md}"))
            .with_context(|| format!("writing {}", chapter_path.display()))?;

        written_chapters.push((manifest_item.href.clone(), chapter_filename));
    }

    // Generate metadata.yml
    let meta_yaml = frontmatter::BookMetadataYaml::from_epub_metadata(
        &book.metadata,
        &book.navigation.epub_version.to_string(),
    );
    std::fs::write(output_dir.join("metadata.yml"), meta_yaml.to_yaml()?)?;

    // Generate SUMMARY.md
    let summary_content = summary::generate_summary(&book.navigation.toc, &written_chapters);
    std::fs::write(output_dir.join("SUMMARY.md"), summary_content)?;

    // Extract assets
    asset_extract::extract_assets(book, output_dir, &opf_dir)?;

    Ok(())
}

/// Extract a single chapter by ID or index
pub fn extract_single_chapter(book: &EpubBook, id_or_index: &str) -> anyhow::Result<String> {
    let opf_dir = book.detect_opf_dir();
    let path_map = asset_extract::build_path_map(book, &opf_dir, &[]);

    let (manifest_item, _index) = find_chapter(book, id_or_index)?;

    let full_path = if opf_dir.is_empty() {
        manifest_item.href.clone()
    } else {
        format!("{opf_dir}{}", manifest_item.href)
    };

    let xhtml = book
        .resources
        .get(&full_path)
        .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
        .ok_or_else(|| anyhow::anyhow!("chapter content not found: {}", manifest_item.href))?;

    Ok(html_to_md::xhtml_to_markdown(&xhtml, &path_map))
}

fn find_chapter(book: &EpubBook, id_or_index: &str) -> anyhow::Result<(epub::ManifestItem, usize)> {
    // Try as index first
    if let Ok(index) = id_or_index.parse::<usize>()
        && let Some(spine_item) = book.spine.get(index)
        && let Some(item) = book.manifest.iter().find(|m| m.id == spine_item.idref)
    {
        return Ok((item.clone(), index));
    }

    // Try as ID
    for (i, spine_item) in book.spine.iter().enumerate() {
        if spine_item.idref == id_or_index
            && let Some(item) = book.manifest.iter().find(|m| m.id == spine_item.idref)
        {
            return Ok((item.clone(), i));
        }
    }

    anyhow::bail!("chapter not found: {id_or_index}")
}
