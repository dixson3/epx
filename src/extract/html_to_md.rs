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

    // Unwrap SVG cover images: replace <svg>...<image xlink:href="X"/>...</svg>
    // with <img src="X" alt="Cover image"/> when SVG contains only a single <image>
    // and no drawing elements (to avoid replacing actual SVG diagrams).
    if let Ok(svg_re) = Regex::new(r"(?is)<svg\b[^>]*>(.*?)</svg>") {
        let drawing_re =
            Regex::new(r"(?i)<(?:rect|circle|path|text|line|polygon|polyline|ellipse)\b")
                .expect("valid regex");
        let image_href_re = Regex::new(r#"(?i)<image\b[^>]*(?:xlink:)?href="([^"]+)"[^>]*/?\s*>"#)
            .expect("valid regex");
        html = svg_re
            .replace_all(&html, |caps: &regex::Captures| {
                let inner = &caps[1];
                // Only unwrap if there's exactly one <image> and no drawing elements
                if drawing_re.is_match(inner) {
                    return caps[0].to_string();
                }
                let image_caps: Vec<_> = image_href_re.captures_iter(inner).collect();
                if image_caps.len() == 1 {
                    let href = &image_caps[0][1];
                    format!(r#"<img src="{href}" alt="Cover image"/>"#)
                } else {
                    caps[0].to_string()
                }
            })
            .to_string();
    }

    // Fill in empty or missing alt attributes on images with derived text
    if let Ok(empty_alt_re) = Regex::new(r#"(<img\b[^>]*)\balt\s*=\s*""([^>]*>)"#) {
        html = empty_alt_re
            .replace_all(&html, |caps: &regex::Captures| {
                let before = &caps[1];
                let after = &caps[2];
                let alt = derive_alt_from_tag(before);
                format!(r#"{before}alt="{alt}"{after}"#)
            })
            .to_string();
    }
    // Inject alt for <img> tags missing it entirely
    if let Ok(img_tag_re) = Regex::new(r"<img\b[^>]*>") {
        let alt_attr_re = Regex::new(r#"\balt\s*="#).expect("valid regex");
        html = img_tag_re
            .replace_all(&html, |caps: &regex::Captures| {
                let tag = &caps[0];
                if alt_attr_re.is_match(tag) {
                    return tag.to_string(); // already has alt
                }
                let alt = derive_alt_from_tag(tag);
                // Insert alt after <img
                format!(r#"<img alt="{alt}"{}"#, &tag[4..])
            })
            .to_string();
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

    // Step 2: Any element IDs (except <a>, handled above) — preserve if referenced
    if let Ok(elem_id_re) = Regex::new(r#"(<(\w+)\b)([^>]*?)\sid="([^"]+)"([^>]*>)"#) {
        html = elem_id_re
            .replace_all(&html, |caps: &regex::Captures| {
                let tag_start = &caps[1];
                let tag_name = &caps[2];
                let before = &caps[3];
                let id = &caps[4];
                let after = &caps[5];
                // Skip <a> tags — already handled in Steps 1a/1b
                if tag_name.eq_ignore_ascii_case("a") {
                    return caps[0].to_string();
                }
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
///
/// Converts anchor placeholders to pandoc-style markdown syntax:
/// - Heading anchors: `## Heading {#id}`
/// - Inline/block anchors: `[]{#id}`
fn postprocess_markdown(md: &str) -> String {
    let mut result = md.to_string();

    // Step 1: Restore placeholders to intermediate format {{EPX_ID:id}}
    let anchor_re = Regex::new(r"EPXANCHOR__(.+?)__ENDEPX").expect("valid regex");
    result = anchor_re
        .replace_all(&result, |caps: &regex::Captures| {
            let id = &caps[1];
            format!("{{{{EPX_ID:{id}}}}}")
        })
        .to_string();

    // Step 2: Extract anchors from inside bold markers
    // **{{EPX_ID:X}}text** → {{EPX_ID:X}}**text**
    let bold_anchor_re = Regex::new(r"\*\*\{\{EPX_ID:([^}]+)\}\}([^*]*)\*\*").expect("valid regex");
    result = bold_anchor_re
        .replace_all(&result, |caps: &regex::Captures| {
            let id = &caps[1];
            let text = &caps[2];
            if text.is_empty() {
                format!("{{{{EPX_ID:{id}}}}}")
            } else {
                format!("{{{{EPX_ID:{id}}}}}**{text}**")
            }
        })
        .to_string();

    // Step 3: Merge heading IDs — anchor on same line as heading
    // {{EPX_ID:id}}## text → ## text {#id}
    // ## {{EPX_ID:id}}text → ## text {#id}  (after bold extraction)
    let heading_inline_re =
        Regex::new(r"(?m)^(\{\{EPX_ID:[^}]+\}\})(#{1,6}\s+.+)$").expect("valid regex");
    result = heading_inline_re
        .replace_all(&result, |caps: &regex::Captures| {
            let anchor_part = &caps[1];
            let heading = &caps[2];
            format!("{heading}<<PENDING:{anchor_part}>>")
        })
        .to_string();

    let heading_contains_re =
        Regex::new(r"(?m)^(#{1,6}\s+)(.*?)\{\{EPX_ID:([^}]+)\}\}(.*)$").expect("valid regex");
    result = heading_contains_re
        .replace_all(&result, |caps: &regex::Captures| {
            let hashes = &caps[1];
            let before = &caps[2];
            let id = &caps[3];
            let after = &caps[4];
            let text = format!("{before}{after}").trim().to_string();
            format!("{hashes}{text} {{#{id}}}")
        })
        .to_string();

    // Resolve PENDING markers (from pre-heading anchors moved to end)
    let pending_re =
        Regex::new(r"(?m)^(#{1,6}\s+.+?)<<PENDING:\{\{EPX_ID:([^}]+)\}\}>>$").expect("valid regex");
    result = pending_re
        .replace_all(&result, |caps: &regex::Captures| {
            let heading = &caps[1];
            let id = &caps[2];
            format!("{heading} {{#{id}}}")
        })
        .to_string();

    // Step 3b: Merge heading IDs — anchor on line immediately before heading
    // {{EPX_ID:id}}\n\n### text → ### text {#id}
    // Handle multiple anchors: first becomes {#id}, rest become []{#id}
    let pre_heading_re =
        Regex::new(r"(?m)((?:\{\{EPX_ID:[^}]+\}\}\s*)+)\n\n(#{1,6}\s+.+)$").expect("valid regex");
    result = pre_heading_re
        .replace_all(&result, |caps: &regex::Captures| {
            let anchors_block = &caps[1];
            let heading = &caps[2];
            let id_re = Regex::new(r"\{\{EPX_ID:([^}]+)\}\}").expect("valid regex");
            let ids: Vec<String> = id_re
                .captures_iter(anchors_block)
                .map(|c| c[1].to_string())
                .collect();
            if ids.is_empty() {
                return caps[0].to_string();
            }
            let mut lines = Vec::new();
            // Extra IDs become []{#id} on preceding lines
            for extra_id in &ids[1..] {
                lines.push(format!("[]{{#{extra_id}}}"));
            }
            // First ID merges into the heading
            lines.push(format!("{heading} {{#{}}}", ids[0]));
            lines.join("\n")
        })
        .to_string();

    // Step 4: Convert remaining {{EPX_ID:id}} to []{#id} (pandoc inline span)
    let remaining_re = Regex::new(r"\{\{EPX_ID:([^}]+)\}\}").expect("valid regex");
    result = remaining_re
        .replace_all(&result, |caps: &regex::Captures| {
            let id = &caps[1];
            format!("[]{{#{id}}}")
        })
        .to_string();

    // Step 5: Clean excessive blank lines (3+ to 2)
    let blank_re = Regex::new("\\n{3,}").expect("valid regex");
    result = blank_re.replace_all(&result, "\n\n").to_string();

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

/// Derive alt text from an `<img>` tag's `src` attribute.
///
/// Extracts the filename, strips the extension, and humanizes it.
/// Purely numeric filenames (like `338838561`) become `"Image"`.
fn derive_alt_from_tag(tag: &str) -> String {
    let src_re = Regex::new(r#"src="([^"]+)""#).expect("valid regex");
    let src = src_re
        .captures(tag)
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    // Extract filename without extension
    let filename = src
        .rsplit('/')
        .next()
        .unwrap_or(&src)
        .rsplit('\\')
        .next()
        .unwrap_or(&src);
    let name = match filename.rfind('.') {
        Some(pos) => &filename[..pos],
        None => filename,
    };

    if name.is_empty() {
        return "Image".to_string();
    }

    // If purely numeric, use generic label
    if name.chars().all(|c| c.is_ascii_digit()) {
        return "Image".to_string();
    }

    // Humanize: replace underscores with spaces, keep hyphens
    name.replace('_', " ")
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
    fn test_anchor_id_preservation_pandoc() {
        let xhtml =
            r#"<html><body><a id="41401"></a><h2>Section Title</h2><p>Content</p></body></html>"#;
        let refs = refs_containing(&["41401"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        // Anchor before heading should merge as {#id} attribute
        assert!(
            md.contains("{#41401}"),
            "anchor ID not preserved as pandoc attribute: {md}"
        );
        assert!(
            !md.contains("<a id="),
            "should not contain raw HTML anchors: {md}"
        );
        assert!(md.contains("Section Title"));
    }

    #[test]
    fn test_multiple_anchor_ids_pandoc() {
        let xhtml = r#"<html><body><a id="100"></a><h2>First</h2><a id="200"></a><h2>Second</h2></body></html>"#;
        let refs = refs_containing(&["100", "200"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(md.contains("{#100}"), "first anchor missing: {md}");
        assert!(md.contains("{#200}"), "second anchor missing: {md}");
        assert!(
            !md.contains("<a id="),
            "should not contain raw HTML anchors: {md}"
        );
    }

    #[test]
    fn test_element_id_preservation_pandoc() {
        let xhtml = r#"<html><body><p id="abc123" class="toc">Chapter 1</p></body></html>"#;
        let refs = refs_containing(&["abc123"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(
            md.contains("{#abc123}"),
            "element ID not preserved as pandoc syntax: {md}"
        );
    }

    #[test]
    fn test_adjacent_anchor_ids_pandoc() {
        let xhtml = r#"<html><body><a id="111"></a><a id="222"></a><h2>Title</h2></body></html>"#;
        let refs = refs_containing(&["111", "222"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(md.contains("{#111}"), "first adjacent anchor missing: {md}");
        assert!(
            md.contains("{#222}"),
            "second adjacent anchor missing: {md}"
        );
        assert!(
            !md.contains("<a id="),
            "should not contain raw HTML anchors: {md}"
        );
    }

    #[test]
    fn test_unreferenced_anchors_stripped() {
        // Empty anchors not in referenced set should be dropped entirely
        let xhtml =
            r#"<html><body><a id="orphan1"></a><a id="keep"></a><h2>Title</h2></body></html>"#;
        let refs = refs_containing(&["keep"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(md.contains("{#keep}"), "referenced anchor missing: {md}");
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
            !md.contains("{#100}") && !md.contains("{#200}"),
            "no anchors should be preserved with empty refs: {md}"
        );
        assert!(md.contains("Text"));
    }

    // ─── SVG cover unwrapping tests ──────────────────────────

    #[test]
    fn test_svg_single_image_unwrapped() {
        let xhtml = r#"<html><body><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><image xlink:href="cover.jpeg"/></svg></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &empty_refs());
        assert!(
            md.contains("Cover image"),
            "SVG should be unwrapped to img: {md}"
        );
        assert!(!md.contains("<svg"), "SVG tag should be removed: {md}");
    }

    #[test]
    fn test_svg_with_drawing_elements_preserved() {
        let xhtml = r#"<html><body><svg xmlns="http://www.w3.org/2000/svg"><rect x="0" y="0"/><image xlink:href="diagram.png"/></svg></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &empty_refs());
        // SVG with drawing elements should NOT be unwrapped
        assert!(
            !md.contains("Cover image"),
            "SVG with drawings should not be unwrapped: {md}"
        );
    }

    // ─── Universal anchor preservation tests ─────────────────

    #[test]
    fn test_div_id_preserved() {
        let xhtml = r#"<html><body><div id="myref">Content</div></body></html>"#;
        let refs = refs_containing(&["myref"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(
            md.contains("{#myref}"),
            "div ID not preserved as pandoc syntax: {md}"
        );
    }

    #[test]
    fn test_span_id_preserved() {
        let xhtml = r#"<html><body><p><span id="target1">text</span></p></body></html>"#;
        let refs = refs_containing(&["target1"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(
            md.contains("{#target1}"),
            "span ID not preserved as pandoc syntax: {md}"
        );
    }

    #[test]
    fn test_blockquote_id_preserved() {
        let xhtml = r#"<html><body><blockquote id="bq1">Quote text</blockquote></body></html>"#;
        let refs = refs_containing(&["bq1"]);
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &refs);
        assert!(
            md.contains("{#bq1}"),
            "blockquote ID not preserved as pandoc syntax: {md}"
        );
    }

    // ─── Alt-text fallback tests ─────────────────────────────

    #[test]
    fn test_empty_alt_gets_derived() {
        let xhtml = r#"<html><body><img src="images/fig_3-2.png" alt=""/></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &empty_refs());
        assert!(!md.contains("![]"), "empty alt should be replaced: {md}");
        assert!(
            md.contains("fig 3-2"),
            "alt should be derived from filename: {md}"
        );
    }

    #[test]
    fn test_missing_alt_gets_injected() {
        let xhtml = r#"<html><body><img src="images/diagram.png"/></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &empty_refs());
        assert!(!md.contains("![]"), "missing alt should be injected: {md}");
        assert!(
            md.contains("diagram"),
            "alt should be derived from filename: {md}"
        );
    }

    #[test]
    fn test_numeric_filename_becomes_image() {
        let xhtml = r#"<html><body><img src="images/338838561.jpg" alt=""/></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &empty_refs());
        assert!(
            md.contains("Image"),
            "numeric filename should become 'Image': {md}"
        );
    }

    #[test]
    fn test_existing_alt_preserved() {
        let xhtml = r#"<html><body><img src="foo.png" alt="My photo"/></body></html>"#;
        let md = xhtml_to_markdown(xhtml, &HashMap::new(), &empty_refs());
        assert!(
            md.contains("My photo"),
            "existing alt should be preserved: {md}"
        );
    }

    #[test]
    fn test_derive_alt_from_tag_helper() {
        assert_eq!(
            derive_alt_from_tag(r#"<img src="images/fig_3-2.png""#),
            "fig 3-2"
        );
        assert_eq!(derive_alt_from_tag(r#"<img src="338838561.jpg""#), "Image");
        assert_eq!(derive_alt_from_tag(r#"<img src="cover.jpeg""#), "cover");
        assert_eq!(derive_alt_from_tag(r#"<img"#), "Image");
    }
}
