"""Behavioral tests for scripts/render-showcase.sh.

The helper runs for real (props normalization, staging, cast conversion, probing,
cleanup) against synthetic ffmpeg media. Only the `npx remotion ...` boundary is
replaced by a shim on PATH that records its argv and the staged files it can see.

Run:  python3 -m unittest discover -s plugins/droid-control/tests -p 'test_*.py'
Set RENDER_SHOWCASE_SH to point the suite at another copy of the helper (used as a
negative control against the pre-fix script).

RealRenderTest drives the real `npx remotion render` on a tiny synthetic clip and
asserts on the encoded file. It needs the remotion deps and a browser installed and
runs only with RENDER_SHOWCASE_E2E=1.
"""

import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

PLUGIN_ROOT = Path(__file__).resolve().parents[1]
HELPER = Path(os.environ.get("RENDER_SHOWCASE_SH", PLUGIN_ROOT / "scripts" / "render-showcase.sh"))
REMOTION_PUBLIC = PLUGIN_ROOT / "remotion" / "public"

FAKE_NPX = r"""#!/usr/bin/env bash
# Stands in for `npx remotion ...`: records argv and the staged clips visible at render time.
set -u
printf '%s\n' "$@" > "$FAKE_NPX_ARGS"
find "$PWD/public" -mindepth 2 -type f -exec sha256sum {} + | sort > "$FAKE_NPX_STAGED"
if [[ -n "${FAKE_NPX_READY:-}" ]]; then touch "$FAKE_NPX_READY"; fi
if [[ -n "${FAKE_NPX_BARRIER:-}" ]]; then
  while [[ ! -e "$FAKE_NPX_BARRIER" ]]; do sleep 0.05; done
fi
exit 0
"""

BASE_PROPS = {
    "layout": "single",
    "labels": [],
    "title": "t",
    "subtitle": "s",
    "preset": "macos",
    "keys": [],
    "effects": [],
}


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def make_video(path, seconds):
    path.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            "ffmpeg", "-v", "error", "-y",
            "-f", "lavfi", "-i", "testsrc2=size=320x240:rate=30",
            "-t", str(seconds), "-pix_fmt", "yuv420p", str(path),
        ],
        check=True,
    )


def make_cast(path, events):
    header = {"version": 2, "width": 40, "height": 10}
    lines = [json.dumps(header)] + [json.dumps([t, "o", text]) for t, text in events]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def staging_dirs():
    return sorted(p for p in REMOTION_PUBLIC.glob("render-*") if p.is_dir())


class RenderShowcaseTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        for tool in ("ffmpeg", "ffprobe", "bash"):
            if shutil.which(tool) is None:
                raise unittest.SkipTest(f"{tool} is required for these tests")
        if not HELPER.is_file():
            raise RuntimeError(f"helper not found: {HELPER}")
        cls.root = Path(tempfile.mkdtemp(prefix="render-showcase-test-"))
        cls.before = cls.root / "before" / "recording.mp4"
        cls.after = cls.root / "after" / "recording.mp4"
        make_video(cls.before, 2)
        make_video(cls.after, 5)
        cls.source_hashes = {cls.before: sha256(cls.before), cls.after: sha256(cls.after)}
        cls.bin = cls.root / "bin"
        cls.bin.mkdir()
        shim = cls.bin / "npx"
        shim.write_text(FAKE_NPX, encoding="utf-8")
        shim.chmod(0o755)
        cls.preexisting = staging_dirs()

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.root, ignore_errors=True)

    def setUp(self):
        self.case_dir = Path(tempfile.mkdtemp(prefix="case-", dir=self.root))

    def env(self, **extra):
        env = dict(os.environ)
        env["PATH"] = f"{self.bin}{os.pathsep}{env['PATH']}"
        env["FAKE_NPX_ARGS"] = str(self.case_dir / "npx-args.txt")
        env["FAKE_NPX_STAGED"] = str(self.case_dir / "npx-staged.txt")
        env.update({k: str(v) for k, v in extra.items()})
        return env

    def run_helper(self, props, clips, extra_args=(), env=None):
        props_path = self.case_dir / "props.json"
        props_path.write_text(json.dumps({**BASE_PROPS, **props}), encoding="utf-8")
        cmd = [
            "bash", str(HELPER),
            "--props", str(props_path),
            "--output", str(self.case_dir / "out.mp4"),
            *extra_args,
            *[str(c) for c in clips],
        ]
        return subprocess.run(cmd, env=env or self.env(), capture_output=True, text=True)

    def rendered_props(self, env=None):
        args = Path((env or self.env())["FAKE_NPX_ARGS"]).read_text(encoding="utf-8").splitlines()
        props_arg = next(a for a in args if a.startswith("--props="))
        return args, json.loads(props_arg[len("--props="):])

    def staged_hashes(self, env=None):
        text = Path((env or self.env())["FAKE_NPX_STAGED"]).read_text(encoding="utf-8")
        return {line.split()[1]: line.split()[0] for line in text.splitlines()}

    def assert_sources_intact(self):
        for path, digest in self.source_hashes.items():
            self.assertEqual(sha256(path), digest, f"raw input was modified: {path}")

    def assert_no_new_staging_dirs(self):
        self.assertEqual(staging_dirs(), self.preexisting, "staged directory left behind in remotion/public")

    def test_same_basename_inputs_stay_distinct(self):
        result = self.run_helper(
            {"layout": "side-by-side", "labels": ["BEFORE", "AFTER"]},
            [self.before, self.after],
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        _, props = self.rendered_props()
        self.assertEqual(len(props["clips"]), 2)
        self.assertNotEqual(props["clips"][0], props["clips"][1])
        staged = self.staged_hashes()
        self.assertEqual(len(staged), 2, staged)
        self.assertEqual(
            sorted(staged.values()),
            sorted(self.source_hashes.values()),
            "staged clips must be byte-identical copies of both inputs",
        )
        staged_names = {Path(p).name for p in staged}
        self.assertEqual(staged_names, {Path(c).name for c in props["clips"]})
        self.assert_sources_intact()
        self.assert_no_new_staging_dirs()

    def test_clip_duration_is_longest_source_in_source_seconds(self):
        result = self.run_helper(
            {"layout": "side-by-side", "labels": ["a", "b"], "speed": 2},
            [self.before, self.after],
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        _, props = self.rendered_props()
        self.assertAlmostEqual(props["clipDuration"], 5.0, delta=0.1)
        self.assertEqual(props["speed"], 2)

    def test_speed_must_be_positive_and_finite(self):
        for bad in (0, -1, "2", True, float("nan"), float("inf")):
            with self.subTest(speed=bad):
                self.case_dir = Path(tempfile.mkdtemp(prefix="case-", dir=self.root))
                env = self.env()
                result = self.run_helper({"layout": "single", "speed": bad}, [self.after], env=env)
                self.assertNotEqual(result.returncode, 0, f"speed {bad!r} must be rejected")
                self.assertIn("speed must be a positive finite number", result.stderr)
                self.assertFalse(Path(env["FAKE_NPX_ARGS"]).exists(), "render must not be invoked for a bad speed")
        self.assert_no_new_staging_dirs()

    def test_encode_flags_pin_pixel_format_and_color_space(self):
        for props in ({"layout": "single"}, {"layout": "side-by-side", "labels": ["a", "b"]}, {"layout": "single", "fidelity": "compact"}):
            with self.subTest(props=props):
                self.case_dir = Path(tempfile.mkdtemp(prefix="case-", dir=self.root))
                clips = [self.before, self.after] if props["layout"] == "side-by-side" else [self.after]
                result = self.run_helper(props, clips)
                self.assertEqual(result.returncode, 0, result.stderr)
                args, _ = self.rendered_props()
                self.assertIn("--pixel-format=yuv420p", args)
                self.assertIn("--color-space=bt709", args)

    def test_fidelity_omitted_follows_layout(self):
        result = self.run_helper({"layout": "side-by-side", "labels": ["a", "b"]}, [self.before, self.after])
        self.assertEqual(result.returncode, 0, result.stderr)
        args, props = self.rendered_props()
        self.assertEqual(props["fidelity"], "inspect")
        self.assertEqual((props["width"], props["height"]), (2560, 1440))
        self.assertIn("--crf=14", args)

        self.case_dir = Path(tempfile.mkdtemp(prefix="case-", dir=self.root))
        result = self.run_helper({"layout": "single"}, [self.after])
        self.assertEqual(result.returncode, 0, result.stderr)
        args, props = self.rendered_props()
        self.assertEqual(props["fidelity"], "standard")
        self.assertEqual((props["width"], props["height"]), (1920, 1080))
        self.assertIn("--crf=18", args)

    def test_fidelity_explicit_override(self):
        result = self.run_helper({"layout": "side-by-side", "labels": ["a", "b"], "fidelity": "compact"}, [self.before, self.after])
        self.assertEqual(result.returncode, 0, result.stderr)
        args, props = self.rendered_props()
        self.assertEqual(props["fidelity"], "compact")
        self.assertIn("--crf=21", args)

        self.case_dir = Path(tempfile.mkdtemp(prefix="case-", dir=self.root))
        result = self.run_helper({"layout": "single", "fidelity": "standard"}, [self.after], extra_args=["--fidelity", "inspect"])
        self.assertEqual(result.returncode, 0, result.stderr)
        _, props = self.rendered_props()
        self.assertEqual(props["fidelity"], "inspect")

    def test_png_clip_is_refused_before_staging(self):
        still = self.case_dir / "proof.png"
        subprocess.run(
            ["ffmpeg", "-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=64x64", "-frames:v", "1", str(still)],
            check=True,
        )
        env = self.env()
        result = self.run_helper({"layout": "single"}, [still], env=env)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unsupported clip type", result.stderr)
        self.assertFalse(Path(env["FAKE_NPX_ARGS"]).exists(), "render must not be invoked for a refused clip")
        self.assert_no_new_staging_dirs()

    def test_still_preview_uses_same_normalization(self):
        result = self.run_helper({"layout": "side-by-side", "labels": ["a", "b"]}, [self.before, self.after], extra_args=["--still", "150"])
        self.assertEqual(result.returncode, 0, result.stderr)
        args, props = self.rendered_props()
        self.assertEqual(args[:2], ["remotion", "still"])
        self.assertIn("--frame=150", args)
        self.assertEqual(props["fidelity"], "inspect")
        self.assertAlmostEqual(props["clipDuration"], 5.0, delta=0.1)
        self.assertEqual(len(set(props["clips"])), 2)

    def test_concurrent_renders_keep_separate_staging_and_cleanup(self):
        runs = []
        for label in ("x", "y"):
            case_dir = Path(tempfile.mkdtemp(prefix=f"case-{label}-", dir=self.root))
            self.case_dir = case_dir
            env = self.env(FAKE_NPX_READY=case_dir / "ready", FAKE_NPX_BARRIER=self.root / "barrier")
            props_path = case_dir / "props.json"
            props_path.write_text(json.dumps({**BASE_PROPS, "layout": "side-by-side", "labels": ["a", "b"]}), encoding="utf-8")
            proc = subprocess.Popen(
                ["bash", str(HELPER), "--props", str(props_path), "--output", str(case_dir / "out.mp4"), str(self.before), str(self.after)],
                env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
            )
            runs.append((proc, env))

        deadline = time.time() + 60
        while time.time() < deadline and not all(Path(env["FAKE_NPX_READY"]).exists() for _, env in runs):
            time.sleep(0.05)
        self.assertTrue(all(Path(env["FAKE_NPX_READY"]).exists() for _, env in runs), "both renders should reach the render step")

        live = [d for d in staging_dirs() if d not in self.preexisting]
        self.assertEqual(len(live), 2, f"expected two isolated staging dirs, saw {live}")
        seen_dirs = []
        for _, env in runs:
            staged_paths = list(self.staged_hashes(env).keys())
            _, props = self.rendered_props(env)
            own_dir = {Path(p).parent.name for p in props["clips"]}
            self.assertEqual(len(own_dir), 1)
            seen_dirs.append(own_dir.pop())
            for clip in props["clips"]:
                self.assertTrue(
                    any(p.endswith(f"/public/{clip}") for p in staged_paths),
                    f"each render must see its own staged clip {clip} at render time: {staged_paths}",
                )
        self.assertEqual(len(set(seen_dirs)), 2, "renders must not share a staging directory")

        (self.root / "barrier").touch()
        for proc, _ in runs:
            _, err = proc.communicate(timeout=60)
            self.assertEqual(proc.returncode, 0, err)
        (self.root / "barrier").unlink()
        self.assert_sources_intact()
        self.assert_no_new_staging_dirs()

    def test_cancelled_render_removes_its_staging_dir(self):
        env = self.env(FAKE_NPX_READY=self.case_dir / "ready", FAKE_NPX_BARRIER=self.case_dir / "never")
        props_path = self.case_dir / "props.json"
        props_path.write_text(json.dumps({**BASE_PROPS, "layout": "single"}), encoding="utf-8")
        proc = subprocess.Popen(
            ["bash", str(HELPER), "--props", str(props_path), "--output", str(self.case_dir / "out.mp4"), str(self.after)],
            env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True,
        )
        deadline = time.time() + 60
        while time.time() < deadline and not Path(env["FAKE_NPX_READY"]).exists():
            time.sleep(0.05)
        self.assertTrue(Path(env["FAKE_NPX_READY"]).exists())
        self.assertEqual(len([d for d in staging_dirs() if d not in self.preexisting]), 1)

        os.killpg(proc.pid, signal.SIGTERM)
        proc.communicate(timeout=60)
        self.assertNotEqual(proc.returncode, 0)
        self.assert_no_new_staging_dirs()

    def test_cast_conversion_preserves_source_timeline(self):
        if shutil.which("agg") is None:
            self.fail("agg is required to test .cast conversion (plugin prerequisite)")
        cast = self.case_dir / "demo.cast"
        # A 7s pause between events: agg's default idle limit would shrink it to 5s.
        make_cast(cast, [(0.5, "hello\r\n"), (7.5, "world\r\n")])
        result = self.run_helper({"layout": "single"}, [cast])
        self.assertEqual(result.returncode, 0, result.stderr)
        _, props = self.rendered_props()
        # 7.5s of events plus agg's 3s hold on the final frame; idle compression would give ~8.5s.
        self.assertGreaterEqual(
            props["clipDuration"], 10.0,
            f"cast pauses must survive conversion unchanged (got {props['clipDuration']}s; ~8.5s means idle compression)",
        )
        self.assertTrue(props["clips"][0].endswith("clip-0.mp4"), props["clips"])


def ffprobe_stream(path):
    out = subprocess.run(
        [
            "ffprobe", "-v", "error", "-count_frames", "-select_streams", "v:0",
            "-show_entries", "stream=pix_fmt,color_range,color_space,nb_read_frames,width,height,r_frame_rate",
            "-of", "json", str(path),
        ],
        check=True, capture_output=True, text=True,
    ).stdout
    return json.loads(out)["streams"][0]


def center_pixel(path, frame_index):
    raw = subprocess.run(
        [
            "ffmpeg", "-v", "error", "-i", str(path),
            "-vf", f"select=eq(n\\,{frame_index}),format=rgb24,crop=1:1:iw/2:ih/2",
            "-fps_mode", "passthrough", "-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-",
        ],
        check=True, capture_output=True,
    ).stdout
    if len(raw) != 3:
        raise AssertionError(f"frame {frame_index} not found in {path}")
    return tuple(raw)


@unittest.skipUnless(os.environ.get("RENDER_SHOWCASE_E2E") == "1", "set RENDER_SHOWCASE_E2E=1 to render for real")
class RealRenderTest(unittest.TestCase):
    """End-to-end: the encoded file carries the pinned pixel format, the duration contract,
    and an unobscured final clip frame.

    Timeline (30 fps): title 0-119, clips start at 120 (after the title crossfade), the
    last clip frame is at 120 + content - 1 and is held through the outro crossfade.
    """

    FPS = 30
    TITLE_FRAMES = 4 * FPS
    OUTRO_FRAMES = int(3.5 * FPS)

    @classmethod
    def setUpClass(cls):
        for tool in ("ffmpeg", "ffprobe", "npx"):
            if shutil.which(tool) is None:
                raise RuntimeError(f"{tool} is required for the real render test")
        cls.root = Path(tempfile.mkdtemp(prefix="render-showcase-e2e-"))
        cls.clip = cls.root / "clip.mp4"
        # 1s clip: blue for the first half, lime for the second; the final state is lime.
        subprocess.run(
            [
                "ffmpeg", "-v", "error", "-y",
                "-f", "lavfi", "-i", "color=c=blue:size=320x320:rate=30", "-t", "1",
                "-vf", "drawbox=color=lime:t=fill:enable='gte(t,0.5)'",
                "-pix_fmt", "yuv420p", str(cls.clip),
            ],
            check=True,
        )

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.root, ignore_errors=True)

    def render(self, name, speed):
        props = {**BASE_PROPS, "layout": "single", "fidelity": "compact", "width": 640, "height": 360, "speed": speed}
        props_path = self.root / f"{name}.json"
        props_path.write_text(json.dumps(props), encoding="utf-8")
        out = self.root / f"{name}.mp4"
        result = subprocess.run(
            ["bash", str(HELPER), "--props", str(props_path), "--output", str(out), str(self.clip)],
            capture_output=True, text=True, timeout=900,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        subprocess.run(["ffmpeg", "-v", "error", "-xerror", "-i", str(out), "-f", "null", "-"], check=True)
        return out

    def test_encoded_output_matches_contract(self):
        content_frames = 30  # 1s clip at speed 1
        out = self.render("main", speed=1)
        stream = ffprobe_stream(out)
        self.assertEqual(stream["pix_fmt"], "yuv420p", stream)
        self.assertEqual(stream["color_range"], "tv", stream)
        self.assertEqual(stream["color_space"], "bt709", stream)
        self.assertEqual((stream["width"], stream["height"]), (640, 360))
        self.assertEqual(int(stream["nb_read_frames"]), self.TITLE_FRAMES + content_frames + self.OUTRO_FRAMES)

        first_clip_frame = self.TITLE_FRAMES
        last_clip_frame = first_clip_frame + content_frames - 1
        # Window chrome fades in over the first 15 content frames; sample once it is opaque.
        r, g, b = center_pixel(out, first_clip_frame + 14)
        self.assertTrue(b > r + 60 and b > g + 60, f"expected blue at first-half clip frame, got {(r, g, b)}")
        r, g, b = center_pixel(out, last_clip_frame)
        self.assertTrue(g > r + 60 and g > b + 60, f"expected lime at the last clip frame, got {(r, g, b)}")

    def test_content_shorter_than_transition_still_renders(self):
        content_frames = 8  # ceil(1s / 4 * 30): shorter than the 15-frame transitions
        out = self.render("short", speed=4)
        stream = ffprobe_stream(out)
        self.assertEqual(int(stream["nb_read_frames"]), self.TITLE_FRAMES + content_frames + self.OUTRO_FRAMES)
        r, g, b = center_pixel(out, self.TITLE_FRAMES + content_frames - 1)
        self.assertTrue(g > r and g > b, f"expected the lime final state at the last clip frame, got {(r, g, b)}")


if __name__ == "__main__":
    unittest.main(argv=[sys.argv[0], "-v"])
