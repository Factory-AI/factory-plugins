import React from 'react';
import { useCurrentFrame, useVideoConfig, interpolate, Easing } from 'remotion';

interface ZoomRegion {
  x: string;
  y: string;
  w: string;
  h: string;
}

/** Parse a percentage string (e.g. "25%") to a 0–1 fraction. */
function pctToFraction(val: string): number {
  return parseFloat(val.replace('%', '')) / 100;
}

/** Cinematic ease-in-out for directed zoom motion. */
const ZOOM_EASING = Easing.bezier(0.25, 0.1, 0.25, 1);

/**
 * Directed zoom to a target region.
 *
 * Animates from the full view (scale=1) into a zoomed view where the
 * target region fills the frame, then back out.
 *
 * Phases (relative to duration):
 *   0–30%  IN   — scale up to target
 *   30–70% HOLD — maintain zoom
 *   70–100% OUT — scale back to 1
 */
export const ZoomEffect: React.FC<{
  startTime: number;
  duration: number;
  to: ZoomRegion;
  children: React.ReactNode;
}> = ({ startTime, duration, to, children }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const startFrame = startTime * fps;
  const endFrame = (startTime + duration) * fps;
  const totalFrames = endFrame - startFrame;

  // Phase boundaries (absolute frame numbers)
  const inEnd = startFrame + totalFrames * 0.3;
  const holdEnd = startFrame + totalFrames * 0.7;

  // Convert percentage-based region to 0–1 fractions
  const xFrac = pctToFraction(to.x);
  const yFrac = pctToFraction(to.y);
  const wFrac = pctToFraction(to.w);
  const hFrac = pctToFraction(to.h);

  const targetScale = Math.min(
    1 / Math.max(0.01, wFrac),
    1 / Math.max(0.01, hFrac)
  );

  // Anchor scaling at the center of the target region so the camera
  // "pushes in" toward that point.
  const originX = (xFrac + wFrac / 2) * 100;
  const originY = (yFrac + hFrac / 2) * 100;

  // Determine zoom progress (0 = full view, 1 = fully zoomed) based
  // on the current phase.
  let zoomProgress: number;
  if (frame < startFrame || frame > endFrame) {
    zoomProgress = 0;
  } else if (frame <= inEnd) {
    // IN — ease from 0 → 1
    zoomProgress = interpolate(frame, [startFrame, inEnd], [0, 1], {
      easing: ZOOM_EASING,
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
  } else if (frame <= holdEnd) {
    // HOLD
    zoomProgress = 1;
  } else {
    // OUT — ease from 1 → 0
    zoomProgress = interpolate(frame, [holdEnd, endFrame], [1, 0], {
      easing: ZOOM_EASING,
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    });
  }

  const currentScale = interpolate(zoomProgress, [0, 1], [1, targetScale]);

  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        overflow: 'hidden',
        transformOrigin: `${originX}% ${originY}%`,
        transform: `scale(${currentScale})`,
      }}
    >
      {children}
    </div>
  );
};

/**
 * Ambient "breathing" Ken Burns zoom.
 *
 * Wraps children in a slowly scaling container (1.0 → 1.04) over the
 * full composition duration, giving a subtle cinematic drift.
 */
export const BreathingZoom: React.FC<{
  children: React.ReactNode;
}> = ({ children }) => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();

  const scale = interpolate(frame, [0, durationInFrames], [1.0, 1.04], {
    easing: Easing.bezier(0.45, 0, 0.55, 1),
    extrapolateRight: 'clamp',
  });

  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        overflow: 'hidden',
        transform: `scale(${scale})`,
      }}
    >
      {children}
    </div>
  );
};
