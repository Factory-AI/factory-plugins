#!/usr/bin/env python3
import argparse
import json
import os
import re
import shlex
import shutil
import subprocess
import time
from pathlib import Path


SUPPORTED_BACKENDS = ("ghostty", "kitty", "alacritty")
SUPPORTED_COMBOS = ("enter", "shift-enter", "ctrl-l", "escape", "shift-tab")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Capture exact PTY bytes for common key combos across supported true-input terminals."
    )
    parser.add_argument(
        "--backend",
        action="append",
        default=[],
        help="Terminal backend to test (ghostty, kitty, alacritty, all). Repeatable.",
    )
    parser.add_argument(
        "--combo",
        action="append",
        default=[],
        help="Key combo to test (enter, shift-enter, ctrl-l, escape, shift-tab, all). Repeatable.",
    )
    parser.add_argument(
        "--format",
        choices=("table", "json"),
        default="table",
        help="Output format.",
    )
    parser.add_argument("--cols", default="80", help="Terminal columns for the session.")
    parser.add_argument("--rows", default="24", help="Terminal rows for the session.")
    parser.add_argument(
        "--wait-timeout",
        default="15000",
        help="Timeout in ms for initial READY wait.",
    )
    parser.add_argument(
        "--settle-time",
        type=float,
        default=0.6,
        help="Seconds to wait after sending the key before snapshotting.",
    )
    return parser.parse_args()


def resolve_targets(requested: list[str], supported: tuple[str, ...], label: str) -> list[str]:
    if not requested or "all" in requested:
        return list(supported)

    resolved: list[str] = []
    for item in requested:
        if item not in supported:
            raise SystemExit(f"unsupported {label}: {item}")
        if item not in resolved:
            resolved.append(item)
    return resolved


def escaped_bytes(raw: bytes) -> str:
    return "".join(
        f"\\x{byte:02x}" if byte < 32 or byte >= 127 else chr(byte) for byte in raw
    )


def run(args: list[str], *, capture: bool = False) -> subprocess.CompletedProcess[str]:
    kwargs = {
        "check": True,
        "text": True,
    }
    if capture:
        kwargs["capture_output"] = True
    else:
        kwargs["stdout"] = subprocess.DEVNULL
    return subprocess.run(args, **kwargs)


def send_combo(tctl: str, session: str, combo: str) -> None:
    match combo:
        case "enter":
            run([tctl, "-s", session, "press", "enter"])
        case "shift-enter":
            run([tctl, "-s", session, "press", "shift", "enter"])
        case "ctrl-l":
            run([tctl, "-s", session, "press", "ctrl", "l"])
        case "escape":
            run([tctl, "-s", session, "press", "esc"])
        case "shift-tab":
            run([tctl, "-s", session, "press", "shift", "tab"])
        case _:
            raise SystemExit(f"unsupported combo: {combo}")


def normalize_hex_snapshot(snapshot: str) -> str:
    body = (
        snapshot.strip()
        .replace("\\r\\n", "\n")
        .replace("\\n", "\n")
        .replace("\\r", "\n")
    )
    if "READY" in body:
        body = body.split("READY", 1)[1].strip()
    return " ".join(body.split())


def capture_matrix(skill_dir: Path, backends: list[str], combos: list[str], args: argparse.Namespace) -> list[dict[str, str]]:
    tctl = str(skill_dir / "bin" / "tctl")
    dumper = skill_dir / "scripts" / "pty-hex-dumper.py"
    child_command = f"{shlex.quote(str(dumper))} --ready"

    results: list[dict[str, str]] = []
    for backend in backends:
        installed = shutil.which(backend) is not None
        for combo in combos:
            if not installed:
                results.append(
                    {
                        "backend": backend,
                        "combo": combo,
                        "status": "unavailable",
                        "hex": "",
                        "escaped": "",
                        "note": f"{backend} is not installed",
                    }
                )
                continue

            session = f"capture-{backend}-{combo}-{os.getpid()}".replace("_", "-")
            record = {
                "backend": backend,
                "combo": combo,
                "status": "ok",
                "hex": "",
                "escaped": "",
                "note": "",
            }

            try:
                run(
                    [
                        tctl,
                        "launch",
                        child_command,
                        "-s",
                        session,
                        "--backend",
                        backend,
                        "--cols",
                        args.cols,
                        "--rows",
                        args.rows,
                    ]
                )
                run(
                    [
                        tctl,
                        "-s",
                        session,
                        "wait",
                        "READY",
                        "--timeout",
                        args.wait_timeout,
                    ]
                )
                send_combo(tctl, session, combo)
                time.sleep(args.settle_time)
                snapshot = run(
                    [tctl, "-s", session, "snapshot", "--trim"], capture=True
                ).stdout
                hex_bytes = normalize_hex_snapshot(snapshot)

                if hex_bytes and not re.fullmatch(
                    r"(?:[0-9a-f]{2}(?: [0-9a-f]{2})*)", hex_bytes
                ):
                    record["status"] = "unexpected-output"
                    record["note"] = snapshot.strip()
                else:
                    raw = bytes.fromhex(hex_bytes) if hex_bytes else b""
                    record["hex"] = hex_bytes
                    record["escaped"] = escaped_bytes(raw)
            except subprocess.CalledProcessError as exc:
                record["status"] = "error"
                record["note"] = (exc.stderr or exc.stdout or str(exc)).strip()
            finally:
                subprocess.run(
                    [tctl, "-s", session, "close"],
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    text=True,
                )

            results.append(record)

    return results


def print_table(results: list[dict[str, str]]) -> None:
    headers = ("backend", "combo", "status", "hex", "escaped", "note")
    widths = {header: len(header) for header in headers}
    for row in results:
        for header in headers:
            widths[header] = max(
                widths[header], len(row.get(header, "").replace("\n", "\\n"))
            )

    def fmt(row: dict[str, str] | None = None) -> str:
        values = row or {header: header for header in headers}
        return " | ".join(
            values.get(header, "").replace("\n", "\\n").ljust(widths[header])
            for header in headers
        )

    print(fmt())
    print("-+-".join("-" * widths[header] for header in headers))
    for row in results:
        print(fmt(row))


def main() -> int:
    args = parse_args()
    script_path = Path(__file__).resolve()
    skill_dir = script_path.parent.parent

    backends = resolve_targets(args.backend, SUPPORTED_BACKENDS, "backend")
    combos = resolve_targets(args.combo, SUPPORTED_COMBOS, "combo")
    results = capture_matrix(skill_dir, backends, combos, args)

    if args.format == "json":
        print(json.dumps(results, indent=2))
    else:
        print_table(results)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
