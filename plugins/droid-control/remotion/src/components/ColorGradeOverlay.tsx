import React from 'react';
import { AbsoluteFill } from 'remotion';
import type { Palette } from '../lib/palettes';

/**
 * Global colour-grade post-processing layer — analogous to a DaVinci Resolve
 * colour grade applied over the final composite.
 *
 * - Factory (warm): warm amber tint via `mix-blend-mode: color`
 * - Catppuccin (cool): cool blue tint at 75 % of the given intensity
 * - A nested radial vignette darkens the corners for a cinematic frame.
 *
 * Place this as the **last** child in the z-stack so it sits above all
 * content, including any noise overlays.
 */
export const ColorGradeOverlay: React.FC<{
  palette: Palette;
  intensity?: number;
}> = ({ palette, intensity = 0.04 }) => {
  const isWarm = palette.accent === '#EE6018';

  const gradeColor = isWarm
    ? `rgba(200, 120, 40, ${intensity})`
    : `rgba(80, 100, 200, ${intensity * 0.75})`;

  return (
    <AbsoluteFill style={{ pointerEvents: 'none' }}>
      {/* Colour tint — shifts the entire frame's temperature */}
      <AbsoluteFill
        style={{
          backgroundColor: gradeColor,
          mixBlendMode: 'color',
        }}
      />

      {/* Radial vignette — subtle darkening at the corners */}
      <AbsoluteFill
        style={{
          background:
            'radial-gradient(ellipse at 50% 50%, transparent 40%, rgba(0, 0, 0, 0.15) 100%)',
        }}
      />
    </AbsoluteFill>
  );
};
