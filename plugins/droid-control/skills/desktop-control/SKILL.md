---
name: desktop-control
description: Background knowledge for droid-control workflows -- not invoked directly. Desktop-control driver mechanics for native GUI app automation via trycua cua-driver.
user-invocable: false
---

# Desktop Control

Operate the requested GUI through Cua, with one controller owning discovery, observation, input, verification, and cleanup.

## Act

| Goal | Action | Bundled reference |
|---|---|---|
| Check readiness | Resolve `cua-driver`; inspect `--version`, `status`, `doctor`, and unfamiliar tool schemas | [Runtime](../../references/cua-driver/RUNTIME.md) |
| Install missing software | Explain the dependency and obtain approval; use the official installer | [Installation](../../references/cua-driver/README.md#install-cua-driver) |
| Discover and operate a window | `list_apps` / `launch_app` → select `window_id` → `get_window_state` → action → fresh proof | [Workflow](../../references/cua-driver/WORKFLOW.md) |
| Recover unavailable window capture | Inspect the structured error; broaden to desktop only when authorized | [Linux capture recovery](../../references/cua-driver/LINUX.md#capture-recovery) |
| Use the visible desktop | `get_desktop_state` → action with `target:{kind:"desktop",display_id:"primary"}` → `get_desktop_state` | [Desktop loop](../../references/cua-driver/WORKFLOW.md#desktop-loop) |
| Record when requested | Use one persistent connection; verify recorder ownership, video status, and final artifacts | [Recording](../../references/cua-driver/RECORDING.md) |
| Finish | Stop after the postcondition is proved; end this run, not the shared daemon | [Cleanup](../../references/cua-driver/RUNTIME.md#cleanup-and-evidence) |

## Setup contract

The plugin includes its reference material under `references/cua-driver/`. **No separately installed cua skill is required.** Do not run `cua-driver skills install` or load documentation from a user's home directory as a prerequisite.

The executable is a separate dependency. Preserve existing wrappers, service ownership, and permission settings. Missing software or access is a setup blocker, not permission to silently install, upgrade, restart a shared service, or approve an OS dialog.

The bundled source revision is recorded in [source.json](../../references/cua-driver/source.json). It describes the reference, not the installed daemon. Inspect live schemas for version-dependent parameters; do not infer capability from a platform name.

Read the current host's guide only when its setup or behavior matters:
[macOS](../../references/cua-driver/MACOS.md),
[Windows](../../references/cua-driver/WINDOWS.md),
[Linux](../../references/cua-driver/LINUX.md).

## Run ownership

Use a unique run label and artifact directory. Reuse the orchestrator's `RUN_ID` / `RUN_DIR` when supplied; otherwise create them once. Named CLI calls must repeat the same session label on every tool that accepts it. Tools without a public session parameter need a persistent connection when lifecycle continuity matters.

Keep short interactive tasks in the parent agent. A screenshot worker adds a competing observer and makes permission waits harder to handle. Separate sessions/cursors do not isolate shared desktop focus, keyboard input, application state, or snapshot caches. Delegate independent rendering or analysis, not simultaneous input to one desktop.

## Method and permission boundaries

1. A user-requested cua-only or GUI-only method overrides the default Electron/browser route. Use native Cua input and observation; do not substitute CDP, DOM, app APIs, or shell/media shortcuts.
2. Prefer fresh semantic handles where usable. Missing semantics can justify screenshot-grounded pixels; a missing screenshot cannot. Use the returned coordinate frame, not an unaccounted-for downsampled preview.
3. Background window input and visible desktop control have different effects. Foreground escalation and desktop capture/input require the appropriate authorization; a failed narrow route grants none.
4. Let the user or trusted host handle OS/security approval. After approval or interruption, reacquire current state rather than replaying an uncertain action.
5. The action result reports delivery, not task completion. Confirm the requested postcondition and stop; an unchanged tree, selected track, or written file path alone is not sufficient proof.

## Evidence handoff

For an ordinary task, report the observed result and relevant limitations directly. Load **capture** for a recording or multi-step evidence deliverable, **verify** for a formal proof/QA report, and **compose** only for a produced artifact. Never record video merely because this skill was loaded.

When handing evidence onward, include:

- driver version, OS/compositor, target window or display, and input route;
- raw screenshot/state/action-result paths in the run directory;
- requested postcondition, observed outcome, and any pending permission or capability blocker;
- for video, the finalized path and actual dimensions/duration, plus any capture failure.

Keep raw evidence unmodified. Review captures for private content before sharing them. Closing a window does not prove the process exited; end the app only when requested and preserve unsaved work.

## References

Load on demand; do not reabsorb these into this file:

- [Workflow](../../references/cua-driver/WORKFLOW.md): exact targets, bounded observation, coordinates, action semantics, and proof.
- [Runtime](../../references/cua-driver/RUNTIME.md): transport, sessions, permissions, and cleanup.
- [Browser](../../references/cua-driver/BROWSER.md): optional typed page automation only when the user's method permits it.
- [Recording](../../references/cua-driver/RECORDING.md): ownership, opt-in video, artifacts, and replay limits.
- [Reference maintenance](../../references/README.md): pinned source, license, and offline validation.
