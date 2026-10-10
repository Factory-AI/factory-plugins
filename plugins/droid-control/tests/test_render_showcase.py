"""Behavioral tests for scripts/render-showcase.sh and the droid-showcase binary it runs.

Every test drives the real script against synthetic ffmpeg media. The binary validates
props, stages and probes the clips, prints its resolved plan as one `showcase plan: {...}`
line on stderr, then renders. Stills keep most cases fast; RealRenderTest encodes videos.

Run:  python3 -m unittest discover -s plugins/droid-control/tests -p 'test_*.py'
Needs cargo (the script builds ../fframes on first use), ffmpeg, ffprobe, and agg.
"""

import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

PLUGIN_ROOT = Path(__file__).resolve().parents[1]
HELPER = PLUGIN_ROOT / "scripts" / "render-showcase.sh"
PLAN_PREFIX = "showcase plan: "

BASE_PROPS = {
    "layout": "single",
    "labels": [],
    "title": "t",
    "subtitle": "s",
    "preset": "macos",
    "keys": [],
    "effects": [],
}


def require_tools(*tools):
    missing = [tool for tool in tools if shutil.which(tool) is None]
    if missing:
        raise unittest.SkipTest(f"required for these tests: {', '.join(missing)}")


def make_video(path, seconds, source="testsrc2=size=320x240:rate=30", vf=None):
    path.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            "ffmpeg", "-v", "error", "-y", "-f", "lavfi", "-i", source, "-t", str(seconds),
            *(["-vf", vf] if vf else []), "-pix_fmt", "yuv420p", str(path),
        ],
        check=True,
    )


def make_cast(path, events):
    header = {"version": 2, "width": 40, "height": 10}
    lines = [json.dumps(header)] + [json.dumps([t, "o", text]) for t, text in events]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def plan_of(stderr):
    lines = [line for line in stderr.splitlines() if line.startswith(PLAN_PREFIX)]
    return json.loads(lines[0][len(PLAN_PREFIX):]) if lines else None


def helper_cmd(props_path, output, clips, extra_args=()):
    return ["bash", str(HELPER), "--props", str(props_path), "--output", str(output), *extra_args, *map(str, clips)]


def start_until_plan(cmd):
    """Starts a render in its own process group and returns it once it has printed its plan."""
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
    for line in proc.stderr:
        if line.startswith(PLAN_PREFIX):
            return proc, json.loads(line[len(PLAN_PREFIX):])
    proc.wait()
    raise AssertionError(f"render exited ({proc.returncode}) before printing its plan")


class RenderShowcaseTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        require_tools("bash", "cargo", "ffmpeg", "ffprobe")
        cls.root = Path(tempfile.mkdtemp(prefix="render-showcase-test-"))
        cls.before = cls.root / "before" / "recording.mp4"
        cls.after = cls.root / "after" / "recording.mp4"
        make_video(cls.before, 2)
        make_video(cls.after, 5)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.root, ignore_errors=True)

    def case_dir(self):
        return Path(tempfile.mkdtemp(prefix="case-", dir=self.root))

    def write_props(self, case, props):
        path = case / "props.json"
        path.write_text(json.dumps({**BASE_PROPS, **props}), encoding="utf-8")
        return path

    def still(self, props, clips, extra_args=()):
        """Renders frame 0 and returns (result, plan, output)."""
        case = self.case_dir()
        output = case / "frame.png"
        cmd = helper_cmd(self.write_props(case, props), output, clips, ["--still", "0", *extra_args])
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=900)
        return result, plan_of(result.stderr), output

    def test_same_basename_inputs_stay_distinct_and_leave_nothing_behind(self):
        result, plan, output = self.still(
            {"layout": "side-by-side", "labels": ["BEFORE", "AFTER"], "speed": 2, "width": 320, "height": 180},
            [self.before, self.after],
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([Path(clip).name for clip in plan["clips"]], ["clip-0.mp4", "clip-1.mp4"])
        self.assertAlmostEqual(plan["longestClip"], 5.0, delta=0.1, msg="longest source, in source seconds")
        self.assertEqual(plan["speed"], 2)
        self.assertEqual(result.stdout.strip(), str(output))
        self.assertTrue(output.stat().st_size > 0)
        self.assertFalse(Path(plan["workDir"]).exists(), "staged clips must not outlive the render")

    def test_fidelity_resolves_size_and_encoding(self):
        sbs = {"layout": "side-by-side", "labels": ["a", "b"]}
        cases = [
            (sbs, (), ("inspect", 2560, 1440, 14, "slow")),
            ({"layout": "single"}, (), ("standard", 1920, 1080, 18, "slow")),
            ({**sbs, "fidelity": "compact"}, (), ("compact", 1920, 1080, 21, "medium")),
            ({"layout": "single", "fidelity": "standard"}, ("--fidelity", "inspect"), ("inspect", 2560, 1440, 14, "slow")),
        ]
        for props, extra_args, expected in cases:
            with self.subTest(props=props, extra_args=extra_args):
                clips = [self.before, self.after] if props["layout"] == "side-by-side" else [self.after]
                result, plan, _ = self.still(props, clips, extra_args)
                self.assertEqual(result.returncode, 0, result.stderr)
                actual = tuple(plan[key] for key in ("fidelity", "width", "height", "crf", "preset"))
                self.assertEqual(actual, expected)

    def test_refused_inputs_fail_before_rendering(self):
        png = self.root / "proof.png"
        subprocess.run(
            ["ffmpeg", "-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=64x64", "-frames:v", "1", str(png)],
            check=True,
        )
        cases = [
            ({}, [png], "unsupported clip type"),
            ({"speed": 0}, [self.after], "speed must be a positive finite number"),
            ({"fidelity": "ultra"}, [self.after], "fidelity"),
        ]
        for props, clips, message in cases:
            with self.subTest(message=message):
                result, plan, output = self.still(props, clips)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(message, result.stderr)
                self.assertIsNone(plan, "nothing may be staged for refused input")
                self.assertFalse(output.exists())

    def test_concurrent_renders_use_separate_work_dirs(self):
        runs = []
        for _ in range(2):
            case = self.case_dir()
            props = self.write_props(case, {"fidelity": "compact", "width": 640, "height": 360})
            runs.append(start_until_plan(helper_cmd(props, case / "out.mp4", [self.before])))
        work_dirs = [Path(plan["workDir"]) for _, plan in runs]
        self.assertNotEqual(work_dirs[0], work_dirs[1])
        self.assertTrue(all(d.is_dir() for d in work_dirs), "both renders hold their staged clips at once")
        for proc, _ in runs:
            _, err = proc.communicate(timeout=900)
            self.assertEqual(proc.returncode, 0, err)
        self.assertFalse(any(d.exists() for d in work_dirs))

    def test_cancelled_render_removes_its_work_dir(self):
        case = self.case_dir()
        props = self.write_props(case, {"fidelity": "compact", "width": 640, "height": 360})
        proc, plan = start_until_plan(helper_cmd(props, case / "out.mp4", [self.after]))
        self.assertTrue(Path(plan["workDir"]).is_dir())
        os.killpg(proc.pid, signal.SIGTERM)
        proc.communicate(timeout=60)
        self.assertEqual(proc.returncode, 128 + signal.SIGTERM)
        self.assertFalse(Path(plan["workDir"]).exists())

    def test_cast_conversion_preserves_source_timeline(self):
        if shutil.which("agg") is None:
            self.fail("agg is required to test .cast conversion (plugin prerequisite)")
        cast = self.root / "demo.cast"
        # A 7s pause between events: agg's default idle limit would shrink it to 5s.
        make_cast(cast, [(0.5, "hello\r\n"), (7.5, "world\r\n")])
        result, plan, _ = self.still({"width": 320, "height": 180}, [cast])
        self.assertEqual(result.returncode, 0, result.stderr)
        # 7.5s of events plus agg's 3s hold on the final frame; idle compression would give ~8.5s.
        self.assertGreaterEqual(
            plan["longestClip"], 10.0,
            f"cast pauses must survive conversion unchanged (got {plan['longestClip']}s; ~8.5s means idle compression)",
        )
        self.assertEqual(Path(plan["clips"][0]).name, "clip-0.mp4")


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
        require_tools("bash", "cargo", "ffmpeg", "ffprobe")
        cls.root = Path(tempfile.mkdtemp(prefix="render-showcase-e2e-"))
        cls.clip = cls.root / "clip.mp4"
        # 1s clip: blue for the first half, lime for the second; the final state is lime.
        make_video(cls.clip, 1, "color=c=blue:size=320x320:rate=30", "drawbox=color=lime:t=fill:enable='gte(t,0.5)'")

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.root, ignore_errors=True)

    def render(self, name, speed):
        props = {**BASE_PROPS, "fidelity": "compact", "width": 640, "height": 360, "speed": speed}
        props_path = self.root / f"{name}.json"
        props_path.write_text(json.dumps(props), encoding="utf-8")
        out = self.root / f"{name}.mp4"
        result = subprocess.run(helper_cmd(props_path, out, [self.clip]), capture_output=True, text=True, timeout=900)
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
