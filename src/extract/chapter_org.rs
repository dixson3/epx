use crate::epub::{EpubBook, NavPoint};
use slug::slugify;

/// Generate chapter filename from TOC, heading, or original filename
pub fn chapter_filename(index: usize, book: &EpubBook, href: &str) -> String {
    let base_name = if let Some(label) = find_toc_label(&book.navigation.toc, href) {
        slugify(&label)
    } else {
        // Fall back to original filename without extension
        let fname = href.rsplit('/').next().unwrap_or(href);
        let stem = fname.rsplit_once('.').map_or(fname, |(s, _)| s);
        slugify(stem)
    };

    let name = if base_name.is_empty() {
        format!("chapter-{index}")
    } else {
        base_name
    };

    format!("{index:02}-{name}.md")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epub::*;

    fn book_with_toc(toc: Vec<NavPoint>) -> EpubBook {
        EpubBook {
            navigation: Navigation {
                toc,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_filename_from_toc_label() {
        let book = book_with_toc(vec![NavPoint {
            label: "Introduction".to_string(),
            href: "ch1.xhtml".to_string(),
            children: vec![],
        }]);
        let name = chapter_filename(0, &book, "ch1.xhtml");
        assert_eq!(name, "00-introduction.md");
    }

    #[test]
    fn test_filename_from_original_file() {
        let book = book_with_toc(vec![]);
        let name = chapter_filename(1, &book, "my-chapter.xhtml");
        assert_eq!(name, "01-my-chapter.md");
    }

    #[test]
    fn test_filename_empty_slug() {
        let book = book_with_toc(vec![NavPoint {
            label: "".to_string(),
            href: "_.xhtml".to_string(),
            children: vec![],
        }]);
        // href "_.xhtml" with empty toc label and stem "_" slugs to empty
        let name = chapter_filename(2, &book, "_.xhtml");
        // Falls back to original filename stem slug
        assert!(name.starts_with("02-"));
    }

    #[test]
    fn test_index_padding() {
        let book = book_with_toc(vec![]);
        let name = chapter_filename(5, &book, "ch.xhtml");
        assert!(name.starts_with("05-"), "got: {name}");
    }
}

fn find_toc_label(toc: &[NavPoint], href: &str) -> Option<String> {
    for point in toc {
        // Match by href (ignoring fragment)
        let point_href = point.href.split('#').next().unwrap_or(&point.href);
        let target_href = href.split('#').next().unwrap_or(href);
        if point_href == target_href || target_href.ends_with(point_href) {
            return Some(point.label.clone());
        }
        if let Some(label) = find_toc_label(&point.children, href) {
            return Some(label);
        }
    }
    None
}
