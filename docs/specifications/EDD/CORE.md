# Engineering Design Document

## Overview

`epx` is architected as a single Rust crate with 6 top-level modules organized by concern:

```
src/
  main.rs          # CLI dispatch (693 lines)
  lib.rs           # Module re-exports
  error.rs         # EpxError enum (thiserror)
  cli/             # CLI definitions (clap derive)
    mod.rs         # Cli struct + Resource enum (7 variants)
    book.rs        # BookCommand (extract, assemble, info, validate)
    chapter.rs     # ChapterCommand (list, extract, add, remove, reorder)
    metadata.rs    # MetadataCommand (show, set, remove, import, export)
    toc.rs         # TocCommand (show, set, generate)
    spine.rs       # SpineCommand (list, reorder, set)
    asset.rs       # AssetCommand (list, extract, extract-all, add, remove)
    content.rs     # ContentCommand (search, replace, headings)
    output.rs      # OutputConfig (JSON, table, TTY detection)
  epub/            # EPUB domain model + I/O
    mod.rs         # EpubBook, EpubMetadata, ManifestItem, SpineItem, Navigation, NavPoint
    reader.rs      # read_epub() orchestrator
    writer.rs      # write_epub() with OPF/nav generation
    container.rs   # META-INF/container.xml parser
    opf.rs         # OPF parser (metadata, manifest, spine)
    navigation.rs  # nav.xhtml + NCX parser
    zip_utils.rs   # ZIP open, read, validate, list
  extract/         # EPUB -> Markdown
    mod.rs         # extract_book() orchestrator
    html_to_md.rs  # XHTML->Markdown with pre/post-processing
    frontmatter.rs # metadata.yml + chapter YAML headers
    summary.rs     # SUMMARY.md generation from NavPoints
    chapter_org.rs # Chapter filename generation (index+slug)
    asset_extract.rs # Asset path mapping + extraction
  assemble/        # Markdown -> EPUB
    mod.rs         # assemble_book() orchestrator
    package.rs     # package_epub() entry point
    md_to_xhtml.rs # Markdown->XHTML via pulldown-cmark
    metadata_build.rs # metadata.yml -> EpubMetadata
    spine_build.rs # SUMMARY.md -> spine order + navigation
    asset_embed.rs # Media type inference
  manipulate/      # In-place EPUB editing
    mod.rs         # Module re-exports
    meta_edit.rs   # modify_epub() + metadata set/remove/import/export
    chapter_manage.rs # add/remove/reorder chapters
    toc_edit.rs    # TOC set/generate, spine reorder/set
    content_edit.rs # search, replace, headings, restructure
    asset_manage.rs # add/remove assets
  util.rs          # Shared utilities: strip_html_tags, find_resource_key, build_nav_tree, format_iso8601, format_iso8601_date
```

Total: ~6,800 lines of Rust across 38 source files and 9 test files.

## Non-Functional Requirements

| ID | Requirement | Criteria | Status |
|:---|:---|:---|:---|
| NFR-001 | Single binary with no runtime dependencies | `cargo build --release` produces a single binary with no runtime dependencies; no pandoc, Python, or external tools required | Active |
| NFR-002 | Atomic file writes for all EPUB modifications | All write operations use `.epub.tmp` intermediate file with `std::fs::rename` for atomic commit; no partial writes on failure | Active |
| NFR-003 | EPUB version compatibility | Reader must parse both EPUB 2.x and 3.x files; writer always produces EPUB 3.3 with NCX fallback for backward compat | Active |
| NFR-004 | Cross-platform support | CI runs on ubuntu-latest and macos-latest; release builds target 4 platform triples | Active |
| NFR-005 | Scriptable output | All read-only commands support `--json` flag for machine-parseable output; tables auto-detect TTY vs pipe | Active |
| NFR-006 | Round-trip fidelity | `extract` then `assemble` produces a structurally valid EPUB 3.3 (verified by integration tests) | Active |
| NFR-007 | Content safety during replacement | `content replace` modifies only text nodes between HTML tags; tag attributes and element names are never modified | Active |

## Design Decisions

### DD-001: Noun-Verb CLI Pattern
- **Context:** The tool needs an intuitive command structure for managing multiple EPUB concerns (books, chapters, metadata, TOC, spine, assets, content).
- **Decision:** Use a two-level noun-verb pattern: `epx <resource> <action>` (e.g., `epx chapter list`, `epx metadata set`), modeled after GitHub CLI (`gh`).
- **Rationale:** Noun-verb is more discoverable than flat commands. Users can run `epx <resource> --help` to see all available actions. 7 resource nouns map naturally to EPUB concepts.
- **Consequences:** Requires clap nested subcommands (enum-of-enums). All global flags (--json, --verbose, --quiet, --no-color) must be propagated via `global = true`.
- **Related:** REQ-011

### DD-002: EpubBook In-Memory Domain Model
- **Context:** Multiple operations (read, extract, manipulate, write) need to share a consistent representation of an EPUB.
- **Decision:** All EPUB data is loaded into a single `EpubBook` struct containing `EpubMetadata`, `Vec<ManifestItem>`, `Vec<SpineItem>`, `Navigation`, and `HashMap<String, Vec<u8>>` for all resources (including binary data).
- **Rationale:** Loading everything into memory simplifies the programming model -- any operation can access any part of the EPUB. EPUB files are typically small (< 100 MB), so memory is not a concern.
- **Consequences:** Very large EPUBs (e.g., multi-GB audiobook EPUBs) may consume significant memory. The `resources` HashMap holds all file content as `Vec<u8>`, including potentially large images and media.
- **Related:** REQ-001, NFR-003

### DD-003: Read-Modify-Write Pattern for Manipulation
- **Context:** All manipulation commands (metadata set, chapter add, asset remove, etc.) need to modify EPUBs safely.
- **Decision:** Provide a `modify_epub()` helper that reads the EPUB into `EpubBook`, applies a closure for modification, then writes back atomically via temporary file + rename.
- **Rationale:** This ensures the original file is never corrupted by partial writes. The closure pattern keeps manipulation logic clean and testable.
- **Consequences:** Every modification re-reads and re-writes the entire EPUB. This is acceptable for the expected use case (manual editing workflows, not batch processing of thousands of files).
- **Related:** REQ-014, NFR-002

### DD-004: Opinionated Extraction Directory Structure
- **Context:** Extracted EPUB content needs a predictable, editor-friendly layout.
- **Decision:** Extract to a fixed directory structure: `metadata.yml` at root, `SUMMARY.md` for TOC/spine, `chapters/NN-slug.md` for content, `assets/images/` for images, `assets/fonts/` for fonts, `styles/` for CSS. Each chapter gets YAML frontmatter with `original_file`, `original_id`, and `spine_index`.
- **Rationale:** This mirrors conventions from mdBook and other Markdown-based book tools. The SUMMARY.md format allows round-trip: it defines both navigation and spine order during assembly.
- **Consequences:** Some EPUB structure is lost during extraction (exact directory layout, non-standard metadata). The opinionated format means EPUBs re-assembled from extracted content may differ structurally from originals while remaining semantically equivalent.
- **Related:** REQ-002, REQ-003

### DD-005: EPUB 3 Navigation with NCX Fallback
- **Context:** Reading needs to handle both EPUB 2 (NCX) and EPUB 3 (nav.xhtml) navigation. Writing needs maximum compatibility.
- **Decision:** Reader prefers EPUB 3 nav.xhtml (identified by `properties="nav"` in manifest), falls back to NCX. Writer always generates both `toc.xhtml` (EPUB 3 nav) and `toc.ncx` (EPUB 2 compat).
- **Rationale:** Most modern readers support EPUB 3 nav, but generating NCX as well ensures backward compatibility with older e-readers.
- **Consequences:** Navigation generation is duplicated across two formats. The writer must maintain consistency between nav.xhtml and toc.ncx.
- **Related:** REQ-006, NFR-003

### DD-006: Text-Node-Only Content Replacement
- **Context:** `content replace` must modify EPUB XHTML content without corrupting HTML structure.
- **Decision:** The replace function walks XHTML character by character, identifying text nodes (content between `>` and `<`), and only applies regex replacement within those text nodes. Tag names and attribute values are never modified.
- **Rationale:** A simple approach that avoids needing a full DOM parser. Prevents accidental modification of CSS class names, href attributes, or HTML structure.
- **Consequences:** Cannot replace text that spans across element boundaries. The character-level parsing is simplistic and could be confused by edge cases (e.g., `<` in attributes).
- **Related:** REQ-009, NFR-007

### DD-007: Error Handling Strategy
- **Context:** The tool needs clear error messages for CLI users while also supporting library-style error propagation.
- **Decision:** Two-tier error handling: `EpxError` (thiserror) for domain-specific errors in the `epub` module with typed variants (InvalidEpub, Xml, Zip, Io, Json, Yaml, Regex), and `anyhow::Result` for all higher-level operations (extract, assemble, manipulate, CLI).
- **Rationale:** `thiserror` enables pattern matching on error types within the epub layer. `anyhow` provides context chaining (`.with_context()`) for user-facing error messages without boilerplate.
- **Consequences:** The two error types require conversion at module boundaries. All `EpxError` variants wrap `From` impls for standard library errors (io, xml, zip, json, yaml, regex).
- **Related:** REQ-011
