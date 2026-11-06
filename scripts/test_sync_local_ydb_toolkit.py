#!/usr/bin/env python3

import os
import runpy
import stat
import tempfile
import unittest
from pathlib import Path, PurePosixPath


SCRIPT_PATH = Path(__file__).with_name("sync-local-ydb-toolkit.py")


def load_sync_module() -> dict[str, object]:
    return runpy.run_path(str(SCRIPT_PATH))


class SyncLocalYdbToolkitTest(unittest.TestCase):
    def test_existing_managed_files_does_not_follow_directory_symlinks(self) -> None:
        module = load_sync_module()
        with tempfile.TemporaryDirectory(prefix="factory-rglob-check-") as temp_dir:
            repository_root = Path(temp_dir) / "repository"
            plugin_root = repository_root / "plugins" / "local-ydb-toolkit"
            managed_root = plugin_root / "skills" / "local-ydb"
            outside = Path(temp_dir) / "outside"
            managed_root.mkdir(parents=True)
            outside.mkdir()
            (outside / "marker.txt").write_text("outside", encoding="utf-8")
            (managed_root / "external-dir").symlink_to(outside, target_is_directory=True)

            globals_dict = module["existing_managed_files"].__globals__
            globals_dict["REPOSITORY_ROOT"] = repository_root
            globals_dict["PLUGIN_ROOT"] = plugin_root
            globals_dict["MANAGED_DIRECTORY_TARGETS"] = (PurePosixPath("skills/local-ydb"),)

            existing = module["existing_managed_files"]()

            self.assertIn(PurePosixPath("skills/local-ydb/external-dir"), existing)
            self.assertNotIn(PurePosixPath("skills/local-ydb/external-dir/marker.txt"), existing)

    @unittest.skipUnless(hasattr(os, "fchmod"), "descriptor chmod is unavailable")
    def test_synchronize_writes_content_and_executable_mode(self) -> None:
        module = load_sync_module()
        with tempfile.TemporaryDirectory(prefix="factory-exclusive-write-") as temp_dir:
            repository_root = Path(temp_dir) / "repository"
            plugin_root = repository_root / "plugins" / "local-ydb-toolkit"
            plugin_root.mkdir(parents=True)
            target = PurePosixPath("tool.sh")

            globals_dict = module["synchronize"].__globals__
            globals_dict["REPOSITORY_ROOT"] = repository_root
            globals_dict["PLUGIN_ROOT"] = plugin_root
            globals_dict["SOURCE_MAPPINGS"] = ((PurePosixPath("unused"), target),)
            generated = {target: module["SourceFile"](content=b"#!/bin/sh\n", executable=True)}

            previous_umask = os.umask(0o777)
            try:
                module["synchronize"](generated)
            finally:
                os.umask(previous_umask)

            destination = plugin_root / target
            self.assertEqual(destination.read_bytes(), b"#!/bin/sh\n")
            self.assertTrue(destination.stat().st_mode & stat.S_IXUSR)

    def test_synchronize_rejects_reintroduced_leaf_symlink(self) -> None:
        module = load_sync_module()
        with tempfile.TemporaryDirectory(prefix="factory-leaf-race-") as temp_dir:
            repository_root = Path(temp_dir) / "repository"
            plugin_root = repository_root / "plugins" / "local-ydb-toolkit"
            plugin_root.mkdir(parents=True)
            outside = Path(temp_dir) / "outside.txt"
            outside.write_bytes(b"safe")
            target = PurePosixPath("mcp.json")

            globals_dict = module["synchronize"].__globals__
            globals_dict["REPOSITORY_ROOT"] = repository_root
            globals_dict["PLUGIN_ROOT"] = plugin_root
            globals_dict["SOURCE_MAPPINGS"] = ((PurePosixPath("unused"), target),)
            original_destination_for = globals_dict["destination_for"]
            destination_calls = 0

            def racing_destination_for(candidate: PurePosixPath) -> Path:
                nonlocal destination_calls
                destination = original_destination_for(candidate)
                destination_calls += 1
                if destination_calls == 2:
                    destination.symlink_to(outside)
                return destination

            globals_dict["destination_for"] = racing_destination_for
            generated = {target: module["SourceFile"](content=b"overwritten", executable=False)}

            with self.assertRaisesRegex(module["SyncError"], "Destination appeared during synchronization"):
                module["synchronize"](generated)

            self.assertEqual(outside.read_bytes(), b"safe")


if __name__ == "__main__":
    unittest.main()
