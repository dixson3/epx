# Reference-Aware Anchor Preservation & Link Validation

## Context

EPUB extraction currently preserves `id` attributes from all anchor elements and select content elements (`p`, `h1-h6`). This produces thousands of orphaned anchors from publisher toolchains (Calibre `a:XX`, `calibre_pb_N`, `page_N`, UUID-like reading-position markers like `id_8yJFzb-ORGCBlTwHr56MZw994`) that clutter the markdown without serving any navigational purpose. Cross-chapter links and fragments are correctly rewritten, but unreferenced anchors add noise.

Goals:
1. Only preserve IDs that are actual targets of fragment links within the EPUB
2. Post-extraction validation confirming all markdown links resolve correctly
3. Re-extract all books after fixing

## Current State (audit of 7 books)

| Book | Fragment Refs | Referenced Anchors | Orphaned Anchors | Pattern |
|------|-------------|-------------------|------------------|---------|
| c-programming | 16,903 | 6,114 | 1,107 | `a:XX` calibre |
| leviathan-wakes | 137 | 134 | 139 | `a:XX` calibre |
| talisman | 0 | 0 | 168 | `page_N` |
| the-blessing-way | 26 | 26 | 0 | clean |
| the-silence-of-animals | 26 | 26 | 0 | UUID-like (referenced from TOC) |
| william-shakespeare | 215 | 215 | 208 | `calibre_*` |
| wizard-of-earthsea | 0 | 0 | 0 | clean |

## Files to Modify

| File | Action |
|------|--------|
| `src/extract/html_to_md.rs` | Add `referenced_ids` parameter; only preserve IDs in the set |
| `src/extract/mod.rs` | Add reference-scan pass; pass referenced IDs through pipeline; add post-extraction link validation |

No new files — both new functions (reference collection and link validation) are small enough to live in `mod.rs`.

## Implementation

### 1. Reference collection in `extract_book()` — new `collect_referenced_ids()`

Add to `src/extract/mod.rs`:

```rust
fn collect_referenced_ids(book: &EpubBook, opf_dir: &str) -> HashSet<String> {
    // Iterate ALL spine XHTML files
    // Read each from book.resources
    // Regex-scan for href="...#fragment" — extract fragment portion
    // Also handles same-file refs: href="#fragment"
    // Returns a single HashSet<String> of all referenced fragment IDs
}
```

Called in `extract_book()` between Pass 1 (chapter filename collection) and `build_path_map()`.

### 2. Modify `xhtml_to_markdown()` signature

```rust
pub fn xhtml_to_markdown(
    xhtml: &str,
    path_map: &HashMap<String, String>,
    referenced_ids: &HashSet<String>,
) -> String
```

Thread `referenced_ids` into `preprocess_xhtml()`.

### 3. Filter ID preservation by reference set

In all three ID preservation steps (1a: empty anchors, 1b: content anchors, 2: element IDs):
- If `referenced_ids` is empty: preserve nothing (no filtering, no anchors)
- If non-empty: check if the extracted `id` is in `referenced_ids`
  - Yes → inject placeholder as before
  - No → for empty anchors, drop entirely; for element/content anchors, strip the `id` attribute only

### 4. Update `extract_single_chapter()` call site

Pass `&HashSet::new()` — single-chapter extraction doesn't have full-book context, so no anchors are preserved (consistent with current behavior where it passes `&[]` for chapter_files).

### 5. Post-extraction link validation

New function in `mod.rs`:

```rust
fn validate_extraction_links(output_dir: &Path) -> Vec<String> {
    // Scan all .md files in chapters/
    // Collect all <a id="..."> anchors per file
    // Collect all ](file.md#fragment) and ](#fragment) references
    // Cross-check: report dangling references (fragment target missing)
    // Also check: report file references to non-existent .md files
    // Return vec of warning strings
}
```

Called at the end of `extract_book()`. Warnings emitted via `eprintln!`.

### 6. Update existing tests

- Unit tests in `html_to_md.rs` pass `&HashSet::new()` (empty set = no anchors preserved = keeps current test expectations for tests that don't assert on anchors)
- Anchor-specific tests (`test_anchor_id_preservation`, `test_multiple_anchor_ids`, etc.) pass a set containing the expected IDs
- Add test for `collect_referenced_ids` (integration-level, using test fixtures)
- Add test for validation function

## Verification

1. `cargo fmt && cargo clippy -- -D warnings` — clean
2. `cargo test` — all unit/integration tests pass
3. `rm -rf _books/ && cargo test deep_extract_all -- --ignored` — all 7 books extract
4. Audit confirms:
   - 0 orphaned anchors across all books
   - 0 dangling fragment references
   - Referenced anchor counts match: ~6,515 total across all books
5. Spot-check: `the-silence-of-animals/chapters/05-1-an-old-chaos.md` has `id_8yJFzb-*` anchors only if referenced by the extracted TOC chapter
6. `cargo test deep_roundtrip_all -- --ignored` — roundtrip tests pass
