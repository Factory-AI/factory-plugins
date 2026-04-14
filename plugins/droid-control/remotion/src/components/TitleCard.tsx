import {
  AbsoluteFill,
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  Easing,
} from 'remotion';
import { fitText } from '@remotion/layout-utils';
import type { Palette } from '../lib/palettes';
import { RotorMark } from './RotorMark';
import { BreathingZoom } from './ZoomEffect';

const TITLE_MAX_FONT = 64;
const SUBTITLE_MAX_FONT = 24;
const TITLE_FONT_FAMILY = "'Geist', system-ui, sans-serif";

export const TitleCard: React.FC<{
  title: string;
  subtitle: string;
  palette: Palette;
  speedNote?: string;
}> = ({ title, subtitle, palette, speedNote }) => {
  const frame = useCurrentFrame();
  const { fps, width } = useVideoConfig();

  const titleFontSize = Math.min(
    TITLE_MAX_FONT,
    fitText({
      text: title,
      withinWidth: width * 0.8,
      fontFamily: TITLE_FONT_FAMILY,
      fontWeight: 700,
      letterSpacing: '-0.04em',
    }).fontSize,
  );

  const subtitleFontSize = Math.min(
    SUBTITLE_MAX_FONT,
    fitText({
      text: subtitle || ' ',
      withinWidth: width * 0.7,
      fontFamily: TITLE_FONT_FAMILY,
      fontWeight: 400,
    }).fontSize,
  );

  // Tagline fades in first (monospace label above title)
  const taglineProgress = interpolate(frame, [0, 0.4 * fps], [0, 1], {
    easing: Easing.bezier(0.16, 1, 0.3, 1),
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const taglineY = interpolate(taglineProgress, [0, 1], [15, 0]);
  const taglineOpacity = taglineProgress;

  // Title: scale 1.08→1.0 + translateY 30→0 + fade in
  const titleProgress = interpolate(frame, [0, 0.6 * fps], [0, 1], {
    easing: Easing.bezier(0.16, 1, 0.3, 1),
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const titleY = interpolate(titleProgress, [0, 1], [30, 0]);
  const titleScale = interpolate(titleProgress, [0, 1], [1.08, 1.0]);
  const titleOpacity = titleProgress;

  // Subtitle follows with a slight delay
  const subtitleProgress = interpolate(
    frame,
    [0.15 * fps, 0.75 * fps],
    [0, 1],
    {
      easing: Easing.bezier(0.16, 1, 0.3, 1),
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    }
  );
  const subtitleY = interpolate(subtitleProgress, [0, 1], [20, 0]);
  const subtitleOpacity = subtitleProgress;

  const isFactory = palette.accent === '#EE6018';

  // --- Factory preset: spinning rotor leads the accent line ---
  const SWEEP_WIDTH = 280;
  const ROTOR_SIZE = 28;
  const sweepStart = 0.15 * fps;
  const sweepEnd = 0.7 * fps;

  // Rotor horizontal progress: 0 → SWEEP_WIDTH
  const rotorProgress = interpolate(frame, [sweepStart, sweepEnd], [0, 1], {
    easing: Easing.bezier(0.16, 1, 0.3, 1),
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const rotorX = rotorProgress * SWEEP_WIDTH;

  // Continuous spin: ties to progress so it rolls naturally
  const rotorRotation = interpolate(rotorProgress, [0, 1], [0, 720]);

  // Rotor fades in at the start, fades out after reaching the end
  const rotorOpacity = interpolate(
    frame,
    [sweepStart, sweepStart + 0.08 * fps, sweepEnd, sweepEnd + 0.25 * fps],
    [0, 1, 1, 0],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }
  );

  // Trail line grows to follow the rotor
  const trailWidth = rotorX;
  const trailOpacity = interpolate(
    frame,
    [sweepStart, sweepStart + 0.1 * fps],
    [0, 1],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }
  );

  // --- Non-factory: original simple line animation ---
  const lineWidth = interpolate(frame, [0.2 * fps, 0.7 * fps], [0, 120], {
    easing: Easing.bezier(0.16, 1, 0.3, 1),
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const sweepPosition = interpolate(
    frame,
    [0.2 * fps, 0.2 * fps + 0.8 * fps],
    [-200, 100],
    {
      easing: Easing.bezier(0.16, 1, 0.3, 1),
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    }
  );

  return (
    <BreathingZoom>
      <AbsoluteFill
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          fontFamily: "'Geist', system-ui, sans-serif",
        }}
      >
      {/* Monospace tagline above title (shows speedNote if present) */}
      {speedNote && (
        <div
          style={{
            transform: `translateY(${taglineY}px)`,
            opacity: taglineOpacity,
            color: palette.muted,
            fontSize: 14,
            fontWeight: 400,
            letterSpacing: '0.08em',
            textTransform: 'uppercase',
            fontFamily:
              "'Geist Mono', 'SF Mono', 'Cascadia Code', 'Fira Code', monospace",
            marginBottom: 16,
          }}
        >
          {speedNote}
        </div>
      )}

      {/* Title with glow, scale entrance, and accent text-shadow */}
      <div
        style={{
          transform: `translateY(${titleY}px) scale(${titleScale})`,
          opacity: titleOpacity,
          color: palette.text,
          fontSize: titleFontSize,
          fontWeight: 700,
          letterSpacing: '-0.04em',
          textAlign: 'center',
          maxWidth: '80%',
          textShadow: `0 0 40px ${palette.accent}4d, 0 0 80px ${palette.accent}1a`,
        }}
      >
        {title}
      </div>

      {/* Accent line — factory: spinning rotor with trailing line; other: gradient sweep */}
      {isFactory ? (
        <div
          style={{
            position: 'relative',
            width: SWEEP_WIDTH,
            height: ROTOR_SIZE + 8,
            marginTop: 16,
            marginBottom: 16,
            display: 'flex',
            alignItems: 'center',
          }}
        >
          {/* Trailing line: grows from left edge toward the rotor */}
          <div
            style={{
              position: 'absolute',
              left: 0,
              top: '50%',
              width: trailWidth,
              height: 2,
              transform: 'translateY(-50%)',
              background: `linear-gradient(90deg, transparent 0%, ${palette.accent}66 30%, ${palette.accent} 100%)`,
              opacity: trailOpacity,
              borderRadius: 1,
            }}
          />

          {/* Spinning rotor at the leading edge */}
          <div
            style={{
              position: 'absolute',
              left: rotorX - ROTOR_SIZE / 2,
              top: '50%',
              marginTop: -ROTOR_SIZE / 2,
              width: ROTOR_SIZE,
              height: ROTOR_SIZE,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              transform: `rotate(${rotorRotation}deg)`,
              opacity: rotorOpacity,
              filter: `drop-shadow(0 0 6px rgba(255,255,255,0.3))`,
            }}
          >
            <RotorMark size={ROTOR_SIZE} color="white" />
          </div>
        </div>
      ) : (
        <div
          style={{
            width: lineWidth,
            height: 2,
            background: `linear-gradient(90deg, transparent, ${palette.accent}, white, ${palette.accent}, transparent)`,
            backgroundSize: '200% 100%',
            backgroundPosition: `${sweepPosition}% 0`,
            marginTop: 20,
            marginBottom: 20,
            borderRadius: 1,
          }}
        />
      )}

      <div
        style={{
          transform: `translateY(${subtitleY}px)`,
          opacity: subtitleOpacity,
          color: palette.muted,
          fontSize: subtitleFontSize,
          fontWeight: 400,
          textAlign: 'center',
          maxWidth: '70%',
        }}
      >
        {subtitle}
      </div>
    </AbsoluteFill>
    </BreathingZoom>
  );
};
