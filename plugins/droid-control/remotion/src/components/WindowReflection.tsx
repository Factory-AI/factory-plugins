import React from 'react';

// Uses -webkit-box-reflect to avoid rendering children twice (which would
// double Video decoding and hurt render performance).
export const WindowReflection: React.FC<{
  opacity?: number;
  children: React.ReactNode;
}> = ({ opacity = 0.06, children }) => {
  return (
    <div
      style={
        {
          WebkitBoxReflect: `below 4px linear-gradient(transparent 60%, rgba(0,0,0,${opacity}))`,
        } as React.CSSProperties
      }
    >
      {children}
    </div>
  );
};
