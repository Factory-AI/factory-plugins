import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';

const TARGET_HZ = 1.5;

export const ActiveInputPulse: React.FC<{
  y?: string;
  color?: string;
  active?: boolean;
}> = ({ y = '50%', color = '#EE6018', active = true }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  if (!active) {
    return null;
  }

  // Oscillate opacity between 0.08 and 0.22 at ~1.5 Hz
  const omega = (2 * Math.PI * TARGET_HZ) / fps;
  const sinValue = Math.sin(frame * omega);
  const opacity = 0.15 + sinValue * 0.07; // range: 0.08 .. 0.22

  return (
    <AbsoluteFill style={{ pointerEvents: 'none' }}>
      <div
        style={{
          position: 'absolute',
          left: 0,
          right: 0,
          top: y,
          height: 3,
          transform: 'translateY(-50%)',
          backgroundColor: color,
          opacity,
          boxShadow: `0 0 12px ${color}26`,
        }}
      />
    </AbsoluteFill>
  );
};
