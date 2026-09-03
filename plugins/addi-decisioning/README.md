# addi-decisioning

Addi-specific skills for validating the Decisioning platform.

## Skills

### `validating-decisioning-e2e-tests`

Given a Notion page URL that lists decisioning E2E test scenarios (rows with
`application-id`, an expected `Offers result` and an `allyName`), this skill:

1. Extracts every test row from the Notion page via the Notion MCP.
2. Queries `originations_v3_sas.legacy_loan_application` on the **staging**
   Aurora cluster through a local port-forward (`localhost:15432`) using AWS
   RDS IAM auth.
3. Evaluates each case against a fixed rulebook (APR bands, guarantee
   provider/rate, learning population, IDV tokens, rejection statuses,
   `allyName` presence).
4. Prints a per-row verdict table and, on user confirmation, writes the
   verdicts back into the Notion `Result` column.

Read-only against the DB. Never edits the Notion page without explicit
confirmation. Never persists the RDS IAM token.

## Prerequisites

- Access to the Addi VPN / bastion tunnel forwarding
  `main-staging-aurora-0-cluster:5432` to `localhost:15432`.
- A fresh RDS IAM token for the `developers_iam` user.
- The Notion MCP server configured in the Droid so `notion-fetch` and
  `notion-update-page` are available.
- `psql` and `python3` on PATH.

## Install

```bash
droid plugin marketplace add https://github.com/Factory-AI/factory-plugins
droid plugin install addi-decisioning@factory-plugins
```

## Invoke

```
/validating-decisioning-e2e-tests
```

Or just paste the Notion URL and ask, e.g.:

> Valida las pruebas del notion `https://app.notion.com/p/addico/decisioning-e2e-quickpay-cupo-phase-3`
