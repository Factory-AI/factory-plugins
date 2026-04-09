import React, { useMemo } from 'react';
import { AbsoluteFill, useCurrentFrame } from 'remotion';

interface ParticleSeed {
  baseX: number;
  baseY: number;
  offsetX: number;
  offsetY: number;
  size: number;
}

export const FloatingParticles: React.FC<{
  count?: number;
  color?: string;
  opacity?: number;
}> = ({ count = 30, color = '#EE6018', opacity = 0.07 }) => {
  const frame = useCurrentFrame();

  // Deterministic particle seeds — stable across frames, varies only by count
  const seeds = useMemo<ParticleSeed[]>(
    () =>
      Array.from({ length: count }, (_, i) => ({
        baseX: (i * 73 + 17) % 100,
        baseY: (i * 47 + 31) % 100,
        offsetX: (i * 13 + 7) * 0.1,
        offsetY: (i * 19 + 11) * 0.1,
        size: 2 + (i % 3) * 2, // 2, 4, or 6px
      })),
    [count]
  );

  return (
    <AbsoluteFill style={{ overflow: 'hidden', pointerEvents: 'none' }}>
      {seeds.map((p, i) => {
        // Slow Lissajous paths driven by frame count
        const dx = Math.sin(frame * 0.008 + p.offsetX) * 60;
        const dy = Math.cos(frame * 0.006 + p.offsetY) * 40;

        return (
          <div
            key={i}
            style={{
              position: 'absolute',
              left: `${p.baseX}%`,
              top: `${p.baseY}%`,
              width: p.size,
              height: p.size,
              borderRadius: '50%',
              backgroundColor: color,
              opacity,
              transform: `translate(${dx}px, ${dy}px)`,
            }}
          />
        );
      })}
    </AbsoluteFill>
  );
};
