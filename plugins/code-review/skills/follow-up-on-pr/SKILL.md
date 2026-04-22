---
name: follow-up-on-pr
description: Follow up on an existing PR by rebasing on the base branch, addressing reviewer comments, fixing CI issues, and pushing updates. Use when the user provides a PR URL or number and wants to get it ready for merge.
---

# Follow Up on PR

Take over an existing PR, bring it up to date, address all feedback, and push it to a merge-ready state. This skill is language- and framework-agnostic — substitute your project's actual build, lint, test, and format commands where examples are shown.

## Inputs

- **PR URL or number** (required): e.g. `https://github.com/<owner>/<repo>/pull/9996` or `#9996`
- **Branch name** (optional): If not provided, extract from PR metadata

## Workflow

### 1. Study the PR

Fetch the PR via `FetchUrl` (or `gh pr view`) to get:
- File changes (diffs)
- Reviewer comments (inline and general)
- CI workflow status and logs
- PR description and any linked ticket

Read all changed files in depth. Understand the intent and scope of the change.

**If the PR fixes a specific issue** (e.g. error report, user-reported bug): investigate the root cause before reviewing the code. Confirm the fix addresses the actual problem. PRs are sometimes already superseded by other fixes.

### 2. Fetch and Check Out the Branch

```bash
git fetch origin <branch-name> <base-branch>
git checkout <branch-name>
```

If the branch doesn't exist locally, `git checkout` will create a tracking branch from `origin/<branch-name>`.

### 3. Rebase on Latest Base Branch

```bash
git pull origin <base-branch> --rebase
```

**If conflicts occur:**
1. Read each conflicted file to understand both sides
2. Resolve conflicts, preserving the PR's intent while incorporating base-branch changes
3. `git add <resolved-files>`
4. `git rebase --continue`

**If rebase is clean:** Verify the state with `git log --oneline -5` and `git diff --stat origin/<base-branch>..HEAD`.

### 4. Address Reviewer Comments

Review comments were already fetched in step 1. For each unresolved comment:
1. Read the comment and understand what's being asked
2. Make the code change (or explain why it's not needed)
3. Add tests if requested
4. Commit the fix with a descriptive message

**Already-addressed comments:** Check if a reply already exists (`in_reply_to_id` field). Skip comments that have been resolved.

### 5. Run Local CI Checks

Run the project's lint, format, typecheck/compile, and test commands for the affected areas. Discover them by reading the repo root (e.g. `Makefile`, `package.json`, `pyproject.toml`, `Cargo.toml`, `go.mod`, `build.gradle`, `mix.exs`, `Gemfile`, `composer.json`, `justfile`, `Taskfile.yml`, `README.md`, or the CI workflow config).

Prefer filter / target flags to scope runs to the affected areas — it is faster than running the whole repo. Common patterns:

- JS/TS monorepos: `npm run test -- --filter=<workspace>`, `pnpm -r --filter <pkg> test`, `nx test <project>`
- Python: `pytest <path>`, `tox -e <env>`
- Rust: `cargo test -p <crate>`, `cargo clippy -p <crate>`
- Go: `go test ./<pkg>/...`, `golangci-lint run ./<pkg>/...`
- Java/Kotlin: `./gradlew :<module>:test`, `./mvnw -pl <module> test`
- Bazel: `bazel test //path/...`

See the `create-pr` skill's "CI Checks Reference" section for a broader template of local commands matching common CI checks.

**Distinguishing pre-existing failures from PR issues:**
Some CI failures exist on the base branch and are unrelated to the PR. If a failure occurs in a file not touched by the PR, verify it exists on the base branch too before spending time fixing it. Common pre-existing issues include module / package resolution errors for recently-added dependencies.

**E2E tests:** If the PR changes user-facing behavior (UI flow, defaults, keyboard handling, CLI output), E2E tests may break even if the code is correct. Read the failing test to understand what it expects, then update it to match the new behavior. Don't assume E2E failures are flaky — read them first.

### 6. Commit and Push

```bash
git add -A
git commit -m "<type>(<scope>): <description>"

git push origin <branch-name> --force-with-lease
```

**If a commit-scanning bot blocks the push** (Droid-Shield, GitGuardian, TruffleHog, etc.): This happens when unrelated test fixtures contain strings that look like secrets. The agent cannot override these. Tell the user to push manually or temporarily disable the scanner per your org's docs.

### 7. Reply to Reviewer Comments

Reply to each addressed comment on the PR so reviewers know their feedback was handled.

**For inline review comments** (the most common type):
```bash
# Reply to a specific inline comment thread
gh api repos/<owner>/<repo>/pulls/<N>/comments/<COMMENT_ID>/replies \
  -X POST \
  -f body="<explanation of what was done>"
```

**For general PR-level summary:**
```bash
gh pr comment <N> --body "<summary of all changes made>"
```

**To check thread resolution status** (optional):
```graphql
gh api graphql -f query='{
  repository(owner: "<owner>", name: "<repo>") {
    pullRequest(number: <N>) {
      reviewThreads(first: 20) {
        nodes {
          isResolved
          comments(first: 3) { nodes { body author { login } } }
        }
      }
    }
  }
}'
```

### 8. Update PR Description

If the changes made during follow-up are significant (new tests, architectural changes, additional scope), update the PR description:

```bash
gh pr edit <N> --body "<updated description>"
```

Use your org's PR template format. Update the testing section to reflect the additional tests added.

## Verification

Before considering the task complete, confirm:
- [ ] Branch is rebased on the latest base branch
- [ ] All reviewer comments are addressed with code changes
- [ ] Local lint, typecheck/compile, and tests pass for affected packages
- [ ] Changes are pushed to remote
- [ ] All reviewer comments have replies explaining what was done
- [ ] PR description is up to date
