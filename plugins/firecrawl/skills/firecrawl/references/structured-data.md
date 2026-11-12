# Collect structured data

For fields from one known page, use `firecrawl_scrape` to read the page and
extract the fields yourself. For records available through a data provider,
use its published capability. For research spanning sources or unknown URLs,
use `firecrawl_agent`.

## Data providers

Search results can include provider suggestions in `data.tools`. These describe
capabilities; they are not executed results. `alexandria_library_search` explores the
provider library as read-only files with shell commands and reads contracts. It
does not discover this host's MCP tools.

Inspect the selected provider and capability with `alexandria_library_search` when
the full contract is not already available: `grep -ril "<term>" /library | head -20`
finds capabilities, `cat /library/<category>/README.md` lists a category's providers,
and `cat /library/<category>/<provider>/<capability>.md` reads one contract, whose
path names the provider and capability. Use its input contract to
build `options`, and execute with `alexandria_library_read` using
`calls: [{provider, capability, options}]`. Preserve returned identifiers;
do not invent provider names, capability names, or required inputs.

`firecrawl_find_tools` answers the catalogue questions the library files do
not: `urls` finds the providers for a website, `query` matches a data need by
meaning, and `includeCosts: true` returns each capability's listed credit price.

Check the capability's stated price and external effects against the user's
request before execution; a capability priced over the user's limit does not
fit. If access is unavailable or no capability fits,
continue with suitable web sources without representing a catalogue entry as
retrieved data.

For Alexandria feedback, set `endpoint: alexandria`, omit `jobId`, and supply
`requestedWebsite` (the website URL the user needed data from and the requested
functionality), `objective` (the underlying goal behind the session, not only
what was needed from this website), `rating`, and a concise `rationale`
grounded in observed results. Rate useful results `good`, incomplete coverage `partial`, and an
unmet need or failed execution `bad`. When no provider covered the site, add
`providerFeedback` with `name` (the website or the provider you needed),
`issue: "missing_provider"`, and `why`, and rate `bad`. Add
`providerFeedback` or `capabilityFeedback` entries (each an array of
objects) for specific gaps or errors; use
discovered names for existing providers and capabilities. See
[feedback](feedback.md) for when to send it and opt-outs.

## Research jobs

Give `firecrawl_agent` a prompt describing the entities, fields, and scope.
Include a JSON `schema` when the output structure matters, seed `urls` when
known, and `maxCredits` when the user has supplied a spending limit.

The initial call returns a job ID. Retrieve that job with
`firecrawl_agent_status` until it completes or fails, using a bounded wait
rather than rapid repeated requests. Do not submit duplicate jobs while waiting.
If the wait must end before completion, report the pending job ID.

Inspect completed data for missing fields and source support. A failed job can
still return useful `partial` records; report them labeled as incomplete. A returned
`threadId` can continue the same research with a follow-up prompt when requested.
