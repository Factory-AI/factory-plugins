import React from 'react';
import {
  AbsoluteFill,
  useCurrentFrame,
  useVideoConfig,
  interpolate,
} from 'remotion';
import { RotorMark } from './RotorMark';

export const Watermark: React.FC<{
  opacity?: number;
  size?: number;
}> = ({ opacity = 0.2, size = 48 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const fadeIn = interpolate(frame, [0, fps], [0, opacity], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  return (
    <AbsoluteFill style={{ pointerEvents: 'none' }}>
      <div
        style={{
          position: 'absolute',
          bottom: 24,
          right: 32,
          opacity: fadeIn,
        }}
      >
        <RotorMark size={size} color="white" />
      </div>
    </AbsoluteFill>
  );
};
