import { useCurrentFrame, useVideoConfig, interpolate, Easing } from 'remotion';
import type { Keystroke } from '../lib/schema';
import type { Palette } from '../lib/palettes';
import type { PresetConfig } from '../lib/presets';

const KeystrokePill: React.FC<{
  label: string;
  palette: Palette;
  config: PresetConfig;
  enterFrame: number;
  exitFrame: number;
}> = ({ label, palette, config, enterFrame, exitFrame }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const isVisible = frame >= enterFrame && frame < exitFrame;
  if (!isVisible) return null;

  const localFrame = frame - enterFrame;

  // Pop-in animation over 0.2s
  const enterProgress = interpolate(localFrame, [0, 0.2 * fps], [0, 1], {
    easing: Easing.bezier(0.34, 1.56, 0.64, 1),
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  // Fade out over last 0.15s
  const totalFrames = exitFrame - enterFrame;
  const fadeOutStart = totalFrames - 0.15 * fps;
  const exitOpacity = interpolate(
    localFrame,
    [fadeOutStart, totalFrames],
    [1, 0],
    {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    }
  );

  const scale = interpolate(enterProgress, [0, 1], [0.8, 1]);
  const enterOpacity = enterProgress;

  return (
    <div
      style={{
        position: 'absolute',
        bottom: Math.max(50, config.margin + 40),
        left: '50%',
        transform: `translateX(-50%) scale(${scale})`,
        opacity: Math.min(enterOpacity, exitOpacity),
        backgroundColor: `${palette.surface}BF`,
        color: palette.text,
        fontSize: 22,
        fontFamily: "'Geist Mono', 'Berkeley Mono', monospace",
        fontWeight: 500,
        padding: '8px 20px',
        borderRadius: 8,
        border: `1px solid ${palette.border}`,
        backdropFilter: 'blur(8px)',
        whiteSpace: 'nowrap',
      }}
    >
      {label}
    </div>
  );
};

export const KeystrokeOverlay: React.FC<{
  keys: Keystroke[];
  palette: Palette;
  config: PresetConfig;
}> = ({ keys, palette, config }) => {
  const { fps } = useVideoConfig();
  const defaultDur = 1.2;

  return (
    <>
      {keys.map((key, i) => {
        const enterFrame = Math.round(key.t * fps);
        const naturalExit = Math.round((key.t + (key.dur ?? defaultDur)) * fps);
        const nextStart =
          i + 1 < keys.length ? Math.round(keys[i + 1].t * fps) : Infinity;
        const exitFrame = Math.min(naturalExit, nextStart);

        return (
          <KeystrokePill
            key={`${key.t}-${key.label}`}
            label={key.label}
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
