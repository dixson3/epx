use pulldown_cmark::{Options, Parser, html};

/// Convert Markdown to EPUB 3.3 XHTML
pub fn markdown_to_xhtml(md: &str, title: &str, stylesheet: Option<&str>) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_HEADING_ATTRIBUTES;

    let parser = Parser::new_ext(md, options);

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
}
