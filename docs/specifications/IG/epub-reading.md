# Implementation Guide: EPUB Reading

## Overview

The EPUB reading layer parses EPUB files (both v2 and v3) into the `EpubBook` in-memory domain model. This is the foundation for all other operations.

## Use Cases

### UC-001: Read EPUB File
- **Actor:** CLI user or library consumer
- **Preconditions:** File exists and is a valid ZIP archive with `mimetype` as first entry
- **Flow:**
  1. Open ZIP archive via `zip_utils::open_epub()`
  2. Validate mimetype entry (`application/epub+zip`, stored compression)
  3. Parse `META-INF/container.xml` to locate OPF rootfile path
  4. Parse OPF to extract metadata (Dublin Core), manifest, spine, and version
  5. Load all non-meta resources into `HashMap<String, Vec<u8>>`
  6. Parse navigation: try EPUB 3 nav.xhtml first, fall back to NCX
  7. Return populated `EpubBook`
- **Postconditions:** All EPUB content is in memory; subsequent operations require no file I/O
- **Related:** REQ-001, DD-002

### UC-002: Display Book Information
- **Actor:** CLI user running `epx book info <file>`
- **Preconditions:** Valid EPUB file
- **Flow:**
  1. Read EPUB (UC-001)
  2. Extract title, creators, languages, EPUB version, chapter count, asset count
  3. Format as human-readable text or JSON (if `--json` flag)
- **Postconditions:** Information displayed to stdout
- **Related:** REQ-011, REQ-012

### UC-003: Validate EPUB Structure
- **Actor:** CLI user running `epx book validate <file>`
- **Preconditions:** Valid EPUB file
- **Flow:**
  1. Read EPUB (UC-001)
  2. Check: titles present, languages present, identifiers present
  3. Check: all spine idrefs reference existing manifest items
  4. Check: spine is non-empty
  5. Report issues or "valid"
- **Postconditions:** Validation result displayed; exit code 0 regardless
- **Related:** REQ-010

## Implementation Notes

- OPF directory detection: check resource keys for `.opf` suffix, then try common prefixes (OEBPS/, OPS/, EPUB/, content/)
- OPF parser uses quick-xml event-based streaming; handles both `Start`/`End` events (for metadata text) and `Empty` events (for manifest/spine items)
- Navigation parser closure pattern: `parse_navigation(&manifest, &|href| { ... })` allows the reader to resolve href to content without coupling to ZIP I/O
- Key files: `src/epub/reader.rs`, `src/epub/opf.rs`, `src/epub/navigation.rs`
