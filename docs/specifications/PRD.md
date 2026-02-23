# Product Requirements Document (PRD)

## 1. Purpose & Goals

`epx` is a self-contained Rust CLI tool for extracting, manipulating, and assembling EPUB files. Its primary workflow is bidirectional EPUB-to-Markdown conversion, enabling users to work with EPUB content as opinionated Markdown+assets directories, edit in familiar text tooling, and reassemble back to valid EPUB 3.3.

**Core goals:**
- Provide a single-binary, zero-external-dependency tool for EPUB workflows
- Support noun-verb CLI syntax (like `gh`) with 7 resource types and 28 subcommands
- Extract any EPUB version (2.x/3.x) to a well-structured Markdown directory
- Assemble Markdown directories into valid EPUB 3.3 packages
- Enable in-place EPUB manipulation (metadata, chapters, TOC, spine, assets, content)
- Target macOS + Linux with distribution via `cargo install`, Homebrew tap, and GitHub releases

## 2. Technical Constraints

- **Runtime/Language:** Rust (edition 2024), single binary, no runtime dependencies
- **Platforms:** macOS (aarch64, x86_64), Linux (x86_64, aarch64)
- **Dependencies (key):**
  - `clap 4.5` (CLI framework with derive macros)
  - `zip 7.4` (EPUB ZIP handling)
  - `quick-xml 0.37` (OPF/container.xml/NCX parsing)
  - `html-to-markdown-rs 2.24` (XHTML to Markdown conversion)
  - `pulldown-cmark 0.13` with SIMD (Markdown to XHTML)
  - `serde + serde_yaml_ng + serde_json` (metadata serialization)
  - `thiserror 2 + anyhow 1` (error handling)
- **EPUB constraints:** mimetype must be first ZIP entry (stored, uncompressed); OPF directory varies across EPUBs; EPUB 3 nav.xhtml preferred with NCX fallback
- **Distribution:** GitHub Actions CI/CD, GitHub Releases. Homebrew tap (`dixson3/homebrew-tap`) and cargo-dist pending (see TODO-019)

## 3. Requirement Traceability Matrix

| ID | Requirement Description | Priority | Status | Code Reference |
|:---|:---|:---|:---|:---|
| REQ-001 | Parse and read EPUB 2.x and 3.x files into an in-memory domain model (EpubBook) | Critical | Complete | `src/epub/reader.rs`, `src/epub/opf.rs`, `src/epub/container.rs`, `src/epub/navigation.rs` |
| REQ-002 | Extract full EPUB to opinionated directory structure: chapters/ (Markdown), assets/ (images, fonts), styles/ (CSS), metadata.yml, SUMMARY.md | Critical | Complete | `src/extract/mod.rs`, `src/extract/html_to_md.rs`, `src/extract/frontmatter.rs`, `src/extract/asset_extract.rs`, `src/extract/summary.rs` |
| REQ-003 | Assemble Markdown directory (metadata.yml + SUMMARY.md + chapters/ + assets/) into valid EPUB 3.3 | Critical | Complete | `src/assemble/mod.rs`, `src/assemble/package.rs`, `src/assemble/md_to_xhtml.rs`, `src/assemble/metadata_build.rs`, `src/assemble/spine_build.rs` |
| REQ-004 | Manipulate EPUB metadata in-place: set, remove, import (YAML), export (YAML) for all Dublin Core fields | High | Complete | `src/manipulate/meta_edit.rs` |
| REQ-005 | Manipulate chapters: add from Markdown, remove by ID/index, reorder in spine | High | Complete | `src/manipulate/chapter_manage.rs` |
| REQ-006 | Manipulate TOC: show tree, set from Markdown, auto-generate from XHTML headings with configurable depth | High | Complete | `src/manipulate/toc_edit.rs` |
| REQ-007 | Manipulate spine: list, reorder, set from YAML file | High | Complete | `src/manipulate/toc_edit.rs` (reorder_spine, set_spine_order) |
| REQ-008 | Manipulate assets: list with type filtering, extract single/all, add with media type inference, remove with reference warning | High | Complete | `src/manipulate/asset_manage.rs`, `src/assemble/asset_embed.rs` |
| REQ-009 | Content operations: search (literal + regex), replace (with dry-run), heading listing, heading restructure (level remapping) | High | Complete | `src/manipulate/content_edit.rs` |
| REQ-010 | Validate EPUB structure: required metadata fields, spine-manifest integrity, non-empty spine | Medium | Complete | `src/main.rs` (handle_book / BookCommand::Validate) |
| REQ-011 | Noun-verb CLI with 7 resources (book, chapter, metadata, toc, spine, asset, content) and global flags (--json, --verbose, --quiet, --no-color) | Critical | Complete | `src/cli/mod.rs`, `src/cli/*.rs` |
| REQ-012 | Output formatting: JSON mode (`--json`) for list/show/info/validate commands; extract commands output content data directly. TTY-aware table formatting, NO_COLOR support | Medium | Complete | `src/cli/output.rs` |
| REQ-013 | Round-trip fidelity: extract then assemble produces a valid EPUB 3.3 file | Critical | Complete | `tests/roundtrip_test.rs` |
| REQ-014 | Atomic writes for all EPUB modification operations (write to .tmp then rename) | High | Complete | `src/epub/writer.rs` |
| REQ-015 | Binary distribution for macOS (arm64 + x86_64) and Linux (x86_64 + arm64) via GitHub releases and Homebrew | Medium | Complete | `.github/workflows/release.yml`, `Cargo.toml` (workspace.metadata.dist) |

## 4. Functional Specifications

### Book Operations
- **Logic:** `book info` reads EPUB and displays title, author, language, version, chapter count, asset count. `book extract` converts full EPUB to directory. `book assemble` reverses the process. `book validate` checks structural integrity.
- **Validation:** Input must be valid EPUB ZIP with correct mimetype entry. Output directory must be writable.
- **Related:** DD-001, DD-002, NFR-001

### Chapter Operations
- **Logic:** `chapter list` shows spine-ordered chapters with IDs and HREFs. `chapter extract` converts single chapter XHTML to Markdown. `chapter add` converts Markdown file to XHTML and inserts at position. `chapter remove` removes from spine, manifest, resources, and navigation. `chapter reorder` moves chapter within spine.
- **Validation:** Chapter ID or index must exist. Markdown file must be readable. Reorder indices must be within bounds.
- **Related:** REQ-005, DD-003

### Metadata Operations
- **Logic:** `metadata show` displays all Dublin Core fields. `metadata set` updates a single field (title, creator, language, publisher, description, rights, identifier, date, subject, or custom). `metadata remove` clears a field. `metadata import` reads from YAML file. `metadata export` writes to YAML file.
- **Validation:** Field name must be recognized or treated as custom. YAML must be valid.
- **Related:** REQ-004, DD-004

### TOC/Spine Operations
- **Logic:** `toc show` renders hierarchical tree with optional depth limit. `toc set` parses Markdown link list format. `toc generate` scans XHTML headings across all spine items. `spine list` shows ordered idrefs with linear flag. `spine reorder` and `spine set` modify reading order.
- **Validation:** Max depth for TOC generation defaults to 3. Spine idrefs must exist.
- **Related:** REQ-006, REQ-007, DD-005

### Content Operations
- **Logic:** `content search` finds text patterns across EPUB chapters with optional regex and chapter filtering. `content replace` modifies text nodes only (preserves HTML attributes). `content headings` lists all heading elements. `content headings --restructure` remaps heading levels.
- **Validation:** Regex patterns must compile. Heading levels must be 1-6. Mapping format is "hN->hM".
- **Related:** REQ-009, DD-006

### Asset Operations
- **Logic:** `asset list` shows manifest items with optional type filtering (image, css, font, audio). `asset extract` retrieves single asset by path. `asset extract-all` exports all assets organized by type. `asset add` reads file, infers media type, and adds to manifest+resources. `asset remove` removes from manifest+resources with reference-checking warning.
- **Validation:** Asset path must exist in resources. Added files must be readable. Media type can be overridden.
- **Related:** REQ-008, DD-007
