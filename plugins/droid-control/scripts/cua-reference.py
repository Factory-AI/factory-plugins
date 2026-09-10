#!/usr/bin/env python3
"""Import reviewed Cua documentation or verify the offline bundle; never run Cua."""

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path

REPOSITORY = "https://github.com/trycua/cua"
SOURCE_PATH = "libs/cua-driver/rust/Skills/cua-driver"
BUNDLE = Path(__file__).resolve().parents[1] / "references" / "cua-driver"


def check(bundle: Path = BUNDLE) -> None:
    manifest = json.loads((bundle / "source.json").read_text())
    if manifest["repository"] != REPOSITORY or manifest["source_path"] != SOURCE_PATH:
        raise ValueError("unexpected Cua reference source")
    if not re.fullmatch(r"[0-9a-f]{40}", manifest["revision"]):
        raise ValueError("Cua reference requires a full commit SHA")
    files = manifest["files"]
    required = {"SKILL.md", "WORKFLOW.md", "RUNTIME.md", "README.md", "LICENSE.md"}
    if not required <= files.keys():
        raise ValueError("incomplete Cua reference manifest")
    if {p.name for p in bundle.iterdir()} != set(files) | {"source.json"}:
        raise ValueError("Cua reference file set differs from its manifest")
    for name, expected in files.items():
        path = bundle / name
        if Path(name).name != name or path.is_symlink() or not path.is_file():
            raise ValueError(f"invalid bundled file: {name}")
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            raise ValueError(f"Cua reference drift: {name}")


def sync(source: Path, revision: str, bundle: Path = BUNDLE) -> None:
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError(
            "supply a reviewed full commit SHA, not a moving branch or tag"
        )

    def git(*args: str) -> bytes:
        return subprocess.check_output(["git", "-C", str(source), *args])

    names = (
        git("ls-tree", "--name-only", f"{revision}:{SOURCE_PATH}").decode().splitlines()
    )
    payload = {
        name: git("show", f"{revision}:{SOURCE_PATH}/{name}")
        for name in names
        if name.endswith(".md") and Path(name).name == name
    }
    payload["LICENSE.md"] = git("show", f"{revision}:LICENSE.md")
    if not {"SKILL.md", "WORKFLOW.md", "RUNTIME.md", "README.md"} <= payload.keys():
        raise ValueError("source revision lacks the required reference files")
    if bundle.exists():
        check(bundle)
    bundle.mkdir(parents=True, exist_ok=True)
    for name, contents in payload.items():
        (bundle / name).write_bytes(contents)
    for path in bundle.iterdir():
        if path.name not in payload and path.name != "source.json":
            path.unlink()
    manifest = {
        "repository": REPOSITORY,
        "revision": revision,
        "source_path": SOURCE_PATH,
        "files": {
            name: hashlib.sha256(contents).hexdigest()
            for name, contents in sorted(payload.items())
        },
    }
    (bundle / "source.json").write_text(json.dumps(manifest, indent=2) + "\n")
    check(bundle)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("check", help="verify bundled files offline")
    update = commands.add_parser(
        "sync", help="import from a reviewed local Cua checkout"
    )
    update.add_argument("--source", type=Path, required=True)
    update.add_argument("--revision", required=True)
    args = parser.parse_args()
    if args.command == "sync":
        sync(args.source, args.revision)
    else:
        check()
    print("Cua reference bundle verified.")


if __name__ == "__main__":
    main()
