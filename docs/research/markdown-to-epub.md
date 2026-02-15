# Strategy for Mapping Markdown Folder with Assets to EPUB

## Executive Summary

Converting a markdown folder with assets to EPUB is a well-established workflow with multiple mature tooling options. The key decisions involve folder structure conventions, asset organization, metadata mapping, and tool selection. **Pandoc** emerges as the most flexible and widely-adopted solution, with **HonKit** as a strong alternative for book-oriented projects.

---

## 1. Folder Structure Conventions

### Recommended Structure

```
my-book/
├── metadata.yml          # Book metadata (title, author, etc.)
├── SUMMARY.md            # Table of contents / chapter ordering (HonKit style)
├── chapters/             # Markdown content
│   ├── 00-preface.md
│   ├── 01-introduction.md
│   ├── 02-chapter-one.md
│   └── ...
├── images/               # All image assets
│   ├── cover.png
│   ├── figure-01.png
│   └── ...
├── styles/               # CSS stylesheets
│   └── epub.css
└── build/                # Generated output
    └── book.epub
```

### Key Conventions

| Convention | Description | Confidence |
|------------|-------------|------------|
| One chapter per file | Each markdown file = one chapter, using H1 (`#`) for chapter title | High |
| Numbered prefixes | Files ordered with numeric prefixes (`01-`, `02-`) for predictable ordering | High |
| Dedicated assets folder | All images in `images/` or `assets/` directory | High |
| Separate metadata file | YAML metadata in dedicated file or frontmatter block | High |

**Sources:**
- [Pandoc - Creating an ebook with pandoc](https://pandoc.org/epub.html)
- [wikiti/pandoc-book-template](https://github.com/wikiti/pandoc-book-template)
- [Opensource.com - Book to website and ePub](https://opensource.com/article/18/10/book-to-website-epub-using-pandoc)

---

## 2. Asset Organization

### Images

| Aspect | Best Practice | Notes |
|--------|--------------|-------|
| Location | Centralized `images/` folder | Relative paths from markdown files |
| Cover image | `cover.png` or `cover.jpg` in root or images folder | Recommended < 1000px width/height |
| References | Use relative paths (`images/figure-01.png`) | Pandoc embeds referenced images in EPUB |
| Formats | PNG, JPG, SVG (EPUB3) | EPUB2 limited to raster formats |

### Stylesheets

- Place CSS in `styles/epub.css` or `assets/epub.css`
- Reference via metadata: `stylesheet: styles/epub.css`
- Pandoc uses `epub.css` from user data directory as fallback

### Fonts (Optional)

- Custom fonts can be embedded via `--epub-embed-font` option
- Place in `fonts/` directory

**Sources:**
- [Pandoc User's Guide - EPUB options](https://pandoc.org/MANUAL.html)
- [Customizing pandoc for PDF and EPUB](https://learnbyexample.github.io/customizing-pandoc/)

---

## 3. Frontmatter to EPUB Metadata Mapping

### YAML Frontmatter Fields

```yaml
---
title: "Book Title"
subtitle: "Optional Subtitle"
author:
  - name: "Author Name"
    role: author
  - name: "Translator Name"
    role: translator
publisher: "Publisher Name"
date: 2026-01-15
lang: en-US
rights: "© 2026 Author Name. All rights reserved."
cover-image: images/cover.png
stylesheet: styles/epub.css
description: "Book description for metadata"
subject:
  - Fiction
  - Science Fiction
identifier:
  - scheme: ISBN
    text: 978-0-123456-78-9
belongs-to-collection: "Series Name"
group-position: 1
page-progression-direction: ltr
---
```

### Mapping to EPUB OPF

| YAML Field | OPF/Dublin Core Element | Notes |
|------------|------------------------|-------|
| `title` | `<dc:title>` | Supports `type: main/subtitle` |
| `author` | `<dc:creator>` | Supports `role` attribute |
| `publisher` | `<dc:publisher>` | |
| `date` | `<dc:date>` | ISO 8601 format |
| `lang` | `<dc:language>` | BCP 47 code |
| `rights` | `<dc:rights>` | Copyright statement |
| `identifier` | `<dc:identifier>` | ISBN, UUID, etc. |
| `description` | `<dc:description>` | Book blurb |
| `subject` | `<dc:subject>` | Categories/keywords |
| `cover-image` | `<meta name="cover">` | EPUB cover |

### Known Issues

- Title `type` attributes may be stripped in some Pandoc versions (see [Issue #3393](https://github.com/jgm/pandoc/issues/3393))
- Multiple creators need explicit role disambiguation for kindlegen compatibility

**Confidence:** High - Pandoc's YAML-to-OPF mapping is well-documented

**Sources:**
- [Pandoc EPUB Metadata](https://pandoc.org/demo/example33/11.1-epub-metadata.html)
- [YAML Frontmatter for Markdown](https://sushantvema.github.io/notes/yaml_frontmatter_for_markdown)

---

## 4. TOC Generation

### Automatic Generation (Recommended)

Pandoc automatically generates navigation from markdown headings:

```bash
pandoc --toc --toc-depth=3 -o book.epub chapters/*.md
```

| Option | Effect |
|--------|--------|
| `--toc` | Generate table of contents |
| `--toc-depth=N` | Include headings H1 through HN |

### Heading Hierarchy

| Markdown | Role in EPUB |
|----------|--------------|
| `# Title` | Chapter title (NCX navPoint / nav toc entry) |
| `## Section` | Section entry |
| `### Subsection` | Subsection entry |

### EPUB 3 Navigation

EPUB 3 requires a `nav` element with `epub:type="toc"`. Pandoc generates this automatically:

```html
<nav epub:type="toc" id="toc">
  <ol>
    <li><a href="chapter1.xhtml">Chapter 1</a></li>
    <li><a href="chapter2.xhtml">Chapter 2</a>
      <ol>
        <li><a href="chapter2.xhtml#section">Section</a></li>
      </ol>
    </li>
  </ol>
</nav>
```

### Manual TOC (HonKit Style)

For explicit control, use a `SUMMARY.md` file:

```markdown
# Summary

* [Introduction](chapters/00-intro.md)
* [Chapter 1](chapters/01-chapter.md)
  * [Section 1.1](chapters/01-chapter.md#section-11)
* [Chapter 2](chapters/02-chapter.md)
```

**Confidence:** High

**Sources:**
- [EPUB Knowledge - Table of Contents](https://epubknowledge.com/docs/toc/)
- [DAISY - TOC Best Practices](https://kb.daisy.org/publishing/docs/navigation/toc.html)

---

## 5. Multi-File Organization Strategies

### Strategy A: Flat File List (Pandoc)

List files explicitly in build command:

```bash
pandoc metadata.yml \
  chapters/00-preface.md \
  chapters/01-intro.md \
  chapters/02-chapter.md \
  -o build/book.epub
```

**Pros:** Explicit ordering, simple
**Cons:** Long command lines, manual maintenance

### Strategy B: Makefile Automation

```makefile
CHAPTERS := $(wildcard chapters/*.md)
METADATA := metadata.yml

book.epub: $(METADATA) $(CHAPTERS)
	pandoc --toc -o $@ $^
```

**Pros:** Automated, rebuilds on changes
**Cons:** Requires Make knowledge

### Strategy C: SUMMARY.md (HonKit/GitBook)

```
book/
├── SUMMARY.md      # Defines structure
├── README.md       # Book introduction
├── chapter1/
│   ├── README.md   # Chapter intro
│   └── section1.md
└── chapter2/
    └── README.md
```

**Pros:** Self-documenting structure, web+epub from same source
**Cons:** Requires HonKit toolchain

### Strategy D: Pandoc with Glob

```bash
pandoc metadata.yml chapters/*.md -o book.epub
```

**Pros:** Minimal configuration
**Cons:** Relies on alphabetical file ordering

**Confidence:** High

**Sources:**
- [HonKit - ebook generation](https://github.com/honkit/honkit/blob/master/docs/ebook.md)
- [mdBook](https://github.com/rust-lang/mdBook)

---

## 6. Tool Comparison

### Primary Tools

| Tool | Type | EPUB Support | Strengths | Weaknesses |
|------|------|--------------|-----------|------------|
| **Pandoc** | CLI converter | Native EPUB 2/3 | Flexible, extensive format support, active development | CLI complexity, no GUI |
| **HonKit** | Book framework | Native (via Calibre) | Book-oriented, plugin ecosystem, web+ebook output | Requires Calibre for EPUB |
| **Calibre** | Library manager | ebook-convert CLI | Format conversions, metadata editing | Heavy dependency, less MD-native |
| **mdBook** | Documentation | Via plugin | Rust ecosystem, fast | EPUB requires mdbook-epub plugin |

### Detailed Tool Analysis

#### Pandoc (Recommended)

```bash
# Basic conversion
pandoc -o book.epub chapters/*.md

# Full-featured
pandoc --toc --toc-depth=3 \
  --epub-cover-image=images/cover.png \
  --css=styles/epub.css \
  --metadata-file=metadata.yml \
  -o book.epub chapters/*.md
```

**Confidence:** Very High
**Ecosystem:** Mature, widely adopted, excellent documentation

#### HonKit

```bash
# Install
npm install -g honkit

# Generate EPUB (requires Calibre)
honkit epub ./ ./book.epub
```

**Confidence:** High
**Ecosystem:** GitBook-compatible, good for web+ebook dual output

#### Custom Python (mark2epub)

```python
# For programmatic EPUB generation
# github.com/AlexPof/mark2epub
```

**Confidence:** Medium
**Use case:** Custom pipelines, automation

### Hybrid Approaches

| Workflow | Tools | Use Case |
|----------|-------|----------|
| Markdown to EPUB to MOBI | Pandoc + Calibre | Kindle distribution |
| Markdown to Web + EPUB | HonKit | Documentation sites |
| CI/CD automation | Pandoc + Make | Automated publishing |

**Sources:**
- [Pandoc EPUB documentation](https://pandoc.org/epub.html)
- [HonKit GitHub](https://github.com/honkit/honkit)
- [mdbook-epub plugin](https://github.com/Michael-F-Bryan/mdbook-epub)
- [InfoWorld - Markdown documentation tools compared](https://www.infoworld.com/article/3526306/text-in-docs-out-popular-markdown-documentation-tools-compared.html)

---

## 7. Recommendations

### For Simple Projects

Use **Pandoc** with a flat structure:

```
project/
├── metadata.yml
├── content.md        # Single file or
├── chapters/*.md     # Multiple files
├── images/
└── styles/epub.css
```

Build: `pandoc --toc --metadata-file=metadata.yml -o book.epub chapters/*.md`

### For Book Projects

Use **Pandoc with Makefile** or **HonKit**:

- Makefile provides rebuild automation
- HonKit provides web preview + EPUB from same source

### For Documentation Projects

Consider **mdBook** with the mdbook-epub plugin for Rust-style documentation.

### Metadata Strategy

1. Use a dedicated `metadata.yml` file for clean separation
2. Include minimal frontmatter in chapter files (just title if needed)
3. Keep cover image and stylesheet paths in metadata file

### Asset Strategy

1. Centralize all images in `images/` folder
2. Use relative paths consistently
3. Optimize images before including (< 1000px for cover)
4. Use PNG for diagrams, JPG for photos

---

## 8. Confidence Ratings Summary

| Topic | Confidence | Notes |
|-------|------------|-------|
| Folder structure conventions | High | Well-established patterns |
| Pandoc YAML to OPF mapping | High | Official documentation |
| TOC generation | High | Standard feature |
| HonKit EPUB support | High | Requires Calibre dependency |
| mdBook EPUB support | Medium | Third-party plugin |
| Custom Python solutions | Medium | Less mainstream |
| Cross-platform compatibility | High | All tools work on macOS/Linux/Windows |

---

## Sources

### Official Documentation
- [Pandoc - Creating an ebook with pandoc](https://pandoc.org/epub.html)
- [Pandoc User's Guide](https://pandoc.org/MANUAL.html)
- [Pandoc EPUB Metadata](https://pandoc.org/demo/example33/11.1-epub-metadata.html)

### Tool Repositories
- [HonKit GitHub](https://github.com/honkit/honkit)
- [mdBook GitHub](https://github.com/rust-lang/mdBook)
- [mdbook-epub plugin](https://github.com/Michael-F-Bryan/mdbook-epub)
- [pandoc-book-template](https://github.com/wikiti/pandoc-book-template)
- [mark2epub](https://github.com/AlexPof/mark2epub)
- [Sigil EPUB Editor](https://github.com/Sigil-Ebook/Sigil)

### Tutorials and Guides
- [Opensource.com - Book to website and ePub using Pandoc](https://opensource.com/article/18/10/book-to-website-epub-using-pandoc)
- [Customizing pandoc for PDF and EPUB](https://learnbyexample.github.io/customizing-pandoc/)
- [Medium - Turn Markdown into an Ebook](https://medium.com/codex/turn-markdown-into-an-ebook-a-comprehensive-guide-c3627539665a)
- [Medium - Markdown formatting for ebooks](https://medium.com/@blittler/markdown-formatting-for-ebooks-including-kindle-kobo-and-nook-ee15a5992bc)

### Standards and Best Practices
- [EPUB Knowledge - Table of Contents](https://epubknowledge.com/docs/toc/)
- [DAISY - TOC Best Practices](https://kb.daisy.org/publishing/docs/navigation/toc.html)
- [InfoWorld - Markdown documentation tools compared](https://www.infoworld.com/article/3526306/text-in-docs-out-popular-markdown-documentation-tools-compared.html)

---

*Originally researched: 2026-02-01 | Imported: 2026-02-09*
