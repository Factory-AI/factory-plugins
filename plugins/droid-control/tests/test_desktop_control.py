"""Static link checks for relocated desktop-control documentation."""

import re
import shutil
import tempfile
import unittest
from pathlib import Path

PLUGIN = Path(__file__).resolve().parents[1]


class DesktopControlTests(unittest.TestCase):
    def test_relocated_documentation_links_stay_inside_plugin(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        installed = Path(temp.name) / "plugin"
        entry = Path("skills/desktop-control/SKILL.md")
        (installed / entry).parent.mkdir(parents=True)
        shutil.copyfile(PLUGIN / entry, installed / entry)
        shutil.copytree(PLUGIN / "references", installed / "references")

        documents = [
            installed / entry,
            installed / "references/README.md",
            *sorted((installed / "references/cua-driver").glob("*.md")),
        ]
        links = [
            (document, href)
            for document in documents
            for href in re.findall(r"\[[^\]]*\]\(([^)]+)\)", document.read_text())
            if "://" not in href and not href.startswith("mailto:")
        ]
        self.assertTrue(links)
        for document, href in links:
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


if __name__ == "__main__":
    unittest.main()
