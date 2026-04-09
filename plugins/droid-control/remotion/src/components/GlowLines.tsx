import React from 'react';
import {
  AbsoluteFill,
  useCurrentFrame,
  useVideoConfig,
  interpolate,
} from 'remotion';

interface GlowEvent {
  t: number;
  y: string; // percentage within terminal content area
  dur?: number;
}

const DEFAULT_DURATION = 0.95; // seconds

/**
 * Renders a single glow bar for a content event.
 * Fades in over 0.15s, holds for 0.3s, fades out over 0.5s.
 */
const GlowBar: React.FC<{
  event: GlowEvent;
  color: string;
  fps: number;
  frame: number;
}> = ({ event, color, fps, frame }) => {
  const totalDur = event.dur ?? DEFAULT_DURATION;
  const startFrame = Math.round(event.t * fps);

  // Phase durations in frames
  const fadeInFrames = Math.round(0.15 * fps);
  const holdFrames = Math.round(0.3 * fps);
  const fadeOutFrames = Math.max(0, Math.round((totalDur - 0.15 - 0.3) * fps));
  const endFrame = startFrame + fadeInFrames + holdFrames + fadeOutFrames;

  // Outside active range → don't render
  if (frame < startFrame || frame > endFrame) {
    return null;
  }

  const fadeInEnd = startFrame + fadeInFrames;
  const holdEnd = fadeInEnd + holdFrames;

  let opacity: number;
  if (frame <= fadeInEnd) {
    // Fade in
    opacity = interpolate(frame, [startFrame, fadeInEnd], [0, 1], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
  } else if (frame <= holdEnd) {
    // Hold at full
    opacity = 1;
  } else {
    // Fade out
    opacity = interpolate(frame, [holdEnd, endFrame], [1, 0], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
  }

  return (
    <div
      style={{
        position: 'absolute',
        left: 0,
        right: 0,
        top: event.y,
        height: 40,
        transform: 'translateY(-50%)',
        background: `radial-gradient(ellipse at 50% 50%, ${color}26 0%, transparent 70%)`,
        opacity,
        pointerEvents: 'none',
      }}
    />
  );
};

export const GlowLines: React.FC<{
  events: GlowEvent[];
  color?: string;
}> = ({ events, color = '#EE6018' }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  return (
    <AbsoluteFill style={{ pointerEvents: 'none' }}>
      {events.map((event, i) => (
        <GlowBar key={i} event={event} color={color} fps={fps} frame={frame} />
      ))}
    </AbsoluteFill>
  );
};
