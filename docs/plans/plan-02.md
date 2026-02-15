# Plan 02: Compliance Remediation & Workflow Restoration

**Status:** Completed
**Date:** 2026-02-14

## Overview

A full engineer compliance audit (synthesizer agent `a67a649`) compared the PRD, EDD, IGs, and TODO specs against the actual implementation and test suite. It found **4 PRD gaps, 5 EDD gaps, 10 IG coverage gaps, 9 test coverage gaps, and 12 new findings**. Additionally, the Yoshiko Flow plugin infrastructure has broken symlinks from a marketplace rename that must be fixed to restore auto-chain and swarm lifecycle.

This plan is organized into 6 phases. Phases 0-2 are independent and can be swarmed in parallel. Phase 3 depends on Phase 2. Phase 4 depends on all prior phases. Phase 5 is independent.

## Implementation Sequence

### Phase 0: Restore Yoshiko Flow Workflow (infrastructure) — COMPLETED

Fix broken yf symlinks and state from the `yoshiko-studios-marketplace` → `d3-claude-plugins` rename.

- 0a. Remove dangling symlinks and stale state — DONE
- 0b. Run preflight to regenerate rules — DONE
- 0c. Verified: 24 rules installed in `.claude/rules/yf/` pointing to `d3-claude-marketplace` cache

---

### Phase 1: Fix Custom Metadata Round-Trip in Assembly (critical bug)

**Problem:** Phase 1 of the previous plan fixed OPF read/write for custom metadata, but the assembly path discards it. `metadata_build::read_metadata()` sets `custom: Default::default()`, so custom meta properties (e.g., `rendition:layout`) are lost during extract→assemble.

#### 1a. Add `custom` field to `BookMetadataYaml`

**File:** `src/extract/frontmatter.rs`
- Add `#[serde(default)] pub custom: HashMap<String, String>` to `BookMetadataYaml`
- In `from_epub_metadata()`: copy `metadata.custom` into the yaml struct
- In `to_yaml()`: serialize the custom map

#### 1b. Wire custom metadata through assembly

**File:** `src/assemble/metadata_build.rs`
- In `read_metadata()`: populate `custom` from the yaml struct instead of `Default::default()`

#### 1c. Add round-trip integration test

**File:** `tests/roundtrip_test.rs`
- `test_roundtrip_preserves_custom_metadata` — set custom field on EPUB, extract, assemble, verify field survives

---

### Phase 2: Code Deduplication (~150 lines)

**Problem:** 5 utility functions are duplicated across modules, violating DRY and the spirit of resolved TODO-001.

#### 2a. Create shared utility module

**File:** `src/util.rs` (new)
- Extract `strip_html_tags()` (from 3 copies: `html_to_md.rs`, `toc_edit.rs`, `content_edit.rs`)
- Extract `find_resource_key()` (from 2 copies: `content_edit.rs`, `toc_edit.rs`)
- Extract `build_nav_tree()` (from 2 copies: `spine_build.rs`, `toc_edit.rs`)
- Unify `now_iso8601()` / `chrono_free_date()` into one `format_iso8601()` function (from `writer.rs`, `frontmatter.rs`)

#### 2b. Update `src/lib.rs` or `src/main.rs`
- Add `pub mod util;`

#### 2c. Replace all call sites
- `src/extract/html_to_md.rs` → `use crate::util::strip_html_tags`
- `src/manipulate/toc_edit.rs` → `use crate::util::{strip_html_tags, find_resource_key, build_nav_tree}`
- `src/manipulate/content_edit.rs` → `use crate::util::{strip_html_tags, find_resource_key}`
- `src/assemble/spine_build.rs` → `use crate::util::build_nav_tree`
- `src/epub/writer.rs` → `use crate::util::format_iso8601`
- `src/extract/frontmatter.rs` → `use crate::util::format_iso8601`
- Delete all local copies of these functions

#### 2d. Refactor `content headings --restructure` to use `modify_epub()`

**File:** `src/main.rs` (lines ~553-559)
- Replace direct read→modify→write with `modify_epub(&file, |book| { restructure_headings(book, &mappings) })`
- Fixes DD-003 pattern violation

---

### Phase 3: Cleanup Dead Code & Unused Dependencies

**Depends on:** Phase 2 (dedup must land first to avoid conflicts)

#### 3a. Remove unused `scraper` dependency

**File:** `Cargo.toml`
- Delete `scraper = "0.25"` from `[dependencies]`
- Verify no imports exist (synthesizer confirmed none)

#### 3b. Implement `--verbose`/`--quiet` flags

**File:** `src/cli/output.rs`, `src/main.rs`
- Add `if output.verbose { ... }` for extra detail in extraction/info commands
- Add `if output.quiet { ... }` to suppress informational println messages
- These flags are documented in the PRD (REQ-012)

#### 3c. Remove dead `EpxError` variants

**File:** `src/error.rs`
- Remove unused variants: `ChapterNotFound`, `AssetNotFound`, `ConversionError`, `MetadataError`, `SpineError`, `NavigationError`
- Remove `#[allow(dead_code)]` from the enum
- Keep variants that ARE used: `Io`, `Xml`, `Zip`, `Json`, `Yaml`, `Regex`

---

### Phase 4: CLI Integration Test Coverage (7 command groups)

**Depends on:** Phases 1-3 (code changes must stabilize first)

#### 4a. Metadata import/export tests

**File:** `tests/metadata_test.rs`
- `test_metadata_export` — export to YAML, verify file contents
- `test_metadata_import` — export then import to different EPUB, verify fields match

#### 4b. TOC set/generate tests

**File:** `tests/toc_test.rs`
- `test_toc_generate` — run `toc generate`, verify output changes
- `test_toc_set` — write Markdown TOC file, apply with `toc set`, verify

#### 4c. Asset extract/add/remove tests

**File:** `tests/asset_test.rs`
- `test_asset_extract_single` — extract one asset, verify file written
- `test_asset_extract_all` — extract all assets, verify directory structure
- `test_asset_add` — add a new asset file, verify in manifest
- `test_asset_remove` — remove asset, verify gone from manifest

#### 4d. Content replace and headings tests

**File:** `tests/content_test.rs`
- `test_content_replace_actual` — non-dry-run replace, verify text changed
- `test_content_search_regex` — search with `--regex` flag
- `test_content_headings_restructure` — apply heading remapping, verify

#### 4e. Spine reorder test

**File:** `tests/spine_test.rs`
- `test_spine_reorder` — reorder by index, verify new order

#### 4f. Complex fixture tests

**File:** `tests/roundtrip_test.rs`
- `test_roundtrip_childrens_literature` — exercise the unused fixture
- `test_roundtrip_accessible_epub3` — exercise the unused fixture

---

### Phase 5: Update Specifications & Documentation

**Independent — can run in parallel with any phase**

#### 5a. Add new TODO items

**File:** `docs/specifications/TODO.md`
- Add TODO-013 through TODO-019 (as identified by compliance audit)
- Mark items resolved as phases complete

#### 5b. Update EDD

**File:** `docs/specifications/EDD/CORE.md`
- Fix line count: ~6,710 → ~9,500+
- Document `src/util.rs` shared utilities module
- Fix NFR-001 wording: "single binary, no runtime dependencies" (not "statically-linked")

#### 5c. Expand README.md

**File:** `README.md`
- Add installation instructions (cargo install, binary download)
- Add usage examples for each command group
- Add feature highlights

#### 5d. Fix cargo-dist configuration mismatch

**File:** `Cargo.toml`
- Remove `[workspace.metadata.dist]` section for now (TODO-019)
- Address properly when Homebrew is ready

---

## Swarm Strategy

| Phase | Parallelizable | Agent Type | Notes |
|:---|:---|:---|:---|
| 0 | Independent | Bash | COMPLETED |
| 1 | Independent | code_writer | Critical bug fix |
| 2 | Independent | code_writer | Refactoring |
| 3 | After Phase 2 | code_writer | Cleanup |
| 4 | After Phases 1-3 | code_tester | Tests only |
| 5 | Independent | code_writer | Docs/specs |

Phases 1, 2, and 5 can all run concurrently. Phase 3 gates on Phase 2. Phase 4 gates on Phases 1-3.

## Completion Criteria

- [ ] All custom metadata survives extract→assemble round-trip
- [ ] Zero duplicated utility functions across modules
- [ ] `cargo test` passes with no failures
- [ ] `cargo clippy` reports no warnings
- [ ] `--verbose` and `--quiet` flags functional
- [ ] All 7 command groups have integration tests
- [ ] Specifications updated to match implementation
- [ ] No unused dependencies in Cargo.toml

## Files Modified

| File | Phase | Change |
|:---|:---|:---|
| `.claude/rules/yf/` | 0 | Fixed symlinks (DONE) |
| `src/extract/frontmatter.rs` | 1a | Add custom field to BookMetadataYaml |
| `src/assemble/metadata_build.rs` | 1b | Wire custom metadata through assembly |
| `tests/roundtrip_test.rs` | 1c, 4f | Custom metadata roundtrip + fixture tests |
| `src/util.rs` | 2a | New shared utilities module |
| `src/lib.rs` | 2b | Register util module |
| `src/extract/html_to_md.rs` | 2c | Use shared strip_html_tags |
| `src/manipulate/toc_edit.rs` | 2c | Use shared utilities |
| `src/manipulate/content_edit.rs` | 2c | Use shared utilities |
| `src/assemble/spine_build.rs` | 2c | Use shared build_nav_tree |
| `src/epub/writer.rs` | 2c | Use shared format_iso8601 |
| `src/extract/frontmatter.rs` | 2c | Use shared format_iso8601 |
| `src/main.rs` | 2d | Refactor headings --restructure |
| `Cargo.toml` | 3a, 5d | Remove scraper dep + dist metadata |
| `src/cli/output.rs` | 3b | Implement verbose/quiet |
| `src/main.rs` | 3b | Wire verbose/quiet into handlers |
| `src/error.rs` | 3c | Remove dead EpxError variants |
| `tests/metadata_test.rs` | 4a | Import/export tests |
| `tests/toc_test.rs` | 4b | Set/generate tests |
| `tests/asset_test.rs` | 4c | Extract/add/remove tests |
| `tests/content_test.rs` | 4d | Replace/regex/restructure tests |
| `tests/spine_test.rs` | 4e | Reorder test |
| `docs/specifications/TODO.md` | 5a | Add TODO-013 through TODO-019 |
| `docs/specifications/EDD/CORE.md` | 5b | Fix line count, add util module, fix NFR-001 |
| `README.md` | 5c | Expand with install/usage/features |
