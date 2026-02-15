# Plan 01: epx — EPUB CLI Tool

**Status:** Draft
**Date:** 2026-02-09

## Overview

Build `epx`, a self-contained Rust CLI tool for extracting, manipulating, and assembling EPUB files. The primary flow is EPUB → opinionated Markdown+assets, with reverse assembly and in-place manipulation. Greenfield project — no existing Rust code.

**Key decisions:**
- Noun-verb CLI syntax (like `gh`)
- Pure Rust, single binary, no external dependencies (no pandoc)
- Full manipulation: metadata, TOC, spine, AND content editing
- macOS + Linux; Homebrew tap + cargo install
- Extract any EPUB version; assemble EPUB 3.3

## Implementation Sequence

### Phase 1: Project Bootstrap & CLI Framework
- Cargo init, project structure, all module stubs
- clap derive CLI with noun-verb command tree
- Error types (thiserror), output formatting (TTY/JSON/table)
- Test fixtures (sample EPUB 2 & 3 files)
- Completion: `cargo build` succeeds, all `--help` commands work, `epx book info` returns stub

### Phase 2: EPUB Reading Layer
- Domain model (EpubBook, EpubMetadata, Navigation, etc.)
- ZIP/container/OPF/navigation parsing pipeline
- Reader orchestrator
- Completion: `epx book info`, `chapter list`, `metadata show`, `toc show`, `asset list` work on real EPUBs

### Phase 3: Extraction (EPUB → Markdown)
- XHTML → Markdown conversion with EPUB-specific pre/post-processing
- Metadata → YAML frontmatter
- Chapter naming/sequencing, SUMMARY.md generation
- Asset extraction with path rewriting
- Completion: `epx book extract` produces full directory structure, all links/images correct

### Phase 4: Assembly (Markdown → EPUB)
- Markdown → XHTML conversion
- Metadata YAML → OPF Dublin Core
- SUMMARY.md → spine/nav
- EPUB 3.3 ZIP packaging
- Piecemeal assembly (add chapters/assets to existing EPUB)
- Completion: `epx book assemble` produces valid EPUB 3.3, round-trip works

### Phase 5: Manipulation
- Metadata set/remove/import/export
- TOC/spine reordering, auto-generation
- Content search/replace/headings
- Chapter add/remove, asset add/remove
- Atomic writes for all modifications
- Completion: All manipulation commands produce valid EPUB

### Phase 6: Distribution
- cargo-dist configuration for macOS + Linux
- Homebrew tap formula
- CI/CD (fmt, clippy, test on push; cargo-dist on tag)
- Completion: `cargo install epx` and `brew install` work

## Completion Criteria

- [ ] Phase 1: `cargo build` succeeds, all `--help` commands work, test fixtures committed
- [ ] Phase 2: EPUB reading works on EPUB 2 and 3 files
- [ ] Phase 3: Full extraction produces well-formed Markdown with correct links/images
- [ ] Phase 4: Assembly produces valid EPUB 3.3, round-trip extraction→assembly works
- [ ] Phase 5: All manipulation commands produce valid EPUB
- [ ] Phase 6: Binary distribution via cargo install, Homebrew, and GitHub releases
