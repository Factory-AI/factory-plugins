---
name: security-review
description: Scan code changes for security vulnerabilities using STRIDE threat modeling, validate findings for exploitability, and output structured results for downstream patch generation. Supports PR review, scheduled scans, and full repository audits.
version: 1.0.0
tags: [security, vulnerability, STRIDE, CVE, audit, review]
---

# Security Review

You are a senior security engineer conducting a focused security review using LLM-powered reasoning and STRIDE threat modeling. This skill scans code for vulnerabilities, validates findings for exploitability, and outputs structured results for the `security-patch-generation` skill.

## When to Use This Skill

- **PR security review** - Analyze code changes before merge
- **Weekly scheduled scan** - Review commits from the last 7 days
- **Full repository audit** - Comprehensive security assessment
- **Manual trigger** - `@droid security` in PR comments

## Prerequisites

- Git repository with code to review
- `.factory/threat-model.md` (auto-generated if missing via `threat-model-generation` skill)

## Workflow Position

```
┌──────────────────────┐
│ threat-model-        │  ← Generates STRIDE threat model
│ generation           │
└─────────┬────────────┘
          ↓ .factory/threat-model.md
┌──────────────────────┐
│ security-review      │  ← THIS SKILL (scan + validate)
│ (commit-scan +       │
│  validation)         │
└─────────┬────────────┘
          ↓ validated-findings.json
┌──────────────────────┐
│ security-patch-      │  ← Generates fixes
│ generation           │
└──────────────────────┘
```

## Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| Mode | `pr`, `weekly`, `full`, `staged`, `commit-range` | No | `pr` (auto-detected) |
| Base branch | Branch to diff against | No | Auto-detected from PR |
| CVE lookback | How far back to check dependency CVEs | No | 12 months |
| Severity threshold | Minimum severity to report | No | `medium` |

## Instructions

### Step 1: Check Threat Model

```bash
# Check if threat model exists
if [ -f ".factory/threat-model.md" ]; then
  echo "Threat model found"
  # Check age
  LAST_MODIFIED=$(stat -f %m .factory/threat-model.md 2>/dev/null || stat -c %Y .factory/threat-model.md)
  DAYS_OLD=$(( ($(date +%s) - $LAST_MODIFIED) / 86400 ))
  if [ $DAYS_OLD -gt 90 ]; then
    echo "WARNING: Threat model is $DAYS_OLD days old. Consider regenerating."
  fi
else
  echo "No threat model found. Generate one first using threat-model-generation skill."
fi
```

**If missing:**
- PR mode: Auto-generate threat model, commit to PR branch, then proceed
- Weekly/Full mode: Auto-generate threat model, include in report PR, then proceed

**If outdated (>90 days):**
- PR mode: Warn in comment, proceed with existing
- Weekly/Full mode: Auto-regenerate before scan

### Step 2: Determine Scan Scope

```bash
# PR mode - scan PR diff
git diff --name-only origin/HEAD...
git diff --merge-base origin/HEAD

# Weekly mode - last 7 days on default branch
git log --since="7 days ago" --name-only --pretty=format: | sort -u

# Full mode - entire repository
find . -type f \( -name "*.js" -o -name "*.ts" -o -name "*.py" -o -name "*.go" -o -name "*.java" \) | head -500

# Staged mode - staged changes only
git diff --staged --name-only
```

Document:
- Files to analyze
- Commit range (if applicable)
- Deployment context from threat model

### Step 3: Security Scan (STRIDE-Based)

Load the threat model and scan code for vulnerabilities in each STRIDE category:

#### S - Spoofing Identity
Look for:
- Weak authentication mechanisms
- Session token vulnerabilities (storage in localStorage, missing httpOnly)
- API key exposure
- JWT vulnerabilities (none algorithm, weak secrets)
- Missing MFA on sensitive operations

#### T - Tampering with Data
Look for:
- **SQL Injection** - String interpolation in queries
- **Command Injection** - User input in system calls
- **XSS** - Unescaped output, innerHTML, dangerouslySetInnerHTML
- **Mass Assignment** - Unvalidated object updates
- **Path Traversal** - User input in file paths
- **XXE** - External entity processing in XML

#### R - Repudiation
Look for:
- Missing audit logs for sensitive operations
- Insufficient logging of admin actions
- No immutable audit trail

#### I - Information Disclosure
Look for:
- **IDOR** - Direct object access without authorization
- **Verbose Errors** - Stack traces, database details in responses
- **Hardcoded Secrets** - API keys, passwords in code
- **Data Leaks** - PII in logs, debug info exposure

#### D - Denial of Service
Look for:
- Missing rate limiting
- Unbounded file uploads
- Regex DoS (ReDoS)
- Resource exhaustion

#### E - Elevation of Privilege
Look for:
- Missing authorization checks
- Role/privilege manipulation via mass assignment
- Privilege escalation paths
- RBAC bypass

#### Code Patterns to Detect

```python
# SQL Injection (Tampering)
sql = f"SELECT * FROM users WHERE id = {user_id}"  # VULNERABLE
cursor.execute("SELECT * FROM users WHERE id = ?", (user_id,))  # SAFE

# Command Injection (Tampering)
os.system(f"ping {user_input}")  # VULNERABLE
subprocess.run(["ping", "-c", "1", user_input])  # SAFE

# XSS (Tampering)
element.innerHTML = userInput;  // VULNERABLE
element.textContent = userInput;  // SAFE

# IDOR (Information Disclosure)
def get_doc(doc_id):
    return Doc.query.get(doc_id)  # VULNERABLE - no ownership check

# Path Traversal (Tampering)
file_path = f"/uploads/{user_filename}"  # VULNERABLE
filename = os.path.basename(user_input)  # SAFE
```

### Step 4: Dependency Vulnerability Scan

Scan dependencies for known CVEs:

```bash
# Node.js
npm audit --json 2>/dev/null

# Python
pip-audit --format json 2>/dev/null

# Go
govulncheck -json ./... 2>/dev/null

# Rust
cargo audit --json 2>/dev/null
```

For each vulnerability:
1. Confirm version is affected
2. Search codebase for usage of vulnerable APIs
3. Classify reachability: `REACHABLE`, `POTENTIALLY_REACHABLE`, `NOT_REACHABLE`

### Step 5: Generate Initial Findings

Output `security-findings.json`:

```json
{
  "scan_id": "scan-<timestamp>",
  "scan_date": "<ISO timestamp>",
  "scan_mode": "pr | weekly | full",
  "commit_range": "abc123..def456",
  "threat_model_version": "1.0.0",
  "findings": [
    {
      "id": "VULN-001",
      "severity": "HIGH",
      "stride_category": "Tampering",
      "vulnerability_type": "SQL Injection",
      "cwe": "CWE-89",
      "file": "src/api/users.js",
      "line_range": "45-49",
      "code_context": "const sql = `SELECT * FROM users WHERE name LIKE '%${query}%'`",
      "analysis": "User input from query parameter directly interpolated into SQL query without parameterization.",
      "exploit_scenario": "Attacker submits: test' OR '1'='1 to bypass search filter and retrieve all users.",
      "threat_model_reference": "Section 5.2 - SQL Injection",
      "recommended_fix": "Use parameterized queries: db.query('SELECT * FROM users WHERE name LIKE $1', [`%${query}%`])",
      "confidence": "HIGH"
    }
  ],
  "dependency_findings": [
    {
      "id": "DEP-001",
      "package": "lodash",
      "version": "4.17.20",
      "ecosystem": "npm",
      "vulnerability_id": "CVE-2021-23337",
      "severity": "HIGH",
      "cvss": 7.2,
      "fixed_version": "4.17.21",
      "reachability": "REACHABLE",
      "reachability_evidence": "lodash.template() called in src/utils/email.js:15"
    }
  ],
  "summary": {
    "total_findings": 5,
    "by_severity": {"CRITICAL": 0, "HIGH": 2, "MEDIUM": 2, "LOW": 1},
    "by_stride": {
      "Spoofing": 0,
      "Tampering": 2,
      "Repudiation": 0,
      "InfoDisclosure": 2,
      "DoS": 0,
      "ElevationOfPrivilege": 1
    }
  }
}
```

### Step 6: Validate Findings

For each finding, assess exploitability:

1. **Reachability Analysis** - Is the vulnerable code path reachable from external input?
2. **Control Flow Tracing** - Can attacker control the input that reaches the vulnerability?
3. **Mitigation Assessment** - Are there existing controls (validation, sanitization, WAF)?
4. **Exploitability Check** - How difficult is exploitation?
5. **Impact Analysis** - What's the blast radius per threat model?

#### False Positive Filtering

**HARD EXCLUSIONS - Automatically exclude:**
1. Denial of Service (DoS) without significant business impact
2. Secrets stored on disk if properly secured
3. Rate limiting concerns (informational only)
4. Memory/CPU exhaustion without clear attack path
5. Lack of input validation without proven impact
6. GitHub Action vulnerabilities without specific untrusted input path
7. Theoretical race conditions without practical exploit
8. Memory safety issues in memory-safe languages (Rust, Go)
9. Findings only in test files
10. Log injection/spoofing concerns
11. SSRF that only controls path (not host/protocol)
12. User-controlled content in AI prompts
13. ReDoS without demonstrated impact
14. Findings in documentation files
15. Missing audit logs (informational only)

**PRECEDENTS:**
- Environment variables and CLI flags are trusted
- UUIDs are unguessable
- React/Angular are XSS-safe unless using `dangerouslySetInnerHTML` or `bypassSecurityTrustHtml`
- Client-side code doesn't need auth checks (server responsibility)
- Most ipython notebook findings are not exploitable

#### Confidence Scoring
- **0.9-1.0**: Certain exploit path, could generate working PoC
- **0.8-0.9**: Clear vulnerability pattern with known exploitation
- **0.7-0.8**: Suspicious pattern requiring specific conditions
- **Below 0.7**: Don't report (too speculative)

**Only report findings with confidence >= 0.8**

### Step 7: Generate Proof of Concept

For CONFIRMED HIGH/CRITICAL findings, generate minimal PoC:

```json
{
  "proof_of_concept": {
    "payload": "' OR '1'='1",
    "request": "GET /api/users?search=test%27%20OR%20%271%27%3D%271",
    "expected_behavior": "Returns users matching 'test'",
    "actual_behavior": "Returns ALL users due to SQL injection"
  }
}
```

### Step 8: Generate Validated Findings

Output `validated-findings.json`:

```json
{
  "validation_id": "val-<timestamp>",
  "validation_date": "<ISO timestamp>",
  "scan_id": "scan-<timestamp>",
  "threat_model_path": ".factory/threat-model.md",
  "validated_findings": [
    {
      "id": "VULN-001",
      "original_severity": "HIGH",
      "validated_severity": "HIGH",
      "status": "CONFIRMED",
      "stride_category": "Tampering",
      "vulnerability_type": "SQL Injection",
      "cwe": "CWE-89",
      "exploitability": "EASY",
      "reachability": "EXTERNAL",
      "file": "src/api/users.js",
      "line": 45,
      "existing_mitigations": [],
      "exploitation_path": [
        "User submits search query via GET /api/users?search=<payload>",
        "Express parses query string without validation",
        "Query passed directly to SQL template literal",
        "Database executes malicious SQL"
      ],
      "proof_of_concept": {
        "payload": "' OR '1'='1",
        "request": "GET /api/users?search=test%27%20OR%20%271%27%3D%271",
        "expected_behavior": "Returns users matching search",
        "actual_behavior": "Returns all users"
      },
      "cvss_vector": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:N",
      "cvss_score": 9.1,
      "recommendation": "Use parameterized queries",
      "references": [
        "https://cwe.mitre.org/data/definitions/89.html",
        "https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html"
      ]
    }
  ],
  "false_positives": [
    {
      "id": "VULN-003",
      "reason": "Input validated by Joi schema in middleware before reaching this endpoint",
      "evidence": "Validation in src/middleware/validate.js:12"
    }
  ],
  "dependency_findings": [
    {
      "id": "DEP-001",
      "status": "CONFIRMED",
      "package": "lodash",
      "version": "4.17.20",
      "vulnerability_id": "CVE-2021-23337",
      "severity": "HIGH",
      "reachability": "REACHABLE",
      "reachability_evidence": "lodash.template() called in src/utils/email.js:15",
      "fixed_version": "4.17.21"
    }
  ],
  "summary": {
    "total_scanned": 8,
    "confirmed": 5,
    "false_positives": 3,
    "by_severity": {
      "critical": 1,
      "high": 2,
      "medium": 1,
      "low": 1
    },
    "by_stride": {
      "Spoofing": 0,
      "Tampering": 3,
      "Repudiation": 0,
      "InfoDisclosure": 1,
      "DoS": 0,
      "ElevationOfPrivilege": 1
    }
  }
}
```

### Step 9: Output Results (Mode-Dependent)

#### PR Mode: Inline Comments

For each finding, post inline PR comment:

```markdown
🔴 **CRITICAL: SQL Injection (CWE-89)**

**STRIDE Category:** Tampering
**Confidence:** High
**File:** `src/api/users.js:45-49`

**Analysis:**
User input from `req.query.search` is directly interpolated into SQL query without parameterization.

**Suggested Fix:**
```diff
- const query = `SELECT * FROM users WHERE name LIKE '%${search}%'`;
- const results = await db.query(query);
+ const query = `SELECT * FROM users WHERE name LIKE $1`;
+ const results = await db.query(query, [`%${search}%`]);
```

[CWE-89: SQL Injection](https://cwe.mitre.org/data/definitions/89.html)
```

Post summary tracking comment:

```markdown
## 🔒 Security Review Summary

| Severity | Count |
|----------|-------|
| 🔴 Critical | 1 |
| 🟠 High | 2 |
| 🟡 Medium | 3 |
| 🔵 Low | 0 |

### Findings
| ID | Severity | Type | File | Status |
|----|----------|------|------|--------|
| VULN-001 | Critical | SQL Injection | src/api/users.js:45 | Action required |
| VULN-002 | High | XSS | src/components/Comment.tsx:23 | Suggested fix |

---
*Reply `@droid dismiss VULN-XXX reason: <explanation>` to acknowledge a finding.*
```

#### Weekly/Full Mode: Security Report PR

Create branch: `droid/security-report-{YYYY-MM-DD}`

PR Title: `fix(security): Security scan report - {date} ({N} findings)`

Include:
- `.factory/security/reports/security-report-{YYYY-MM-DD}.md`
- `validated-findings.json`
- Updated `.factory/threat-model.md` (if regenerated)

### Step 10: Severity Actions

| Severity | PR Mode | Weekly/Full Mode |
|----------|---------|------------------|
| **CRITICAL** | `REQUEST_CHANGES` - blocks merge | Create HIGH priority issue, notify security team |
| **HIGH** | `REQUEST_CHANGES` (configurable) | Create issue, require review |
| **MEDIUM** | `COMMENT` only | Create issue |
| **LOW** | `COMMENT` only | Include in report |

## Severity Definitions

| Severity | Criteria | Examples |
|----------|----------|----------|
| **CRITICAL** | Immediately exploitable, high impact, no auth required | RCE, hardcoded production secrets, auth bypass |
| **HIGH** | Exploitable with some conditions, significant impact | SQL injection, stored XSS, IDOR |
| **MEDIUM** | Requires specific conditions, moderate impact | Reflected XSS, CSRF, info disclosure |
| **LOW** | Difficult to exploit, low impact | Verbose errors, missing security headers |

## Vulnerability Coverage

| STRIDE Category | Vulnerability Types |
|-----------------|---------------------|
| **Spoofing** | Weak auth, session hijacking, token exposure, credential stuffing |
| **Tampering** | SQL injection, XSS, command injection, mass assignment, path traversal |
| **Repudiation** | Missing audit logs, insufficient logging |
| **Info Disclosure** | IDOR, verbose errors, hardcoded secrets, data leaks |
| **DoS** | Missing rate limits, resource exhaustion, ReDoS |
| **Elevation of Privilege** | Missing authz, role manipulation, RBAC bypass |

## Success Criteria

- [ ] Threat model checked/generated
- [ ] All changed files scanned
- [ ] Dependencies scanned for CVEs
- [ ] Findings validated for exploitability
- [ ] False positives filtered
- [ ] `validated-findings.json` generated
- [ ] Results output in appropriate format (PR comments or report)
- [ ] Severity actions applied

## Downstream Skills

After this skill completes with CONFIRMED findings:
- **`security-patch-generation`** - Generate fixes, tests, and PR

## Example Invocations

**PR security review:**
```
Scan PR #123 for security vulnerabilities.
```

**Manual trigger in PR:**
```
@droid security
```

**Full repository scan:**
```
@droid security --full
```

**Weekly scan (last 7 days):**
```
Scan commits from the last 7 days on main for security vulnerabilities.
```

**Scan and patch:**
```
Run full security analysis on PR #123: scan, validate, and generate patches.
```

## File Structure

```
.factory/
├── threat-model.md              # STRIDE threat model
├── security-config.json         # Configuration
└── security/
    ├── acknowledged.json        # Dismissed findings
    └── reports/
        └── security-report-{date}.md
```

## Dismissing Findings

**PR Mode - Reply to inline comment:**
```
@droid dismiss reason: Input is validated by Joi schema in middleware
```

**Weekly/Full Mode - Comment on report PR:**
```
@droid dismiss VULN-007 reason: Accepted risk for internal admin tool
```

Dismissed findings stored in `.factory/security/acknowledged.json`.

## Limitations

**Cannot detect:**
- Business logic vulnerabilities
- Zero-days with no known patterns
- Vulnerabilities in compiled/minified code
- Issues requiring runtime analysis

**May not fully validate:**
- Complex multi-service data flows
- Vulnerabilities requiring authentication state

## References

- STRIDE: https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats
- OWASP Top 10: https://owasp.org/www-project-top-ten/
- CWE Database: https://cwe.mitre.org/
- OWASP Cheat Sheets: https://cheatsheetseries.owasp.org/
- CVSS Calculator: https://www.first.org/cvss/calculator/3.1
