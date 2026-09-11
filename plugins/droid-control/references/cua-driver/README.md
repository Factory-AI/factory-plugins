# Cua Driver setup

The plugin includes its desktop guidance. Install the
[`cua-driver`](https://github.com/trycua/cua/tree/main/libs/cua-driver/rust)
executable separately when needed; no additional skill installation is required.

## Install Cua Driver

Preserve an existing installation and its service owner. If the executable is
missing, obtain approval before running the official installer.

macOS or Linux:

```bash
/bin/bash -c "$(curl -fsSL https://cua.ai/driver/install.sh)"
```

Windows PowerShell:

```powershell
irm https://cua.ai/driver/install.ps1 | iex
```

Then verify the current host:

```bash
cua-driver doctor
```

On macOS, the installed `CuaDriver.app` needs Accessibility and Screen
Recording permission. On Windows, the daemon must run in an interactive user
desktop rather than Session 0. On Linux, the daemon must share the graphical
session and AT-SPI session bus.

Return to [desktop-control](../../skills/desktop-control/SKILL.md) for task
routing. [Runtime preflight](RUNTIME.md#preflight-and-transport) covers executable
and daemon checks; [reference maintenance](../README.md) covers documentation.
