import type { Preset } from './schema';

export interface Palette {
  bg: string;
  surface: string;
  accent: string;
  border: string;
  text: string;
  muted: string;
  success: string;
  error: string;
}

const factoryPalette: Palette = {
  bg: '#0a0804',
  surface: '#181818',
  accent: '#EE6018',
  border: '#342F2D',
  text: '#f0e8e0',
  muted: '#948781',
  success: '#6FAB78',
  error: '#D9363E',
};

const catppuccinPalette: Palette = {
  bg: '#0d1117',
  surface: '#181818',
  accent: '#89b4fa',
  border: '#313244',
  text: '#cdd6f4',
  muted: '#6c7086',
  success: '#a6e3a1',
  error: '#f38ba8',
};

export function getPalette(preset: Preset): Palette {
  if (preset === 'factory' || preset === 'factory-hero') {
    return factoryPalette;
  }
  return catppuccinPalette;
}
