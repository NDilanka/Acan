# Same-file parallel work for agents, three mechanisms and a pick

Sources. m1, m2 and m3 are the three research files in this folder. CORE is `/home/factory-user/acan/explore/ui-v2/CORE.md`. W3 is `/home/factory-user/acan/explore/research/agentic-ux/01-synthesis-wave3.md`. Kvarm paths are under `/tmp/kvarm` at revision c80253b (m2:14). A label covers the sentences after it until the next label.

## The premise I keep and the one I drop

[sourced] Converged text is not working code (m1:24). Agents edit by exact replacement, contextual patch and whole-file write (m3:16-42). A write broker prevents lost updates but cannot reconcile incompatible plans (m3:7). [inferred] So I drop shared bytes and keep the shared file as a logical object. Agents change one file concurrently in their own workspaces. The published file is assembled from parts each agent owns, and they share a map of the file, not its buffer.

## Mechanism A. Turnstile, an edit-transaction broker

**Pitch.** [inferred] Every write to a shared checkout passes through one local Acan broker as a transaction against the revision the agent read. A fresh transaction applies. A stale one is rebased with three-way merge or refused with the exact hunks that moved. Whole-file writes become diffs against their read revision, so they cannot silently revert a peer.

**Data shape.**

```ts
type FileHead = { path: string; rev: BlobSha; seq: number }
type EditTxn = {
  txnId: string; agent: PrincipalId; tool: 'replace' | 'patch' | 'write'
  path: string; readRev: BlobSha; body: ReplaceOps | Patch | FullText
  state: 'pending' | 'applied' | 'rebased' | 'refused'
}
```

**Protocol.** [inferred] Claude Code and Droid reach the broker through an MCP edit tool or a pre-tool hook that redirects native edits. A `readRev` that differs from `FileHead.rev` runs line merge3 with `readRev` as base. A clean rebase applies, and a conflict refuses. Acan snapshots the checkout only between transactions and pushes it to one Kvarm workspace for exact-tree checks. Transactions apply whole, so a crash leaves no partial write, and offline changes nothing. [sourced] Codex is passive with no MCP in CORE (CORE:46), so it must stay in a worktree.

**Safety.** [inferred] It removes lost updates. It does not stop an agent from reading a peer's half-finished code and repairing it away (m3:46). Checks still meet mixed states between agents' intents. Shell writes bypass it unless the Droid sandbox denies the path (W3:32).

**Plant UX.** [inferred] The file shows a ticker of applied and refused transactions. The developer has nothing to decide, and that is the weakness.

**Cost and experiment.** [guess] Two to three weeks in Acan, no Kvarm change. [inferred] Force two agents to edit disjoint functions 50 times. Turnstile survives only if lost edits reach zero and composed-tree check failures stay at the worktree baseline. It serves the lost-update class only.

## Mechanism B. Slots, keyed contributions inside a file

**Pitch.** [inferred] Hotspot regions of a file, such as an import block, a route table or a registry object, are declared slots. Agents edit the slot normally. At merge time Kvarm parses each side's slot into keyed entries, merges them by key, and renders them in a declared order. Duplicate keys with different payloads conflict. This is m3's keyed append-merge driver (m3:102), made a declared object on the plant.

**Data shape.**

```ts
type Slot = {
  slotId: string; path: string
  anchor: { query: TreeSitterQuery } | { markers: [begin: string, end: string] }
  keyOf: 'import-specifier' | 'object-key' | 'call-arg-0'
  order: 'by-key' | 'by-precedence' | 'locked'
}
type SlotEntry = {
  slotId: string; key: string; payloadHash: string; precedence?: number
  source: { workspaceId: string; head: CommitSha }
  state: 'proposed' | 'accepted' | 'duplicate' | 'refused'
}
```

**Protocol.** [inferred] Agents add no step. The train and `compose_candidates` call a slot driver. [sourced] Today's hunk callback can only return ours, theirs or null (`/tmp/kvarm/packages/gitops/src/merge3.ts:91`), so the driver needs the full base, ours and theirs blob hook m2 names (m2:72). [inferred] An `order: 'locked'` slot refuses concurrent inserts and routes them to Serialize.

**Safety.** [sourced] Git's union driver can leave lines in random order (m3:111). [inferred] Slots keep order explicit and reject duplicates before exact-tree checks. They cannot know about undeclared route precedence.

**Plant UX.** [inferred] A slot is one rack of keyed entries in contributor colors. A duplicate key is a clash pin.

**Cost and experiment.** [guess] Two weeks for TypeScript imports and object-literal registries. [inferred] Replay Kvarm history and count merge3 conflicts whose hunks lie entirely inside slot queries. Kill it if that is under 5% of same-file conflicts. It serves registry, import and barrel collisions and ordered routes with explicit precedence.

## Mechanism C. Tenure, a region ledger with contract pins (the pick)

**Pitch.** [inferred] Kvarm keeps a ledger per shared file that partitions it into regions by syntax node. Each region has at most one tenant, held by a fenced claim. A region also records which contracts, such as a helper's signature, other tenants rely on. Agents edit in their own workspaces with their normal tools. Kvarm accepts a checkpoint only when the diff stays inside the tenant's regions. The train splices each region from its tenant's head, so disjoint work never produces a text conflict, even on adjacent lines. A contract change that another tenant relies on becomes a clash pin before it lands. Slots handle the commons between regions.

### Data shape

```ts
type FileLedger = {
  repo: RepoId; fileId: string            // follows git rename detection
  basePin: { commit: CommitSha; blob: BlobSha }
  grammar: 'typescript' | 'rust' | 'python'; parserVersion: string
  regions: Region[]                       // a partition of the file at the chosen granularity
}
type Region = {
  regionId: string                        // minted once, never recomputed from text
  kind: 'function' | 'method' | 'class-member' | 'commons' | 'slot' | 'locked-check'
  parentId?: string                       // a class region owns its member regions
  anchor: {
    qualifiedName: string                 // 'PayoutClient.retry'
    ordinal: number                       // disambiguates overloads with one name
    signatureHash: string                 // params and return type, normalized
    bodyHash: string
    spanAtBase: { startByte: number; endByte: number }
  }
  tenure: {
    state: 'free' | 'declared' | 'held' | 'checkpointed' | 'stale' | 'orphaned' | 'locked'
    owner?: { workspaceId: string; taskGeneration: number }
    generation: number                    // fenced_claims generation
    expiresAt?: string
  }
  contract: {
    exposes: string                       // signatureHash others may rely on
    reliedOnBy: { regionId: string; owner: string; signatureHash: string }[]
  }
  rev: number                             // bumps on each accepted checkpoint
  checkpoint?: { commit: CommitSha; regionBlobHash: string }
}
```

[sourced] m2 warns that independent equality-key fences cannot exclude intersecting intervals (m2:71). [inferred] Regions that partition the file never intersect, so equality keys are enough. A claim on a class region also claims every member region in the same statement. Anchors survive line drift because they name nodes, not lines. Only the owner can rename or move a region, so the ledger updates `qualifiedName` from the owner's checkpoint and keeps `regionId`.

[inferred] `set_intent` moves a region from `free` to `declared`, and `claim_regions` moves it to `held`. A push that passes scope validation makes it `checkpointed`. Lease expiry makes it `stale`. A region that no longer parses or matches becomes `orphaned`. `locked` is permanent for witness-derived check regions such as a Rust `mod tests` block, because agents never change their own checks (CORE:12).

### Protocol

1. **Read.** [inferred] The agent's readback lists the regions it will write and the regions it relies on. Acan fills this from a new `get_file_ledger(path)` op. Codex, which cannot call MCP (CORE:46), gets the list in its prompt and is validated only at push.
2. **Declare.** [inferred] `set_intent` gains a `regions` field with `{ fileId, regionId, mode, signatureHash }`, where mode is write or rely. It stays separate from `paths`, whose prefix matching would break on `file#symbol` (m2:61). `forecast` reports two new clash kinds. Two writes on one region is `region:write`. A write that declares a signature change on a region another tenant relies on is `region:contract`. Both appear on the belt before code, like CORE's clash step (CORE:15).
3. **Claim.** [inferred] `claim_regions` reuses `fenced_claims` with kind `region` and subject `repo/fileId/regionId`, all or none in one conditional statement. [sourced] `claimFenced` lets a different `identity` take over a live claim (`/tmp/kvarm/apps/platform/src/services/fencing.ts:78-81`). [inferred] So the identity is the ledger's `basePin.blob`, which is constant for every claimant, and order and task generation go in `owner`. A `claim_task` takeover releases the old owner's regions explicitly.
4. **Edit.** [inferred] Claude Code `Edit`, Codex `apply_patch` and Droid `Edit` or `Create` work unchanged in the agent's own worktree. An optional Claude Code or Droid pre-tool hook warns locally when an edit leaves held regions.
5. **Checkpoint.** [sourced] Push reconciliation already computes workspace paths (m2:20). [inferred] For ledger files it also parses base and head with bounded tree-sitter WASM and maps changed bytes to regions. In-tenure changes bump `rev`. Out-of-tenure changes refuse the checkpoint with `OUT_OF_TENURE` and the region list, though the push still lands on the agent's own ref. Whitespace-only spill is dropped, reusing the train's whitespace rule (m2:25). A new top-level definition becomes a region owned by its author, placed after a declared anchor, only where order is unimportant, as in Mergiraf's order-insensitive contexts (m1:56). Imports and other module-level lines are commons, handled as a Slot or by one owner.
6. **Refresh.** [inferred] When Droid checkpoints `retry` at rev 3, Kvarm offers Claude Code `refresh_regions`. Kvarm writes one commit to Claude Code's workspace ref that splices the new `retry` in, with expected-old protection. It cannot conflict, because Claude Code holds no tenure there. If Claude Code relies on `retry` and its `signatureHash` changed, the refresh carries a stale-read notice with the region, both hashes and the commit. This fills m1's causal feedback gap (m1:175).
7. **Integrate.** [inferred] The train and `compose_candidates` get a splice driver. Start from the base blob and replace each region span with the tenant's checkpointed region bytes. Commons use the Slot driver or the commons owner. Exact-tree checks then run as today, and disjoint regions never justify reusing checks (m2:82-83). `compose_candidates` gains a `regions` selector next to `paths`.

**Conflict, stale, crash, offline.** [inferred] A losing claim returns `CLAIMED` with `retryAfterMs` (fencing.ts:92), and the plant shows a pin. A `region:contract` break at checkpoint holds that checkpoint as `contract-changed` until settled. A tenant whose lease lapsed gets `STALE_GENERATION` on checkpoint, as `reportFenced` already refuses (fencing.ts:112-131). Its commits survive on its ref, and the next tenant starts from the last accepted `rev`. Offline, an agent keeps editing its worktree, its regions read `MATCHED · claim waits for connection` (CORE:84), and its edits stay ghosts until reconnect validates them.

### Safety argument

[inferred] One writer per region removes lost updates and interleaving inside a region. Splicing whole syntax nodes into a file that parsed at base yields a file that parses, if each region parses alone and commons merge cleanly. The driver re-parses the result and refuses on error. Contract pins catch declared signature dependencies inside the file. Compilation and behavior are tested, not proven, because the composed tree passes through the same exact-tree and witness-derived checks as any change.

[inferred] It cannot guarantee four things. Changed behavior behind an unchanged signature passes the contract check. Shared module state, macros and reflection bypass region identity. Files that do not parse fall back to one owner for the whole file. Formatters that rewrite the whole file spill out of tenure, so formatting runs once on the composed tree.

### Plant UX

[inferred] A ledger file appears in machine cutaway (X, W3:113) as a layer like CORE's "effects queue rows" (CORE:47). The layer is a strip of bays in file order, one bay per region and labeled `PayoutClient.retry · r3`. Each bay takes its tenant's color. `stale` bays are dimmed with age, `orphaned` bays are fog hatch, and `locked` bays show a padlock and `K16`, using W3's five states (W3:89). Commons and slots draw as a narrow rack at the top, and thin lines show rely edges.

[inferred] Keys come from CORE's free set (CORE:27). V toggles declared footprints, so a proposed tenure shows as a translucent outline over its bays. I jumps to the next unsettled region pin. Shift 1 to Shift 4 settles from an open card. U opens Waiting on you, as in W3 (W3:116).

[inferred] In the ten-second path, a brass pin on `retry` reads `Claude Code and Droid both write retry`. The developer presses I. The card shows both declarations and the consequence `breaks w3 · p-2307 stays pending` when a witness is at stake. Shift 1 makes Droid the one owner and Claude Code a reviewer. Shift 2 splits the region, so Droid extracts `retryBudget` as a new region. Shift 3 serializes Claude Code after Droid. Shift 4 accepts the new contract and sends Claude Code a refresh. Each choice logs `record_decision` and a narrowing `set_intent`, as clash settlements already do (CORE:89).

### Cost, slice, experiment

[guess] Kvarm upstream needs four to six weeks for TypeScript. The work is two additive migrations (`file_ledgers`, `ledger_regions`, hashes and spans only), three ops (`get_file_ledger`, `claim_regions`, `refresh_regions`), a `regions` field on `set_intent` and `forecast`, push-time scope validation, and the splice driver behind the blob hook. [sourced] D1 already stores imported names (m2:51), and per-file analysis caps at 256 KiB (m2:53). [guess] Acan needs two to three weeks for the bay strip, pins and readback region lists.

[inferred] The smallest shippable slice needs no upstream change. Acan parses TypeScript locally and keeps an advisory ledger. It maps each workspace diff from `get_file_diff` (m2:107) onto regions, splices disjoint workspaces locally, and pushes the spliced result as one workspace into the normal train. The tenure is unenforced, so the plant labels it `advisory tenure`.

[inferred] The experiment has two stages. Stage one replays history. [sourced] `git log` shows 125 commits in `/tmp/kvarm`, and `wc -l` gives `apps/platform/src/services/queue.ts` 2,787 lines. [inferred] For every pair of sibling commits that touch one TypeScript file, compute whether their changes are region-disjoint and whether line merge3 conflicts on them. Stage two runs live. Choose 30 task pairs on `queue.ts` and `tasks.ts` that target different functions. Run Claude Code with Codex under worktrees plus merge3, then under advisory Tenure. Tenure is proven if four things hold. Lost edits are zero, so each agent's region bytes appear unchanged in the composed tree. Region-disjoint pairs have zero text conflicts. Composed-tree check failures do not exceed the baseline. Wall time beats serial work by at least 25%. Tenure is killed if out-of-tenure refusals exceed 20% of checkpoints, if the splice ever emits an unparseable tree, or if stage one finds line conflicts on region-disjoint pairs in under 5% of same-file pairs.

### Collision classes

[inferred] Against m3's taxonomy (m3:100-109), Tenure serves disjoint functions in a cohesive file and independent edits on adjacent lines in different regions. It also stops lost updates from stale whole-file writes, which arrive as out-of-tenure changes and are refused. Registry, import and barrel additions go to Slots, and ordered routes to a locked slot and Serialize. Two agents on one function's algorithm become a `region:write` clash, settled as one owner plus a reviewer. Cross-file coupling goes to CORE's Split at the port, lockfiles and generated files to one producer, and migrations to one ordering owner.

## Pick and why

[inferred] I pick Tenure with Slots as its commons rule. Turnstile fixes mechanics and leaves meaning untouched (m3:127). Slots alone cover only hotspot entries. Tenure makes same-file parallel work a planned object. Ownership is decided before code, on the plant, with the Settle step CORE already teaches. Agent tools stay unchanged, and Kvarm's exact-tree train stays the judge.

[inferred] Tenure borrows definition identity from Unison (m1:94) and "many may edit, one may submit" from Perforce `p4 lock` (m1:112). Locks plus weak-dependency awareness come from CoEclipse (m1:116), durable conflict objects from Jujutsu (m1:84), and generation fencing from Kvarm's `fenced_claims` (fencing.ts:69-93).

[inferred] Four things are new. CodeCRDT shares a Yjs buffer and claims TODOs with a 50 ms wait and reread (m1:128-134). Tenure shares no bytes and fences every checkpoint by generation. Kvarm, not the agent, validates scope. Mergiraf matches unknown edits heuristically after the fact (m1:54). Tenure's splice selects regions whose owners were fixed before code, so it needs no matching. Jujutsu stores conflicts once they exist. Tenure stores the contracts between regions and raises a contract break as a clash before the conflicting code lands. Region-granular refresh with a causal stale-read notice has no counterpart in the reviewed sources (m1:175).

## Two biggest risks

[guess] First, agents may not stay in their lanes. A real fix often adds an import, a helper or a shared type. Frequent `OUT_OF_TENURE` churn would cost more than a later merge. Owned new regions and the commons slot mitigate it, and the 20% kill line measures it.

[inferred] Second, the plant may look safer than the code is. Tinted bays suggest independence, but changed behavior behind an unchanged signature passes every contract check. The bay label must say `checked on tree <sha>` only after exact-tree checks pass, and `advisory` before then.
