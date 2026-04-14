import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig, interpolate, Easing } from 'remotion';
import type { Palette } from '../lib/palettes';

export const BigDroidLogoOutro: React.FC<{
  palette: Palette;
}> = ({ palette }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Scale up from 0.8 to 1.0, fade in
  const progress = interpolate(frame, [0, 1.5 * fps], [0, 1], {
    easing: Easing.bezier(0.16, 1, 0.3, 1),
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  const scale = interpolate(progress, [0, 1], [0.8, 1.0]);
  const opacity = progress;

  const asciiLogo = `
  █████████    █████████     ████████    ███   █████████
  ███    ███   ███    ███   ███    ███   ███   ███    ███
  ███    ███   ███    ███   ███    ███   ███   ███    ███
  ███    ███   █████████    ███    ███   ███   ███    ███
  ███    ███   ███    ███   ███    ███   ███   ███    ███
  ███    ███   ███    ███   ███    ███   ███   ███    ███
  █████████    ███    ███    ████████    ███   █████████
  `.trim();

  return (
    <AbsoluteFill
      style={{
        backgroundColor: palette.bg,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      <div
        style={{
          transform: `scale(${scale})`,
          opacity,
          color: palette.accent,
          fontFamily: "'Geist Mono', 'SF Mono', 'Cascadia Code', 'Fira Code', monospace",
          fontSize: 24,
          lineHeight: 1.2,
          whiteSpace: 'pre',
          textAlign: 'left',
          textShadow: `0 0 20px ${palette.accent}66`,
        }}
      >
        {asciiLogo}
      </div>
      <div
        style={{
          marginTop: 40,
          opacity: interpolate(frame, [1.5 * fps, 2.5 * fps], [0, 1], { extrapolateRight: 'clamp' }),
          color: palette.text,
          fontSize: 32,
          fontWeight: 300,
          fontFamily: "'Geist', system-ui, sans-serif",
          letterSpacing: '0.2em',
        }}
      >
        AUTONOMOUS ENGINEERING
      </div>
    </AbsoluteFill>
  );
};
