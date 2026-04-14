import React from 'react';
import { useCurrentFrame, useVideoConfig, interpolate, Easing } from 'remotion';

export const StaggeredPanelEntrance: React.FC<{
  delay?: number;
  children: React.ReactNode;
}> = ({ delay = 0, children }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const startFrame = delay * fps;
  const progress = interpolate(
    frame,
    [startFrame, startFrame + 0.5 * fps],
    [0, 1],
    {
      easing: Easing.bezier(0.16, 1, 0.3, 1),
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    }
  );

  const scale = interpolate(progress, [0, 1], [0.92, 1]);
  const opacity = interpolate(progress, [0, 1], [0, 1]);
  const translateY = interpolate(progress, [0, 1], [12, 0]);

  return (
    <div
      style={{
        transform: `scale(${scale}) translateY(${translateY}px)`,
        opacity,
      }}
    >
      {children}
    </div>
  );
};
