use crate::util::strip_html_tags;
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Convert EPUB XHTML content to Markdown
///
/// `referenced_ids` controls which anchor IDs are preserved:
/// - Empty set: no anchors preserved (single-chapter extraction without full-book context)
/// - Non-empty set: only IDs in the set are preserved (full-book extraction)
pub fn xhtml_to_markdown(
    xhtml: &str,
    path_map: &HashMap<String, String>,
    referenced_ids: &HashSet<String>,
) -> String {
    let preprocessed = preprocess_xhtml(xhtml, path_map, referenced_ids);
    let md = html_to_markdown_rs::convert(&preprocessed, None).unwrap_or_default();
    postprocess_markdown(&md)
}

/// Pre-process EPUB XHTML before Markdown conversion
fn preprocess_xhtml(
    xhtml: &str,
    path_map: &HashMap<String, String>,
    referenced_ids: &HashSet<String>,
) -> String {
    let mut html = xhtml.to_string();

    // Strip XML declaration
    if let Some(end) = html.find("?>")
        && html.starts_with("<?xml")
    {
        html = html[end + 2..].to_string();
    }

    // Strip <head> section to prevent the converter from emitting frontmatter
    // from <title> and <meta> tags (epx generates its own frontmatter from metadata)
    if let Ok(head_re) = Regex::new("(?is)<head[^>]*>.*?</head>") {
        html = head_re.replace_all(&html, "").to_string();
    }

    // Preserve fragment-target IDs as placeholders before the markdown converter strips them.
    // EPUBs use id attributes as fragment targets for cross-references (#id links).
    // The markdown converter drops all id attributes, so we extract them as text tokens
    // that survive conversion and are restored to HTML anchors in postprocessing.
    //
    // Only IDs present in `referenced_ids` are preserved. If the set is empty,
    // no anchors are preserved (single-chapter mode or no references in the EPUB).

    // Step 1a: Empty <a id="..."></a> anchors — preserve if referenced, drop if not
    if let Ok(anchor_re) = Regex::new(r#"<a\s[^>]*id="([^"]+)"[^>]*>\s*</a>"#) {
        html = anchor_re
            .replace_all(&html, |caps: &regex::Captures| {
                let id = &caps[1];
                if referenced_ids.contains(id) {
                    format!("EPXANCHOR__{id}__ENDEPX")
                } else {
                    String::new()
                }
            })
            .to_string();
    }

    // Step 1b: Non-empty <a id="...">content</a> — preserve id if referenced, strip if not
    if let Ok(anchor_id_re) = Regex::new(r#"(<a\b)([^>]*?)\sid="([^"]+)"([^>]*>)"#) {
        html = anchor_id_re
            .replace_all(&html, |caps: &regex::Captures| {
                let tag_start = &caps[1];
                let before = &caps[2];
                let id = &caps[3];
                let after = &caps[4];
                if referenced_ids.contains(id) {
                    format!("EPXANCHOR__{id}__ENDEPX{tag_start}{before}{after}")
                } else {
                    format!("{tag_start}{before}{after}")
                }
            })
            .to_string();
    }

    // Step 2: Element IDs (p, h1-h6, section, article, li) — preserve if referenced
    if let Ok(elem_id_re) =
        Regex::new(r#"(<(?:p|h[1-6]|section|article|li)\b)([^>]*?)\sid="([^"]+)"([^>]*>)"#)
    {
        html = elem_id_re
            .replace_all(&html, |caps: &regex::Captures| {
                let tag_start = &caps[1];
                let before = &caps[2];
                let id = &caps[3];
                let after = &caps[4];
                if referenced_ids.contains(id) {
                    format!("{tag_start}{before}{after}EPXANCHOR__{id}__ENDEPX")
                } else {
                    format!("{tag_start}{before}{after}")
                }
            })
            .to_string();
    }

    // Strip epub namespace prefixes from tags
    html = html.replace("epub:", "data-epub-");

    // Rewrite image/asset paths using placeholders to prevent double-replacement
    // (e.g. replacing "cover.jpeg" inside an already-rewritten "../assets/images/cover.jpeg")
    let mut path_entries: Vec<_> = path_map.iter().collect();
    path_entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    let mut placeholders: Vec<(String, String)> = Vec::new();
    for (i, (old_path, new_path)) in path_entries.iter().enumerate() {
        let placeholder = format!("\x00EPX_PATH_{i}\x00");
        html = html.replace(old_path.as_str(), &placeholder);
        placeholders.push((placeholder, new_path.to_string()));
    }
    for (placeholder, new_path) in &placeholders {
        html = html.replace(placeholder, new_path);
    }

    // Convert epub:type footnotes to markdown-style footnote markers
    if let Ok(footnote_re) =
        Regex::new("<aside[^>]*data-epub-type=\"footnote\"[^>]*id=\"([^\"]*)\"[^>]*>(.*?)</aside>")
    {
        html = footnote_re
            .replace_all(&html, |caps: &regex::Captures| {
                let id = &caps[1];
                let content = &caps[2];
                let text = strip_html_tags(content);
                format!("[^{id}]: {text}")
            })
            .to_string();
    }

    // Convert footnote references
    if let Ok(fn_ref_re) =
        Regex::new("<a[^>]*data-epub-type=\"noteref\"[^>]*href=\"#([^\"]*)\"[^>]*>[^<]*</a>")
    {
        html = fn_ref_re
            .replace_all(&html, |caps: &regex::Captures| {
                let id = &caps[1];
                format!("[^{id}]")
            })
            .to_string();
    }

    html
}

/// Post-process converted Markdown
fn postprocess_markdown(md: &str) -> String {
    let mut result = md.to_string();

    // Restore anchor ID placeholders as HTML anchor tags.
    // The placeholder tokens EPXANCHOR_<id>_ENDEPX were inserted during
    // preprocessing to survive the markdown converter.
    if let Ok(anchor_re) = Regex::new(r"EPXANCHOR__(.+?)__ENDEPX") {
        result = anchor_re
            .replace_all(&result, |caps: &regex::Captures| {
                let id = &caps[1];
                format!("<a id=\"{id}\"></a>")
            })
            .to_string();
    }

    // Clean excessive blank lines (3+ to 2)
    if let Ok(blank_re) = Regex::new("\\n{3,}") {
        result = blank_re.replace_all(&result, "\n\n").to_string();
    }

    // Trim trailing whitespace from lines
    result = result
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    // Ensure file ends with single newline
    result = result.trim_end().to_string();
    result.push('\n');

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_refs() -> HashSet<String> {
        HashSet::new()
    }

    fn refs_containing(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_basic_xhtml_to_markdown() {
        let xhtml = r#"<html><body><h1>Title</h1><p>Text paragraph.</p></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &empty_refs());
        assert!(
            md.contains("# Title") || md.contains("Title\n="),
            "expected heading in: {md}"
        );
        assert!(md.contains("Text paragraph."));
    }

    #[test]
    fn test_path_rewriting() {
        let xhtml = r#"<html><body><img src="images/foo.png"/></body></html>"#;
        let mut path_map = HashMap::new();
        path_map.insert(
            "images/foo.png".to_string(),
            "../assets/images/foo.png".to_string(),
        );
        let md = xhtml_to_markdown(xhtml, &path_map, &empty_refs());
        assert!(
            md.contains("../assets/images/foo.png"),
            "path not rewritten: {md}"
        );
    }

    #[test]
    fn test_xml_declaration_stripping() {
        let xhtml =
            r#"<?xml version="1.0" encoding="UTF-8"?><html><body><p>Hello</p></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &empty_refs());
        assert!(!md.contains("<?xml"));
        assert!(md.contains("Hello"));
    }

    #[test]
    fn test_footnote_conversion() {
        let xhtml = r##"<html><body><p>Text<a epub:type="noteref" href="#fn1">1</a></p><aside epub:type="footnote" id="fn1"><p>A footnote</p></aside></body></html>"##;
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &empty_refs());
        assert!(md.contains("[^fn1]"), "footnote ref not found: {md}");
    }

    #[test]
    fn test_excessive_blank_line_cleanup() {
        let input = "Line 1\n\n\n\n\nLine 2";
        let result = postprocess_markdown(input);
        assert!(
            !result.contains("\n\n\n"),
            "too many blank lines: {result:?}"
        );
    }

    #[test]
    fn test_empty_input() {
        let md = xhtml_to_markdown("", &HashMap::new(), &empty_refs());
        assert_eq!(md, "\n");
    }

    #[test]
    fn test_strip_html_tags() {
        assert_eq!(strip_html_tags("<p>Hello <b>world</b></p>"), "Hello world");
    }

    #[test]
    fn test_anchor_id_preservation() {
        let xhtml =
            r#"<html><body><a id="41401"></a><h2>Section Title</h2><p>Content</p></body></html>"#;
        let refs = refs_containing(&["41401"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(
            md.contains(r#"<a id="41401"></a>"#),
            "anchor ID not preserved: {md}"
        );
        assert!(md.contains("Section Title"));
    }

    #[test]
    fn test_multiple_anchor_ids() {
        let xhtml = r#"<html><body><a id="100"></a><h2>First</h2><a id="200"></a><h2>Second</h2></body></html>"#;
        let refs = refs_containing(&["100", "200"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(
            md.contains(r#"<a id="100"></a>"#),
            "first anchor missing: {md}"
        );
        assert!(
            md.contains(r#"<a id="200"></a>"#),
            "second anchor missing: {md}"
        );
    }

    #[test]
    fn test_element_id_preservation() {
        let xhtml = r#"<html><body><p id="abc123" class="toc">Chapter 1</p></body></html>"#;
        let refs = refs_containing(&["abc123"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(
            md.contains(r#"<a id="abc123"></a>"#),
            "element ID not preserved: {md}"
        );
    }

    #[test]
    fn test_adjacent_anchor_ids() {
        let xhtml = r#"<html><body><a id="111"></a><a id="222"></a><h2>Title</h2></body></html>"#;
        let refs = refs_containing(&["111", "222"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(
            md.contains(r#"<a id="111"></a>"#),
            "first adjacent anchor missing: {md}"
        );
        assert!(
            md.contains(r#"<a id="222"></a>"#),
            "second adjacent anchor missing: {md}"
        );
    }

    #[test]
    fn test_unreferenced_anchors_stripped() {
        // Empty anchors not in referenced set should be dropped entirely
        let xhtml =
            r#"<html><body><a id="orphan1"></a><a id="keep"></a><h2>Title</h2></body></html>"#;
        let refs = refs_containing(&["keep"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(
            md.contains(r#"<a id="keep"></a>"#),
            "referenced anchor missing: {md}"
        );
        assert!(
            !md.contains("orphan1"),
            "orphaned anchor should be stripped: {md}"
        );
    }

    #[test]
    fn test_unreferenced_element_ids_stripped() {
        // Element IDs not in referenced set should be stripped (id attr only)
        let xhtml = r#"<html><body><p id="calibre_pb_1">Content</p></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &empty_refs());
        assert!(
            !md.contains("calibre_pb_1"),
            "unreferenced element ID should be stripped: {md}"
        );
        assert!(md.contains("Content"));
    }

    #[test]
    fn test_empty_refs_preserves_nothing() {
        // With empty referenced_ids, no anchors should be preserved
        let xhtml = r#"<html><body><a id="100"></a><p id="200">Text</p></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &empty_refs());
        assert!(
            !md.contains(r#"<a id="#),
            "no anchors should be preserved with empty refs: {md}"
        );
        assert!(md.contains("Text"));
    }
}
