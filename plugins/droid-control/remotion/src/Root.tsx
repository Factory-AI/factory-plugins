import { Composition } from 'remotion';
import { ShowcaseComposition, showcaseSchema } from './compositions/Showcase';
import { calculateShowcaseDuration } from './lib/duration';

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="Showcase"
        component={ShowcaseComposition}
        schema={showcaseSchema}
        calculateMetadata={async ({ props }) => {
          const fps = 30;
          return {
            durationInFrames: calculateShowcaseDuration(props, fps),
            fps,
            width: props.width ?? 1920,
            height: props.height ?? 1080,
          };
        }}
        defaultProps={{
          clips: [],
          layout: 'single' as const,
          labels: [],
          title: 'Demo',
          subtitle: '',
          preset: 'factory' as const,
          keys: [],
          effects: [],
          width: 1920,
          height: 1080,
        }}
      />
    </>
  );
};
