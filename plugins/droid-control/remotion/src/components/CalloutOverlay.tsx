import { useCurrentFrame, useVideoConfig, interpolate, Easing } from 'remotion';
import type { Effect } from '../lib/schema';
import type { Palette } from '../lib/palettes';

type Callout = Extract<Effect, { fx: 'callout' }>;

const CalloutPill: React.FC<{
  callout: Callout;
  palette: Palette;
  enterFrame: number;
  exitFrame: number;
}> = ({ callout, palette, enterFrame, exitFrame }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const isVisible = frame >= enterFrame && frame < exitFrame;
  if (!isVisible) return null;

  const localFrame = frame - enterFrame;

  // Pop-in animation over 0.25s (same overshoot bezier as KeystrokePill)
  const enterProgress = interpolate(localFrame, [0, 0.25 * fps], [0, 1], {
    easing: Easing.bezier(0.34, 1.56, 0.64, 1),
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  // Fade out over last 0.2s
  const totalFrames = exitFrame - enterFrame;
  const fadeOutStart = totalFrames - 0.2 * fps;
  const exitOpacity = interpolate(
    localFrame,
    [fadeOutStart, totalFrames],
    [1, 0],
    {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    }
  );

  const scale = interpolate(enterProgress, [0, 1], [0.85, 1]);

  return (
    <div
      style={{
        position: 'absolute',
        left: callout.at.x,
        top: callout.at.y,
        transform: `translate(-50%, -50%) scale(${scale})`,
        opacity: Math.min(enterProgress, exitOpacity),
        backgroundColor: `${palette.surface}E6`,
        color: palette.text,
        fontSize: 24,
        fontFamily: "'Geist', 'Inter', sans-serif",
        fontWeight: 500,
        lineHeight: 1.4,
        padding: '12px 24px',
        borderRadius: 12,
        border: `1px solid ${palette.border}`,
        borderLeft: `3px solid ${palette.accent}`,
        backdropFilter: 'blur(12px)',
        boxShadow: '0 8px 32px rgba(0, 0, 0, 0.35)',
        maxWidth: '40%',
        textAlign: 'center',
        zIndex: 90,
        pointerEvents: 'none',
      }}
    >
      {callout.text}
    </div>
  );
};

export const CalloutOverlay: React.FC<{
  callouts: Callout[];
  palette: Palette;
}> = ({ callouts, palette }) => {
  const { fps } = useVideoConfig();

  return (
    <>
      {callouts.map((callout) => (
        <CalloutPill
          key={`${callout.t}-${callout.text}`}
          callout={callout}
          palette={palette}
          enterFrame={Math.round(callout.t * fps)}
          exitFrame={Math.round((callout.t + callout.dur) * fps)}
        />
      ))}
    </>
  );
};
