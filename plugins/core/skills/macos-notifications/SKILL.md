---
name: macos-notifications
version: 1.0.0
description: |
  Diagnose and fix macOS desktop notifications for the Droid CLI, including tmux and iTerm2 setups. Use when the user wants to:
  - Get a banner when Droid finishes a turn or needs approval
  - Fix notifications that never appear or silently stopped working
  - Wire Stop or Notification hooks up to a desktop notifier
  Covers the silent terminal-notifier failure on modern macOS. macOS only.
---

# macOS notifications for the Droid CLI

Droid fires hook events when a turn ends (`Stop`) and when it wants attention
(`Notification`). Pointing those at a notifier gives you a banner when a long
run finishes. This skill is about making that actually deliver, which is harder
than it looks.

## Read this first: exit codes lie here

`terminal-notifier` **returns exit 0 even when macOS discards the banner.** So
does `osascript`. The whole pipeline reports success while delivering nothing.

Consequences for how you work:

- Never conclude notifications work because the hook exited 0.
- You cannot verify delivery programmatically. Notification Center's database
  (`~/Library/Group Containers/group.com.apple.usernoted/db2/db`) is
  TCC-protected and returns `authorization denied`.
- **Ask the human which banners they actually saw.** That is the only reliable
  signal. Send clearly distinguishable test banners and have them confirm.

Skipping this is the single most common way to "fix" notifications and leave the
user exactly as broken as before, but now confident.

## Diagnose in this order

Work outside in. The backend is the usual culprit, but confirm the cheap things
first so you are not debugging a notifier that was never invoked.

### 1. Find which config file is actually in force

This trips people constantly. Per the hooks docs:

> If `hooks.json` is absent, Droid also reads hook declarations from the `hooks`
> key in the matching `settings.json`.

So if `~/.factory/hooks.json` exists, **the entire `hooks` block in
`settings.json` is ignored.** Users often have both, edit the wrong one, and
conclude hooks are broken. It also means stale hooks in `settings.json` are dead
config that would silently reactivate if `hooks.json` were removed.

```bash
ls -la ~/.factory/hooks.json ~/.factory/settings.json
cat ~/.factory/hooks.json
```

Check for project-level `.factory/hooks.json` too, and use `/hooks` in the CLI
to inspect the merged live config.

### 2. Confirm the hook script runs

```bash
hook_event_name=Stop message="probe" ~/.factory/hooks/droid-notify
echo "exit=$?"
```

Also confirm it is executable (`ls -la`). A non-executable or wrong-path script
shows up in the TUI as a hook error.

### 3. Confirm the backend delivers

This is where it usually dies. See the next section.

### 4. Rule out Focus / Do Not Disturb

```bash
ls ~/Library/DoNotDisturb/DB/Assertions.json 2>/dev/null \
  && echo "a Focus mode is ACTIVE" || echo "no active Focus assertion"
```

`ModeConfigurations.json` is TCC-protected and will fail to read; that is normal
and not the problem.

## Why terminal-notifier silently fails

Most guides on the internet recommend `terminal-notifier`. On modern macOS it is
usually the bug, not the fix. Two independent reasons, both checkable:

```bash
# 1. It links the legacy notification API that macOS no longer delivers.
nm -u "$(brew --prefix)"/Cellar/terminal-notifier/*/terminal-notifier.app/Contents/MacOS/terminal-notifier \
  | grep -i usernotification
# _OBJC_CLASS_$_NSUserNotification  <- deprecated in macOS 11, dead by macOS 26

# 2. Its bundle cannot hold notification authorization.
codesign -dv --verbose=2 "$(brew --prefix)"/Cellar/terminal-notifier/*/terminal-notifier.app
# Signature=adhoc ... Info.plist=not bound ... TeamIdentifier=not set
```

terminal-notifier 2.0.0 is the final release, shipped 2017, and is unmaintained.
There is no newer version to upgrade to. Do not try to fix it, replace it.

A corroborating symptom: `~/Library/Preferences/com.apple.ncprefs.plist` is
absent and terminal-notifier appears nowhere in notification preferences,
because it never successfully registered with Notification Center.

## The fix: route through osascript

`osascript` is Apple-signed and already authorized, so `display notification`
gets delivered. Tradeoffs versus terminal-notifier, both acceptable for this use
case:

- No click-to-focus. terminal-notifier's `-execute` has no osascript
  equivalent. Put the location in the subtitle as text instead.
- No grouping/coalescing. There is no `-group` equivalent.

### Reference implementation

Known-good, verified end to end. Install at `~/.factory/hooks/droid-notify`,
`chmod +x`, and register it for both `Stop` and `Notification`.

```bash
#!/usr/bin/env bash
# macOS Notification Center banner for Droid CLI hook events.
#
# Droid's HookService exports every scalar hook-input field as an env var, so
# $hook_event_name / $notification_type / $message / $cwd are read straight
# from the environment.
#
# Delivery goes through osascript, not terminal-notifier: terminal-notifier
# links against NSUserNotification, which modern macOS no longer delivers. It
# still exits 0 when the banner is dropped, so that failure is silent.
#
# Two hard requirements:
#   1. never write to stdout - Droid feeds hook stdout back to the agent as
#      additional context, so any chatter here pollutes the conversation.
#   2. always exit 0 - a non-zero hook shows an error row in the TUI.

set -u

event="${hook_event_name:-Stop}"
notif_type="${notification_type:-}"
msg="${message:-}"

case "$event" in
  Notification)
    case "$notif_type" in
      permission_prompt) title="Droid needs approval" ;;
      idle_prompt) title="Droid is waiting" ;;
      elicitation_dialog) title="Droid needs input" ;;
      auth_success) title="Droid signed in" ;;
      *) title="Droid" ;;
    esac
    body="${msg:-Waiting for you}"
    ;;
  SessionEnd)
    title="Droid session ended"
    body="${msg:-Session closed}"
    ;;
  SubagentStop)
    title="Droid subagent finished"
    body="${msg:-Subagent complete}"
    ;;
  *)
    title="Droid finished"
    body="${msg:-Turn complete}"
    ;;
esac

# Window name beats tmux's #S:#I target: the index shifts whenever windows are
# closed or reordered.
subtitle=""
if [ -n "${TMUX:-}" ] && [ -n "${TMUX_PANE:-}" ]; then
  subtitle="$(tmux display-message -p -t "$TMUX_PANE" '#W' 2>/dev/null || true)"
fi

# Outside tmux there is no window name, so name the directory instead of
# shipping a blank subtitle.
[ -n "$subtitle" ] || subtitle="$(basename "${cwd:-$PWD}")"

[ "${#body}" -gt 400 ] && body="${body:0:397}..."

# AppleScript string literals accept neither raw newlines nor bare quotes.
# Backslashes have to be doubled before quotes are escaped, or the escapes we
# add here get mangled in turn.
as_quote() {
  local s=$1
  s=${s//$'\r'/ }
  s=${s//$'\n'/ }
  s=${s//\\/\\\\}
  s=${s//\"/\\\"}
  printf '%s' "$s"
}

osascript \
  -e "display notification \"$(as_quote "$body")\" with title \"$(as_quote "$title")\" subtitle \"$(as_quote "$subtitle")\"" \
  >/dev/null 2>&1

exit 0
```

Matching `~/.factory/hooks.json`:

```json
{
  "Notification": [
    { "hooks": [{ "type": "command", "command": "/Users/YOU/.factory/hooks/droid-notify", "timeout": 5 }] }
  ],
  "Stop": [
    { "hooks": [{ "type": "command", "command": "/Users/YOU/.factory/hooks/droid-notify", "timeout": 5 }] }
  ]
}
```

Use an absolute path. Hooks run from Droid's working directory, which is not
necessarily the repo root.

## Hook contract, do not violate these

- **Never write to stdout.** Droid feeds hook stdout back to the agent as extra
  context, so any output pollutes the conversation. Redirect the notifier with
  `>/dev/null 2>&1` and verify with `out=$(...); echo ${#out}` expecting `0`.
- **Always exit 0.** A non-zero hook renders an error row in the TUI.
- Keep `timeout` small (5s). A hung notifier should not stall the turn.

## AppleScript escaping

`display notification` takes AppleScript string literals, which accept neither
raw newlines nor bare double quotes. Escape order matters: **double backslashes
before escaping quotes**, otherwise you mangle the escapes you just added.

Worth testing without sending a banner:

```bash
osascript -e "set x to \"$escaped\""
```

Verify against embedded quotes, backslashes, trailing backslashes, newlines, and
text containing `$(...)` or backticks. Hook `message` values are untrusted input;
they must be passed through literally, never evaluated.

## tmux specifics

- `TMUX` and `TMUX_PANE` are inherited by the hook, so the pane is identifiable.
- Prefer `#W` (window name) over `#S:#I`. The window *index* shifts when windows
  are closed or reordered, and the session is often just named `0` (tmux's
  default when started without `-s`), carrying no information.
- `automatic-rename` is commonly `on` globally but flips `off` per-window as soon
  as a window is explicitly named. So `#W` is stable for named windows, but an
  unnamed new window's `#W` drifts to the running command. Check with:
  `tmux list-windows -F '#{window_index} #{window_name} auto_rename=#{?automatic-rename,ON,off}'`
- `#S:#I` is valid `-t` target syntax, so it is worth mentioning to the user as a
  manual way to jump back (`tmux select-window -t 0:5`) now that click-to-focus
  is gone.

## Sounds are separate

Droid plays its own audio via `general.completionSound` /
`general.awaitingInputSound` in `settings.json`, with `soundFocusMode`
controlling when. Do not add a `sound name` to the notification as well, or the
user gets doubled audio.

## Verified environment

Findings above were confirmed on macOS 26.5.2 (build 25F84, arm64),
terminal-notifier 2.0.0, tmux with iTerm2, bash 3.2.57 (so the script avoids
bash 4+ syntax). The terminal-notifier diagnosis applies to macOS versions that
dropped `NSUserNotification` delivery; on older macOS it may still work. Re-run
the `nm` and `codesign` checks rather than assuming.
