//! The droid-control showcase composition, rendered with fframes.
//!
//! A showcase is a title card, one or two screen recordings in window chrome with timed
//! overlays (keystrokes, sections, zooms, spotlights, callouts, code), and the Droid outro.
//! [`Showcase`] draws any frame as SVG; [`render`] rasterises stills and whole videos.

pub mod clips;
pub mod code;
pub mod fonts;
pub mod motion;
pub mod props;
pub mod render;
pub mod stage;
pub mod svg;
pub mod theme;
pub mod timing;

mod content;
mod outro;
mod overlays;
mod scenery;
mod showcase;
mod title;
mod transition;

pub use showcase::Showcase;
