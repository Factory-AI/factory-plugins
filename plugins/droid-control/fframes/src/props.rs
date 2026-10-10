//! The Showcase props contract. Missing fields take the composition defaults and unknown
//! fields are ignored. Clips are not props: they are passed to the binary and probed.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Preset {
    Macos,
    Minimal,
    Hero,
    Presentation,
    Factory,
    FactoryHero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    Single,
    SideBySide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fidelity {
    Compact,
    Standard,
    Inspect,
}

/// How clip video is sized inside its panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObjectFit {
    /// Preserve the aspect ratio and letterbox.
    #[default]
    Contain,
    /// Fill the panel and crop the overflow.
    Cover,
    /// Stretch to the panel (distorts the aspect ratio).
    Fill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransitionStyle {
    #[default]
    MotionBlur,
    Flash,
    WhipPan,
    LightLeak,
    GlitchLite,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Keystroke {
    pub t: f32,
    pub label: String,
    pub dur: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section {
    pub t: f32,
    pub title: String,
}

/// A CSS-style percentage such as `"25%"`, stored as a fraction (0.25).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pct(pub f32);

impl<'de> Deserialize<'de> for Pct {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        let number = raw.trim().trim_end_matches('%').trim();
        number
            .parse::<f32>()
            .ok()
            .filter(|value| value.is_finite())
            .map(|value| Pct(value / 100.))
            .ok_or_else(|| serde::de::Error::custom(format!("expected a percentage like \"25%\", got {raw:?}")))
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Region {
    pub x: Pct,
    pub y: Pct,
    pub w: Pct,
    pub h: Pct,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Point {
    pub x: Pct,
    pub y: Pct,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "fx", rename_all = "kebab-case")]
pub enum Effect {
    FadeIn {
        t: f32,
        dur: f32,
    },
    FadeOut {
        t: f32,
        dur: f32,
    },
    Zoom {
        t: f32,
        dur: f32,
        to: Region,
    },
    Spotlight {
        t: f32,
        dur: f32,
        on: Region,
        #[serde(default = "default_dim")]
        dim: f32,
    },
    Callout {
        t: f32,
        dur: f32,
        text: String,
        at: Point,
    },
}

fn default_dim() -> f32 {
    0.6
}

impl Effect {
    /// Schema-valid effects this composition does not draw.
    pub fn unrendered_name(&self) -> Option<&'static str> {
        match self {
            Effect::FadeIn { .. } => Some("fade-in"),
            Effect::FadeOut { .. } => Some("fade-out"),
            _ => None,
        }
    }
}

/// Inclusive 1-based line range.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct CodeRange {
    pub start: u32,
    pub end: u32,
}

impl CodeRange {
    pub fn contains(&self, line: u32) -> bool {
        (self.start..=self.end).contains(&line)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodePosition {
    Center,
    #[default]
    TopRight,
    BottomLeft,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CodeAnnotation {
    pub t: f32,
    pub dur: f32,
    pub code: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub highlight: Vec<CodeRange>,
    #[serde(default)]
    pub focus: Vec<CodeRange>,
    #[serde(default)]
    pub position: CodePosition,
}

fn default_language() -> String {
    "tsx".to_owned()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ShowcaseProps {
    pub layout: Layout,
    pub labels: Vec<String>,
    pub title: String,
    pub subtitle: String,
    pub preset: Preset,
    pub keys: Vec<Keystroke>,
    pub effects: Vec<Effect>,
    pub sections: Vec<Section>,
    /// Output size; omitted => 2560x1440 for `inspect`, 1920x1080 otherwise.
    pub width: u32,
    pub height: u32,
    pub speed_note: String,
    pub window_title: String,
    /// Playback multiplier applied once, to every clip. Overlay times (keys, sections,
    /// effects, codeAnnotations) are already output seconds.
    pub speed: f64,
    /// Omitted => `inspect` for side-by-side, `standard` otherwise.
    pub fidelity: Fidelity,
    pub object_fit: ObjectFit,
    /// Timed syntax-highlighted code overlays, relative to the first clip frame.
    pub code_annotations: Vec<CodeAnnotation>,
    /// Presentation of the title->content and content->outro transitions.
    pub transition_style: TransitionStyle,
}

impl Default for ShowcaseProps {
    fn default() -> Self {
        Self {
            layout: Layout::Single,
            labels: Vec::new(),
            title: "Demo".to_owned(),
            subtitle: String::new(),
            preset: Preset::Factory,
            keys: Vec::new(),
            effects: Vec::new(),
            sections: Vec::new(),
            width: 1920,
            height: 1080,
            speed_note: String::new(),
            window_title: String::new(),
            speed: 1.,
            fidelity: Fidelity::Standard,
            object_fit: ObjectFit::Contain,
            code_annotations: Vec::new(),
            transition_style: TransitionStyle::MotionBlur,
        }
    }
}

impl ShowcaseProps {
    /// Parses props JSON. `fidelity` (a profile name) overrides the one in the JSON; the
    /// fidelity and output size defaults depend on other fields, so they are resolved here.
    pub fn from_json(json: &str, fidelity: Option<&str>) -> Result<Self, String> {
        if json.trim().is_empty() {
            return Err("props JSON is empty".to_owned());
        }
        let mut value: Value = serde_json::from_str(json).map_err(|err| format!("props JSON is invalid: {err}"))?;
        let object = value.as_object_mut().ok_or("props JSON must be an object")?;
        resolve_defaults(object, fidelity);
        let props: Self = serde_path_to_error::deserialize(value).map_err(|err| format!("invalid props: {err}"))?;
        props.validate()?;
        Ok(props)
    }

    fn validate(&self) -> Result<(), String> {
        if !(self.speed.is_finite() && self.speed > 0.) {
            return Err(format!("invalid props: speed must be a positive finite number, got {}", self.speed));
        }
        if self.width == 0 || self.height == 0 || self.width % 2 == 1 || self.height % 2 == 1 {
            return Err(format!(
                "invalid props: width and height must be positive even numbers (yuv420p), got {}x{}",
                self.width, self.height
            ));
        }
        let ranges = self.code_annotations.iter().flat_map(|a| a.highlight.iter().chain(&a.focus));
        if ranges.clone().any(|range| range.start == 0 || range.end == 0) {
            return Err("invalid props: code line ranges are 1-based (start and end must be positive)".to_owned());
        }
        Ok(())
    }
}

fn resolve_defaults(props: &mut Map<String, Value>, fidelity: Option<&str>) {
    fn default(props: &mut Map<String, Value>, key: &str, value: impl Into<Value>) {
        if props.get(key).is_none_or(Value::is_null) {
            props.insert(key.to_owned(), value.into());
        }
    }
    if let Some(fidelity) = fidelity {
        props.insert("fidelity".to_owned(), Value::from(fidelity));
    }
    let side_by_side = props.get("layout").and_then(Value::as_str) == Some("side-by-side");
    default(props, "fidelity", if side_by_side { "inspect" } else { "standard" });
    let inspect = props.get("fidelity").and_then(Value::as_str) == Some("inspect");
    default(props, "width", if inspect { 2560 } else { 1920 });
    default(props, "height", if inspect { 1440 } else { 1080 });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> ShowcaseProps {
        ShowcaseProps::from_json(json, None).unwrap()
    }

    #[test]
    fn missing_fields_take_composition_defaults() {
        let props = parse(r#"{"clips":["ignored.mp4"],"unknown":1}"#);
        assert_eq!(props.preset, Preset::Factory);
        assert_eq!(props.fidelity, Fidelity::Standard);
        assert_eq!((props.width, props.height), (1920, 1080));
        assert_eq!(props.speed, 1.);
        assert_eq!(props.transition_style, TransitionStyle::MotionBlur);
    }

    #[test]
    fn fidelity_defaults_by_layout_and_sizes_by_fidelity() {
        let side = parse(r#"{"layout":"side-by-side"}"#);
        assert_eq!(side.fidelity, Fidelity::Inspect);
        assert_eq!((side.width, side.height), (2560, 1440));

        let explicit = parse(r#"{"layout":"side-by-side","fidelity":"compact","height":720}"#);
        assert_eq!(explicit.fidelity, Fidelity::Compact);
        assert_eq!((explicit.width, explicit.height), (1920, 720));

        let overridden = ShowcaseProps::from_json(r#"{"fidelity":"compact"}"#, Some("inspect")).unwrap();
        assert_eq!(overridden.fidelity, Fidelity::Inspect);
        assert_eq!(overridden.width, 2560);
    }

    #[test]
    fn effects_parse_by_fx_tag_with_percent_regions() {
        let props = parse(
            r#"{"effects":[
                {"fx":"zoom","t":1,"dur":2,"to":{"x":"10%","y":"20%","w":"50%","h":"40%"},"ease":"x"},
                {"fx":"spotlight","t":1,"dur":2,"on":{"x":"0%","y":"0%","w":"50%","h":"50%"}},
                {"fx":"fade-out","t":1,"dur":2}
            ]}"#,
        );
        let Effect::Zoom { to, .. } = &props.effects[0] else { panic!("zoom") };
        assert_eq!((to.x, to.w), (Pct(0.1), Pct(0.5)));
        let Effect::Spotlight { dim, .. } = &props.effects[1] else { panic!("spotlight") };
        assert_eq!(*dim, 0.6);
        assert_eq!(props.effects[2].unrendered_name(), Some("fade-out"));
    }

    #[test]
    fn code_annotations_take_schema_defaults() {
        let props = parse(r#"{"codeAnnotations":[{"t":0,"dur":1,"code":"x"}]}"#);
        let annotation = &props.code_annotations[0];
        assert_eq!(annotation.language, "tsx");
        assert_eq!(annotation.position, CodePosition::TopRight);
        assert!(annotation.highlight.is_empty() && annotation.focus.is_empty());
    }

    #[test]
    fn invalid_values_are_rejected() {
        for json in [
            "",
            "[]",
            r#"{"speed":0}"#,
            r#"{"speed":-2}"#,
            r#"{"speed":true}"#,
            r#"{"width":641}"#,
            r#"{"preset":"neon"}"#,
            r#"{"fidelity":"ultra"}"#,
            r#"{"effects":[{"fx":"zoom","t":0,"dur":1,"to":{"x":"left","y":"0%","w":"1%","h":"1%"}}]}"#,
            r#"{"codeAnnotations":[{"t":0,"dur":1,"code":"x","highlight":[{"start":0,"end":1}]}]}"#,
        ] {
            assert!(ShowcaseProps::from_json(json, None).is_err(), "{json} must be rejected");
        }
    }
}
