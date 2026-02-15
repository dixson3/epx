use crate::epub::NavPoint;

/// Generate SUMMARY.md content from navigation tree
pub fn generate_summary(toc: &[NavPoint], chapter_files: &[(String, String)]) -> String {
    let mut output = String::from("# Summary\n\n");
    write_nav_entries(&mut output, toc, chapter_files, 0);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_summary() {
        let toc = vec![
            NavPoint { label: "Chapter 1".to_string(), href: "ch1.xhtml".to_string(), children: vec![] },
            NavPoint { label: "Chapter 2".to_string(), href: "ch2.xhtml".to_string(), children: vec![] },
        ];
        let files = vec![
            ("ch1.xhtml".to_string(), "01-chapter-1.md".to_string()),
            ("ch2.xhtml".to_string(), "02-chapter-2.md".to_string()),
        ];
        let summary = generate_summary(&toc, &files);
        assert!(summary.starts_with("# Summary"));
        assert!(summary.contains("[Chapter 1](chapters/01-chapter-1.md)"));
        assert!(summary.contains("[Chapter 2](chapters/02-chapter-2.md)"));
    }

    #[test]
    fn test_nested_summary() {
        let toc = vec![
            NavPoint {
                label: "Part 1".to_string(),
                href: "p1.xhtml".to_string(),
                children: vec![
                    NavPoint { label: "Ch 1".to_string(), href: "ch1.xhtml".to_string(), children: vec![] },
                ],
            },
        ];
        let files = vec![
            ("p1.xhtml".to_string(), "00-part-1.md".to_string()),
            ("ch1.xhtml".to_string(), "01-ch-1.md".to_string()),
        ];
        let summary = generate_summary(&toc, &files);
        assert!(summary.contains("  - [Ch 1]"), "no indented entry: {summary}");
    }

    #[test]
    fn test_missing_chapter_file() {
        let toc = vec![
            NavPoint { label: "Missing Chapter".to_string(), href: "missing.xhtml".to_string(), children: vec![] },
        ];
        let files = vec![];
        let summary = generate_summary(&toc, &files);
        assert!(summary.contains("- Missing Chapter"));
        assert!(!summary.contains("]("));
    }
}

fn write_nav_entries(
    output: &mut String,
    points: &[NavPoint],
    chapter_files: &[(String, String)],
    indent: usize,
) {
    for point in points {
        let prefix = "  ".repeat(indent);
        let href = point.href.split('#').next().unwrap_or(&point.href);

        // Find matching chapter file
        let link = chapter_files
            .iter()
            .find(|(orig, _)| href == orig || orig.ends_with(href))
            .map(|(_, md_file)| format!("chapters/{md_file}"));

        if let Some(path) = link {
            output.push_str(&format!("{prefix}- [{}]({path})\n", point.label));
        } else {
            output.push_str(&format!("{prefix}- {}\n", point.label));
        }

        write_nav_entries(output, &point.children, chapter_files, indent + 1);
    }
}
