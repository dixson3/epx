use crate::epub::EpubBook;
use std::collections::HashMap;
use std::path::Path;

/// Build path mapping from EPUB-internal paths to extracted paths.
///
/// Maps both asset paths (images, CSS) and chapter cross-references to their
/// extracted equivalents. XHTML files reference resources using relative paths
/// from the XHTML file's directory, so we compute relative-path variants from
/// every XHTML directory to ensure string replacement catches all reference forms.
///
/// `chapter_files` is a list of (manifest_href, extracted_filename) pairs
/// pre-computed from the spine before extraction begins.
pub fn build_path_map(
    book: &EpubBook,
    opf_dir: &str,
    chapter_files: &[(String, String)],
) -> HashMap<String, String> {
    let mut map = HashMap::new();

    // Collect unique XHTML directory prefixes (relative to ZIP root)
    let xhtml_dirs: Vec<String> = book
        .spine
        .iter()
        .filter_map(|si| book.manifest.iter().find(|m| m.id == si.idref))
        .filter(|m| m.media_type.contains("html") || m.media_type.contains("xml"))
        .map(|m| {
            let full = if opf_dir.is_empty() {
                m.href.clone()
            } else {
                format!("{opf_dir}{}", m.href)
            };
            match full.rfind('/') {
                Some(idx) => full[..=idx].to_string(),
                None => String::new(),
            }
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Map asset paths (images, CSS)
    for item in &book.manifest {
        let extracted = if item.media_type.starts_with("image/") {
            let filename = item.href.rsplit('/').next().unwrap_or(&item.href);
            Some(format!("../assets/images/{filename}"))
        } else if item.media_type == "text/css" {
            let filename = item.href.rsplit('/').next().unwrap_or(&item.href);
            Some(format!("../styles/{filename}"))
        } else {
            None
        };

        let Some(extracted) = extracted else {
            continue;
        };

        let asset_zip_path = if opf_dir.is_empty() {
            item.href.clone()
        } else {
            format!("{opf_dir}{}", item.href)
        };

        insert_with_variants(
            &mut map,
            &xhtml_dirs,
            &asset_zip_path,
            &item.href,
            &extracted,
        );
    }

    // Map chapter cross-references (XHTML → markdown filenames)
    for (href, md_filename) in chapter_files {
        let chapter_zip_path = if opf_dir.is_empty() {
            href.clone()
        } else {
            format!("{opf_dir}{href}")
        };

        // Chapters are siblings in chapters/, so just the filename
        insert_with_variants(&mut map, &xhtml_dirs, &chapter_zip_path, href, md_filename);
    }

    map
}

/// Insert a path mapping with all relative-path variants from XHTML directories.
fn insert_with_variants(
    map: &mut HashMap<String, String>,
    xhtml_dirs: &[String],
    zip_path: &str,
    manifest_href: &str,
    extracted: &str,
) {
    map.insert(manifest_href.to_string(), extracted.to_string());
    map.insert(zip_path.to_string(), extracted.to_string());

    for xhtml_dir in xhtml_dirs {
        if let Some(rel) = relative_path(xhtml_dir, zip_path) {
            map.insert(rel, extracted.to_string());
        }
    }
}

/// Compute a relative path from `from_dir` to `to_path` within the ZIP.
/// Both paths use `/` separators. `from_dir` ends with `/` or is empty.
fn relative_path(from_dir: &str, to_path: &str) -> Option<String> {
    // If to_path starts with from_dir, just strip the prefix
    if let Some(rest) = to_path.strip_prefix(from_dir)
        && rest != to_path
    {
        return Some(rest.to_string());
    }

    // Otherwise compute ../ navigation
    let from_parts: Vec<&str> = if from_dir.is_empty() {
        vec![]
    } else {
        from_dir.trim_end_matches('/').split('/').collect()
    };
    let to_parts: Vec<&str> = to_path.split('/').collect();

    // Find common prefix length
    let common = from_parts
        .iter()
        .zip(to_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let ups = from_parts.len() - common;
    let remainder = &to_parts[common..];

    if ups == 0 && remainder.len() == to_parts.len() {
        return None; // No simplification possible
    }

    let mut rel = String::new();
    for _ in 0..ups {
        rel.push_str("../");
    }
    rel.push_str(&remainder.join("/"));
    Some(rel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epub::*;

    fn book_with_manifest(items: Vec<ManifestItem>) -> EpubBook {
        EpubBook {
            manifest: items,
            ..Default::default()
        }
    }

    #[test]
    fn test_build_path_map_images() {
        let book = book_with_manifest(vec![ManifestItem {
            id: "img1".to_string(),
            href: "images/cover.png".to_string(),
            media_type: "image/png".to_string(),
            properties: None,
        }]);
        let map = build_path_map(&book, "", &[]);
        assert_eq!(
            map.get("images/cover.png"),
            Some(&"../assets/images/cover.png".to_string())
        );
    }

    #[test]
    fn test_build_path_map_css() {
        let book = book_with_manifest(vec![ManifestItem {
            id: "css1".to_string(),
            href: "styles/main.css".to_string(),
            media_type: "text/css".to_string(),
            properties: None,
        }]);
        let map = build_path_map(&book, "", &[]);
        assert_eq!(
            map.get("styles/main.css"),
            Some(&"../styles/main.css".to_string())
        );
    }

    #[test]
    fn test_build_path_map_with_opf_dir() {
        let book = book_with_manifest(vec![ManifestItem {
            id: "img1".to_string(),
            href: "images/pic.jpg".to_string(),
            media_type: "image/jpeg".to_string(),
            properties: None,
        }]);
        let map = build_path_map(&book, "OEBPS/", &[]);
        // Both unprefixed and prefixed should be in map
        assert!(map.contains_key("images/pic.jpg"));
        assert!(map.contains_key("OEBPS/images/pic.jpg"));
    }
}

/// Extract all assets from an EPUB to the output directory
pub fn extract_assets(book: &EpubBook, output_dir: &Path, opf_dir: &str) -> anyhow::Result<()> {
    let images_dir = output_dir.join("assets").join("images");
    let styles_dir = output_dir.join("styles");

    for item in &book.manifest {
        let full_path = if opf_dir.is_empty() {
            item.href.clone()
        } else {
            format!("{opf_dir}{}", item.href)
        };

        if item.media_type.starts_with("image/") {
            std::fs::create_dir_all(&images_dir)?;
            let filename = item.href.rsplit('/').next().unwrap_or(&item.href);
            if let Some(data) = book.resources.get(&full_path) {
                std::fs::write(images_dir.join(filename), data)?;
            }
        } else if item.media_type == "text/css" {
            std::fs::create_dir_all(&styles_dir)?;
            let filename = item.href.rsplit('/').next().unwrap_or(&item.href);
            if let Some(data) = book.resources.get(&full_path) {
                std::fs::write(styles_dir.join(filename), data)?;
            }
        } else if item.media_type.contains("font")
            || item.media_type == "application/vnd.ms-opentype"
        {
            let fonts_dir = output_dir.join("assets").join("fonts");
            std::fs::create_dir_all(&fonts_dir)?;
            let filename = item.href.rsplit('/').next().unwrap_or(&item.href);
            if let Some(data) = book.resources.get(&full_path) {
                std::fs::write(fonts_dir.join(filename), data)?;
            }
        }
    }

    Ok(())
}
