use pulldown_cmark::{Options, Parser, html};
use regex::Regex;

/// Convert Markdown to EPUB 3.3 XHTML
pub fn markdown_to_xhtml(md: &str, title: &str, stylesheet: Option<&str>) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_HEADING_ATTRIBUTES;

    // Convert pandoc inline spans []{#id} back to HTML anchors for pulldown-cmark.
    // Heading attributes {#id} are handled natively by ENABLE_HEADING_ATTRIBUTES.
    let preprocessed = preprocess_pandoc_spans(md);
    let parser = Parser::new_ext(&preprocessed, options);

    let mut body_html = String::new();
    html::push_html(&mut body_html, parser);

    let css_link = stylesheet
        .map(|href| format!("<link rel=\"stylesheet\" type=\"text/css\" href=\"{href}\"/>"))
        .unwrap_or_default();

    format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<!DOCTYPE html>\n",
            "<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\">\n",
            "<head>\n",
            "  <meta charset=\"UTF-8\"/>\n",
            "  <title>{title}</title>\n",
            "  {css}\n",
            "</head>\n",
            "<body>\n",
            "{body}",
            "</body>\n",
            "</html>\n",
        ),
        title = xml_escape(title),
        css = css_link,
        body = body_html,
    )
}

/// Convert pandoc inline spans `[]{#id}` to HTML anchors for pulldown-cmark.
fn preprocess_pandoc_spans(md: &str) -> String {
    let re = Regex::new(r#"\[\]\{#([^}]+)\}"#).expect("valid regex");
    re.replace_all(md, r#"<a id="$1"></a>"#).to_string()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown_to_xhtml() {
        let xhtml = markdown_to_xhtml("# Hello\n\nWorld", "Test", None);
        assert!(xhtml.contains("<h1>Hello</h1>"));
        assert!(xhtml.contains("<p>World</p>"));
    }

    #[test]
    fn test_with_stylesheet() {
        let xhtml = markdown_to_xhtml("text", "Title", Some("styles.css"));
        assert!(xhtml.contains(r#"<link rel="stylesheet" type="text/css" href="styles.css"/>"#));
    }

    #[test]
    fn test_without_stylesheet() {
        let xhtml = markdown_to_xhtml("text", "Title", None);
        assert!(!xhtml.contains("stylesheet"));
    }

    #[test]
    fn test_title_escaping() {
        let xhtml = markdown_to_xhtml("text", "A<B>&C", None);
        assert!(xhtml.contains("<title>A&lt;B&gt;&amp;C</title>"));
    }

    #[test]
    fn test_heading_attributes() {
        let xhtml = markdown_to_xhtml("## Section {#sec1}\n\nText", "Test", None);
        assert!(
            xhtml.contains(r#"id="sec1""#),
            "heading attribute not preserved: {xhtml}"
        );
    }

    #[test]
    fn test_pandoc_span_conversion() {
        let xhtml = markdown_to_xhtml("[]{#anchor1}\n\nText", "Test", None);
        assert!(
            xhtml.contains(r#"id="anchor1""#),
            "pandoc span not converted to anchor: {xhtml}"
        );
    }

    #[test]
    fn test_preprocess_pandoc_spans() {
        assert_eq!(preprocess_pandoc_spans("[]{#foo}"), r#"<a id="foo"></a>"#);
        assert_eq!(
            preprocess_pandoc_spans("text []{#bar} more"),
            r#"text <a id="bar"></a> more"#
        );
        // Heading attributes should NOT be touched
        assert_eq!(
            preprocess_pandoc_spans("## Heading {#id}"),
            "## Heading {#id}"
        );
    }
}
