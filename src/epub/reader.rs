use crate::epub::{container, navigation, opf, zip_utils, EpubBook};
use crate::error::Result;
use std::path::Path;

/// Read and parse an EPUB file into an EpubBook model
pub fn read_epub(path: &Path) -> Result<EpubBook> {
    let mut archive = zip_utils::open_epub(path)?;
    zip_utils::validate_mimetype(&mut archive)?;

    // Parse container.xml to find OPF path
    let container_xml = zip_utils::read_entry_string(&mut archive, "META-INF/container.xml")?;
    let opf_path = container::parse_container(&container_xml)?;

    // Determine the base directory of the OPF file for resolving relative paths
    let opf_dir = if let Some(idx) = opf_path.rfind('/') {
        &opf_path[..=idx]
    } else {
        ""
    };

    // Parse OPF
    let opf_xml = zip_utils::read_entry_string(&mut archive, &opf_path)?;
    let opf_data = opf::parse_opf(&opf_xml)?;

    // Load resources
    let mut resources = std::collections::HashMap::new();
    let entries = zip_utils::list_entries(&archive);
    for entry_name in &entries {
        if entry_name == "mimetype" || entry_name.starts_with("META-INF/") {
            continue;
        }
        if let Ok(data) = zip_utils::read_entry(&mut archive, entry_name) {
            resources.insert(entry_name.clone(), data);
        }
    }

    // Parse navigation
    let nav = navigation::parse_navigation(&opf_data.manifest, &|href| {
        let full_path = format!("{opf_dir}{href}");
        resources
            .get(&full_path)
            .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
    })?;

    Ok(EpubBook {
        metadata: opf_data.metadata,
        manifest: opf_data.manifest,
        spine: opf_data.spine,
        navigation: nav,
        resources,
    })
}
