# Kvarm hooks for concurrent same-file edits

## Six-line summary

[sourced] Same-file edits already merge when their line changes are compatible (`/tmp/kvarm/packages/gitops/test/resolve.test.ts:123-145`).
[sourced] Nexus warns at file or directory-prefix granularity, without locking edits (`/tmp/kvarm/apps/platform/src/services/intents/forecast.ts:6-15`).
[sourced] Semantic overlap means heuristic dependency evidence, not AST-region collision detection (`/tmp/kvarm/docs/architecture.md:321-326`).
[sourced] `compose_candidates` folds file-selected workspace snapshots into a new commit, workspace and change (`/tmp/kvarm/apps/platform/src/services/compositions.ts:4-16`).
[inferred] Revision-anchored advisory regions fit the existing architecture better than mandatory region ownership.
[inferred] Start with same-file hunk evidence. Keep separate workspaces and the exact-tree train.

## 1. What happens today

[sourced] Inspected revision: `c80253b14bbf2c16d67461f6e4dbe58b2e1d98e4` (`/tmp/kvarm/.git/refs/heads/main:1`).

### Two workspaces, one file

1. [sourced] Each workspace starts with its own server-created ref at the base head. Creation may include an intent. `set_intent` replaces an advisory declaration. Nothing is locked (`/tmp/kvarm/packages/ops/src/index.js:350-362`, `/tmp/kvarm/packages/ops/src/index.js:881-892`).
2. [sourced] Before pushes, `forecast(paths)` can find intersecting declarations and existing observed paths. Matching uses exact paths, ancestor prefixes and descendant ranges. The response distinguishes declared and observed evidence and carries revision, observation time, declaration expiry and truncation (`/tmp/kvarm/apps/platform/src/services/intents/forecast.ts:458-508`, `/tmp/kvarm/apps/platform/src/services/intents/forecast.ts:581-608`).
3. [sourced] Push processing reconciles cumulative workspace-base-to-head paths, not unsaved buffers. It also computes semantic rows. A newer-push guard prevents late reconciliation from replacing newer observations (`/tmp/kvarm/apps/platform/src/services/intents/forecast.ts:219-264`, `/tmp/kvarm/apps/platform/src/services/intents/store.ts:49-102`).
4. [sourced] Once both pushes are indexed, `get_overlap` reports active workspace pairs sharing observed paths on the selected base branch. It does not pair declarations. `workspace_path_shares` supplies shared-path counts; collapsed prefixes receive separate probes (`/tmp/kvarm/apps/platform/src/services/intents/forecast.ts:736-785`, `/tmp/kvarm/apps/platform/migrations/0022_nexus_path_shares.sql:4-24`).
5. [sourced] A workspace opens a change with `review`, `auto` or `draft` policy. Auto publication uses the queue; draft never merges; review permits explicit merge. Overlap predictions are not publication gates (`/tmp/kvarm/packages/ops/src/index.js:384-396`, `/tmp/kvarm/docs/architecture.md:103-105`, `/tmp/kvarm/docs/architecture.md:320-326`).
6. [sourced] The train prepares up to eight candidates, stacking each on the preceding prepared result. Four may be checking. Queued entries have per-principal fairness ordering, so this is not strict global FIFO (`/tmp/kvarm/docs/architecture.md:338-339`, `/tmp/kvarm/apps/platform/src/services/queue.ts:194-219`).
7. [sourced] Preparation calls `prepareMerge`, which runs three-way merge against the merge base. A clean result is either the original fast-forward commit or a two-parent merge commit. Both-modified files enter line-level diff3 (`/tmp/kvarm/apps/platform/src/services/queue.ts:1245-1251`, `/tmp/kvarm/apps/platform/src/services/merge.ts:412-440`, `/tmp/kvarm/apps/platform/src/services/merge.ts:490-509`, `/tmp/kvarm/packages/gitops/src/merge3.ts:678-734`).
8. [sourced] Non-overlapping and identical changes can merge. Touching or overlapping line changes normally conflict. The train additionally chooses the target's variant for whitespace-equivalent conflicting hunks, except lockfiles. Lockfile conflicts request regeneration (`/tmp/kvarm/packages/gitops/src/mergeText.ts:6-19`, `/tmp/kvarm/packages/gitops/src/resolve.ts:5-15`, `/tmp/kvarm/apps/platform/src/services/merge.ts:100-101`, `/tmp/kvarm/apps/platform/src/services/merge.ts:422-434`).
9. [sourced] A conflict against speculative predecessors waits until they publish. A conflict against the real target ejects the entry, marks the change conflicted and dispatches a resolution task (`/tmp/kvarm/apps/platform/src/services/queue.ts:1264-1277`).
10. [sourced] Clean composed trees receive exact-tree CI staging and required-check evaluation. Failed checks eject the affected entry. Before publication, Kvarm revalidates policy, authorization, reviews and check identity. A short fenced lease and receive-pack expected-old SHA protect the target. Foreign movement causes recomposition (`/tmp/kvarm/apps/platform/src/services/queue.ts:1279-1339`, `/tmp/kvarm/apps/platform/src/services/queue.ts:1355-1377`, `/tmp/kvarm/docs/architecture.md:340-346`).

[sourced] Resolution is whole-change work today. One live `resolve-conflict` task carries head, base and conflicting paths. Completion requires a pushed descendant of the change head that merges cleanly with the current base, plus the current claim generation and owner (`/tmp/kvarm/apps/platform/src/services/conflicts.ts:6-18`, `/tmp/kvarm/apps/platform/src/services/conflicts.ts:130-135`, `/tmp/kvarm/apps/platform/src/services/tasks.ts:691-728`, `/tmp/kvarm/apps/platform/src/services/tasks.ts:756-759`).

### What `compose_candidates` actually does

[sourced] This is an explicit alternative to integrating changes individually. It accepts 2–8 active workspaces in fold order. Each piece optionally selects whole paths or directory prefixes, at most 64 selectors. Each piece uses its own merge base with the current base head. The fold is `acc = merge3(pieceBase, acc, selectedHead)` (`/tmp/kvarm/apps/platform/src/services/compositions.ts:49-56`, `/tmp/kvarm/apps/platform/src/services/compositions.ts:293-341`, `/tmp/kvarm/packages/gitops/src/compose.ts:2-15`).

[sourced] Selection does not choose individual hunks or symbols. It applies selected changed leaves. Textual conflicts, unmatched selectors, budget stops or an empty result refuse creation. Successful composition creates one commit whose only parent is the base head, with source-workspace/head/path trailers, then a new workspace ref and change (`/tmp/kvarm/packages/gitops/src/compose.ts:124-162`, `/tmp/kvarm/apps/platform/src/services/compositions.ts:356-397`, `/tmp/kvarm/apps/platform/src/services/compositions.ts:437-449`, `/tmp/kvarm/apps/platform/src/services/compositions.ts:485-494`).

[sourced] It asks the checks gate after opening that change. A failure is recorded as `{kind: "semantic", checks, treeSha}`. Pending checks are not a semantic conflict. The stored composition answer is a creation-time snapshot, and a failed change-opening operation can leave the composed workspace with `changeSetError` (`/tmp/kvarm/apps/platform/src/services/compositions.ts:134-151`, `/tmp/kvarm/apps/platform/src/services/compositions.ts:495-531`).

[sourced] Composition currently passes no deterministic whitespace resolver, unlike train preparation (`/tmp/kvarm/apps/platform/src/services/compositions.ts:346-355`, `/tmp/kvarm/apps/platform/src/services/merge.ts:412-424`).
[inferred] A whitespace conflict can therefore fail explicit composition while the train resolves it. Neither operation provides live co-editing.

## 2. Granularity and semantic overlap

[sourced] Intent fields are `paths`, `prefixes`, and TTL. Stored declarations have at most 64 path entries. Observations cap at 300 entries and collapse wide edits into directory prefixes. A forecast accepts at most 20 query paths. Intersection is equality or prefix coverage (`/tmp/kvarm/apps/platform/src/services/intents/paths.ts:8-38`, `/tmp/kvarm/apps/platform/src/services/intents/paths.ts:63-70`, `/tmp/kvarm/apps/platform/src/services/intents/paths.ts:106-141`, `/tmp/kvarm/apps/platform/src/services/intents/paths.ts:152-186`).

[sourced] Overlap pairs contain workspace IDs, paths, counts and observed-head timestamps. No region coordinates occur in that shape (`/tmp/kvarm/apps/platform/src/services/intents/forecast.ts:624-655`).
[inferred] Two agents changing distant functions in one file still produce a path-overlap warning. Nexus cannot distinguish that case from editing the same function.

[sourced] Semantic analysis uses regex extraction over comment- and string-masked source. It follows supported TS/JS, Python and Go imports up to two hops. It compares TS/JS exported-name sets and groups contract files by kind and directory. It misses aliases, computed imports and several export forms (`/tmp/kvarm/packages/review/src/semantic.ts:1-42`, `/tmp/kvarm/docs/architecture.md:322`).

[sourced] `workspace_semantic` stores import target paths, chains and imported names; added/removed export names; and contract-family paths. Forecasts join this index rather than scanning repository source. `semantic:export` means a changed module removed an imported name, or imports were wildcard/unknown. Otherwise the dependency hit is `semantic:import`. Shared contract families produce `semantic:contract` (`/tmp/kvarm/apps/platform/migrations/0029_nexus_semantic.sql:5-28`, `/tmp/kvarm/docs/architecture.md:323-325`, `/tmp/kvarm/apps/platform/src/services/intents/semantic.ts:265-290`).

[sourced] Analysis is bounded to 40 changed sources, 96 blob reads, 4 MiB total, 256 KiB per file and 1.5 seconds, among other caps. Truncation is explicit. This signal gates nothing (`/tmp/kvarm/packages/review/src/semantic.ts:23-42`, `/tmp/kvarm/docs/architecture.md:324-326`).
[inferred] Export-name evidence is a limited symbol notion, not symbol ownership, signature checking or an AST-region index. Semantic overlap also differs from composition's check-failure-based “semantic conflict.”

[sourced] Sub-file primitives already exist outside Nexus. Diffcore hunks have old/new starts and lengths. Review coverage has content-keyed hunk/file units. Findings have line ranges (`/tmp/kvarm/packages/diffcore/src/index.ts:33-52`, `/tmp/kvarm/packages/review/src/guide.ts:29-45`, `/tmp/kvarm/packages/review/src/guide.ts:239-263`, `/tmp/kvarm/apps/platform/src/services/reviewGuide.ts:1012-1016`).
[inferred] These are reusable evidence primitives, not existing leases or AST identities.

## 3. Extension points and candidate mechanisms

[inferred] Keep the current path index as a coarse candidate filter. Add region evidence alongside it. Do not reinterpret path strings as `file#symbol` because prefix matching would lose its meaning.

[sourced] All operations belong in `/tmp/kvarm/packages/ops/src/index.js`; MCP dispatches those entries to REST. Intent routes are thin adapters (`/tmp/kvarm/docs/architecture.md:15-17`, `/tmp/kvarm/apps/platform/src/mcp/server.ts:1-16`, `/tmp/kvarm/apps/platform/src/routes/api/intents.ts:1-13`).

[guess] Sizes below are single-engineer implementation-and-test estimates, not production commitments. All proposed tables, operations and files are additions, not existing capabilities.

| Mechanism | Where it lives and tables | Change size | Risk | What Kvarm must add upstream |
|---|---|---|---|---|
| [inferred] Range or symbol intents | Extend `/tmp/kvarm/apps/platform/src/services/intents/paths.ts`, `/tmp/kvarm/apps/platform/src/services/intents/store.ts`, `/tmp/kvarm/apps/platform/src/services/intents/forecast.ts`, `/tmp/kvarm/apps/platform/src/services/workspaces.ts` and `/tmp/kvarm/apps/platform/src/routes/api/intents.ts`. Add `workspace_regions` plus freshness fields on `workspaces`. Preserve the coarse indexes. [sourced] Existing path schema: `/tmp/kvarm/apps/platform/migrations/0011_intents.sql:3-32`. | [guess] 4–8 days for ranges; 2–4 weeks with one-language symbol extraction. | [inferred] Line drift, renames, insertion anchors, mixed revisions and truncation can falsely suggest disjointness. | [inferred] Extend `create_workspace`, `set_intent`, `get_intent`, `forecast`, `get_overlap` with structured regions and revision IDs. Add observed-region reconciliation and bounded interval probes. |
| [inferred] Symbol-level leases | Reuse `/tmp/kvarm/apps/platform/src/services/fencing.ts` and `fenced_claims`. Add `region_claims` metadata and `/tmp/kvarm/apps/platform/src/services/regions.ts` for canonical identity. [sourced] Existing claims key arbitrary kind/subject and carry identity (`/tmp/kvarm/apps/platform/src/services/fencing.ts:56-84`). | [guess] 1–2 weeks for advisory leases after symbol identity exists. | [inferred] Renamed, overloaded or nested symbols can alias. A changed identity can supersede a live claim. | [inferred] Add claim/heartbeat/release-region operations. Derive canonical subjects and identities server-side. Fence every consequential write, not just heartbeat. |
| [inferred] Region-scoped task claims | Extend `/tmp/kvarm/apps/platform/src/services/tasks.ts`; add `agent_task_regions` and `region_claims`. [sourced] Existing `agent_tasks` permits only one live task per change and kind (`/tmp/kvarm/apps/platform/migrations/0016_conflict_tasks.sql:14-35`). | [guess] 2–3 weeks advisory; enforced edits materially larger. | [inferred] Two differently named claims can overlap. Independent equality-key fences cannot exclude intersecting intervals. | [inferred] Add region-aware claim subjects, atomic overlap exclusion, multi-region acquisition policy and generation-bound completion. Use guarded D1 transactions or one file DO, not read-then-claim. |
| [inferred] Structured merge driver | Extend `/tmp/kvarm/packages/gitops/src/merge3.ts:678-734`, `/tmp/kvarm/apps/platform/src/services/merge.ts:412-424`, `/tmp/kvarm/apps/platform/src/services/queue.ts` and `/tmp/kvarm/apps/platform/src/services/compositions.ts:346-355`. Add driver/version metadata to `merge_queue` and `compositions` if selection is configurable. | [guess] 2–4 weeks for one restricted format. General language merging is larger. | [inferred] Silent semantic loss, unstable output and inconsistent train/composition rules. | [inferred] Add a bounded full-base/ours/theirs blob hook. [sourced] The current hunk callback only returns ours, theirs or null (`/tmp/kvarm/packages/gitops/src/merge3.ts:87-91`). [inferred] Version driver policy, record resolution provenance and check the produced tree. |
| [inferred] First-class conflict objects | Extend `/tmp/kvarm/apps/platform/src/services/conflicts.ts`, `/tmp/kvarm/apps/platform/src/services/compositions.ts` and `/tmp/kvarm/packages/gitops/src/mergeText.ts`. Add `merge_conflicts` and region/task links. [sourced] Current persistence reduces conflicts to head/base plus path/kind, capped at 100 (`/tmp/kvarm/apps/platform/src/services/conflicts.ts:23-61`). | [guess] 1–2 weeks for durable metadata and read APIs. | [inferred] Resolutions become stale when either side moves. Persisting refused compositions changes today's create-nothing contract. | [inferred] Add conflict IDs, side blob hashes, hunk spans, identity/lifecycle and list/get/resolve operations. Keep refused compositions ephemeral unless a separate operation explicitly persists them. |
| [inferred] Live shared buffer | Add `/tmp/kvarm/apps/platform/src/do/fileBuffer.ts` and `/tmp/kvarm/apps/platform/src/routes/api/buffers.ts`. Update `/tmp/kvarm/apps/platform/src/env.ts`, `/tmp/kvarm/apps/platform/wrangler.jsonc` and DO exports. Add `buffer_sessions` metadata in D1; store draft state/log in DO storage, checkpoints in Artifacts. [sourced] Existing binding pattern: `/tmp/kvarm/apps/platform/wrangler.jsonc:73-109`. | [guess] 6–12+ weeks for a credible pilot. | [inferred] New draft-content authority, offline convergence, multi-file atomicity, lost attribution and unchecked checkpoint publication. | [inferred] Add authenticated edit transport, CRDT or lock protocol, durable snapshots/resync, deduplication, quotas, git checkpoint CAS and explicit session-to-commit attribution. |

### Invariants that must survive

- [sourced] Fencing checks generation and owner; different identities permit takeover. Renewal can succeed after expiry if no takeover happened (`/tmp/kvarm/apps/platform/src/services/fencing.ts:68-84`, `/tmp/kvarm/apps/platform/src/services/fencing.ts:112-147`).
  [inferred] Define whether region edits invalidate a claim or reanchor it. Task fencing is not automatically git-path enforcement. Hard enforcement must also cover direct pushes and platform writes.
- [sourced] Git admission currently checks refs, ownership and protection before streaming objects (`/tmp/kvarm/docs/architecture.md:78-89`).
  [inferred] Enforcing “this push edited only my region” needs content validation and a new publication protocol. A lease alone cannot prove it.
- [sourced] Checks bind to composed trees. With no producers, the required set is empty and successful (`/tmp/kvarm/docs/architecture.md:24-28`).
  [inferred] Never reuse checks merely because regions are disjoint. A structured driver or buffer checkpoint must finish before check identity is fixed.
- [sourced] Attribution is commit-level. Composition records source heads and selected paths, not line contributions (`/tmp/kvarm/docs/architecture.md:264-265`, `/tmp/kvarm/apps/platform/migrations/0012_candidates.sql:32-45`).
  [inferred] Add contribution mappings before claiming per-symbol authorship.
- [sourced] Limits are 600 reads and 120 writes per minute per token; pushes cost about two writes (`/tmp/kvarm/README.md:433-436`).
  [inferred] Batch regions and heartbeats. Per-keystroke REST writes do not fit. WebSocket frames need explicit quotas, not an assumed exemption.
- [sourced] RepoEvents retains only 200 events, accepts no client edit messages and expires connections after at most ten minutes. Emission is best effort (`/tmp/kvarm/apps/platform/src/do/repoEvents.ts:19-20`, `/tmp/kvarm/apps/platform/src/do/repoEvents.ts:54-55`, `/tmp/kvarm/apps/platform/src/do/repoEvents.ts:66-95`, `/tmp/kvarm/apps/platform/src/services/live.ts:12-30`).
  [inferred] Use it for invalidation, not CRDT durability or complete edit replay. A buffer needs its own snapshot and operation recovery.

## 4. Runtime fit

[sourced] Kvarm is one Cloudflare Worker. Artifacts is remote-only, D1 holds metadata rather than file content, and server-side git writes use receive-pack with expected-old values (`/tmp/kvarm/docs/architecture.md:3`, `/tmp/kvarm/docs/architecture.md:87-93`, `/tmp/kvarm/CONTRIBUTING.md:18`).
[inferred] There is no local-first git fallback here. A desktop shared buffer would be a separate Acan capability until checkpointed remotely.

[inferred] Advisory ranges, bounded D1 metadata, and deterministic JS or WASM merging fit. Native git custom-driver executables or a persistent checkout do not fit the existing Worker merge path. Unbounded repository-wide AST analysis conflicts with its budget discipline.
[sourced] Merge budgets include 4 MiB per blob, 300 blob reads, 16 MiB created objects and one concurrent content merge (`/tmp/kvarm/apps/platform/src/services/merge.ts:63-73`).

[inferred] A DO keyed by repository, collaboration session and file identity can host a lock or CRDT. Path alone is insufficient because branches and renames need identity. This fits the existing storage/WebSocket pattern, but is not implemented. Cross-file commits need a checkpoint coordinator because independent file DOs cannot themselves provide one atomic multi-file snapshot.
[inferred] CRDT convergence would preserve a shared sequence of text operations. It would not prove that concurrent code changes are semantically compatible.

[sourced] Optional Cloudflare Containers already host Linux runners and agent sandboxes, separately from Worker services (`/tmp/kvarm/docs/architecture.md:45-51`).
[inferred] Native parsing could run there or on a user runner, reporting a revision-bound result. That adds scheduling and trust boundaries, not an in-process merge hook. DO draft content also needs an explicit architectural exception or documented draft tier. Keep D1 content-free and Artifacts authoritative for committed git.

## Recommended smallest first step

[sourced] Existing `get_file_diff` already exposes per-file hunks between explicit commit SHAs (`/tmp/kvarm/packages/ops/src/index.js:319-330`).
[inferred] Acan can first prototype the revision-anchored overlap view with that read API, without upstream changes.

[inferred] Add optional same-file region evidence to Nexus, not locks or a shared buffer. First compute bounded observed changed-line spans from a common immutable base blob for already path-overlapping workspace pairs. Reuse diffcore, but distinguish context hunks from actual edits. Include insertion anchors, side SHAs, stale/approximate/truncated status and whole-file fallback.

[inferred] Next add optional declared ranges shaped as `{path, revision, startLine, endLine}`. Compare only coordinates mapped to the same revision. Never suppress the coarse path warning when mapping is uncertain. Defer AST identities, mandatory leases and hunk-selecting composition.

[inferred] Acan should label this “same file, separate observed edits,” not “safe.” Continue independent workspace commits and exact-tree integration. Validate non-overlapping edits, adjacent insertions, identical edits, moved lines, renames, stale pushes and budget exhaustion before enabling the advisory view.
