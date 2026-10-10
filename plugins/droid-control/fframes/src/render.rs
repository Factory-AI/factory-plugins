//! Rasterising the composition at the props size: single PNG stills, and whole videos
//! rendered on every core and piped to ffmpeg for the H.264 encode.

use crate::Showcase;
use crate::clips::Clip;
use crate::fonts;
use fframes::media::VideoMedia;
use fframes::{Color, CpuFrameRenderer, DynamicMediaProvider, FrameRenderer, Previewer, RenderOptions, RgbaFrame};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

/// Consecutive frames per worker turn: long enough runs keep each worker's decoders reading
/// forward, short enough that the in-order writer never waits long.
const BLOCK: usize = 8;

/// The clip files and the embedded fonts, as fframes media.
pub fn media(clips: &[Clip]) -> DynamicMediaProvider<'static> {
    let videos =
        clips.iter().map(|clip| (clip.name.clone(), VideoMedia { path: clip.path.clone(), metadata: None })).collect();
    DynamicMediaProvider::new(HashMap::new(), HashMap::new(), HashMap::new(), videos, fonts::font_data())
}

struct FrameSource<'a> {
    previewer: Previewer<'a, 'a, Showcase>,
    renderer: CpuFrameRenderer,
    size: (u32, u32),
}

impl<'a> FrameSource<'a> {
    fn new(showcase: &'a Showcase, media: &'a DynamicMediaProvider<'static>) -> Result<Self, String> {
        // System fonts back the embedded Geist families for glyphs they lack (CJK, symbols).
        let options = RenderOptions { media: Some(media), load_system_fonts: true, ..Default::default() };
        let previewer =
            Previewer::new(showcase, &options).map_err(|err| format!("could not prepare the renderer: {err}"))?;
        let props = showcase.props();
        Ok(Self { previewer, renderer: CpuFrameRenderer::default(), size: (props.width, props.height) })
    }

    fn render(&mut self, frame: usize) -> Result<RgbaFrame, String> {
        let tree = self.previewer.svg_tree(frame).map_err(|err| format!("frame {frame}: {err}"))?;
        let (width, height) = self.size;
        self.renderer.render_tree(&tree, Color::BLACK, width, height).map_err(|err| format!("frame {frame}: {err}"))
    }
}

/// Renders one frame to a PNG.
pub fn still(
    showcase: &Showcase,
    media: &DynamicMediaProvider<'static>,
    frame: usize,
    output: &Path,
) -> Result<(), String> {
    let total = showcase.timeline().total();
    if frame >= total {
        return Err(format!("--still {frame} is past the last frame ({})", total - 1));
    }
    FrameSource::new(showcase, media)?
        .render(frame)?
        .save_png(output)
        .map_err(|err| format!("could not write {}: {err}", output.display()))
}

/// Renders every frame and encodes them to H.264 (yuv420p, BT.709 limited range).
pub fn video(showcase: &Showcase, media: &DynamicMediaProvider<'static>, output: &Path) -> Result<(), String> {
    let total = showcase.timeline().total();
    let props = showcase.props();
    let encoding = props.fidelity.encoding();
    let mut ffmpeg = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba"])
        .args(["-s", &format!("{}x{}", props.width, props.height), "-framerate", "30", "-i", "-"])
        .args(["-c:v", "libx264", "-preset", encoding.preset, "-crf", &encoding.crf.to_string()])
        .args(["-vf", "scale=out_color_matrix=bt709:out_range=tv", "-pix_fmt", "yuv420p"])
        .args(["-colorspace", "bt709", "-color_primaries", "bt709", "-color_trc", "bt709", "-color_range", "tv"])
        .args(["-movflags", "+faststart"])
        .arg(output)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|err| format!("could not run ffmpeg: {err}"))?;
    let mut encoder = ffmpeg.stdin.take().expect("ffmpeg stdin is piped");
    let workers = thread::available_parallelism().map_or(1, usize::from).min(total.div_ceil(BLOCK)).max(1);

    let rendered = thread::scope(|scope| {
        // Worker `w` renders blocks w, w + workers, ...; the writer drains them in frame order.
        let queues: Vec<mpsc::Receiver<Result<RgbaFrame, String>>> = (0..workers)
            .map(|worker| {
                let (frames, queue) = mpsc::sync_channel(BLOCK);
                scope.spawn(move || {
                    let mut source = match FrameSource::new(showcase, media) {
                        Ok(source) => source,
                        Err(err) => return drop(frames.send(Err(err))),
                    };
                    for frame in (0..total).filter(|frame| (frame / BLOCK) % workers == worker) {
                        let result = source.render(frame);
                        let failed = result.is_err();
                        if frames.send(result).is_err() || failed {
                            return;
                        }
                    }
                });
                queue
            })
            .collect();
        for frame in 0..total {
            let rgba = queues[(frame / BLOCK) % workers].recv().map_err(|_| "a render worker stopped".to_owned())??;
            encoder.write_all(&rgba.pixels).map_err(|err| format!("ffmpeg stopped accepting frames: {err}"))?;
            if (frame + 1) % 30 == 0 || frame + 1 == total {
                eprint!("\rrendered {}/{total} frames", frame + 1);
            }
        }
        eprintln!();
        Ok::<(), String>(())
    });
    drop(encoder);
    if rendered.is_err() {
        let _ = ffmpeg.kill();
    }
    let status = ffmpeg.wait().map_err(|err| format!("ffmpeg did not finish: {err}"))?;
    rendered?;
    if !status.success() {
        return Err(format!("ffmpeg failed to encode {} ({status})", output.display()));
    }
    Ok(())
}
