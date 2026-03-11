---
name: wiki
version: 1.0.0
description: |
  Generate comprehensive codebase documentation for a repository.
  Uploads the wiki to view in the Factory app.
user-invocable: true
disable-model-invocation: true
---

# Wiki generation

Read a repository, then produce a set of interconnected documentation pages that explain what the code does and how it fits together. The output is a `droid-wiki/` directory of markdown files, uploaded to Factory via `droid wiki-upload`.

## 1. Survey the repository

Before writing anything, build a mental model of the codebase.

Read these files (when they exist):

- `README.md`, `AGENTS.md`, `CONTRIBUTING.md` — project intent and conventions
- `package.json`, `Cargo.toml`, `go.mod`, `pyproject.toml` — dependencies and scripts
- `docs/` directory — existing documentation
- Entry points (`src/index.ts`, `main.go`, `app.py`, etc.) — how the application starts
- CI/CD config (`.github/workflows/`, `.gitlab-ci.yml`, `Jenkinsfile`, `azure-pipelines.yml`, etc.)
- Build tool config (`webpack.config.*`, `vite.config.*`, `Makefile`, `build/`, `Gulpfile.*`, etc.)
- Lint/quality config (`eslint.config.*`, custom lint plugins, `rustfmt.toml`, `.golangci.yml`, etc.)
- Directory listing of the project root and key subdirectories

Build a map of:

- **What the project does** — its purpose in one or two sentences
- **Major subsystems** — the main areas of the codebase (e.g., API layer, database models, CLI, frontend components)
- **Key data flows** — how data moves through the system (request → handler → database → response)
- **External dependencies** — databases, APIs, message queues, third-party services
- **Build and test commands** — how to build, test, and run the project

### Exhaustive subsystem discovery

After listing the major subsystems, walk every top-level source directory (and one level below) to check for subsystems you missed. For each directory that contains its own service, module, or feature, decide:

- **Tier 1** — core subsystems most contributors will encounter. Full dedicated page.
- **Tier 2** — important but specialized. Shorter dedicated page.
- **Tier 3** — niche or thin wrapper. A paragraph in an "Other subsystems" page with directory pointers.

Small repos may only need a few domain pages. Large repos should have as many as the codebase warrants. Do not cap arbitrarily — let the repo's actual structure determine coverage.

### Often-missed areas

After scanning the source tree, check for these commonly overlooked areas:

- **Custom lint/analysis rules** — plugins or config that enforce project-specific conventions
- **Automation workflows** — CI/CD, bots, scheduled jobs, code generation scripts
- **CLI or dev tools** — internal tools, scripts in `scripts/`, `tools/`, or `bin/`
- **Test infrastructure** — custom test frameworks, fixtures, or automation harnesses beyond standard test runners
- **Multi-language components** — if the repo has code in a second language (e.g., Rust CLI in a TypeScript project), document it

If any of these are non-trivial, they deserve coverage — either as their own page or as a section in a related page.

## 2. Plan the table of contents

Design a page tree before writing any prose. The wiki has three tiers of content: always-present pages, organizational lenses, and conditional sections.

### Always-present pages

These pages appear in every wiki, in this order:

1. `overview/` — introductory material grouped under one section
   - `index.md` — project overview: what it does, who uses it, quick links
   - `architecture.md` — system architecture with Mermaid diagrams
   - `getting-started.md` — prerequisites, install, build, test, run
   - `glossary.md` — project-specific terms and domain vocabulary
2. `by-the-numbers.md` — codebase statistics snapshot (see below)
3. `lore.md` — timeline and history of the codebase (see below)
4. `how-to-contribute/` — how to work in this codebase
   - `index.md` — work pickup, PR process, review expectations, definition of done
   - `development-workflow.md` — branch, code, test, PR, merge cycle
   - `testing.md` — frameworks, patterns, how to run, mock, and cover
   - `debugging.md` — logs, common errors, troubleshooting runbook
   - `patterns-and-conventions.md` — error handling, coding style, cross-cutting concerns
   - `tooling.md` — build system, linters, code generators, CI tooling (if the repo's tooling is the product itself, promote this to a top-level section instead)

### By the numbers

A top-level `by-the-numbers.md` page that gives a quantitative snapshot of the codebase. Start the page with a "Data collected on [date]" note so readers know how current the numbers are.

Include these sections:

- **Size** — lines of code by language (with a Mermaid horizontal bar chart), total source files vs test files vs config files, package/module count for monorepos
- **Activity** — commits per week/month (recent trend), most actively changed files/directories in the last 90 days (churn hotspots)
- **Bot-attributed commits** — percentage of commits with bot co-authorship (e.g., `Co-authored-by: factory-droid[bot]`, `dependabot[bot]`, `github-actions[bot]`, `copilot[bot]`). This is a lower bound on AI-assisted work since inline AI tools like Copilot leave no trace in git history. Be transparent about what's counted.
- **Complexity** — average file size by directory, deepest import chains, number of exported symbols per package

Use Mermaid `xychart-beta` (horizontal bar charts) for language breakdown and any other stat where a visual helps. Do NOT use Mermaid `pie` charts — they are not supported by the renderer. Use tables for lists of files/directories.

**Never include individual contributor stats** (top committers, lines per person, leaderboards). The by-the-numbers page is about the codebase, not the people. Per-person metrics create toxic comparisons and don't belong in team documentation. The `maintainers.md` page handles ownership mapping separately.

**Inline stats in other pages:** In addition to this summary page, weave relevant stats into existing pages:
- Language breakdown in `architecture.md`
- Churn hotspots in `cleanup-opportunities/` (if that section exists)
- File counts, bus factor (unique committers), and test-to-code ratio per subsystem on each domain page
- Dependency counts in `reference/dependencies.md`

### Lore

A top-level `lore.md` page that tells the story of how the codebase evolved. This is a narrative history, not a technical reference. It answers "what happened here and when?"

**Boundaries with other sections:**
- `by-the-numbers.md` = current snapshot (what the codebase looks like today)
- `lore.md` = timeline and history (what changed and when)
- `fun-facts.md` = light trivia (easter eggs, amusing discoveries)
- `background/` = technical rationale (why decisions were made)

**Every event, era, and milestone must include a date or month** (e.g., "Mar 2023", "Q4 2024"). Derive dates from git commit timestamps, tag dates, and file creation dates. If an exact date isn't available, use the month of the earliest relevant commit.

Include these sections:

- **Eras** — group the codebase history into 3-8 major phases, each with a short narrative description and key event bullet points. Derive from git history: tag dates, large merge commits, contributor patterns, directory creation dates. Example: "The TypeScript Migration (Mar–Aug 2023): The entire backend was rewritten from JavaScript to TypeScript over 5 months..."
- **Longest-standing features** — code or subsystems that have survived the most refactors and are still actively used. Include when they were first introduced and how many changes they've weathered.
- **Deprecated features** — things that were built, used, and then removed or replaced. Identify from directory names, README mentions, obvious `@deprecated` annotations, and removed routes. What was the feature, when was it introduced, when was it deprecated, and what replaced it.
- **Major rewrites** — large changes that touched many files. What existed before, what replaced it, and when. Derive from git history (large PRs, branch names with "migration" or "rewrite").
- **Growth trajectory** — how the codebase expanded over time: when packages/apps were added, contributor growth signals from git log.

**Speculation:** When the "why" behind a change isn't clear from commits, use natural hedging language ("appears to have been", "likely replaced due to"). No special formatting for speculative content.

### Organizational lenses

Five lenses are available for organizing the codebase deep-dives. Use any combination based on what the repo actually contains. At least one lens is required. Most repos use 2-3. The **features** lens is strongly encouraged -- it's the most intuitive entry point for new engineers ("what does this thing do?"). Even small repos typically have user-visible or developer-visible capabilities worth documenting. Only skip it if the repo is a single-purpose library with no distinct features.

| Concept | Default label | Also called | When to use |
|---|---|---|---|
| Deployable units | `applications/` | `services/`, `apps/` | Repo ships multiple distinct runtimes |
| Internal building blocks | `systems/` | `services/`, `modules/`, `subsystems/` | Architectural components that don't map to a single app or package |
| Cross-cutting capabilities | `features/` | `capabilities/`, `workflows/` | User-visible or developer-visible things that span multiple systems |
| Workspace packages | `packages/` | `libraries/`, `crates/`, `modules/` | Monorepo with shared libraries worth documenting individually |
| Foundational domain objects | `primitives/` | `core-concepts/`, `domain-models/`, `entities/` | Types/concepts that appear across 3+ systems (e.g., session, user, message) |

**Choosing labels:** Mirror the repo's own vocabulary. If the repo has an `apps/` directory, call the section `apps/`, not `applications/`. If the repo calls things "services," use `services/`. The default labels are fallbacks for when the repo has no existing convention.

**Placement rules:**
- Place each concept where the repo's structure suggests it belongs. If agent logic lives in `packages/droid-core`, document it under packages, not systems.
- The systems lens is for things that don't have a natural home in apps or packages -- emergent architectural patterns, cross-package systems, infrastructure that spans multiple directories.
- Do not duplicate content across lenses. If something is documented under packages, the relevant app page should cross-link to it, not repeat it.

**Heuristics for identifying each lens:**
- If it has its own entry point and deployment, it's an **application**
- If it's a workspace package that other packages import, it's a **package**
- If it's a module with internal logic and clear boundaries that doesn't map to a single package, it's a **system**
- If it's a type or concept that appears in 3+ systems, it's a **primitive**
- If understanding it requires tracing through multiple systems or apps, it's a **feature**

### Conditional sections

Include these based on your judgment after surveying the repo. Skip any that don't apply.

- `api/` — if the repo exposes REST, GraphQL, WebSocket, or other APIs
- `deployment/` — if there's a non-trivial deployment process (CI/CD, environments, rollback, infrastructure)
- `security/` — if there are meaningful trust boundaries (auth, authorization, secrets, input validation)
- `background/` — if the repo has meaningful history (design decisions, pitfalls/danger zones, migration context)
- `how-to-monitor/` — if the repo runs as a service with logging, metrics, tracing, or alerting infrastructure
- `cleanup-opportunities/` — if the repo has dead code, accumulated TODOs/FIXMEs, oversized files, or outdated dependencies. Only include if there is actual content to report (see below)
- `fun-facts.md` — easter eggs, origin stories, oldest code, naming origins

### How to monitor

This conditional section documents how to see what the system is doing. Only generate it for repos that run as services with logging, metrics, or tracing infrastructure. Not applicable to libraries, CLI tools, or packages.

Sub-pages:

- `logging.md` — where logs go, how to query them, log levels and conventions, structured logging patterns, how to add new log statements
- `metrics.md` — what metrics are tracked, key SLIs/SLOs, available dashboards, how to add new metrics
- `tracing.md` — distributed tracing setup, how to trace a request end-to-end, span naming conventions, how to instrument new code paths
- `alerting.md` — what alerts exist, alert thresholds and rationale, escalation paths, known noisy alerts, how to add new alerts

Skip any sub-page the repo has no infrastructure for. If only one sub-page has content, collapse `how-to-monitor/` into a single `how-to-monitor.md` file instead of a directory.

### Cleanup opportunities

This conditional section surfaces actionable maintenance work. Only generate it if the scan finds meaningful content. Possible sub-pages:

- `dead-ends.md` — files, exports, or modules that nothing imports. The code equivalent of a ghost town.
- `todos-and-fixmes.md` — accumulated TODO, FIXME, and HACK comments with file locations. Include the oldest ones.
- `complexity-hotspots.md` — the largest source files, deepest nesting, or most complex functions. A gentle nudge toward refactoring.
- `dependency-freshness.md` — outdated or unmaintained dependencies. The oldest dependency still in use.

Skip any sub-page that has no findings. If only one sub-page has content, collapse `cleanup-opportunities/` into a single `cleanup-opportunities.md` file instead of a directory.

### Maintainers

Include a top-level `maintainers.md` page that maps subsystems to the people who know them. This page uses two data sources:

- **CODEOWNERS file** (if it exists) — official ownership assignments
- **Git blame / git log** — the 2-3 most recent or frequent committers per directory or subsystem

Present as a table:

```markdown
| Subsystem | Official owners (CODEOWNERS) | Recent contributors (git history) | Last activity |
|---|---|---|---|
| Authentication | @alice | alice, bob | 2 weeks ago |
| CLI | @charlie, @dave | charlie, eve | 3 days ago |
```

If the repo has no CODEOWNERS file, omit that column and derive all data from git history. If the repo has very few contributors (e.g., a solo project), skip this page entirely.

### Per-page active contributors

Each domain page (apps, systems, features, packages, primitives) should include an "Active contributors" byline as the very first line after the page heading, before the Purpose section:

```markdown
# Authentication

Active contributors: alice, bob

## Purpose
...
```

Derive the names from CODEOWNERS (if available) merged with the top 2-3 recent committers from git blame for that subsystem's directory. Use first names or GitHub usernames, no @ symbols.

**Exclude bot accounts** from contributor lists — filter out usernames ending in `[bot]` (e.g., `factory-droid[bot]`, `dependabot[bot]`, `github-actions[bot]`). Bots are not people you'd reach out to with questions. This applies to both the per-page active contributors byline and the maintainers page.

**Use the default branch for contributor data.** When deriving contributors from git blame or git log, always query against the default branch (`main` or `dev`), not the current branch. Feature branches skew contributor data toward whoever is working on that branch. Use `git log origin/main -- <path>` or `git log origin/dev -- <path>` to get accurate contributor history.

### Bottom sections

These appear at the end of every wiki:

- `reference/` — configuration, data models, external dependencies
- `maintainers.md` — subsystem ownership table (conditional, skip for solo projects; always the very last page)

### Page ordering

The full ordering in the wiki is:

1. overview/ (index, architecture, getting-started, glossary)
2. by-the-numbers.md (if present)
3. lore.md (if present)
4. fun-facts.md (if present)
5. how-to-contribute/
6. [organizational lenses, in whatever order makes sense]
7. [conditional sections: api, deployment, security, how-to-monitor, background, cleanup-opportunities]
8. reference/
9. maintainers.md (if applicable, always last)

### Nesting rules

- Any page can expand into a directory with sub-pages, except the four pages inside `overview/` (`index.md`, `architecture.md`, `getting-started.md`, `glossary.md`) which are always single files
- Maximum depth: 3 levels (e.g., `apps/cli/command-structure.md`)
- Every directory must contain an `index.md`
- For large repos (50+ source directories or 10+ distinct subsystems), lean toward splitting pages rather than cramming. A 3000-word page covering an entire subsystem is less useful than three focused pages covering its distinct aspects. Use all 3 levels of nesting when the repo's complexity warrants it.
- For small repos, default to single pages and only split when a topic has clearly distinct sub-areas
- Deployment and security start as single pages; expand to directories only if the repo has enough substance

### Naming rules

- Use lowercase filenames with hyphens: `getting-started.md`, not `GettingStarted.md`
- File names use lowercase with hyphens. No spaces, no uppercase.

## 3. Generate pages (with parallelization)

Page generation uses sub-agents to parallelize independent work. The top-level agent orchestrates the overall flow and writes pages that require accumulated context. Sub-agents handle sections that are self-contained.

### Execution DAG

```
Survey (top-level, serial)
  │
  ▼
overview/ (top-level, serial — sets tone and vocabulary)
  │
  ├────────────────────────────────────┐
  ▼                                    ▼
how-to-contribute/ (top-level)   ┌ by-the-numbers ┐
  │                              │ lore   │ (sub-agents, parallel)
  ▼                              └ fun-facts ──────┘
apps/ (top-level, serial)
  │
  ▼
systems/ (top-level, serial — references apps)
  │
  ▼
features/ (top-level, serial — references apps + systems)
  │
  ▼
packages/ (top-level, serial — references all above)
  │
  ▼
primitives/ (top-level, serial — references all above)
  │
  ├───────────────────────────────────────────────────┐
  ▼                                                   ▼
┌ api/ ──────────────┐                          ┌ reference/ ┐
│ deployment         │                          └ maintainers┘ (sub-agents)
│ security           │ (sub-agents, parallel)
│ how-to-monitor     │
│ background         │
└ cleanup-opportunities┘
```

### What the top-level agent writes (serial)

These pages require accumulated context and cross-referencing:

1. `overview/` — project overview, architecture, getting-started, glossary
2. `how-to-contribute/` — development workflow, testing, debugging, patterns, tooling
3. Organizational lens pages in order — apps → systems → features → packages → primitives (whichever apply). Each lens builds on the previous.

### What sub-agents write (parallel)

**Batch 1** — spawned after overview/ is complete, runs in parallel with how-to-contribute/:

- `by-the-numbers.md` — codebase statistics snapshot with Mermaid charts
- `lore.md` — timeline, eras, deprecated features, major rewrites, growth trajectory
- `fun-facts.md` — pick the most interesting 3-5 topics for the specific repo

These only need git history and source file structure. No cross-references to other wiki pages.

**Batch 2** — spawned after all lens pages are complete:

- Each conditional section as its own sub-agent: api, deployment, security, how-to-monitor, background, cleanup-opportunities (whichever apply)
- `reference/` + `maintainers.md` as one sub-agent

These sections are self-contained and don't cross-reference each other.

### Sub-agent context

Each sub-agent prompt must include:

1. **Brief repo summary** — 3-5 sentences about what the repo is, its tech stack, and structure. Extracted from the survey phase.
2. **Relevant file paths** — the specific source files the sub-agent should read for its section. The top-level agent identifies these during the survey.
3. **Writing conventions** — the page template (sections 3a-3e below), Mermaid guidelines, and cross-linking rules.
4. **Output directory** — where to write the generated .md files.

Sub-agents do NOT need the full survey context. They re-read only the files relevant to their section. This keeps their context window focused on writing quality content.

For each page:

### 3a. Read the relevant code

Open and read the actual source files for the section you are writing. Do not guess or hallucinate file contents. If a file is too large, read the parts that matter for the current section.

### 3b. Write prose

Explain what the code does in plain language. Start with the high-level purpose, then drill into specifics. Every claim should be traceable to a specific file or function.

Each domain page should include these sections (skip any that don't apply to the subsystem):

0. **Active contributors** — a one-line byline immediately after the heading (see "Per-page active contributors" in Section 2)
1. **Purpose** — what this subsystem does, in 2-3 sentences
2. **Directory layout** — a file tree showing the key files and folders
3. **Key abstractions** — a table of the most important types (classes, interfaces, traits, structs, functions) with their file path and a one-line description
4. **How it works** — the main data/control flow, with a Mermaid diagram if it involves 3+ components
5. **Integration points** — how this subsystem connects to others (what it imports, what calls it, what events it emits/listens to)
6. **Entry points for modification** — 2-3 sentences telling a developer where to start if they need to change or extend this subsystem

Let the complexity of the subsystem determine how long the page is. A thin wrapper might only need sections 1, 3, and 5. A complex subsystem might need all six with multiple diagrams.

### 3c. Add Mermaid diagrams

Use Mermaid diagrams to illustrate:

- **Architecture** — system components and how they connect
- **Data flows** — request lifecycle, event pipelines, processing stages
- **State machines** — authentication flows, order states, build pipelines

Mermaid diagram guidelines:

- Use `graph TD` or `graph LR` for architecture and flow diagrams
- Use `sequenceDiagram` for request/response flows between services
- Use `stateDiagram-v2` for state machines
- Keep diagrams focused — 5 to 15 nodes maximum. Split larger diagrams into multiple smaller ones
- Label edges with the action or data being passed
- Use subgraphs to group related components

Example:

````markdown
```mermaid
graph LR
    Client -->|HTTP request| APIGateway
    APIGateway -->|validate + route| Handler
    Handler -->|query| Database
    Database -->|rows| Handler
    Handler -->|JSON response| Client
```
````

Do not use Mermaid for simple relationships that a sentence can explain. A diagram should earn its place by showing something that is hard to describe in words.

### 3d. Add file references

Each domain page must include a **"Key source files"** table listing the most important files for that subsystem:

```markdown
| File | Purpose |
|---|---|
| `src/auth/middleware.ts` | Validates JWT tokens, attaches user to request |
| `src/auth/token-service.ts` | Token creation, refresh, and revocation |
```

The table should cover whatever files are important — don't pad it with trivial files and don't skip files just because there are few.

Reference every file you mention in prose. When mentioning a class, interface, function, or type, include its file path in backticks on first mention. Readers should be able to go from the documentation to the code in one step.

### 3e. Cross-link pages

Link between pages using relative markdown links:

```markdown
For details on how the auth middleware integrates with the API layer,
see [API authentication](../api/authentication.md).
```

Each page should link to at least one other page. The reader should be able to navigate the wiki without using the sidebar.

### 3f. Fun facts content

The `fun-facts.md` page is optional but encouraged. Pick the 3-5 most interesting topics for the specific repo from this list:

- **Oldest surviving code** — find the oldest file or function via git blame. How old is it? Has it changed much?
- **Dependency archaeology** — the oldest dependency still in use, or the one with the most major version bumps
- **Naming origins** — why is the project or its internal tools named what they are? Engineers name things weirdly and there's usually a story
- **TODO/FIXME count** — how many TODO/FIXME/HACK comments exist? What's the oldest one?
- **The longest file** — which source file has the most lines? A gentle call-out that doubles as a refactoring hint

Do not force all of these into every wiki. Pick only the ones where the repo has something genuinely interesting to say. If nothing stands out, skip fun-facts entirely.

## 4. Write the meta file

After generating all pages, create `.wiki-meta.json` in the wiki directory root. The `pageOrder` array is critical -- it controls the display order of pages in the wiki sidebar. List every generated file path in the exact order you want them to appear. Without this, pages sort alphabetically.

```json
{
  "generatedAt": "2025-01-15T10:30:00Z",
  "pageCount": 42,
  "topLevelSections": ["overview", "by-the-numbers", "lore", "fun-facts", "how-to-contribute", "apps", "systems", "features", "packages", "primitives", "api", "deployment", "security", "how-to-monitor", "background", "cleanup-opportunities", "reference", "maintainers"],
  "pageOrder": [
    "overview/index.md",
    "overview/architecture.md",
    "overview/getting-started.md",
    "overview/glossary.md",
    "by-the-numbers.md",
    "lore.md",
    "fun-facts.md",
    "how-to-contribute/index.md",
    "how-to-contribute/development-workflow.md",
    "how-to-contribute/testing.md",
    "how-to-contribute/debugging.md",
    "how-to-contribute/patterns-and-conventions.md",
    "how-to-contribute/tooling.md",
    "apps/index.md",
    "apps/cli/index.md",
    "apps/cli/command-structure.md",
    "apps/cli/tui-rendering.md",
    "apps/daemon.md",
    "systems/index.md",
    "systems/auth.md",
    "features/index.md",
    "features/wiki-generation.md",
    "packages/index.md",
    "packages/common.md",
    "primitives/index.md",
    "primitives/session.md",
    "api/index.md",
    "api/rest-endpoints.md",
    "deployment.md",
    "security.md",
    "how-to-monitor/index.md",
    "how-to-monitor/logging.md",
    "how-to-monitor/metrics.md",
    "how-to-monitor/tracing.md",
    "how-to-monitor/alerting.md",
    "background/index.md",
    "background/design-decisions.md",
    "background/pitfalls.md",
    "background/migration-context.md",
    "cleanup-opportunities/index.md",
    "cleanup-opportunities/dead-ends.md",
    "cleanup-opportunities/todos-and-fixmes.md",
    "cleanup-opportunities/complexity-hotspots.md",
    "cleanup-opportunities/dependency-freshness.md",
    "reference/index.md",
    "reference/configuration.md",
    "reference/data-models.md",
    "reference/dependencies.md",
    "maintainers.md"
  ]
}
```

The example above is abbreviated. In practice, list every `.md` file in the wiki directory. The order must match the page ordering defined in Section 2: overview → by-the-numbers → lore → fun-facts → how-to-contribute → lenses → conditional → reference → maintainers.

## 5. Upload

### Standard upload (local wiki directory)

When the user wants to keep a local copy (the default):

```bash
droid wiki-upload \
  --repo-url "$REPO_URL" \
  --wiki-dir ./droid-wiki
```

Arguments:

- `--repo-url` — the repository URL (the remote origin, e.g., `https://github.com/org/repo`)
- `--wiki-dir` — path to the directory containing the generated markdown files
- `--cleanup` — (optional) delete the wiki directory after a successful upload

### Remote-only upload (--no-local handling)

When the user asks to generate the wiki without leaving files on disk (e.g., the user says "don't leave files locally" or passes a `--no-local` flag):

```bash
# Create a temporary directory
WIKI_TMPDIR=$(mktemp -d)

# Write all wiki files to the temporary directory instead of ./droid-wiki
# ... generate pages into $WIKI_TMPDIR ...

# Upload with --cleanup to remove the temp directory after success
droid wiki-upload \
  --repo-url "$REPO_URL" \
  --wiki-dir "$WIKI_TMPDIR" \
  --cleanup
```

The `--cleanup` flag tells the CLI to delete the `--wiki-dir` directory after a successful upload. If the upload fails, the directory is preserved so the user can retry.

## Content principles

### Progressive disclosure

Start every page with a 1–3 sentence summary of what the page covers. Follow with an overview section that explains the main concepts. Put implementation details, edge cases, and configuration options later in the page.

A reader skimming the first paragraph of each page should get a useful overview of the entire system.

### Page size limit

Keep individual pages under 500KB. If a page approaches this limit, split it into sub-pages. For example, a large API reference page could become a directory with one page per endpoint group.

### Human writing rules

Write documentation that reads like a person wrote it. Technical docs are especially prone to AI-sounding patterns because the subject matter is dry. Fight that tendency.

**Specific rules to follow:**

1. **Cut inflated significance.** Do not write "serves as a testament to," "pivotal role in the evolving landscape," "setting the stage for," or "underscores the importance of." Just state what the thing does.

   Bad: "The authentication module serves as a critical pillar in the application's security landscape."
   Good: "The authentication module validates JWT tokens and attaches user context to requests."

2. **Cut promotional language.** Do not write "boasts," "vibrant," "rich," "profound," "showcasing," "exemplifies," "commitment to," "groundbreaking," "renowned," or "breathtaking." Technical docs describe; they do not sell.

   Bad: "The codebase boasts a rich set of vibrant utilities that showcase the team's commitment to developer experience."
   Good: "The `utils/` directory has helpers for string formatting, date parsing, and retry logic."

3. **Kill superficial -ing analyses.** Do not tack "highlighting," "ensuring," "reflecting," "symbolizing," "showcasing," or "contributing to" onto sentences to add fake depth.

   Bad: "The service processes events asynchronously, ensuring scalability while highlighting the system's robust architecture."
   Good: "The service processes events asynchronously. It pulls from an SQS queue and can handle ~500 events/second per instance."

4. **Avoid AI vocabulary words.** These words appear far more often in AI-generated text: additionally, crucial, delve, emphasizing, enduring, enhance, fostering, garner, interplay, intricate/intricacies, landscape (abstract), pivotal, showcase, tapestry (abstract), testament, underscore (verb), vibrant. Replace them with plainer alternatives.

5. **Skip the rule of three.** Do not force ideas into groups of three to sound comprehensive (e.g., "innovation, inspiration, and industry insights"). If there are two things, list two. If there are four, list four.

6. **Do not use copula avoidance.** Write "X is Y" or "X has Y" instead of "X serves as Y," "X stands as Y," "X represents Y," "X boasts Y," "X features Y," or "X offers Y."

   Bad: "The config module serves as the central hub for environment variable management."
   Good: "The config module reads environment variables and exports typed constants."

7. **Do not use negative parallelisms.** Avoid "It's not just X, it's Y" and "Not only X but Y" constructions.

8. **Use sentence case in headings.** Write "Getting started with authentication," not "Getting Started With Authentication."

9. **Cut filler phrases.** Replace "in order to" with "to," "due to the fact that" with "because," "it is important to note that" with nothing (just state the fact).

10. **Be specific, not vague.** Replace "industry experts believe" with a concrete reference. Replace "several components" with the actual component names. Replace "various configurations" with the actual config options.

11. **Avoid em dash overuse.** Use commas or periods instead of em dashes (—) in most cases. One em dash per page is fine; three or more is a pattern.

12. **Do not use chatbot artifacts.** Never write "I hope this helps," "Let me know if," "Here is an overview of," "Certainly!", or "Great question!" These are conversation patterns, not documentation.

### Concrete file references

Every factual claim about the code should point to the source file. Do not say "the system handles authentication" without saying where. Do not say "the database schema includes a users table" without pointing to the migration or model file.

If you cannot find the file that implements something, say so: "The retry logic is referenced in `config.ts` but the implementation was not found in the current codebase."

### Mermaid diagram usage

Include at least one Mermaid diagram in the architecture page. Include diagrams in domain pages when they help explain data flows or component relationships. Do not add diagrams to every page — a page about configuration options or environment variables probably does not need one.

## File structure specification

The generated wiki follows this layout:

```
droid-wiki/
├── .wiki-meta.json

# Always present (in this order)
├── overview/                             # Introductory material
│   ├── index.md                          # Project overview
│   ├── architecture.md                   # System architecture with Mermaid diagrams
│   ├── getting-started.md                # Prerequisites, install, build, test, run
│   └── glossary.md                       # Project-specific terms and vocabulary
├── by-the-numbers.md                     # Codebase statistics snapshot
├── lore.md                      # Timeline, eras, deprecated features, rewrites
├── fun-facts.md                          # Easter eggs, origin stories, oldest code
├── how-to-contribute/                    # How to work in this codebase
│   ├── index.md
│   ├── development-workflow.md
│   ├── testing.md
│   ├── debugging.md
│   ├── patterns-and-conventions.md
│   └── tooling.md

# Organizational lenses (use any combination, at least one required)
# Labels mirror the repo's own vocabulary
├── <apps|services|applications>/         # Deployable units
│   ├── index.md
│   ├── <simple-app>.md                   # Single page for simple apps
│   └── <complex-app>/                    # Directory for complex apps
│       ├── index.md
│       └── <sub-topic>.md
├── <systems|modules|subsystems>/         # Internal building blocks
│   ├── index.md
│   ├── <simple-system>.md
│   └── <complex-system>/                 # 3rd level for complex subsystems
│       ├── index.md
│       └── <sub-topic>.md
├── <features|capabilities|workflows>/    # Cross-cutting capabilities
│   ├── index.md
│   ├── <simple-feature>.md
│   └── <complex-feature>/                # Features that span many systems deserve sub-pages
│       ├── index.md
│       └── <sub-topic>.md
├── <packages|libraries|crates>/          # Workspace packages
│   ├── index.md
│   ├── <simple-package>.md
│   └── <complex-package>/
│       ├── index.md
│       └── <sub-topic>.md
├── <primitives|core-concepts|entities>/  # Foundational domain objects
│   ├── index.md
│   └── *.md

# Conditional sections (LLM judgment)
├── api/                                  # If the repo exposes APIs
│   ├── index.md
│   └── *.md
├── deployment.md                         # Single page or directory
├── security.md                           # Single page or directory
├── how-to-monitor/                        # Logging, metrics, tracing, alerting (services only)
│   ├── index.md
│   ├── logging.md                        # Where logs go, how to query, log levels
│   ├── metrics.md                        # What's tracked, dashboards, SLIs
│   ├── tracing.md                        # Distributed tracing, request tracing
│   └── alerting.md                       # Alerts, thresholds, escalation, noisy alerts
├── background/                           # Design decisions, pitfalls, migration context
│   ├── index.md
│   └── *.md
├── cleanup-opportunities/                # Dead code, TODOs, complexity hotspots, stale deps
│   ├── index.md
│   ├── dead-ends.md                      # Unused files, exports, modules
│   ├── todos-and-fixmes.md               # Accumulated TODO/FIXME/HACK comments
│   ├── complexity-hotspots.md            # Largest files, deepest nesting
│   └── dependency-freshness.md           # Outdated or unmaintained dependencies

# Always present (bottom)
├── reference/
│   ├── index.md
│   ├── configuration.md
│   ├── data-models.md
│   └── dependencies.md
└── maintainers.md                        # Subsystem ownership table (conditional, always last)
```

**Rules:**

- Every `.md` file must start with a level-1 heading (`# Title`). The upload tool extracts the title from this heading.
- Every directory must contain an `index.md`.
- File names use lowercase with hyphens. No spaces, no uppercase.
- The `.wiki-meta.json` file is for tracking purposes and is not uploaded as a page.
- The four pages inside `overview/` (`index.md`, `architecture.md`, `getting-started.md`, `glossary.md`) are always single files. All other pages can expand into directories with sub-pages.
- Maximum tree depth: 3 levels (e.g., `apps/cli/command-structure.md`).
- For large repos, use all 3 levels. A complex subsystem like an editor core or extension host should have its own directory with focused sub-pages, not a single monolithic page.
- Maximum 200 pages per wiki run. If a project needs more, prioritize the most important subsystems.
