# Desktop-Control: Windows

cua-driver on Windows walks UI Automation (UIA) trees and dispatches actions through a layered UIA + `PostMessage` chain -- per-window message posting, not HID synthesis, so the user's foreground app is untouched.

## Install and daemon

The upstream installer is per-user (no admin elevation): binary under `%LOCALAPPDATA%\Programs\Cua\cua-driver\bin`, data and skill pack under `%USERPROFILE%\.cua-driver`, and an autostart task (`cua-driver autostart status|kick|disable`) registered for the daemon.

```powershell
cua-driver doctor
cua-driver serve     # required for element_index workflows
cua-driver status
cua-driver stop
```

`cua-driver permissions` is a no-op surface on Windows (TCC is a macOS concept) -- there is no grant dance. The real constraint is **Session 0 isolation**: anything launched by a service (including some SSH daemons) lives in a session with no interactive desktop, where window enumeration returns nothing. Tool calls auto-proxy to an interactive-session daemon when one is reachable; if results come back empty, confirm the daemon was started from the logged-in interactive session, not a service context.

## JSON quoting (the PowerShell 5.1 footgun)

Windows PowerShell 5.1 strips quotes around JSON field names in multi-field arguments, so positional JSON fails to parse. Pipe via stdin, or use PowerShell 7+ (`pwsh`):

```powershell
'{"pid":1234,"window_id":5678}' | cua-driver get_window_state
```

From `cmd.exe`, escape inner quotes instead: `cua-driver get_window_state "{\"pid\":1234,\"window_id\":5678}"`.

## Patterns

**UWP / packaged apps** -- Store apps (Calculator, Settings) are hosted by `ApplicationFrameHost.exe`, so the visible window's pid is the host's, not the app process's. If `list_windows` against the app's own pid comes up empty, enumerate `ApplicationFrameHost.exe`'s windows and match by title. Classic Win32 apps (Notepad, Explorer) own their windows directly.

**Minimized windows** -- `get_window_state` and element-index actions work in place, but `press_key` commits silently no-op (no message pump focus). Use `set_value` or element-index-click the commit-equivalent button instead.

**Browsers / Electron** -- prefer **agent-browser**. If you must stay in desktop-control, launch the browser with `--remote-debugging-port=<port>` and export `CUA_DRIVER_CDP_PORT=<port>` so `execute_javascript` / `query_dom` can attach; UIA covers `get_text` either way.

## Failure modes

| Symptom | Fix |
|---|---|
| `UIA invoke failed` on an element | Try `click` with an explicit `action` (`show_menu`, `confirm`, ...) or fall through to a pixel click on the element's center |
| Empty window lists, blank screenshots | Session 0 daemon -- restart `cua-driver serve` from the interactive desktop session |
| Positional JSON "did not parse" errors | PowerShell 5.1 quote-stripping -- pipe JSON via stdin or use `pwsh` |
| Target window not under the app's pid | UWP hosting -- enumerate `ApplicationFrameHost.exe` windows |

Deep mechanics (UIA tree semantics, click-dispatch layering, focus-steal vectors, UAC boundaries) live in the upstream pack: `~/.cua-driver/skills/cua-driver/WINDOWS.md`.
