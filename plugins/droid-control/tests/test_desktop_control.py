"""Offline integration checks for a plugin-only desktop-control installation."""

import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

PLUGIN = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "cua_reference", PLUGIN / "scripts" / "cua-reference.py"
)
REFERENCE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REFERENCE)


class DesktopControlTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bundle = self.root / "bundle"
        shutil.copytree(REFERENCE.BUNDLE, self.bundle)

    def test_exact_bundle_passes_offline(self):
        REFERENCE.check(self.bundle)
        manifest = json.loads((self.bundle / "source.json").read_text())
        self.assertRegex(manifest["revision"], r"^[0-9a-f]{40}$")
        for name in ("LINUX.md", "MACOS.md", "WINDOWS.md", "LICENSE.md"):
            self.assertIn(name, manifest["files"])

    def test_modified_missing_and_extra_files_fail(self):
        path = self.bundle / "WORKFLOW.md"
        original = path.read_bytes()
        path.write_bytes(original + b"\nlocal edit\n")
        with self.assertRaisesRegex(ValueError, "drift: WORKFLOW.md"):
            REFERENCE.check(self.bundle)
        path.unlink()
        with self.assertRaisesRegex(ValueError, "file set"):
            REFERENCE.check(self.bundle)
        path.write_bytes(original)
        extra = self.bundle / "unexpected.md"
        extra.write_text("unreviewed instructions")
        with self.assertRaisesRegex(ValueError, "file set"):
            REFERENCE.check(self.bundle)
        extra.unlink()
        REFERENCE.check(self.bundle)

    def test_sync_rejects_a_moving_revision_before_accessing_source(self):
        with self.assertRaisesRegex(ValueError, "full commit SHA"):
            REFERENCE.sync(self.root / "no-checkout", "main", self.bundle)
        REFERENCE.check(self.bundle)

    def test_plugin_only_copy_needs_no_home_skill_or_driver(self):
        installed = self.root / "installed-plugin"
        shutil.copytree(
            PLUGIN,
            installed,
            ignore=shutil.ignore_patterns("node_modules", "__pycache__"),
        )
        home = self.root / "empty-home"
        home.mkdir()
        result = subprocess.run(
            [sys.executable, str(installed / "scripts/cua-reference.py"), "check"],
            cwd=self.root,
            env={**os.environ, "HOME": str(home), "PATH": ""},
            text=True,
            capture_output=True,
            check=True,
        )
        self.assertIn("Cua reference bundle verified.", result.stdout)
        self.assertEqual(list(home.iterdir()), [])

        documents = [
            installed / "skills/desktop-control/SKILL.md",
            installed / "references/README.md",
            *sorted((installed / "references/cua-driver").glob("*.md")),
        ]
        for document in documents:
            for href in re.findall(r"\[[^\]]*\]\(([^)]+)\)", document.read_text()):
                if "://" in href or href.startswith("mailto:"):
                    continue
                name, _, anchor = href.partition("#")
                target = (document.parent / name).resolve() if name else document
                self.assertTrue(target.is_relative_to(installed), (document, href))
                self.assertTrue(target.is_file(), (document, href))
                if anchor:
                    headings = re.findall(r"^#+ (.+)$", target.read_text(), re.M)
                    slugs = {
                        re.sub(r"[^\w\s-]", "", title.lower()).replace(" ", "-")
                        for title in headings
                    }
                    self.assertIn(anchor, slugs, (document, href))

    def test_desktop_route_is_self_contained_and_single_controller(self):
        desktop = (PLUGIN / "skills/desktop-control/SKILL.md").read_text()
        routing = (PLUGIN / "skills/droid-control/SKILL.md").read_text()
        self.assertIn("No separately installed cua skill is required", desktop)
        self.assertIn("Keep short interactive tasks in the parent agent", desktop)
        self.assertIn("method constraints override the defaults below", routing)
        self.assertNotIn("~/.cua-driver/skills", desktop)
        self.assertEqual(
            list((PLUGIN / "skills/desktop-control/platforms").glob("*.md")), []
        )


if __name__ == "__main__":
    unittest.main()
