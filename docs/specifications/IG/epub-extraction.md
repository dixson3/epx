# Implementation Guide: EPUB Extraction

## Overview

Extraction converts an in-memory `EpubBook` to an opinionated Markdown directory structure suitable for editing with standard text tools.

## Use Cases

### UC-004: Full Book Extraction
- **Actor:** CLI user running `epx book extract <file>`
- **Preconditions:** Valid EPUB file; output directory writable
- **Flow:**
  1. Read EPUB into EpubBook
  2. Detect OPF directory prefix
  3. Build asset path map (EPUB-internal paths -> extracted relative paths)
  4. For each spine item: locate XHTML in resources, convert to Markdown via `html_to_md::xhtml_to_markdown()`, prepend YAML frontmatter, write to `chapters/NN-slug.md`
  5. Generate `metadata.yml` from EpubMetadata (BookMetadataYaml)
  6. Generate `SUMMARY.md` from navigation tree + chapter file mapping
  7. Extract assets (images -> `assets/images/`, CSS -> `styles/`, fonts -> `assets/fonts/`)
- **Postconditions:** Complete directory structure created; all image/asset links in Markdown are rewritten to relative paths
- **Related:** REQ-002, DD-004

### UC-005: Single Chapter Extraction
- **Actor:** CLI user running `epx chapter extract <file> <id-or-index>`
- **Preconditions:** Valid EPUB; chapter ID or spine index exists
- **Flow:**
  1. Read EPUB
  2. Resolve chapter by ID (spine idref match) or index (spine position)
  3. Locate XHTML content in resources (with OPF dir prefix)
  4. Convert XHTML to Markdown with path map
  5. Output to file or stdout
- **Postconditions:** Markdown content available
- **Related:** REQ-002

### UC-006: Asset Extraction
- **Actor:** CLI user running `epx asset extract-all <file>`
- **Preconditions:** Valid EPUB
- **Flow:**
  1. Read EPUB
  2. Iterate manifest items, categorize by media type
  3. Write images to `assets/images/`, CSS to `styles/`, fonts to `assets/fonts/`
- **Postconditions:** All assets written to organized directory
- **Related:** REQ-008

## Implementation Notes

- XHTML-to-Markdown pipeline: preprocess (strip XML declaration, rewrite epub: namespace prefixes, rewrite asset paths, convert footnotes) -> `html_to_markdown_rs::convert()` -> postprocess (clean blank lines, trim trailing whitespace, ensure final newline)
- Chapter filename: `{index:02}-{slug}.md` where slug comes from TOC label or original filename stem
- Frontmatter includes `original_file`, `original_id`, `spine_index` for traceability
- BookMetadataYaml includes `epx` section with source_format, epub_version, extracted_date
- Key files: `src/extract/mod.rs`, `src/extract/html_to_md.rs`, `src/extract/frontmatter.rs`
