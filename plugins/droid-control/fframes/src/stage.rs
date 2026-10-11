//! Clip staging: every input gets a distinct `clip-<index>` name inside the render's work
//! directory (fframes keys decoders by file name), and asciinema casts become MP4s on their
//! own timeline.

use crate::cast::Cast;
use crate::theme::Encoding;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The Droid CLI terminal palette for agg.
const DROID_CLI_THEME: &str = "181818,e0d0c0,15161e,f7768e,9ece6a,e0af68,7aa2f7,bb9af7,7dcfff,a9b1d6,414868,f7768e,9ece6a,e0af68,7aa2f7,bb9af7,7dcfff,c0caf5";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Cast,
    Video,
}

/// The clip kind by extension; anything else (stills included) is refused.
pub fn kind(clip: &Path) -> Result<Kind, String> {
    match clip.extension().and_then(|ext| ext.to_str()) {
        Some("cast") => Ok(Kind::Cast),
        Some("mp4" | "webm") => Ok(Kind::Video),
        _ => Err(format!(
            "unsupported clip type: {} (clips are .cast, .mp4, or .webm; stills belong in the screenshot artifact path)",
            clip.display()
        )),
    }
}

/// A clip as the composition reads it: the staged video, and the recording a cast was
/// rendered from.
pub struct Staged {
    pub path: PathBuf,
    pub cast: Option<Cast>,
}

/// Links (or converts) every clip into `work_dir`, in order.
pub fn stage(clips: &[PathBuf], work_dir: &Path, encoding: &Encoding) -> Result<Vec<Staged>, String> {
    clips
        .iter()
        .enumerate()
        .map(|(index, clip)| match kind(clip)? {
            Kind::Cast => {
                let path = work_dir.join(format!("clip-{index}.mp4"));
                let cast = Cast::read(clip)?;
                convert_cast(clip, &cast, &path, encoding)?;
                Ok(Staged { path, cast: Some(cast) })
            }
            Kind::Video => {
                let ext = clip.extension().and_then(|ext| ext.to_str()).unwrap_or_default();
                let staged = work_dir.join(format!("clip-{index}.{ext}"));
                let source =
                    fs::canonicalize(clip).map_err(|err| format!("clip is not readable: {}: {err}", clip.display()))?;
                std::os::unix::fs::symlink(&source, &staged)
                    .map_err(|err| format!("could not stage {}: {err}", clip.display()))?;
                Ok(Staged { path: staged, cast: None })
            }
        })
        .collect()
}

fn run(command: &mut Command, what: &str) -> Result<(), String> {
    let output = command.output().map_err(|err| format!("could not run {what}: {err}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!("{what} failed ({}): {}", output.status, String::from_utf8_lossy(&output.stderr).trim()))
}

/// agg renders the cast at its own pace (no idle compression: the composition applies
/// `speed` to every clip alike), then ffmpeg encodes the GIF losslessly: the composition
/// decodes it again, so it loses nothing to a second lossy encode, and the fastest preset
/// costs only file size.
fn convert_cast(clip: &Path, cast: &Cast, output: &Path, encoding: &Encoding) -> Result<(), String> {
    let gif = output.with_extension("gif");
    run(
        Command::new("agg")
            .args(["--speed", "1", "--idle-time-limit", "1000000000", "--renderer", "fontdue"])
            .args(["--cols", &cast.cols.to_string(), "--rows", &cast.rows.to_string()])
            .args(["--fps-cap", &encoding.cast_fps_cap.to_string(), "--theme", DROID_CLI_THEME])
            .arg(clip)
            .arg(&gif),
        "agg",
    )?;
    run(
        Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-i"])
            .arg(&gif)
            .args(["-movflags", "+faststart", "-pix_fmt", "yuv420p", "-c:v", "libx264", "-preset", "ultrafast"])
            .args(["-qp", "0", "-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2"])
            .arg(output),
        "ffmpeg",
    )?;
    fs::remove_file(&gif).map_err(|err| format!("could not remove {}: {err}", gif.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_casts_and_videos_are_clips() {
        assert_eq!(kind(Path::new("a/demo.cast")), Ok(Kind::Cast));
        assert_eq!(kind(Path::new("demo.webm")), Ok(Kind::Video));
        let refused = kind(Path::new("proof.png")).unwrap_err();
        assert!(refused.starts_with("unsupported clip type: proof.png"), "{refused}");
    }
}
