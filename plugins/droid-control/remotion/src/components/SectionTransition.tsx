import React from 'react';
import {
  AbsoluteFill,
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  Easing,
} from 'remotion';
import type { Section } from '../lib/schema';

const SWEEP_DURATION_S = 0.5;
const BLUR_PX = 28;

const FrostedSweep: React.FC<{
  triggerFrame: number;
}> = ({ triggerFrame }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const sweepFrames = Math.round(SWEEP_DURATION_S * fps);
  const endFrame = triggerFrame + sweepFrames;

  if (frame < triggerFrame || frame > endFrame) return null;

  const localFrame = frame - triggerFrame;

  // Band sweeps from left (-10%) to right (110%)
  const bandX = interpolate(localFrame, [0, sweepFrames], [-10, 110], {
    easing: Easing.bezier(0.4, 0, 0.2, 1),
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  // Opacity: quick fade in, hold, quick fade out
  const opacity = interpolate(
    localFrame,
    [0, sweepFrames * 0.15, sweepFrames * 0.85, sweepFrames],
    [0, 1, 1, 0],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' },
  );

  return (
    <AbsoluteFill style={{ pointerEvents: 'none', overflow: 'hidden' }}>
      <div
        style={{
          position: 'absolute',
          top: 0,
          bottom: 0,
          left: `${bandX}%`,
          width: '20%',
          transform: 'translateX(-50%)',
          backdropFilter: `blur(${BLUR_PX}px)`,
          WebkitBackdropFilter: `blur(${BLUR_PX}px)`,
          background:
            'linear-gradient(90deg, transparent, rgba(255,255,255,0.04), transparent)',
          opacity,
        }}
      />
    </AbsoluteFill>
  );
};

export const SectionTransitionOverlay: React.FC<{
  sections: Section[];
}> = ({ sections }) => {
  const { fps } = useVideoConfig();

  // Skip the first section -- it marks the start, not a transition point
  return (
    <>
      {sections.slice(1).map((sec) => {
        const triggerFrame = Math.round(sec.t * fps);
        return (
          <FrostedSweep key={`sweep-${sec.t}`} triggerFrame={triggerFrame} />
        );
      })}
    </>
  );
};
