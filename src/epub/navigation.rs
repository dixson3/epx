use crate::epub::{EpubVersion, ManifestItem, NavPoint, Navigation};
use crate::error::{EpxError, Result};
use quick_xml::events::Event;
use quick_xml::Reader;

/// Try to parse navigation from manifest items and content.
/// Prefers EPUB 3 nav.xhtml, falls back to NCX.
pub fn parse_navigation(
    manifest: &[ManifestItem],
    get_content: &dyn Fn(&str) -> Option<String>,
) -> Result<Navigation> {
    // Try EPUB 3 nav.xhtml first
    if let Some(nav_item) = manifest.iter().find(|item| {
        item.properties
            .as_deref()
            .is_some_and(|p| p.contains("nav"))
    })
        && let Some(content) = get_content(&nav_item.href)
        && let Ok(nav) = parse_nav_xhtml(&content)
    {
        return Ok(Navigation {
            epub_version: EpubVersion::V3,
            ..nav
        });
    }

    // Fall back to NCX
    if let Some(ncx_item) = manifest
        .iter()
        .find(|item| item.media_type == "application/x-dtbncx+xml")
        && let Some(content) = get_content(&ncx_item.href)
    {
        let toc = parse_ncx(&content)?;
        return Ok(Navigation {
            toc,
            landmarks: Vec::new(),
            page_list: Vec::new(),
            epub_version: EpubVersion::V2,
        });
    }

    Ok(Navigation::default())
}

fn parse_nav_xhtml(html: &str) -> Result<Navigation> {
    // Simplified parsing: extract nav[epub:type="toc"] list items
    let mut toc = Vec::new();

    // Use quick-xml to parse the XHTML
    let mut reader = Reader::from_str(html);
    let mut buf = Vec::new();
    let mut in_nav_toc = false;
    let mut depth: usize = 0;
    let mut stack: Vec<Vec<NavPoint>> = vec![Vec::new()];
    let mut current_href = String::new();
    let mut current_label = String::new();
    let mut in_a = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if local == "nav" {
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        if key.ends_with("type") {
                            let val = String::from_utf8_lossy(&attr.value);
                            if val == "toc" {
                                in_nav_toc = true;
                            }
                        }
                    }
                } else if in_nav_toc {
                    if local == "ol" {
                        depth += 1;
                        stack.push(Vec::new());
                    } else if local == "a" {
                        in_a = true;
                        current_label.clear();
                        current_href.clear();
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"href" {
                                current_href =
                                    String::from_utf8_lossy(&attr.value).into_owned();
                            }
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_a && in_nav_toc {
                    current_label.push_str(&e.unescape().unwrap_or_default());
                }
            }
            Ok(Event::End(ref e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if local == "nav" && in_nav_toc {
                    in_nav_toc = false;
                } else if in_nav_toc {
                    if local == "a" {
                        in_a = false;
                        if let Some(current) = stack.last_mut() {
                            current.push(NavPoint {
                                label: current_label.trim().to_string(),
                                href: current_href.clone(),
                                children: Vec::new(),
                            });
                        }
                    } else if local == "ol" {
                        depth = depth.saturating_sub(1);
                        let children = stack.pop().unwrap_or_default();
                        if let Some(parent_list) = stack.last_mut() {
                            if let Some(parent) = parent_list.last_mut() {
                                parent.children = children;
                            } else {
                                // Top level
                                stack.last_mut().unwrap().extend(children);
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(EpxError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    if let Some(items) = stack.into_iter().next() {
        toc = items;
    }

    Ok(Navigation {
        toc,
        landmarks: Vec::new(),
        page_list: Vec::new(),
        epub_version: EpubVersion::V3,
    })
}

fn parse_ncx(xml: &str) -> Result<Vec<NavPoint>> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut nav_points = Vec::new();
    let mut stack: Vec<NavPoint> = Vec::new();
    let mut in_text = false;
    let mut current_label = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if local == "navPoint" {
                    stack.push(NavPoint {
                        label: String::new(),
                        href: String::new(),
                        children: Vec::new(),
                    });
                } else if local == "text" {
                    in_text = true;
                    current_label.clear();
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if local == "content"
                    && let Some(current) = stack.last_mut()
                {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"src" {
                            current.href =
                                String::from_utf8_lossy(&attr.value).into_owned();
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text {
                    current_label.push_str(&e.unescape().unwrap_or_default());
                }
            }
            Ok(Event::End(ref e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if local == "text" {
                    in_text = false;
                    if let Some(current) = stack.last_mut() {
                        current.label = current_label.trim().to_string();
                    }
                } else if local == "navPoint" {
                    let point = stack.pop().unwrap();
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(point);
                    } else {
                        nav_points.push(point);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(EpxError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(nav_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_nav_xhtml_basic() {
        let nav_html = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Nav</title></head>
<body>
<nav epub:type="toc">
<ol>
<li><a href="ch1.xhtml">Chapter 1</a></li>
<li><a href="ch2.xhtml">Chapter 2</a></li>
</ol>
</nav>
</body>
</html>"#;

        let manifest = vec![ManifestItem {
            id: "nav".to_string(),
            href: "toc.xhtml".to_string(),
            media_type: "application/xhtml+xml".to_string(),
            properties: Some("nav".to_string()),
        }];

        let nav = parse_navigation(&manifest, &|href| {
            if href == "toc.xhtml" {
                Some(nav_html.to_string())
            } else {
                None
            }
        }).unwrap();

        assert_eq!(nav.toc.len(), 2);
        assert_eq!(nav.toc[0].label, "Chapter 1");
        assert_eq!(nav.toc[0].href, "ch1.xhtml");
        assert_eq!(nav.toc[1].label, "Chapter 2");
    }

    #[test]
    fn parse_ncx_basic() {
        let ncx_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
<navMap>
<navPoint id="np1" playOrder="1">
  <navLabel><text>Chapter 1</text></navLabel>
  <content src="ch1.xhtml"/>
</navPoint>
<navPoint id="np2" playOrder="2">
  <navLabel><text>Chapter 2</text></navLabel>
  <content src="ch2.xhtml"/>
</navPoint>
</navMap>
</ncx>"#;

        let manifest = vec![ManifestItem {
            id: "ncx".to_string(),
            href: "toc.ncx".to_string(),
            media_type: "application/x-dtbncx+xml".to_string(),
            properties: None,
        }];

        let nav = parse_navigation(&manifest, &|href| {
            if href == "toc.ncx" {
                Some(ncx_xml.to_string())
            } else {
                None
            }
        }).unwrap();

        assert_eq!(nav.toc.len(), 2);
        assert_eq!(nav.toc[0].label, "Chapter 1");
        assert!(matches!(nav.epub_version, EpubVersion::V2));
    }

    #[test]
    fn parse_nav_both_missing() {
        let manifest = vec![ManifestItem {
            id: "ch1".to_string(),
            href: "ch1.xhtml".to_string(),
            media_type: "application/xhtml+xml".to_string(),
            properties: None,
        }];

        let nav = parse_navigation(&manifest, &|_| None).unwrap();
        assert!(nav.toc.is_empty());
    }

    #[test]
    fn parse_nav_nested() {
        let nav_html = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<body>
<nav epub:type="toc">
<ol>
<li><a href="part1.xhtml">Part 1</a>
<ol>
<li><a href="ch1.xhtml">Chapter 1</a></li>
<li><a href="ch2.xhtml">Chapter 2</a></li>
</ol>
</li>
</ol>
</nav>
</body>
</html>"#;

        let manifest = vec![ManifestItem {
            id: "nav".to_string(),
            href: "nav.xhtml".to_string(),
            media_type: "application/xhtml+xml".to_string(),
            properties: Some("nav".to_string()),
        }];

        let nav = parse_navigation(&manifest, &|href| {
            if href == "nav.xhtml" {
                Some(nav_html.to_string())
            } else {
                None
            }
        }).unwrap();

        assert_eq!(nav.toc.len(), 1);
        assert_eq!(nav.toc[0].label, "Part 1");
        assert_eq!(nav.toc[0].children.len(), 2);
        assert_eq!(nav.toc[0].children[0].label, "Chapter 1");
    }

    #[test]
    fn parse_nav_fallback_to_ncx() {
        let ncx_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
<navMap>
<navPoint id="np1" playOrder="1">
  <navLabel><text>From NCX</text></navLabel>
  <content src="ch1.xhtml"/>
</navPoint>
</navMap>
</ncx>"#;

        let manifest = vec![
            ManifestItem {
                id: "nav".to_string(),
                href: "nav.xhtml".to_string(),
                media_type: "application/xhtml+xml".to_string(),
                properties: Some("nav".to_string()),
            },
            ManifestItem {
                id: "ncx".to_string(),
                href: "toc.ncx".to_string(),
                media_type: "application/x-dtbncx+xml".to_string(),
                properties: None,
            },
        ];

        // nav.xhtml content is invalid/missing, so falls back to NCX
        let nav = parse_navigation(&manifest, &|href| {
            if href == "toc.ncx" {
                Some(ncx_xml.to_string())
            } else {
                None
            }
        }).unwrap();

        assert_eq!(nav.toc[0].label, "From NCX");
    }
}
