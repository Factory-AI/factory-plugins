# typescript

Opinionated TypeScript and React patterns for cleaner, safer code.

## Skills

### `ban-type-assertions`

Enable `@typescript-eslint/consistent-type-assertions` with `assertionStyle: 'never'` and replace every `as X` cast with patterns the compiler can verify. Covers zod parsing for data boundaries, control-flow narrowing for unions, and the narrow cases where `eslint-disable` is acceptable.

### `no-use-effect`

Five replacement patterns for `useEffect`: derived state, query libraries, event handlers, `useMountEffect` for one-time external sync, and the `key` prop for state resets. Based on Alvin Sng's [tweet](https://x.com/alvinsng/status/2033969062834045089) and the React docs [You Might Not Need an Effect](https://react.dev/learn/you-might-not-need-an-effect).

### `fix-knip-unused-exports`

Fix every category of knip "Unused exports" violation: extract test-only exports into new files, remove dead barrel re-exports, un-export internally-used symbols, and delete dead code. Explains how knip traces usage across a monorepo and how to handle cross-package test imports.

## Install

```bash
droid plugin install typescript@factory-plugins
```
