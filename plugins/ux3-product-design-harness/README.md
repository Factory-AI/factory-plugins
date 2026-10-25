# UX3 Product Design Harness

One skill for reviewing product work before implementation and validating that
the implementation handoff preserves the approved execution boundary.

Use it when a product direction, feature, workflow, experiment, launch, or
feedback iteration needs a User, Evidence, and Business review. The existing
workflow produces continue, verify, or stop_reframe outcomes. Its joint
validator checks that context-pack scope exactly matches the review's approved
may_do list; a continue verdict does not expand execution authority.

## Contents and source

`skills/product-design-harness/` contains the complete upstream 91-file MIT
bundle: SKILL.md, schemas, examples, prompts, knowledge, documentation, Python
validators, both SVG diagrams, and its original LICENSE. All 91 files are
byte-identical to [the fixed upstream skill](https://github.com/cis2042/ux3-product-design-harness/tree/2ea880647e86ba99c65a334864fff8962bb3034e/skills/product-design-harness).

The plugin-level LICENSE is the same upstream MIT license. It applies to this
contribution, without assigning a license to other marketplace content. This
package adds no droids, slash commands, hooks, MCP servers, global context, or
tool permissions. The validator scripts inside the original skill remain
unchanged; nothing invokes them automatically.

## Installation after catalog acceptance

Register the official marketplace and select this plugin:

```text
droid plugin marketplace add https://github.com/Factory-AI/factory-plugins
droid plugin install ux3-product-design-harness@factory-plugins
```

These are the documented Factory commands for the proposed catalog entry.
Until the contribution is accepted, they do not establish that the official
catalog already contains this plugin. Droid installation and model execution
were not tested for this contribution; portable gh skill installation is
separate evidence.

## Validate an implementation handoff

The installed skill's `resources` directory contains the example review,
context pack, schemas, and validator. With an existing Python environment that
provides jsonschema 4.26.0, run from that directory:

```text
python scripts/check_review.py examples/live-coding-review.json --context-pack examples/live-coding-context-pack.json
```

A valid pair returns `ok`. An extra context-pack scope action must be refused.
See `skills/product-design-harness/resources/docs/LIVE-CODING.md` for the
complete workflow, and `skills/product-design-harness/resources/docs/HARNESS.md`
for the bundled diagrams.

The skill is local content with no publisher-hosted service or bundled
credentials. Optional host/provider capabilities depend on the user's existing
environment and authorization. File installation and deterministic validation
do not demonstrate model effectiveness, safety certification, or permission
to take external actions.
