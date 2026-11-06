#!/usr/bin/env python3

import argparse
import json
import os
import re
import shutil
import stat
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath


REPOSITORY_ROOT = Path(__file__).resolve().parent.parent
PLUGIN_ROOT = REPOSITORY_ROOT / "plugins" / "local-ydb-toolkit"
SOURCE_METADATA = PLUGIN_ROOT / "SOURCE.json"
EXPECTED_REPOSITORY = "https://github.com/astandrik/local-ydb-toolkit.git"
SOURCE_MAPPINGS = (
    (PurePosixPath(".claude-plugin/plugin.json"), PurePosixPath(".factory-plugin/plugin.json")),
    (PurePosixPath("LICENSE"), PurePosixPath("LICENSE")),
    (PurePosixPath("mcp.json"), PurePosixPath("mcp.json")),
    (PurePosixPath("skills/local-ydb"), PurePosixPath("skills/local-ydb")),
)
MANAGED_DIRECTORY_TARGETS = (PurePosixPath("skills/local-ydb"),)
MAX_FILES = 64
MAX_FILE_BYTES = 1_000_000
MAX_TOTAL_BYTES = 5_000_000


class SyncError(RuntimeError):
    pass


@dataclass(frozen=True)
class SourceFile:
    content: bytes
    executable: bool


def run_git(source: Path, *arguments: str) -> bytes:
    result = subprocess.run(
        ["git", "-C", str(source), *arguments],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        message = result.stderr.decode("utf-8", errors="replace").strip()
        raise SyncError(f"git {' '.join(arguments)} failed: {message}")
    return result.stdout


def load_source_commit() -> str:
    try:
        metadata = json.loads(SOURCE_METADATA.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise SyncError(f"Cannot read {SOURCE_METADATA}: {error}") from error

    expected_keys = {"schemaVersion", "repository", "commit"}
    if set(metadata) != expected_keys:
        raise SyncError(f"{SOURCE_METADATA} must contain exactly {sorted(expected_keys)}")
    if metadata["schemaVersion"] != 1:
        raise SyncError("Unsupported source metadata schema")
    if metadata["repository"] != EXPECTED_REPOSITORY:
        raise SyncError(f"Source repository must be {EXPECTED_REPOSITORY}")

    commit = metadata["commit"]
    if not isinstance(commit, str) or re.fullmatch(r"[0-9a-f]{40}", commit) is None:
        raise SyncError("Source commit must be a full lowercase SHA-1")
    return commit


def parse_tree_entry(raw_entry: bytes) -> tuple[str, str, str]:
    try:
        metadata, raw_path = raw_entry.split(b"\t", 1)
        mode, object_type, object_id = metadata.decode("ascii").split(" ")
        source_path = raw_path.decode("utf-8")
    except (ValueError, UnicodeDecodeError) as error:
        raise SyncError("Unexpected git ls-tree output") from error

    if object_type != "blob" or mode not in {"100644", "100755"}:
        raise SyncError(f"Unsupported source entry {source_path}: mode={mode}, type={object_type}")
    return mode, object_id, source_path


def safe_relative_path(raw_path: str) -> PurePosixPath:
    path = PurePosixPath(raw_path)
    if path.is_absolute() or not path.parts or any(part in {"", ".", ".."} for part in path.parts):
        raise SyncError(f"Unsafe relative path: {raw_path}")
    return path


def read_source_files(source: Path, commit: str) -> dict[PurePosixPath, SourceFile]:
    run_git(source, "cat-file", "-e", f"{commit}^{{commit}}")
    generated: dict[PurePosixPath, SourceFile] = {}
    total_bytes = 0

    for source_root, target_root in SOURCE_MAPPINGS:
        tree = run_git(source, "ls-tree", "-r", "-z", commit, "--", source_root.as_posix())
        entries = [entry for entry in tree.split(b"\0") if entry]
        if not entries:
            raise SyncError(f"Missing source path at {commit}: {source_root}")

        source_root_is_file = len(entries) == 1 and parse_tree_entry(entries[0])[2] == source_root.as_posix()
        for entry in entries:
            mode, object_id, raw_source_path = parse_tree_entry(entry)
            source_path = safe_relative_path(raw_source_path)
            if source_root_is_file:
                target_path = target_root
            else:
                try:
                    suffix = source_path.relative_to(source_root)
                except ValueError as error:
                    raise SyncError(f"Source path escaped mapping root: {source_path}") from error
                target_path = target_root / suffix

            target_path = safe_relative_path(target_path.as_posix())
            if target_path in generated:
                raise SyncError(f"Duplicate generated target: {target_path}")

            content = run_git(source, "cat-file", "blob", object_id)
            if len(content) > MAX_FILE_BYTES:
                raise SyncError(f"Source file exceeds {MAX_FILE_BYTES} bytes: {source_path}")
            total_bytes += len(content)
            if total_bytes > MAX_TOTAL_BYTES:
                raise SyncError(f"Generated content exceeds {MAX_TOTAL_BYTES} bytes")
            generated[target_path] = SourceFile(content=content, executable=mode == "100755")

    if len(generated) > MAX_FILES:
        raise SyncError(f"Generated file count exceeds {MAX_FILES}")
    return generated


def destination_for(target: PurePosixPath) -> Path:
    plugins_root = REPOSITORY_ROOT / "plugins"
    if plugins_root.is_symlink() or PLUGIN_ROOT.is_symlink():
        raise SyncError("Plugin destination root must not be a symlink")

    destination = PLUGIN_ROOT.joinpath(*target.parts)
    parent = PLUGIN_ROOT
    for part in target.parts[:-1]:
        parent /= part
        if parent.is_symlink():
            raise SyncError(f"Plugin destination parent must not be a symlink: {parent}")
    return destination


def existing_managed_files() -> set[PurePosixPath]:
    existing: set[PurePosixPath] = set()
    for target in MANAGED_DIRECTORY_TARGETS:
        root = destination_for(target)
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if path.is_file() or path.is_symlink():
                existing.add(PurePosixPath(path.relative_to(PLUGIN_ROOT).as_posix()))
    return existing


def check_generated_files(generated: dict[PurePosixPath, SourceFile]) -> None:
    errors: list[str] = []
    for target, source_file in sorted(generated.items(), key=lambda item: item[0].as_posix()):
        destination = destination_for(target)
        if destination.is_symlink() or not destination.is_file():
            errors.append(f"missing regular file: {target}")
            continue
        if destination.read_bytes() != source_file.content:
            errors.append(f"content differs: {target}")
        executable = bool(destination.stat().st_mode & stat.S_IXUSR)
        if executable != source_file.executable:
            errors.append(f"executable bit differs: {target}")

    expected_managed = {
        target
        for target in generated
        if any(target.is_relative_to(root) for root in MANAGED_DIRECTORY_TARGETS)
    }
    for extra in sorted(existing_managed_files() - expected_managed, key=PurePosixPath.as_posix):
        errors.append(f"unexpected generated file: {extra}")

    if errors:
        raise SyncError("Vendored Local YDB Toolkit is stale:\n  - " + "\n  - ".join(errors))


def remove_managed_target(target: PurePosixPath) -> None:
    destination = destination_for(target)
    if destination.is_symlink() or destination.is_file():
        destination.unlink()
    elif destination.is_dir():
        shutil.rmtree(destination)


def write_generated_file(destination: Path, source_file: SourceFile) -> None:
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    mode = 0o755 if source_file.executable else 0o644

    try:
        descriptor = os.open(destination, flags, mode)
    except FileExistsError as error:
        target = destination.relative_to(PLUGIN_ROOT)
        raise SyncError(f"Destination appeared during synchronization: {target}") from error

    with os.fdopen(descriptor, "wb") as output:
        output.write(source_file.content)
        if hasattr(os, "fchmod"):
            os.fchmod(output.fileno(), mode)


def synchronize(generated: dict[PurePosixPath, SourceFile]) -> None:
    for _, target in SOURCE_MAPPINGS:
        remove_managed_target(target)

    for target, source_file in sorted(generated.items(), key=lambda item: item[0].as_posix()):
        destination = destination_for(target)
        destination.parent.mkdir(parents=True, exist_ok=True)
        write_generated_file(destination, source_file)


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Synchronize the vendored Local YDB Toolkit plugin")
    parser.add_argument("--source", required=True, type=Path, help="Path to a local-ydb-toolkit Git repository")
    parser.add_argument("--check", action="store_true", help="Fail if vendored files differ without writing")
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        commit = load_source_commit()
        generated = read_source_files(arguments.source.resolve(), commit)
        if arguments.check:
            check_generated_files(generated)
            print(f"Verified {len(generated)} vendored files from {commit}")
        else:
            synchronize(generated)
            check_generated_files(generated)
            print(f"Synchronized {len(generated)} vendored files from {commit}")
    except SyncError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
