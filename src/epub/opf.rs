use crate::epub::{EpubMetadata, EpubVersion, ManifestItem, SpineItem};
use crate::error::{EpxError, Result};
use quick_xml::Reader;
use quick_xml::events::Event;

#[allow(dead_code)]
pub struct OpfData {
    pub metadata: EpubMetadata,
    pub manifest: Vec<ManifestItem>,
    pub spine: Vec<SpineItem>,
    pub version: EpubVersion,
}

pub fn parse_opf(xml: &str) -> Result<OpfData> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut metadata = EpubMetadata::default();
    let mut manifest = Vec::new();
    let mut spine = Vec::new();
    let mut version = EpubVersion::V3;

    let mut in_metadata = false;
    let mut current_element = String::new();
    let mut current_text = String::new();
    let mut current_meta_property = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                if local == "package" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"version" {
                            let v = String::from_utf8_lossy(&attr.value);
                            version = if v.starts_with('2') {
                                EpubVersion::V2
                            } else {
                                EpubVersion::V3
                            };
                        }
                    }
                } else if local == "metadata" {
                    in_metadata = true;
                } else if in_metadata {
                    current_element = local.clone();
                    current_text.clear();
                    current_meta_property.clear();
                    if local == "meta" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"property" {
                                current_meta_property =
                                    String::from_utf8_lossy(&attr.value).into_owned();
                            }
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                if local == "metadata" {
                    in_metadata = false;
                } else if in_metadata && !current_text.is_empty() {
                    let text = current_text.trim().to_string();
                    match current_element.as_str() {
                        "identifier" => metadata.identifiers.push(text),
                        "title" => metadata.titles.push(text),
                        "language" => metadata.languages.push(text),
                        "creator" => metadata.creators.push(text),
                        "publisher" => metadata.publishers.push(text),
                        "date" => metadata.dates.push(text),
                        "description" => metadata.description = Some(text),
                        "subject" => metadata.subjects.push(text),
                        "rights" => metadata.rights = Some(text),
                        "meta" if !current_meta_property.is_empty() => {
                            match current_meta_property.as_str() {
                                "dcterms:modified" => metadata.modified = Some(text),
                                _ => {
                                    metadata.custom.insert(current_meta_property.clone(), text);
                                }
                            }
                        }
                        _ => {}
                    }
                    current_text.clear();
                    current_element.clear();
                    current_meta_property.clear();
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_metadata {
                    current_text.push_str(&e.unescape().unwrap_or_default());
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                if local == "item" {
                    let mut item = ManifestItem {
                        id: String::new(),
                        href: String::new(),
                        media_type: String::new(),
                        properties: None,
                    };
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"id" => item.id = String::from_utf8_lossy(&attr.value).into_owned(),
                            b"href" => {
                                item.href = String::from_utf8_lossy(&attr.value).into_owned()
                            }
                            b"media-type" => {
                                item.media_type = String::from_utf8_lossy(&attr.value).into_owned()
                            }
                            b"properties" => {
                                item.properties =
                                    Some(String::from_utf8_lossy(&attr.value).into_owned())
                            }
                            _ => {}
                        }
                    }
                    manifest.push(item);
                } else if local == "itemref" {
                    let mut spine_item = SpineItem {
                        idref: String::new(),
                        linear: true,
                        properties: None,
                    };
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"idref" => {
                                spine_item.idref = String::from_utf8_lossy(&attr.value).into_owned()
                            }
                            b"linear" => {
                                spine_item.linear = String::from_utf8_lossy(&attr.value) != "no"
                            }
                            b"properties" => {
                                spine_item.properties =
                                    Some(String::from_utf8_lossy(&attr.value).into_owned())
                            }
                            _ => {}
                        }
                    }
                    spine.push(spine_item);
                } else if in_metadata && local == "meta" {
                    // Handle EPUB 2 <meta name="cover" content="cover-image"/>
                    let mut name = String::new();
                    let mut content = String::new();
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"name" => name = String::from_utf8_lossy(&attr.value).into_owned(),
                            b"content" => {
                                content = String::from_utf8_lossy(&attr.value).into_owned()
                            }
                            b"property" => name = String::from_utf8_lossy(&attr.value).into_owned(),
                            _ => {}
                        }
                    }
                    if name == "cover" {
                        metadata.cover_id = Some(content);
                    } else if name == "dcterms:modified" {
                        // EPUB 3 modified timestamp stored in content attr won't be here,
                        // it uses text content — handled in Start/End events
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(EpxError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(OpfData {
        metadata,
        manifest,
        spine,
        version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_opf(
        version: &str,
        metadata_extra: &str,
        manifest_extra: &str,
        spine_extra: &str,
    ) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="{version}" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="uid">urn:uuid:test</dc:identifier>
    <dc:title>Test Book</dc:title>
    <dc:language>en</dc:language>
    {metadata_extra}
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    {manifest_extra}
  </manifest>
  <spine>
    <itemref idref="ch1"/>
    {spine_extra}
  </spine>
</package>"#
        )
    }

    #[test]
    fn parse_opf_epub3_version() {
        let opf = minimal_opf("3.0", "", "", "");
        let data = parse_opf(&opf).unwrap();
        assert!(matches!(data.version, EpubVersion::V3));
    }

    #[test]
    fn parse_opf_epub2_version() {
        let opf = minimal_opf("2.0", "", "", "");
        let data = parse_opf(&opf).unwrap();
        assert!(matches!(data.version, EpubVersion::V2));
    }

    #[test]
    fn parse_opf_metadata_titles() {
        let opf = minimal_opf("3.0", "", "", "");
        let data = parse_opf(&opf).unwrap();
        assert_eq!(data.metadata.titles, vec!["Test Book"]);
    }

    #[test]
    fn parse_opf_metadata_creators() {
        let opf = minimal_opf("3.0", "<dc:creator>Jane Doe</dc:creator>", "", "");
        let data = parse_opf(&opf).unwrap();
        assert_eq!(data.metadata.creators, vec!["Jane Doe"]);
    }

    #[test]
    fn parse_opf_metadata_identifiers() {
        let opf = minimal_opf("3.0", "", "", "");
        let data = parse_opf(&opf).unwrap();
        assert_eq!(data.metadata.identifiers, vec!["urn:uuid:test"]);
    }

    #[test]
    fn parse_opf_metadata_languages() {
        let opf = minimal_opf("3.0", "", "", "");
        let data = parse_opf(&opf).unwrap();
        assert_eq!(data.metadata.languages, vec!["en"]);
    }

    #[test]
    fn parse_opf_spine_linear() {
        let opf = minimal_opf("3.0", "", "", r#"<itemref idref="ch1" linear="no"/>"#);
        let data = parse_opf(&opf).unwrap();
        // Second spine item should have linear=false
        assert!(!data.spine[1].linear);
    }

    #[test]
    fn parse_opf_cover_image_meta() {
        let opf = minimal_opf(
            "2.0",
            r#"<meta name="cover" content="cover-image"/>"#,
            "",
            "",
        );
        let data = parse_opf(&opf).unwrap();
        assert_eq!(data.metadata.cover_id, Some("cover-image".to_string()));
    }

    #[test]
    fn parse_opf_manifest_properties() {
        let opf = minimal_opf(
            "3.0",
            "",
            r#"<item id="nav" href="toc.xhtml" media-type="application/xhtml+xml" properties="nav"/>"#,
            "",
        );
        let data = parse_opf(&opf).unwrap();
        let nav_item = data.manifest.iter().find(|m| m.id == "nav").unwrap();
        assert_eq!(nav_item.properties, Some("nav".to_string()));
    }

    #[test]
    fn parse_opf_modified_timestamp() {
        let opf = minimal_opf(
            "3.0",
            r#"<meta property="dcterms:modified">2024-06-15T10:30:00Z</meta>"#,
            "",
            "",
        );
        let data = parse_opf(&opf).unwrap();
        assert_eq!(
            data.metadata.modified,
            Some("2024-06-15T10:30:00Z".to_string())
        );
    }

    #[test]
    fn parse_opf_custom_meta_property() {
        let opf = minimal_opf(
            "3.0",
            r#"<meta property="rendition:layout">pre-paginated</meta>"#,
            "",
            "",
        );
        let data = parse_opf(&opf).unwrap();
        assert_eq!(
            data.metadata.custom.get("rendition:layout"),
            Some(&"pre-paginated".to_string())
        );
    }

    #[test]
    fn parse_opf_empty_metadata() {
        let xml = r#"<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
  </metadata>
  <manifest/>
  <spine/>
</package>"#;
        let data = parse_opf(xml).unwrap();
        assert!(data.metadata.titles.is_empty());
        assert!(data.manifest.is_empty());
        assert!(data.spine.is_empty());
    }
}
