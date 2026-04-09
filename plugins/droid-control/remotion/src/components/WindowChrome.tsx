import { useCurrentFrame, useVideoConfig, interpolate, Easing } from 'remotion';
import type { Palette } from '../lib/palettes';
import type { PresetConfig } from '../lib/presets';

const TrafficLight: React.FC<{
  color: string;
  cx: number;
}> = ({ color, cx }) => <circle cx={cx} cy="50%" r="6" fill={color} />;

const TrafficLightRing: React.FC<{
  color: string;
  cx: number;
}> = ({ color, cx }) => (
  <circle cx={cx} cy="50%" r="6" fill="none" stroke={color} strokeWidth="1.5" />
);

const WindowBar: React.FC<{
  config: PresetConfig;
  palette: Palette;
  width: number;
  title?: string;
}> = ({ config, palette, width, title }) => {
  if (config.bar === 'none') return null;

  const Dot = config.bar === 'rings' ? TrafficLightRing : TrafficLight;
  const isRight = config.barSide === 'right';

  const dots = (
    <svg width={80} height={36}>
      <Dot color="#ff5f57" cx={isRight ? 62 : 14} />
      <Dot color="#febc2e" cx={isRight ? 42 : 34} />
      <Dot color="#28c840" cx={isRight ? 22 : 54} />
    </svg>
  );

  return (
    <div
      style={{
        width,
        height: 36,
        backgroundColor: palette.surface,
        display: 'flex',
        alignItems: 'center',
        justifyContent: isRight ? 'flex-end' : 'flex-start',
        paddingLeft: isRight ? 0 : 8,
        paddingRight: isRight ? 8 : 0,
        borderTopLeftRadius: config.radius,
        borderTopRightRadius: config.radius,
        position: 'relative',
      }}
    >
      {dots}
      {title && (
        <span
          style={{
            position: 'absolute',
            left: '50%',
            transform: 'translateX(-50%)',
            color: palette.muted,
            fontSize: 13,
            fontFamily: 'system-ui, sans-serif',
          }}
        >
          {title}
        </span>
      )}
    </div>
  );
};

export const WindowChrome: React.FC<{
  config: PresetConfig;
  palette: Palette;
  width: number;
  height: number;
  title?: string;
  /** Set to false when an ancestor already handles the entrance animation. */
  animate?: boolean;
  children: React.ReactNode;
}> = ({ config, palette, width, height, title, animate = true, children }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Animate the window scaling in over 0.5s with a crisp ease-out.
  // Skipped when `animate` is false (e.g. StaggeredPanelEntrance handles it).
  const enterProgress = animate
    ? interpolate(frame, [0, 0.5 * fps], [0, 1], {
        easing: Easing.bezier(0.16, 1, 0.3, 1),
        extrapolateLeft: 'clamp',
        extrapolateRight: 'clamp',
      })
    : 1;

  const scale = interpolate(enterProgress, [0, 1], [0.92, 1]);
  const opacity = enterProgress;

  const barHeight = config.bar !== 'none' ? 36 : 0;
  const contentHeight = height - barHeight;

  return (
    <div
      style={{
        width,
        height,
        transform: `scale(${scale})`,
        opacity,
        borderRadius: config.radius,
        overflow: 'hidden',
        position: 'relative',
        ...(config.shadow && {
          boxShadow:
            palette.accent === '#EE6018'
              ? // Warm brown-tinted shadow with faint accent glow
                `0 8px 60px rgba(30, 12, 4, 0.6), 0 2px 20px rgba(60, 24, 8, 0.3), 0 0 120px rgba(238, 96, 24, 0.07)`
              : // Cool neutral shadow for non-Factory palettes
                `0 8px 40px rgba(0, 0, 0, 0.45), 0 2px 12px rgba(0, 0, 0, 0.3)`,
        }),
        // Subtle border for glass effect
        border: `1px solid rgba(255, 255, 255, 0.06)`,
      }}
    >
      <WindowBar
        config={config}
        palette={palette}
        width={width}
        title={title}
      />
      <div
        style={{
          width,
          height: contentHeight,
          boxSizing: 'border-box',
          padding: config.padding,
          backgroundColor: palette.surface,
          overflow: 'hidden',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        {children}
      </div>
    </div>
  );
};
