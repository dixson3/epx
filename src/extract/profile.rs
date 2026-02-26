use crate::epub::EpubBook;
use regex::Regex;
use std::fmt;

/// Genre classification for an EPUB book
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookGenre {
    Fiction,
    Technical,
    Reference,
    Illustrated,
    Minimal,
}

impl fmt::Display for BookGenre {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BookGenre::Fiction => write!(f, "Fiction"),
            BookGenre::Technical => write!(f, "Technical"),
            BookGenre::Reference => write!(f, "Reference"),
            BookGenre::Illustrated => write!(f, "Illustrated"),
            BookGenre::Minimal => write!(f, "Minimal"),
        }
    }
}

/// Structural profile of an EPUB book
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BookProfile {
    pub genre: BookGenre,
    pub spine_count: usize,
    pub image_count: usize,
    pub cross_reference_count: usize,
    pub has_image_gallery: bool,
    pub has_svg_cover: bool,
    pub empty_alt_count: usize,
}

/// Analyze an `EpubBook` to produce a structural profile.
///
/// Scans spine XHTML content for images, cross-references, SVG covers,
/// and empty alt attributes. Classifies genre based on heuristics.
pub fn analyze_book(book: &EpubBook) -> BookProfile {
    let opf_dir = book.detect_opf_dir();

    let href_re = Regex::new(r#"href="[^"]*#[^"]+""#).expect("valid regex");
    let img_re = Regex::new(r"<img\b[^>]*>").expect("valid regex");
    let svg_image_re =
        Regex::new(r"(?is)<svg\b[^>]*>.*?<image\b[^>]*>.*?</svg>").expect("valid regex");
    let empty_alt_re = Regex::new(r#"<img\b[^>]*\balt\s*=\s*""[^>]*>"#).expect("valid regex");
    let all_img_re = Regex::new(r"<img\b[^>]*>").expect("valid regex");
    let has_alt_re = Regex::new(r#"\balt\s*="#).expect("valid regex");

    let mut image_count = 0usize;
    let mut cross_reference_count = 0usize;
    let mut has_svg_cover = false;
    let mut empty_alt_count = 0usize;
    let mut gallery_chapters = 0usize;

    let spine_count = book.spine.len();

    for spine_item in &book.spine {
        let Some(manifest_item) = book.manifest.iter().find(|m| m.id == spine_item.idref) else {
            continue;
        };
        if !manifest_item.media_type.contains("html") && !manifest_item.media_type.contains("xml") {
            continue;
        }

        let full_path = if opf_dir.is_empty() {
            manifest_item.href.clone()
        } else {
            format!("{opf_dir}{}", manifest_item.href)
        };

        let xhtml = book
            .resources
            .get(&full_path)
            .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
            .unwrap_or_default();

        let chapter_images = img_re.find_iter(&xhtml).count();
        image_count += chapter_images;
        cross_reference_count += href_re.find_iter(&xhtml).count();
        empty_alt_count += empty_alt_re.find_iter(&xhtml).count()
            + all_img_re
                .find_iter(&xhtml)
                .filter(|m| !has_alt_re.is_match(m.as_str()))
                .count();

        if svg_image_re.is_match(&xhtml) {
            has_svg_cover = true;
        }

        // A chapter where images dominate: more images than text paragraphs
        let text_len = xhtml.len().saturating_sub(chapter_images * 200);
        if chapter_images > 5 && chapter_images * 100 > text_len {
            gallery_chapters += 1;
        }
    }

    let has_image_gallery = gallery_chapters > 0;

    let genre = classify_genre(spine_count, image_count, cross_reference_count);

    BookProfile {
        genre,
        spine_count,
        image_count,
        cross_reference_count,
        has_image_gallery,
        has_svg_cover,
        empty_alt_count,
    }
}

fn classify_genre(spine_count: usize, image_count: usize, cross_refs: usize) -> BookGenre {
    if image_count > 100 && cross_refs > 500 {
        BookGenre::Technical
    } else if spine_count > 100 {
        BookGenre::Reference
    } else if image_count > 10 && cross_refs < 10 {
        BookGenre::Illustrated
    } else if spine_count < 15 && image_count < 5 {
        BookGenre::Minimal
    } else {
        BookGenre::Fiction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epub::{EpubBook, ManifestItem, SpineItem};

    fn make_book(spine_count: usize, xhtml_content: &str) -> EpubBook {
        let mut book = EpubBook::default();
        for i in 0..spine_count {
            let href = format!("ch{i}.xhtml");
            let full_path = format!("OEBPS/{href}");
            book.manifest.push(ManifestItem {
                id: format!("ch{i}"),
                href,
                media_type: "application/xhtml+xml".to_string(),
                properties: None,
            });
            book.spine.push(SpineItem {
                idref: format!("ch{i}"),
                linear: true,
                properties: None,
            });
            book.resources
                .insert(full_path, xhtml_content.as_bytes().to_vec());
        }
        book.resources
            .insert("OEBPS/content.opf".to_string(), vec![]);
        book
    }

    #[test]
    fn classify_technical() {
        assert_eq!(classify_genre(50, 200, 1000), BookGenre::Technical);
    }

    #[test]
    fn classify_reference() {
        assert_eq!(classify_genre(150, 5, 20), BookGenre::Reference);
    }

    #[test]
    fn classify_illustrated() {
        assert_eq!(classify_genre(20, 50, 5), BookGenre::Illustrated);
    }

    #[test]
    fn classify_minimal() {
        assert_eq!(classify_genre(10, 2, 3), BookGenre::Minimal);
    }

    #[test]
    fn classify_fiction() {
        assert_eq!(classify_genre(30, 5, 20), BookGenre::Fiction);
    }

    #[test]
    fn analyze_minimal_book() {
        let book = make_book(3, "<html><body><p>Hello</p></body></html>");
        let profile = analyze_book(&book);
        assert_eq!(profile.genre, BookGenre::Minimal);
        assert_eq!(profile.spine_count, 3);
        assert_eq!(profile.image_count, 0);
        assert!(!profile.has_svg_cover);
    }

    #[test]
    fn analyze_detects_svg_cover() {
        let xhtml = r#"<html><body><svg xmlns="http://www.w3.org/2000/svg"><image xlink:href="cover.jpg"/></svg></body></html>"#;
        let book = make_book(1, xhtml);
        let profile = analyze_book(&book);
        assert!(profile.has_svg_cover);
    }

    #[test]
    fn analyze_counts_empty_alts() {
        let xhtml = r#"<html><body><img src="a.png" alt=""/><img src="b.png"/><img src="c.png" alt="good"/></body></html>"#;
        let book = make_book(1, xhtml);
        let profile = analyze_book(&book);
        assert_eq!(profile.empty_alt_count, 2);
        assert_eq!(profile.image_count, 3);
    }

    #[test]
    fn genre_display() {
        assert_eq!(BookGenre::Technical.to_string(), "Technical");
        assert_eq!(BookGenre::Fiction.to_string(), "Fiction");
    }
}
