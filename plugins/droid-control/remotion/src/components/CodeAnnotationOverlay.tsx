import React, { useMemo } from 'react';
import { useCurrentFrame, useVideoConfig, interpolate, Easing } from 'remotion';
import { Highlight, themes, type PrismTheme } from 'prism-react-renderer';
import type { CodeAnnotation, CodeRange } from '../lib/schema';
import type { Palette } from '../lib/palettes';
import type { PresetConfig } from '../lib/presets';

const MONO =
  "'Geist Mono', 'Berkeley Mono', 'SF Mono', 'Cascadia Code', 'Fira Code', monospace";

const inRange = (line: number, ranges: CodeRange[]): boolean =>
  ranges.some((r) => line >= r.start && line <= r.end);

const buildPrismTheme = (palette: Palette): PrismTheme => {
  const base = themes.vsDark;
  return {
    ...base,
    plain: {
      ...base.plain,
      color: palette.text,
      backgroundColor: 'transparent',
    },
  };
};

const positionToAnchor = (
  position: CodeAnnotation['position'],
  margin: number,
): React.CSSProperties => {
  const edge = Math.max(40, margin + 24);
  switch (position) {
    case 'center':
      return {
        left: '50%',
        top: '50%',
        transform: 'translate(-50%, -50%)',
      };
    case 'bottom-left':
      return { left: edge, bottom: edge };
    case 'top-right':
    default:
      return { right: edge, top: edge };
  }
};

const CodeCard: React.FC<{
  annotation: CodeAnnotation;
  palette: Palette;
  config: PresetConfig;
  enterFrame: number;
  exitFrame: number;
}> = ({ annotation, palette, config, enterFrame, exitFrame }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const lines = useMemo(
    () => annotation.code.replace(/\n$/, '').split('\n'),
    [annotation.code],
  );
  const prismTheme = useMemo(() => buildPrismTheme(palette), [palette]);

  if (frame < enterFrame || frame >= exitFrame) return null;

  const localFrame = frame - enterFrame;
  const totalFrames = exitFrame - enterFrame;

  const enterProgress = interpolate(localFrame, [0, 0.35 * fps], [0, 1], {
    easing: Easing.bezier(0.16, 1, 0.3, 1),
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  const exitStart = Math.max(0, totalFrames - 0.35 * fps);
  const exitProgress = interpolate(
    localFrame,
    [exitStart, totalFrames],
    [0, 1],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' },
  );

  const opacity = enterProgress * (1 - exitProgress);
  const scale = interpolate(enterProgress, [0, 1], [0.96, 1]);
  const blur = interpolate(enterProgress, [0, 1], [6, 0]);

  const highlightRanges = annotation.highlight ?? [];
  const focusRanges = annotation.focus ?? [];
  const hasFocus = focusRanges.length > 0;
  const anchor = positionToAnchor(
    annotation.position ?? 'top-right',
    config.margin,
  );
  const maxLine = lines.length;
  const gutterWidth = `${String(maxLine).length + 1}ch`;

  return (
    <div
      style={{
        position: 'absolute',
        ...anchor,
        opacity,
        transform: `${anchor.transform ?? ''} scale(${scale})`.trim(),
        filter: `blur(${blur}px)`,
        zIndex: 90,
        maxWidth: '42%',
        minWidth: 320,
        backgroundColor: `${palette.surface}F2`,
        border: `1px solid ${palette.border}`,
        borderBottom: `2px solid ${palette.accent}AA`,
        borderRadius: 12,
        padding: 16,
        boxShadow: `0 18px 48px rgba(0,0,0,0.45), 0 0 0 1px rgba(255,255,255,0.04)`,
        backdropFilter: 'blur(10px)',
        WebkitBackdropFilter: 'blur(10px)',
        fontFamily: MONO,
        fontSize: 14,
        lineHeight: 1.5,
        color: palette.text,
      }}
    >
      {annotation.title && (
        <div
          style={{
            color: palette.muted,
            fontSize: 11,
            textTransform: 'uppercase',
            letterSpacing: '0.1em',
            marginBottom: 10,
            fontFamily: "'Geist', system-ui, sans-serif",
            fontWeight: 500,
          }}
        >
          {annotation.title}
        </div>
      )}
      <Highlight
        code={annotation.code.replace(/\n$/, '')}
        language={annotation.language ?? 'tsx'}
        theme={prismTheme}
      >
        {({ tokens, getLineProps, getTokenProps }) => (
          <pre
            style={{
              margin: 0,
              padding: 0,
              backgroundColor: 'transparent',
              fontFamily: MONO,
              overflow: 'hidden',
            }}
          >
            {tokens.map((line, i) => {
              const lineNumber = i + 1;
              const isHighlighted = inRange(lineNumber, highlightRanges);
              const isFocused = !hasFocus || inRange(lineNumber, focusRanges);
              const lineOpacity = isFocused ? 1 : 0.3;
              const lineFilter = isFocused ? 'none' : 'blur(1.2px)';
              const bg = isHighlighted ? `${palette.accent}22` : 'transparent';
              const lineBorder = isHighlighted
                ? `2px solid ${palette.accent}`
                : `2px solid transparent`;
              const { key: _lineKey, ...lineRest } = getLineProps({
                line,
                key: i,
              });

              return (
                <div
                  key={`line-${i}`}
                  {...lineRest}
                  style={{
                    display: 'flex',
                    opacity: lineOpacity,
                    filter: lineFilter,
                    backgroundColor: bg,
                    borderLeft: lineBorder,
                    paddingLeft: 8,
                    marginLeft: -8,
                    marginRight: -8,
                    paddingRight: 8,
                  }}
                >
                  <span
                    style={{
                      width: gutterWidth,
                      flex: '0 0 auto',
                      color: palette.muted,
                      userSelect: 'none',
                      textAlign: 'right',
                      paddingRight: 12,
                    }}
                  >
                    {lineNumber}
                  </span>
                  <span style={{ flex: 1 }}>
                    {line.map((token, tokenIndex) => {
                      const { key: _tokenKey, ...tokenRest } = getTokenProps({
                        token,
                        key: tokenIndex,
                      });
                      return (
                        <span
                          key={`tok-${i}-${tokenIndex}`}
                          {...tokenRest}
                        />
                      );
                    })}
                  </span>
                </div>
              );
            })}
          </pre>
        )}
      </Highlight>
    </div>
  );
};

export const CodeAnnotationOverlay: React.FC<{
  annotations: CodeAnnotation[];
  palette: Palette;
  config: PresetConfig;
}> = ({ annotations, palette, config }) => {
  const { fps } = useVideoConfig();

  if (!annotations.length) return null;

  return (
    <>
      {annotations.map((annotation, i) => {
        const enterFrame = Math.round(annotation.t * fps);
        const exitFrame = Math.round((annotation.t + annotation.dur) * fps);
        return (
          <CodeCard
            key={`code-${i}-${annotation.t}`}
            annotation={annotation}
            palette={palette}
            config={config}
            enterFrame={enterFrame}
            exitFrame={exitFrame}
          />
        );
      })}
    </>
  );
};
