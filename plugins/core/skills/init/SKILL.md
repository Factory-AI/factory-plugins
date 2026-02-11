---
name: init
version: 1.0.0
description: Initialize a new repository with AGENTS.md
disable-model-invocation: true
---

Please analyze this codebase and create a AGENTS.md file, which will be given to future instances of Droid to operate in this repository.

What to add:

1. Commands that will be commonly used, such as how to build, lint, and run tests. Include the necessary commands to develop in this codebase, such as how to run a single test.
2. High-level code architecture and structure so that future instances can be productive more quickly. Focus on the "big picture" architecture that requires reading multiple files to understand.

Usage notes:

- After analyzing the codebase, if no AGENTS.md exists, create it directly. If an AGENTS.md already exists, show the proposed new contents and ask for confirmation before modifying the file.
- When you make the initial AGENTS.md, do not repeat yourself and do not include obvious instructions like "Provide helpful error messages to users", "Write unit tests for all new utilities", "Never include sensitive information (API keys, tokens) in code or commits".
- Avoid listing every component or file structure that can be easily discovered.
- Don't include generic development practices.
- If there are Cursor rules (in .cursor/rules/ or .cursorrules), Copilot rules (in .github/copilot-instructions.md), or CLAUDE.md files, make sure to include the important parts.
- If there is a README.md, make sure to include the important parts.
- Do not make up information such as "Common Development Tasks", "Tips for Development", "Support and Documentation" unless this is expressly included in other files that you read.
