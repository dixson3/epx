use std::path::Path;

/// Infer media type from file extension
pub fn infer_media_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("mp3") => "audio/mpeg",
        Some("mp4") => "video/mp4",
        Some("xhtml") | Some("html") => "application/xhtml+xml",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_infer_known_types() {
        assert_eq!(infer_media_type(Path::new("img.jpg")), "image/jpeg");
        assert_eq!(infer_media_type(Path::new("img.jpeg")), "image/jpeg");
        assert_eq!(infer_media_type(Path::new("img.png")), "image/png");
        assert_eq!(infer_media_type(Path::new("img.gif")), "image/gif");
        assert_eq!(infer_media_type(Path::new("img.svg")), "image/svg+xml");
        assert_eq!(infer_media_type(Path::new("style.css")), "text/css");
        assert_eq!(infer_media_type(Path::new("font.woff2")), "font/woff2");
        assert_eq!(infer_media_type(Path::new("font.ttf")), "font/ttf");
        assert_eq!(infer_media_type(Path::new("chapter.xhtml")), "application/xhtml+xml");
    }

    #[test]
    fn test_infer_unknown_type() {
        assert_eq!(infer_media_type(Path::new("file.xyz")), "application/octet-stream");
        assert_eq!(infer_media_type(Path::new("no_ext")), "application/octet-stream");
    }
}
