# Deep Regression Tests: _resources/ EPUB Extraction

## Context

The `_resources/` directory contains 5 real-world EPUB files (gitignored) used for manual quality review. We need integration tests that extract these into a persistent `_books/` directory (also gitignored) so the operator can inspect the markdown output. Tests are `#[ignore]` so they never run in CI — only when explicitly invoked.

## Files to Modify

| File | Action |
|------|--------|
| `tests/deep_regression_test.rs` | **Create** — 5 extraction tests + 5 roundtrip tests |
| `.gitignore` | **Edit** — add `_books/` |

## Design

### Test structure: one test per EPUB, not a loop
- Isolation: if one book fails, others still run and produce output
- Clear names in output: `deep_extract_leviathan_wakes ... ok`

### 5 extraction tests (`deep_extract_*`)
Each test:
1. Resolves EPUB from `_resources/<name>.epub`
2. Cleans and creates `_books/<name>/`
3. Runs `epx book extract <file> -o <output_dir>`
4. Asserts: command succeeds, `chapters/` has >= 1 `.md` file, `metadata.yml` exists and non-empty, `SUMMARY.md` exists

### 5 roundtrip tests (`deep_roundtrip_*`)
Each test:
1. Extracts to `_books/<name>-rt/`
2. Assembles to `_books/<name>-roundtrip.epub`
3. Validates EPUB structure via `common::assert_valid_epub()`

### Helpers (local to test file, not in common/)
- `resource_path(name)` — resolves `_resources/<name>`, panics with clear message if missing
- `books_output_root()` — returns `_books/`, creates if needed
- `book_output_dir(stem)` — returns `_books/<stem>/`, cleans prior content for fresh output
- `assert_extraction_structure(dir)` — structural checks on extraction output

### Books
| Stem | File | Size |
|------|------|------|
| leviathan-wakes | leviathan-wakes.epub | 770 KB |
| talisman | talisman.epub | 1.0 MB |
| the-blessing-way | the-blessing-way.epub | 426 KB |
| the-silence-of-animals | the-silence-of-animals.epub | 166 KB |
| wizard-of-earthsea | wizard-of-earthsea.epub | 4.8 MB |

## Operator Usage

```bash
# All deep tests (extract + roundtrip)
cargo test deep_ -- --ignored

# Extract only (for review)
cargo test deep_extract_ -- --ignored

# Roundtrip only (structural validation)
cargo test deep_roundtrip_ -- --ignored

# Single book
cargo test deep_extract_wizard -- --ignored
```

## Verification

1. `cargo test` — confirm ignored tests don't run (0 filtered)
2. `cargo test deep_extract_ -- --ignored` — all 5 extractions succeed
3. Inspect `_books/` — each subdirectory has `chapters/`, `metadata.yml`, `SUMMARY.md`
4. `cargo test deep_roundtrip_ -- --ignored` — all 5 roundtrips produce valid EPUBs
5. `git status` — confirm `_books/` does not appear (gitignored)
