---
name: disk-cleanup
description: Diagnose low disk space and safely retire explicitly approved disposable outputs or local Git worktrees. Use for disk-full warnings, disk-usage triage, or obsolete-checkout cleanup. Diagnosis does not authorize deletion; not for package-cache purges or merging PRs.
---

# Disk cleanup

Start read-only. Removing a checkout does not require merging its PR, and merging
a PR or closing a session does not reclaim its checkout. This workflow does not
grant permission to delete anything.

## 1. Measure the right computer

- Confirm the affected host, filesystem, user-requested scope, and current
  restrictions. A worker's computer may differ from the user's. Do not substitute
  its measurements or launch new writers, installations, or dependency copies.
- Record UTC scan bounds, filesystem total/used/available/free bytes and inodes,
  command/native exit status, permission gaps, and excluded paths. Available
  space can differ from free space because of reserved blocks.
- Start with known relevant directories, not a recursive OS-wide walk. Measure
  recursive **allocated blocks**, not logical lengths. Do not follow symlinks.
  Count hardlinks once across disjoint roots in one inventory; state which path
  receives attribution. Separate scans need reconciliation before summing.
- Keep the full inventory outside proposed deletion roots. Parent totals include
  children: never add both. Account explicitly for direct files, metadata, and
  deeper/unexpanded contents. If using filesystem used space as context, label
  the residual **unscanned filesystem usage / accounting**, not a guessed folder.
  Preserve nonzero scan exits even when a parser succeeds; a partial scan is not
  complete coverage. Concurrent filesystem changes make a scan non-atomic.
- If a chart is requested, reuse an available visualization owner/tool with this
  data and its accounting limits. A treemap is optional, not a deletion plan or
  a reason to install dependencies or build another renderer.

Produce a candidate table: exact path, purpose, allocated attribution, owner,
retention needs, unique work, consumers, permission, and blocking questions.
A large rectangle or an old timestamp is not a deletion qualification.

## 2. Separate permission from ownership

Protect user work, secrets, source/evidence archives, shared documents, operational
Factory data (including installed tools, plugins, logs and transcripts), package
caches, environments, and installed dependencies by default. Protection applies
to nested paths too. Do not truncate unknown logs or manually imitate npm log
rotation, and do not call rotated or unknown log bytes reclaimed.

An owned derived output can be proposed only with provenance, a current retention
decision, and a reconstruction/preservation plan. A `.cache` suffix alone proves
neither disposability nor package-cache status.

For each whole checkout, obtain both:

1. **Explicit human permission for its exact canonical path and contents.**
   Permission to diagnose, remove source-only copies, or remove a different
   output does not include installed dependencies. Get a specific exception
   before retiring a dependency-bearing copy. That exception does not waive
   package-cache or private-state protection.
2. **Current whole-copy owner release.** Confirm no remaining harness,
   acceptance, evidence, active-use, or planned-use need for that physical copy.
   Historical authorship, a clean status, or release of one old binary is not
   whole-copy release. If ownership or retention remains unknown, hold it.

Do not push unpublished work, merge/close PRs, delete branches, or change another
consumer merely to make a checkout removable.

## 3. Qualify each target independently

Use read-only Git commands with optional index writes disabled where supported.
Capture registration/lock state, branch and HEAD, all retained branch refs, the
common Git directory's identity, and retained worktrees' source-state baselines.
The main checkout may own `.git` for every linked worktree; keep that store and
its refs. Clean files do not prove commits were published. Distinguish verified
remote publication from cached remote-tracking reachability, and preserve
unpublished commits through retained refs/object storage and agreed backups.

For the exact candidate:

- Verify canonical root identity, that the root is not a symlink, and its
  expected filesystem, registration, branch and HEAD. Do not cross mounts or
  traverse symlink targets during inventory.
- Capture staged, unstaged and untracked source, plus ignored state. Check
  assume-unchanged/skip-worktree flags. Do not classify an entire ignored tree
  as disposable without inventorying its contents.
- Preserve unique changes, untracked evidence, skills, local configuration and
  required proof before deletion. Verify nonsecret backups and retain their
  reconstruction base/hashes outside the target. Do not read, upload, or print
  secret contents to classify or back them up; stop for an approved preservation
  method when private state is present.
- Inventory entries without following links: paths, types, identities, sizes,
  allocated blocks, link counts, modification times and link destinations.
  Classify ignored dependencies/build products separately from package caches,
  unknown files, operational Factory data, private files and special entries.
  A code directory named `credentials` is not itself a credential file. Resolve
  that distinction using entry type and tracked/package provenance, never a
  blanket exemption for credential-like paths.
- Check accessible process cwd/executable/open-file references, other worktrees'
  symlinks and operational configs, and installed dev wrappers' fallback paths.
  “No process hits” with inaccessible processes is not global proof of no use;
  retain the limitations and rely on explicit current owner release as well.
  A wrapper pointing to a checkout or an unqualified nested cache blocks that
  target. Do not automatically repoint the wrapper or move/delete the cache.

Unknown state, an active/unknown consumer, or a failed preservation gate means
**BLOCKED for that target**. Do not weaken the gate, but independently qualified
targets can proceed within their own permission. Preserve complete inventories
and the exact reviewed command/helper and guards, outside deletion roots.
Revalidate identities, source state and inventory fingerprints immediately before
acting; concurrent changes invalidate the seal and require reassessment.

## 4. Persist intent before acting

Write a durable local intent **outside every deletion root**, containing:

- Exact allowlisted target, approved method, permission and current owner release.
- Manifest identity/hash, source/branch/HEAD/common-store identities and backups.
- Preservation baselines, consumer checks and their limits, unresolved blocks.
- Timestamped pre-operation filesystem measurements and planned verification.

Flush/sync durable records before proceeding; do not keep the only copy in process
memory or the target itself. When shared evidence is required, publish and verify
the intent before deletion using the existing shared-context/publication workflow.
Use fresh body/id/version compare-and-swap for updates, not stale replacements.
Do not invent another storage/publication mechanism for this skill.

## 5. Perform only the approved operation

For a fully qualified linked worktree, use normal, **non-force** Git removal
against the verified common checkout and exact candidate:

```bash
git -C "$common_checkout" worktree remove "$candidate"
```

If Git refuses, **STOP**. Do not retry with force, broad filesystem removal,
manual dependency `rmtree`, sparse-checkout, or another destructive workaround.
For an owned non-Git output, use only the exact scoped method already approved;
this is not permission for recursive home/tmp cleanup.

Inspect the native subprocess exit, not just an outer parser/tool's status.
A final `echo`, pipeline, or wrapper must not turn a failed command into success.
A nonzero exit, timeout or signal is a failure: stop that target and record any
partial state. A running background command is not finished.

**Immediately when the operation returns**, capture native exit/signal, exact
command, start/end times and safe stdout/stderr. Persist and flush/sync that native
result durably **locally outside all deletion roots before any filesystem
statistics, verification, other post-check or shared/network call**.

Only after that durable write succeeds, collect post-operation filesystem
statistics and verification, and persist them separately. A failed measurement
must not erase the known native outcome or trigger another deletion. If the
native receipt cannot be made durable, stop further destructive steps.

## 6. Verify and publish the outcome

- Verify the exact target is absent and its worktree registration is gone.
  Verify retained branches/HEADs, common Git identity, other worktrees' source
  states, backups and protected data against the baseline. Never undo concurrent
  user work to make verification pass.
- Record observed before/after available space, with timestamps and concurrency
  limits. Hardlinked blocks may remain elsewhere; nominal folder allocation and
  sums of copy sizes are not promised reclaim. Label pre-cleanup treemaps as
  dated historical views instead of silently editing their numbers.
- Publish each target's local result and verification through fresh shared CAS
  when applicable. Keep partial success honest: a later failure does not erase
  an earlier verified removal. Report blocked targets and exact next gates.
- A post-delete HTTP 503 is a publication failure, **not a reason to repeat the
  deletion**. Recover from local native results and read-only physical checks.
  If the native exit was lost, record **UNKNOWN**, independently verify physical
  completion and preservation, and never rerun to retrieve an exit code.
  For an unknown publication-write outcome, read the authoritative record before
  retrying publication. Keep local evidence; if required post-receipts cannot be
  reconciled, stop further destructive steps.
- Warning clearance is **UNKNOWN** unless actually observed in the relevant
  current UI/monitor. Crossing an inferred threshold or increasing free space
  does not establish clearance.

Finish with per-target removed-and-verified / blocked / verification-incomplete
states, known or unknown native outcomes, preserved artifacts, measured changes
and limitations. Use the existing PR-publication workflow only if separately
requested; cleanup does not authorize a PR or external post.

Before changing this workflow, walk through the non-destructive
[safety cases](safety-cases.md).
