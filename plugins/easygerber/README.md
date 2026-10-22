# EasyGerber for Factory Droid

![EasyGerber](assets/logo.svg)

AI PCB design, layout, and routing.

Design circuit boards with AI, from your first idea to the files your manufacturer needs. Describe what you want to build in everyday language, or start with the pin assignments in your firmware.

Choose from over one million electronic components in EasyGerber’s catalog, including LEDs, sensors, connectors, and microcontrollers. Build with real catalog parts, their footprints, and available 3D models—from a simple LED circuit to a custom electronics project.

Create and edit schematics, generate PCB layouts with automatic component placement and routing, and refine your design through conversation. Adjust board settings, explore copper layers, inspect your board in interactive 3D, and check for blocking layout issues before exporting.

Download Gerber files, drill files, a bill of materials (BOM), component placement files, and an editable EasyGerber file to continue working on your design.

Start with one guest board without signing up through `https://www.easygerber.com/api/mcp`. No account authorization is needed for guest access. When you sign in to keep or continue working on the guest board, the agent saves it to your account after consent. If you connect through client settings for that purpose, resume the task so the agent can finish saving it; the sign-in callback alone does not transfer the board. Signing in solely to delete an account board or submit feedback does not authorize claiming a guest board.

## Install the plugin

This Factory plugin wraps the published EasyGerber 0.3.4 integration at [immutable source revision `fb61fbb68621d15ecb4861fe849346d733cc8f76`](https://github.com/EasyGerber/easygerber-agent-plugin/tree/fb61fbb68621d15ecb4861fe849346d733cc8f76/claude). Its manifest, MCP configuration, skill, workflow reference, artwork, and license retain the published file contents; only the two native configuration paths are renamed for Factory. The npm installer configures Codex only and is not needed here.

Requires the Factory Droid CLI with plugin support and an internet connection to EasyGerber. From a checkout containing this contribution:

```sh
droid plugin marketplace add .
droid plugin install easygerber@factory-plugins --scope user
```

After this contribution is accepted into the official marketplace:

```sh
droid plugin marketplace add https://github.com/Factory-AI/factory-plugins
droid plugin install easygerber@factory-plugins --scope user
```

Restart Droid after installation. Use `/plugins` to inspect the EasyGerber plugin and `/mcp` to inspect its connection or authorize account access when needed. Keep normal tool approvals enabled. Factory model access has its own account and plan requirements.

For a connection check without board changes, ask: “Read the EasyGerber skill and report the available EasyGerber tool names. Do not call a tool, create a guest session or board, authenticate, or send feedback.” Native installation, guest workflows, and account authorization are separate checks; this package does not claim that an untested flow passed.

Try: “Create a small 3.3 V LED board, show a preview, and export the manufacturing files and EasyGerber file.” Use an authorized account if connected; otherwise the service supports one guest board per session. The guest board has no automatic expiry, while generated download links expire after fifteen minutes. The private guest viewer link grants access to the design; keep it private. Signing in to continue working on that guest board includes saving it to your personal account. The agent retains the original guest capability for the claim and continues with the returned account board.

## Connection and data

The plugin connects directly to EasyGerber’s hosted MCP service at `https://www.easygerber.com/api/mcp`. It runs no local server, installer, hooks, or package downloads. Board layout, routing, and scoring run on EasyGerber’s servers. Droid can read relevant firmware locally to identify pin assignments; it sends only the board requirements, published component selections, schematic changes, board or organization identifiers, and feedback messages you approve for the requested work. It should not upload an entire repository or unrelated files.

The remote service supplies catalog results, saved board data, previews, and private export links. Account access uses the service’s advertised OAuth flow and your consent. Do not paste credentials into chat or add tokens to plugin files. Organization permissions, plan limits, and normal request and compute protections apply. The service retains agent activity analytics for one year, including guest history after claim. Removing the plugin does not revoke account authorization or invalidate a private guest link; manage account connections in Droid or EasyGerber.

To delete a specified account board, ask explicitly. The agent reads its current revision and uses `delete_board` only when the service advertises it. Deletion is permanent, has no restore operation, and cancels active jobs. It requires separate `boards:delete` consent through a client that supports requesting that scope, plus personal ownership or an organization owner/admin role; existing `boards:write` access is insufficient. A client registration that excludes this scope must be deliberately updated or replaced through the normal connection flow before consent; existing connections do not gain deletion access automatically. Guest deletion is unsupported, and a guest board must not be claimed merely to delete it. Signing in only to delete an account board does not authorize claiming a guest board. Only `{ boardId, deleted: true }` confirms deletion; `not_found` or a lost response does not.

Creating or exporting a board does not purchase fabrication, make the design public, or delete the source board. A successful layout check is not a safety certification. The MIT license covers this integration package; the hosted service and private engines have separate terms.

## Feedback and support

Ask Droid to send EasyGerber a bug report, feature request, or general feedback. The `submit_feedback` tool sends only a message you ask or agree to share, from 10 to 2000 characters after trimming, to a private administrator inbox. It does not automatically attach transcripts, raw diagnostics, board exports, credentials, or private links. Anonymous feedback needs no account, guest session, or board. Existing valid account or guest access can attribute a report without additional board permissions; rejected credentials do not silently become anonymous. A guest board must never be claimed merely to submit feedback.

The agent reuses the same request ID and identical report when retrying an uncertain submission. A receipt ID and timestamp confirm submission, without promising a response or roadmap commitment. If the connection or service is unavailable, use [GitHub Issues](https://github.com/EasyGerber/easygerber-agent-plugin/issues). Issues are public and require a GitHub account to post; remove credentials, private links, and confidential board data before publishing.

See [documentation and support](https://easygerber.com/agents), [privacy policy](https://easygerber.com/privacy), and [terms](https://easygerber.com/terms).

Submission and support contact: hello@easygerber.com.
