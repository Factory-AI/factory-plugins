import type { Preset } from './schema';

export interface PresetConfig {
  bar: 'colorful' | 'rings' | 'none';
  barSide: 'left' | 'right';
  radius: number;
  padding: number;
  margin: number;
  shadow: boolean;
  bgStyle: 'solid' | 'gradient';
}

const presetConfigs: Record<Preset, PresetConfig> = {
  macos: {
    bar: 'colorful',
    barSide: 'left',
    radius: 12,
    padding: 20,
    margin: 60,
    shadow: true,
    bgStyle: 'solid',
  },
  minimal: {
    bar: 'none',
    barSide: 'left',
    radius: 8,
    padding: 16,
    margin: 32,
    shadow: false,
    bgStyle: 'solid',
  },
  hero: {
    bar: 'colorful',
    barSide: 'left',
    radius: 16,
    padding: 24,
    margin: 80,
    shadow: true,
    bgStyle: 'gradient',
  },
  presentation: {
    bar: 'colorful',
    barSide: 'left',
    radius: 12,
    padding: 24,
    margin: 48,
    shadow: true,
    bgStyle: 'solid',
  },
  factory: {
    bar: 'colorful',
    barSide: 'left',
    radius: 12,
    padding: 20,
    margin: 80,
    shadow: true,
    bgStyle: 'solid',
  },
  'factory-hero': {
    bar: 'colorful',
    barSide: 'left',
    radius: 12,
    padding: 24,
    margin: 80,
    shadow: true,
    bgStyle: 'gradient',
  },
};

export function getPresetConfig(preset: Preset): PresetConfig {
  return presetConfigs[preset];
}
