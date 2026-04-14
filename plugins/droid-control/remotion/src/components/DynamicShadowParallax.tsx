import { useCurrentFrame, useVideoConfig, interpolate } from 'remotion';

/**
 * Provides subtle shadow-offset parallax that shifts over the video duration,
 * creating a subliminal sense of depth as if the light source drifts with a
 * slow Ken Burns camera move.
 *
 * offsetX: 0 → 2 px  (horizontal drift)
 * offsetY: 8 → 10 px (vertical drift deepens)
 */
export function useShadowParallax(): { offsetX: number; offsetY: number } {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();

  const offsetX = interpolate(frame, [0, durationInFrames], [0, 2], {
    extrapolateRight: 'clamp',
  });
  const offsetY = interpolate(frame, [0, durationInFrames], [8, 10], {
    extrapolateRight: 'clamp',
  });

  return { offsetX, offsetY };
}
