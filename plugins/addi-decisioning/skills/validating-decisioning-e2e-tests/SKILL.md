---
name: validating-decisioning-e2e-tests
description: >
  Validates Decisioning E2E test scenarios documented in Notion against the staging database.
  Given a Notion page URL that lists test cases with an `application-id`, an expected
  `Offers result`, and an `allyName`, this skill queries the staging `legacy_loan_application`
  table (via a local port-forward on `localhost:15432` using AWS RDS IAM auth), evaluates each
  case against a fixed set of rules (APR bands, guarantee provider/rate, learning population,
  IDV decisions, and rejection statuses), and produces the per-row `Result` values ready to be
  written back into the Notion "Result" column.
  Use when the user pastes a Notion URL like
  `decisioning-e2e-quickpay-*` and asks to validate the tests recorded there, or when
  running a post-deployment E2E validation cycle for decisioning fixtures.
  Do NOT use for editing Decisioning code, tuning the risk configuration, or debugging live
  decisioning workflows in production (this is a passive read-only DB validation).
---

# Validating Decisioning E2E Tests

Validates decisioning E2E scenarios documented in Notion against the staging `legacy_loan_application` JSONB rows. Read-only against staging. Never edits Notion without explicit user confirmation.

## Inputs

- **Notion URL** (required): a page like `https://app.notion.com/p/addico/decisioning-e2e-quickpay-*` containing a table with columns `Case`, `allyName` (or the ally fixed in the header), `Offers result` / `Expected result`, `application-id`, and `Result`.
- **RDS IAM token** (required when the previous one has expired): the URL-form token the user copies from the AWS console for `main-staging-aurora-0-cluster`, DBUser `developers_iam`. The user pastes it verbatim as PGPASSWORD.

If the token is missing or expired (psql prints `PAM authentication failed` or `password expired`), STOP and ask the user for a fresh token before continuing. Do NOT fall back to other credentials.

## Prerequisites

- SSH port-forward or bastion tunnel is UP on `localhost:15432` pointing at `main-staging-aurora-0-cluster.*.rds.amazonaws.com:5432`. Verify with:
  ```bash
  nc -z -v localhost 15432
  ```
- `psql` is available on PATH.
- Notion MCP is configured so `notion-fetch` can read the page.

## Connection parameters

| Key | Value |
|-----|-------|
| Host | `localhost` |
| Port | `15432` |
| Database | `addi_staging` |
| Username | `developers_iam` |
| Password | AWS RDS IAM token (paste verbatim, URL-encoded, no `https://` prefix) |
| sslmode | `require` |

Do not persist the token to disk. Pass it via `PGPASSWORD` in the same shell command and unset when done.

## Validation rules

The `Offers result` cell in Notion is a comma-separated list of tokens. Each token maps to one check:

| Token in Notion | DB check |
|-----------------|----------|
| `STANDARD_BELOW_USURY` | `data->'loan'->>'effectiveAnnualRate'` cast to float `< 0.3` |
| `FGA` | `data->'loan'->'guarantee'->>'provider' = 'FGA'` |
| `FGA 0.00` | `data->'loan'->'guarantee'->>'rate'` is `NULL` or numerically `= 0` |
| `NONE` | `data->'loan'->'guarantee'->>'provider'` is `NULL` or empty |
| `non-LP` | `data->'creditCheck'->>'learningPopulation' = 'false'` |
| `LP` | `data->'creditCheck'->>'learningPopulation' = 'true'` |
| `ZERO` | `data->'loan'->>'effectiveAnnualRate'` is `NULL` or numerically `= 0` |
| `LBL_BAND` | `data->'loan'->>'effectiveAnnualRate'` cast to float `> 0.4` |
| `HARD_IDV`, `SOFT_IDV`, `PHONE_VERIFICATION`, `EMAIL_VERIFICATION`, `HIGH_SCRUTINY`, `VALIDATION_CALL` | Only `allyName` check (see below). No further rules yet — flag remaining checks as "pending". |
| `REJECTED_BY_HARDCUTS`, `REJECTED_BY_POLICY` | `data->'creditCheck'->>'status'` must be `NULL` or `'REJECTED'` (must NOT be `APPROVED`). |

**allyName check (always applied):** the Notion column `allyName` (or the fixed ally in the page header) must appear as a substring in the raw `data::text`. Convenient path: check that `data->'ally'->>'slug'` equals the expected ally. If the search is negative, flag as failed.

**Missing loan/creditCheck for OFFERS tokens:** if a case is expected to produce a loan proposal but the DB row has no `loan` or `creditCheck` node, treat that as fail for each offers token that needs it (except `ZERO`, which explicitly allows null APR).

## Workflow

- [ ] Read the Notion page with `notion-fetch` and extract, for every row: `Case`, `allyName` (from the row or page header), `application-id`, and `Offers result`. Skip rows with `application-id = N/A`.
- [ ] Verify the port-forward is up (`nc -z -v localhost 15432`). If down, stop and ask the user to open it.
- [ ] Ask the user for the fresh IAM token if it has not been provided yet or the previous one is older than 15 min.
- [ ] Run the query below in ONE call for all application_ids and materialize the results as JSON.
- [ ] For each Notion row, apply the rules for its expected tokens, plus the `allyName` check.
- [ ] Build a per-row verdict: `✅` when all checks pass, otherwise `❌` followed by a short list of the specific failures.
- [ ] Present the full table to the user first. Do NOT edit Notion until the user confirms.
- [ ] Once confirmed, write the verdict into the Notion `Result` column via `notion-update-page` (one call per row).

## Query

Adjust the `application_id` list to the ones extracted from the Notion page. Save to a temp file for post-processing:

```bash
export PGPASSWORD='<paste RDS IAM token verbatim>'

PGSSLMODE=require psql "host=localhost port=15432 dbname=addi_staging user=developers_iam sslmode=require" -A -t -c "
SELECT jsonb_agg(row_to_json(t)::jsonb)::text
FROM (
  SELECT
    application_id::text                         AS application_id,
    data->'loan'->>'effectiveAnnualRate'         AS apr,
    data->'loan'->'guarantee'->>'rate'           AS guarantee_rate,
    data->'loan'->'guarantee'->>'provider'       AS guarantee_provider,
    data->'creditCheck'->>'learningPopulation'   AS is_learning_population,
    data->'creditCheck'->>'statusReason'         AS status_reason,
    data->'creditCheck'->>'status'               AS credit_status,
    data->'creditCheck'->'creditPolicy'->>'name' AS credit_policy_name,
    (data ? 'loan')::text                        AS has_loan,
    (data ? 'creditCheck')::text                 AS has_credit_check,
    data->'ally'->>'slug'                        AS ally_slug,
    data->'ally'->>'name'                        AS ally_name,
    (data::text ILIKE '%<EXPECTED_ALLY>%')::text AS data_contains_ally
  FROM originations_v3_sas.legacy_loan_application
  WHERE application_id IN ( '<uuid-1>', '<uuid-2>', ... )
) t;
" > /tmp/decisioning-validation/rows.json

unset PGPASSWORD
```

Replace `<EXPECTED_ALLY>` with the ally from the Notion header (e.g. `addi-quickpay-pap-cupo-test`, `addi-quickpay-cupo-test`). Batch every application_id in one call — the table is small and this is fastest.

## Validator (Python — embed inline)

```python
import json

rows = json.load(open('/tmp/decisioning-validation/rows.json'))
by_id = {r['application_id']: r for r in rows}

EXPECTED_ALLY   = "addi-quickpay-cupo-test"   # <-- fill from Notion header
OFFERS_RULES    = {"STANDARD_BELOW_USURY","FGA","FGA_ZERO","non-LP","NONE","LP","ZERO","LBL_BAND"}
REJECTED_RULES  = {"REJECTED_BY_HARDCUTS","REJECTED_BY_POLICY"}
REJECTED_OK     = {None, "REJECTED"}
IDV_TOKENS      = {"HARD_IDV","SOFT_IDV","PHONE_VERIFICATION","EMAIL_VERIFICATION","HIGH_SCRUTINY","VALIDATION_CALL"}

# Notion emits "FGA 0.00" with a space — normalize before this step.

def check(app_id, tokens):
    row = by_id.get(app_id)
    fails = []
    if row is None:
        return "❌", ["application_id not found in DB"]

    if row.get("data_contains_ally") != "true":
        fails.append(f"allyName '{EXPECTED_ALLY}' NOT in data (ally_slug={row.get('ally_slug')})")

    apr_raw = row.get("apr");  apr = None if apr_raw in (None,"") else float(apr_raw)
    rate_raw= row.get("guarantee_rate"); rate = None if rate_raw in (None,"") else float(rate_raw)
    prov    = row.get("guarantee_provider")
    lp      = row.get("is_learning_population")
    status  = row.get("credit_status")

    for tok in tokens:
        if tok in REJECTED_RULES:
            if status not in REJECTED_OK:
                fails.append(f"{tok}: expected status null or REJECTED, got {status!r}")
            continue
        if tok in IDV_TOKENS:
            continue  # only allyName check applies
        if tok == "STANDARD_BELOW_USURY" and (apr is None or apr >= 0.3):
            fails.append(f"STANDARD_BELOW_USURY: apr expected < 0.3, got {apr_raw}")
        elif tok == "FGA" and (prov or "").upper() != "FGA":
            fails.append(f"FGA: expected guarantee_provider=FGA, got {prov!r}")
        elif tok == "FGA_ZERO" and rate not in (None, 0.0):
            fails.append(f"FGA 0.00: expected guarantee_rate=0/null, got {rate_raw}")
        elif tok == "NONE" and prov not in (None, ""):
            fails.append(f"NONE: expected guarantee_provider null/empty, got {prov!r}")
        elif tok == "non-LP" and lp != "false":
            fails.append(f"non-LP: expected learningPopulation=false, got {lp!r}")
        elif tok == "LP" and lp != "true":
            fails.append(f"LP: expected learningPopulation=true, got {lp!r}")
        elif tok == "ZERO" and not (apr is None or apr == 0):
            fails.append(f"ZERO: expected apr=0/null, got {apr_raw}")
        elif tok == "LBL_BAND" and (apr is None or apr <= 0.4):
            fails.append(f"LBL_BAND: expected apr > 0.4, got {apr_raw}")
    return ("✅" if not fails else "❌"), fails
```

Normalize Notion tokens before calling `check`:

```python
def normalize(tokens_str):
    # "Standard below usury, FGA 0.00, non-LP" -> ["STANDARD_BELOW_USURY","FGA_ZERO","non-LP"]
    aliases = {
        "standard below usury": "STANDARD_BELOW_USURY",
        "zero apr": "ZERO",
        "lbl band": "LBL_BAND",
        "fga 0.00": "FGA_ZERO",
    }
    out = []
    for raw in [t.strip() for t in tokens_str.split(",")]:
        key = raw.lower()
        out.append(aliases.get(key, raw if raw in {"FGA","NONE","LP","non-LP","ZERO","LBL_BAND",
                                                   "STANDARD_BELOW_USURY","HARD_IDV","SOFT_IDV",
                                                   "PHONE_VERIFICATION","EMAIL_VERIFICATION",
                                                   "HIGH_SCRUTINY","VALIDATION_CALL",
                                                   "REJECTED_BY_HARDCUTS","REJECTED_BY_POLICY"}
                              else raw))
    return out
```

## Output format

Present a table of `Case | Application ID | Expected | Status | Verdict` to the user, then a per-row detail block for every failure. Save this to `/tmp/decisioning-validation/report.md` for reference. Only after the user says "ok, actualiza el Notion" (or equivalent), use `notion-update-page` to write the verdict into each row's `Result` cell.

## Constraints

- **Read-only DB access.** Never issue INSERT/UPDATE/DELETE against staging.
- **Never edit the Notion page without explicit user confirmation.** Present the output first.
- **Never persist the RDS IAM token.** Use `PGPASSWORD` inline and unset after the query.
- **Ask, do not guess, when the token is missing or expired.** Print the exact error from psql and request a new token.
- **Skip N/A rows.** If the Notion `application-id` is `N/A`, leave the row's `Result` unchanged.
- **Preserve ✅ / ❌ emoji verdicts.** For failures, list each violated rule so the reader can act.

## Example invocations

- "Valida las pruebas del notion `decisioning-e2e-quickpay-cupo-phase-3`."
- "Ejecuta la validación de decisioning para este notion `<url>`. Este es el token nuevo: `<paste>`."
- "Vuelve a correr las validaciones del Phase 2 y muéstrame el diff con la corrida anterior."
