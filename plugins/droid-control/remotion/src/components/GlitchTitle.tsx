import React, { useMemo } from 'react';
import {
  AbsoluteFill,
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  Easing,
} from 'remotion';
import type { Palette } from '../lib/palettes';

const BLOCK_SIZE = 12;
const GRID_WIDTH = 800;
const GRID_HEIGHT = 120;

interface PixelBlock {
  /** Grid-assembled position (top-left corner) */
  x: number;
  y: number;
  /** Scattered offset from assembled position */
  scatterDx: number;
  scatterDy: number;
  /** true = accent color, false = text color */
  isAccent: boolean;
}

/**
 * Deterministic "garbled" subtitle: replace ~30% of characters with random ASCII,
 * seeded by character index.
 */
function garbleText(text: string): string {
  const chars: string[] = [];
  for (let i = 0; i < text.length; i++) {
    const shouldGarble = (i * 17 + 3) % 10 < 3; // ~30%
    if (shouldGarble && text[i] !== ' ') {
      const code = 33 + ((i * 17) % 94);
      chars.push(String.fromCharCode(code));
    } else {
      chars.push(text[i]);
    }
  }
  return chars.join('');
}

/**
 * Build a deterministic grid of pixel blocks that represent the title area.
 * Each block gets a fixed scatter target computed from its index.
 */
function buildBlockGrid(): PixelBlock[] {
  const cols = Math.floor(GRID_WIDTH / BLOCK_SIZE);
  const rows = Math.floor(GRID_HEIGHT / BLOCK_SIZE);
  const blocks: PixelBlock[] = [];

  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const i = row * cols + col;
      blocks.push({
        x: col * BLOCK_SIZE,
        y: row * BLOCK_SIZE,
        scatterDx: Math.sin(i * 7.3 + 3.1) * 200,
        scatterDy: Math.cos(i * 5.7 + 1.4) * 150,
        isAccent: (i * 13 + 7) % 5 === 0, // ~20% accent
      });
    }
  }

  return blocks;
}

export const GlitchTitle: React.FC<{
  title: string;
  subtitle?: string;
  palette: Palette;
}> = ({ title, subtitle, palette }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const blocks = useMemo(buildBlockGrid, []);
  const garbled = useMemo(
    () => (subtitle ? garbleText(subtitle) : undefined),
    [subtitle]
  );

  // Phase boundaries in frames
  const scatterEnd = Math.round(0.5 * fps); // ~15 frames at 30fps
  const assembleEnd = Math.round(1.5 * fps); // ~45 frames at 30fps

  // --- Garbled subtitle fade-in (during scatter phase) ---
  const garbledOpacity = interpolate(frame, [0, scatterEnd], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  // --- Title text fade-in (after assembly completes) ---
  const titleOpacity = interpolate(
    frame,
    [assembleEnd - Math.round(0.2 * fps), assembleEnd],
    [0, 1],
    {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    }
  );

  return (
    <AbsoluteFill
      style={{
        backgroundColor: palette.bg,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        fontFamily: 'monospace',
        overflow: 'hidden',
      }}
    >
      {/* Garbled subtitle line above the pixel grid */}
      {garbled && (
        <div
          style={{
            opacity: garbledOpacity,
            color: palette.muted,
            fontSize: 16,
            letterSpacing: '0.05em',
            marginBottom: 24,
            whiteSpace: 'pre',
          }}
        >
          {garbled}
        </div>
      )}

      {/* Pixel block grid container */}
      <div
        style={{
          position: 'relative',
          width: GRID_WIDTH,
          height: GRID_HEIGHT,
        }}
      >
        {/* Rendered pixel blocks */}
        {/* Hoisted outside .map() — same value for every block */}
        {(() => {
          const scatterOpacity = interpolate(frame, [0, scatterEnd], [0, 1], {
            extrapolateLeft: 'clamp',
            extrapolateRight: 'clamp',
          });
          const assembleProgress = interpolate(
            frame,
            [scatterEnd, assembleEnd],
            [0, 1],
            {
              easing: Easing.bezier(0.16, 1, 0.3, 1),
              extrapolateLeft: 'clamp',
              extrapolateRight: 'clamp',
            }
          );
          return blocks.map((block, i) => {
            // Phase (c): subtle jitter when assembled
            const jitterX =
              frame >= assembleEnd ? Math.sin(frame * 0.1 + i) * 1.5 : 0;
            const jitterY =
              frame >= assembleEnd ? Math.cos(frame * 0.13 + i * 0.7) * 1.5 : 0;

            // Interpolate from scatter position to grid position
            const dx = interpolate(
              assembleProgress,
              [0, 1],
              [block.scatterDx, 0]
            );
            const dy = interpolate(
              assembleProgress,
              [0, 1],
              [block.scatterDy, 0]
            );

            const opacity = frame < scatterEnd ? scatterOpacity : 1;
            const x = block.x + dx + jitterX;
            const y = block.y + dy + jitterY;

            return (
              <div
                key={i}
                style={{
                  position: 'absolute',
                  left: x,
                  top: y,
                  width: BLOCK_SIZE,
                  height: BLOCK_SIZE,
                  backgroundColor: block.isAccent
                    ? palette.accent
                    : palette.text,
                  opacity,
                }}
              />
            );
          });
        })()}

        {/* Title text rendered on top of assembled blocks */}
        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: palette.bg,
            fontSize: 72,
            fontWeight: 700,
            letterSpacing: '-0.04em',
            opacity: titleOpacity,
            // Title text punches through the block layer via mix-blend
            mixBlendMode: 'difference',
            whiteSpace: 'nowrap',
          }}
        >
          {title}
        </div>
      </div>

      {/* Scanline overlay */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          background:
            'repeating-linear-gradient(0deg, transparent 0px, transparent 2px, rgba(0,0,0,0.08) 2px, rgba(0,0,0,0.08) 4px)',
          pointerEvents: 'none',
        }}
      />
    </AbsoluteFill>
  );
};
