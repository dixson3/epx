# Self-Consistency Audit: epx Documentation vs Implementation

## Context

A full review of epx documentation (README, PRD, EDD, 4 IGs, TODO register) against the actual Rust implementation to identify inconsistencies, stale claims, and contradictions between documents. The goal is to bring all artifacts into alignment so any document can be trusted as a source of truth.

---

## Findings

### CRITICAL — Docs contradict each other or implementation

| # | Location | Issue | Fix |
|---|----------|-------|-----|
| C1 | README line 14 | Claims "30+ commands" but implementation has exactly **28** (4+5+5+3+3+5+3). PRD line 9 says "25+ subcommands" — also inconsistent with README. | README: "28 commands". PRD: "28 subcommands". |
| C2 | README lines 66-75 | CLI examples use `--id`, `--file`, `--position` named flags for chapter commands, but clap defines **positional** args: `epx chapter extract <file> <id>`, `epx chapter add <file> <markdown>`, `epx chapter reorder <file> <from> <to>`. Every chapter example is wrong. | Rewrite README chapter examples to match actual clap positionals. |
| C3 | README lines 97, 133, 139, 142 | `metadata import --file`, `asset extract --id`, `asset add --file`, `asset remove --id` — all use named flags that don't exist. Actual: `metadata import <file> <metadata>`, `asset extract <file> <asset_path>`, `asset add <file> <asset>`, `asset remove <file> <asset_path>`. | Rewrite README examples to match clap positionals. |
| C4 | README lines 107, 120, 123 | `toc set --file toc.md`, `spine reorder --id --position`, `spine set --file` use named flags. Actual: `toc set <file> <toc>`, `spine reorder <file> <from> <to>`, `spine set <file> <spine>`. | Rewrite to match clap positionals. |
| C5 | TODO-011 vs TODO-013 | TODO-011 (custom metadata OPF write) marked **Resolved**, but TODO-013 (same issue, different wording) is still **Open**. These are the same bug — contradictory status. | Close TODO-013 as duplicate of resolved TODO-011, or clarify if a distinct issue remains. |
| C6 | EDD line 9 | Claims `main.rs` is "584 lines" — actual is **693 lines**. | Update to 693. |
| C7 | EDD line 54 | Claims "~5,800 lines across 37 source files and 9 test files". Actual: **6,231 lines across 42 source files**. | Update to ~6,200 lines across 42 files. |
| C8 | EDD line 51 | `util.rs` described as "(planned) shared utilities" with "see TODO-014..016". But `util.rs` **already exists** and contains `strip_html_tags`, `find_resource_key`, `build_nav_tree`, `format_iso8601`, `format_iso8601_date`. The "(planned)" label and TODO references are stale. | Remove "(planned)" and TODO references; describe actual contents. |
| C9 | PRD line 24 | Lists `scraper 0.25` as a key dependency. **scraper is not in Cargo.toml** — it was apparently removed. | Remove scraper from PRD dependency list. |
| C10 | IG manipulation line 68 | Claims OPF dir detection "duplicated in extract/mod.rs, manipulate/chapter_manage.rs, and manipulate/asset_manage.rs". Per MEMORY.md and TODO-001 (Resolved), this was deduplicated into `EpubBook::detect_opf_dir()`. IG is stale. | Update to reference `EpubBook::detect_opf_dir()` shared method. |

### MODERATE — Stale or misleading

| # | Location | Issue | Fix |
|---|----------|-------|-----|
| M1 | PRD line 29 | References `workspace.metadata.dist` for cargo-dist distribution. TODO-019 notes this section was removed from Cargo.toml. PRD still references it as active. | Add note that cargo-dist config is pending (per TODO-019). |
| M2 | EDD DD-007 line 114 | Claims `EpxError` has typed variants "ChapterNotFound, AssetNotFound, etc." Need to verify these exist. TODO-018 says several variants are dead code. | Verify actual variants in `error.rs`; update DD-007 to match reality. |
| M3 | README line 161 | Shows `--restructure "h1:h2,h2:h3"` with colon separator. Clap definition (content.rs:42) and PRD line 75 use `->` arrow: `"h2->h1,h3->h2"`. | Fix README to use `->` separator. |
| M4 | PRD line 75 | Heading mapping described as `"hN->hM"`. This is correct per implementation — but README uses `"h1:h2"` (see M3). Cross-doc inconsistency. | Covered by M3 fix. |
| M5 | TODO-014, TODO-015, TODO-016 | All three claim dedup is needed for `strip_html_tags`, `find_resource_key`, `build_nav_tree`. Per TODO-001 resolved status and MEMORY.md, these were consolidated into `util.rs`. These TODOs may be stale. | Verify if duplicates still exist in source; close if resolved. |
| M6 | IG manipulation UC-009 line 10 | Shows syntax `epx metadata set <file> <field> <value>` (positional). Actual clap: `epx metadata set <file> --field NAME --value VALUE` (named flags). IG is wrong in the opposite direction of the README. | Fix to `epx metadata set <file> --field <name> --value <value>`. |

### LOW — Minor cosmetic

| # | Location | Issue | Fix |
|---|----------|-------|-----|
| L1 | README line 85 | `epx metadata show book.epub --json` — global flag placement after subcommand. Works with clap `global = true` but unconventional vs `epx --json metadata show`. | Optional: show canonical placement. |
| L2 | EDD DD-004 line 93 | Mentions `assets/images/`, `assets/fonts/`, `styles/` structure. Should verify this matches actual extraction output. | Minor — verify and align. |
| L3 | Cargo.toml | `workspace.metadata.dist` section removed per TODO-019, but PRD still references Homebrew tap `dixson3/homebrew-tap`. | Align with actual distribution status. |

---

## Recommended Changes

### Files to modify

| File | Changes |
|------|---------|
| `README.md` | Fix all CLI examples (C2, C3, C4, M3) to match actual clap arg definitions; fix command count to 28 (C1) |
| `docs/specifications/PRD.md` | Fix subcommand count to 28 (C1); remove scraper dep (C9); update cargo-dist note (M1) |
| `docs/specifications/EDD/CORE.md` | Fix main.rs line count (C6); fix total line/file counts (C7); fix util.rs description (C8); verify error variants (M2) |
| `docs/specifications/IG/epub-manipulation.md` | Fix stale dedup claim (C10); fix metadata set syntax (M6) |
| `docs/specifications/TODO.md` | Resolve TODO-013 as dup of TODO-011 (C5); evaluate TODO-014/015/016 staleness (M5) |

### Verification

1. After edits, run `cargo build --release` to confirm no code changes needed
2. Spot-check 3-4 README examples by running them against a test fixture:
   - `epx chapter list tests/fixtures/minimal-v3.epub`
   - `epx metadata set tests/fixtures/minimal-v3.epub --field title --value "Test"` (verify named-flag syntax)
   - `epx content headings tests/fixtures/basic-v3plus2.epub --restructure "h2->h1"`
3. Grep for any remaining `scraper` references
4. Verify `util.rs` contains the functions TODO-014/015/016 claim are duplicated
