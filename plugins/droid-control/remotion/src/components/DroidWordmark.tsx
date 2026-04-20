import React from 'react';
import type { Palette } from '../lib/palettes';

const DROID_ASCII = `
  ██████████    ██████████    ██████████    ███    ██████████
  ███    ███    ███    ███    ███    ███    ███    ███    ███
  ███    ███    ███    ███    ███    ███    ███    ███    ███
  ███    ███    ██████████    ███    ███    ███    ███    ███
  ███    ███    ███ ███       ███    ███    ███    ███    ███
  ███    ███    ███  ███      ███    ███    ███    ███    ███
  ██████████    ███   ███     ██████████    ███    ██████████
`.trim();

export const DroidWordmark: React.FC<{
  palette: Palette;
  logoColor?: string;
  taglineColor?: string;
}> = ({ palette, logoColor, taglineColor }) => (
  <div
    style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
    }}
  >
    <div
      style={{
        color: logoColor ?? 'white',
        fontFamily: "'Geist Mono', 'SF Mono', 'Cascadia Code', 'Fira Code', monospace",
        fontSize: 24,
        lineHeight: 1.2,
        whiteSpace: 'pre',
        textAlign: 'left',
        textShadow: '0 0 20px rgba(255,255,255,0.4)',
      }}
    >
      {DROID_ASCII}
    </div>
    <div
      style={{
        marginTop: 40,
        color: taglineColor ?? palette.accent,
        fontSize: 32,
        fontWeight: 300,
        fontFamily: "'Geist', system-ui, sans-serif",
        letterSpacing: '0.2em',
        textShadow: `0 0 15px ${palette.accent}66`,
      }}
    >
      AUTONOMOUS ENGINEERING
    </div>
  </div>
);
