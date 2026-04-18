---
name: create-pr
description: Create a pull request with Conventional Commits formatting, a templated body, and local verification. Use when the user asks to create a PR, open a PR, submit changes for review, or put code up for review.
---

# Create Pull Request

Create a PR with proper conventions: local verification, Conventional Commits title, a templated body, and an optional linked ticket.

## Prerequisites

Before starting, verify:
1. Current branch has commits not on the base branch (`git log origin/<base-branch>..HEAD --oneline`)
2. Branch is pushed to remote (`git push -u origin HEAD` if not)
3. No uncommitted changes that should be included (`git status`)

## Workflow

### 1. Understand the Changes

Run in parallel:
```bash
git log origin/<base-branch>..HEAD --oneline
git diff origin/<base-branch>..HEAD --stat
```

Determine:
- **What changed**: Which apps/packages were modified
- **Change type**: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`, `ci`, `build`, `revert`
- **Scope**: Primary workspace affected (use directory name or `monorepo` for cross-cutting changes)
- **Is this a code change?**: If the PR modifies code (not only docs, markdown, or config-only changes), run the local verification checklist in step 2 before creating the PR.

### 2. Local Verification (for code changes)

**Skip this step** if the PR only touches documentation, markdown files, or other non-code files. For any change that touches `.ts`, `.tsx`, `.js`, `.jsx`, `.css`, or similar source files, run these checks locally before creating the PR.

Use a filter flag (turbo `--filter`, nx `--projects`, pnpm `--filter`) to target only the affected workspaces when possible — it is faster than running the whole repo.

#### Typecheck
```bash
# Repo-wide
npm run typecheck
# Filtered to affected workspaces (preferred — faster)
npm run typecheck -- --filter=<workspace1> --filter=<workspace2>
```

#### Lint
```bash
# Autofix (preferred — fixes formatting + lint in one pass)
npm run fix
# Or filtered
npm run fix -- --filter=<workspace>
# Lint-only (no autofix)
npm run lint
```

#### Tests
```bash
# Repo-wide (slow — runs all workspaces)
npm run test
# Filtered to affected workspaces (preferred)
npm run test -- --filter=<workspace>
```

#### Additional checks (run when relevant)
- **Knip** (unused exports): `npm run knip` — always run if you added/removed exports.
- **Depcheck**: `npm run depcheck` — run if you changed dependencies.
- **Lockfile**: If you modified any `package.json`, run `npm install` at repo root and commit any `package-lock.json` changes. CI fails if the lockfile is out of date.
- **OpenAPI / codegen**: If your backend has an OpenAPI spec or generated client, regenerate and commit any changes.
- **Stylelint**: `npm run stylelint` — run if you changed CSS/style files.

Substitute your package manager and check names if you do not use npm / turbo.

### 3. Link to a Ticket (optional)

If your org uses an issue tracker, ask the user whether to:
- **Create a new ticket**: Use the appropriate tool (Linear, Jira, GitHub Issues, etc.)
- **Link an existing ticket**: Ask for the identifier (e.g. `TEAM-1234`, `JIRA-567`, `#42`)
- **Skip**: Only if user explicitly says no ticket is needed

Most CI systems can be configured to require the ticket identifier in the PR body. Follow your org's convention.

### 4. Format PR Title

Follow Conventional Commits: `type(scope): description`

- `type`: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`, `ci`, `build`, `revert`
- `scope`: Workspace name from your `apps/*` or `packages/*` directories, or `monorepo` / `repo` for cross-cutting changes. Multiple scopes can be comma-separated: `fix(a, b, c): ...`

Examples:
- `feat(web): add dark mode toggle`
- `fix(cli, daemon): load shell env at entrypoint`
- `chore(repo): bump dependencies`

### 5. Generate PR Body

Fill in all sections from your PR template. A typical template has four sections:

```markdown
## Description

<concise summary of what changed and why>

## Related Issue

Closes TEAM-XXXX
<!-- or: Part of TEAM-XXXX -->

## Potential Risk & Impact

<list risks, performance implications, technical debt>
<!-- Use "N/A" only if truly no risk -->

## How Has This Been Tested?

<describe testing performed: unit tests, manual testing, typecheck, lint>
```

### 6. Create the PR

```bash
gh pr create \
  --base <base-branch> \
  --head <branch-name> \
  --title "<type>(<scope>): <description>" \
  --body "<generated body>"
```

If the body is long, write it to a temp file and use `--body-file`:
```bash
gh pr create --base <base-branch> --head <branch> --title "..." --body-file /tmp/pr-body.md
```

### 7. Report Result

Return the PR URL to the user.

## CI Checks Reference (template)

These are typical checks that run on every PR. Map them to your repo's actual commands when adapting this skill.

### Always-run checks
| Check | What it does | Local equivalent |
|---|---|---|
| **Typecheck** | `npm run typecheck` | `npm run typecheck` (or `--filter=<workspace>`) |
| **ESLint** | `npm run lint` | `npm run lint` or `npm run fix` |
| **Prettier** | `npm exec prettier -- --check .` | `npm run format` (to fix) |
| **Stylelint** | `npm run stylelint` | `npm run stylelint` |
| **Depcheck** | `npm run depcheck` | `npm run depcheck` |
| **Tests** | `turbo run test` (or equivalent) | `npm run test -- --filter=<workspace>` |
| **Knip** | `npm run knip` | `npm run knip` |
| **Lockfile** | Fails if `npm install` would modify `package-lock.json` | Run `npm install` and commit lockfile |
| **PR Conventions** | Validates branch name, semantic title, ticket presence | Follow the formatting rules above |

### Conditional checks (run only when affected files change)
- **OpenAPI validation**: Triggered by backend API/spec changes. Regenerate locally.
- **Desktop/mobile build**: Triggered when those apps are affected.
- **E2E tests**: Triggered when the consumer app is affected.

### Typical PR conventions CI enforces
- **Branch name**: Max length, allowed characters (e.g. `[A-Za-z0-9/-]`).
- **Title**: Conventional Commits format with a valid scope.
- **Ticket reference**: PR body must contain a ticket identifier (often skipped for `chore:` and `revert:` types).

## Common Mistakes to Avoid

- **Wrong base branch**: Use the branch your org takes PRs into (e.g. `dev`, `main`, `develop`).
- **Missing scope**: PR title CI check often requires a valid scope.
- **Missing ticket reference**: Description must reference your ticket ID for CI to pass (except `chore:`/`revert:`).
- **Forgetting to push**: Branch must be on remote before `gh pr create`.
- **Lockfile drift**: Always run `npm install` and commit `package-lock.json` after dependency changes.
- **Skipping local checks on code PRs**: Typecheck, lint, and tests should be run locally before sending out code changes to catch issues early and avoid CI round-trips.
- **Uncommitted OpenAPI spec / generated client**: After backend API changes, regenerate and commit.
