import React from 'react';
import { AbsoluteFill, interpolate } from 'remotion';
import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from '@remotion/transitions';

export type MotionBlurProps = {
  /** Peak blur radius in pixels (default 6). */
  maxBlur?: number;
  /** Initial scale of the entering scene (default 1.03). */
  enterScale?: number;
};

/**
 * Custom presentation component for @remotion/transitions.
 *
 * Combines three effects during the crossfade:
 *   (a) CSS filter: blur() — 6 px at the edges, 0 px at the center
 *   (b) scale 1.03 → 1.0 on the entering scene (camera-dolly feel)
 *   (c) opacity crossfade
 */
const MotionBlurPresentation: React.FC<
  TransitionPresentationComponentProps<MotionBlurProps>
> = ({
  children,
  presentationDirection,
  presentationProgress,
  passedProps,
}) => {
  const maxBlur = passedProps.maxBlur ?? 6;
  const enterScale = passedProps.enterScale ?? 1.03;
  const isEntering = presentationDirection === 'entering';

  let style: React.CSSProperties;
  if (isEntering) {
    const blur = interpolate(presentationProgress, [0, 1], [maxBlur, 0], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
    const scale = interpolate(presentationProgress, [0, 1], [enterScale, 1], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
    style = {
      opacity: presentationProgress,
      filter: `blur(${blur}px)`,
      transform: `scale(${scale})`,
    };
  } else {
    const blur = interpolate(presentationProgress, [0, 1], [0, maxBlur], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
    style = {
      opacity: 1 - presentationProgress,
      filter: `blur(${blur}px)`,
    };
  }

  return <AbsoluteFill style={style}>{children}</AbsoluteFill>;
};

/**
 * Factory function returning a TransitionPresentation compatible with
 * `<TransitionSeries.Transition presentation={motionBlurTransition()} />`.
 */
export const motionBlurTransition = (
  props?: MotionBlurProps
): TransitionPresentation<MotionBlurProps> => {
  return {
    component: MotionBlurPresentation,
    props: props ?? {},
  };
};
