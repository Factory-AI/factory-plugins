//! Rasterising the composition at the props size: single PNG stills, and whole videos
//! rendered on every core, converted to yuv420p there and piped to ffmpeg for the H.264
//! encode.

use crate::Showcase;
use crate::clips::Clip;
use crate::fonts;
use crate::raster;
use fframes::media::VideoMedia;
use fframes::{DynamicMediaProvider, Previewer, RenderOptions, RgbaFrame};
use multiversion::multiversion;
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
}

impl<'a> FrameSource<'a> {
    fn new(showcase: &'a Showcase, media: &'a DynamicMediaProvider<'static>) -> Result<Self, String> {
        // System fonts back the embedded Geist families for glyphs they lack (CJK, symbols).
        let options = RenderOptions { media: Some(media), load_system_fonts: true, ..Default::default() };
        let previewer =
            Previewer::new(showcase, &options).map_err(|err| format!("could not prepare the renderer: {err}"))?;
        Ok(Self { previewer })
    }

    /// The showcase composites each frame itself, so its tree is that one image.
    fn render(&mut self, frame: usize) -> Result<RgbaFrame, String> {
        let tree = self.previewer.svg_tree(frame).map_err(|err| format!("frame {frame}: {err}"))?;
        let pixels = raster::sole_image(tree).expect("showcase frames are one image");
        // Opaque, so the premultiplied pixels are the straight ones.
        Ok(RgbaFrame { width: pixels.width, height: pixels.height, pixels: pixels.data })
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

/// `frame` in yuv420p with BT.709 limited range, what the encoder takes, with each chroma
/// sample the mean of its 2x2 pixels. Coefficients are in 2^-15.
#[multiversion(targets = "simd")]
fn yuv420(frame: &RgbaFrame) -> Vec<u8> {
    let (width, height) = (frame.width as usize, frame.height as usize);
    let mut yuv = vec![0; width * height * 3 / 2];
    let (luma, chroma) = yuv.split_at_mut(width * height);
    let (blue, red) = chroma.split_at_mut(width * height / 4);
    for (y, px) in luma.iter_mut().zip(frame.pixels.as_chunks::<4>().0) {
        let [r, g, b, _] = px.map(i32::from);
        *y = ((5983 * r + 20127 * g + 2032 * b + (16 << 15) + (1 << 14)) >> 15) as u8;
    }
    let rows =
        frame.pixels.chunks_exact(width * 8).zip(blue.chunks_exact_mut(width / 2).zip(red.chunks_exact_mut(width / 2)));
    for (pair, (blue, red)) in rows {
        let (top, bottom) = pair.split_at(width * 4);
        let blocks = top.as_chunks::<8>().0.iter().zip(bottom.as_chunks::<8>().0);
        for ((cb, cr), (top, bottom)) in blue.iter_mut().zip(red.iter_mut()).zip(blocks) {
            let sum =
                |c: usize| i32::from(top[c]) + i32::from(top[c + 4]) + i32::from(bottom[c]) + i32::from(bottom[c + 4]);
            let (r, g, b) = (sum(0), sum(1), sum(2));
            *cb = ((-3298 * r - 11094 * g + 14392 * b + (128 << 17) + (1 << 16)) >> 17) as u8;
            *cr = ((14392 * r - 13072 * g - 1320 * b + (128 << 17) + (1 << 16)) >> 17) as u8;
        }
    }
    yuv
}

/// Renders every frame and encodes them to H.264 (yuv420p, BT.709 limited range).
pub fn video(showcase: &Showcase, media: &DynamicMediaProvider<'static>, output: &Path) -> Result<(), String> {
    let total = showcase.timeline().total();
    let props = showcase.props();
    let encoding = props.fidelity.encoding();
    let mut ffmpeg = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "rawvideo", "-pix_fmt", "yuv420p"])
        .args(["-s", &format!("{}x{}", props.width, props.height), "-framerate", "30"])
        .args([
            "-colorspace",
            "bt709",
            "-color_primaries",
            "bt709",
            "-color_trc",
            "bt709",
            "-color_range",
            "tv",
            "-i",
            "-",
        ])
        .args(["-c:v", "libx264", "-preset", encoding.preset, "-crf", &encoding.crf.to_string(), "-pix_fmt", "yuv420p"])
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
        let queues: Vec<mpsc::Receiver<Result<Vec<u8>, String>>> = (0..workers)
            .map(|worker| {
                let (frames, queue) = mpsc::sync_channel(BLOCK);
                scope.spawn(move || {
                    let mut source = match FrameSource::new(showcase, media) {
                        Ok(source) => source,
                        Err(err) => return drop(frames.send(Err(err))),
                    };
                    for frame in (0..total).filter(|frame| (frame / BLOCK) % workers == worker) {
                        let result = source.render(frame).map(|rgba| yuv420(&rgba));
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
            let yuv = queues[(frame / BLOCK) % workers].recv().map_err(|_| "a render worker stopped".to_owned())??;
            encoder.write_all(&yuv).map_err(|err| format!("ffmpeg stopped accepting frames: {err}"))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yuv420_is_bt709_limited_range_with_averaged_chroma() {
        // A 2x2 block: white, black, and pure red twice.
        let pixels = [[255, 255, 255, 255], [0, 0, 0, 255], [255, 0, 0, 255], [255, 0, 0, 255]].concat();
        let yuv = yuv420(&RgbaFrame { width: 2, height: 2, pixels });
        assert_eq!(yuv[..4], [235, 16, 63, 63]);
        // Red's (102, 240) and black and white's neutral 128, averaged.
        assert_eq!(yuv[4..], [115, 184]);
    }
}
