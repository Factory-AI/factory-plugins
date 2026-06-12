# Desktop-Control: Linux

cua-driver on Linux enumerates windows via **X11**, walks semantic trees via **AT-SPI**, and injects input via **XSendEvent** (synthetic events targeted at a window XID -- no focus change, nothing leaks to the user's focused app). Upstream calls this tier pre-release, and it shows: the lifecycle (install, daemon, doctor, sessions, one-shot CLI), window discovery, and per-window screenshots are solid; Wayland-native enumeration, AT-SPI tree quality, and input delivery are not. Plan workflows around the reliable half.

## Install and daemon

Same installer and lifecycle as everywhere else (no sudo, `~/.cua-driver`):

```bash
cua-driver doctor    # trustworthy probes: catches missing DISPLAY, verifies X11 + AT-SPI before you waste a run
cua-driver serve     # required for element_index workflows
cua-driver status
```

`cua-driver permissions` is a no-op surface on Linux.

## The Wayland boundary

Window enumeration is **X11-only**. On a modern Plasma/GNOME Wayland desktop, native-Wayland windows are invisible to `list_windows` -- which is most windows.

- Targets running under **Xwayland** (or a plain X11 session) enumerate and screenshot fine.
- To drive an app that defaults to native Wayland, force its X11 backend at launch where the toolkit allows it: `QT_QPA_PLATFORM=xcb` (Qt), `GDK_BACKEND=x11` (GTK), `--ozone-platform=x11` (Chromium/Electron).
- If the target cannot be put on X11, desktop-control cannot see it -- fall back to **agent-browser** (web/Electron) or **true-input** (terminal emulators).

## Semantic layer (AT-SPI) reliability

AT-SPI trees can collapse: the registry's `GetChildren` may time out, and Qt apps can render as a single root node even with `QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1`. When `get_window_state` returns a near-empty tree:

```bash
cua-driver config set capture_mode vision   # screenshot-only snapshots
```

and work the pixel path (`click '{"pid":N,"window_id":W,"x":X,"y":Y}'`) against the returned PNG. Don't burn turns re-snapshotting hoping the tree fills in -- on this tier, pixel-first is a legitimate default.

## The toolkit boundary: synthetic input is silently dropped by Qt and GTK4

XSendEvent marks events with the `send_event` flag, and major toolkits **ignore flagged input entirely**. Verified on v0.5.1: Qt apps (kcalc) and GTK4 apps (zenity) no-op on *every* action -- pixel clicks, `press_key`, `type_text` -- while the driver reports success. There is no error to catch; only the post-action snapshot reveals it.

Practical consequence: the Act stage only works against apps that honor synthetic events (verified: winit-based apps like alacritty; generally simpler/older X11 toolkits). **Probe before committing to a workflow**: send one cheap keystroke, re-snapshot, and check it rendered. If the target ignores synthetic input, desktop-control cannot act on it on this tier -- Observe (screenshots, window enumeration) still works, but route the interaction through **agent-browser** (web/Electron) or **true-input** (terminal) instead.

## Text input is lossy even where it lands

In apps that do accept synthetic input, typing drops and mangles characters: shifted symbols can inject as their unshifted key (`*` arriving as `8`), trailing characters get dropped (verified: `type_text "echo ok42"` rendered `echo ok4`), and `type_text_chars` with generous per-char delays still loses keystrokes. `hotkey` chords (including paste shortcuts) and middle-click paste do **not** land reliably, so the clipboard is not a workaround here.

What works: short bursts plus verification. After every `type_text`, re-snapshot, compare the rendered text against what you sent, and repair the diff (`press_key` backspace, retype the missing tail). On Linux the post-action screenshot is not a formality -- it is the only way to know what actually arrived.

## Failure modes

| Symptom | Fix |
|---|---|
| Expected window missing from `list_windows` | Native-Wayland target -- relaunch it on the X11 backend (`QT_QPA_PLATFORM=xcb` / `GDK_BACKEND=x11` / `--ozone-platform=x11`) |
| Tree is a single root node / AT-SPI timeouts | `capture_mode vision` + pixel actions |
| Every action "succeeds" but nothing changes | Toolkit drops `send_event` input (Qt, GTK4) -- target is unreachable on this tier; use agent-browser or true-input for the interaction |
| Typed text arrives mangled or truncated | Verify-and-repair loop: re-snapshot, diff rendered text, backspace + retype the tail |
| `doctor` reports no DISPLAY | Run from the graphical session (or export the session's `DISPLAY`/`XAUTHORITY`), not a bare TTY/SSH context |

Deep mechanics live in the upstream pack: `~/.cua-driver/skills/cua-driver/LINUX.md`.
