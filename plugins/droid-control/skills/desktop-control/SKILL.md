---
name: desktop-control
description: Background knowledge for droid-control workflows -- not invoked directly. Desktop-control driver mechanics for native GUI app automation via trycua cua-driver.
user-invocable: false
---

# Desktop Control

One controller operates an exact GUI target, observes each effect, and stops when the user's postcondition is proved.

## Act

| Goal | Command / tool |
|---|---|
| Discover | `cua-driver list_apps`; use `launch_app` when launch is requested, then select the intended window from its response or `list_windows` |
| Observe | `get_window_state` with observed `pid`, `window_id`, and the run's `session`; use `query`, `max_elements`, or `max_depth` to bound large trees |
| Act | `click` / `type_text` with an exact `target` and fresh `element_token`; use `x,y` only from a valid target screenshot |
| Menu / geometry | Prefer `invoke_menu` with an observed menu path, or `set_window_frame`; verify the resulting window state |
| Verify | Fresh `get_window_state` / `get_desktop_state`, or `verify_state` for an expressible exact-window postcondition |
| Finish | Finalize any owned recording, then `end_session`; leave personal apps and shared services running unless closure was requested |

Use the CLI by default or an existing MCP connection. Replace example IDs, tokens, coordinates, and `RUN_ID` with current observations and a unique run label:

```bash
cua-driver get_window_state '{"pid":844,"window_id":10725,"session":"RUN_ID"}'
cua-driver click '{"target":{"kind":"window","pid":844,"window_id":10725},"element_token":"s0000002a:14","session":"RUN_ID"}'
# Observe again and verify the requested effect before another action.
```

## Detect and setup

```bash
command -v cua-driver
cua-driver --version
cua-driver status
cua-driver doctor
cua-driver describe click
```

No separate Cua skill installation is required. Preserve existing executable wrappers and service ownership. If the binary is missing, obtain approval before using the official [macOS/Linux installer](https://cua.ai/driver/install.sh) or [Windows installer](https://cua.ai/driver/install.ps1). Inspect unfamiliar live schemas; a client version does not identify an already-running daemon.

| Host | Required setup |
|---|---|
| macOS | The responsible app/host needs Accessibility and Screen Recording grants; let the user run `cua-driver permissions grant` and approve prompts |
| Windows | The runtime must run in the interactive desktop session, not Session 0; user/host handles installation and security prompts |
| Linux | Run as the graphical user on its display/session bus; native Wayland may require `CUA_DRIVER_RS_ENABLE_WAYLAND=1` in the service environment. Compositor capture/input/video support varies |

## Rules

1. **Never substitute methods.** Cua-only/native-input excludes CDP, DOM, application APIs, and shell/media shortcuts, including for Electron.
2. **Never share desktop control.** Keep interactive observation, input, permission waits, and cleanup in the parent. Reuse one run label and artifact directory; repeat `session` on each supported CLI call. Labels do not isolate focus, app state, or snapshot caches.
3. **Never reuse stale or ambiguous targets.** A new snapshot invalidates old handles. Do not combine `target` with flat targeting fields; observation and semantic-only tools keep their own schemas. Re-resolve cold launches or vanished windows with bounded discovery, not repeated launches.
4. **Never infer pixels from absent evidence.** Read the actual image and its dimensions; account for resized previews/crops. Tree-only capture cannot ground pixel input. `capture_mode` does not repair capture failures.
5. **Never escalate implicitly.** Window background input is the default. Foreground delivery, temporary menu activation, desktop capture/input, service changes, and OS approvals require the appropriate user/host authorization.
6. **Never equate delivery with completion.** Reobserve after uncertain, partial, or interrupted input before retrying. Check the task's postcondition: selection is not playback; an unchanged frame is not proof of a freeze; a closed window is not proof of process exit.

## Failure map

| Symptom | Next action |
|---|---|
| Missing binary, permission, or capability | Stop that route; resolve setup with the user rather than silently installing, restarting, or changing security settings |
| Sparse tree | Inspect `degraded_reason`; retry once for lazy initialization. Use pixels only if a valid image exists |
| Missing image / `surface_identity_unproven` | Use returned semantics if sufficient; otherwise request desktop scope or report the blocker. Do not relabel a crop as verified window capture |
| Permission wait | Let the user approve or deny; after approval reacquire state instead of replaying the timed-out action |
| `background_unavailable` | Reobserve; retry only the necessary action with `delivery_mode:"foreground"` if visible control is authorized |

For an authorized desktop loop, use `get_desktop_state` → input with `target:{"kind":"desktop","display_id":"primary"}` → fresh desktop state. Keyboard input follows visible focus: stop if the user or another controller changes it.

## Recording

Only when requested, use one persistent MCP connection: `get_recording_state({})` → `start_recording({"output_dir":"/absolute/unused/run-dir","record_video":true})` → authorized actions → `stop_recording({})`.

These recording tools have no public `session` parameter. Video is off by default; check `video_active` and `last_error`. The recorder is shared within its runtime, and manual stop is unconditional: coordinate with an existing owner rather than taking over. Finalize before disconnecting; inspect `last_video_path`, decode the video, and check its scope/dimensions/duration. A working PNG does not prove Wayland video support.

## Evidence handoff

Report ordinary task results directly. Load **capture** for recorded or multi-step evidence, **verify** for formal proof/QA, and **compose** only for a produced artifact. Include driver/host, exact target/input route, observed postcondition, raw evidence paths, and any limitation. Preserve partial recordings as incomplete evidence; review private content before sharing.
