import type { z } from 'zod';
import type { showcaseSchema } from '../compositions/Showcase';

type ShowcaseProps = z.infer<typeof showcaseSchema>;
type TimingProps = Pick<ShowcaseProps, 'clipDuration' | 'clips' | 'speed'>;

export const TITLE_DURATION_S = 4;
export const OUTRO_DURATION_S = 3.5;
export const TRANSITION_FRAMES = 15;

export const playbackSpeed = (props: Pick<ShowcaseProps, 'speed'>): number =>
  props.speed ?? 1;

// Frames during which the clips actually play: the longest source clip
// (clipDuration, in source seconds) at `speed`. Without a probed clipDuration,
// clips fall back to 60s and clip-less title-only videos to 10s.
export function contentFrames(props: TimingProps, fps: number): number {
  const sourceSeconds =
    props.clipDuration ?? (props.clips.length > 0 ? 60 : 10);
  return Math.ceil((sourceSeconds / playbackSpeed(props)) * fps);
}

// The content sequence pads the clips with one transition length on each
// side: the title crossfade precedes playback; the outro crossfade begins
// after the frame-rounded playback interval and shows held frames.
// The padding also keeps the sequence longer than
// its two transitions for any positive clip length, which TransitionSeries
// requires.
export function contentSequenceFrames(props: TimingProps, fps: number): number {
  return contentFrames(props, fps) + 2 * TRANSITION_FRAMES;
}

// Both transitions overlap the content padding, so the total is simply
// title + clips + outro.
export function calculateShowcaseDuration(props: TimingProps, fps = 30): number {
  return (
    TITLE_DURATION_S * fps + contentFrames(props, fps) + OUTRO_DURATION_S * fps
  );
}
