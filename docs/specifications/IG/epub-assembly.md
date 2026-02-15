# Implementation Guide: EPUB Assembly

## Overview

Assembly converts an opinionated Markdown directory back into a valid EPUB 3.3 file, reversing the extraction process.

## Use Cases

### UC-007: Full Book Assembly
- **Actor:** CLI user running `epx book assemble <dir>`
- **Preconditions:** Directory contains `metadata.yml`, `SUMMARY.md`, and `chapters/` with Markdown files
- **Flow:**
  1. Read `metadata.yml` -> EpubMetadata
  2. Parse `SUMMARY.md` -> chapter ordering + Navigation tree
  3. For each chapter: read Markdown, strip YAML frontmatter, extract title from first `# ` heading, convert to EPUB 3.3 XHTML via pulldown-cmark
  4. Detect and include stylesheets from `styles/` directory
  5. Recursively add assets from `assets/` directory with media type inference
  6. Assemble EpubBook struct
  7. Write EPUB via `writer::write_epub()`
- **Postconditions:** Valid EPUB 3.3 file created with mimetype, container.xml, OPF, nav.xhtml, toc.ncx, all chapters and assets
- **Related:** REQ-003, DD-004, DD-005

### UC-008: EPUB Writing
- **Actor:** Assembly or manipulation operation
- **Preconditions:** Populated EpubBook in memory
- **Flow:**
  1. Create temp file (`path.epub.tmp`)
  2. Write `mimetype` as first entry (stored, no compression)
  3. Write `META-INF/container.xml` pointing to `OEBPS/content.opf`
  4. Generate and write OPF with Dublin Core metadata, manifest, spine
  5. Generate and write `toc.xhtml` (EPUB 3 nav) and `toc.ncx` (EPUB 2 compat)
  6. Write all resources under `OEBPS/` prefix
  7. Finish ZIP, atomic rename to final path
- **Postconditions:** Valid EPUB file on disk; original file untouched on failure
- **Related:** REQ-014, DD-003, NFR-002

## Implementation Notes

- Markdown-to-XHTML uses pulldown-cmark with options: tables, footnotes, strikethrough, heading attributes
- Generated XHTML includes proper XML declaration, DOCTYPE, XHTML namespace, epub namespace
- OPF generation auto-creates UUID identifier and defaults to `en` language if not specified
- Modified timestamp (`dcterms:modified`) auto-generated if not present
- Writer rebases all resource paths under `OEBPS/` prefix
- Key files: `src/assemble/mod.rs`, `src/epub/writer.rs`, `src/assemble/md_to_xhtml.rs`
