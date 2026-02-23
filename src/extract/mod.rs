pub mod asset_extract;
pub mod chapter_org;
pub mod frontmatter;
pub mod html_to_md;
pub mod summary;

use crate::epub::{self, EpubBook};
use crate::extract::frontmatter::ChapterFrontmatter;
use anyhow::Context;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Collect all fragment IDs that are targets of href="#..." links in the EPUB.
///
/// Scans every spine XHTML file for `href="...#fragment"` patterns and returns
/// the set of fragment identifiers. Only IDs in this set should be preserved
/// as anchors during markdown conversion — all others are orphaned.
fn collect_referenced_ids(book: &EpubBook, opf_dir: &str) -> HashSet<String> {
    let mut ids = HashSet::new();
    let href_re = Regex::new(r#"href="[^"]*#([^"]+)""#).expect("valid regex");

    for spine_item in &book.spine {
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

        for cap in href_re.captures_iter(&xhtml) {
            ids.insert(cap[1].to_string());
        }
    }

    ids
}

/// Validate that all markdown links in extracted chapters resolve correctly.
///
/// Scans `chapters/` for `<a id="...">` anchors and `](file.md#fragment)` or
/// `](#fragment)` link references, then cross-checks that every fragment target
/// exists. Returns a list of warning strings for any dangling references.
fn validate_extraction_links(output_dir: &Path) -> Vec<String> {
    let chapters_dir = output_dir.join("chapters");
    if !chapters_dir.exists() {
        return vec![];
    }

    let anchor_re = Regex::new(r#"<a id="([^"]+)"></a>"#).expect("valid regex");
    // Matches [text](file.md#fragment) and [text](#fragment)
    let link_re = Regex::new(r"\]\(([^)]*#[^)]+)\)").expect("valid regex");

    // Collect anchors per file: filename -> set of IDs
    let mut anchors: HashMap<String, HashSet<String>> = HashMap::new();
    let mut md_files: HashSet<String> = HashSet::new();

    let entries: Vec<_> = std::fs::read_dir(&chapters_dir)
        .unwrap_or_else(|_| panic!("read chapters/"))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();

    for entry in &entries {
        let filename = entry.file_name().to_string_lossy().to_string();
        md_files.insert(filename.clone());

        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let mut ids = HashSet::new();
        for cap in anchor_re.captures_iter(&content) {
            ids.insert(cap[1].to_string());
        }
        anchors.insert(filename, ids);
    }

    // Check all links
    let mut warnings = Vec::new();
    for entry in &entries {
        let filename = entry.file_name().to_string_lossy().to_string();
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();

        for cap in link_re.captures_iter(&content) {
            let link = &cap[1];
            let (target_file, fragment) = if let Some(hash_pos) = link.find('#') {
                let file_part = &link[..hash_pos];
                let frag = &link[hash_pos + 1..];
                if file_part.is_empty() {
                    (filename.clone(), frag.to_string())
                } else {
                    (file_part.to_string(), frag.to_string())
                }
            } else {
                continue;
            };

            // Check file exists
            if !md_files.contains(&target_file) {
                warnings.push(format!(
                    "{filename}: link to non-existent file '{target_file}'"
                ));
                continue;
            }

            // Check fragment target exists
            if !fragment.is_empty() {
                let file_anchors = anchors.get(&target_file);
                if !file_anchors.is_some_and(|a| a.contains(&fragment)) {
                    warnings.push(format!(
                        "{filename}: dangling fragment '#{fragment}' in '{target_file}'"
                    ));
                }
            }
        }
    }

    warnings
}

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

    // Collect referenced fragment IDs (between Pass 1 and path map)
    let referenced_ids = collect_referenced_ids(book, &opf_dir);

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
        let md = html_to_md::xhtml_to_markdown(&xhtml, &path_map, &referenced_ids);

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

    // Post-extraction link validation
    let warnings = validate_extraction_links(output_dir);
    for w in &warnings {
        eprintln!("link warning: {w}");
    }

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

    Ok(html_to_md::xhtml_to_markdown(
        &xhtml,
        &path_map,
        &HashSet::new(),
    ))
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
