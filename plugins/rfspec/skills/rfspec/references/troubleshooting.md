# Troubleshooting

## All three models failed

```
Error: All three models failed. Check that your droid CLI is authenticated...
```

**Cause:** droid CLI not authenticated or models unavailable.
**Solution:** Run `droid` interactively to verify auth, then retry.

## jq not installed

```
Error: jq is required but not installed.
```

**Cause:** jq not on PATH.
**Solution:** `brew install jq` (macOS) or `apt-get install jq` (Linux).

## One or two models failed

```
Note: The following models encountered errors: Opus 4.6
```

**Cause:** Specific model unavailable or rate-limited.
**Solution:** This is handled gracefully -- compare the options that did return. No action needed unless the failed model was critical.

## Command not found

```
/rfspec: command not found
```

**Cause:** Plugin not installed.
**Solution:** Install via `/plugins` UI or `droid plugin install rfspec@factory-plugins --scope user`.
