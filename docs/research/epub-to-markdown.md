# EPUB to Markdown Conversion Strategy Research

## Executive Summary

This research evaluates strategies for converting EPUB files to Markdown with proper asset handling, structure preservation, and metadata extraction. The analysis covers existing tools, custom implementation approaches, and recommendations based on project requirements.

**Confidence Rating: HIGH** - Based on well-documented standards (EPUB 2/3, Dublin Core) and mature ecosystem of tools.

---

## 1. EPUB Structure Overview

### File Anatomy

An EPUB is a ZIP archive with standardized structure:

```
mimetype                    # Must be first, uncompressed
META-INF/
    container.xml           # Points to OPF file
OEBPS/                      # Common content directory
    content.opf             # Package document (manifest, spine, metadata)
    toc.ncx                 # EPUB 2 navigation (deprecated in EPUB 3)
    nav.xhtml               # EPUB 3 navigation document
    chapter1.xhtml          # Content documents
    styles/
        main.css
    images/
        cover.jpg
        figure1.png
```

### Key Components

| Component | Purpose | Conversion Relevance |
|-----------|---------|---------------------|
| `container.xml` | Bootstrap - locates OPF | Entry point for parsing |
| `content.opf` | Manifest, spine, metadata | Source for frontmatter + reading order |
| `toc.ncx` | EPUB 2 table of contents | Hierarchical navigation structure |
| `nav.xhtml` | EPUB 3 navigation | Human + machine readable TOC |
| XHTML documents | Chapter content | Convert to Markdown |
| CSS files | Styling | Usually discarded or selectively preserved |
| Images | Figures, covers | Extract to asset directory |

**Sources:**
- [Anatomy of an EPUB 3 file - EDRLab](https://www.edrlab.org/open-standards/anatomy-of-an-epub-3-file/)
- [EPUB Format Construction Guide - HXA7241](https://www.hxa.name/articles/content/epub-guide_hxa7241_2007.html)
- [Introduction to EPUB Files - APLN](https://apln.ca/introduction-to-epub-files/)

---

## 2. Metadata to Frontmatter Mapping

### Dublin Core Elements in EPUB

EPUBs use Dublin Core metadata in the OPF file. Required elements:

| Dublin Core | OPF Tag | YAML Frontmatter |
|-------------|---------|------------------|
| Title | `<dc:title>` | `title:` |
| Identifier | `<dc:identifier>` | `isbn:` or `identifier:` |
| Language | `<dc:language>` | `language:` |
| Creator | `<dc:creator>` | `author:` |
| Publisher | `<dc:publisher>` | `publisher:` |
| Date | `<dc:date>` | `date:` |
| Description | `<dc:description>` | `description:` |
| Subject | `<dc:subject>` | `tags:` or `categories:` |
| Rights | `<dc:rights>` | `license:` |
| Contributor | `<dc:contributor>` | `contributors:` |

### Recommended Frontmatter Structure

```yaml
---
title: "Book Title"
author: "Author Name"
authors:
  - name: "Author Name"
    role: "aut"
publisher: "Publisher Name"
isbn: "978-0-12345-678-9"
language: "en"
date: "2024-01-15"
description: |
  Multi-line description extracted from
  the EPUB metadata.
tags:
  - fiction
  - science-fiction
cover: "./assets/cover.jpg"
source_format: "epub"
---
```

**Sources:**
- [EPUB Package Metadata Guide - IDPF](https://idpf.github.io/epub-guides/package-metadata/)
- [Dublin Core - MobileRead Wiki](https://wiki.mobileread.com/wiki/Dublin_Core)
- [Introduction to OPF Metadata - APLN](https://apln.ca/introduction-to-opf-metadata/)

---

## 3. Chapter Organization Strategies

### Strategy A: Single File Output

```
book.md                     # All content in one file
assets/
    images/
        cover.jpg
        figure1.png
```

**Pros:** Simple, portable, easy to process
**Cons:** Unwieldy for large books, no chapter-level navigation

### Strategy B: Multi-File with Index

```
book/
    index.md                # Frontmatter + TOC links
    chapters/
        01-introduction.md
        02-chapter-one.md
        03-chapter-two.md
    assets/
        images/
            cover.jpg
            figure1.png
```

**Pros:** Clean organization, chapter-level editing, better git diffs
**Cons:** More complex to reassemble, link management

### Strategy C: Hybrid (Recommended)

```
book/
    book.md                 # Complete book with frontmatter
    _chapters/              # Optional: individual chapters
        01-introduction.md
        02-chapter-one.md
    assets/
        cover.jpg
        images/
            figure1.png
```

**Pros:** Best of both - single file for reading, chapters for editing
**Cons:** Requires sync mechanism between formats

### Chapter Naming Conventions

1. **Sequential Numbering:** `01-chapter-name.md` (preserves order)
2. **Slug-based:** `introduction.md`, `the-journey-begins.md`
3. **Original IDs:** `chapter-001.md` (matches EPUB internal IDs)

---

## 4. TOC/NCX Handling

### EPUB 2: toc.ncx

The NCX (Navigation Control for XML) provides hierarchical navigation:

```xml
<navMap>
  <navPoint id="ch1" playOrder="1">
    <navLabel><text>Chapter 1</text></navLabel>
    <content src="chapter1.xhtml"/>
    <navPoint id="ch1-1" playOrder="2">
      <navLabel><text>Section 1.1</text></navLabel>
      <content src="chapter1.xhtml#sec1"/>
    </navPoint>
  </navPoint>
</navMap>
```

### EPUB 3: nav.xhtml

Human-readable HTML with `epub:type` attributes:

```html
<nav epub:type="toc">
  <ol>
    <li><a href="chapter1.xhtml">Chapter 1</a>
      <ol>
        <li><a href="chapter1.xhtml#sec1">Section 1.1</a></li>
      </ol>
    </li>
  </ol>
</nav>
```

### Conversion Strategy

1. **Parse both formats** - NCX for structure, nav.xhtml as fallback
2. **Generate Markdown TOC** in index file with relative links
3. **Preserve hierarchy** using nested lists or heading levels
4. **Include landmarks** (cover, toc, bodymatter) when available

**Output Example:**

```markdown
## Table of Contents

1. [Chapter 1: Introduction](./chapters/01-introduction.md)
   - [Section 1.1: Background](./chapters/01-introduction.md#background)
   - [Section 1.2: Methodology](./chapters/01-introduction.md#methodology)
2. [Chapter 2: Analysis](./chapters/02-analysis.md)
```

**Sources:**
- [Table of Contents - DAISY Knowledge Base](https://kb.daisy.org/publishing/docs/navigation/toc.html)
- [toc.ncx - Epub Knowledge](https://epubknowledge.com/docs/ncx/)
- [NCX - MobileRead Wiki](https://wiki.mobileread.com/wiki/NCX)

---

## 5. Image/Asset Extraction

### Extraction Approaches

| Approach | Method | Notes |
|----------|--------|-------|
| Pandoc `--extract-media` | Built-in flag | Automatic, preserves original names |
| ebooklib iteration | `get_items_of_type(ITEM_IMAGE)` | Full control, Python-native |
| ZIP extraction | Direct unzip | Fastest, requires path mapping |

### Image Processing Pipeline

```
1. Extract from EPUB
2. Determine original path (OEBPS/images/fig1.png)
3. Map to new path (assets/images/fig1.png)
4. Update references in Markdown
5. Optional: optimize (resize, compress)
```

### Reference Rewriting

```python
# Original EPUB reference (in XHTML)
<img src="../images/figure1.png" alt="Figure 1"/>

# Converted Markdown
![Figure 1](./assets/images/figure1.png)
```

### Asset Types to Handle

- **Images:** PNG, JPEG, GIF, SVG
- **Fonts:** TTF, OTF, WOFF (usually excluded from Markdown output)
- **Audio/Video:** MP3, MP4 (rare, preserve if present)
- **Cover:** Special handling - extract to root assets

**Sources:**
- [Pandoc User's Guide](https://pandoc.org/MANUAL.html)
- [ebooklib GitHub](https://github.com/aerkalov/ebooklib)

---

## 6. Tool Comparison

### Option A: Pandoc

**Command:**
```bash
pandoc input.epub --extract-media=./assets -o output.md
```

| Aspect | Rating | Notes |
|--------|--------|-------|
| Installation | 5/5 | Single binary, all platforms |
| Structure preservation | 3/5 | Flattens to single file |
| Image extraction | 4/5 | Built-in `--extract-media` |
| Metadata handling | 3/5 | Partial, requires post-processing |
| Customization | 4/5 | Filters, templates, Lua scripts |
| Confidence | HIGH | Mature, widely used |

**Best for:** Quick conversions, simple books, pipeline integration

### Option B: Calibre (ebook-convert)

**Command:**
```bash
ebook-convert input.epub output.txt --txt-output-formatting=markdown
```

| Aspect | Rating | Notes |
|--------|--------|-------|
| Installation | 3/5 | Large dependency, GUI-oriented |
| Structure preservation | 2/5 | Limited Markdown output options |
| Image extraction | 2/5 | Not directly to Markdown |
| Metadata handling | 4/5 | Excellent metadata tools |
| Customization | 3/5 | Conversion options, plugins |
| Confidence | MEDIUM | Primarily for format conversion |

**Best for:** Metadata extraction, format validation, chained conversions

### Option C: Python ebooklib + markdownify

**Implementation:**
```python
from ebooklib import epub
from markdownify import markdownify as md

book = epub.read_epub('input.epub')
for item in book.get_items_of_type(ebooklib.ITEM_DOCUMENT):
    html = item.get_content().decode('utf-8')
    markdown = md(html)
```

| Aspect | Rating | Notes |
|--------|--------|-------|
| Installation | 4/5 | pip install, lightweight |
| Structure preservation | 5/5 | Full control over output |
| Image extraction | 5/5 | Direct file access |
| Metadata handling | 5/5 | Complete Dublin Core access |
| Customization | 5/5 | Any logic possible |
| Confidence | HIGH | Well-maintained libraries |

**Best for:** Custom pipelines, specific requirements, programmatic control

### Option D: epub-to-markdown (kyktommy)

**Features:**
- Multiple interfaces (CLI, API, Web, MCP)
- Image extraction with optimization
- Chapter-based output
- Metadata preservation

| Aspect | Rating | Notes |
|--------|--------|-------|
| Installation | 4/5 | pip/clone, Python 3.8+ |
| Structure preservation | 4/5 | Multi-file chapters |
| Image extraction | 5/5 | Optimized JPEG output |
| Metadata handling | 4/5 | Title, author, publisher |
| Customization | 3/5 | Limited extensibility |
| Confidence | MEDIUM | Newer project, active development |

**Best for:** Ready-made solution, batch processing, API integration

### Option E: E2M (wisupai)

**Features:**
- Parser-converter architecture
- Multiple file format support
- LLM integration options

| Aspect | Rating | Notes |
|--------|--------|-------|
| Installation | 3/5 | pip, depends on unstructured |
| Structure preservation | 3/5 | Focused on text extraction |
| Image extraction | 3/5 | Basic support |
| Metadata handling | 3/5 | Partial |
| Customization | 4/5 | Pluggable engines |
| Confidence | MEDIUM | RAG/LLM focus, less book-oriented |

**Best for:** LLM corpus preparation, multi-format workflows

**Sources:**
- [Pandoc Creating an ebook](https://pandoc.org/epub.html)
- [Calibre ebook-convert documentation](https://manual.calibre-ebook.com/generated/en/ebook-convert.html)
- [epub-to-markdown GitHub](https://github.com/kyktommy/epub-to-markdown)
- [E2M GitHub](https://github.com/wisupai/e2m)

---

## 7. Custom Implementation Architecture

### Recommended Stack

```
┌─────────────────────────────────────────────────┐
│                  CLI Interface                   │
│              (click or argparse)                 │
└─────────────────────────────────────────────────┘
                       │
┌─────────────────────────────────────────────────┐
│               EPUBParser Module                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │
│  │   ebooklib  │  │  lxml/bs4   │  │  zipfile│ │
│  └─────────────┘  └─────────────┘  └─────────┘ │
└─────────────────────────────────────────────────┘
                       │
┌─────────────────────────────────────────────────┐
│              Intermediate Model                  │
│  Book(metadata, chapters[], assets[], toc)      │
└─────────────────────────────────────────────────┘
                       │
┌─────────────────────────────────────────────────┐
│             MarkdownWriter Module                │
│  ┌─────────────┐  ┌─────────────┐              │
│  │ markdownify │  │  frontmatter│              │
│  └─────────────┘  └─────────────┘              │
└─────────────────────────────────────────────────┘
                       │
┌─────────────────────────────────────────────────┐
│                Output Files                      │
│     book.md + assets/ + _chapters/              │
└─────────────────────────────────────────────────┘
```

### Key Dependencies

```toml
[project]
dependencies = [
    "ebooklib>=0.18",      # EPUB parsing
    "beautifulsoup4>=4.12", # HTML parsing
    "markdownify>=0.11",    # HTML to Markdown
    "lxml>=4.9",            # XML/HTML processing
    "pyyaml>=6.0",          # Frontmatter generation
    "click>=8.1",           # CLI framework
]
```

### Core Classes

```python
@dataclass
class BookMetadata:
    title: str
    authors: list[str]
    identifier: str
    language: str
    publisher: str | None
    date: str | None
    description: str | None
    subjects: list[str]
    rights: str | None

@dataclass
class Chapter:
    id: str
    title: str
    content_html: str
    order: int
    level: int  # Heading depth in TOC

@dataclass
class Asset:
    id: str
    original_path: str
    media_type: str
    content: bytes

@dataclass
class Book:
    metadata: BookMetadata
    chapters: list[Chapter]
    assets: list[Asset]
    toc: list[TocEntry]
    cover_id: str | None
```

---

## 8. Recommendations

### For Simple Conversion Needs

**Use Pandoc** with `--extract-media`:

```bash
pandoc book.epub --extract-media=./media -o book.md
```

Then post-process to add YAML frontmatter using metadata from:
```bash
pandoc book.epub -t json | jq '.meta'
```

### For Production Pipeline

**Build custom solution** using:
- **ebooklib** for EPUB parsing
- **markdownify** for HTML to Markdown
- **Custom frontmatter generator** mapping Dublin Core

This provides:
- Full control over output structure
- Proper chapter organization
- Complete metadata preservation
- Extensibility for edge cases

### Implementation Priority

1. **Phase 1:** Basic conversion with Pandoc wrapper
2. **Phase 2:** Custom metadata extraction + frontmatter
3. **Phase 3:** Multi-file chapter output
4. **Phase 4:** TOC reconstruction
5. **Phase 5:** Asset optimization (optional)

### Edge Cases to Handle

- **No TOC:** Generate from heading structure
- **Inline images:** Base64 or extract and reference
- **Math:** MathML to LaTeX or image fallback
- **Tables:** HTML tables to Markdown tables (limited)
- **Footnotes:** Convert to Markdown footnote syntax
- **Links:** Internal EPUB links to Markdown cross-references

---

## 9. Confidence Assessment

| Topic | Confidence | Rationale |
|-------|------------|-----------|
| EPUB Structure | HIGH | Well-documented standards (IDPF/W3C) |
| Pandoc capabilities | HIGH | Mature tool, extensive documentation |
| ebooklib capabilities | HIGH | Active project, good samples |
| Metadata mapping | HIGH | Dublin Core is standardized |
| Tool comparison | MEDIUM-HIGH | Based on documentation + community |
| Custom implementation | MEDIUM | Architecture is sound, untested |

---

## 10. Sources

### Standards and Specifications
- [EPUB 3 Overview - W3C](https://www.w3.org/publishing/epub3/epub-overview.html)
- [Open Packaging Format 2.0 - IDPF](https://idpf.org/epub/20/spec/OPF_2.0_final_spec.html)
- [Dublin Core Metadata Initiative](https://www.dublincore.org/)

### Tools and Libraries
- [Pandoc User's Guide](https://pandoc.org/MANUAL.html)
- [Pandoc EPUB Documentation](https://pandoc.org/epub.html)
- [ebooklib GitHub](https://github.com/aerkalov/ebooklib)
- [markdownify GitHub](https://github.com/matthewwithanm/python-markdownify)
- [Calibre ebook-convert](https://manual.calibre-ebook.com/generated/en/ebook-convert.html)

### Tutorials and Guides
- [Anatomy of an EPUB 3 file - EDRLab](https://www.edrlab.org/open-standards/anatomy-of-an-epub-3-file/)
- [EPUB Format Construction Guide](https://www.hxa.name/articles/content/epub-guide_hxa7241_2007.html)
- [Introduction to EPUB Files - APLN](https://apln.ca/introduction-to-epub-files/)
- [Customizing Pandoc for PDF/EPUB](https://learnbyexample.github.io/customizing-pandoc/)

### Community Tools
- [epub-to-markdown (kyktommy)](https://github.com/kyktommy/epub-to-markdown)
- [E2M (wisupai)](https://github.com/wisupai/e2m)
- [epub-utils](https://github.com/ernestofgonzalez/epub-utils)

---

*Research conducted: 2026-02-01*
*Originally researched: 2026-02-01 | Imported: 2026-02-09*
