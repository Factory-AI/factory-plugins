import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig, interpolate, Easing } from 'remotion';
import type { Palette } from '../lib/palettes';
import { RotorMark } from './RotorMark';

export const FanningRotorOutro: React.FC<{
  palette: Palette;
}> = ({ palette }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Phase 1: Fan out the 8 triangles (0s to 1.5s)
  const fanDuration = 1.5 * fps;
  // Phase 2: Crossfade to ASCII Droid (1.5s to 2.5s)
  const fadeDuration = 1.0 * fps;

  // 8 overlapping wedges that form the RotorMark logo
  const wedges = Array.from({ length: 8 }).map((_, i) => {
    // 1. Scale up as a single stacked blade (frames 0 to 12)
    const scaleProgress = interpolate(frame, [0, 12], [0, 1], {
      easing: Easing.out(Easing.back(1.5)),
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });

    // 2. Fan out from the single blade (frames 12 to fanDuration)
    const rotationProgress = interpolate(frame, [12, fanDuration], [0, i * 45], {
      easing: Easing.out(Easing.cubic),
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });

    // Glitch effect: flash opacity slightly during the fanning phase
    const isGlitching = frame > 12 && frame < 24 && frame % 3 === 0;
    const baseOpacity = interpolate(frame, [0, 12], [0, 1], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
    const opacity = isGlitching ? 0.5 : baseOpacity;

    return (
      <div
        key={i}
        style={{
          position: 'absolute',
          top: '50%',
          left: '50%',
          width: '613px',
          height: '650px',
          marginLeft: '-306.5px',
          marginTop: '-325px',
          transformOrigin: '50% 50%',
          transform: `scale(${scaleProgress}) rotate(${rotationProgress}deg)`,
          opacity,
          mixBlendMode: 'screen',
          // A 45-degree pie slice capturing exactly one blade (top-right)
          // 50% 50% = center
          // 100% 50% = straight right
          // 100% 2.85% = exactly 45 degrees up (y = 18.5px out of 650px)
          clipPath: 'polygon(50% 50%, 100% 50%, 100% 2.85%)',
        }}
      >
        <RotorMark width={613} height={650} color="white" />
      </div>
    );
  });

  // Crossfade between the rotor triangles and the ASCII logo
  const wedgesOpacity = interpolate(frame, [fanDuration, fanDuration + fadeDuration], [1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  const asciiOpacity = interpolate(frame, [fanDuration, fanDuration + fadeDuration], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

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
      {/* The fanning rotor layer */}
      <div style={{ position: 'absolute', inset: 0, opacity: wedgesOpacity }}>
        {wedges}
      </div>

      {/* The ASCII Droid layer */}
      <div
        style={{
          position: 'absolute',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          opacity: asciiOpacity,
        }}
      >
        <div
          style={{
            color: 'white', // ASCII DROID in white
            fontFamily: "'Geist Mono', 'SF Mono', 'Cascadia Code', 'Fira Code', monospace",
            fontSize: 24,
            lineHeight: 1.2,
            whiteSpace: 'pre',
            textAlign: 'left',
            textShadow: '0 0 20px rgba(255,255,255,0.4)',
          }}
        >
          {asciiLogo}
        </div>
        <div
          style={{
            marginTop: 40,
            color: palette.accent, // AUTONOMOUS ENGINEERING in orange
            fontSize: 32,
            fontWeight: 300,
            fontFamily: "'Geist', system-ui, sans-serif",
            letterSpacing: '0.2em',
            textShadow: `0 0 15px ${palette.accent}66`,
          }}
        >
          AUTONOMOUS ENGINEERING
        </div>
      </div>
    </AbsoluteFill>
  );
};
