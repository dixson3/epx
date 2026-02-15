use crate::assemble::md_to_xhtml;
use crate::epub::{EpubBook, ManifestItem, NavPoint, SpineItem};
use std::path::Path;

/// Add a chapter to an EPUB from a Markdown file
pub fn add_chapter(
    book: &mut EpubBook,
    md_path: &Path,
    after: Option<&str>,
    title: Option<&str>,
) -> anyhow::Result<String> {
    let md_content = std::fs::read_to_string(md_path)?;

    let chapter_title = title
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Try to get title from first heading
            for line in md_content.lines() {
                let trimmed = line.trim();
                if let Some(heading) = trimmed.strip_prefix("# ") {
                    return heading.trim().to_string();
                }
            }
            md_path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "New Chapter".to_string())
        });

    let xhtml = md_to_xhtml::markdown_to_xhtml(&md_content, &chapter_title, None);

    // Generate unique ID
    let id = format!("chapter-added-{}", slug::slugify(&chapter_title));
    let href = format!("{}.xhtml", slug::slugify(&chapter_title));

    // Determine insertion position
    let insert_pos = if let Some(after_ref) = after {
        find_spine_position(book, after_ref)?.map(|p| p + 1)
    } else {
        None
    };

    // Add to resources
    let opf_dir = book.detect_opf_dir();
    let resource_key = format!("{opf_dir}{href}");
    book.resources.insert(resource_key, xhtml.into_bytes());

    // Add to manifest
    book.manifest.push(ManifestItem {
        id: id.clone(),
        href: href.clone(),
        media_type: "application/xhtml+xml".to_string(),
        properties: None,
    });

    // Add to spine
    let spine_item = SpineItem {
        idref: id.clone(),
        linear: true,
        properties: None,
    };

    if let Some(pos) = insert_pos {
        book.spine.insert(pos, spine_item);
    } else {
        book.spine.push(spine_item);
    }

    // Add to navigation
    let nav_point = NavPoint {
        label: chapter_title,
        href,
        children: Vec::new(),
    };

    if let Some(pos) = insert_pos {
        if pos <= book.navigation.toc.len() {
            book.navigation.toc.insert(pos, nav_point);
        } else {
            book.navigation.toc.push(nav_point);
        }
    } else {
        book.navigation.toc.push(nav_point);
    }

    Ok(id)
}

/// Remove a chapter from an EPUB by ID or index
pub fn remove_chapter(book: &mut EpubBook, id_or_index: &str) -> anyhow::Result<String> {
    let (spine_idx, idref) = resolve_chapter(book, id_or_index)?;

    // Find manifest item
    let manifest_item = book.manifest.iter()
        .find(|m| m.id == idref)
        .cloned();

    // Remove from spine
    book.spine.remove(spine_idx);

    // Remove from manifest
    book.manifest.retain(|m| m.id != idref);

    // Remove resource
    if let Some(item) = &manifest_item {
        let opf_dir = book.detect_opf_dir();
        let resource_key = format!("{opf_dir}{}", item.href);
        book.resources.remove(&resource_key);
        book.resources.remove(&item.href);

        // Remove from navigation
        remove_from_nav(&mut book.navigation.toc, &item.href);
    }

    Ok(idref)
}

/// Reorder a chapter in the spine
pub fn reorder_chapter(book: &mut EpubBook, from: usize, to: usize) -> anyhow::Result<()> {
    if from >= book.spine.len() {
        anyhow::bail!("source index {from} out of range (0..{})", book.spine.len());
    }
    if to >= book.spine.len() {
        anyhow::bail!("target index {to} out of range (0..{})", book.spine.len());
    }
    let item = book.spine.remove(from);
    book.spine.insert(to, item);
    Ok(())
}

fn find_spine_position(book: &EpubBook, id_or_index: &str) -> anyhow::Result<Option<usize>> {
    if let Ok(index) = id_or_index.parse::<usize>()
        && index < book.spine.len()
    {
        return Ok(Some(index));
    }
    for (i, item) in book.spine.iter().enumerate() {
        if item.idref == id_or_index {
            return Ok(Some(i));
        }
    }
    Ok(None)
}

fn resolve_chapter(book: &EpubBook, id_or_index: &str) -> anyhow::Result<(usize, String)> {
    if let Ok(index) = id_or_index.parse::<usize>()
        && let Some(item) = book.spine.get(index)
    {
        return Ok((index, item.idref.clone()));
    }
    for (i, item) in book.spine.iter().enumerate() {
        if item.idref == id_or_index {
            return Ok((i, item.idref.clone()));
        }
    }
    anyhow::bail!("chapter not found: {id_or_index}")
}

fn remove_from_nav(toc: &mut Vec<NavPoint>, href: &str) {
    toc.retain(|point| point.href != href);
    for point in toc.iter_mut() {
        remove_from_nav(&mut point.children, href);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epub::*;
    use std::collections::HashMap;

    fn test_book() -> EpubBook {
        let xhtml = b"<html><body><h1>Ch1</h1><p>Hello</p></body></html>";
        let xhtml2 = b"<html><body><h1>Ch2</h1><p>Goodbye</p></body></html>";

        let mut resources = HashMap::new();
        resources.insert("OEBPS/ch1.xhtml".to_string(), xhtml.to_vec());
        resources.insert("OEBPS/ch2.xhtml".to_string(), xhtml2.to_vec());

        EpubBook {
            metadata: EpubMetadata {
                titles: vec!["Test".to_string()],
                identifiers: vec!["urn:uuid:test".to_string()],
                languages: vec!["en".to_string()],
                ..Default::default()
            },
            manifest: vec![
                ManifestItem { id: "ch1".to_string(), href: "ch1.xhtml".to_string(), media_type: "application/xhtml+xml".to_string(), properties: None },
                ManifestItem { id: "ch2".to_string(), href: "ch2.xhtml".to_string(), media_type: "application/xhtml+xml".to_string(), properties: None },
            ],
            spine: vec![
                SpineItem { idref: "ch1".to_string(), linear: true, properties: None },
                SpineItem { idref: "ch2".to_string(), linear: true, properties: None },
            ],
            navigation: Navigation {
                toc: vec![
                    NavPoint { label: "Chapter 1".to_string(), href: "ch1.xhtml".to_string(), children: vec![] },
                    NavPoint { label: "Chapter 2".to_string(), href: "ch2.xhtml".to_string(), children: vec![] },
                ],
                ..Default::default()
            },
            resources,
        }
    }

    #[test]
    fn test_add_chapter_at_end() {
        let mut book = test_book();
        let tmp = tempfile::TempDir::new().unwrap();
        let md_path = tmp.path().join("new.md");
        std::fs::write(&md_path, "# New Chapter\n\nContent here.").unwrap();

        let id = add_chapter(&mut book, &md_path, None, None).unwrap();
        assert_eq!(book.spine.len(), 3);
        assert_eq!(book.spine[2].idref, id);
    }

    #[test]
    fn test_add_chapter_with_title() {
        let mut book = test_book();
        let tmp = tempfile::TempDir::new().unwrap();
        let md_path = tmp.path().join("new.md");
        std::fs::write(&md_path, "Some content without heading.").unwrap();

        let id = add_chapter(&mut book, &md_path, None, Some("Custom Title")).unwrap();
        assert!(id.contains("custom-title"));
    }

    #[test]
    fn test_remove_chapter_by_index() {
        let mut book = test_book();
        let removed = remove_chapter(&mut book, "0").unwrap();
        assert_eq!(removed, "ch1");
        assert_eq!(book.spine.len(), 1);
    }

    #[test]
    fn test_remove_chapter_by_id() {
        let mut book = test_book();
        let removed = remove_chapter(&mut book, "ch1").unwrap();
        assert_eq!(removed, "ch1");
        assert!(!book.manifest.iter().any(|m| m.id == "ch1"));
    }

    #[test]
    fn test_remove_chapter_not_found() {
        let mut book = test_book();
        assert!(remove_chapter(&mut book, "nonexistent").is_err());
    }

    #[test]
    fn test_reorder_chapter_out_of_bounds() {
        let mut book = test_book();
        assert!(reorder_chapter(&mut book, 99, 0).is_err());
    }
}
