use crate::epub::{EpubBook, NavPoint};
use crate::util::{build_nav_tree, find_resource_key, strip_html_tags};

/// Reorder a spine item from one position to another
pub fn reorder_spine(book: &mut EpubBook, from: usize, to: usize) -> anyhow::Result<()> {
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

/// Set spine order from a list of idrefs
pub fn set_spine_order(book: &mut EpubBook, idrefs: &[String]) -> anyhow::Result<()> {
    let mut new_spine = Vec::new();
    for idref in idrefs {
        let item = book
            .spine
            .iter()
            .find(|s| s.idref == *idref)
            .ok_or_else(|| anyhow::anyhow!("spine item not found: {idref}"))?
            .clone();
        new_spine.push(item);
    }
    book.spine = new_spine;
    Ok(())
}

/// Set TOC from a markdown TOC file (same format as SUMMARY.md)
pub fn set_toc_from_markdown(book: &mut EpubBook, toc_content: &str) -> anyhow::Result<()> {
    use pulldown_cmark::{Event, Parser, Tag, TagEnd};

    let parser = Parser::new(toc_content);
    let mut links: Vec<(String, String, usize)> = Vec::new();
    let mut current_label = String::new();
    let mut current_href = String::new();
    let mut in_link = false;
    let mut list_depth: usize = 0;

    for event in parser {
        match event {
            Event::Start(Tag::List(_)) => list_depth += 1,
            Event::End(TagEnd::List(_)) => list_depth = list_depth.saturating_sub(1),
            Event::Start(Tag::Link { dest_url, .. }) => {
                in_link = true;
                current_href = dest_url.to_string();
                current_label.clear();
            }
            Event::End(TagEnd::Link) => {
                in_link = false;
                links.push((
                    current_label.trim().to_string(),
                    current_href.clone(),
                    list_depth.saturating_sub(1),
                ));
            }
            Event::Text(text) => {
                if in_link {
                    current_label.push_str(&text);
                }
            }
            _ => {}
        }
    }

    book.navigation.toc = build_nav_tree(&links);
    Ok(())
}

/// Generate TOC from XHTML headings in spine order
pub fn generate_toc(book: &mut EpubBook, max_depth: Option<usize>) -> anyhow::Result<()> {
    let max_depth = max_depth.unwrap_or(3);
    let mut toc = Vec::new();

    let heading_re = regex::Regex::new(r"<h([1-6])[^>]*>(.*?)</h[1-6]>")?;

    for spine_item in &book.spine {
        let manifest_item = book.manifest.iter().find(|m| m.id == spine_item.idref);

        let Some(manifest_item) = manifest_item else {
            continue;
        };
        if !manifest_item.media_type.contains("html") {
            continue;
        }

        let href = &manifest_item.href;
        let full_path = find_resource_key(&book.resources, href);
        let Some(full_path) = full_path else { continue };

        let xhtml = match String::from_utf8(book.resources[&full_path].clone()) {
            Ok(s) => s,
            Err(_) => continue,
        };

        for cap in heading_re.captures_iter(&xhtml) {
            let level: usize = cap[1].parse().unwrap_or(1);
            if level > max_depth {
                continue;
            }
            let text = strip_html_tags(&cap[2]);
            if !text.is_empty() {
                toc.push(NavPoint {
                    label: text,
                    href: href.clone(),
                    children: Vec::new(),
                });
            }
        }
    }

    book.navigation.toc = toc;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epub::*;
    use std::collections::HashMap;

    fn test_book() -> EpubBook {
        let xhtml = b"<?xml version=\"1.0\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>Ch1</title></head><body><h1>Chapter 1</h1><p>Hello world.</p></body></html>";
        let xhtml2 = b"<?xml version=\"1.0\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>Ch2</title></head><body><h2>Section A</h2><p>Goodbye.</p></body></html>";

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
                ManifestItem {
                    id: "ch1".to_string(),
                    href: "ch1.xhtml".to_string(),
                    media_type: "application/xhtml+xml".to_string(),
                    properties: None,
                },
                ManifestItem {
                    id: "ch2".to_string(),
                    href: "ch2.xhtml".to_string(),
                    media_type: "application/xhtml+xml".to_string(),
                    properties: None,
                },
            ],
            spine: vec![
                SpineItem {
                    idref: "ch1".to_string(),
                    linear: true,
                    properties: None,
                },
                SpineItem {
                    idref: "ch2".to_string(),
                    linear: true,
                    properties: None,
                },
            ],
            navigation: Navigation {
                toc: vec![
                    NavPoint {
                        label: "Chapter 1".to_string(),
                        href: "ch1.xhtml".to_string(),
                        children: vec![],
                    },
                    NavPoint {
                        label: "Chapter 2".to_string(),
                        href: "ch2.xhtml".to_string(),
                        children: vec![],
                    },
                ],
                ..Default::default()
            },
            resources,
        }
    }

    #[test]
    fn test_reorder_spine_valid() {
        let mut book = test_book();
        reorder_spine(&mut book, 0, 1).unwrap();
        assert_eq!(book.spine[0].idref, "ch2");
        assert_eq!(book.spine[1].idref, "ch1");
    }

    #[test]
    fn test_reorder_spine_out_of_bounds() {
        let mut book = test_book();
        assert!(reorder_spine(&mut book, 10, 0).is_err());
    }

    #[test]
    fn test_set_spine_order_valid() {
        let mut book = test_book();
        set_spine_order(&mut book, &["ch2".to_string(), "ch1".to_string()]).unwrap();
        assert_eq!(book.spine[0].idref, "ch2");
        assert_eq!(book.spine[1].idref, "ch1");
    }

    #[test]
    fn test_set_spine_order_missing_idref() {
        let mut book = test_book();
        assert!(set_spine_order(&mut book, &["nonexistent".to_string()]).is_err());
    }

    #[test]
    fn test_set_toc_from_markdown() {
        let mut book = test_book();
        let toc_md = "- [New Ch 1](ch1.xhtml)\n- [New Ch 2](ch2.xhtml)\n";
        set_toc_from_markdown(&mut book, toc_md).unwrap();
        assert_eq!(book.navigation.toc.len(), 2);
        assert_eq!(book.navigation.toc[0].label, "New Ch 1");
    }

    #[test]
    fn test_generate_toc_from_headings() {
        let mut book = test_book();
        generate_toc(&mut book, None).unwrap();
        assert!(!book.navigation.toc.is_empty());
        assert_eq!(book.navigation.toc[0].label, "Chapter 1");
    }

    #[test]
    fn test_generate_toc_max_depth() {
        let mut book = test_book();
        generate_toc(&mut book, Some(1)).unwrap();
        // Only h1 should be included, not h2
        for entry in &book.navigation.toc {
            // h2 "Section A" from ch2 should be excluded at max_depth=1
            assert_ne!(entry.label, "Section A");
        }
    }
}
