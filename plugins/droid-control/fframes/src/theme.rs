//! Palettes, window presets and fidelity treatments.

use crate::props::{Fidelity, Preset};

#[derive(Debug)]
pub struct Palette {
    pub bg: &'static str,
    pub surface: &'static str,
    pub accent: &'static str,
    pub border: &'static str,
    pub text: &'static str,
    pub muted: &'static str,
    /// The Factory palette carries a warm treatment throughout (glows, grade, shadows).
    pub warm: bool,
}

const FACTORY: Palette = Palette {
    bg: "#0a0804",
    surface: "#181818",
    accent: "#EE6018",
    border: "#342F2D",
    text: "#f0e8e0",
    muted: "#948781",
    warm: true,
};

const CATPPUCCIN: Palette = Palette {
    bg: "#0d1117",
    surface: "#181818",
    accent: "#89b4fa",
    border: "#313244",
    text: "#cdd6f4",
    muted: "#6c7086",
    warm: false,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bar {
    Colorful,
    Rings,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgStyle {
    Solid,
    Gradient,
}

#[derive(Debug)]
pub struct PresetConfig {
    pub bar: Bar,
    pub bar_side: BarSide,
    pub radius: f32,
    pub padding: f32,
    pub margin: f32,
    pub shadow: bool,
    pub bg_style: BgStyle,
}

const fn preset(radius: f32, padding: f32, margin: f32, shadow: bool, bg_style: BgStyle) -> PresetConfig {
    PresetConfig { bar: Bar::Colorful, bar_side: BarSide::Left, radius, padding, margin, shadow, bg_style }
}

const MACOS: PresetConfig = preset(12., 20., 60., true, BgStyle::Solid);
const MINIMAL: PresetConfig = PresetConfig { bar: Bar::None, ..preset(8., 16., 32., false, BgStyle::Solid) };
const HERO: PresetConfig = preset(16., 24., 80., true, BgStyle::Gradient);
const PRESENTATION: PresetConfig = preset(12., 24., 48., true, BgStyle::Solid);
const FACTORY_PRESET: PresetConfig = preset(12., 20., 80., true, BgStyle::Solid);
const FACTORY_HERO: PresetConfig = preset(12., 24., 80., true, BgStyle::Gradient);

impl Preset {
    pub fn palette(self) -> &'static Palette {
        match self {
            Preset::Factory | Preset::FactoryHero => &FACTORY,
            _ => &CATPPUCCIN,
        }
    }

    pub fn config(self) -> &'static PresetConfig {
        match self {
            Preset::Macos => &MACOS,
            Preset::Minimal => &MINIMAL,
            Preset::Hero => &HERO,
            Preset::Presentation => &PRESENTATION,
            Preset::Factory => &FACTORY_PRESET,
            Preset::FactoryHero => &FACTORY_HERO,
        }
    }

    pub fn is_factory(self) -> bool {
        matches!(self, Preset::Factory | Preset::FactoryHero)
    }
}

/// Film grain and colour grade strength: lighter where detail must stay readable.
pub struct Treatment {
    pub noise_opacity: f32,
    pub grade_intensity: f32,
}

impl Fidelity {
    pub fn treatment(self) -> Treatment {
        match self {
            Fidelity::Compact => Treatment { noise_opacity: 0.03, grade_intensity: 0.04 },
            Fidelity::Standard => Treatment { noise_opacity: 0.02, grade_intensity: 0.025 },
            Fidelity::Inspect => Treatment { noise_opacity: 0.008, grade_intensity: 0.012 },
        }
    }
}

/// x264 settings of the final encode and of `.cast` conversions, and agg's frame-rate cap.
pub struct Encoding {
    pub crf: u8,
    pub preset: &'static str,
    pub cast_fps_cap: u8,
}

impl Fidelity {
    pub fn encoding(self) -> Encoding {
        match self {
            Fidelity::Compact => Encoding { crf: 21, preset: "medium", cast_fps_cap: 24 },
            Fidelity::Standard => Encoding { crf: 18, preset: "slow", cast_fps_cap: 30 },
            Fidelity::Inspect => Encoding { crf: 14, preset: "slow", cast_fps_cap: 30 },
        }
    }
}
