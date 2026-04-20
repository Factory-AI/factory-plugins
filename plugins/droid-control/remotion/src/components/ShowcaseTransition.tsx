// Transition style names (`whip-pan`, `light-leak`, `flash`, `glitch-lite`) and
// the "selectable enum of named transitions" API shape are inspired by
// @hyperframes/shader-transitions (Apache-2.0). Implementations below are
// original Remotion-native CSS/SVG overlays — no GLSL/WebGL involved.
// https://github.com/heygen-com/hyperframes/tree/main/packages/shader-transitions

import React from 'react';
import { AbsoluteFill, interpolate, Easing, random } from 'remotion';
import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from '@remotion/transitions';
import type { TransitionStyle } from '../lib/schema';
import type { Palette } from '../lib/palettes';

type BaseProps = {
  palette: Palette;
};

const clamp01 = (v: number): number => Math.min(1, Math.max(0, v));

// ---------------------------------------------------------------------------
// Motion blur: blur() + scale + opacity crossfade. (Existing aesthetic.)
// ---------------------------------------------------------------------------
const MotionBlurPresentation: React.FC<
  TransitionPresentationComponentProps<BaseProps>
> = ({ children, presentationDirection, presentationProgress }) => {
  const maxBlur = 6;
  const enterScale = 1.03;
  const isEntering = presentationDirection === 'entering';

  if (isEntering) {
    const blur = interpolate(presentationProgress, [0, 1], [maxBlur, 0], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
    const scale = interpolate(presentationProgress, [0, 1], [enterScale, 1], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
    return (
      <AbsoluteFill
        style={{
          opacity: presentationProgress,
          filter: `blur(${blur}px)`,
          transform: `scale(${scale})`,
        }}
      >
        {children}
      </AbsoluteFill>
    );
  }

  const blur = interpolate(presentationProgress, [0, 1], [0, maxBlur], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  return (
    <AbsoluteFill
      style={{
        opacity: 1 - presentationProgress,
        filter: `blur(${blur}px)`,
      }}
    >
      {children}
    </AbsoluteFill>
  );
};

// ---------------------------------------------------------------------------
// Flash: hard cut emphasised by a warm/white flash overlay at the midpoint.
// ---------------------------------------------------------------------------
const FlashPresentation: React.FC<
  TransitionPresentationComponentProps<BaseProps>
> = ({ children, presentationDirection, presentationProgress, passedProps }) => {
  const isEntering = presentationDirection === 'entering';
  const palette = passedProps.palette;
  const isWarm = palette.accent === '#EE6018';

  const sceneOpacity = isEntering
    ? clamp01((presentationProgress - 0.45) / 0.35)
    : clamp01(1 - (presentationProgress - 0.2) / 0.35);

  // Flash is rendered only on the entering side so the peak is single-source.
  const flashOpacity = isEntering
    ? interpolate(
        presentationProgress,
        [0, 0.35, 0.55, 0.85, 1],
        [0, 0.25, 1, 0.2, 0],
        { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' },
      )
    : 0;

  const flashColor = isWarm
    ? `rgba(255, 220, 180, ${flashOpacity})`
    : `rgba(235, 240, 255, ${flashOpacity})`;

  return (
    <AbsoluteFill>
      <AbsoluteFill style={{ opacity: sceneOpacity }}>{children}</AbsoluteFill>
      {flashOpacity > 0 && (
        <AbsoluteFill
          style={{
            backgroundColor: flashColor,
            mixBlendMode: 'screen',
            pointerEvents: 'none',
          }}
        />
      )}
    </AbsoluteFill>
  );
};

// ---------------------------------------------------------------------------
// Whip-pan: horizontal pan + motion blur for energetic scene changes.
// ---------------------------------------------------------------------------
const WhipPanPresentation: React.FC<
  TransitionPresentationComponentProps<BaseProps>
> = ({ children, presentationDirection, presentationProgress }) => {
  const isEntering = presentationDirection === 'entering';
  const easing = Easing.bezier(0.6, 0, 0.1, 1);
  const eased = easing(presentationProgress);
  const maxBlur = 14;

  if (isEntering) {
    const translateX = interpolate(eased, [0, 1], [30, 0]);
    const blur = interpolate(presentationProgress, [0, 0.6, 1], [maxBlur, 4, 0], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
    const opacity = interpolate(presentationProgress, [0, 0.2, 1], [0, 1, 1], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
    return (
      <AbsoluteFill
        style={{
          transform: `translateX(${translateX}%)`,
          filter: `blur(${blur}px)`,
          opacity,
        }}
      >
        {children}
      </AbsoluteFill>
    );
  }

  const translateX = interpolate(eased, [0, 1], [0, -30]);
  const blur = interpolate(presentationProgress, [0, 0.4, 1], [0, 10, maxBlur], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const opacity = interpolate(presentationProgress, [0, 0.8, 1], [1, 1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  return (
    <AbsoluteFill
      style={{
        transform: `translateX(${translateX}%)`,
        filter: `blur(${blur}px)`,
        opacity,
      }}
    >
      {children}
    </AbsoluteFill>
  );
};

// ---------------------------------------------------------------------------
// Light leak: warm moving gradient sweeps across while scenes crossfade.
// ---------------------------------------------------------------------------
const LightLeakPresentation: React.FC<
  TransitionPresentationComponentProps<BaseProps>
> = ({ children, presentationDirection, presentationProgress, passedProps }) => {
  const isEntering = presentationDirection === 'entering';
  const palette = passedProps.palette;
  const isWarm = palette.accent === '#EE6018';

  const sceneOpacity = isEntering
    ? interpolate(presentationProgress, [0, 0.5, 1], [0, 0.4, 1], {
        extrapolateLeft: 'clamp',
        extrapolateRight: 'clamp',
      })
    : interpolate(presentationProgress, [0, 0.5, 1], [1, 0.6, 0], {
        extrapolateLeft: 'clamp',
        extrapolateRight: 'clamp',
      });

  const sweepOpacity = isEntering
    ? interpolate(
        presentationProgress,
        [0, 0.3, 0.6, 1],
        [0, 1, 0.6, 0],
        { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' },
      )
    : 0;

  const sweepPosition = interpolate(presentationProgress, [0, 1], [-40, 140]);

  const leakColor = isWarm ? '238, 96, 24' : '137, 180, 250';

  return (
    <AbsoluteFill>
      <AbsoluteFill style={{ opacity: sceneOpacity }}>{children}</AbsoluteFill>
      {sweepOpacity > 0 && (
        <AbsoluteFill
          style={{
            pointerEvents: 'none',
            mixBlendMode: 'screen',
            opacity: sweepOpacity,
            background: `linear-gradient(105deg,
              rgba(${leakColor}, 0) 0%,
              rgba(${leakColor}, 0) ${sweepPosition - 30}%,
              rgba(${leakColor}, 0.45) ${sweepPosition}%,
              rgba(255, 240, 200, 0.35) ${sweepPosition + 12}%,
              rgba(${leakColor}, 0) ${sweepPosition + 40}%,
              rgba(${leakColor}, 0) 100%)`,
          }}
        />
      )}
    </AbsoluteFill>
  );
};

// ---------------------------------------------------------------------------
// Glitch-lite: brief RGB offset + clipped horizontal bands near midpoint.
// ---------------------------------------------------------------------------
const GlitchLitePresentation: React.FC<
  TransitionPresentationComponentProps<BaseProps>
> = ({ children, presentationDirection, presentationProgress }) => {
  const isEntering = presentationDirection === 'entering';

  const sceneOpacity = isEntering
    ? interpolate(presentationProgress, [0, 0.4, 1], [0, 1, 1], {
        extrapolateLeft: 'clamp',
        extrapolateRight: 'clamp',
      })
    : interpolate(presentationProgress, [0, 0.6, 1], [1, 1, 0], {
        extrapolateLeft: 'clamp',
        extrapolateRight: 'clamp',
      });

  const glitchIntensity = interpolate(
    presentationProgress,
    [0, 0.35, 0.55, 0.75, 1],
    [0, 1, 1, 0.3, 0],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' },
  );

  const seedBase = isEntering ? 1000 : 2000;
  const jitterX = (random(`glitch-x-${Math.round(presentationProgress * 60) + seedBase}`) - 0.5) * 16 * glitchIntensity;
  const rgbSplit = 6 * glitchIntensity;

  const bandTop = 20 + 40 * random(`glitch-band-top-${Math.round(presentationProgress * 30) + seedBase}`);
  const bandHeight = 6 + 14 * random(`glitch-band-h-${Math.round(presentationProgress * 30) + seedBase}`);
  const showBand = glitchIntensity > 0.6;

  return (
    <AbsoluteFill style={{ opacity: sceneOpacity }}>
      {/* Red ghost */}
      <AbsoluteFill
        style={{
          transform: `translate(${jitterX - rgbSplit}px, 0)`,
          mixBlendMode: 'screen',
          opacity: rgbSplit > 0 ? 0.45 : 0,
        }}
      >
        <AbsoluteFill
          style={{
            filter:
              'sepia(1) saturate(6) hue-rotate(-50deg) contrast(1.1)',
          }}
        >
          {children}
        </AbsoluteFill>
      </AbsoluteFill>
      {/* Blue ghost */}
      <AbsoluteFill
        style={{
          transform: `translate(${jitterX + rgbSplit}px, 0)`,
          mixBlendMode: 'screen',
          opacity: rgbSplit > 0 ? 0.45 : 0,
        }}
      >
        <AbsoluteFill
          style={{
            filter:
              'sepia(1) saturate(6) hue-rotate(180deg) contrast(1.1)',
          }}
        >
          {children}
        </AbsoluteFill>
      </AbsoluteFill>
      {/* Main scene */}
      <AbsoluteFill style={{ transform: `translate(${jitterX}px, 0)` }}>
        {children}
      </AbsoluteFill>
      {/* Displaced band */}
      {showBand && (
        <AbsoluteFill
          style={{
            clipPath: `inset(${bandTop}% 0 ${100 - bandTop - bandHeight}% 0)`,
            transform: `translateX(${jitterX * 2.2}px)`,
            opacity: 0.85,
          }}
        >
          {children}
        </AbsoluteFill>
      )}
    </AbsoluteFill>
  );
};

const presentationMap: Record<
  TransitionStyle,
  React.FC<TransitionPresentationComponentProps<BaseProps>>
> = {
  'motion-blur': MotionBlurPresentation,
  flash: FlashPresentation,
  'whip-pan': WhipPanPresentation,
  'light-leak': LightLeakPresentation,
  'glitch-lite': GlitchLitePresentation,
};

export const getTransitionPresentation = (
  style: TransitionStyle,
  palette: Palette,
): TransitionPresentation<BaseProps> => {
  const component = presentationMap[style];
  return {
    component,
    props: { palette },
  };
};
