import { z } from 'zod';

export const presetSchema = z.enum([
  'macos',
  'minimal',
  'hero',
  'presentation',
  'factory',
  'factory-hero',
]);
export type Preset = z.infer<typeof presetSchema>;

export const layoutSchema = z.enum(['single', 'side-by-side']);
export type Layout = z.infer<typeof layoutSchema>;

export const fidelitySchema = z.enum(['compact', 'standard', 'inspect']);
export type Fidelity = z.infer<typeof fidelitySchema>;

export const keystrokeSchema = z.object({
  t: z.number(),
  label: z.string(),
  dur: z.number().optional(),
});
export type Keystroke = z.infer<typeof keystrokeSchema>;

export const sectionSchema = z.object({
  t: z.number(),
  title: z.string(),
});
export type Section = z.infer<typeof sectionSchema>;

export const effectSchema = z.discriminatedUnion('fx', [
  z.object({
    fx: z.literal('fade-in'),
    t: z.number(),
    dur: z.number(),
  }),
  z.object({
    fx: z.literal('fade-out'),
    t: z.number(),
    dur: z.number(),
  }),
  z.object({
    fx: z.literal('zoom'),
    t: z.number(),
    dur: z.number(),
    to: z.object({
      x: z.string(),
      y: z.string(),
      w: z.string(),
      h: z.string(),
    }),
    ease: z.string().optional(),
  }),
  z.object({
    fx: z.literal('spotlight'),
    t: z.number(),
    dur: z.number(),
    on: z.object({
      x: z.string(),
      y: z.string(),
      w: z.string(),
      h: z.string(),
    }),
    dim: z.number().optional(),
  }),
  z.object({
    fx: z.literal('callout'),
    t: z.number(),
    dur: z.number(),
    text: z.string(),
    at: z.object({ x: z.string(), y: z.string() }),
  }),
]);
export type Effect = z.infer<typeof effectSchema>;

export const codeRangeSchema = z.object({
  start: z.number().int().positive(),
  end: z.number().int().positive(),
});
export type CodeRange = z.infer<typeof codeRangeSchema>;

export const codeAnnotationSchema = z.object({
  t: z.number(),
  dur: z.number(),
  code: z.string(),
  language: z.string().default('tsx'),
  title: z.string().optional(),
  highlight: z.array(codeRangeSchema).default([]),
  focus: z.array(codeRangeSchema).default([]),
  position: z
    .enum(['center', 'top-right', 'bottom-left'])
    .default('top-right'),
});
export type CodeAnnotation = z.infer<typeof codeAnnotationSchema>;

export const transitionStyleSchema = z.enum([
  'motion-blur',
  'flash',
  'whip-pan',
  'light-leak',
  'glitch-lite',
]);
export type TransitionStyle = z.infer<typeof transitionStyleSchema>;
