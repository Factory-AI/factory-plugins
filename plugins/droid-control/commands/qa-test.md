---
description: Run an automated QA test flow against a terminal CLI or web/Electron app
argument-hint: '"<URL>" or "<app-name>" or "<PR-number> [-- focus area]" or "<description>"'
---

Load skills: **droid-control**.

## Parse Arguments

`$ARGUMENTS` can be:
- **URL** (`https://app.factory.ai`, `localhost:3000`) → web app
- **Electron app name** (`Slack`, `VS Code`, `Figma`) → Electron app via CDP
- **CLI command** (`droid-dev`, `htop`, `my-cli --flag`) → terminal TUI
- **PR reference** (`11386`) with optional `-- focus area` → infer target from the diff
- **Free-text description** ("test the login flow on staging") → infer target and flow

If a PR reference is found, fetch the PR description and diff to determine what to test.

Determine commitments:

- [ ] **Video recording**: YES if "record", "video", or "demo" appears (implies compose stage)
- [ ] **Showcase**: YES if "polished", "showcase" appears (implies video + showcase)

If showcase is committed, resolve the **preset** using the first matching rule:

| User keywords | Preset |
|---|---|
| `factory`, `official`, `branded` | `factory` |
| `factory hero`, `factory landing` | `factory-hero` |
| `hero`, `landing page`, `social`, `marketing` | `hero` |
| `presentation`, `slides`, `deck` | `presentation` |
| `minimal`, `inline`, `docs embed` | `minimal` |
| _(none of the above)_ | `macos` |

## Load Skills

Use the **droid-control** routing tables:

1. **Target route** -- find the row matching your target, load listed driver/target skills
2. **Stage route** -- load **capture** + **verify** always; load **compose** if video recording or showcase was committed
3. **Artifact route** -- if showcase committed, also load **showcase**

## Define Test Steps

If the user provides specific steps, use them. Otherwise, design a reasonable flow based on the target:

**Web/Electron**: open page → wait for load → screenshot → interact with primary UI → verify state changes → screenshot → close.

**Terminal**: launch app → wait for ready → snapshot → exercise primary features → verify output → snapshot → close.

If the flow is ambiguous or success criteria are unclear, ask the user.

## Capture

Follow the **capture** atom. Provide:
- The target to launch
- The test steps as the interaction script
- Evidence capture at every step (snapshots for terminal, screenshots for browser)

If a step fails:
- Record the failure with evidence
- Continue to the next step for maximum coverage
- Unless the failure blocks everything downstream (e.g., login failed)

## Compose (if committed)

Follow the **compose** atom if a video deliverable was committed. Hand it:

### Mechanical
- layout: single
- clips: [paths to recordings]
- title: "QA Test: <target>"
- output: /tmp/qa-<identifier>.mp4

### Creative
What the test flow covers, which steps passed/failed, what the viewer should focus on.

## Verify

Follow the **verify** atom. It checks the deliverable and QA report completeness.

## Report

```
## QA Test Report

**Target:** <URL or app>
**Driver:** <driver>

### Results

| Step | Status | Notes |
|------|--------|-------|
| ... | PASS/FAIL | ... |

### Issues Found

- <description with screenshot/snapshot reference>

### Evidence

<saved to ./qa-results/>
```
