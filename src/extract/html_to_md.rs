use crate::util::strip_html_tags;
use regex::Regex;
use std::collections::HashMap;

/// Convert EPUB XHTML content to Markdown
pub fn xhtml_to_markdown(xhtml: &str, path_map: &HashMap<String, String>) -> String {
    let preprocessed = preprocess_xhtml(xhtml, path_map);
    let md = html_to_markdown_rs::convert(&preprocessed, None).unwrap_or_default();
    postprocess_markdown(&md)
}

/// Pre-process EPUB XHTML before Markdown conversion
fn preprocess_xhtml(xhtml: &str, path_map: &HashMap<String, String>) -> String {
    let mut html = xhtml.to_string();

    // Strip XML declaration
    if let Some(end) = html.find("?>")
        && html.starts_with("<?xml")
    {
        html = html[end + 2..].to_string();
    }

    // Strip epub namespace prefixes from tags
    html = html.replace("epub:", "data-epub-");

    // Rewrite image paths
    for (old_path, new_path) in path_map {
        html = html.replace(old_path, new_path);
    }

    // Convert epub:type footnotes to markdown-style footnote markers
    if let Ok(footnote_re) = Regex::new(
        "<aside[^>]*data-epub-type=\"footnote\"[^>]*id=\"([^\"]*)\"[^>]*>(.*?)</aside>",
    ) {
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
    if let Ok(fn_ref_re) = Regex::new(
        "<a[^>]*data-epub-type=\"noteref\"[^>]*href=\"#([^\"]*)\"[^>]*>[^<]*</a>",
    ) {
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

    #[test]
    fn test_basic_xhtml_to_markdown() {
        let xhtml = r#"<html><body><h1>Title</h1><p>Text paragraph.</p></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new());
        assert!(md.contains("# Title") || md.contains("Title\n="), "expected heading in: {md}");
        assert!(md.contains("Text paragraph."));
    }

    #[test]
    fn test_path_rewriting() {
        let xhtml = r#"<html><body><img src="images/foo.png"/></body></html>"#;
        let mut path_map = HashMap::new();
        path_map.insert("images/foo.png".to_string(), "./assets/images/foo.png".to_string());
        let md = xhtml_to_markdown(xhtml, &path_map);
        assert!(md.contains("./assets/images/foo.png"), "path not rewritten: {md}");
    }

    #[test]
    fn test_xml_declaration_stripping() {
        let xhtml = r#"<?xml version="1.0" encoding="UTF-8"?><html><body><p>Hello</p></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new());
        assert!(!md.contains("<?xml"));
        assert!(md.contains("Hello"));
    }

    #[test]
    fn test_footnote_conversion() {
        let xhtml = r##"<html><body><p>Text<a epub:type="noteref" href="#fn1">1</a></p><aside epub:type="footnote" id="fn1"><p>A footnote</p></aside></body></html>"##;
        let md = xhtml_to_markdown(xhtml, &HashMap::new());
        assert!(md.contains("[^fn1]"), "footnote ref not found: {md}");
    }

    #[test]
    fn test_excessive_blank_line_cleanup() {
        let input = "Line 1\n\n\n\n\nLine 2";
        let result = postprocess_markdown(input);
        assert!(!result.contains("\n\n\n"), "too many blank lines: {result:?}");
    }

    #[test]
    fn test_empty_input() {
        let md = xhtml_to_markdown("", &HashMap::new());
        assert_eq!(md, "\n");
    }

    #[test]
    fn test_strip_html_tags() {
        assert_eq!(strip_html_tags("<p>Hello <b>world</b></p>"), "Hello world");
    }
}
