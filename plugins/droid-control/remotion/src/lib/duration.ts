import type { z } from 'zod';
import type { showcaseSchema } from '../compositions/Showcase';

const TITLE_DURATION_S = 4;
const TRANSITION_FRAMES = 15;

export function calculateShowcaseDuration(
  props: z.infer<typeof showcaseSchema>,
  fps = 30
): number {
  const titleFrames = TITLE_DURATION_S * fps;

  // Use the explicit clipDuration prop when provided; fall back to 60s for
  // clips (can't probe ahead of time) or 10s for clip-less title-only videos.
  const clipDurationS =
    props.clipDuration ?? (props.clips.length > 0 ? 60 : 10);
  const clipFrames = Math.ceil(clipDurationS * fps);

  const outroFrames = 3.5 * fps;

  return titleFrames + clipFrames + outroFrames - (2 * TRANSITION_FRAMES);
}
