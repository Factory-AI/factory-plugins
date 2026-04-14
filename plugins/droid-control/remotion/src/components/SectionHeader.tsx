import React from 'react';
import { useCurrentFrame, useVideoConfig, interpolate, Easing } from 'remotion';
import type { Section } from '../lib/schema';
import type { Palette } from '../lib/palettes';
import type { PresetConfig } from '../lib/presets';

const SectionTitle: React.FC<{
  title: string;
  palette: Palette;
  config: PresetConfig;
  enterFrame: number;
  exitFrame: number;
}> = ({ title, palette, config, enterFrame, exitFrame }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const isVisible = frame >= enterFrame && frame < exitFrame;
  if (!isVisible) return null;

  const localFrame = frame - enterFrame;

  // Slide down & fade in
  const enterProgress = interpolate(localFrame, [0, 0.4 * fps], [0, 1], {
    easing: Easing.bezier(0.22, 1, 0.36, 1),
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  const totalFrames = exitFrame - enterFrame;
  const fadeOutStart = totalFrames - 0.3 * fps;
  const exitProgress = interpolate(
    localFrame,
    [fadeOutStart, totalFrames],
    [0, 1],
    {
      easing: Easing.bezier(0.32, 0, 0.67, 0),
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    }
  );

  const opacity =
    interpolate(enterProgress, [0, 1], [0, 1]) *
    interpolate(exitProgress, [0, 1], [1, 0]);
  const translateY =
    interpolate(enterProgress, [0, 1], [-20, 0]) -
    interpolate(exitProgress, [0, 1], [0, -20]);

  return (
    <div
      style={{
        position: 'absolute',
        top: Math.max(30, config.margin / 2 - 20),
        left: '50%',
        transform: `translate(-50%, ${translateY}px)`,
        opacity,
        backgroundColor: `${palette.surface}E6`,
        color: palette.text,
        fontSize: 28,
        fontFamily: "'Geist', 'Inter', sans-serif",
        fontWeight: 600,
        padding: '12px 32px',
        borderRadius: 16,
        border: `1px solid ${palette.border}`,
        borderBottom: `2px solid ${palette.accent}80`,
        backdropFilter: 'blur(12px)',
        boxShadow: `0 8px 32px rgba(0, 0, 0, 0.4)`,
        zIndex: 100,
        whiteSpace: 'nowrap',
        letterSpacing: '-0.02em',
      }}
    >
      {title}
    </div>
  );
};

export const SectionHeaderOverlay: React.FC<{
  sections: Section[];
  palette: Palette;
  config: PresetConfig;
}> = ({ sections, palette, config }) => {
  const { fps, durationInFrames } = useVideoConfig();

  return (
    <>
      {sections.map((sec, i) => {
        const enterFrame = Math.round(sec.t * fps);
        const nextStart =
          i + 1 < sections.length
            ? Math.round(sections[i + 1].t * fps)
            : durationInFrames;
        const exitFrame = Math.min(nextStart, durationInFrames);

        return (
          <SectionTitle
            key={`${sec.t}-${sec.title}`}
            title={sec.title}
            palette={palette}
            config={config}
            enterFrame={enterFrame}
            exitFrame={exitFrame}
          />
        );
      })}
    </>
  );
};
