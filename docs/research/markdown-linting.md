# Markdown Linting Tools for CLI Integration

## Research Summary

**Date:** 2026-02-01
**Confidence Rating:** High (85%)
**Primary Use Case:** CLI integration for AI agent workflows with YAML/TOML frontmatter support

---

## Executive Summary

For CLI tool integration, **markdownlint-cli2** is the recommended primary tool, with **rumdl** as a high-performance alternative for speed-critical workflows. Both handle frontmatter well and offer excellent CLI integration.

---

## Tool Comparison Matrix

| Tool | Language | Speed | Rules | Frontmatter | Config Format | Fix Mode | Best For |
|------|----------|-------|-------|-------------|---------------|----------|----------|
| markdownlint-cli2 | Node.js | Fast | 60 | YAML/TOML/JSON | JSON/YAML/JS | Yes | General use, broad ecosystem |
| rumdl | Rust | Very Fast | 57 | Auto-detect | TOML | Yes | Speed-critical, modern setup |
| mado | Rust | 49-60x faster | ~45 | Unknown | TOML | No | Maximum speed |
| remark-lint | Node.js | Moderate | 70+ | Via plugin | JSON/YAML/JS | Yes | Unified ecosystem, transformation |
| Vale | Go | Fast | Extensible | Yes | INI | N/A | Prose quality, style guides |
| textlint | Node.js | Moderate | Plugin-based | Via plugin | JSON/YAML | Yes | Natural language, MCP support |

---

## Detailed Tool Analysis

### 1. markdownlint-cli2 (Recommended)

**Repository:** [DavidAnson/markdownlint-cli2](https://github.com/DavidAnson/markdownlint-cli2)

**Strengths:**
- Configuration-driven design (ideal for agent workflows)
- 60 built-in rules covering CommonMark + GFM
- Native YAML, TOML, and JSON frontmatter support via regex patterns
- Excellent VS Code integration shares configuration
- GitHub Actions support built-in
- Active maintenance by original markdownlint author

**Frontmatter Configuration:**
```yaml
# .markdownlint-cli2.yaml
frontMatter: "(^---\\s*$[^]*?^---\\s*$)(\\r\\n|\\r|\\n|$)"
config:
  MD041:
    front_matter_title: "^\\s*title\\s*[:=]"  # Handles both YAML and TOML
```

**CLI Usage:**
```bash
# Basic linting
markdownlint-cli2 "docs/**/*.md"

# With fix mode
markdownlint-cli2 --fix "**/*.md"

# Custom config
markdownlint-cli2 --config .markdownlint-cli2.yaml "docs/"
```

**Agent Integration:**
- Exit codes: 0 (success), 1 (errors), 2 (failure)
- JSON output available for parsing
- `.gitignore` integration for performance

**Confidence:** 90%

---

### 2. rumdl (Speed Alternative)

**Repository:** [rvben/rumdl](https://github.com/rvben/rumdl)

**Strengths:**
- Rust-based, significantly faster than Node.js alternatives
- 57 rules, markdownlint-compatible
- Intelligent caching (only re-lints changed files)
- Multi-flavor support: GFM, MkDocs, MDX, Quarto
- Auto-detection of Markdown flavor
- TOML configuration with JSON Schema support
- VS Code, Zed, and Obsidian extensions

**Configuration:**
```toml
# .rumdl.toml
[rules]
MD013 = false  # Disable line length
MD033 = false  # Allow inline HTML

[exclude]
patterns = ["vendor/**", "node_modules/**"]
```

**CLI Usage:**
```bash
# Basic check
rumdl check .

# With auto-fix
rumdl check --fix docs/

# Watch mode
rumdl check --watch .

# Specific rules
rumdl check --disable MD013,MD033 README.md
```

**Agent Integration:**
- Clean stdin/stdout support
- Respects `.gitignore`
- Pre-commit hook support

**Confidence:** 85%

---

### 3. mado (Maximum Speed)

**Repository:** [akiomik/mado](https://github.com/akiomik/mado)

**Strengths:**
- 49-60x faster than markdownlint (benchmarked on GitLab docs)
- Rust-based, CommonMark + GFM compatible
- TOML configuration
- Most markdownlint rules supported

**Limitations:**
- Frontmatter handling not explicitly documented
- No fix mode
- Some rules still unstable
- Smaller ecosystem

**CLI Usage:**
```bash
mado check .
mado check --config mado.toml docs/**/*.md
```

**Confidence:** 70%

---

### 4. remark-lint (Unified Ecosystem)

**Repository:** [remarkjs/remark-lint](https://github.com/remarkjs/remark-lint)

**Strengths:**
- Part of unified/remark ecosystem (extensive plugin library)
- 70+ rules across ~70 individual plugins
- Powerful AST-based analysis
- Frontmatter validation via `remark-lint-frontmatter-schema`
- Auto-fix capabilities
- Transformation pipeline (lint + format + transform)

**Use When:**
- Already using unified/remark for Markdown processing
- Need custom AST-based rules
- Require Markdown transformation alongside linting

**Configuration:**
```json
{
  "plugins": [
    "remark-preset-lint-recommended",
    "remark-preset-lint-consistent",
    ["remark-lint-frontmatter-schema", { "schema": "./frontmatter.schema.json" }]
  ]
}
```

**CLI Usage:**
```bash
npx remark doc/ --use remark-preset-lint-recommended
```

**Confidence:** 80%

---

### 5. Vale (Prose Quality)

**Repository:** [errata-ai/vale](https://github.com/errata-ai/vale)

**Purpose:** Natural language linting, not structural Markdown linting

**Best For:**
- Technical writing quality
- Style guide enforcement
- Terminology consistency
- Inclusive language checking

**Supported Formats:** Markdown, AsciiDoc, reStructuredText, HTML, XML, Org mode

**Notable Users:** Datadog, Elastic, Grafana, GitLab

**Use With:** Combine with markdownlint-cli2 for comprehensive linting

**Confidence:** 85%

---

### 6. textlint (Natural Language)

**Repository:** [textlint/textlint](https://github.com/textlint/textlint)

**Strengths:**
- Pluggable architecture (ESLint-like for prose)
- MCP server support (AI assistant integration)
- Plugin ecosystem for various formats

**Limitations:**
- No bundled rules (requires installing plugins)
- Node.js 20+ required
- More complex setup

**Confidence:** 75%

---

## Recommendations by Use Case

### Primary Recommendation: markdownlint-cli2

**Why:**
1. **Mature ecosystem** - Broad adoption, extensive documentation
2. **Configuration-first** - Ideal for agent workflows (no CLI flag parsing needed)
3. **Full frontmatter support** - YAML, TOML, JSON with configurable patterns
4. **VS Code parity** - Same config works in editor and CLI
5. **Fix mode** - Agents can auto-correct simple issues
6. **Active maintenance** - Regular updates, responsive maintainer

**Installation:**
```bash
npm install -g markdownlint-cli2
# or
npx markdownlint-cli2 "**/*.md"
```

### Speed-Critical Alternative: rumdl

**Why:**
1. **Rust performance** - Order of magnitude faster
2. **Caching** - Incremental linting for large repos
3. **markdownlint compatible** - Easy migration
4. **Modern tooling** - TOML config, multiple editors

**Installation:**
```bash
# Via Homebrew
brew install rvben/tap/rumdl

# Via Cargo
cargo install rumdl
```

### Comprehensive Quality Stack

For maximum coverage, combine:
1. **markdownlint-cli2** - Structural linting
2. **Vale** - Prose quality and style
3. **remark-lint-frontmatter-schema** - Frontmatter validation

---

## Configuration Examples

### Minimal Setup (.markdownlint-cli2.yaml)

```yaml
# Ignore frontmatter
frontMatter: "(^---\\s*$[^]*?^---\\s*$)(\\r\\n|\\r|\\n|$)"

# Relaxed rules for AI-generated content
config:
  MD013: false          # Line length (AI often produces long lines)
  MD033: false          # Inline HTML (may be needed)
  MD041:
    front_matter_title: "^\\s*title\\s*[:=]"

# Ignore patterns
ignores:
  - "node_modules/**"
  - "vendor/**"
  - ".git/**"
```

### Agent Integration Pattern

```bash
#!/bin/bash
# lint-markdown.sh - For agent integration

# Run linter with JSON output for parsing
markdownlint-cli2 --format json "$@" 2>&1 | jq .

# Exit code reflects lint status
exit ${PIPESTATUS[0]}
```

---

## Ecosystem Context

### Node.js Ecosystem
- **markdownlint** / **markdownlint-cli2**: Industry standard
- **remark-lint**: Part of unified ecosystem
- **textlint**: Natural language focus

### Rust Ecosystem (Emerging)
- **rumdl**: Most mature, actively developed
- **mado**: Fastest, but less feature-complete

### Go Ecosystem
- **Vale**: Prose/style linting (different purpose)
- **gomarklint**: Newer, Go-native option

### Multi-Language
- **MegaLinter**: Aggregates multiple linters
- **trunk**: Multi-language linting platform

---

## Sources

- [markdownlint](https://github.com/DavidAnson/markdownlint) - Node.js linter library
- [markdownlint-cli2](https://github.com/DavidAnson/markdownlint-cli2) - Recommended CLI
- [rumdl](https://github.com/rvben/rumdl) - Rust-based linter
- [mado](https://github.com/akiomik/mado) - Fast Rust linter
- [remark-lint](https://github.com/remarkjs/remark-lint) - Unified ecosystem linter
- [Vale](https://github.com/errata-ai/vale) - Prose linter
- [textlint](https://github.com/textlint/textlint) - Pluggable natural language linter
- [npm trends comparison](https://npmtrends.com/markdownlint-vs-markdownlint-cli-vs-markdownlint-cli2)
- [Scott Lowe's Markdown Linting Guide](https://blog.scottlowe.org/2024/03/01/linting-your-markdown-files/)

---

## Confidence Notes

**High confidence (85%+):**
- Tool feature sets and capabilities
- Frontmatter support in markdownlint-cli2
- CLI integration patterns

**Medium confidence (70-85%):**
- Relative performance claims (benchmark conditions vary)
- rumdl stability for production use
- mado frontmatter handling

**Lower confidence (<70%):**
- Long-term maintenance trajectories
- Specific edge cases with complex frontmatter

---

*Originally researched: 2026-02-01 | Imported: 2026-02-09*
