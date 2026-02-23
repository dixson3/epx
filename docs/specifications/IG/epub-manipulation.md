# Implementation Guide: EPUB Manipulation

## Overview

Manipulation enables in-place editing of EPUB files without full extract/assemble round-trips. All operations use the read-modify-write pattern with atomic file writes.

## Use Cases

### UC-009: Metadata Editing
- **Actor:** CLI user running `epx metadata set <file> --field <name> --value <value>`
- **Preconditions:** Valid EPUB file; writable location
- **Flow:**
  1. `modify_epub()` reads EPUB into EpubBook
  2. `set_field()` dispatches on field name: title, creator/author, language, publisher, description, rights, identifier, date, subject, or custom field
  3. For known fields: replaces first element or pushes if empty (except `subject` which always appends)
  4. For unknown fields: inserts into `custom` HashMap
  5. Write back atomically
- **Postconditions:** EPUB file updated with new metadata
- **Related:** REQ-004, DD-003

### UC-010: Chapter Management
- **Actor:** CLI user running `epx chapter add/remove/reorder`
- **Preconditions:** Valid EPUB; for add: Markdown file must exist
- **Flow (add):**
  1. Read Markdown file, extract or accept title
  2. Convert to XHTML via pulldown-cmark
  3. Generate unique ID and href from slugified title
  4. Add to resources (under detected OPF dir), manifest, spine (at position if `--after`), and navigation
- **Flow (remove):**
  1. Resolve chapter by ID or spine index
  2. Remove from spine, manifest, resources, and navigation tree (recursive)
- **Flow (reorder):**
  1. Validate from/to indices within bounds
  2. Remove spine item at `from`, insert at `to`
- **Postconditions:** EPUB updated; structural integrity maintained
- **Related:** REQ-005, DD-003

### UC-011: Content Search and Replace
- **Actor:** CLI user running `epx content search/replace`
- **Preconditions:** Valid EPUB; valid pattern (regex must compile if --regex)
- **Flow (search):**
  1. Iterate spine items, optionally filter by chapter ID/index
  2. For each XHTML resource: strip HTML tags to plain text
  3. Match pattern line by line, collect matches with chapter ID, href, line number, context
- **Flow (replace):**
  1. Iterate spine items (same filtering)
  2. For each XHTML: apply regex only in text nodes (between > and <)
  3. Count matches in text-stripped version, replace in original
  4. Write modified XHTML back to resources
- **Postconditions:** Matches displayed (search) or content modified (replace)
- **Related:** REQ-009, DD-006, NFR-007

### UC-012: TOC Generation from Headings
- **Actor:** CLI user running `epx toc generate <file>`
- **Preconditions:** Valid EPUB with XHTML chapters containing heading elements
- **Flow:**
  1. Iterate spine items in order
  2. For each XHTML: find all `<h1>`-`<h6>` via regex
  3. Filter by max depth (default 3)
  4. Build flat NavPoint list (label from heading text, href from chapter)
  5. Replace book navigation TOC
- **Postconditions:** TOC regenerated from actual content headings
- **Related:** REQ-006

## Implementation Notes

- `modify_epub()` pattern: `|book: &mut EpubBook| -> Result<()>` closure ensures all modifications happen between read and write
- OPF directory detection is handled by the shared `EpubBook::detect_opf_dir()` method in `epub/mod.rs`, called from extract, chapter_manage, and asset_manage
- Asset removal includes reference checking: warns (stderr) if asset href appears in any XHTML content
- Key files: `src/manipulate/meta_edit.rs`, `src/manipulate/chapter_manage.rs`, `src/manipulate/content_edit.rs`, `src/manipulate/toc_edit.rs`, `src/manipulate/asset_manage.rs`
