// Run: npm test (node --test; needs Node 22.18+ or 24+ for built-in type stripping)
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  TITLE_DURATION_S,
  OUTRO_DURATION_S,
  TRANSITION_FRAMES,
  calculateShowcaseDuration,
  contentFrames,
  contentSequenceFrames,
  playbackSpeed,
} from '../src/lib/duration.ts';

const FPS = 30;

test('content frames are the longest source duration divided by speed', () => {
  assert.equal(
    contentFrames({ clipDuration: 5, speed: 2, clips: ['a.mp4', 'b.mp4'] }, FPS),
    75
  );
});

test('content frames round up partial frames', () => {
  assert.equal(contentFrames({ clipDuration: 5.01, speed: 3, clips: ['a.mp4'] }, FPS), 51);
});

test('speed defaults to 1x', () => {
  assert.equal(playbackSpeed({}), 1);
  assert.equal(contentFrames({ clipDuration: 30, clips: ['a.mp4'] }, FPS), 900);
});

test('unprobed clips fall back to 60s and title-only videos to 10s', () => {
  assert.equal(contentFrames({ clips: ['a.mp4'] }, FPS), 1800);
  assert.equal(contentFrames({ clips: [] }, FPS), 300);
});

test('content sequence pads the clips by one transition on each side', () => {
  const props = { clipDuration: 5, speed: 2, clips: ['a.mp4', 'b.mp4'] };
  assert.equal(contentSequenceFrames(props, FPS), 75 + 2 * TRANSITION_FRAMES);
});

test('content shorter than a transition still yields a sequence longer than both transitions', () => {
  const props = { clipDuration: 5, speed: 20, clips: ['a.mp4'] };
  assert.equal(contentFrames(props, FPS), 8);
  assert.ok(contentSequenceFrames(props, FPS) > 2 * TRANSITION_FRAMES);
  assert.equal(calculateShowcaseDuration(props, FPS), 4 * FPS + 8 + 3.5 * FPS);
});

test('total length is title + clips + outro; transitions overlap the content padding', () => {
  const props = { clipDuration: 5, speed: 2, clips: ['a.mp4', 'b.mp4'] };
  const sequenceTotal =
    TITLE_DURATION_S * FPS +
    contentSequenceFrames(props, FPS) +
    OUTRO_DURATION_S * FPS -
    2 * TRANSITION_FRAMES;
  assert.equal(calculateShowcaseDuration(props, FPS), sequenceTotal);
  assert.equal(
    calculateShowcaseDuration(props, FPS),
    TITLE_DURATION_S * FPS + contentFrames(props, FPS) + OUTRO_DURATION_S * FPS
  );
  assert.equal(calculateShowcaseDuration(props, FPS), 300);
});
