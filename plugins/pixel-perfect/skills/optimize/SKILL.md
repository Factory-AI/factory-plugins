---
name: optimize
description: "Diagnoses and fixes UI performance using real browser measurements—Lighthouse, Core Web Vitals, profiling. Use when the user mentions slow, laggy, janky, performance, bundle size, load time, or wants a faster experience."
user-invocable: true
---

Measure actual performance in the browser, identify bottlenecks, and verify improvements with before/after data.

## 1. Baseline Measurement via agent-browser

Run Lighthouse and collect Core Web Vitals before changing anything:

```javascript
// Run Lighthouse audit via eval in agent-browser
// Collect: Performance score, LCP, FID/INP, CLS, FCP, TBT, SI
```

- Navigate to the target page with agent-browser
- Run a Lighthouse performance audit via eval
- Record Core Web Vitals: **LCP** (< 2.5s), **FID/INP** (< 200ms), **CLS** (< 0.1)
- Screenshot the Lighthouse results
- Note the overall performance score as the baseline

Also collect runtime metrics:
- Use Performance API via eval to measure navigation timing
- Check `performance.getEntriesByType('resource')` for slow-loading assets
- Measure JavaScript execution time for key interactions

## 2. Identify Bottlenecks

### Image Issues
- Find images without `width`/`height` attributes (causes CLS)
- Check for oversized images (3000px served at 300px)
- Look for missing `loading="lazy"` on below-fold images
- Identify images not using modern formats (WebP/AVIF)
- Check for missing `srcset` / responsive images

### Font Loading
- Check for fonts without `font-display: swap` (causes FOIT)
- Look for unused font weights being loaded
- Verify font preloading for critical fonts
- Check for font subsetting opportunities

### JavaScript
- Identify large bundles (> 200KB compressed)
- Find render-blocking scripts missing `async`/`defer`
- Look for code that should be lazy-loaded
- Check for long tasks blocking the main thread (> 50ms)

### CSS & Network
- Find render-blocking or unused CSS
- Check for missing `content-visibility: auto` on long pages
- Count total requests and payload size
- Check for missing compression (gzip/brotli) and caching headers

## 3. Runtime Profiling via agent-browser

Use the browser's profiling capabilities:

```javascript
// Measure interaction responsiveness
const start = performance.now();
// ... trigger interaction ...
const duration = performance.now() - start;
```

- Profile key user interactions (clicks, scrolls, form submissions)
- Identify jank during scroll (frame drops below 60fps)
- Check memory usage for leaks during navigation
- Measure time-to-interactive for dynamic content

## 4. Apply Fixes

Address issues in order of impact:

### Critical (LCP/CLS/FID)
- Add dimensions to images and embeds (fixes CLS)
- Preload LCP image or critical font
- Defer non-critical JavaScript
- Inline critical CSS

### High Impact
- Convert images to WebP/AVIF
- Add `loading="lazy"` to below-fold images
- Implement code splitting for large bundles
- Add `font-display: swap` to all `@font-face` rules

### Medium/Low Impact
- Add `srcset` for responsive images
- Remove unused CSS and JavaScript
- Add `content-visibility: auto` to long sections
- Subset fonts, preconnect to origins, add resource hints

## 5. Verify with Before/After Measurements

After fixes, re-run everything via agent-browser:

- Run Lighthouse again and compare scores
- Re-measure Core Web Vitals and compare against baseline
- Screenshot the new Lighthouse results alongside the original
- Verify no visual regressions in the rendered page
- Test on throttled connection (agent-browser network throttling) to confirm improvements under stress

Report the delta:
- Performance score: before → after
- LCP: before → after
- CLS: before → after
- FID/INP: before → after
- Total bundle size: before → after

## Rules

- Always measure before optimizing—don't guess at bottlenecks
- Never sacrifice accessibility or functionality for performance
- Don't lazy-load above-fold content
- Test with throttled CPU/network, not just fast connections
- Run lint, type-check, and tests after changes
