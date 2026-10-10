//! Clip sources, probed once with ffprobe: the longest one sets the content length and
//! every clip holds its own last frame once it ends.

use crate::cast::Cast;
use crate::stage::Staged;
use crate::timing::FPS;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub struct Clip {
    /// The file name: fframes keys media and decoders on it, so it is unique per render.
    pub name: String,
    pub path: PathBuf,
    /// Container duration in source seconds.
    pub duration: f64,
    /// Output-frame offset of the last frame; playback holds it after the clip ends.
    pub last_frame: usize,
    /// Frame size in pixels.
    pub size: (f32, f32),
    /// The recording a cast clip was rendered from.
    pub cast: Option<Cast>,
}

#[derive(Deserialize)]
struct Probe {
    format: ProbeFormat,
    #[serde(default)]
    streams: Vec<ProbeStream>,
    #[serde(default)]
    packets: Vec<ProbePacket>,
}

#[derive(Deserialize)]
struct ProbeStream {
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
struct ProbeFormat {
    duration: Option<String>,
}

#[derive(Deserialize)]
struct ProbePacket {
    pts_time: Option<String>,
}

impl Clip {
    pub fn probe_all(staged: Vec<Staged>) -> Result<Vec<Clip>, String> {
        let mut names = HashSet::new();
        staged
            .into_iter()
            .map(|Staged { path, cast }| {
                let clip = Clip { cast, ..Clip::probe(&path)? };
                if !names.insert(clip.name.clone()) {
                    return Err(format!(
                        "clips must have distinct file names, {} repeats one (stage them as clip-<index>.<ext>)",
                        path.display()
                    ));
                }
                Ok(clip)
            })
            .collect()
    }

    fn probe(path: &Path) -> Result<Clip, String> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "format=duration:stream=width,height:packet=pts_time",
                "-of",
                "json",
            ])
            .arg(path)
            .output()
            .map_err(|err| format!("could not run ffprobe: {err}"))?;
        if !output.status.success() {
            return Err(format!(
                "ffprobe could not read clip {}: {}",
                path.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let probe: Probe = serde_json::from_slice(&output.stdout)
            .map_err(|err| format!("unexpected ffprobe output for {}: {err}", path.display()))?;
        Clip::from_probe(path, &probe)
    }

    fn from_probe(path: &Path, probe: &Probe) -> Result<Clip, String> {
        let seconds = |value: &Option<String>| value.as_deref().and_then(|v| v.parse::<f64>().ok());
        let duration = seconds(&probe.format.duration)
            .ok_or_else(|| format!("ffprobe could not read a duration from clip: {}", path.display()))?;
        let last_pts = probe
            .packets
            .iter()
            .filter_map(|packet| seconds(&packet.pts_time))
            .fold(None, |max: Option<f64>, pts| Some(max.map_or(pts, |m| m.max(pts))))
            .ok_or_else(|| format!("clip has no video frames: {}", path.display()))?;
        let stream = probe.streams.first().ok_or_else(|| format!("clip has no video stream: {}", path.display()))?;
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("clip path has no UTF-8 file name: {}", path.display()))?
            .to_owned();
        Ok(Clip {
            name,
            path: path.to_owned(),
            duration,
            // The decoder shows the first frame at or after the requested time, so the last
            // frame's own timestamp (not the container end) is the latest offset to ask for.
            last_frame: (last_pts * FPS as f64 + 1e-6).floor().max(0.) as usize,
            size: (stream.width as f32, stream.height as f32),
            cast: None,
        })
    }
}

/// The longest clip in source seconds, rounded to centiseconds.
pub fn longest(clips: &[Clip]) -> Option<f64> {
    clips.iter().map(|clip| (clip.duration * 100.).round() / 100.).reduce(f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe(json: &str) -> Result<Clip, String> {
        Clip::from_probe(Path::new("/tmp/clip-0.mp4"), &serde_json::from_str(json).unwrap())
    }

    #[test]
    fn last_frame_comes_from_the_last_presented_packet() {
        let clip = probe(
            r#"{"format":{"duration":"5.000000"},"streams":[{"width":978,"height":650}],"packets":[{"pts_time":"0.000000"},{"pts_time":"4.966667"},{"pts_time":"4.933333"}]}"#,
        )
        .unwrap();
        assert_eq!(clip.name, "clip-0.mp4");
        assert_eq!(clip.duration, 5.);
        assert_eq!(clip.last_frame, 149);
        assert_eq!(clip.size, (978., 650.));
    }

    #[test]
    fn unreadable_probes_are_errors() {
        let stream = r#""streams":[{"width":2,"height":2}]"#;
        assert!(
            probe(&format!(r#"{{"format":{{"duration":"N/A"}},{stream},"packets":[{{"pts_time":"0"}}]}}"#)).is_err()
        );
        assert!(probe(&format!(r#"{{"format":{{"duration":"1.0"}},{stream},"packets":[]}}"#)).is_err());
        assert!(probe(r#"{"format":{"duration":"1.0"},"packets":[{"pts_time":"0"}]}"#).is_err());
    }

    #[test]
    fn longest_rounds_to_centiseconds() {
        let clip = |duration| Clip {
            name: String::new(),
            path: PathBuf::new(),
            duration,
            last_frame: 0,
            size: (2., 2.),
            cast: None,
        };
        assert_eq!(longest(&[clip(1.234), clip(5.006)]), Some(5.01));
        assert_eq!(longest(&[]), None);
    }
}
