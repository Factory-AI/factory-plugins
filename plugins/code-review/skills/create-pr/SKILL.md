---
name: create-pr
description: Create a pull request with Conventional Commits formatting, a templated body, and local verification. Use when the user asks to create a PR, open a PR, submit changes for review, or put code up for review.
---

# Create Pull Request

Create a PR with proper conventions: local verification, Conventional Commits title, a templated body, and an optional linked ticket. This skill is language- and framework-agnostic — substitute your project's actual build, lint, test, and format commands where examples are shown.

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
- **What changed**: Which modules, packages, services, or directories were modified
- **Change type**: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`, `ci`, `build`, `revert`
- **Scope**: Primary module/package/service affected (use directory name or `monorepo` / `repo` for cross-cutting changes)
- **Is this a code change?**: If the PR modifies source code (not only docs, markdown, or config-only changes), run the local verification checklist in step 2 before creating the PR.

### 2. Local Verification (for code changes)

**Skip this step** if the PR only touches documentation, markdown files, or other non-code files. For any change that touches source files, run your project's verification commands locally before creating the PR.

Discover the commands by reading the repo root (e.g. `Makefile`, `package.json`, `pyproject.toml`, `Cargo.toml`, `go.mod`, `build.gradle`, `mix.exs`, `Gemfile`, `composer.json`, `justfile`, `Taskfile.yml`, `README.md`, or the CI workflow config). Use filter/target flags where available (e.g. `turbo --filter`, `nx --projects`, `pnpm --filter`, `bazel //path/...`, `cargo -p <crate>`, `pytest <path>`, `go test ./<pkg>/...`) to run only the affected portions — it is faster than running the whole repo.

Common verification categories to run when applicable:

#### Typecheck / Compile
Run the project's static type check or compile step if it has one.

Examples across ecosystems (use whatever the repo defines):
- TypeScript: `npm run typecheck`, `pnpm -r typecheck`, `tsc --noEmit`
- Python (typed): `mypy .`, `pyright`, `ty check`
- Rust: `cargo check`
- Go: `go build ./...`, `go vet ./...`
- Java/Kotlin: `./gradlew compileJava`, `./mvnw compile`

#### Lint / Format
Run the project's linter and formatter. Prefer an autofix target if one exists.

Examples:
- JS/TS: `npm run fix`, `npm run lint`, `eslint .`, `prettier --check .`
- Python: `ruff check --fix .`, `ruff format .`, `black .`, `flake8`
- Rust: `cargo clippy --all-targets`, `cargo fmt --check`
- Go: `golangci-lint run`, `gofmt -l .`
- Shell: `shellcheck`, `shfmt -d .`

#### Tests
Run the unit/integration tests for affected packages.

Examples:
- JS/TS: `npm run test -- --filter=<workspace>`, `pnpm -r test`, `vitest run <path>`, `jest <path>`
- Python: `pytest <path>`, `tox -e <env>`, `python -m unittest`
- Rust: `cargo test -p <crate>`
- Go: `go test ./<pkg>/...`
- Java/Kotlin: `./gradlew test`, `./mvnw test`
- Ruby: `bundle exec rspec <path>`, `rake test`

#### Additional checks (run when relevant)
- **Unused exports / dead code**: Run your project's dead-code check if it has one (e.g. `knip`, `ts-prune`, `vulture` for Python, `deadcode` / `unused` for Go, `cargo udeps` for Rust).
- **Dependency hygiene**: Run your project's dependency check if it has one (e.g. `depcheck`, `pip check`, `cargo audit`, `bundle audit`).
- **Lockfile in sync**: If you modified any dependency manifest (`package.json`, `requirements.txt`, `pyproject.toml`, `Cargo.toml`, `go.mod`, `Gemfile`, etc.), run the install command (`npm install`, `pnpm install`, `uv sync`, `poetry lock --no-update`, `cargo update -w`, `go mod tidy`, `bundle install`) and commit any lockfile changes. CI commonly fails if the lockfile is out of date.
- **Generated code / codegen**: If the repo has an OpenAPI spec, protobuf, GraphQL schema, SQL migrations, or any other generated artifacts, regenerate and commit any changes.
- **Style / asset linters**: Run stylesheet linters (`stylelint`, etc.) or asset linters if you changed those files.
- **Security scans**: Run any security/secret scanners configured in the repo (`trivy`, `semgrep`, `gitleaks`, etc.).

### 3. Link to a Ticket (optional)

If your org uses an issue tracker, ask the user whether to:
- **Create a new ticket**: Use the appropriate tool (Linear, Jira, GitHub Issues, etc.)
- **Link an existing ticket**: Ask for the identifier (e.g. `TEAM-1234`, `JIRA-567`, `#42`)
- **Skip**: Only if user explicitly says no ticket is needed

Most CI systems can be configured to require the ticket identifier in the PR body. Follow your org's convention.

### 4. Format PR Title

Follow Conventional Commits: `type(scope): description`

- `type`: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`, `ci`, `build`, `revert`
- `scope`: Name of the affected module/package/service/directory, or `monorepo` / `repo` for cross-cutting changes. Multiple scopes can be comma-separated: `fix(a, b, c): ...`

Examples:
- `feat(web): add dark mode toggle`
- `fix(cli, daemon): load shell env at entrypoint`
- `fix(api): handle nil response from upstream`
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

These are typical check categories that run on every PR. Map them to your repo's actual commands when adapting this skill.

### Always-run checks
| Category | What it does | How to find the local command |
|---|---|---|
| **Typecheck / compile** | Verifies the project compiles or passes static types | Check `package.json`, `Makefile`, `pyproject.toml`, `Cargo.toml`, `go.mod`, CI config |
| **Lint** | Enforces code style / correctness rules | Check for `lint`, `check`, or equivalent scripts in the repo root |
| **Format** | Enforces consistent formatting | Check for `format`, `fmt`, `prettier`, `black`, `gofmt`, `rustfmt`, etc. |
| **Tests** | Runs unit and integration tests | Check for `test` script / target |
| **Dead code / unused exports** | Flags unused code | Check for `knip`, `ts-prune`, `vulture`, `cargo udeps`, etc. |
| **Dependency check** | Flags unused / vulnerable dependencies | Check for `depcheck`, `audit`, `cargo audit`, etc. |
| **Lockfile in sync** | Fails if lockfile is stale relative to the manifest | Run your package manager's install command and commit the lockfile |
| **PR Conventions** | Validates branch name, semantic title, ticket presence | Follow the formatting rules above |

### Conditional checks (run only when affected files change)
- **API / schema validation**: Triggered by API or schema changes. Regenerate locally.
- **Platform-specific builds**: Triggered when desktop/mobile/embedded targets are affected.
- **E2E tests**: Triggered when the consumer app or top-level binary is affected.

### Typical PR conventions CI enforces
- **Branch name**: Max length, allowed characters (e.g. `[A-Za-z0-9/-]`).
- **Title**: Conventional Commits format with a valid scope.
- **Ticket reference**: PR body must contain a ticket identifier (often skipped for `chore:` and `revert:` types).

## Common Mistakes to Avoid

- **Wrong base branch**: Use the branch your org takes PRs into (e.g. `dev`, `main`, `develop`, `trunk`).
- **Missing scope**: PR title CI check often requires a valid scope.
- **Missing ticket reference**: Description must reference your ticket ID for CI to pass (except `chore:`/`revert:`).
- **Forgetting to push**: Branch must be on remote before `gh pr create`.
- **Lockfile drift**: Always run the install command and commit lockfile changes after dependency changes.
- **Skipping local checks on code PRs**: Typecheck/compile, lint, and tests should be run locally before sending out code changes to catch issues early and avoid CI round-trips.
- **Uncommitted generated artifacts**: After API/schema changes, regenerate and commit.
