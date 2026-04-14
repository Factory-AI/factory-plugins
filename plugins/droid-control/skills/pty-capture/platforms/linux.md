# PTY Byte Capture: Linux / Wayland

## Quick start

```bash
# Full matrix across all installed terminals
${DROID_PLUGIN_ROOT}/scripts/capture-terminal-bytes.py --format table

# Single terminal + combo
${DROID_PLUGIN_ROOT}/scripts/capture-terminal-bytes.py --backend ghostty --combo shift-enter --format json
```

Supported backends: `ghostty`, `kitty`, `alacritty`.
Supported combos: `enter`, `shift-enter`, `ctrl-l`, `escape`, `shift-tab`.

## Architecture

```
wtype  ->  cage (headless compositor)  ->  terminal emulator  ->  PTY child  ->  hex output
```

The child process (`pty-hex-dumper.py`) switches stdin to raw mode, prints `READY`, then dumps each byte as space-separated hex pairs.

## Prerequisites

```bash
sudo apt-get install -y cage wtype
ls /dev/dri/    # must be non-empty
```

Plus a Wayland terminal (`ghostty`, `kitty`, or `alacritty`).

## Manual capture (escape hatch)

When the scripts are insufficient, use a zero-buffered PTY child directly:

```bash
TCTL=${DROID_PLUGIN_ROOT}/bin/tctl

$TCTL launch "${DROID_PLUGIN_ROOT}/scripts/pty-hex-dumper.py --ready" \
  -s capture --backend ghostty
$TCTL -s capture wait "READY" --timeout 10000
$TCTL -s capture press shift enter
sleep 0.5
$TCTL -s capture snapshot --trim
$TCTL -s capture close
```

Or use a one-liner Perl child:

```bash
perl -e '$|=1; while(sysread(STDIN,$b,1)){printf "%02x ",ord($b)}'
```
