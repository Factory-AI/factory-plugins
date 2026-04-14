# Architecture

## The problem

Put all driver knowledge, recording lifecycle, video rendering, and verification logic in one skill and two things happen. First, the droid loads 3000 tokens of Windows KVM docs to record a tuistory demo on Linux. Second, and this is the one that actually hurts, the droid gets all the information at once and has to figure out what's relevant *right now*. It makes worse decisions, skips steps it shouldn't, and invents steps it doesn't need to.

## UX for droids

Droids aren't exempt from information architecture. They have finite context, they get distracted by irrelevant detail, and they degrade under overload.

Every skill in this plugin is a surface the droid interacts with at a specific moment in a workflow: scoped to what it needs right now, actionable on first read, with an explicit handoff to the next surface. The same instincts that make a good CLI or a good API apply. Don't dump everything. Sequence the information. Make the next step obvious.

A command like `/demo` doesn't contain Remotion props or driver-specific logic. It parses intent, builds commitments, and tells the droid which skills to load. The droid never sees video rendering details while it's still planning what to record.

## Waterfall routing

Skills chain into each other without hardcoding. The orchestrator doesn't call skills or build a pipeline. It tells the droid which skills to load based on three independent lookups. Once loaded, each skill's exit is the next skill's entry:

```
/demo parses intent, commits deliverables
  → orchestrator routes to driver + stage + artifact skills
    → capture launches the app, records, hands off clips + metadata
      → compose receives clips, builds Remotion props, renders video
        → verify checks the output against the original commitments
```

No state machine or orchestration framework. Just documents whose outputs naturally feed into the next document's inputs. The droid follows the waterfall because each skill makes the next step obvious, not because something forces it to. Complex multi-stage workflows emerge from skill composition rather than control flow.

## Task delegation

Because each stage's inputs and outputs are explicit, mechanical work naturally decomposes into worker tasks. The parent agent retains planning and editorial control; workers execute exact commands and return file paths.

Capture workers for both branches run in parallel. They need tctl commands and worktree paths, not PR context. The render worker gets a props JSON and clip paths. It doesn't need to know what the PR does or why the demo matters. Verification stays with the parent because it requires the original commitments and judgment about whether the evidence holds up.

The skill boundaries are the delegation boundaries. You don't need a separate delegation framework because the decomposition into self-contained stages already defines what can be farmed out, what needs creative judgment, and what's too trivial to be worth the overhead.

## Orthogonal routing

The orchestrator makes three independent lookups:

- **Target**: what are you driving? (droid-cli, other TUI, web app, byte capture)
- **Stage**: what does the workflow need? (capture, compose, verify)
- **Artifact**: does compose need polish tools? (showcase)

These compose without cross-product explosion. 6 targets + 3 stages + 1 artifact route = 10 skills, not 18. Adding a new target means writing one skill and adding one row to the routing table. Every existing stage and artifact skill works with it immediately.

## Hybrid handoffs

Commands hand off to the compose stage with two sections: structured fields for mechanical decisions, natural language for creative ones.

Structured: layout, speed, preset, fidelity, effects tier. These have correct answers. A side-by-side layout is either `side-by-side` or it isn't.

Natural language: what the viewer should take away, which moments to hold, how to frame the story. These are editorial judgments that benefit from the droid's understanding of the PR context.

Two failure modes this prevents: over-specifying creative decisions up front (the droid produces rigid, paint-by-numbers output) and under-specifying mechanical params (the droid hallucinates presets and layouts). The effects tier is a concrete example. The command commits a single word ("utilitarian" or "full"), and compose makes specific, grounded choices after capture, when it has actual recordings to look at.

## Platform isolation

Platform-specific content lives in `platforms/` subdirs under the relevant skill. A droid on Linux loads `true-input/platforms/linux.md`. It never sees the Windows KVM or macOS QEMU docs. This is a routing decision, not a reading-comprehension test.
