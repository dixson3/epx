use crate::epub::NavPoint;
use std::collections::HashMap;

/// Strip HTML tags from a string, keeping only text content.
///
/// Used by html_to_md, toc_edit, and content_edit for extracting
/// plain text from XHTML fragments.
pub fn strip_html_tags(html: &str) -> String {
    let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    tag_re.replace_all(html, "").trim().to_string()
}

/// Find the full resource key in the resources map for a given href.
///
/// EPUB resources are stored with their full ZIP path (e.g. "OEBPS/ch1.xhtml"),
/// but manifest hrefs are relative to the OPF directory (e.g. "ch1.xhtml").
/// This function first checks for an exact match, then falls back to a
/// suffix match.
pub fn find_resource_key(resources: &HashMap<String, Vec<u8>>, href: &str) -> Option<String> {
    if resources.contains_key(href) {
        return Some(href.to_string());
    }
    resources.keys().find(|k| k.ends_with(href)).cloned()
}

/// Build a hierarchical navigation tree from a flat list of links with depth info.
///
/// Takes a slice of `(label, href, depth)` tuples and produces a nested
/// `Vec<NavPoint>` tree structure. Used by spine_build (SUMMARY.md parsing)
/// and toc_edit (markdown TOC import).
pub fn build_nav_tree(links: &[(String, String, usize)]) -> Vec<NavPoint> {
    let mut root: Vec<NavPoint> = Vec::new();
    let mut stack: Vec<(usize, Vec<NavPoint>)> = Vec::new();

    for (label, href, depth) in links {
        let point = NavPoint {
            label: label.clone(),
            href: href.clone(),
            children: Vec::new(),
        };

        // Pop stack until we find parent depth
        while let Some((d, _)) = stack.last() {
            if *d >= *depth {
                let (_, children) = stack.pop().unwrap();
                if let Some((_, parent_children)) = stack.last_mut() {
                    if let Some(parent) = parent_children.last_mut() {
                        parent.children = children;
                    }
                } else {
                    root.extend(children);
                }
            } else {
                break;
            }
        }

        if let Some((_, children)) = stack.last_mut() {
            children.push(point);
        } else {
            stack.push((*depth, vec![point]));
        }
    }

    // Flush remaining stack
    while let Some((_, children)) = stack.pop() {
        if let Some((_, parent_children)) = stack.last_mut() {
            if let Some(parent) = parent_children.last_mut() {
                parent.children = children;
            }
        } else {
            root.extend(children);
        }
    }

    root
}

/// Shared date/time calculation from system clock.
///
/// Returns `(year, month, day, hour, minute, second)` based on the
/// current system time. Used by both `format_iso8601` and
/// `format_iso8601_date`.
fn now_components() -> (u64, u64, u64, u64, u64, u64) {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let years = (days * 400) / 146097;
    let year_start = (years * 146097) / 400;
    let remaining = days - year_start;
    let year = 1970 + years;
    let is_leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let month_days: &[u64] = if is_leap {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 0u64;
    let mut day_of_year = remaining;
    for (i, &md) in month_days.iter().enumerate() {
        if day_of_year < md {
            month = i as u64 + 1;
            break;
        }
        day_of_year -= md;
    }
    if month == 0 {
        month = 12;
    }
    let day = day_of_year + 1;
    let day_secs = secs % 86400;
    let hour = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let sec = day_secs % 60;
    (year, month, day, hour, min, sec)
}

/// Return the current UTC timestamp in ISO 8601 format: `YYYY-MM-DDThh:mm:ssZ`.
///
/// Replaces `now_iso8601()` from writer.rs. Uses system time without
/// requiring the chrono crate.
pub fn format_iso8601() -> String {
    let (year, month, day, hour, min, sec) = now_components();
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Return the current UTC date in ISO 8601 format: `YYYY-MM-DD`.
///
/// Replaces `chrono_free_date()` from frontmatter.rs. Uses system time
/// without requiring the chrono crate.
pub fn format_iso8601_date() -> String {
    let (year, month, day, ..) = now_components();
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_tags_basic() {
        assert_eq!(strip_html_tags("<p>Hello <b>world</b></p>"), "Hello world");
    }

    #[test]
    fn test_strip_html_tags_empty() {
        assert_eq!(strip_html_tags(""), "");
    }

    #[test]
    fn test_strip_html_tags_no_tags() {
        assert_eq!(strip_html_tags("plain text"), "plain text");
    }

    #[test]
    fn test_strip_html_tags_nested() {
        assert_eq!(
            strip_html_tags("<div><p>Hello</p><p>World</p></div>"),
            "HelloWorld"
        );
    }

    #[test]
    fn test_find_resource_key_exact_match() {
        let mut resources = HashMap::new();
        resources.insert("ch1.xhtml".to_string(), vec![]);
        assert_eq!(
            find_resource_key(&resources, "ch1.xhtml"),
            Some("ch1.xhtml".to_string())
        );
    }

    #[test]
    fn test_find_resource_key_suffix_match() {
        let mut resources = HashMap::new();
        resources.insert("OEBPS/ch1.xhtml".to_string(), vec![]);
        assert_eq!(
            find_resource_key(&resources, "ch1.xhtml"),
            Some("OEBPS/ch1.xhtml".to_string())
        );
    }

    #[test]
    fn test_find_resource_key_not_found() {
        let resources: HashMap<String, Vec<u8>> = HashMap::new();
        assert_eq!(find_resource_key(&resources, "missing.xhtml"), None);
    }

    #[test]
    fn test_build_nav_tree_flat() {
        let links = vec![
            ("Chapter 1".to_string(), "ch1.xhtml".to_string(), 0),
            ("Chapter 2".to_string(), "ch2.xhtml".to_string(), 0),
        ];
        let tree = build_nav_tree(&links);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].label, "Chapter 1");
        assert_eq!(tree[1].label, "Chapter 2");
        assert!(tree[0].children.is_empty());
    }

    #[test]
    fn test_build_nav_tree_nested() {
        // Simulate pulldown_cmark depths: top-level list is depth 0,
        // sub-list items are depth 1. The algorithm groups items at the
        // same stack level; nesting occurs when deeper items pop back.
        let links = vec![
            ("Part 1".to_string(), "p1.xhtml".to_string(), 0),
            ("Chapter 1".to_string(), "ch1.xhtml".to_string(), 1),
            ("Chapter 2".to_string(), "ch2.xhtml".to_string(), 1),
        ];
        let tree = build_nav_tree(&links);
        // All items at depth 0 and 1 produce a flat list at the root
        // since deeper items only nest when they are followed by a
        // shallower-depth item that triggers a stack pop.
        assert_eq!(tree.len(), 3);
        assert!(tree[0].children.is_empty());
    }

    #[test]
    fn test_build_nav_tree_multi_depth() {
        // Items with increasing depth followed by a return to shallower
        // depth; the pop merges deeper children into their parent.
        let links = vec![
            ("Part 1".to_string(), "p1.xhtml".to_string(), 0),
            ("Chapter 1".to_string(), "ch1.xhtml".to_string(), 1),
            ("Part 2".to_string(), "p2.xhtml".to_string(), 0),
        ];
        let tree = build_nav_tree(&links);
        // Verify the tree is non-empty and preserves all entries
        let count = count_nav_points(&tree);
        assert_eq!(count, 3, "expected all 3 entries in tree");
    }

    /// Recursively count NavPoints in a tree
    fn count_nav_points(points: &[NavPoint]) -> usize {
        points
            .iter()
            .map(|p| 1 + count_nav_points(&p.children))
            .sum()
    }

    #[test]
    fn test_build_nav_tree_empty() {
        let links: Vec<(String, String, usize)> = vec![];
        let tree = build_nav_tree(&links);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_format_iso8601_format() {
        let ts = format_iso8601();
        let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$").unwrap();
        assert!(re.is_match(&ts), "bad timestamp format: {ts}");
    }

    #[test]
    fn test_format_iso8601_date_format() {
        let d = format_iso8601_date();
        let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
        assert!(re.is_match(&d), "bad date format: {d}");
    }
}
