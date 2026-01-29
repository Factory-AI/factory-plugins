---
name: init
version: 1.0.0
description: |
  Generate an AGENTS.md contributor guide for the current repository.
  Use when the user wants to create repository guidelines, initialize a project
  for AI-assisted development, or when they run the /init command.
---

# Repository initialization

Create an AGENTS.md file that serves as a contributor guide for AI agents working in this repository.

## Instructions

1. **Analyze the repository** by examining:
   - Project structure (directories, key files)
   - Build configuration (package.json, Makefile, Cargo.toml, etc.)
   - Existing documentation (README, CONTRIBUTING, etc.)
   - Git history for commit message conventions
   - Test setup and configuration
   - Linting and formatting tools

2. **Create AGENTS.md** in the repository root with these sections:

### Document requirements

- Title it "Repository Guidelines"
- Use markdown headings for structure
- Keep it concise: 200-400 words is optimal
- Be specific to this repository, not generic advice
- Include concrete examples (commands, paths, patterns)

### Recommended sections

**Project Structure & Module Organization**
- Describe the high-level organization (e.g., "monorepo with apps/ and packages/")
- Mention key entry points, not exhaustive file listings
- Avoid detailed file trees that will become outdated
- Focus on patterns and conventions, not current state

**Build, Test, and Development Commands**
- How to install dependencies
- How to run tests
- How to build the project
- How to run locally (if applicable)

**Coding Style & Naming Conventions**
- Indentation (tabs vs spaces, width)
- Naming patterns (camelCase, snake_case, etc.)
- Formatting and linting tools used
- Any style guides followed

**Testing Guidelines**
- Testing framework(s) used
- How to run specific tests
- Test naming conventions
- Coverage requirements (if any)

**Commit & Pull Request Guidelines**
- Commit message format (look at git log for patterns)
- PR requirements (descriptions, linked issues, etc.)

Add other sections if relevant to this specific project, such as:
- Security considerations
- Architecture overview
- Environment setup
- Deployment process

## Important

- Never overwrite an existing AGENTS.md file
- If AGENTS.md already exists, inform the user and offer to review or update it instead
- Focus on information that helps AI agents work effectively in this codebase
- Omit sections that don't apply to this project
