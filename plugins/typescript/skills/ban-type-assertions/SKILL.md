---
name: ban-type-assertions
description: Ban `as` type assertions in a package via the `@typescript-eslint/consistent-type-assertions` lint rule, replacing them with compiler-verified type-safe alternatives. Use when enabling the assertion ban in a new package or fixing violations in an existing one.
---

# Ban Type Assertions

Enable `@typescript-eslint/consistent-type-assertions` with `assertionStyle: 'never'` in a package and replace all `as X` casts with patterns the compiler can verify.

## Core Philosophy

> Pick the strictly correct path, not the simpler one.

Every `as` assertion is a spot where the developer told the compiler "trust me." The goal is to make the compiler *verify* instead. If you replace `as Foo` with a type guard that is equally unverified, you have not improved anything -- you have just moved the assertion.

## Quick Reference

- Rule: `@typescript-eslint/consistent-type-assertions`
- Config: `{ assertionStyle: 'never' }`
- Location: `packages/<name>/.eslintrc.js`

## Workflow

### 1. Enable the Rule

Add to the package's `.eslintrc.js`:

```js
rules: {
  '@typescript-eslint/consistent-type-assertions': ['error', { assertionStyle: 'never' }],
}
```

### 2. Enumerate Violations

```bash
cd packages/<name> && npm run lint 2>&1 | grep "consistent-type-assertions"
```

Group violations by file and pattern before fixing.

### 3. Research Before Fixing

Before writing any replacement code:

1. **Check for existing zod schemas** -- grep for `Schema` alongside the type name in `@factory/common` and across the repo.
2. **Check if schemas exist but aren't exported** -- if so, export them rather than creating new ones.
3. **Check for duplicate types/interfaces** across packages -- consolidate into `@factory/common` if found.
4. **Understand the data flow** -- is this a parse boundary (external data), a narrowing site (union type), or a library type gap?

### 4. Fix Violations Using the Pattern Hierarchy

#### Tier 1: Zod Parsing (for external data boundaries)

Use for any data entering the system from JSON, disk, network, IPC, etc. This gives **runtime validation**, not just a type annotation.

```typescript
// BAD
const data = JSON.parse(raw) as MyType;

// GOOD
const data = MySchema.parse(JSON.parse(raw));
```

Use `safeParse` when you need to handle errors gracefully (e.g., returning an error response with context like a request id):

```typescript
// BAD: throws before you can extract the request id
const request = RequestSchema.parse(JSON.parse(raw));

// GOOD: safeParse lets you return a proper error
const parsed = RequestSchema.safeParse(JSON.parse(raw));
if (!parsed.success) {
  return errorResponse(rawObj?.id ?? null, INVALID_PARAMS, parsed.error.message);
}
const request = parsed.data;
```

#### Tier 2: Control Flow Narrowing (for union types)

Use `switch`, `in`, `instanceof`, or discriminated unions:

```typescript
// BAD
(error as NodeJS.ErrnoException).code

// GOOD
if (error instanceof Error && 'code' in error) {
  const code = error.code;
}
```

```typescript
// BAD
if (METHODS.has(method as Method)) { ... }

// GOOD: switch narrows exhaustively
switch (method) {
  case 'foo':
  case 'bar':
    return handle(method); // narrowed
}
```

#### Tier 3: eslint-disable with Justification (last resort)

Only for genuinely unavoidable cases (library type gaps, generic parameters that can't be inferred). Always explain *why*:

```typescript
// eslint-disable-next-line @typescript-eslint/consistent-type-assertions -- ws library types require generic parameter
ws.on('message', handler);
```

#### Anti-Pattern: Type Guards That Are Disguised Assertions

```typescript
// NOT an improvement -- checks shape but not content
function isDaemonRequest(x: unknown): x is DaemonRequest {
  return typeof x === 'object' && x !== null && 'method' in x;
}
```

A zod schema validates values. A type guard like this is an unverified assertion with extra steps. Only use type guards when the narrowing logic is truly sufficient.

### 5. Use Strict Schemas, Not Permissive Ones

When a schema exists (e.g., `SessionSettingsSchema`), use it strictly rather than `z.record(z.unknown())`. This ensures forward compatibility -- if fields are removed in a migration, stale data gets cleaned on read.

```typescript
// BAD: accepts anything
const settings = z.record(z.unknown()).parse(raw);

// GOOD: validates against the real shape
const settings = SessionSettingsSchema.parse(raw);
```

### 6. Promote Shared Schemas to `@factory/common`

If you find duplicate interfaces, types, or schemas across packages, consolidate them:

1. Create the schema in `@factory/common/<domain>/<subdomain>/schema.ts`
2. Put any enums in a sibling `enums.ts` (required by `factory/enum-file-organization`)
3. Export via a **subpath** (e.g., `@factory/common/session/summary`), **not** the barrel `index.ts`
4. Delete all local duplicates
5. Update all consumers to import from the common subpath
6. Run `npm run knip` at repo root to catch unused barrel re-exports

### 7. Fix Test Mocks to Match Schemas

Once you replace `as X` with `.parse()`, test mocks that relied on the assertion will fail validation. Fix the mocks -- do not disable the rule in tests.

Create helper functions to centralize valid test fixtures:

```typescript
function mockSessionSummary(
  overrides?: Partial<SessionSummaryEvent>,
): SessionSummaryEvent {
  return {
    type: 'session_start',
    id: 'test-id',
    title: 'Test Session',
    owner: 'test-owner',
    ...overrides,
  };
}
```

### 8. Parse at the Boundary, Inside Error Handling

Make sure parsing happens where failures produce proper error responses, not unhandled exceptions:

```typescript
// BAD: parse outside try/catch -- if it throws, you lose context
const request = RequestSchema.parse(data);
try { handle(request); } catch { ... }

// GOOD: safeParse before try, handle error with context
const parsed = RequestSchema.safeParse(data);
if (!parsed.success) {
  return errorResponse(rawData?.id ?? null, INVALID_PARAMS, parsed.error.message);
}
try { handle(parsed.data); } catch { ... }
```

## Verification

Run for **all affected packages** (a change in `@factory/common` can break downstream lint):

```bash
# Lint (all affected packages)
cd packages/<name> && npm run lint

# Typecheck
npm run typecheck

# Tests
npm run test

# Unused exports (repo root)
npm run knip
```

## Reminders

- `factory/enum-file-organization` requires TypeScript enums to live in files named `enums.ts`
- `no-barrel-files` prevents re-exporting types from barrel files -- consumers must import from the subpath directly
- When promoting types to common, add a `package.json` exports entry for the new subpath if one doesn't exist
- Test overrides for the rule in `.eslintrc.js` may be needed if test files use assertion syntax in mock setup -- but prefer fixing mocks over disabling the rule
