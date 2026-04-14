import { useCurrentFrame, useVideoConfig, interpolate, Easing } from 'remotion';

interface Region {
  x: string;
  y: string;
  w: string;
  h: string;
}

function pctToNum(val: string): number {
  return parseFloat(val.replace('%', '')) / 100;
}

export const SpotlightOverlay: React.FC<{
  startTime: number;
  duration: number;
  region: Region;
  dim?: number;
}> = ({ startTime, duration, region, dim = 0.6 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const startFrame = startTime * fps;
  const endFrame = (startTime + duration) * fps;

  if (frame < startFrame || frame > endFrame) return null;

  const localFrame = frame - startFrame;
  const totalFrames = endFrame - startFrame;

  // Fade in over 0.3s, fade out over 0.3s
  const fadeIn = interpolate(localFrame, [0, 0.3 * fps], [0, 1], {
    easing: Easing.bezier(0.16, 1, 0.3, 1),
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const fadeOut = interpolate(
    localFrame,
    [totalFrames - 0.3 * fps, totalFrames],
    [1, 0],
    {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
    }
  );
  const opacity = Math.min(fadeIn, fadeOut);

  const rx = pctToNum(region.x);
  const ry = pctToNum(region.y);
  const rw = pctToNum(region.w);
  const rh = pctToNum(region.h);

  // SVG mask: white everywhere except a black (transparent) cutout for the spotlight region.
  // Use only the standard `maskImage` property (Chrome uses it in headless rendering).
  const svgMask = `url("data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns='http://www.w3.org/2000/svg' width='100%25' height='100%25'>` +
      `<rect width='100%25' height='100%25' fill='white'/>` +
      `<rect x='${rx * 100}%25' y='${ry * 100}%25' width='${rw * 100}%25' height='${rh * 100}%25' rx='8' fill='black'/>` +
      `</svg>`
  )}")`;

  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        opacity,
        background: `rgba(0, 0, 0, ${dim})`,
        WebkitMaskImage: svgMask,
        WebkitMaskSize: '100% 100%',
        maskImage: svgMask,
        maskSize: '100% 100%',
        pointerEvents: 'none',
      }}
    />
  );
};
