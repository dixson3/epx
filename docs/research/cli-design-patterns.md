# GitHub CLI (gh) Command Syntax Analysis for Agent-Optimized Tools

**Research Date**: 2026-02-01
**gh Version Analyzed**: 2.86.0
**Confidence Rating**: High (based on official documentation, source code, and established ecosystem standards)

## Executive Summary

The GitHub CLI (`gh`) represents a modern reference implementation for agent-friendly command-line tools. Its design balances human usability with machine parsability through consistent patterns: hierarchical subcommands, structured JSON output with field selection, TTY-aware formatting, semantic exit codes, and comprehensive environment variable support. This analysis extracts patterns applicable to any CLI tool targeting both human users and AI agents.

---

## 1. Command Structure Patterns

### 1.1 Hierarchical Noun-Verb Organization

```
gh <resource> <action> [<target>] [flags]
```

**Examples**:
```bash
gh pr list              # resource: pr, action: list
gh issue create         # resource: issue, action: create
gh repo clone cli/cli   # resource: repo, action: clone, target: cli/cli
```

**Pattern Analysis**:
- Resources (nouns) come first: `pr`, `issue`, `repo`, `gist`, `release`
- Actions (verbs) come second: `list`, `view`, `create`, `edit`, `delete`, `close`
- Targets are positional (optional): PR number, repo name, issue URL
- Flags modify behavior, not identify targets

**Agent-Friendly Benefits**:
- Predictable command discovery: agents can enumerate resources, then actions
- Consistent mental model across all commands
- Natural language mapping: "list my pull requests" -> `gh pr list`

### 1.2 Command Categories

| Category | Pattern | Examples |
|----------|---------|----------|
| **General** | List/status operations | `gh pr list`, `gh issue status` |
| **Targeted** | Operations on specific items | `gh pr view 123`, `gh issue close 456` |
| **Utility** | Cross-cutting operations | `gh api`, `gh search`, `gh alias` |
| **Meta** | Tool management | `gh config`, `gh extension`, `gh auth` |

### 1.3 Argument Flexibility

`gh` accepts multiple formats for the same target:

```bash
gh pr view 123                                    # by number
gh pr view https://github.com/owner/repo/pull/123 # by URL
gh pr view feature-branch                          # by branch name
gh pr view OWNER:feature-branch                    # by qualified branch
```

**Agent Optimization**: Agents can use whichever format is available from context without format conversion.

---

## 2. Flag Conventions

### 2.1 Flag Syntax Standards

| Convention | Example | Notes |
|------------|---------|-------|
| Long flags with `--` | `--json`, `--web`, `--draft` | Primary, always available |
| Short flags with `-` | `-R`, `-l`, `-a` | Only for frequent operations |
| Value assignment | `--limit 30` or `--limit=30` | Space or equals both work |
| Boolean toggles | `--draft` / `--no-draft` | Explicit on/off for clarity |
| Repeatable flags | `-l bug -l urgent` | Multiple values for same flag |

### 2.2 Standard Flag Vocabulary

**Universal flags (inherited across all commands)**:
```
--help          Show help for command
-R, --repo      Select repository (HOST/OWNER/REPO format)
```

**Output control flags**:
```
--json <fields>    Output JSON with specified fields
-q, --jq           Filter JSON with jq expression
-t, --template     Format output with Go template
-w, --web          Open in browser instead of terminal
```

**Filtering flags (list commands)**:
```
-L, --limit        Maximum items to fetch
-s, --state        Filter by state (open/closed/merged/all)
-l, --label        Filter by label
-a, --assignee     Filter by assignee
-A, --author       Filter by author
-S, --search       Advanced search query
```

### 2.3 Flag Design Principles

1. **Semantic naming**: `--author` not `--user-filter-type-1`
2. **Consistent across commands**: `-l` always means label, `-L` always means limit
3. **Defaults that work**: `--state open` is default; `--limit 30` is sensible
4. **No required flags for basic operations**: `gh pr list` works without any flags

---

## 3. Output Formats

### 3.1 TTY-Aware Adaptive Output

**Terminal (human-readable)**:
```
$ gh pr list
Showing 23 of 23 open pull requests in cli/cli

#123  A helpful contribution    contribution-branch    about 1 day ago
#124  Improve the docs          docs-branch            about 2 days ago
```

**Piped (machine-readable)**:
```
$ gh pr list | head -2
123	A helpful contribution	contribution-branch	2024-01-15T10:30:00Z
124	Improve the docs	docs-branch	2024-01-14T08:15:00Z
```

**Automatic behaviors when piped**:
- Tab-delimited fields (parseable with `cut`, `awk`)
- No color escape codes
- No text truncation
- No decorative headers
- ISO 8601 timestamps instead of "about 1 day ago"

### 3.2 JSON Output Mode

**Field selection**:
```bash
gh pr list --json number,title,author
```

**Output**:
```json
[
  {
    "author": {"login": "monalisa"},
    "number": 123,
    "title": "A helpful contribution"
  }
]
```

**Available JSON fields are documented per command**:
```bash
gh pr list --json          # Lists all available fields
# additions, assignees, author, baseRefName, body, changedFiles, ...
```

**Agent-Friendly Features**:
- Agents request only needed fields (reduces token usage)
- Field names are discoverable programmatically
- Consistent schema across commands (author always has login)

### 3.3 JQ Integration

Built-in jq support without requiring jq installation:

```bash
# Extract just author logins
gh pr list --json author --jq '.[].author.login'

# Complex filtering
gh issue list --json number,title,labels --jq \
  'map(select((.labels | length) > 0)) | .[:3]'
```

### 3.4 Go Template Formatting

For custom human-readable output:

```bash
gh pr list --json number,title --template \
  '{{range .}}{{printf "#%v" .number | autocolor "green"}} {{.title}}{{"\n"}}{{end}}'
```

**Template helpers**:
- `autocolor`: colorize only in TTY
- `timeago`: relative timestamps
- `tablerow`/`tablerender`: aligned tables
- `truncate`: length limits
- `hyperlink`: clickable terminal links

---

## 4. Scriptability Features

### 4.1 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Cancelled by user (Ctrl-C) |
| 4 | Authentication required |

**Agent Usage**: Agents can branch on exit codes without parsing output.

### 4.2 Environment Variables

**Authentication**:
```bash
GH_TOKEN=xxx gh pr list           # Token for github.com
GH_ENTERPRISE_TOKEN=xxx gh pr list # Token for GHE
```

**Behavior control**:
```bash
GH_HOST=github.mycompany.com      # Default host
GH_REPO=owner/repo                # Default repository
GH_PROMPT_DISABLED=1              # Never prompt (fail instead)
NO_COLOR=1                        # Disable colors
GH_DEBUG=1                        # Verbose debugging
GH_PAGER=cat                      # Disable pagination
```

**Agent-Critical**: `GH_PROMPT_DISABLED=1` ensures commands fail fast with error codes rather than hanging on interactive prompts.

### 4.3 Stdin/Stdout Patterns

**Reading from stdin**:
```bash
echo "Bug description" | gh issue create --title "Bug" --body-file -
cat payload.json | gh api repos/{owner}/{repo}/issues --input -
```

**Single-dash convention**: `-` means stdin/stdout, enabling pipes.

### 4.4 Non-Interactive Mode

When `stdin` is not a TTY or `GH_PROMPT_DISABLED=1`:
- Commands proceed with defaults or fail with clear errors
- No interactive prompts that would hang scripts
- Status output goes to stderr, data to stdout

---

## 5. API Escape Hatch

The `gh api` command provides direct GitHub API access:

```bash
# GET request with placeholder expansion
gh api repos/{owner}/{repo}/releases

# POST with typed parameters
gh api repos/{owner}/{repo}/issues -f title="Bug" -F labels[]="bug"

# GraphQL
gh api graphql -f query='{ viewer { login }}'

# Pagination handling
gh api --paginate repos/{owner}/{repo}/issues --jq '.[].title'
```

**Agent Power Features**:
- `{owner}` and `{repo}` auto-populate from git context
- `--paginate` handles multi-page results automatically
- `--cache 3600s` enables response caching
- `--silent` suppresses output for side-effect-only calls

---

## 6. Extensibility

### 6.1 Aliases

```bash
gh alias set pv 'pr view'
gh alias set --shell igrep 'gh issue list | grep $1'
```

### 6.2 Extensions

Third-party commands installed as `gh-<name>`:
```bash
gh extension install owner/gh-custom
gh custom [args]  # Uses the extension
```

---

## 7. Agent-Friendly Design Patterns (Synthesis)

### 7.1 Patterns to Adopt

| Pattern | Implementation | Benefit |
|---------|---------------|---------|
| **JSON output with field selection** | `--json field1,field2` | Token efficiency, predictable parsing |
| **TTY detection** | Automatic format switching | Works for humans and scripts |
| **Semantic exit codes** | 0/1/2/4 with documented meanings | No output parsing for status |
| **Environment variable control** | `TOOL_PROMPT_DISABLED`, `NO_COLOR` | Non-interactive mode |
| **Stdin flag** | `--input -`, `--body-file -` | Pipe-friendly composition |
| **Placeholder expansion** | `{owner}`, `{repo}` from context | Reduce required arguments |
| **Discoverable fields** | `--json` without args lists fields | Self-documenting |
| **Consistent subcommands** | resource -> action -> target | Predictable grammar |
| **Aliases for common operations** | `gh pr ls` = `gh pr list` | Convenience without breaking patterns |

### 7.2 Anti-Patterns to Avoid

| Anti-Pattern | Problem | Better Approach |
|--------------|---------|-----------------|
| Interactive prompts without bypass | Hangs in automation | `--no-input` flag or env var |
| Human-readable-only output | Fragile parsing | `--json` option |
| Inconsistent flag names | Cognitive load | Vocabulary consistency |
| Required positional arguments | Hard to script | Named flags with defaults |
| Colored output to pipes | Breaks parsing | TTY detection |
| Undocumented exit codes | Silent failures | Document all codes |

---

## 8. Recommendations for New CLI Tools

### 8.1 Minimum Viable Agent-Friendliness

1. **`--json` flag** on all data-returning commands
2. **Exit code 0/1** with additional codes for auth, cancellation
3. **`NO_COLOR` and `TOOL_PROMPT_DISABLED`** environment variables
4. **TTY detection** for format switching
5. **`--help`** with examples and field documentation

### 8.2 Enhanced Agent Support

1. **Field selection**: `--json field1,field2` to reduce payload
2. **Built-in jq**: `--jq 'expr'` for inline transformation
3. **Pagination handling**: `--paginate` or `--all` flags
4. **Stdin support**: `--input -` pattern for piping
5. **Context inference**: Auto-detect repo, project from environment

### 8.3 Output Schema Design

```json
{
  "items": [...],           // Primary data array
  "totalCount": 150,        // For pagination
  "pageInfo": {             // Cursor-based pagination
    "hasNextPage": true,
    "endCursor": "abc123"
  }
}
```

**Field naming**:
- Use semantic names: `createdAt` not `created_at` or `timestamp1`
- Nested objects for relationships: `author.login` not `author_login`
- ISO 8601 for all timestamps
- URLs as full strings, not requiring assembly

---

## 9. Ecosystem Context

### 9.1 Historical Evolution

- **POSIX utilities**: Single-letter flags, cryptic but composable
- **GNU coreutils**: Added `--long-options`
- **Git**: Established `command subcommand` pattern
- **Docker**: Popularized `docker <resource> <action>`
- **gh**: Synthesized all patterns with JSON-first scriptability

### 9.2 Related Tools Following Similar Patterns

| Tool | Similar Patterns |
|------|------------------|
| `kubectl` | `--output json`, resource/action structure |
| `aws` | `--output json`, subcommand hierarchy |
| `gcloud` | `--format json`, consistent flags |
| `jj` (Jujutsu) | Modern Git alternative with similar CLI design |

### 9.3 AI Agent Tooling Considerations

From Anthropic's agent tool design guidelines:
- **Consolidate multi-step operations**: One tool call vs. many reduces errors
- **Semantic field names**: `name` not `uuid` reduces hallucinations
- **Response format control**: `concise` vs `detailed` options
- **Helpful error messages**: Actionable guidance, not stack traces
- **Clear namespacing**: `asana_projects_search` not `search`

---

## 10. Confidence and Limitations

### Confidence Rating: High

**Strong evidence from**:
- Official gh documentation and `--help` output
- GitHub CLI source code (cli/cli repository)
- GitHub Engineering blog posts on design decisions
- POSIX/GNU standards for baseline conventions
- Anthropic's published agent tool guidelines

### Limitations

- Some patterns are GitHub-specific (OAuth flows, GraphQL API)
- Extension ecosystem not fully analyzed
- Performance characteristics not benchmarked
- Edge cases in TTY detection across platforms not tested

---

## Sources

- [GitHub CLI Manual](https://cli.github.com/manual/)
- [Scripting with GitHub CLI - GitHub Blog](https://github.blog/engineering/engineering-principles/scripting-with-github-cli/)
- [Scriptability Audit - Issue #940](https://github.com/cli/cli/issues/940)
- [Command Line Interface Guidelines](https://clig.dev/)
- [GNU Coding Standards - CLI](https://www.gnu.org/prep/standards/html_node/Command_002dLine-Interfaces.html)
- [POSIX Utility Conventions](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap12.html)
- [Writing Tools for Agents - Anthropic](https://www.anthropic.com/engineering/writing-tools-for-agents)
- [CLI Best Practices Collection](https://github.com/arturtamborski/cli-best-practices)

---

*Originally researched: 2026-02-01 | Imported: 2026-02-09*
