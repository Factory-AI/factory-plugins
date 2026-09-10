# Bundled Cua reference

`cua-driver/` is an unmodified documentation bundle from
[trycua/cua](https://github.com/trycua/cua), pinned by full commit SHA and
per-file SHA-256 hashes in `cua-driver/source.json`. The MIT license is
included as `cua-driver/LICENSE.md`. The Cua executable is not bundled.

The initial pin is the reviewed contribution in
[Cua PR #3719](https://github.com/trycua/cua/pull/3719), not a claim that the
contribution has merged or that every installed driver has its capabilities.
The skill's version is source metadata. Runtime capability checks remain
necessary, including for users with older releases.

Desktop-control owns plugin routing, setup, and evidence handoff. Cua owns
driver mechanics. Keep the upstream files byte-for-byte intact rather than
maintaining another copy of each platform's rules. They live outside `skills/`;
the vendored `SKILL.md` is documentation, not another registered skill.

## Validate offline

From the plugin directory:

```bash
python3 scripts/cua-reference.py check
python3 -m unittest discover -s tests -p 'test_desktop_control.py'
```

No Cua installation, personal skill directory, GUI access, or network is
needed. Integrity validation detects missing, extra, or modified bundle files;
tests check reference resolution and plugin-only packaging.

## Update the pin

Review a new Cua commit first. With a local checkout containing that commit:

```bash
python3 scripts/cua-reference.py sync --source /path/to/cua --revision FULL_40_CHARACTER_COMMIT_SHA
```

The importer reads Git blobs, never the checkout's uncommitted edits, and does
not fetch or execute upstream code. It refuses a dirty existing bundle rather
than overwriting hand edits. Review the resulting diff, run the checks above,
and validate representative argument examples against the supported driver.
Do not update the pin merely to silence an integrity failure.
