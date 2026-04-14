import {
  AbsoluteFill,
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  staticFile,
} from 'remotion';
import type { Palette } from '../lib/palettes';
import type { PresetConfig } from '../lib/presets';

/**
 * Full-bleed background layer.
 *
 * For warm (Factory) palettes: a radial vignette (dark center, warm
 * amber-brown edges) with 4 asymmetric corner glow blobs whose opacity
 * intensifies over the video duration — from a mild warm wash (~0.05) to
 * a rich reddish-amber bloom (~0.2).
 *
 * For cool palettes (Catppuccin, etc.): a cool-toned radial or flat fill.
 */
export const Background: React.FC<{
  palette: Palette;
  config: PresetConfig;
  totalFrames?: number;
}> = ({ palette, config, totalFrames }) => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const total = totalFrames ?? durationInFrames;

  const isWarm = palette.accent === '#EE6018';

  if (isWarm) {
    // Progressive warmth: 0 → 1 over full duration
    const warmth = interpolate(frame, [0, total], [0, 1], {
      extrapolateRight: 'clamp',
    });

    // Glow opacity ramps from subtle to prominent
    const glowBase = interpolate(warmth, [0, 1], [0.05, 0.2]);

    return (
      <AbsoluteFill
        style={{
          background: `radial-gradient(ellipse at 50% 50%, ${palette.bg} 0%, #1a0e08 55%, #2a1510 85%, #351a12 100%)`,
        }}
      >
        {/* Bottom-left: strongest warm amber glow */}
        <div
          style={{
            position: 'absolute',
            bottom: '-10%',
            left: '-10%',
            width: '60%',
            height: '60%',
            background: `radial-gradient(ellipse at center, rgba(200, 80, 20, ${glowBase}) 0%, transparent 70%)`,
            pointerEvents: 'none',
          }}
        />

        {/* Top-right: slightly cooler amber */}
        <div
          style={{
            position: 'absolute',
            top: '-15%',
            right: '-10%',
            width: '55%',
            height: '55%',
            background: `radial-gradient(ellipse at center, rgba(180, 60, 15, ${glowBase * 0.8}) 0%, transparent 70%)`,
            pointerEvents: 'none',
          }}
        />

        {/* Bottom-right: deep red accent */}
        <div
          style={{
            position: 'absolute',
            bottom: '-5%',
            right: '-5%',
            width: '45%',
            height: '50%',
            background: `radial-gradient(ellipse at center, rgba(160, 40, 10, ${glowBase * 0.6}) 0%, transparent 65%)`,
            pointerEvents: 'none',
          }}
        />

        {/* Top-left: faint Factory-orange kiss */}
        <div
          style={{
            position: 'absolute',
            top: '-8%',
            left: '-5%',
            width: '40%',
            height: '40%',
            background: `radial-gradient(ellipse at center, rgba(238, 96, 24, ${glowBase * 0.4}) 0%, transparent 60%)`,
            pointerEvents: 'none',
          }}
        />

        {/* Halftone texture overlay */}
        <AbsoluteFill
          style={{
            backgroundImage: `url(${staticFile('bg-halftone-rotor.jpg')})`,
            backgroundSize: 'cover',
            backgroundPosition: 'center',
            opacity: 0.15,
            mixBlendMode: 'screen',
            pointerEvents: 'none',
          }}
        />
      </AbsoluteFill>
    );
  }

  // Cool palettes — radial gradient or flat fill
  if (config.bgStyle === 'gradient') {
    return (
      <AbsoluteFill
        style={{
          background: `radial-gradient(ellipse at 50% 50%, ${palette.bg} 0%, #0f0f1a 50%, #121220 80%, #1a1028 100%)`,
        }}
      />
    );
  }

  return <AbsoluteFill style={{ backgroundColor: palette.bg }} />;
};
