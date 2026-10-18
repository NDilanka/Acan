# History replay of same-file TypeScript merges

## Six-line summary

[measured] The pinned Kvarm, Hono, and tRPC histories contain 1,137 same-file parent pairs across 151 merge commits.  
[measured] Line merge conflicts on 608 pairs, or 53.47%, including 384 commons-only overlaps and 222 declaration overlaps.  
[measured] Only 2 pairs are region-disjoint and line-conflicting, or 0.18% of all pairs, below Tenure's 5% kill line.  
[measured] All 134 disjoint-region splices parse, but only 124 match the recorded merge exactly, and neither conflicting splice matches.  
[measured] Nine conflicts qualify as conservative Slots candidates, or 1.48% of conflicts, including 3 of Kvarm's 10 conflicts.  
[inferred] Kill Tenure's merge-conflict justification on this corpus, retain line merge, and limit Slots to a small declared-registry experiment.

## What this experiment decides

[inferred] A maintainer choosing between ordinary separate-workspace merges and a region ledger needs the incremental benefit beyond line merge. The relevant numerator is not all disjoint work. It is disjoint work that line merge rejects.

[sourced] `invent-anthropic.md`, under "Cost, slice, experiment", kills Tenure when that numerator is under 5% of same-file pairs. Its Slots experiment instead uses 5% of same-file conflicts. Those denominators are different. `invent-xai.md` proposes Stencil for keyed contributions and Loom for admitted edits. This replay tests structural merge opportunities, not admission, stale-read fencing, or lost-update prevention.

## Artifacts and rerun

The executable experiment is `/home/factory-user/orrery/explore/research/agentic-ux/same-file/replay/replay.py`.

The full evidence is `/home/factory-user/orrery/explore/research/agentic-ux/same-file/replay/results.json`. It contains every merge and exclusion, every file pair, blob IDs, changed byte ranges, region IDs, conflict excerpts and source lines, and splice outcomes.

Run the following command.

```sh
python3 /home/factory-user/orrery/explore/research/agentic-ux/same-file/replay/replay.py
```

The script requires Python 3, Node, npm, Git, and network access for its first run. It clones missing repositories and installs the pinned TypeScript parser under `/tmp/same-file-replay/`. It reads `/tmp/kvarm` only as a clone source. It never checks out or edits corpus source files.

For the fixtures alone, use `--self-test`. For a smaller exploratory run, use `--only kvarm` or `--max-merges 20`, with a separate `--output` path. Those options change the corpus.

[measured] Thirteen fixtures pass. They cover adjacent declarations, class members, declaration conflicts, imports, objects, arrays, duplicate keys, UTF-8 byte offsets, TSX, add/add files, clean merges, empty-base insertion hunks, and parser failure. Every real parent diff also passes an exact-byte reconstruction assertion, both as edits and as a one-parent region splice.

[measured] Two final full runs produce byte-identical JSON, checked with `cmp`. The output SHA-256 is `8185a5c5e050a6b8a7ca85525616d7aff00ea5fcd154eb4172dfa77c352b4064`. The recorded runtimes are Python 3.12.3, Node v24.21.0, Git 2.43.0, and TypeScript 5.9.3. The JSON records Python, Git, and the parser version.

## Corpus and pinned revisions

| Repository | Pinned commit | Reachable commits | Reachable merges | Eligible merges | Merges with pairs |
| --- | --- | ---: | ---: | ---: | ---: |
| Kvarm, copied from `/tmp/kvarm` | `c80253b14bbf2c16d67461f6e4dbe58b2e1d98e4` | 125 | 25 | 19 | 7 |
| `honojs/hono` | `5ca49b0720755c585c9a9835f0a479ceea36fb52` | 2,889 | 86 | 86 | 40 |
| `trpc/trpc` | `6d92747fa267c51829ec99242be15e0ed968d0de` | 4,963 | 337 | 336 | 104 |
| Total | | 7,977 | 448 | 441 | 151 |

[measured] The public sources are `https://github.com/honojs/hono.git` and `https://github.com/trpc/trpc.git`. Their clones are `/tmp/same-file-replay/hono` and `/tmp/same-file-replay/trpc`. The Kvarm copy is `/tmp/same-file-replay/kvarm`.

[measured] This is a census of reachable merge commits at those pins, not a last-N sample. Six Kvarm merges have unrelated parents and no merge base. One tRPC merge, `773a12069ab6868186b6713adf9f678fda776957`, has two merge bases. Those seven merges are excluded rather than inventing a base or approximating Git's recursive virtual base.

[measured] The contributing merge dates span October 13, 2026 for Kvarm, February 7, 2022 through July 16, 2024 for Hono, and August 19, 2020 through February 3, 2024 for tRPC. The public tips are from October 2026. Recent tip commits therefore do not make this a recent-merge sample.

## Method

1. Enumerate all merge commits reachable from each pinned commit. For each two-parent merge M, obtain P1, P2, and every merge base. Continue only with one base B.
2. Intersect the paths changed from B to each parent. Include `.ts`, `.tsx`, and `.d.ts` files, tests, generated files, additions, and deletions. Disable rename detection so the unit remains a path. A pair is one M and one path, not an arbitrary pair of sibling commits or a pair of adjacent commits.
3. Load B, P1, P2, and M by blob ID. A missing file has empty bytes for content replay. Preserve its add or delete status separately. Run `git merge-file --stdout --diff3` with P1, B, and P2. Any conflict exit code counts as a line conflict.
4. Parse B, P1, and P2 with the TypeScript compiler API. Partition B into top-level declarations and class members. Class headers and closing text belong to a class-shell region. Leading trivia belongs to its following statement. Imports share a commons bucket. Other module-level statements share another commons bucket. Remaining trivia and new boundary insertions have commons buckets.
5. Compute line diffs with Python `difflib.SequenceMatcher`, then refine each changed block at byte granularity. Map changed base spans and insertion anchors to B's regions. Count all changed bytes, including whitespace. A shared declaration takes precedence over a shared commons bucket. No shared region means region-disjoint. A syntax failure becomes unknown, not disjoint.
6. For disjoint pairs, reconstruct the affected whole region intervals from each parent and splice them onto B. Adjacent affected intervals from one parent can form one replacement. Verify the resulting file parses. Compare it with M as identical, whitespace-only, or different. Whitespace-only comparison uses TypeScript scanner tokens and preserves comment and literal token text. It is not a behavioral equivalence test.
7. Separately inspect conflicting hunks for Slots opportunities. A conservative candidate must put every conflicting edit inside an import block or a keyed object or array literal. Both sides must preserve every existing entry and add nonempty, disjoint key sets. Reject spreads, unknown keys, duplicate keys, mutations, and ambiguous base locations. Imports use module, import kind, imported name, local name, type-only status, and attributes. Arrays accept literal values, literal first call arguments, or literal `id`, `key`, `name`, or `path` fields.

[measured] All 1,137 parent triples parse. There are no parser or UTF-8 exclusions. The 772 conflict hunks include 18 whose base location remains ambiguous. Those hunks cannot qualify for Slots. Empty base hunks qualify only when unique text in both parents maps to the same insertion anchor.

[inferred] The region overlap classes describe the whole file pair. They are not a claim that every hunk touches the shared declaration. Slots classification instead inspects the conflicting hunks. A file can overlap in a declaration yet have all its actual conflicts confined to imports.

## Results

The percentages in the disjoint-conflict column use all same-file pairs. The percentages in the Slots column use line-conflicting pairs.

| Repository | Same-file pairs | Line conflicts | Disjoint and conflicting | Commons-only conflicts | Declaration conflicts | Slots candidates |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Kvarm | 17 | 10, 58.82% | 0, 0.00% | 6 | 4 | 3, 30.00% |
| Hono | 173 | 57, 32.95% | 0, 0.00% | 8 | 49 | 2, 3.51% |
| tRPC | 947 | 541, 57.13% | 2, 0.21% | 370 | 169 | 4, 0.74% |
| Total | 1,137 | 608, 53.47% | 2, 0.18% | 384 | 222 | 9, 1.48% |

[measured] Of the 608 conflicts, 0.33% are region-disjoint, 63.16% overlap only commons, and 36.51% overlap a declaration. Commons does not mean imports. Top-level test calls, route registrations, export statements, and other executable module statements also live there.

[measured] Across all pairs, 134 are region-disjoint, 613 overlap only commons, and 390 overlap a declaration. Line merge already accepts 132 of the 134 disjoint pairs, or 98.51%.

| Repository | Splices built | Parse successfully | Identical to M | Whitespace-only difference | Different from M | Exact or whitespace match |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Kvarm | 4 | 4 | 4 | 0 | 0 | 100.00% |
| Hono | 49 | 49 | 46 | 0 | 3 | 93.88% |
| tRPC | 81 | 81 | 74 | 0 | 7 | 91.36% |
| Total | 134 | 134 | 124 | 0 | 10 | 92.54% |

[measured] No eligible splice is refused or fails to parse. Both line-conflicting splices differ from M. The other eight mismatches also occur with ordinary clean line merge. Recorded merge output is therefore not always the automatic byte composition, even when line merge succeeds.

[measured] The nine Slots candidates include six import cases, three array cases, and two object cases. Two pairs contain more than one kind, so these counts do not sum to nine. All three Kvarm candidates are tests. Hono contributes both a source file and its generated Deno counterpart from one merge.

[measured] A looser detector finds 161 conflicting pairs, or 26.48%, whose conflicting edit footprints lie within imports or array or object literals. Only nine pass the additive-key test. The loose count includes import removals and mutations, ordinary data objects, and test fixtures. It is not evidence that 26.48% are safe keyed merges. The strict count is conservative because a clean mutation elsewhere in the same container can disqualify an otherwise mergeable addition.

### Sensitivity to file lifecycle and duplicate observations

[measured] Restricting the corpus to paths modified on both sides gives 853 pairs and 344 line conflicts, or 40.33%. It leaves one disjoint line conflict, or 0.12% of pairs. All nine Slots candidates remain, or 2.62% of conflicts. Tenure stays below 5% without deletion or add/add pairs.

[measured] Deduplicating by repository, path, base blob, and unordered parent blobs removes no observations. This does not make the observations independent. Related release merges and generated copies still share work.

## Sample conflicts

The following file lines refer to the named base commit, not the current tip. Full parent SHAs and hunk excerpts are in `replay/results.json`.

### Slots candidate in a registry list

[measured] Kvarm `packages/ops/test/registry.test.ts:56` at B `e7a43dab33ef1f76eafd729842835db4dac7a540` conflicts in M `b5dc94434f612eada88b4777d979132be60b34b1`. P1 adds `environments`, `sandboxes`, and `ci` to the family list. P2 adds `provenance`. The additions have distinct literal keys and preserve the old entries. A second merge, `b126f945a10a3b652a2c1ab21c9bd2e3f0fc5d2b`, conflicts on the same test's family list at line 56.

[inferred] These are useful keyed-list examples, but they are test expectations rather than evidence that production registry writes dominate Kvarm conflicts.

### Slots candidate in an import block

[measured] tRPC `packages/server/src/adapters/fetch/fetchRequestHandler.ts:2` at B `9756de869b42d23ef9734689bf0efe93980e8789` conflicts in M `1bb51030ef8b803534bff5efa86060e7ef07322d`. P1 adds `ResolveHTTPRequestOptionsContextFn` to the HTTP import. P2 adds `getBatchStreamFormatter` and another import for `HTTPResponse` and `ResponseChunk`. The file has declaration overlap elsewhere, but its sole textual conflict is in the imports.

### A real same-member conflict

[measured] Hono `src/hono-base.ts:103` at B `8ad3b61bb4a7c7861e051ac32ad9dcc6a4ba3e8f` conflicts in M `b24f9e38da067137c0c2c4489bc93e09c0e80484`. Both sides change the Hono constructor's `this.on` implementation. P1 changes `path` from one string to a string or array and adds a path loop. P2 adds braces around the early return. Both edits belong to the same constructor region.

[inferred] Tenure would identify a shared owner decision here, not provide a disjoint splice. A finer parser or formatter might merge this particular change, but that is not the proposed declaration-granularity benefit.

### The first apparent Tenure win is a boundary insertion

[measured] tRPC `packages/server/src/http/internals/types.ts:41-53` at B `663c63ea80e2de267d8d4708f88c1d38a598ab6e` conflicts in M `19700c275f9efd4e92d3644ba9bcf11d277fb5b0`. P1 appends `ProcedureCall`, `TRPCRequestInfo`, and `ResolveHTTPRequestOptionsContextFn` after the final declaration. P2 removes that final interface and other types and imports. The byte-region model assigns the append to commons insertions and the removals to existing regions.

[measured] The splice parses, but M exactly equals P2's file. The splice instead retains P1's 544 appended bytes. This is not an observed case of two agents independently editing adjacent functions.

### The second apparent win resurrects a deleted file

[measured] tRPC `packages/server/test/dataloader.test.ts:1` at B `9e334d1f28bfbec847cc2a94029c00250d30da42` conflicts in M `b4a60f29c66ac15e1b9e1106feeaa6641d18f544`. P1 deletes the file. P2 appends a `maxBatchSize` regression test after its final line. The splice parses as 1,097 appended bytes without the deleted base content. M deletes the file.

[inferred] A production coordinator must treat file deletion as a file-lifecycle decision, not accept this content-only splice. Counting both apparent wins makes 0.18% a generous estimate of demonstrated Tenure benefit. Neither matches the actual merge.

## What limits the number

[measured] tRPC supplies 947 of 1,137 pairs. Its single merge `a7769cb5434166224914eceb875ea1f9f06a69b9` supplies 238 pairs, or 20.93% of the corpus. The census therefore weights large release and maintenance merges heavily. It is not a representative sample of individual development tasks.

[measured] Tests and generated files remain in the primary denominator. Hono's two strict import candidates include a generated counterpart. Deletions and additions also raise the observed conflict rate. The modified-both sensitivity removes those lifecycle effects without changing Tenure's outcome.

[inferred] The structural granularity is conservative. All module-level expression statements share commons. All imports share commons. Interface members and nested functions are not independent regions. Distinct new declarations at boundaries share an insertion bucket. A larger structural merger could recover some of those cases, but that would test a different mechanism.

[inferred] These are recorded repository histories, not controlled concurrent-agent runs. Their author identities were not audited. Maintainers can resolve conflicts before a merge, rebase away the original parents, squash the branch, or avoid same-file work. The replay cannot recover those missing task pairs or count conflicts that caused a merge to be abandoned. Hono's long interval between its latest contributing merge and its current tip makes that selection issue concrete, but it does not identify the exact workflow used.

[inferred] M is an observed resolution, not a correctness oracle. A mismatch can reflect a deliberate merge-time fix or a bad resolution. A match can still be wrong. This experiment runs syntax parsing, not typechecking, integration tests, witness checks, or dependency analysis.

[inferred] The small numerator is bounded by what line merge already accepts and by whole-declaration ownership. In the measured disjoint set, 132 of 134 pairs already merge cleanly. It is not caused by skipped parser failures, a last-N cutoff, or repeated identical blob triples. It does not measure lease churn, agent compliance, contract pins, stale reads, lost edits, or time saved.

## Verdict

### Tenure

[inferred] Stop Tenure as a merge-conflict solution under its own stage-one rule. The measured rate is 2 of 1,137, or 0.18%, against a 5% threshold. The modified-both rate is lower at 0.12%. Neither apparent win reproduces M, and neither is independent editing of two existing functions.

[inferred] Region displays may still improve advisory overlap explanations, but this corpus does not justify region ownership, leases, a region-splice publication path, or claims that disjoint regions are semantically safe. Keep Kvarm's separate workspaces, line merge, and exact-tree checks. Live agent tests would need a different justification, not a claim that this history passed Tenure's gate.

### Slots and Stencil

[inferred] Keep a small experiment for declared imports or a known keyed registry. Do not build a general contribution-file system from these numbers. The pooled strict rate is only 1.48% of conflicts, below 5%, but the detector is conservative and the loose syntactic footprint is much larger. That does not establish a universal Slots kill.

[inferred] Kvarm's local numerical gate is not tripped. Three of ten conflicts qualify, or 30%, although all three are tests and two concern the same registry-family list in different merges. A narrow driver must still demonstrate ordering rules, duplicate binding and key rejection, preservation of both contributions, and exact-tree checks. This replay classifies opportunities. It does not implement or validate the keyed fold.

[inferred] Loom's admitted-edit and stale-read guarantees remain untested. The evidence supports retaining its existing line-merge baseline, not paying for a region ledger to replace it.
