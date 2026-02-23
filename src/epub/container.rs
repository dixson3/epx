use crate::error::{EpxError, Result};
use quick_xml::Reader;
use quick_xml::events::Event;

/// Parse META-INF/container.xml to find the OPF rootfile path
pub fn parse_container(xml: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e))
                if e.local_name().as_ref() == b"rootfile" =>
            {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"full-path" {
                        let path = String::from_utf8_lossy(&attr.value).into_owned();
                        return Ok(path);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(EpxError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Err(EpxError::InvalidEpub(
        "no rootfile found in container.xml".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_container_epub3() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
        assert_eq!(parse_container(xml).unwrap(), "OEBPS/content.opf");
    }

    #[test]
    fn parse_container_epub2() {
        let xml = r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
        assert_eq!(parse_container(xml).unwrap(), "content.opf");
    }

    #[test]
    fn parse_container_missing_rootfile() {
        let xml = r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
  </rootfiles>
</container>"#;
        assert!(parse_container(xml).is_err());
    }

    #[test]
    fn parse_container_malformed_xml() {
        let xml = "<container><not-closed>";
        assert!(parse_container(xml).is_err());
    }
}
