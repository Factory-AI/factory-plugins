# STRIDE Threat Model Template

This template defines the structure for `.factory/threat-model.md`. When generating a threat model, follow this structure and replace all `{placeholder}` values with actual content.

---

## Output File: `.factory/threat-model.md`

```markdown
# Threat Model for {Repository Name}

**Last Updated:** {YYYY-MM-DD}
**Version:** {X.Y.Z}
**Methodology:** STRIDE + Natural Language Analysis

---

## 1. System Overview

### Architecture Description

{Write a natural language description of the system, as if explaining to a security researcher. Include:}

This is a {type of application} that allows users to {primary functions}. The system is built using {technology stack} and consists of {number} main components:

1. **{Component Name}** - {Description of what it does and why it exists}
2. **{Component Name}** - {Description of what it does and why it exists}
3. **{Component Name}** - {Description of what it does and why it exists}

### Key Components

| Component   | Purpose   | Security Criticality | Attack Surface |
| ----------- | --------- | -------------------- | -------------- |
| {Component} | {Purpose} | {HIGH/MEDIUM/LOW}    | {Entry points} |
| {Component} | {Purpose} | {HIGH/MEDIUM/LOW}    | {Entry points} |

### Data Flow

{Describe how data moves through the system in natural language:}

When a user {action}, the system {process}. This involves {data flow description}. The data is validated at {points} and authenticated using {mechanism}.

---

## 2. Trust Boundaries & Security Zones

### Trust Boundary Definition

The system has **{N} trust zones**:

1. **Public Zone** - Untrusted external users and systems

   - Assumes: Malicious input, no authentication
   - Entry Points: {List all public entry points}

2. **Authenticated Zone** - Verified users with valid sessions

   - Assumes: User may be malicious but has valid credentials
   - Entry Points: {List protected endpoints}

3. **Internal Zone** - Service-to-service communication
   - Assumes: Services are trusted but data may be poisoned
   - Entry Points: {List internal APIs, databases}

### Authentication & Authorization

{Explain how auth works in natural language:}

Users authenticate using {method}. Sessions are managed via {mechanism} with {expiry}. Authorization is enforced using {RBAC/ABAC/custom} at {enforcement points}.

**Critical Security Controls:**

- {Control 1}
- {Control 2}
- {Control 3}

---

## 3. Attack Surface Inventory

### External Interfaces

#### Public HTTP Endpoints

{List all endpoints exposed to the internet:}

- `{METHOD} {/path}` - {Description}

  - **Input:** {Parameters and types}
  - **Validation:** {What validation is performed}
  - **Risk:** {Potential attack vectors}

- `{METHOD} {/path}` - {Description}
  - **Input:** {Parameters and types}
  - **Validation:** {What validation is performed}
  - **Risk:** {Potential attack vectors}

#### File Upload Endpoints

- `{METHOD} {/path}` - {Description}
  - **Input:** {File types, metadata}
  - **Validation:** {Type whitelist, size limits, malware scan}
  - **Risk:** {Malicious upload, path traversal, XXE}

### Data Input Vectors

The system accepts user input from:

1. {Input vector 1}
2. {Input vector 2}
3. {Input vector 3}

---

## 4. Critical Assets & Data Classification

### Data Classification

#### PII (Personally Identifiable Information)

- **{Data type}** - {How it's used}
- **{Data type}** - {How it's used}

**Protection Measures:** {Encryption, access controls, logging}

#### Credentials & Secrets

- **{Secret type}** - {How it's protected}
- **{Secret type}** - {How it's protected}

**Protection Measures:** {Secrets manager, rotation policy, never logged}

#### Business-Critical Data

- **{Data type}** - {Why it's critical}
- **{Data type}** - {Why it's critical}

---

## 5. Threat Analysis (STRIDE Framework)

### Understanding STRIDE for This System

We analyze threats using Microsoft's STRIDE methodology. Each category represents a different type of security threat.

---

### S - Spoofing Identity

**What is Spoofing?**
An attacker pretends to be someone or something they're not to gain unauthorized access.

#### Threat: {Threat Name}

**Scenario:** {Describe the attack scenario}

**Vulnerable Components:**

- {Component 1}
- {Component 2}

**Attack Vector:**
```

1. {Step 1}
2. {Step 2}
3. {Step 3}
4. {Outcome}

````

**Code Pattern to Look For:**
```{language}
// VULNERABLE: {Why this is vulnerable}
{vulnerable code example}

// SAFE: {Why this is safe}
{safe code example}
````

**Existing Mitigations:**

- {Mitigation 1}
- {Mitigation 2}

**Gaps:**

- {Gap 1}
- {Gap 2}

**Severity:** {CRITICAL/HIGH/MEDIUM/LOW} | **Likelihood:** {VERY HIGH/HIGH/MEDIUM/LOW}

---

### T - Tampering with Data

**What is Tampering?**
Unauthorized modification of data in memory, storage, or transit.

#### Threat: {Threat Name}

{Follow same structure as Spoofing section}

---

### R - Repudiation

**What is Repudiation?**
Users can deny performing actions because there's insufficient audit logging.

#### Threat: {Threat Name}

{Follow same structure as Spoofing section}

---

### I - Information Disclosure

**What is Information Disclosure?**
Exposing information to users who shouldn't have access.

#### Threat: {Threat Name}

{Follow same structure as Spoofing section}

---

### D - Denial of Service

**What is Denial of Service?**
Attacks that prevent legitimate users from accessing the system.

#### Threat: {Threat Name}

{Follow same structure as Spoofing section}

---

### E - Elevation of Privilege

**What is Elevation of Privilege?**
Gaining higher privileges than intended.

#### Threat: {Threat Name}

{Follow same structure as Spoofing section}

---

## 6. Vulnerability Pattern Library

### How to Use This Section

This section contains code patterns that indicate vulnerabilities. When analyzing code:

1. Look for these specific patterns
2. Consider the context (is input sanitized earlier?)
3. Check if mitigations are in place
4. Cross-reference with STRIDE threats above

---

### SQL Injection Patterns

```{language}
# PATTERN 1: String concatenation in SQL
{vulnerable pattern}

# PATTERN 2: Dynamic query building
{vulnerable pattern}

# SAFE ALTERNATIVE:
{safe pattern}
```

### XSS (Cross-Site Scripting) Patterns

```{language}
// PATTERN 1: innerHTML with user data
{vulnerable pattern}

// PATTERN 2: Unescaped template rendering
{vulnerable pattern}

// SAFE ALTERNATIVE:
{safe pattern}
```

### Command Injection Patterns

```{language}
# PATTERN 1: Shell command with user input
{vulnerable pattern}

# PATTERN 2: Eval-style functions
{vulnerable pattern}

# SAFE ALTERNATIVE:
{safe pattern}
```

### Path Traversal Patterns

```{language}
# PATTERN 1: User-controlled file paths
{vulnerable pattern}

# SAFE ALTERNATIVE:
{safe pattern}
```

### Authentication Bypass Patterns

```{language}
# PATTERN 1: Missing authentication check
{vulnerable pattern}

# PATTERN 2: Client-side role checking only
{vulnerable pattern}

# SAFE ALTERNATIVE:
{safe pattern}
```

### IDOR Patterns

```{language}
# PATTERN: Direct object access without authorization
{vulnerable pattern}

# SAFE ALTERNATIVE:
{safe pattern}
```

---

## 7. Security Testing Strategy

### Automated Testing

| Tool                 | Purpose                 | Frequency         |
| -------------------- | ----------------------- | ----------------- |
| {SAST tool}          | Static analysis         | Every commit      |
| {Dependency scanner} | Vulnerable dependencies | Daily             |
| {Secrets detection}  | Leaked credentials      | Every commit      |
| {DAST tool}          | Dynamic testing         | Weekly on staging |

### Manual Security Reviews

Human review is required for:

- HIGH/CRITICAL findings
- New authentication/authorization code
- Changes to cryptographic functions
- Admin privilege management changes

---

## 8. Assumptions & Accepted Risks

### Security Assumptions

1. **{Assumption}** - {Why we assume this is secure}
2. **{Assumption}** - {Why we assume this is secure}
3. **{Assumption}** - {Why we assume this is secure}

### Accepted Risks

1. **{Risk}** - {Why we're accepting it, mitigation timeline if any}
2. **{Risk}** - {Why we're accepting it, mitigation timeline if any}

---

## 9. Threat Model Changelog

### Version {X.Y.Z} ({YYYY-MM-DD})

- Initial threat model created
- STRIDE analysis completed for all components
- Vulnerability pattern library established

### Version {X.Y.Z} ({YYYY-MM-DD})

- {What changed}

```

---

## Guidelines for Using This Template

### Writing Style

1. **Use natural language** - Write as if explaining to a security researcher
2. **Include code examples** - Show vulnerable AND safe patterns
3. **Be specific** - Reference actual file paths, function names, endpoints
4. **Attack scenarios as narratives** - Step-by-step, numbered sequences

### Severity Ratings

| Severity | Definition |
|----------|------------|
| **CRITICAL** | Immediate exploitation possible, severe impact (data breach, RCE) |
| **HIGH** | Exploitation likely, significant impact (auth bypass, privilege escalation) |
| **MEDIUM** | Exploitation requires specific conditions, moderate impact |
| **LOW** | Difficult to exploit, minimal impact |

### Likelihood Ratings

| Likelihood | Definition |
|------------|------------|
| **VERY HIGH** | Trivial to exploit, commonly targeted |
| **HIGH** | Easy to exploit with basic skills |
| **MEDIUM** | Requires specific knowledge or conditions |
| **LOW** | Difficult to exploit, rarely targeted |

### LLM Optimization Tips

For maximum effectiveness with downstream security skills:

1. **Explicit code patterns** - LLMs match patterns better than prose descriptions
2. **Step-by-step attack vectors** - Numbered steps help trace exploitability
3. **Structured sections** - Consistent headings enable targeted retrieval
4. **Cross-references** - Link threats to specific code locations when known
```
