#!/usr/bin/env python3
import argparse
import sys
import termios
import tty


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Dump stdin as space-separated hex pairs."
    )
    parser.add_argument(
        "--ready",
        action="store_true",
        help="Print READY before reading stdin so callers can wait for initialization.",
    )
    parser.add_argument(
        "--no-raw",
        action="store_true",
        help="Do not switch stdin into raw mode before reading.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if args.ready:
        print("READY", flush=True)

    fd = sys.stdin.fileno()
    old_attrs = None
    if not args.no_raw and sys.stdin.isatty():
        old_attrs = termios.tcgetattr(fd)
        tty.setraw(fd)

    try:
        while True:
            chunk = sys.stdin.buffer.read(1)
            if not chunk:
                break
            sys.stdout.write(f"{chunk[0]:02x} ")
            sys.stdout.flush()
    finally:
        if old_attrs is not None:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_attrs)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
