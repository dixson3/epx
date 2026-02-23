# epx

A command-line tool for extracting, manipulating, and assembling EPUB files.

## Features

- **EPUB 2 and 3 support** -- reads both EPUB 2.x and 3.x files; writes EPUB 3.3 with NCX fallback for backward compatibility
- **Extract to Markdown** -- converts EPUB chapters to Markdown with YAML frontmatter, preserving navigation structure via SUMMARY.md
- **Assemble from Markdown** -- builds a valid EPUB from a directory of Markdown files, metadata.yml, and assets
- **Round-trip fidelity** -- extract then assemble produces a structurally valid EPUB
- **In-place editing** -- modify metadata, chapters, TOC, spine, assets, and content without re-extracting
- **Atomic writes** -- all modifications use temp-file-and-rename to prevent corruption
- **Scriptable output** -- all read-only commands support `--json` for machine-parseable output; tables auto-detect TTY vs pipe
- **Noun-verb CLI** -- intuitive `epx <resource> <action>` pattern (7 resource groups, 28 commands)

## Installation

### From source (requires Rust toolchain)

```sh
cargo install --path .
```

Or clone and build:

```sh
git clone https://github.com/dixson3/epx.git
cd epx
cargo build --release
# Binary is at target/release/epx
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/dixson3/epx/releases) for:
- macOS (Apple Silicon and Intel)
- Linux (x86_64 and aarch64)

## Usage

epx uses a noun-verb pattern: `epx <resource> <action>`. Global flags include `--json`, `--verbose`, `--quiet`, and `--no-color`.

### book -- whole-book operations

```sh
# Extract an EPUB to a Markdown directory
epx book extract book.epub -o ./extracted

# Assemble a Markdown directory into an EPUB
epx book assemble ./extracted -o rebuilt.epub

# Show EPUB info (title, author, chapter count, etc.)
epx book info book.epub

# Validate EPUB structure
epx book validate book.epub
```

### chapter -- chapter operations

```sh
# List all chapters
epx chapter list book.epub

# Extract a single chapter to Markdown
epx chapter extract book.epub chap01 -o chapter1.md

# Add a Markdown file as a new chapter
epx chapter add book.epub new-chapter.md --title "New Chapter"

# Remove a chapter by ID
epx chapter remove book.epub chap03

# Reorder a chapter (move from position 2 to position 0)
epx chapter reorder book.epub 2 0
```

### metadata -- metadata operations

```sh
# Show all metadata
epx metadata show book.epub

# Show metadata as JSON
epx metadata show book.epub --json

# Set a metadata field
epx metadata set book.epub --field title --value "New Title"

# Remove a metadata field
epx metadata remove book.epub --field description

# Export metadata to YAML
epx metadata export book.epub -o metadata.yml

# Import metadata from YAML
epx metadata import book.epub metadata.yml
```

### toc -- table of contents

```sh
# Show the table of contents
epx toc show book.epub

# Set TOC from a Markdown file
epx toc set book.epub toc.md

# Generate TOC from chapter headings
epx toc generate book.epub
```

### spine -- reading order

```sh
# List spine items
epx spine list book.epub

# Reorder a spine item (move from position 1 to position 0)
epx spine reorder book.epub 1 0

# Set spine order from a YAML file
epx spine set book.epub spine.yml
```

### asset -- images, fonts, and stylesheets

```sh
# List all assets
epx asset list book.epub

# Extract a single asset
epx asset extract book.epub cover.jpg -o cover.jpg

# Extract all assets to a directory
epx asset extract-all book.epub -o ./assets

# Add an asset
epx asset add book.epub logo.png

# Remove an asset
epx asset remove book.epub old-image.jpg
```

### content -- search, replace, and headings

```sh
# Search for text across all chapters
epx content search book.epub "Chapter"

# Replace text (with regex support)
epx content replace book.epub "colour" "color"

# Dry-run replacement (show matches without modifying)
epx content replace book.epub "colour" "color" --dry-run

# List all headings
epx content headings book.epub

# Restructure heading levels (e.g., shift h2 -> h1)
epx content headings book.epub --restructure "h2->h1,h3->h2"
```

## License

MIT License. Copyright (c) 2026 James Dixson.
