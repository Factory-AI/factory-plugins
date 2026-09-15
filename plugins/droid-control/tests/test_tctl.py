"""tctl's native input boundary.

The default suite runs the real tctl against a recording wtype executable.
TCTL_NATIVE_E2E=1 also checks incoming bytes in real headless Ghostty, without a
CLI, login fixture, clipboard, or extra script(1) wrapper. TCTL_SH selects an
alternate tctl for negative controls.
"""

import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
import uuid
from pathlib import Path

TCTL = Path(os.environ.get("TCTL_SH", Path(__file__).resolve().parents[1] / "bin" / "tctl"))
ASCII = "Reply with exactly: QA ready. Do not call any tools."
UNICODE = "Unicode stays intact: café → 你好 👩🏽‍💻 e\u0301"

RECORD_INPUT = """#!/usr/bin/env python3
import json, os, sys
with open(os.environ["INPUT_CALLS"], "a") as out:
    out.write(json.dumps({
        "argv": sys.argv[1:],
        "display": os.environ.get("WAYLAND_DISPLAY"),
        "runtime": os.environ.get("XDG_RUNTIME_DIR"),
    }) + "\\n")
sys.exit(int(os.environ.get("INPUT_EXIT", "0")))
"""

RAW_PROBE = r"""
import os, sys, termios, tty
from pathlib import Path
saved = termios.tcgetattr(0)
tty.setraw(0)
if sys.argv[2] == "kitty":
    os.write(1, b"\x1b[>1u")
os.write(1, b"PROBE_READY\r\n")
received = bytearray()
try:
    while True:
        received.extend(os.read(0, 4096))
        Path(sys.argv[1]).write_bytes(received)
        os.write(1, b"CAPTURED\r\n")
finally:
    termios.tcsetattr(0, termios.TCSANOW, saved)
"""


class TctlInputTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="tctl-input-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.session = f"native-test-{uuid.uuid4().hex}"
        self.session_dir = Path("/tmp/tctl-sessions") / self.session
        self.session_dir.mkdir(parents=True)
        self.addCleanup(shutil.rmtree, self.session_dir)
        self.calls_path = self.root / "calls.jsonl"
        for name in ("wtype", "tuistory"):
            executable = self.root / name
            executable.write_text(RECORD_INPUT)
            executable.chmod(0o755)
        self.env = {
            **os.environ,
            "PATH": f"{self.root}{os.pathsep}{os.environ['PATH']}",
            "INPUT_CALLS": str(self.calls_path),
            "LC_ALL": "C.UTF-8",
        }
        self.meta()

    def meta(self, backend="true-input", warmed="1"):
        (self.session_dir / "meta").write_text(
            f"BACKEND={backend}\nWARMED_UP={warmed}\n"
            f"RUNTIME_DIR={shlex.quote(str(self.root))}\nWAYLAND_DISPLAY_NAME=wayland-test\n"
        )

    def ctl(self, *args, check=True, **extra_env):
        return subprocess.run(
            ["bash", str(TCTL), "-s", self.session, *args],
            env={**self.env, **extra_env}, capture_output=True, text=True, check=check,
        )

    def calls(self):
        return [json.loads(line) for line in self.calls_path.read_text().splitlines()]

    def test_text_gets_one_literal_character_per_native_keymap(self):
        # Bulk allocation aliases the 14th/15th distinct characters to
        # Backspace/Tab. The real-emulator test below owns the encoded bytes.
        text = f"{ASCII}\n{UNICODE}\t-m -- ' \" \\\n"
        self.ctl("type", text)
        calls = self.calls()
        self.assertEqual([call["argv"] for call in calls], [["--", ch] for ch in text])
        self.assertTrue(all(call["runtime"] == str(self.root) for call in calls))
        self.assertTrue(all(call["display"] == "wayland-test" for call in calls))

    def test_consecutive_calls_and_chords_release_modifiers(self):
        self.ctl("type", ":Q")
        self.ctl("press", "control", "shift", "left")
        self.ctl("type", "éa")
        self.ctl("press", "-")
        self.ctl("press", "enter")
        self.assertEqual([call["argv"] for call in self.calls()], [
            ["--", ":"], ["--", "Q"],
            ["-M", "ctrl", "-M", "shift", "-k", "Left", "-m", "shift", "-m", "ctrl"],
            ["--", "é"], ["--", "a"], ["--", "-"], ["-k", "Return"],
        ])

    def test_warmup_runs_once_before_input(self):
        self.meta(warmed="0")
        self.ctl("type", "Q:")
        self.ctl("type", "é")
        self.assertEqual([call["argv"] for call in self.calls()], [
            ["-M", "shift", "-m", "shift"], ["--", "Q"], ["--", ":"], ["--", "é"],
        ])

    def test_empty_text_does_not_inject_a_key(self):
        self.ctl("type", "")
        self.assertFalse(self.calls_path.exists())

    def test_injector_failure_stops_remaining_text(self):
        result = self.ctl("type", "QA", check=False, INPUT_EXIT="23")
        self.assertEqual(result.returncode, 23)
        self.assertEqual([call["argv"] for call in self.calls()], [["--", "Q"]])

    def test_tuistory_keeps_its_bulk_text_contract(self):
        self.meta(backend="tuistory")
        self.ctl("type", UNICODE)
        self.assertEqual(
            [call["argv"] for call in self.calls()],
            [["-s", self.session, "type", UNICODE]],
        )


@unittest.skipUnless(os.environ.get("TCTL_NATIVE_E2E") == "1", "set TCTL_NATIVE_E2E=1 for real Ghostty")
class TctlGhosttyTest(unittest.TestCase):
    def assert_capture(self, capture, expected):
        # Enter is a positive completion marker, including when corrupted
        # text is shorter than expected.
        deadline = time.monotonic() + 5
        actual = b""
        while time.monotonic() < deadline:
            actual = capture.read_bytes() if capture.exists() else b""
            if actual.endswith(b"\r"):
                break
            time.sleep(0.02)
        self.assertEqual(actual, bytes(expected))

    def test_native_bytes_across_text_calls_and_chords(self):
        # Opt-in means missing prerequisites are failures, not silent skips.
        for tool in ("ghostty", "cage", "wtype", "script", "setsid"):
            self.assertIsNotNone(shutil.which(tool), f"required tool missing: {tool}")
        with tempfile.TemporaryDirectory(prefix="tctl-ghostty-test-") as tmp:
            root = Path(tmp)
            probe = root / "probe.py"
            probe.write_text(RAW_PROBE)
            for mode in ("plain", "kitty"):
                with self.subTest(mode=mode):
                    session = f"native-test-{uuid.uuid4().hex}"
                    capture = root / f"{mode}.bin"

                    def ctl(*args):
                        return subprocess.run(
                            ["bash", str(TCTL), "-s", session, *args],
                            check=True, capture_output=True, text=True, timeout=20,
                        )

                    try:
                        subprocess.run(
                            ["bash", str(TCTL), "launch",
                             shlex.join([sys.executable, str(probe), str(capture), mode]),
                             "-s", session, "--backend", "ghostty", "--cwd", str(root)],
                            check=True, capture_output=True, text=True, timeout=20,
                        )
                        ctl("wait", "PROBE_READY", "--timeout", "15000")
                        ctl("type", ASCII)
                        ctl("press", "enter")
                        expected = bytearray(ASCII.encode() + b"\r")
                        self.assert_capture(capture, expected)
                        steps = [
                            (("type", UNICODE), UNICODE.encode()),
                            (("type", "abcdefghijklm:Q"), b"abcdefghijklm:Q"),
                            (("type", "abcdefghijklmQ:"), b"abcdefghijklmQ:"),
                            (("press", "ctrl", "c"), b"\x03" if mode == "plain" else b"\x1b[99;5u"),
                            (("type", "q"), b"q"),
                            (("press", "shift", "left"), b"\x1b[1;2D"),
                            (("type", "a:Q"), b"a:Q"),
                            (("press", "-"), b"-"),
                            (("type", "\n\t'\"\\"), b"\r\t'\"\\"),
                            (("type", ""), b""),
                            (("press", "enter"), b"\r"),
                        ]
                        for args, data in steps:
                            ctl(*args)
                            expected.extend(data)
                        self.assert_capture(capture, expected)
                    finally:
                        ctl("close")


if __name__ == "__main__":
    unittest.main()
