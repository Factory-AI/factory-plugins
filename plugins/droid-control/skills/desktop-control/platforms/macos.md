# Desktop-Control: macOS

cua-driver on macOS posts events per-pid through Accessibility (AX) and captures via ScreenCaptureKit. Both are gated by TCC, and TCC attributes grants to the **app bundle that asks** -- which is why every flow below routes through `CuaDriver.app` instead of your terminal.

## Permissions (TCC)

```bash
cua-driver permissions grant    # LaunchServices-routed: the Accessibility + Screen Recording dialogs
                                # attribute to com.trycua.driver, then it confirms the driver's own status
cua-driver permissions status   # read-only via the daemon; reports `unknown` when no daemon is up
```

Do not grant by clicking through System Settings for your terminal app -- the daemon runs under the bundle identity, and terminal-attributed grants do nothing for it. The first real screen capture may trigger one extra consent sheet; accept it.

## Daemon launch

Launch from the logged-in GUI session so the daemon attaches to it with the bundle's TCC identity:

```bash
open -n -g -a CuaDriver --args serve
cua-driver status
cua-driver stop
```

SSH-launched bare binaries often miss the GUI session and their AX/capture probes hang. (`cua-driver mcp` and CLI tool calls auto-proxy to a properly attributed daemon when one is reachable.)

## Patterns

**Reliable terminal command entry** -- when `type_text` or raw key posting drops characters in Terminal-class apps, route through the pasteboard:

```bash
printf '%s' 'your command' | pbcopy
cua-driver hotkey    '{"pid":<term-pid>,"window_id":<wid>,"keys":["cmd","v"]}'
cua-driver press_key '{"pid":<term-pid>,"window_id":<wid>,"key":"return"}'
```

**Native security / modal sheets** (SecurityAgent, Keychain prompts, auth dialogs) -- these often report `is_on_screen: false` even while visible. Locate by process, then enumerate everything:

```bash
pgrep -fl SecurityAgent
cua-driver list_windows '{"pid":<sa-pid>,"on_screen_only":false}'
cua-driver get_window_state '{"pid":<sa-pid>,"window_id":<wid>}'
```

Only enter credentials in environments you own and were explicitly authorized to drive.

**Menu commands / app shortcuts** -- pass `window_id` so AppKit routes the key equivalent to the target app instead of the frontmost one:

```bash
cua-driver hotkey '{"pid":835,"window_id":79,"keys":["cmd","q"]}'
```

**Backgrounded / off-space windows** -- the driver acts on `(pid, window_id)` without raising. Enumerate with `on_screen_only: false` and target directly.

## Failure modes

| Symptom | Fix |
|---|---|
| AX write fails (`AXPress` returns `-25204`) on a system sheet | Fall back to `press_key` / `hotkey` / pixel `click` |
| ScreenCaptureKit error (e.g. SCK `-3801`) in `som`/`vision` capture | `cua-driver config set capture_mode ax` (tree-only, skips Screen Recording), or retry |
| Known dialog missing from `list_windows` results | Re-query with `"on_screen_only": false` |
| Probes hang / permissions report `unknown` | Daemon was launched without GUI attribution -- `cua-driver stop`, relaunch via `open -n -g -a CuaDriver --args serve` |

Deep mechanics (no-foreground forbidden-list, AXMenuBar navigation, SkyLight click dispatch, Apple-Events browser bridge) live in the upstream pack: `~/.cua-driver/skills/cua-driver/MACOS.md`.
