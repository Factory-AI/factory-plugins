//! `droid-showcase`: renders the droid-control showcase video (or one still) from props JSON
//! and clip files.

use clap::Parser;
use droid_showcase::clips::Clip;
use droid_showcase::props::ShowcaseProps;
use droid_showcase::{Showcase, render, stage};
use serde_json::json;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Render a droid-control showcase: title card, clips with overlays, Droid outro.
#[derive(Debug, Parser)]
#[command(name = "droid-showcase", version)]
struct Cli {
    /// Props JSON file.
    #[arg(long, value_name = "FILE", required_unless_present = "props_inline", conflicts_with = "props_inline")]
    props: Option<PathBuf>,
    /// Props JSON given inline.
    #[arg(long, value_name = "JSON")]
    props_inline: Option<String>,
    /// Fidelity profile (compact, standard, inspect); overrides the props.
    #[arg(long, value_name = "PROFILE")]
    fidelity: Option<String>,
    /// Render only this frame, as a PNG.
    #[arg(long, value_name = "FRAME")]
    still: Option<usize>,
    /// Output file: an .mp4, or a .png with --still.
    #[arg(long, short)]
    output: PathBuf,
    /// Clips (.cast, .mp4 or .webm); side-by-side shows the first two.
    #[arg(required = true, value_name = "CLIP")]
    clips: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let work_dir = match tempfile::Builder::new().prefix("droid-showcase-").tempdir() {
        Ok(dir) => dir.keep(),
        Err(err) => {
            eprintln!("error: could not create a work directory: {err}");
            return ExitCode::FAILURE;
        }
    };
    remove_on_signal(work_dir.clone());
    let result = run(&cli, &work_dir);
    let _ = fs::remove_dir_all(&work_dir);
    match result {
        Ok(()) => {
            println!("{}", cli.output.display());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Ctrl-C or a cancelled task stops the render without leaving staged clips behind; the
/// ffmpeg and agg children share the process group and stop with it.
fn remove_on_signal(work_dir: PathBuf) {
    let mut signals = Signals::new([SIGINT, SIGTERM]).expect("SIGINT and SIGTERM handlers install");
    std::thread::spawn(move || {
        if let Some(signal) = signals.forever().next() {
            let _ = fs::remove_dir_all(&work_dir);
            std::process::exit(128 + signal);
        }
    });
}

fn run(cli: &Cli, work_dir: &Path) -> Result<(), String> {
    let json = match (&cli.props, &cli.props_inline) {
        (Some(file), _) => {
            fs::read_to_string(file).map_err(|err| format!("props file is not readable: {}: {err}", file.display()))?
        }
        (None, Some(inline)) => inline.clone(),
        (None, None) => unreachable!("clap requires --props or --props-inline"),
    };
    for clip in &cli.clips {
        stage::kind(clip)?;
        fs::File::open(clip).map_err(|err| format!("clip is not readable: {}: {err}", clip.display()))?;
    }
    let props = ShowcaseProps::from_json(&json, cli.fidelity.as_deref())?;
    let encoding = props.fidelity.encoding();

    let staged = stage::stage(&cli.clips, work_dir, &encoding)?;
    let clips = Clip::probe_all(&staged)?;
    let showcase = Showcase::new(props, clips)?;

    let props = showcase.props();
    let timeline = showcase.timeline();
    let plan = json!({
        "fidelity": props.fidelity,
        "width": props.width,
        "height": props.height,
        "speed": props.speed,
        "longestClip": droid_showcase::clips::longest(showcase.clips()),
        "frames": timeline.total(),
        "crf": encoding.crf,
        "preset": encoding.preset,
        "clips": showcase.clips().iter().map(|clip| clip.path.display().to_string()).collect::<Vec<_>>(),
        "workDir": work_dir.display().to_string(),
    });
    eprintln!("showcase plan: {plan}");

    let media = render::media(showcase.clips());
    match cli.still {
        Some(frame) => render::still(&showcase, &media, frame, &cli.output),
        None => render::video(&showcase, &media, &cli.output),
    }
}
