import React from 'react';
import { AbsoluteFill } from 'remotion';

export const NoiseOverlay: React.FC<{
  opacity?: number;
}> = ({ opacity = 0.03 }) => {
  return (
    <AbsoluteFill
      style={{
        mixBlendMode: 'overlay',
        opacity,
        pointerEvents: 'none',
      }}
    >
      <svg
        width="100%"
        height="100%"
        xmlns="http://www.w3.org/2000/svg"
        style={{ display: 'block' }}
      >
        <filter id="noise-filter">
          <feTurbulence
            type="fractalNoise"
            baseFrequency="0.65"
            numOctaves={3}
            stitchTiles="stitch"
          />
        </filter>
        <rect width="100%" height="100%" filter="url(#noise-filter)" />
      </svg>
    </AbsoluteFill>
  );
};
