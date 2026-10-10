//! The showcase timeline: title card, clip content, outro, joined by two transitions.

pub const FPS: usize = 30;
pub const TITLE_FRAMES: usize = 4 * FPS;
/// 3.5 s.
pub const OUTRO_FRAMES: usize = 7 * FPS / 2;
pub const TRANSITION_FRAMES: usize = 15;

/// Frames during which the clips actually play: the longest source clip (in source seconds)
/// at `speed`. A video without clips holds its empty content for 10 s.
pub fn content_frames(longest_clip: Option<f64>, speed: f64) -> usize {
    ((longest_clip.unwrap_or(10.) / speed) * FPS as f64).ceil() as usize
}

/// Frame ranges of the three segments. The content segment pads the clips with one
/// transition on each side: the title crossfade precedes playback, and the outro crossfade
/// begins after the frame-rounded playback interval and shows held frames. Both transitions
/// overlap that padding, so the total is simply title + clips + outro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timeline {
    pub content: usize,
}

impl Timeline {
    pub fn content_sequence(&self) -> usize {
        self.content + 2 * TRANSITION_FRAMES
    }

    pub fn content_start(&self) -> usize {
        TITLE_FRAMES - TRANSITION_FRAMES
    }

    /// The first clip frame: playback starts once the title crossfade is over.
    pub fn clips_start(&self) -> usize {
        TITLE_FRAMES
    }

    pub fn outro_start(&self) -> usize {
        TITLE_FRAMES + self.content
    }

    pub fn total(&self) -> usize {
        TITLE_FRAMES + self.content + OUTRO_FRAMES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_frames_are_the_longest_source_divided_by_speed() {
        assert_eq!(content_frames(Some(5.), 2.), 75);
    }

    #[test]
    fn content_frames_round_up_partial_frames() {
        assert_eq!(content_frames(Some(5.01), 3.), 51);
    }

    #[test]
    fn videos_without_clips_hold_10s_of_content() {
        assert_eq!(content_frames(None, 1.), 300);
    }

    #[test]
    fn content_sequence_pads_the_clips_by_one_transition_on_each_side() {
        let timeline = Timeline { content: content_frames(Some(5.), 2.) };
        assert_eq!(timeline.content_sequence(), 75 + 2 * TRANSITION_FRAMES);
    }

    #[test]
    fn content_shorter_than_a_transition_still_outlasts_both_transitions() {
        let timeline = Timeline { content: content_frames(Some(5.), 20.) };
        assert_eq!(timeline.content, 8);
        assert!(timeline.content_sequence() > 2 * TRANSITION_FRAMES);
        assert_eq!(timeline.total(), 120 + 8 + 105);
    }

    #[test]
    fn segments_overlap_by_one_transition_and_total_title_clips_outro() {
        let timeline = Timeline { content: content_frames(Some(5.), 2.) };
        let sequence_total = TITLE_FRAMES + timeline.content_sequence() + OUTRO_FRAMES - 2 * TRANSITION_FRAMES;
        assert_eq!(timeline.total(), sequence_total);
        assert_eq!(timeline.total(), 300);
        assert_eq!(timeline.content_start() + TRANSITION_FRAMES, timeline.clips_start());
        assert_eq!(timeline.content_start() + timeline.content_sequence() - TRANSITION_FRAMES, timeline.outro_start());
    }
}
