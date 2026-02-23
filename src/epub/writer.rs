use crate::epub::{EpubBook, NavPoint};
use crate::util::format_iso8601;
use std::io::Write;
use std::path::Path;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// Write an EpubBook to an EPUB file with atomic rename
pub fn write_epub(book: &EpubBook, path: &Path) -> anyhow::Result<()> {
    let tmp_path = path.with_extension("epub.tmp");
    let file = std::fs::File::create(&tmp_path)?;
    let mut zip = ZipWriter::new(file);

    // 1. mimetype (stored, no compression, first entry)
    let stored = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("mimetype", stored)?;
    zip.write_all(b"application/epub+zip")?;

    let deflate = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 2. META-INF/container.xml
    zip.start_file("META-INF/container.xml", deflate)?;
    zip.write_all(generate_container_xml().as_bytes())?;

    // 3. Determine OPF directory
    let opf_dir = "OEBPS";

    // 4. Generate and write OPF
    let opf = generate_opf(book);
    zip.start_file(format!("{opf_dir}/content.opf"), deflate)?;
    zip.write_all(opf.as_bytes())?;

    // 5. Generate and write navigation
    let toc_xhtml = generate_toc_xhtml(&book.navigation.toc, &book.metadata.titles);
    zip.start_file(format!("{opf_dir}/toc.xhtml"), deflate)?;
    zip.write_all(toc_xhtml.as_bytes())?;

    let toc_ncx = generate_toc_ncx(
        &book.navigation.toc,
        &book.metadata.titles,
        &book.metadata.identifiers,
    );
    zip.start_file(format!("{opf_dir}/toc.ncx"), deflate)?;
    zip.write_all(toc_ncx.as_bytes())?;

    // 6. Write content and assets from resources
    for (path_key, data) in &book.resources {
        // Skip OPF and navigation (already written)
        if path_key.ends_with(".opf")
            || path_key.ends_with("toc.xhtml")
            || path_key.ends_with("toc.ncx")
        {
            continue;
        }
        // Rebase into OEBPS if not already
        let zip_path = if path_key.starts_with("OEBPS/") || path_key.starts_with("META-INF/") {
            path_key.clone()
        } else {
            format!("{opf_dir}/{path_key}")
        };
        zip.start_file(&zip_path, deflate)?;
        zip.write_all(data)?;
    }

    zip.finish()?;

    // Atomic rename
    std::fs::rename(&tmp_path, path)?;

    Ok(())
}

fn generate_container_xml() -> String {
    r##"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"##
        .to_string()
}

fn generate_opf(book: &EpubBook) -> String {
    let mut opf = String::new();
    opf.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    opf.push_str("<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"uid\">\n");

    // Metadata
    opf.push_str("  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n");

    for (i, id) in book.metadata.identifiers.iter().enumerate() {
        if i == 0 {
            opf.push_str(&format!(
                "    <dc:identifier id=\"uid\">{}</dc:identifier>\n",
                xml_escape(id)
            ));
        } else {
            opf.push_str(&format!(
                "    <dc:identifier>{}</dc:identifier>\n",
                xml_escape(id)
            ));
        }
    }
    if book.metadata.identifiers.is_empty() {
        let uuid = uuid::Uuid::new_v4();
        opf.push_str(&format!(
            "    <dc:identifier id=\"uid\">urn:uuid:{uuid}</dc:identifier>\n"
        ));
    }

    for title in &book.metadata.titles {
        opf.push_str(&format!("    <dc:title>{}</dc:title>\n", xml_escape(title)));
    }

    for lang in &book.metadata.languages {
        opf.push_str(&format!("    <dc:language>{lang}</dc:language>\n"));
    }
    if book.metadata.languages.is_empty() {
        opf.push_str("    <dc:language>en</dc:language>\n");
    }

    for creator in &book.metadata.creators {
        opf.push_str(&format!(
            "    <dc:creator>{}</dc:creator>\n",
            xml_escape(creator)
        ));
    }

    for publisher in &book.metadata.publishers {
        opf.push_str(&format!(
            "    <dc:publisher>{}</dc:publisher>\n",
            xml_escape(publisher)
        ));
    }

    if let Some(ref desc) = book.metadata.description {
        opf.push_str(&format!(
            "    <dc:description>{}</dc:description>\n",
            xml_escape(desc)
        ));
    }

    for subject in &book.metadata.subjects {
        opf.push_str(&format!(
            "    <dc:subject>{}</dc:subject>\n",
            xml_escape(subject)
        ));
    }

    if let Some(ref rights) = book.metadata.rights {
        opf.push_str(&format!(
            "    <dc:rights>{}</dc:rights>\n",
            xml_escape(rights)
        ));
    }

    for date in &book.metadata.dates {
        opf.push_str(&format!("    <dc:date>{}</dc:date>\n", xml_escape(date)));
    }

    // Modified timestamp (required for EPUB 3)
    opf.push_str("    <meta property=\"dcterms:modified\">");
    if let Some(ref modified) = book.metadata.modified {
        opf.push_str(modified);
    } else {
        opf.push_str(&format_iso8601());
    }
    opf.push_str("</meta>\n");

    // Custom metadata properties
    let mut custom_keys: Vec<&String> = book.metadata.custom.keys().collect();
    custom_keys.sort();
    for key in custom_keys {
        let value = &book.metadata.custom[key];
        opf.push_str(&format!(
            "    <meta property=\"{}\">{}</meta>\n",
            xml_escape(key),
            xml_escape(value)
        ));
    }

    opf.push_str("  </metadata>\n");

    // Manifest
    opf.push_str("  <manifest>\n");
    opf.push_str("    <item id=\"toc\" href=\"toc.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n");
    opf.push_str(
        "    <item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>\n",
    );

    for item in &book.manifest {
        let props = if let Some(ref p) = item.properties {
            format!(" properties=\"{p}\"")
        } else {
            String::new()
        };
        opf.push_str(&format!(
            "    <item id=\"{}\" href=\"{}\" media-type=\"{}\"{props}/>\n",
            xml_escape(&item.id),
            xml_escape(&item.href),
            xml_escape(&item.media_type)
        ));
    }
    opf.push_str("  </manifest>\n");

    // Spine
    opf.push_str("  <spine toc=\"ncx\">\n");
    for item in &book.spine {
        let linear = if item.linear { "" } else { " linear=\"no\"" };
        opf.push_str(&format!(
            "    <itemref idref=\"{}\"{linear}/>\n",
            item.idref
        ));
    }
    opf.push_str("  </spine>\n");

    opf.push_str("</package>\n");
    opf
}

fn generate_toc_xhtml(toc: &[NavPoint], titles: &[String]) -> String {
    let title = titles.first().map_or("Table of Contents", |s| s.as_str());
    let mut html = String::new();
    html.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\">\n");
    html.push_str("<head><title>");
    html.push_str(&xml_escape(title));
    html.push_str("</title></head>\n");
    html.push_str("<body>\n");
    html.push_str("<nav epub:type=\"toc\">\n");
    html.push_str("<h1>Table of Contents</h1>\n");
    write_nav_ol(&mut html, toc);
    html.push_str("</nav>\n");
    html.push_str("</body>\n</html>\n");
    html
}

fn write_nav_ol(html: &mut String, points: &[NavPoint]) {
    if points.is_empty() {
        return;
    }
    html.push_str("<ol>\n");
    for point in points {
        html.push_str(&format!(
            "<li><a href=\"{}\">{}</a>",
            xml_escape(&point.href),
            xml_escape(&point.label)
        ));
        if !point.children.is_empty() {
            html.push('\n');
            write_nav_ol(html, &point.children);
        }
        html.push_str("</li>\n");
    }
    html.push_str("</ol>\n");
}

fn generate_toc_ncx(toc: &[NavPoint], titles: &[String], identifiers: &[String]) -> String {
    let title = titles.first().map_or("", |s| s.as_str());
    let uid = identifiers.first().map_or("", |s| s.as_str());

    let mut ncx = String::new();
    ncx.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    ncx.push_str("<ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\">\n");
    ncx.push_str("<head>\n");
    ncx.push_str(&format!(
        "  <meta name=\"dtb:uid\" content=\"{}\"/>\n",
        xml_escape(uid)
    ));
    ncx.push_str("</head>\n");
    ncx.push_str(&format!(
        "<docTitle><text>{}</text></docTitle>\n",
        xml_escape(title)
    ));
    ncx.push_str("<navMap>\n");
    write_ncx_points(&mut ncx, toc, &mut 1);
    ncx.push_str("</navMap>\n");
    ncx.push_str("</ncx>\n");
    ncx
}

fn write_ncx_points(ncx: &mut String, points: &[NavPoint], counter: &mut usize) {
    for point in points {
        let id = *counter;
        *counter += 1;
        ncx.push_str(&format!(
            "<navPoint id=\"navpoint-{id}\" playOrder=\"{id}\">\n"
        ));
        ncx.push_str(&format!(
            "  <navLabel><text>{}</text></navLabel>\n",
            xml_escape(&point.label)
        ));
        ncx.push_str(&format!(
            "  <content src=\"{}\"/>\n",
            xml_escape(&point.href)
        ));
        write_ncx_points(ncx, &point.children, counter);
        ncx.push_str("</navPoint>\n");
    }
}

pub(crate) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epub::*;
    use std::collections::HashMap;

    fn test_book() -> EpubBook {
        let mut resources = HashMap::new();
        resources.insert(
            "OEBPS/ch1.xhtml".to_string(),
            b"<html><body><h1>Hello</h1></body></html>".to_vec(),
        );

        EpubBook {
            metadata: EpubMetadata {
                titles: vec!["Test Title".to_string()],
                creators: vec!["Test Author".to_string()],
                identifiers: vec!["urn:uuid:12345".to_string()],
                languages: vec!["en".to_string()],
                publishers: vec!["Test Publisher".to_string()],
                description: Some("A test description".to_string()),
                subjects: vec!["Fiction".to_string()],
                rights: Some("CC-BY".to_string()),
                dates: vec!["2024-01-01".to_string()],
                modified: Some("2024-01-01T00:00:00Z".to_string()),
                custom: HashMap::from([("rendition:layout".to_string(), "reflowable".to_string())]),
                ..Default::default()
            },
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
            navigation: Navigation {
                toc: vec![NavPoint {
                    label: "Chapter 1".to_string(),
                    href: "ch1.xhtml".to_string(),
                    children: Vec::new(),
                }],
                ..Default::default()
            },
            resources,
        }
    }

    #[test]
    fn test_generate_container_xml() {
        let xml = generate_container_xml();
        insta::assert_snapshot!("container_xml", xml);
    }

    #[test]
    fn test_generate_opf_full() {
        let book = test_book();
        let opf = generate_opf(&book);
        insta::assert_snapshot!("opf_full", opf);
    }

    #[test]
    fn test_generate_opf_minimal() {
        let book = EpubBook {
            metadata: EpubMetadata {
                modified: Some("2024-01-01T00:00:00Z".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let opf = generate_opf(&book);
        // Should have auto-generated UUID and default language
        assert!(opf.contains("dc:language>en</dc:language"));
        assert!(opf.contains("urn:uuid:"));
    }

    #[test]
    fn test_generate_opf_dates_and_custom() {
        let book = test_book();
        let opf = generate_opf(&book);
        assert!(
            opf.contains("<dc:date>2024-01-01</dc:date>"),
            "missing dc:date"
        );
        assert!(
            opf.contains("<meta property=\"rendition:layout\">reflowable</meta>"),
            "missing custom meta"
        );
    }

    #[test]
    fn test_generate_toc_xhtml() {
        let toc = vec![
            NavPoint {
                label: "Chapter 1".to_string(),
                href: "ch1.xhtml".to_string(),
                children: Vec::new(),
            },
            NavPoint {
                label: "Chapter 2".to_string(),
                href: "ch2.xhtml".to_string(),
                children: Vec::new(),
            },
        ];
        let titles = vec!["My Book".to_string()];
        let html = generate_toc_xhtml(&toc, &titles);
        insta::assert_snapshot!("toc_xhtml", html);
    }

    #[test]
    fn test_generate_toc_ncx() {
        let toc = vec![
            NavPoint {
                label: "Chapter 1".to_string(),
                href: "ch1.xhtml".to_string(),
                children: Vec::new(),
            },
            NavPoint {
                label: "Chapter 2".to_string(),
                href: "ch2.xhtml".to_string(),
                children: Vec::new(),
            },
        ];
        let titles = vec!["My Book".to_string()];
        let ids = vec!["urn:uuid:12345".to_string()];
        let ncx = generate_toc_ncx(&toc, &titles, &ids);
        insta::assert_snapshot!("toc_ncx", ncx);
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("<>&\""), "&lt;&gt;&amp;&quot;");
        assert_eq!(xml_escape("plain text"), "plain text");
    }

    #[test]
    fn test_format_iso8601_format() {
        let ts = format_iso8601();
        let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$").unwrap();
        assert!(re.is_match(&ts), "bad timestamp format: {ts}");
    }

    #[test]
    fn test_write_epub_roundtrip() {
        let book = test_book();
        let tmp = tempfile::TempDir::new().unwrap();
        let epub_path = tmp.path().join("test.epub");

        write_epub(&book, &epub_path).unwrap();
        assert!(epub_path.exists());

        // Read back and verify
        let book2 = crate::epub::reader::read_epub(&epub_path).unwrap();
        assert_eq!(book2.metadata.titles, vec!["Test Title"]);
        assert_eq!(book2.metadata.creators, vec!["Test Author"]);
        assert_eq!(book2.spine.len(), 1);
    }
}
