use crate::epub::Navigation;
use crate::util::build_nav_tree;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::path::Path;

/// Parse SUMMARY.md to extract chapter ordering and navigation
pub fn parse_summary(dir: &Path) -> anyhow::Result<(Vec<String>, Navigation)> {
    let summary_path = dir.join("SUMMARY.md");
    let content = std::fs::read_to_string(&summary_path)?;

    let parser = Parser::new(&content);

    let mut links: Vec<(String, String, usize)> = Vec::new(); // (label, href, depth)
    let mut current_label = String::new();
    let mut current_href = String::new();
    let mut in_link = false;
    let mut list_depth: usize = 0;

    for event in parser {
        match event {
            Event::Start(Tag::List(_)) => {
                list_depth += 1;
            }
            Event::End(TagEnd::List(_)) => {
                list_depth = list_depth.saturating_sub(1);
            }
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

    // Build spine order from chapter links
    let chapter_order: Vec<String> = links
        .iter()
        .map(|(_, href, _)| {
            // Strip "chapters/" prefix to get filename
            href.strip_prefix("chapters/").unwrap_or(href).to_string()
        })
        .collect();

    // Build navigation tree
    let nav_points = build_nav_tree(&links);
    let nav = Navigation {
        toc: nav_points,
        ..Default::default()
    };

    Ok((chapter_order, nav))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_summary_flat() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("SUMMARY.md"), r#"# Summary

- [Chapter 1](chapters/01-intro.md)
- [Chapter 2](chapters/02-main.md)
"#).unwrap();

        let (order, _nav) = parse_summary(tmp.path()).unwrap();
        assert_eq!(order, vec!["01-intro.md", "02-main.md"]);
    }

    #[test]
    fn test_parse_summary_nested() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("SUMMARY.md"), r#"# Summary

- [Part 1](chapters/part1.md)
  - [Chapter 1](chapters/ch1.md)
  - [Chapter 2](chapters/ch2.md)
"#).unwrap();

        let (order, nav) = parse_summary(tmp.path()).unwrap();
        assert_eq!(order.len(), 3);
        // The nav tree should have Part 1 with 2 children
        assert!(!nav.toc.is_empty());
    }

    #[test]
    fn test_parse_summary_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(parse_summary(tmp.path()).is_err());
    }
}
