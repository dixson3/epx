# EPUB Format Research Summary

**Research Date:** 2026-02-01
**Confidence Rating:** High (based on official W3C/IDPF specifications and authoritative sources)

---

## Executive Summary

EPUB (Electronic Publication) is the dominant open standard for reflowable e-books, maintained by W3C after merging with IDPF in 2017. The format has evolved through major versions (2.0 → 3.0 → 3.2 → 3.3), with EPUB 3.3 becoming the first official W3C Recommendation in May 2023. EPUB is essentially a ZIP archive containing XHTML/HTML5 content, CSS styling, images, and metadata in a standardized structure.

---

## 1. Core Structure (OCF - Open Container Format)

**Confidence: Very High** - Based on official W3C/IDPF specifications

### File Architecture

An EPUB file is a ZIP archive with the `.epub` extension containing:

```
mybook.epub (ZIP archive)
├── mimetype                    # MUST be first, uncompressed
├── META-INF/
│   └── container.xml          # REQUIRED: points to OPF file
└── OEBPS/ (or content/)       # Content directory (name flexible)
    ├── content.opf            # Package document
    ├── toc.xhtml              # Navigation document (EPUB 3)
    ├── toc.ncx                # Legacy navigation (EPUB 2)
    ├── chapter1.xhtml
    ├── chapter2.xhtml
    ├── styles/
    │   └── style.css
    └── images/
        └── cover.jpg
```

### Key Files

| File | Purpose | Required |
|------|---------|----------|
| `mimetype` | Contains `application/epub+zip` (must be first file, uncompressed, unencrypted) | Yes |
| `META-INF/container.xml` | Bootstrap file pointing to package document | Yes |
| `*.opf` (Package Document) | Publication manifest, metadata, and spine | Yes |
| `*.xhtml` (Content Documents) | Actual book content in XHTML/HTML5 | Yes |
| Navigation Document | Table of contents (toc.xhtml for EPUB 3, toc.ncx for EPUB 2) | Yes |

### Package Document (OPF) Sections

1. **Metadata**: Publication information (title, author, identifier, language)
2. **Manifest**: Exhaustive list of all resources with media types
3. **Spine**: Linear reading order of content documents

---

## 2. Version Comparison

**Confidence: High** - Based on official specifications and W3C documentation

### EPUB 2.0/2.0.1 (2007-2010)

- **Content Format**: XHTML 1.1 or DTBook (DAISY)
- **Navigation**: NCX file (NavMap)
- **Styling**: CSS 2.0 subset
- **Status**: Obsolete, no longer maintained
- **Key Limitations**: No native multimedia, limited CSS, no scripting

### EPUB 3.0/3.0.1 (2011-2014)

- **Content Format**: XHTML5 (HTML5 serialized as XML)
- **Navigation**: EPUB Navigation Document (HTML5-based)
- **Styling**: CSS 2.1 + subset of CSS 3
- **New Features**:
  - Native audio/video support
  - MathML and SVG
  - Media Overlays (synchronized text-audio)
  - JavaScript scripting (with constraints)
  - Fixed Layout (FXL) support
  - Enhanced global language support (vertical text, RTL)

### EPUB 3.2 (May 2019)

- **Published by**: W3C EPUB 3 Community Group (not a W3C Standard)
- **Key Changes**:
  - References "current" HTML/CSS/SVG instead of fixed versions
  - WOFF 2.0 and SFNT fonts as Core Media Types
  - Removal of epub-prefixed CSS properties
  - 100% backward compatible with EPUB 3.0.1
  - Formal recommendation to follow EPUB Accessibility Guidelines

### EPUB 3.3 (May 2023) - Current Standard

- **Status**: First W3C Recommendation (official international web standard)
- **Key Changes**:
  - Improved bidirectional text support
  - Enhanced security specifications for scripts
  - WebP and Opus media format support
  - Fixed SVG epub:type attribute issues
  - Clarified fixed layout viewport whitespace handling
- **Backward Compatible**: Any EPUB 3.2 file is valid EPUB 3.3

---

## 3. Metadata Specifications

**Confidence: High** - Based on IDPF/W3C specifications and Dublin Core standards

### Required Metadata (EPUB 3)

```xml
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="pub-id">urn:uuid:...</dc:identifier>
    <dc:title>Book Title</dc:title>
    <dc:language>en</dc:language>
    <meta property="dcterms:modified">2026-01-15T00:00:00Z</meta>
</metadata>
```

### Dublin Core Elements

EPUB uses the Dublin Core Metadata Element Set (DCMES) as its reference schema:

| Element | Description | Required |
|---------|-------------|----------|
| `dc:identifier` | Unique publication identifier (ISBN, UUID, DOI) | Yes |
| `dc:title` | Publication title | Yes |
| `dc:language` | Primary language (BCP 47) | Yes |
| `dc:creator` | Author(s) | No |
| `dc:publisher` | Publisher name | No |
| `dc:date` | Publication date | No |
| `dc:description` | Summary/description | No |
| `dc:subject` | Keywords/categories | No |
| `dc:rights` | Copyright statement | No |

### Extended Metadata (meta element)

```xml
<meta property="dcterms:modified">2026-01-15T00:00:00Z</meta>
<meta property="media:active-class">-epub-media-overlay-active</meta>
<meta name="cover" content="cover-image"/>
```

### ONIX Integration

EPUB can link to external ONIX records for supply chain metadata:

```xml
<link rel="onix-record" href="http://example.org/onix/12389347"/>
```

ONIX (ONline Information eXchange) provides commercial metadata for distribution, including pricing, availability, and accessibility information.

### Accessibility Metadata

```xml
<meta property="schema:accessMode">textual</meta>
<meta property="schema:accessModeSufficient">textual</meta>
<meta property="schema:accessibilityFeature">structuralNavigation</meta>
<link rel="dcterms:conformsTo" href="http://www.idpf.org/epub/a11y/accessibility-20170105.html#wcag-aa"/>
```

---

## 4. Core Media Types

**Confidence: Very High** - Based on official specifications

### Content Documents
- **XHTML**: `application/xhtml+xml` (required)
- **SVG**: `image/svg+xml` (required)

### Images
- **PNG**: `image/png` (required)
- **JPEG**: `image/jpeg` (required)
- **GIF**: `image/gif` (required)
- **WebP**: `image/webp` (EPUB 3.3+)

### Audio
- **MP3**: `audio/mpeg` (required)
- **MP4 Audio**: `audio/mp4` (recommended)
- **Opus**: `audio/ogg` (EPUB 3.3+)

### Video
- No required codec (VP8 or H.264 recommended)
- `video/mp4`, `video/webm` commonly used
- Video is "exempt" from fallback requirements

### Fonts
- **OpenType**: `font/otf`, `font/ttf` (required)
- **WOFF**: `font/woff` (required)
- **WOFF 2.0**: `font/woff2` (EPUB 3.2+)
- Font obfuscation supported for licensing compliance

### Styling
- **CSS**: `text/css` (required)

### Scripting
- **JavaScript**: `application/javascript` (optional support)

---

## 5. Extensions and Advanced Features

**Confidence: High** - Based on official specifications

### Fixed Layout (FXL)

For comics, children's books, and complex layouts:

```xml
<meta property="rendition:layout">pre-paginated</meta>
<meta property="rendition:spread">landscape</meta>
<meta property="rendition:orientation">auto</meta>
```

**Limitations**: Text cannot be resized; accessibility challenges; varies by reading system.

### Media Overlays

Synchronized text-audio playback using SMIL (Synchronized Multimedia Integration Language):

```xml
<item id="chapter1-overlay" href="chapter1.smil"
      media-type="application/smil+xml"/>
<itemref idref="chapter1" media-overlay="chapter1-overlay"/>
```

Benefits: Accessibility for visually impaired, language learning, audiobook synchronization.

### JavaScript Scripting

Two types of scripted content:
1. **Spine-level scripting**: JavaScript in content documents
2. **Container-constrained scripting**: Isolated widgets

**Requirements**:
- Must follow progressive enhancement (content readable without scripts)
- Reading system support is optional
- Security sandboxing typically applied

### EPUB CFI (Canonical Fragment Identifiers)

Enables precise location references within publications:

```
book.epub#epubcfi(/6/4[chap01ref]!/4[body01]/10[para05]/3:10)
```

Uses:
- Reading position synchronization
- Annotation anchoring
- Cross-publication linking
- Bookmark portability

### Multiple Renditions

Single EPUB can contain multiple renditions (e.g., different layouts for different devices):

```xml
<!-- In container.xml -->
<rootfile full-path="OEBPS/reflow.opf" media-type="application/oebps-package+xml"/>
<rootfile full-path="OEBPS/fixed.opf" media-type="application/oebps-package+xml"/>
```

---

## 6. Accessibility (EPUB Accessibility 1.1)

**Confidence: High** - Based on W3C EPUB Accessibility specification

### Requirements

- Must meet WCAG 2.0 Level A minimum
- WCAG 2.x Level AA recommended
- Must include accessibility metadata

### Key Features
- Semantic HTML5 elements
- ARIA landmarks and roles
- Alternative text for images
- Reading order preservation
- Navigation document structure

### Conformance Declaration

```xml
<meta property="dcterms:conformsTo">
  http://www.idpf.org/epub/a11y/accessibility-20170105.html#wcag-aa
</meta>
```

---

## 7. Format Limitations

**Confidence: High** - Based on specifications and ecosystem analysis

### Technical Limitations

| Limitation | Description |
|------------|-------------|
| **Video Codec** | No required video format; reading system support varies widely |
| **JavaScript** | Optional support; sandboxed; progressive enhancement required |
| **Fixed Layout Accessibility** | Text resizing impossible; full accessibility not achievable |
| **Font Licensing** | Obfuscation (not encryption) for embedded fonts |
| **Reading System Variation** | Feature support varies significantly across devices |

### Ecosystem Fragmentation

| Platform | EPUB Support | Notable Limitations |
|----------|--------------|---------------------|
| **Kindle** | Conversion only | Converts to KFX; some features lost |
| **Kobo** | Native EPUB 2/3 | Good support; Adobe DRM |
| **Apple Books** | Native EPUB 3 | Excellent support; proprietary .ibooks extensions |
| **Google Play Books** | Native EPUB | Layout inconsistencies reported |
| **Adobe Digital Editions** | Native | EPUB 2 focus; limited EPUB 3 |

### DRM Considerations

- EPUB spec does not mandate specific DRM
- Common DRM systems:
  - **Adobe DRM (ADEPT)**: Most widespread
  - **Apple FairPlay**: Apple ecosystem only
  - **Amazon DRM**: Kindle ecosystem only
  - **Social DRM (watermarking)**: Non-restrictive alternative
- DRM not part of the open standard itself

### Interoperability Challenges

- Reading position sync across systems (CFI support varies)
- Font rendering differences
- CSS/layout interpretation varies
- Media Overlay support optional
- FXL rendering inconsistencies

---

## 8. Ecosystem Context

### Standards Organizations

- **W3C** (2017-present): Current maintainer of EPUB specifications
- **IDPF** (1999-2017): Original developer, merged with W3C
- **ISO**: EPUB 3.0.1 published as ISO/IEC 23736:2020

### Related Standards

- **ONIX**: Commercial book metadata (supply chain)
- **Dublin Core**: Core metadata vocabulary
- **WCAG**: Web accessibility guidelines
- **SMIL**: Media synchronization
- **CSS**: Styling
- **HTML5/XHTML**: Content structure

### Tools Ecosystem

- **Calibre**: Open-source e-book management/conversion
- **Sigil**: Open-source EPUB editor
- **EPUBCheck**: Official validation tool
- **Adobe InDesign**: Professional publishing
- **Pandoc**: Document conversion

---

## 9. Recommendations

### For EPUB Creation

1. **Target EPUB 3.3** for maximum compatibility and features
2. **Include EPUB 2 NCX** for backward compatibility with older readers
3. **Follow EPUB Accessibility** guidelines (WCAG AA)
4. **Use Core Media Types** exclusively when possible; provide fallbacks otherwise
5. **Avoid Fixed Layout** unless content genuinely requires it (comics, image-heavy)
6. **Test across multiple readers** (Kobo, Apple Books, Calibre, ADE)
7. **Validate with EPUBCheck** before distribution

### For EPUB Parsing/Reading Systems

1. **Support both EPUB 2 and EPUB 3** navigation structures
2. **Handle graceful degradation** for unsupported features
3. **Implement Core Media Types** at minimum
4. **Consider CFI support** for annotation/position sync
5. **Follow accessibility requirements** for reading systems

---

## Sources

### Official Specifications
- [EPUB 3.3 - W3C Recommendation](https://www.w3.org/TR/epub-33/)
- [EPUB 3.3 W3C Press Release](https://www.w3.org/press-releases/2023/epub33-rec/)
- [EPUB 3.2 Packages](https://w3c.github.io/epub-specs/archive/epub32/spec/epub-packages.html)
- [EPUB OCF 3.2](https://w3c.github.io/epub-specs/archive/epub32/spec/epub-ocf.html)
- [EPUB 3 Changes from EPUB 2.0.1](https://idpf.org/epub/30/spec/epub30-changes.html)
- [EPUB Accessibility Techniques 1.1](https://www.w3.org/TR/epub-a11y-tech-11/)
- [EPUB CFI Specification](https://idpf.org/epub/linking/cfi/)
- [EPUB Media Overlays 3.2](https://www.w3.org/publishing/epub32/epub-mediaoverlays.html)

### Authoritative References
- [EPUB - Wikipedia](https://en.wikipedia.org/wiki/EPUB)
- [Anatomy of an EPUB 3 file - EDRLab](https://www.edrlab.org/open-standards/anatomy-of-an-epub-3-file/)
- [EPUB 3 Core Media Types](https://idpf.github.io/epub-cmt/v3/)
- [Core Media Types - DAISY Knowledge Base](https://kb.daisy.org/publishing/docs/epub/cmt.html)
- [EPUB at Library of Congress](https://www.loc.gov/preservation/digital/formats/fdd/fdd000309.shtml)

### Ecosystem Documentation
- [Kobo EPUB Spec Support](https://github.com/kobolabs/epub-spec)
- [Apple Books Asset Guide - JavaScript](https://help.apple.com/itc/booksassetguide/en.lproj/itc013b02e4a.html)
- [EPUB Package Metadata Guide](https://idpf.github.io/epub-guides/package-metadata/)
- [Introduction to OPF Metadata - APLN](https://apln.ca/introduction-to-opf-metadata/)
- [EPUB Accessibility Fixed Layout](https://w3c.github.io/epub-specs/wg-notes/fxl-a11y/)

### Recent Updates
- [EPUB 3.3 Recommendations Published (2025)](https://www.w3.org/blog/2025/epub3-3-recommendations-published-work-begins-on-new-features/)

---

*Research compiled for epx project - EPUB format deep-dive*
*Originally researched: 2026-02-01 | Imported: 2026-02-09*
