import React, { useMemo } from 'react';
import { z } from 'zod';
import { AbsoluteFill, staticFile, useVideoConfig } from 'remotion';
import { Video } from '@remotion/media';
import { TransitionSeries, linearTiming } from '@remotion/transitions';
import { getTransitionPresentation } from '../components/ShowcaseTransition';
import {
  fidelitySchema,
  presetSchema,
  layoutSchema,
  keystrokeSchema,
  effectSchema,
  sectionSchema,
  codeAnnotationSchema,
  transitionStyleSchema,
} from '../lib/schema';
import { getPalette } from '../lib/palettes';
import { getPresetConfig } from '../lib/presets';
import { Background } from '../components/Background';
import { WindowChrome } from '../components/WindowChrome';
import { TitleCard } from '../components/TitleCard';
import { SectionHeaderOverlay } from '../components/SectionHeader';
import { KeystrokeOverlay } from '../components/KeystrokeOverlay';
import { SpotlightOverlay } from '../components/SpotlightOverlay';
import { StaggeredPanelEntrance } from '../components/StaggeredPanelEntrance';
import { NoiseOverlay } from '../components/NoiseOverlay';
import { FloatingParticles } from '../components/FloatingParticles';
import { ColorGradeOverlay } from '../components/ColorGradeOverlay';
import { Watermark } from '../components/Watermark';
import { ZoomEffect } from '../components/ZoomEffect';
import { SectionTransitionOverlay } from '../components/SectionTransition';
import { DroidOutro } from '../components/DroidOutro';
import { CodeAnnotationOverlay } from '../components/CodeAnnotationOverlay';

export const showcaseSchema = z.object({
  clips: z.array(z.string()),
  layout: layoutSchema,
  labels: z.array(z.string()),
  title: z.string(),
  subtitle: z.string(),
  preset: presetSchema,
  keys: z.array(keystrokeSchema),
  effects: z.array(effectSchema),
  sections: z.array(sectionSchema).optional(),
  width: z.number().optional(),
  height: z.number().optional(),
  speedNote: z.string().optional(),
  windowTitle: z.string().optional(),
  clipDuration: z.number().optional(),
  speed: z.number().positive().optional(),
  fidelity: fidelitySchema.optional(),
  // How clip video is sized inside its panel.
  // - "contain" (default): preserve aspect ratio, letterbox if needed. Safe default.
  // - "cover": fill the panel, crop overflow. Use when the clip aspect doesn't match the
  //   panel aspect (e.g. 16:9 landscape browser capture in a near-square side-by-side panel)
  //   and you'd rather crop edges than see giant black bars.
  // - "fill": stretch to fill the panel (distorts aspect). Rarely what you want.
  objectFit: z.enum(['contain', 'cover', 'fill']).optional(),
  // Timed syntax-highlighted code overlays rendered during the main content
  // sequence (above content, below noise/color grade). Timings are relative
  // to the start of the main content clip.
  codeAnnotations: z.array(codeAnnotationSchema).optional(),
  // Presentation used for title->content and content->outro transitions.
  // Defaults to 'motion-blur', which preserves existing aesthetic.
  transitionStyle: transitionStyleSchema.optional(),
});

const TITLE_DURATION_S = 4;
const TRANSITION_FRAMES = 15;

const resolveFidelity = (
  props: z.infer<typeof showcaseSchema>
): 'compact' | 'standard' | 'inspect' =>
  props.fidelity ?? (props.layout === 'side-by-side' ? 'inspect' : 'standard');

const visualTreatmentByFidelity = {
  compact: { noiseOpacity: 0.03, gradeIntensity: 0.04 },
  standard: { noiseOpacity: 0.02, gradeIntensity: 0.025 },
  inspect: { noiseOpacity: 0.008, gradeIntensity: 0.012 },
} as const;

const SingleLayout: React.FC<{
  clip: string;
  config: ReturnType<typeof getPresetConfig>;
  palette: ReturnType<typeof getPalette>;
  windowTitle?: string;
  objectFit: 'contain' | 'cover' | 'fill';
}> = ({ clip, config, palette, windowTitle, objectFit }) => {
  const { width, height } = useVideoConfig();

  const frameW = width - 2 * config.margin;
  const frameH = height - 2 * config.margin;

  return (
    <AbsoluteFill
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      <WindowChrome
        config={config}
        palette={palette}
        width={frameW}
        height={frameH}
        title={windowTitle}
      >
        <Video
          src={staticFile(clip)}
          objectFit={objectFit}
          style={{
            width: '100%',
            height: '100%',
            backgroundColor: palette.surface,
          }}
        />
      </WindowChrome>
    </AbsoluteFill>
  );
};

const SideBySideLayout: React.FC<{
  clips: string[];
  labels: string[];
  config: ReturnType<typeof getPresetConfig>;
  palette: ReturnType<typeof getPalette>;
  objectFit: 'contain' | 'cover' | 'fill';
}> = ({ clips, labels, config, palette, objectFit }) => {
  const { width, height } = useVideoConfig();

  const totalW = width - 2 * config.margin;
  const gap = 16;
  const panelW = Math.floor((totalW - gap) / 2);
  const panelH = height - 2 * config.margin;

  return (
    <AbsoluteFill
      style={{
        display: 'flex',
        flexDirection: 'row',
        alignItems: 'center',
        justifyContent: 'center',
        gap,
      }}
    >
      {clips.slice(0, 2).map((clip, i) => (
        <StaggeredPanelEntrance key={clip} delay={i * 0.2}>
          <div style={{ position: 'relative' }}>
            <WindowChrome
              config={config}
              palette={palette}
              width={panelW}
              height={panelH}
              animate={false}
              title={labels[i] ?? `Clip ${i + 1}`}
            >
              <Video
                src={staticFile(clip)}
                objectFit={objectFit}
                style={{
                  width: '100%',
                  height: '100%',
                  backgroundColor: palette.surface,
                }}
              />
            </WindowChrome>
          </div>
        </StaggeredPanelEntrance>
      ))}
    </AbsoluteFill>
  );
};

export const ShowcaseComposition: React.FC<z.infer<typeof showcaseSchema>> = (
  props
) => {
  const { fps } = useVideoConfig();
  const palette = getPalette(props.preset);
  const config = getPresetConfig(props.preset);
  const isFactory =
    props.preset === 'factory' || props.preset === 'factory-hero';
  const fidelity = resolveFidelity(props);
  const visualTreatment = visualTreatmentByFidelity[fidelity];

  const titleFrames = TITLE_DURATION_S * fps;
  const clipFrames = Math.ceil((props.clipDuration ?? 60) * fps);
  const objectFit = props.objectFit ?? 'contain';
  const transition = useMemo(
    () =>
      getTransitionPresentation(
        props.transitionStyle ?? 'motion-blur',
        palette
      ),
    [props.transitionStyle, palette]
  );

  const spotlights = useMemo(
    () =>
      props.effects.filter(
        (e): e is Extract<typeof e, { fx: 'spotlight' }> => e.fx === 'spotlight'
      ),
    [props.effects]
  );

  const zooms = useMemo(
    () =>
      props.effects.filter(
        (e): e is Extract<typeof e, { fx: 'zoom' }> => e.fx === 'zoom'
      ),
    [props.effects]
  );

  return (
    <AbsoluteFill>
      <Background palette={palette} config={config} />
      <FloatingParticles color={palette.accent} />

      <TransitionSeries>
        {/* Title card */}
        <TransitionSeries.Sequence durationInFrames={titleFrames}>
          <TitleCard
            title={props.title}
            subtitle={props.subtitle}
            palette={palette}
            speedNote={props.speedNote}
          />
        </TransitionSeries.Sequence>

        {/* Crossfade from title to content */}
        <TransitionSeries.Transition
          presentation={transition}
          timing={linearTiming({ durationInFrames: TRANSITION_FRAMES })}
        />

        {/* Main content */}
        <TransitionSeries.Sequence durationInFrames={clipFrames}>
          <AbsoluteFill>
            <Background palette={palette} config={config} />

            {(() => {
              let content = (
                <>
                  {props.layout === 'side-by-side' ? (
                    <SideBySideLayout
                      clips={props.clips}
                      labels={props.labels}
                      config={config}
                      palette={palette}
                      objectFit={objectFit}
                    />
                  ) : props.clips[0] ? (
                    <SingleLayout
                      clip={props.clips[0]}
                      config={config}
                      palette={palette}
                      windowTitle={props.windowTitle}
                      objectFit={objectFit}
                    />
                  ) : null}
                </>
              );

              // Apply zooms by wrapping content
              zooms.forEach((zoom, i) => {
                content = (
                  <ZoomEffect
                    key={`zoom-${i}`}
                    startTime={zoom.t}
                    duration={zoom.dur}
                    to={zoom.to}
                  >
                    {content}
                  </ZoomEffect>
                );
              });

              return content;
            })()}

            {/* Spotlight overlays */}
            {spotlights.map((spot, i) => (
              <SpotlightOverlay
                key={`spot-${i}`}
                startTime={spot.t}
                duration={spot.dur}
                region={spot.on}
                dim={spot.dim}
              />
            ))}

            {/* Frosted sweep at section boundaries */}
            {props.sections && props.sections.length > 1 && (
              <SectionTransitionOverlay sections={props.sections} />
            )}

            {/* Section Headers */}
            {props.sections && props.sections.length > 0 && (
              <SectionHeaderOverlay
                sections={props.sections}
                palette={palette}
                config={config}
              />
            )}

            {/* Keystroke overlay */}
            {props.keys.length > 0 && (
              <KeystrokeOverlay
                keys={props.keys}
                palette={palette}
                config={config}
              />
            )}

            {/* Code annotations: timed syntax-highlighted overlays */}
            {props.codeAnnotations && props.codeAnnotations.length > 0 && (
              <CodeAnnotationOverlay
                annotations={props.codeAnnotations}
                palette={palette}
                config={config}
              />
            )}
          </AbsoluteFill>
        </TransitionSeries.Sequence>

        {/* Crossfade to outro */}
        <TransitionSeries.Transition
          presentation={transition}
          timing={linearTiming({ durationInFrames: TRANSITION_FRAMES })}
        />

        {/* Outro card */}
        <TransitionSeries.Sequence durationInFrames={3.5 * fps}>
          <DroidOutro
            palette={palette}
          />
        </TransitionSeries.Sequence>
      </TransitionSeries>

      {/* Watermark: factory presets only, above content, below noise/grade */}
      {isFactory && <Watermark />}

      {/* Noise overlay: above content, below ColorGradeOverlay */}
      <NoiseOverlay opacity={visualTreatment.noiseOpacity} />

      {/* Colour grade: topmost layer — unifies colour temperature */}
      <ColorGradeOverlay
        palette={palette}
        intensity={visualTreatment.gradeIntensity}
      />
    </AbsoluteFill>
  );
};
