---
name: commit-security-scan
description: Analyze code changes for security vulnerabilities using LLM reasoning and threat model patterns. Use for PR reviews, pre-commit checks, or branch comparisons.
version: 1.0.0
tags: [security, scanning, vulnerability-detection, ci-cd]
---

# Commit Security Scan

Analyze code changes (commits, PRs, diffs) using LLM-powered reasoning to detect security vulnerabilities. This skill reads code directly and applies patterns from the repository's threat model to identify issues across all STRIDE categories.

## When to Use This Skill

- **PR review** - Automated security scan on pull requests
- **Pre-commit check** - Scan staged changes before committing
- **Branch comparison** - Review security of feature branch changes
- **Code review assistance** - Help reviewers spot security issues

## Prerequisites

This skill requires:

1. **Threat model** - `.factory/threat-model.md` must exist
2. **Security config** - `.factory/security-config.json` for severity thresholds

**IMPORTANT: If these files don't exist, you MUST generate them first before proceeding with the security scan.**

To generate the prerequisites:

1. Tell the user: "The threat model doesn't exist yet. I'll generate it first before scanning."
2. Run the `threat-model-generation` skill to create both files
3. Once complete, continue with the security scan

Do NOT ask the user to run the skill manually - just do it automatically as part of this workflow.

## Inputs

The skill determines what to scan from the user's request:

| Scan Type         | How to Specify             | Example                                      |
| ----------------- | -------------------------- | -------------------------------------------- |
| PR                | "Scan PR #123"             | `Scan PR #456 for security vulnerabilities`  |
| Commit range      | "Scan commits X..Y"        | `Scan commits abc123..def456`                |
| Single commit     | "Scan commit X"            | `Scan commit abc123`                         |
| Staged changes    | "Scan staged changes"      | `Scan my staged changes for security issues` |
| Uncommitted       | "Scan uncommitted changes" | `Scan working directory changes`             |
| Branch comparison | "Scan from X to Y"         | `Scan changes from main to feature-branch`   |
| Last N commits    | "Scan last N commits"      | `Scan the last 3 commits`                    |

If no scope is specified, prompt the user for clarification.

## Instructions

Follow these steps in order:

### Step 1: Verify Prerequisites (Auto-Generate if Missing)

Try to read these files:

- `.factory/threat-model.md`
- `.factory/security-config.json`

**If either file is missing or cannot be read:**

1. Inform the user: "The security threat model doesn't exist yet. I'll generate it first - this may take a minute."
2. Invoke the `threat-model-generation` skill to analyze the repository and create both files
3. Once generation completes, continue with Step 2

This ensures the security scan always has the threat model context it needs for accurate analysis.

### Step 2: Get Changed Files

Based on the user's request, get the list of changed files and their diffs using git:

- For PRs: use `gh pr diff`
- For commits/ranges: use `git diff` or `git show`
- For staged changes: use `git diff --cached`

Read the full content of each changed file for context.

### Step 3: Load Threat Model

Read `.factory/threat-model.md` and `.factory/security-config.json` to understand:

- The system's architecture and trust boundaries
- Known vulnerability patterns for this codebase
- Severity thresholds for findings

### Step 4: Analyze for Vulnerabilities

For each changed file, systematically check for STRIDE threats:

#### S - Spoofing Identity

- Missing or weak authentication checks
- Session handling vulnerabilities
- Token/credential exposure in code
- Insecure cookie settings

#### T - Tampering with Data

- **SQL Injection**: String concatenation/interpolation in SQL queries
- **Command Injection**: User input in shell commands, `eval()`, `exec()`
- **XSS**: Unescaped user input in HTML/templates
- **Mass Assignment**: Blind assignment from request to model
- **Path Traversal**: User input in file paths without validation

#### R - Repudiation

- Missing audit logging for sensitive operations
- Insufficient error logging
- Log injection vulnerabilities

#### I - Information Disclosure

- **IDOR**: Direct object access without ownership verification
- Verbose error messages exposing internals
- Hardcoded secrets, API keys, credentials
- Sensitive data in logs or responses
- Debug endpoints exposed

#### D - Denial of Service

- Missing rate limiting on endpoints
- Unbounded resource consumption (file uploads, queries)
- Algorithmic complexity attacks (regex, sorting)
- Missing pagination on list endpoints

#### E - Elevation of Privilege

- Missing authorization checks on endpoints
- Role/permission bypass opportunities
- Privilege escalation through parameter manipulation

### Step 5: Assess Each Finding

For each potential vulnerability:

1. **Trace data flow**: Follow user input from source to sink

   - Where does the input come from? (request params, body, headers, files)
   - Does it pass through validation/sanitization?
   - Where does it end up? (database, shell, response, file system)

2. **Check for existing mitigations**:

   - Is there validation elsewhere in the codebase?
   - Are there middleware/decorators that protect this code?
   - Does the framework provide automatic protection?

3. **Determine severity**:

   - **CRITICAL**: Remote code execution, auth bypass, data breach
   - **HIGH**: SQL injection, XSS, IDOR, privilege escalation
   - **MEDIUM**: Information disclosure, missing security headers
   - **LOW**: Best practice violations, minor issues

4. **Assess confidence**:
   - **HIGH**: Clear vulnerable pattern, direct data flow, no mitigations
   - **MEDIUM**: Possible vulnerability, some uncertainty about context
   - **LOW**: Suspicious pattern, likely has mitigations we can't see

### Step 6: Generate Report

Create `security-findings.json` with this structure:

```json
{
  "scan_id": "scan-YYYY-MM-DD-XXX",
  "scan_date": "<ISO 8601 timestamp>",
  "scan_type": "pr|commit|range|staged|working",
  "commit_range": "<base>..<head>",
  "pr_number": null,
  "threat_model_version": "<from security-config.json>",
  "findings": [
    {
      "id": "VULN-001",
      "severity": "HIGH",
      "stride_category": "Tampering",
      "vulnerability_type": "SQL Injection",
      "cwe": "CWE-89",
      "file": "src/api/users.py",
      "line_range": "45-49",
      "code_context": "<vulnerable code snippet>",
      "analysis": "<explanation of why this is vulnerable>",
      "exploit_scenario": "<how an attacker could exploit this>",
      "threat_model_reference": "Section 5.2 - SQL Injection",
      "existing_mitigations": [],
      "recommended_fix": "<how to fix the vulnerability>",
      "confidence": "HIGH",
      "reasoning": "<why this confidence level>"
    }
  ],
  "summary": {
    "total_findings": 0,
    "by_severity": { "CRITICAL": 0, "HIGH": 0, "MEDIUM": 0, "LOW": 0 },
    "by_stride": {
      "Spoofing": 0,
      "Tampering": 0,
      "Repudiation": 0,
      "InfoDisclosure": 0,
      "DoS": 0,
      "ElevationOfPrivilege": 0
    },
    "files_analyzed": 0
  }
}
```

### Step 7: Report Results

1. Save findings to `security-findings.json`
2. Report summary to user (findings count by severity, triggered thresholds)
3. Check severity thresholds from `security-config.json` and note if any are triggered

## CWE Reference

Common CWE mappings for findings:

| Vulnerability Type       | CWE     |
| ------------------------ | ------- |
| SQL Injection            | CWE-89  |
| Command Injection        | CWE-78  |
| XSS (Reflected)          | CWE-79  |
| XSS (Stored)             | CWE-79  |
| Path Traversal           | CWE-22  |
| IDOR                     | CWE-639 |
| Missing Authentication   | CWE-306 |
| Missing Authorization    | CWE-862 |
| Hardcoded Credentials    | CWE-798 |
| Sensitive Data Exposure  | CWE-200 |
| Mass Assignment          | CWE-915 |
| Open Redirect            | CWE-601 |
| SSRF                     | CWE-918 |
| XXE                      | CWE-611 |
| Insecure Deserialization | CWE-502 |

## Example Invocations

**Scan a PR:**

```
Scan PR #123 for security vulnerabilities
```

**Scan staged changes before committing:**

```
Scan my staged changes for security issues
```

**Scan a feature branch:**

```
Scan changes from main to feature/user-auth for vulnerabilities
```

**Scan recent commits:**

```
Scan the last 5 commits for security issues
```

## References

- Analysis examples: `analysis-examples.md` (in this skill directory)
- Threat model: `.factory/threat-model.md`
- Security config: `.factory/security-config.json`
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
