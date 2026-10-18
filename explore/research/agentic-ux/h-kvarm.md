# Kvarm in Orrery

## Summary

[inferred] Make Kvarm the coordination and publication authority, while Orrery owns the drawing, runtime evidence, and interpretation.
[inferred] Ship all four support modes together, with explicit consent before any connection.
[sourced] Kvarm already supplies workspaces, forecasts, fenced claims, exact-tree integration, decisions, and graded provenance (`/tmp/kvarm/README.md:62-70`).
[inferred] Store the blueprint and portable choices in repository files hosted by Kvarm, and publish choice reasoning into Kvarm decisions.
[sourced] Local development without Artifacts cannot read git content (`/tmp/kvarm/CONTRIBUTING.md:18`).
[inferred] Treat remote Kvarm as declared egress, never as an exception hidden behind "local first".

## Evidence and maturity

[sourced] The inspected clone identifies its head as `c80253b14bbf2c16d67461f6e4dbe58b2e1d98e4` (`/tmp/kvarm/.git/refs/heads/main:1`).
[inferred] This is a design assessment dated 2026-10-17, not a deployment certification.
[sourced] Kvarm is a git platform for humans and agents, implemented as a Cloudflare Worker with smart HTTP, REST, MCP, and repository pages, without hosted-model calls (`/tmp/kvarm/README.md:5`).
[sourced] Artifacts stores git, D1 stores coordination metadata, and server-side ref writes use receive-pack with an expected old value (`/tmp/kvarm/README.md:40-44`).

[sourced] The MCP server reports version `0.1.0`, and the original October 7 competition plan explicitly defers to the current architecture (`/tmp/kvarm/apps/platform/src/mcp/server.ts:27`, `/tmp/kvarm/planning/PLAN.md:3-15`).
[sourced] The repository documents workerd tests with fake Artifacts, live deployment drivers, and one 100-workspace train measurement, not universal throughput evidence (`/tmp/kvarm/docs/architecture.md:249`, `/tmp/kvarm/docs/architecture.md:384-386`).
[inferred] Treat this as a young, substantially implemented dependency with a pinned compatibility contract.
[sourced] Documentation already disagrees internally, because architecture section 0.2 calls audit deferred while the admin audit route exists (`/tmp/kvarm/docs/architecture.md:22`, `/tmp/kvarm/apps/platform/src/routes/api/attribution.ts:47-67`).
[inferred] Prefer inspected routes and registry entries over obsolete roadmap claims.

## The data model

- [sourced] Workspaces have server-created refs under `refs/heads/agents/<workspaceId>/<slug>`, a base, a head, an actor, activity, and an explicit lifecycle (`/tmp/kvarm/docs/agents/workflow.md:13-21`, `/tmp/kvarm/docs/architecture.md:263`, `/tmp/kvarm/docs/architecture.md:316`).
- [sourced] Intents declare non-exclusive paths or prefixes with a lifetime, while pushes reconcile cumulative observed edits and expose work outside the declaration (`/tmp/kvarm/docs/agents/workflow.md:21`, `/tmp/kvarm/docs/architecture.md:316-318`).
- [sourced] Nexus forecasts path overlap and advisory semantic dependencies, including imports, changed exports, and contract families (`/tmp/kvarm/docs/architecture.md:321-325`).
  [sourced] Its bounded TS/JS, Python, and Go heuristic follows at most two import hops, misses aliases and generated behavior, reports truncation, and gates nothing (`/tmp/kvarm/docs/architecture.md:321-326`).
- [sourced] Changes are pull-request equivalents with `review`, `auto`, or `draft` policy, plus checks, risk, reviews, and mergeability (`/tmp/kvarm/docs/architecture.md:103-105`, `/tmp/kvarm/docs/architecture.md:185`).
  [sourced] The integration train prepares eight entries per pass, checks four concurrently, admits 64 active entries per repository, and publishes under a 30-second fenced lease against the exact composed tree (`/tmp/kvarm/docs/architecture.md:338-346`).
- [sourced] Tasks include `fix-issue` and `resolve-conflict`, with holder, generation, expiry, workspace, and resolution (`/tmp/kvarm/packages/ops/src/index.js:798-866`).
  [sourced] Claiming creates or reuses a workspace, defaults to a 600-second lease, rejects a competing live claim, and rejects stale-generation heartbeats or completion (`/tmp/kvarm/packages/ops/src/index.js:823-866`).
- [sourced] Candidates compare one to eight workspaces, and composition combines selected paths into a new workspace and change (`/tmp/kvarm/packages/ops/src/index.js:932-978`).
  [sourced] Arenas create two to eight sibling attempts from one brief, rank deliberate duplication separately, and require explicit selection without spawning agents or abandoning losers (`/tmp/kvarm/docs/architecture.md:378-380`).
- [sourced] Decisions store task, paths, chosen candidate, rejected candidates, reason, actor, and time in D1, retrievable through bounded `get_context` pages (`/tmp/kvarm/docs/architecture.md:41`).
  [sourced] Attribution notes instead attach agent, session, prompt hash, model, actor, and time to commits on `refs/notes/kvarm-attribution` (`/tmp/kvarm/docs/architecture.md:264`).
- [sourced] Evidence levels distinguish caller assertions as `declared`, authenticated submitters as `authenticated`, and trusted execution signatures as `verified` (`/tmp/kvarm/docs/architecture.md:265-289`).
  [sourced] Authentication does not prove which model ran, and prompt hashes remain opaque rather than recoverable prompts (`/tmp/kvarm/docs/architecture.md:269-271`, `/tmp/kvarm/docs/architecture.md:376`).
- [sourced] Issues carry paths, fingerprinted occurrences, evidence, sources, comments, close records, and links to changes or workspaces (`/tmp/kvarm/README.md:70`).
  [sourced] `dispatch_issue` creates one live fix task, and confirmed closing links close issues when their changes land on the default branch (`/tmp/kvarm/packages/ops/src/index.js:1345-1386`).
- [sourced] Reviews bind approvals to heads, prefer different declared model families, and open human escalations for disagreement, requested changes, protected targets, or final arena selection (`/tmp/kvarm/docs/architecture.md:36`).
  [sourced] Guided review adds content-keyed acknowledgements, findings, contract declarations, and remaining actions (`/tmp/kvarm/docs/architecture.md:39`).
  [sourced] `/inbox` groups human-held changes across accessible repositories, while `list_pending_reviews` exposes a repository-level API (`/tmp/kvarm/docs/architecture.md:362`, `/tmp/kvarm/packages/ops/src/index.js:649-655`).
- [sourced] Branch rules protect refs, require named checks and approvals, and can reject incomplete guard scans (`/tmp/kvarm/packages/ops/src/index.js:1452-1469`).
  [sourced] Check producers feed an exact-tree gate, but no configured producers means an empty successful requirement set (`/tmp/kvarm/docs/architecture.md:24-32`).
- [sourced] Push events track independently completed effects, and platform CloudEvents drive signed, unordered, at-least-once webhooks (`/tmp/kvarm/docs/architecture.md:350-354`).
  [sourced] Audit exports provenance and recorded ref updates in stable first-seen order, with 200-record pages, but ordinary push and explicit-merge recording are best effort (`/tmp/kvarm/docs/architecture.md:290`, `/tmp/kvarm/README.md:442`).
  [inferred] Neither audit nor webhook delivery history is a complete runtime timeline.

## Mapping Orrery concepts

[sourced] Orrery's current loop includes witnesses, locked checks, readbacks, self-claims, declarations, and pre-code clash settlement (`/home/factory-user/orrery/explore/ui-v2/CORE.md:9-13`).
[inferred] The following fit assessments describe the integration, not additional Kvarm guarantees.

| Orrery concept | Kvarm object or exact operation | Fit and remaining gap |
|---|---|---|
| Territory | [sourced] Workspace and `set_intent` (`/tmp/kvarm/packages/ops/src/index.js:350-378`, `/tmp/kvarm/packages/ops/src/index.js:881-892`). | [inferred] Partial fit, because enduring system ownership differs from temporary path coverage and needs territory IDs in YAML. |
| Ghost | [sourced] `create_workspace`, `create_change_set` with `draft` (`/tmp/kvarm/packages/ops/src/index.js:350-398`). | [inferred] Partial fit, because a draft can back proposed code but a predicted machine need not have code or a branch yet. |
| Order | [sourced] `create_issue`, `dispatch_issue` (`/tmp/kvarm/packages/ops/src/index.js:1287-1300`, `/tmp/kvarm/packages/ops/src/index.js:1345-1352`). | [inferred] Good dispatch fit, with issued blueprint revision, witness IDs, and readback requirements added in Orrery files. |
| Witness | [sourced] Issue `evidence` accepts commit, check, run, workspace, and URL references (`/tmp/kvarm/packages/ops/src/index.js:1298-1300`). | [inferred] Weak fit, because evidence references do not define a replayable must-keep or must-not assertion. |
| Locked check or budget, K16 | [sourced] `create_branch_rule`, `POST /api/v1/repos/:owner/:repo/check-runs` (`/tmp/kvarm/packages/ops/src/index.js:1452-1469`, `/tmp/kvarm/apps/platform/src/routes/api/checks.ts:27-40`). | [inferred] Partial fit, because ref protection is not field-level human ownership of budgets or check definitions. |
| Readback | [sourced] `comment_issue`, `record_decision` (`/tmp/kvarm/packages/ops/src/index.js:1355-1361`, `/tmp/kvarm/packages/ops/src/index.js:1253-1265`). | [inferred] Weak fit, because neither stores a typed interpretation scored against witnesses before claiming. |
| Claim | [sourced] `claim_task`, `heartbeat_task`, `complete_task` (`/tmp/kvarm/packages/ops/src/index.js:823-866`). | [inferred] Strong lease fit, but Kvarm has no readback-match prerequisite. |
| Declaration | [sourced] `set_intent`, `get_intent`, `forecast` (`/tmp/kvarm/packages/ops/src/index.js:872-910`). | [inferred] Good path fit, with machine, belt, assertion, and drawing-revision semantics added locally. |
| Clash pin and resolutions | [sourced] `forecast`, `get_overlap`, `add_dependency`, `compose_candidates`, `record_decision` (`/tmp/kvarm/packages/ops/src/index.js:412-425`, `/tmp/kvarm/packages/ops/src/index.js:896-958`, `/tmp/kvarm/packages/ops/src/index.js:1253-1265`). | [inferred] Partial fit through evidence and consequences, not the complete pre-code resolution menu described below. |
| Inspection gate | [sourced] `list_pending_reviews`, `get_reviews`, `request_review` (`/tmp/kvarm/packages/ops/src/index.js:593-600`, `/tmp/kvarm/packages/ops/src/index.js:649-674`). | [inferred] Good backlog fit, provided attention forecasts remain advisory rather than silently altering publication rules. |
| Bench | [sourced] `create_arena`, `rank_arena`, `select_arena_winner` (`/tmp/kvarm/packages/ops/src/index.js:980-1030`). | [inferred] Partial fit through attempts, without experiment funding, falsification criteria, or runtime replay. |
| Choices ledger | [sourced] `record_decision`, `get_context` (`/tmp/kvarm/packages/ops/src/index.js:1253-1281`). | [inferred] Strong reasoning fit, with stable choice IDs, confidence, reversal cost, and revision links added in YAML. |
| Replay | [sourced] `list_events`, `get_path_history`, `get_commit_diff`, `get_attribution` (`/tmp/kvarm/packages/ops/src/index.js:254-280`, `/tmp/kvarm/packages/ops/src/index.js:333-345`, `/tmp/kvarm/packages/ops/src/index.js:1243-1250`). | [inferred] Strong delivery-history fit, while runtime history requires a join to local telemetry by commit and session. |
| Return brief | [sourced] `get_train`, `list_pending_reviews`, `get_context` (`/tmp/kvarm/packages/ops/src/index.js:581-588`, `/tmp/kvarm/packages/ops/src/index.js:649-655`, `/tmp/kvarm/packages/ops/src/index.js:1269-1281`). | [inferred] Good inputs for Orrery to compute changes since departure and consequences, rather than copy a timeline. |
| Fog | [sourced] Forecast freshness and truncation, attribution evidence, and check completeness (`/tmp/kvarm/docs/architecture.md:30`, `/tmp/kvarm/docs/architecture.md:265-273`, `/tmp/kvarm/docs/architecture.md:318-326`). | [inferred] Partial fit that qualifies evidence without supplying all six Orrery fog signals. |
| Floor sessions | [sourced] Workspace actor and attribution session, plus `list_ci_jobs` (`/tmp/kvarm/docs/architecture.md:264`, `/tmp/kvarm/docs/architecture.md:316`, `/tmp/kvarm/packages/ops/src/index.js:1621-1627`). | [inferred] Partial fit, because a workspace is not a live process and CI claims are not agent heartbeats. |
| Blueprint YAML | [sourced] `read_file`, `get_tree`, and git receive-pack (`/tmp/kvarm/packages/ops/src/index.js:228-250`, `/tmp/kvarm/docs/architecture.md:76-89`). | [inferred] Strong repository storage fit that leaves YAML validation, drawing diffs, and comment-preserving edits with Orrery. |

## Four integrations, one explicit connection

### 1. Draw the work Kvarm records

[inferred] Connect one declared origin and repository, validate with `get_repo`, and cache objects by origin, repository, object ID, and revision.
[inferred] Read `list_workspaces`, `list_changes`, `list_tasks`, `get_train`, `list_pending_reviews`, and `get_overlap` on connection and reconciliation.
[inferred] Fetch selected-object details through `get_workspace`, `get_intent`, `get_change`, `get_reviews`, `list_checks`, `compare`, and `get_file_diff`.
[sourced] These tools expose workspace heads, checks, reviews, diffs, and waiting reasons (`/tmp/kvarm/packages/ops/src/index.js:303-345`, `/tmp/kvarm/packages/ops/src/index.js:366-378`, `/tmp/kvarm/packages/ops/src/index.js:489-600`, `/tmp/kvarm/packages/ops/src/index.js:872-878`).

[inferred] Join paths to blueprint entities without inventing runtime ownership.
[inferred] Show unmapped paths in the tray, label declared versus observed footprints, and retain incomplete or stale markers.
[sourced] Session observation still prioritizes native OTel, then opt-in hooks, then transcripts with drift warnings (`/home/factory-user/orrery/explore/map.md:202-218`).
[inferred] Join that local stream to Kvarm attribution by session and commit, never by a guessed model name.

[sourced] There is also an outbound-client-friendly `GET /api/v1/repos/:owner/:repo/live` WebSocket and `list_repo_events` using `GET .../events?since=<seq>` (`/tmp/kvarm/apps/platform/src/routes/api/events.ts:14-45`, `/tmp/kvarm/packages/ops/src/index.js:263-269`).
[sourced] The live feed retains only 200 events, and emission is best effort (`/tmp/kvarm/apps/platform/src/do/repoEvents.ts:19-20`, `/tmp/kvarm/apps/platform/src/do/repoEvents.ts:66-76`, `/tmp/kvarm/apps/platform/src/services/live.ts:12-32`).
[inferred] Use explicit `since=0` initially, because omitting `since` selects the separate push-event listing.
[inferred] Start with cursor polling plus periodic snapshots, and treat this feed as invalidation rather than durable Replay.

### 2. Write orders and settlements through Kvarm

[inferred] Posting creates an Orrery order revision, then `create_issue` with its ID, paths, witness references, and a fingerprint, followed by `dispatch_issue`.
[inferred] Managed agents read `get_context`, submit a revision-bound readback to Orrery, and call `claim_task` themselves after matching checks.
[inferred] Preserve the returned generation and workspace, heartbeat every 120 seconds for a 600-second lease, and stop on `STALE_GENERATION`.
[sourced] Kvarm supplies the lease and workspace, but not that readback gate (`/tmp/kvarm/packages/ops/src/index.js:823-866`).
[inferred] Label raw Kvarm claims without readbacks as unmatched rather than retroactively claiming compliance.

[inferred] Agents call `set_intent` and `forecast` before editing, push to their returned Kvarm workspace ref, open `create_change_set`, then inspect `get_mergeability`, `list_checks`, and `get_reviews`.
[inferred] Complete fix tasks with `complete_task` after opening their change, and enqueue with both expected `headSha` and `mergePolicy`.
[sourced] Kvarm rejects changed heads or policies, and the train rechecks the composed tree (`/tmp/kvarm/packages/ops/src/index.js:547-559`, `/tmp/kvarm/packages/ops/src/index.js:853-866`).
[inferred] Observe `get_train` and `get_change` until publication, with `merge_change` reserved for an explicit human action.
[inferred] After an ambiguous write timeout, inspect the resulting object before retrying.

[sourced] Orrery offers split at the port, either side winning, serialize, one migration owner, and merged territories before code (`/home/factory-user/orrery/explore/ui-v2/CORE.md:59-66`).
[inferred] Persist each settlement as a choice and `record_decision`, then update affected intents.
[inferred] A winning side makes the other replan, and one migration owner removes that path from other declarations.
[inferred] Serialization records an order dependency first, then calls `add_dependency` only when an exact upstream revision exists.
[sourced] Kvarm pins revisions, not hypothetical future work, and exposes stale or broken pins explicitly (`/tmp/kvarm/docs/architecture.md:330-333`).
[inferred] A port split publishes an Orrery contract file before dependent code, because Kvarm contract capsules remain deferred.
[inferred] Merging territories changes blueprint ownership, while `compose_candidates` applies only when actual compatible code pieces exist.
[sourced] Kvarm composition and deferred capsules are distinct mechanisms (`/tmp/kvarm/docs/architecture.md:40`, `/tmp/kvarm/docs/architecture.md:382`).

### 3. Drive Replay and the return brief

[inferred] Fetch git history from the Kvarm remote for commit ancestry, and combine `list_events`, `get_commit_diff`, `get_attribution`, `get_context`, and cached state transitions.
[sourced] `get_path_history` indexes default-branch first-parent history and reports incomplete indexing or gaps, rather than all workspace ancestry (`/tmp/kvarm/docs/architecture.md:398-407`).
[inferred] Preserve event time separately from first-seen and local-receipt time.
[inferred] Record the engineer's departure watermark locally and reconcile on return.
[inferred] Build the brief around landed outcomes, broken witnesses, changed decisions, unresolved clashes, stale pins, and human decisions now required.
[inferred] Attach source object IDs and exact revisions to each brief item.
[inferred] Use `get_audit` only for an explicitly authorized provenance inspection, not as a mandatory admin credential for ordinary Replay.
[inferred] Missing audit records and live-feed gaps must remain visible, while local runtime Replay continues independently.

### 4. Keep the blueprint and choices in Kvarm

[sourced] Orrery already requires repository YAML under `.orrery/`, with separate system, layout, territory, choice, machine, and protected check files (`/home/factory-user/orrery/explore/map.md:44`, `/home/factory-user/orrery/explore/map.md:297`).
[inferred] Keep that layout in the Kvarm-hosted repository and publish edits through ordinary commits, Kvarm changes, and the train.
[inferred] Add small versioned order, witness, and readback documents that reference stable blueprint IDs.
[inferred] Store portable, sanitized witness fixtures in the repository, but retain raw traces, transcripts, credentials, caches, and sensitive Replay payloads only locally.

[inferred] Make choice YAML canonical for confidence, reversal cost, supersession, and drawing references, and publish chosen and rejected alternatives into `record_decision`.
[inferred] Include the canonical choice path, immutable commit, and stable ID in the decision reason.
[inferred] Use `get_context` to discover choices for agents, then load the referenced file for complete semantics.
[inferred] This mix preserves offline portability while giving Kvarm first-class reasoning, rather than merely hosting YAML.
[sourced] Decision recording creates random IDs without an input idempotency key (`/tmp/kvarm/apps/platform/src/services/decisions.ts:157-193`).
[inferred] Track pending publication locally, reconcile by stable choice identity before retry, and display file-versus-decision divergence.
[inferred] Do not claim atomicity between a git commit and a D1 decision.
[sourced] Attribution has a fixed provenance shape, so it is not the general choices or runtime-evidence store (`/tmp/kvarm/packages/ops/src/index.js:1225-1239`).

[inferred] Enforce K16 with an evaluator owned by Orrery or a pinned human-owned runner, not evaluator code selected by the candidate branch.
[inferred] Compare candidate budgets, checks, and CI configuration against the last human-approved definition digest.
[inferred] Reject changed definitions unless a human approves their exact digest and candidate head.
[inferred] Configure a required `orrery/locked-definitions` check through `create_branch_rule`, and report only through the separate trusted `ci` credential.
[inferred] Agent edits remain proposals until that approval, never live changes to their grading target.
[sourced] Protected-branch human approval counting excludes the change's recorded author user (`/tmp/kvarm/apps/platform/src/services/reviewGate.ts:153-154`).
[inferred] Validate the solo owner's workflow rather than assuming the human who minted an authoring agent can satisfy an independent-reviewer requirement.

### Polling and rate budgets

[sourced] Limits are 600 reads and 120 writes per minute per token, and a git push uses about two writes (`/tmp/kvarm/README.md:433-440`).
[inferred] Cap Orrery at 480 reads and 90 writes per token per minute, across all windows and repositories.
[inferred] For ten active repositories, six state reads every 30 seconds cost 120 reads per minute, and cursor polling every five seconds costs another 120.
[inferred] The remaining 240-read budget covers pagination, selected checks, forecasts, and immutable details.
[inferred] Debounce declaration forecasts by one second, poll selected checks at most every five seconds, and refresh decisions on relevant actions or return.
[inferred] Back off idle repositories to 60 seconds and disconnected repositories exponentially.
[inferred] Reserve 30 writes for leases and 60 for actions, counting pushes twice.
[inferred] Twenty active task leases need ten heartbeats per minute at the proposed cadence.
[inferred] Pause new dispatch before starving an existing lease.
[inferred] Honor `Retry-After`, add jitter, and expose backlog or truncation instead of exceeding budgets.
[inferred] These are scheduling choices, not measured latency promises.

[sourced] Webhook targets must be public HTTPS rather than localhost or private addresses (`/tmp/kvarm/docs/architecture.md:353`).
[inferred] Do not register a desktop webhook, expose a port, or silently introduce a relay.
[inferred] All four modes use outbound polling, while an optional WebSocket can later reduce invalidation latency without replacing reconciliation.

## Local-first honesty and permissions

[sourced] Q16 requires an outbound destination, byte, and reason log, a hard offline switch, and a published data statement (`/home/factory-user/orrery/explore/map.md:230`).
[sourced] V6 keeps update checks opt-in and starts the egress pill at "0 out" (`/home/factory-user/orrery/explore/proposals/ui-v1.md:14`).
[inferred] Kvarm connection must therefore be opt-in, naming the user's own origin and what leaves the device before the first request.
[inferred] Once connected, the pill must show actual egress, even when the destination belongs to the user.
[inferred] Log API calls, git subprocess traffic, WebSocket traffic, redirects, and retries without logging tokens or bodies.
[inferred] Keep local telemetry export separate from sanitized evidence publication.

[sourced] `wrangler dev -c wrangler.dev.jsonc` can serve local routing, authentication, D1-backed metadata, assets, and content-independent pages (`/tmp/kvarm/README.md:249-264`, `/tmp/kvarm/CONTRIBUTING.md:18`).
[sourced] Artifacts is remote-only, and normal development with that binding requires Cloudflare credentials (`/tmp/kvarm/README.md:264`).
[inferred] Local D1 and fake Artifacts tests do not constitute a self-contained local git platform.
[inferred] Full Kvarm operation therefore needs its Cloudflare services unless Kvarm adds another storage implementation.

[inferred] Offline Orrery keeps local files, cached Kvarm history, local agents, OTLP, and MCP working.
[inferred] Mark remote claims, checks, queue positions, and permissions stale.
[inferred] Permit drafts and local commits, but do not report a queued write as a published order, remote lease, or successful landing.
[inferred] Reconnect by checking current heads, policy, and leases before explicitly confirming deferred writes.
[inferred] The hard switch must block Orrery-owned git and connector processes too.
[inferred] It cannot honestly promise to block independently launched agents or the user's other software.

[sourced] Humans own agent principals and mint repository-scoped expiring tokens with read or write roles and ref restrictions (`/tmp/kvarm/docs/agents/README.md:25-51`).
[inferred] Store the human connector credential in the OS credential store, and give each agent its own token, never the human credential.
[sourced] Read-agent tokens receive `read,api`, write-agent tokens receive `read,write,api`, and neither receives `ci` (`/tmp/kvarm/apps/platform/src/auth/agentTokens.ts:35-38`).
[sourced] A human PAT's `api` scope follows the holder's repository role rather than defining read-only access (`/tmp/kvarm/apps/platform/src/services/authz.ts:123-153`).

[sourced] A6 specifies three MCP permission levels without naming their exact boundaries in the supplied map (`/home/factory-user/orrery/explore/map.md:56`).
[inferred] Propose observe, design, and build as the adapter's levels, pending specification.
[inferred] Observe uses a repository read token and read-only tools.
[inferred] Design permits local proposals but needs an Orrery operation allowlist if remote issue or decision writes are enabled.
[inferred] Build adds write-agent workspace refs, claims, intents, changes, and authorized queue actions.
[inferred] Kvarm has no native middle "design-only" role, so giving an agent a raw write token bypasses that local distinction.
[inferred] Human policy administration and escalation resolution stay outside all agent levels, and a separate trusted runner owns the `ci` credential.

## Changes Kvarm should cause in Orrery

[inferred] Stop designing a second branch coordinator, task lock, or merge scheduler.
[sourced] Kvarm already has non-exclusive forecasts, generation-fenced claims, crash-reconciled publication, and exact-tree checks (`/tmp/kvarm/docs/architecture.md:318-346`).
[inferred] Redesign clash pins around that evidence, while retaining Orrery's runtime consequence and witness interpretation.
[inferred] Keep semantic overlap advisory, rather than converting a heuristic hit into a mandatory clash.

[inferred] Replace a single "agent did this" badge with principal, session, and evidence level.
[inferred] Keep fog's owned, tested, and understood signals separate, because authenticated attribution proves none of the other two.
[sourced] Orrery already names six independent fog signals (`/home/factory-user/orrery/explore/map.md:57`).
[inferred] Bind inspection and acceptance to exact heads and witness-definition revisions, and carry human review acknowledgements only under Kvarm's content identity rules.
[sourced] Exact-head approvals and content-keyed acknowledgements already exist (`/tmp/kvarm/docs/architecture.md:36-39`).

[inferred] Show Kvarm publication blockers separately from Orrery's advisory inspection-capacity forecast.
[sourced] The core spec makes the review gate advisory, while the earlier concept text proposes dispatch admission based on review minutes (`/home/factory-user/orrery/explore/ui-v2/CORE.md:3`, `/home/factory-user/orrery/explore/research/agentic-ux/concepts/00-judging.md:30`).
[inferred] Preserve self-claiming and show an attention warning rather than silently withholding the Kvarm task.
[inferred] Make the bench an experiment brief and evidence comparison over arenas, not a replacement arena service.

## Concrete gaps to request from Kvarm

1. [inferred] Add a versioned witness object with blueprint revision, trace or fixture reference, assertion, evaluator identity, and exact-tree result.
   [inferred] Reject changes to locked definitions without human approval of the exact definition digest.
2. [inferred] Add readbacks keyed by order revision and witness set, plus an optional server-side `claim_task` precondition requiring a matching readback.
   [inferred] Orrery-only enforcement cannot govern agents using Kvarm directly.
3. [inferred] Add path-specific protection for `.orrery/checks/`, budget fields in `system.yaml`, evaluator code, and `.kvarm/ci.json`.
   [sourced] Current `protected-path` escalation actually means a protected target ref, not configurable file protection (`/tmp/kvarm/apps/platform/src/services/reviewGate.ts:54-57`).
   [inferred] Until then, require a trusted locked-definition check and refuse managed edits or publication that lack human approval.
4. [inferred] Add stable client IDs and idempotency keys for orders and decisions, with explicit supersession and immutable repository-revision links.
   [inferred] This removes duplicate decisions after lost replies without pretending git and D1 are one transaction.
5. [inferred] Add a durable paginated state-event reader with sequence-gap reporting and coverage guarantees.
   [inferred] Keep the existing 200-event live feed for freshness, not complete Replay.
6. [inferred] Add structured runtime trace references without requiring public uploads, and accept locally resolvable evidence identifiers.
   [sourced] Current check `detailsUrl` accepts only HTTP or HTTPS (`/tmp/kvarm/apps/platform/src/services/checks.ts:565-568`).
7. [inferred] Add restricted operation scopes for design-only clients and document REST-only exceptions such as live sockets and check reporting alongside the registry.
   [inferred] Publish compatibility fixtures for head movement, stale claims, missing events, and offline reconnect.

## Recommended v0.1 scope

- [inferred] Ship one explicit Kvarm connection, bounded polling, an honest network log, offline behavior, and revision-aware cached state.
- [inferred] Draw workspaces, changes, attribution evidence, CI, review blockers, and Nexus evidence without claiming complete runtime visibility.
- [inferred] Support issue-backed orders, managed readbacks, self-claims, intent updates, settlement decisions, and exact-head queue publication.
- [inferred] Commit `.orrery/` through Kvarm and publish portable choice reasoning into Kvarm decisions with visible synchronization status.
- [inferred] Join delivery history to local Replay and produce a source-linked return brief.
- [inferred] Ship K16 enforcement before autonomous publication, and defer hosted fleets, WebSocket optimization, automated bench funding, and contract-capsule assumptions.

## Facts still unconfirmed

[inferred] This read-only assessment does not confirm a deployed origin, token permissions in a real account, current live latency, or end-to-end reconnect behavior.
[sourced] Kvarm's agent guides verify Claude Code, Codex, and Cursor, but list Droid's dedicated guide as unwritten (`/tmp/kvarm/docs/agents/claude-code.md:3`, `/tmp/kvarm/docs/agents/codex.md:3`, `/tmp/kvarm/docs/agents/cursor.md:3`, `/tmp/kvarm/docs/agents/README.md:5-11`).
[inferred] Droid-to-Kvarm setup, the exact A6 level names, and a trusted human-approved witness evaluator remain validation work, not established integration facts.
