use crate::epub::EpubBook;
use crate::util::{find_resource_key, strip_html_tags};

/// Search result with chapter context
pub struct SearchMatch {
    pub chapter_id: String,
    pub chapter_href: String,
    pub line_number: usize,
    pub context: String,
}

/// Search for a pattern in EPUB content
pub fn search(
    book: &EpubBook,
    pattern: &str,
    chapter_filter: Option<&str>,
    use_regex: bool,
) -> anyhow::Result<Vec<SearchMatch>> {
    let re = if use_regex {
        regex::Regex::new(pattern)?
    } else {
        regex::Regex::new(&regex::escape(pattern))?
    };

    let mut matches = Vec::new();

    for spine_item in &book.spine {
        if let Some(filter) = chapter_filter
            && spine_item.idref != filter
        {
            if let Ok(idx) = filter.parse::<usize>() {
                if book.spine.iter().position(|s| s.idref == spine_item.idref) != Some(idx) {
                    continue;
                }
            } else {
                continue;
            }
        }

        let manifest_item = book.manifest.iter().find(|m| m.id == spine_item.idref);
        let Some(manifest_item) = manifest_item else { continue };

        if !manifest_item.media_type.contains("html") {
            continue;
        }

        let full_path = find_resource_key(&book.resources, &manifest_item.href);
        let Some(full_path) = full_path else { continue };

        let xhtml = match String::from_utf8(book.resources[&full_path].clone()) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Extract text from XHTML for searching
        let text = strip_html_tags(&xhtml);

        for (line_number, line) in text.lines().enumerate() {
            if re.is_match(line) {
                matches.push(SearchMatch {
                    chapter_id: spine_item.idref.clone(),
                    chapter_href: manifest_item.href.clone(),
                    line_number: line_number + 1,
                    context: line.trim().to_string(),
                });
            }
        }
    }

    Ok(matches)
}

/// Replace text in EPUB content, returns number of replacements made
pub fn replace(
    book: &mut EpubBook,
    pattern: &str,
    replacement: &str,
    chapter_filter: Option<&str>,
    use_regex: bool,
) -> anyhow::Result<usize> {
    let re = if use_regex {
        regex::Regex::new(pattern)?
    } else {
        regex::Regex::new(&regex::escape(pattern))?
    };

    let mut total_replacements = 0;

    let spine_items: Vec<_> = book.spine.clone();
    for spine_item in &spine_items {
        if let Some(filter) = chapter_filter
            && spine_item.idref != filter
        {
            if let Ok(idx) = filter.parse::<usize>() {
                if book.spine.iter().position(|s| s.idref == spine_item.idref) != Some(idx) {
                    continue;
                }
            } else {
                continue;
            }
        }

        let manifest_item = book.manifest.iter().find(|m| m.id == spine_item.idref);
        let Some(manifest_item) = manifest_item else { continue };

        if !manifest_item.media_type.contains("html") {
            continue;
        }

        let href = manifest_item.href.clone();
        let full_path = find_resource_key(&book.resources, &href);
        let Some(full_path) = full_path else { continue };

        let xhtml = match String::from_utf8(book.resources[&full_path].clone()) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Replace in text nodes only (between > and <)
        let result = replace_in_text_nodes(&xhtml, &re, replacement);
        let count = count_matches(&xhtml, &re);
        total_replacements += count;

        book.resources.insert(full_path, result.into_bytes());
    }

    Ok(total_replacements)
}

/// List headings in the EPUB
pub fn list_headings(book: &EpubBook) -> anyhow::Result<Vec<(String, usize, String)>> {
    let heading_re = regex::Regex::new(r"<h([1-6])[^>]*>(.*?)</h[1-6]>")?;
    let mut headings = Vec::new();

    for spine_item in &book.spine {
        let manifest_item = book.manifest.iter().find(|m| m.id == spine_item.idref);
        let Some(manifest_item) = manifest_item else { continue };

        if !manifest_item.media_type.contains("html") {
            continue;
        }

        let full_path = find_resource_key(&book.resources, &manifest_item.href);
        let Some(full_path) = full_path else { continue };

        let xhtml = match String::from_utf8(book.resources[&full_path].clone()) {
            Ok(s) => s,
            Err(_) => continue,
        };

        for cap in heading_re.captures_iter(&xhtml) {
            let level: usize = cap[1].parse().unwrap_or(1);
            let text = strip_html_tags(&cap[2]);
            headings.push((manifest_item.href.clone(), level, text));
        }
    }

    Ok(headings)
}

/// Restructure headings according to a mapping (e.g., "h2->h1,h3->h2")
pub fn restructure_headings(book: &mut EpubBook, mapping: &str) -> anyhow::Result<usize> {
    let mut level_map = std::collections::HashMap::new();
    for pair in mapping.split(',') {
        let parts: Vec<&str> = pair.split("->").collect();
        if parts.len() != 2 {
            anyhow::bail!("invalid mapping format: {pair}");
        }
        let from: usize = parts[0].trim().trim_start_matches('h').parse()?;
        let to: usize = parts[1].trim().trim_start_matches('h').parse()?;
        if !(1..=6).contains(&from) || !(1..=6).contains(&to) {
            anyhow::bail!("heading levels must be 1-6");
        }
        level_map.insert(from, to);
    }

    let mut total = 0;
    let keys: Vec<String> = book.resources.keys().cloned().collect();

    for key in keys {
        let xhtml = match String::from_utf8(book.resources[&key].clone()) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let mut modified = xhtml.clone();
        for (&from, &to) in &level_map {
            let open_re = regex::Regex::new(&format!(r"<h{from}([^>]*)>"))?;
            let close_re = regex::Regex::new(&format!(r"</h{from}>"))?;
            let count = open_re.find_iter(&modified).count();
            total += count;
            modified = open_re.replace_all(&modified, format!("<h{to}$1>").as_str()).to_string();
            modified = close_re.replace_all(&modified, format!("</h{to}>").as_str()).to_string();
        }

        if modified != xhtml {
            book.resources.insert(key, modified.into_bytes());
        }
    }

    Ok(total)
}

fn replace_in_text_nodes(xhtml: &str, re: &regex::Regex, replacement: &str) -> String {
    // Simple approach: replace in text between > and <
    let mut result = String::new();
    let mut in_tag = false;
    let mut text_buf = String::new();

    for ch in xhtml.chars() {
        if ch == '<' {
            if !text_buf.is_empty() {
                result.push_str(&re.replace_all(&text_buf, replacement));
                text_buf.clear();
            }
            in_tag = true;
            result.push(ch);
        } else if ch == '>' {
            in_tag = false;
            result.push(ch);
        } else if in_tag {
            result.push(ch);
        } else {
            text_buf.push(ch);
        }
    }

    if !text_buf.is_empty() {
        result.push_str(&re.replace_all(&text_buf, replacement));
    }

    result
}

fn count_matches(xhtml: &str, re: &regex::Regex) -> usize {
    let text = strip_html_tags(xhtml);
    re.find_iter(&text).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epub::*;
    use std::collections::HashMap;

    fn test_book() -> EpubBook {
        let xhtml = b"<?xml version=\"1.0\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>Ch1</title></head><body><h1>Chapter 1</h1><p>Hello world.</p></body></html>";
        let xhtml2 = b"<?xml version=\"1.0\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>Ch2</title></head><body><h1>Chapter 2</h1><p>Goodbye world.</p></body></html>";

        let mut resources = HashMap::new();
        resources.insert("OEBPS/ch1.xhtml".to_string(), xhtml.to_vec());
        resources.insert("OEBPS/ch2.xhtml".to_string(), xhtml2.to_vec());

        EpubBook {
            manifest: vec![
                ManifestItem { id: "ch1".to_string(), href: "ch1.xhtml".to_string(), media_type: "application/xhtml+xml".to_string(), properties: None },
                ManifestItem { id: "ch2".to_string(), href: "ch2.xhtml".to_string(), media_type: "application/xhtml+xml".to_string(), properties: None },
            ],
            spine: vec![
                SpineItem { idref: "ch1".to_string(), linear: true, properties: None },
                SpineItem { idref: "ch2".to_string(), linear: true, properties: None },
            ],
            resources,
            ..Default::default()
        }
    }

    #[test]
    fn test_search_literal() {
        let book = test_book();
        let matches = search(&book, "Hello", None, false).unwrap();
        assert!(!matches.is_empty());
        assert_eq!(matches[0].chapter_id, "ch1");
    }

    #[test]
    fn test_search_regex() {
        let book = test_book();
        let matches = search(&book, r"Hello \w+", None, true).unwrap();
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_search_with_chapter_filter() {
        let book = test_book();
        let matches = search(&book, "world", Some("ch1"), false).unwrap();
        assert!(!matches.is_empty());
        for m in &matches {
            assert_eq!(m.chapter_id, "ch1");
        }
    }

    #[test]
    fn test_search_no_matches() {
        let book = test_book();
        let matches = search(&book, "nonexistent_string_xyz", None, false).unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_replace_literal() {
        let mut book = test_book();
        let count = replace(&mut book, "Hello", "Hi", None, false).unwrap();
        assert!(count >= 1);
    }

    #[test]
    fn test_replace_preserves_tags() {
        // "title" appears in <title> tag too, but replace should only affect text nodes
        let result = replace_in_text_nodes(
            "<p title=\"Hello\">Hello world</p>",
            &regex::Regex::new("Hello").unwrap(),
            "Hi",
        );
        // Tag attribute should be preserved
        assert!(result.contains("title=\"Hello\""), "tag attr modified: {result}");
        assert!(result.contains("Hi world"));
    }

    #[test]
    fn test_list_headings() {
        let book = test_book();
        let headings = list_headings(&book).unwrap();
        assert!(!headings.is_empty());
        let (_href, level, text) = &headings[0];
        assert_eq!(*level, 1);
        assert_eq!(text, "Chapter 1");
    }

    #[test]
    fn test_restructure_headings_valid() {
        let mut book = test_book();
        let count = restructure_headings(&mut book, "h1->h2").unwrap();
        assert!(count >= 1);
        // Verify h1 tags are now h2
        let key = book.resources.keys().find(|k| k.contains("ch1")).unwrap().clone();
        let content = String::from_utf8(book.resources[&key].clone()).unwrap();
        assert!(content.contains("<h2>"), "no h2 found: {content}");
        assert!(!content.contains("<h1>"), "h1 still present: {content}");
    }

    #[test]
    fn test_restructure_headings_invalid_mapping() {
        let mut book = test_book();
        assert!(restructure_headings(&mut book, "h1").is_err());
    }
}
