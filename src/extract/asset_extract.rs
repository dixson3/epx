use crate::epub::EpubBook;
use std::collections::HashMap;
use std::path::Path;

/// Build path mapping from EPUB-internal paths to extracted paths
pub fn build_path_map(book: &EpubBook, opf_dir: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for item in &book.manifest {
        let full_path = if opf_dir.is_empty() {
            item.href.clone()
        } else {
            format!("{opf_dir}{}", item.href)
        };

        if item.media_type.starts_with("image/") {
            let filename = item.href.rsplit('/').next().unwrap_or(&item.href);
            map.insert(item.href.clone(), format!("./assets/images/{filename}"));
            map.insert(full_path, format!("./assets/images/{filename}"));
        } else if item.media_type == "text/css" {
            let filename = item.href.rsplit('/').next().unwrap_or(&item.href);
            map.insert(item.href.clone(), format!("./styles/{filename}"));
            map.insert(full_path, format!("./styles/{filename}"));
        }
    }

    map
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
        let map = build_path_map(&book, "");
        assert_eq!(map.get("images/cover.png"), Some(&"./assets/images/cover.png".to_string()));
    }

    #[test]
    fn test_build_path_map_css() {
        let book = book_with_manifest(vec![ManifestItem {
            id: "css1".to_string(),
            href: "styles/main.css".to_string(),
            media_type: "text/css".to_string(),
            properties: None,
        }]);
        let map = build_path_map(&book, "");
        assert_eq!(map.get("styles/main.css"), Some(&"./styles/main.css".to_string()));
    }

    #[test]
    fn test_build_path_map_with_opf_dir() {
        let book = book_with_manifest(vec![ManifestItem {
            id: "img1".to_string(),
            href: "images/pic.jpg".to_string(),
            media_type: "image/jpeg".to_string(),
            properties: None,
        }]);
        let map = build_path_map(&book, "OEBPS/");
        // Both unprefixed and prefixed should be in map
        assert!(map.contains_key("images/pic.jpg"));
        assert!(map.contains_key("OEBPS/images/pic.jpg"));
    }
}

/// Extract all assets from an EPUB to the output directory
pub fn extract_assets(
    book: &EpubBook,
    output_dir: &Path,
    opf_dir: &str,
) -> anyhow::Result<()> {
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
