use crate::assemble::asset_embed;
use crate::epub::{EpubBook, ManifestItem};
use std::path::Path;

/// Add an asset to an EPUB
pub fn add_asset(
    book: &mut EpubBook,
    asset_path: &Path,
    media_type_override: Option<&str>,
) -> anyhow::Result<String> {
    let filename = asset_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("invalid asset path"))?
        .to_string_lossy()
        .to_string();

    let media_type = media_type_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| asset_embed::infer_media_type(asset_path).to_string());

    let data = std::fs::read(asset_path)?;

    let id = format!("asset-{}", slug::slugify(&filename));
    let href = filename.clone();

    // Add to resources (under OPF dir)
    let opf_dir = book.detect_opf_dir();
    let resource_key = format!("{opf_dir}{href}");
    book.resources.insert(resource_key, data);

    // Add to manifest
    book.manifest.push(ManifestItem {
        id: id.clone(),
        href,
        media_type,
        properties: None,
    });

    Ok(id)
}

/// Remove an asset from an EPUB
pub fn remove_asset(book: &mut EpubBook, asset_path: &str) -> anyhow::Result<()> {
    // Find in manifest
    let item = book
        .manifest
        .iter()
        .find(|m| m.href == asset_path || m.id == asset_path)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("asset not found: {asset_path}"))?;

    // Check if still referenced in any XHTML
    let mut referenced = false;
    for (key, data) in &book.resources {
        if !key.ends_with(".xhtml") && !key.ends_with(".html") {
            continue;
        }
        if let Ok(content) = String::from_utf8(data.clone())
            && content.contains(&item.href)
        {
            referenced = true;
            break;
        }
    }

    if referenced {
        eprintln!(
            "warning: asset {} is still referenced in content",
            item.href
        );
    }

    // Remove from manifest
    book.manifest.retain(|m| m.id != item.id);

    // Remove resource
    let opf_dir = book.detect_opf_dir();
    let resource_key = format!("{opf_dir}{}", item.href);
    book.resources.remove(&resource_key);
    book.resources.remove(&item.href);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epub::*;
    use std::collections::HashMap;

    fn test_book() -> EpubBook {
        let xhtml = b"<html><body><p>Content with <img src=\"test.png\"/> image</p></body></html>";
        let mut resources = HashMap::new();
        resources.insert("OEBPS/ch1.xhtml".to_string(), xhtml.to_vec());

        EpubBook {
            manifest: vec![ManifestItem {
                id: "ch1".to_string(),
                href: "ch1.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: None,
            }],
            spine: vec![SpineItem {
                idref: "ch1".to_string(),
                linear: true,
                properties: None,
            }],
            resources,
            ..Default::default()
        }
    }

    #[test]
    fn test_add_asset_inferred_type() {
        let mut book = test_book();
        let tmp = tempfile::TempDir::new().unwrap();
        let asset_path = tmp.path().join("cover.png");
        std::fs::write(&asset_path, b"fake png data").unwrap();

        let id = add_asset(&mut book, &asset_path, None).unwrap();
        let item = book.manifest.iter().find(|m| m.id == id).unwrap();
        assert_eq!(item.media_type, "image/png");
    }

    #[test]
    fn test_add_asset_explicit_type() {
        let mut book = test_book();
        let tmp = tempfile::TempDir::new().unwrap();
        let asset_path = tmp.path().join("data.bin");
        std::fs::write(&asset_path, b"binary data").unwrap();

        let id = add_asset(&mut book, &asset_path, Some("application/x-custom")).unwrap();
        let item = book.manifest.iter().find(|m| m.id == id).unwrap();
        assert_eq!(item.media_type, "application/x-custom");
    }

    #[test]
    fn test_remove_asset_existing() {
        let mut book = test_book();
        let tmp = tempfile::TempDir::new().unwrap();
        let asset_path = tmp.path().join("test.css");
        std::fs::write(&asset_path, "body {}").unwrap();

        let id = add_asset(&mut book, &asset_path, None).unwrap();
        let manifest_len = book.manifest.len();

        // Remove by href
        let item = book.manifest.iter().find(|m| m.id == id).unwrap();
        let href = item.href.clone();
        remove_asset(&mut book, &href).unwrap();
        assert_eq!(book.manifest.len(), manifest_len - 1);
    }

    #[test]
    fn test_remove_asset_not_found() {
        let mut book = test_book();
        assert!(remove_asset(&mut book, "nonexistent.png").is_err());
    }

    #[test]
    fn test_remove_asset_still_referenced() {
        let mut book = test_book();
        // Add an asset whose href "test.png" is referenced in ch1.xhtml content
        let tmp = tempfile::TempDir::new().unwrap();
        let asset_path = tmp.path().join("test.png");
        std::fs::write(&asset_path, b"png data").unwrap();

        add_asset(&mut book, &asset_path, None).unwrap();
        // Should warn but not error
        let result = remove_asset(&mut book, "test.png");
        assert!(result.is_ok());
    }
}
