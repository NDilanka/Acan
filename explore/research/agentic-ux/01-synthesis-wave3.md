# Wave 3 synthesis: Kvarm, dogfood, fidelity, world

Date 2026-10-17. Citation keys are `h`, `i`, `j`, `k`, and `f` for the reports of those letters, `dec` for `decisions.tsv`, and `CORE` for `../../ui-v2/CORE.md`.

## Summary

1. [inferred] Kvarm owns coordination, and Orrery owns the drawing, witnesses, readbacks, and runtime evidence (h:5, h:208).
2. [inferred] A claim is an Orrery readback match plus a Kvarm lease, and Kvarm's rejection of a competing live claim is the atomic step (h:34, h:99-102).
3. [inferred] K16 needs Orrery, Kvarm, and the Droid sandbox together (h:146-152, i:154-164).
4. [inferred] Every semantic mark has an inspectable binding, and unknown, stale, zero, stopped, and simulated never look alike (j:7, j:195-216).
5. [inferred] The world is a canvas of systems, not of agents or terminals like Maestri and VibeCraft (f:35, f:40).
6. [inferred] A thin Orrery self-loop goes first, and Acquit stays the fidelity reference and the week-6 demo (i:108, j:9).

## Convergences

- [inferred] Everything binds to a revision, including orders (i:58), Kvarm caches (h:81), bindings (j:195), and cross-plant endpoints (k:150).
- [inferred] Missing knowledge stays visible, as unmatched claims (h:102), unknown activity (i:126), a fogged tray (j:129), or an unknown rate (k:153).
- [inferred] There is no second coordinator, one queue, and one aggregate budget (h:208, k:164, k:218).
- [inferred] Nothing moves the camera or assigns work for the human (i:66, k:173, CORE:30).

## Conflicts and recommendations

### Kvarm claim lease versus the readback gate

[sourced] h has agents call `claim_task` after an Orrery readback match (h:99), while i keeps Orrery authoritative for claims on day one and mirrors work into Kvarm (i:78).
**Recommendation.** [inferred] Orrery owns the readback verdict. Kvarm owns the lease. The order reads `CLAIMED` only when a matching readback and a live lease with the same generation both exist. [inferred] The adapter calls `claim_task` with the agent's own token (i:62, h:194).
**What would change it.** [inferred] A Kvarm readback precondition on `claim_task` (h:228) moves the gate into Kvarm.

### K16 enforcement split

[sourced] h proposes an Orrery evaluator behind a required Kvarm check (h:146-149). i adds read-only mounts and Droid sandbox deny-write paths (i:156-160). Kvarm protects refs, not files (h:231).
**Recommendation.** [inferred] Use three layers with one job each. The Droid sandbox prevents writes in the worker. The pinned runner compares protected inputs to the approved digest. The Kvarm branch rule blocks publication without that check, reported through the separate `ci` credential (h:149, i:162).
**What would change it.** [inferred] Kvarm path protection (h:230) retires the custom check. [sourced] No native Windows sandbox is documented (i:160), so Windows needs WSL2 or a VM before autonomous writes. [inferred] Kvarm excludes the recorded author from approval counts (h:151), so agents need principals distinct from the solo owner.

### Dogfood ordering versus the Acquit fixture

[sourced] The handover puts Acquit first and Orrery's self-drawing in weeks 10 to 12 (map.md:328, map.md:341). i moves a thin self-loop ahead of Acquit (i:108).
**Recommendation.** [inferred] Run i's milestones 0 to 4 in weeks 2 to 4, then keep the D13 Acquit ghost task as the week-6 demo through the same loop. [inferred] Validate fidelity bindings on Acquit first, because Orrery does not exercise the service catalog (map.md:98, j:9).
**What would change it.** [guess] If milestone 4 slips past week 4, return to Acquit-first to protect the demo.

### Fidelity "unknown" versus Kvarm "stale offline"

[sourced] j marks runtime bindings stale after five seconds (j:209), but Kvarm polls every 30 to 60 seconds (h:158, h:161) and goes stale offline (h:187).
**Recommendation.** [inferred] Keep one stale treatment with per-source freshness (j:209-210). Kvarm fields go stale after two missed polls or at once on disconnect, and Alt+hover names the source.
**What would change it.** [guess] A WebSocket feed (h:170) could shorten the window.

### Global queue versus the advisory gate

[sourced] The gate is advisory (dec:15), though an earlier concept admitted work by review minutes (00-judging.md:30). k recommends holds from one allowance (k:176), and h separates Kvarm publication blockers (h:219).
**Recommendation.** [inferred] The queue has two row kinds. `blocks publication` rows come from Kvarm review rules and are enforced by Kvarm. `recommended hold` rows come from the advisory allowance. The allowance never withholds a Kvarm task from self-claim (h:221, k:178).
**What would change it.** [guess] Measured review omissions (k:259) would justify an opt-in cap.

## Differentiation from Maestri and VibeCraft

[sourced] Maestri is a macOS canvas of terminals, diagrams, agent connections, and notes agents read and write (f:40). VibeCraft puts agent units, folders, terminals, and worktrees on an infinite canvas (f:35, dec:18).
[inferred] "Infinite canvas for agents" is taken (dec:18, 00-synthesis.md:12).

**What Orrery's world does that they do not.** [inferred] Its objects are running services, queues, and stores bound to designed, declared, and observed evidence (j:195, map.md:29). Agents appear on the machines they change (i:38). Lines between plants are declared contracts with revisions (k:150-159). Work arrives as witnessed orders that agents self-claim (CORE:9-12).

**What it must not copy.**
- [inferred] Unit command ("select a unit, assign work") contradicts self-claim (f:35, CORE:12).
- [inferred] Terminals on the canvas make the agent the subject. The terminal stays an outside escape hatch (i:146).
- [inferred] Free-form shared notes bypass typed choices (h:137-140).
- [inferred] Decorative links imply false dependencies (k:9).
- [inferred] Activity animation reads as false progress (00-synthesis.md:9).

## CORE.md amendments

Each concept inlines its own scripted Kvarm fixture, with no network. A strip under the plate logs each op with destination, bytes, and reason, and the egress pill adds the bytes. A hash flag `kv=ok`, `kv=unmatched`, `kv=stale`, or `kv=offline` picks the script.

**Orders.**
1. Post (P, Enter) writes order revision `r1` locally, then logs `create_issue` and `dispatch_issue` (h:98). The flag reads `POSTED LOCALLY` until both return, then `OPEN · kvarm #<n>`.
2. Readback rows say `readback · Orrery`, because Kvarm stores none (h:64).
3. An answer that creates w3 bumps the order to `r2` and logs `comment_issue`. Readbacks show `re-scored on r2`.
4. Claim logs `claim_task` from the agent's token. The flag reads `CLAIMED · Claude Code · lease g1` with a text countdown. The other matched agent shows `yields · Kvarm rejected`.
5. With `kv=unmatched`, Codex holds the Kvarm lease with no readback. The flag reads `UNMATCHED · Codex holds g1 · no readback` in amber, the territory stays fogged, and a Waiting-on-you row appears. Never draw this state as `CLAIMED`.
6. With `kv=stale`, a heartbeat fails. The flag reads `LEASE STALE · g1 expired`, and the bot shows `stopped · STALE_GENERATION` (h:100).
7. With `kv=offline`, Post shows `POSTED LOCALLY · not in Kvarm`, and a matching readback shows `MATCHED · claim waits for connection`. The pill lists pending writes. Reconnect shows `checked heads and leases` and needs Enter to send them (h:188-189).

**Clash.**
8. Footprint labels name their source, for example `declared · kvarm intent` or `static` (h:66).
9. Each clash card shows Kvarm `forecast` evidence with age, or `forecast truncated` (h:30). c3 stays advisory.
10. Every settlement writes a choice file and logs `record_decision` with the choice path and ID (h:111, h:138). Split adds a contract file commit. Split, a winning side, and one migration owner log `set_intent` for each narrowed side. Serialize shows `after · pin waits for upstream revision` and logs no `add_dependency` (h:113). Merge logs a blueprint commit only (h:116).
11. The settled mark shows `decision synced` or `decision pending` (h:142). Ctrl Z logs a superseding decision and says `superseded`, because Kvarm cannot delete it idempotently (h:141).

**Bindings and provenance.**
12. Every semantic mark reads from a binding record with channel, value, state, source class, locator, revision, and age (j:195). Gauge, label, and popover read the same record (j:198).
13. Alt shows labels, as in ui-v1. Alt plus hover on a part opens its provenance, for example `one cell · declared concurrency 1 · deploy/runtime.env:12 · runtime not reported` (j:201-202). The inspector lists the same bindings for keyboard users (j:203).
14. Draw five distinct states (j:7, j:208-215). **Unknown** is fog hatch, empty bezel, and no motion. **Stale** is a dimmed last value with age and no motion. **Zero** is an explicit `0` with the needle at rest. **Stopped** is a dark shell with a lifecycle source. **Simulated** is graph-paper hatch with a `SIM` tag. Invalid draws as unknown with a reason.
15. A missing field never falls back to an `assets.mjs` demo default (j:108, j:232). Acquit's verifier draws one cell.
16. Failure and fog can show together (j:216).
17. Bot badges show principal, session, and evidence level (`declared`, `authenticated`, or `verified`) (h:39, h:213).

## Prototype brief: `world`

**Goal.** Prove that one canvas of five plants replaces tabs without losing direct access (k:5).

**Files.** `concepts/world.mjs`, `concepts/world-data.mjs`, and `concepts/world.steps.json`, started with `#c=world`. Expose `window.worldState` with `{ focus, level, history: { i, n }, bookmarks, queueOpen, kvarm: { state, pending }, egressBytes }`.

**Plant sites.** Positions are fixed and never move.
- Orrery. One Rust housing with receiver, matcher, evaluator, SQLite, telemetry ring, and webview renderer (i:112-114).
- Acquit. The existing scene, with the corrected one-cell verifier (j:122).
- Kvarm. Worker, D1, Artifacts, repo events, and the train (h:16-17, h:32, h:92).
- `orrery-format`. [guess] The spec's reference validator library (map.md:298).
- `orrery-site`. [guess] A static waitlist page (map.md:116).

**Contract belts.** Each has a line pattern and text (k:151).
- Acquit to Orrery, runtime OTLP/HTTP (map.md:221).
- Orrery to Kvarm, runtime REST, marked as declared egress.
- `orrery-format` to Orrery, a package dependency with version.
- Orrery to `orrery-site`, artifact delivery.

**Semantic levels.** World shows name, freshness, decision count, and top condition (k:131). Plant shows silhouettes and territories (k:135). Machine fills the inspector (k:139). Cutaway uses X, the existing explode key (k:143). Scroll changes level with hysteresis (k:145). World view is read-only, and a drag across a boundary cancels (k:248-249).

**Keys.** Free keys from CORE:25, plus two exceptions the parent must approve.
- U opens or closes Waiting on you. Shift 1 to 4 jumps to a row.
- I jumps to the next waiting item, as in `orders` and `clash`.
- V toggles contract belts. Selecting a belt end offers "jump to other end".
- P fits the focused plant. Shift P fits the world.
- M marks the current place as a bookmark.
- Ctrl K is the global jump. This overrides the ui-v1 palette (`actions.mjs:143`) while the concept is active. The empty query lists Recent, Bookmarks, and Waiting.
- Alt Left and Alt Right go back and forward through `history.pushState` and `popstate`. Alt alone keeps its labels role.

**Waiting on you.** Each row names plant, why you, cost of waiting, latest safe response, and an estimate range (k:165, k:180). The five rows are an Orrery unmatched claim, the Acquit p-2307 question, the Acquit clash c1, a Kvarm review, and an `orrery-format` check digest. A sixth row arrives after 3 seconds without moving the camera.

**Kvarm pill.** It reads `Kvarm · <origin> · 3 repos · fresh 8 s`, or not connected, stale, or offline. A click or Ctrl K "Go offline" flips the hard switch. The egress pill opens the network log.

**Screenshots that prove it.**
1. `world-01-overview`, five readable names.
2. `world-02-queue`, five rows with plant names.
3. `world-03-jump`, p-2307 with its departure note (k:201).
4. `world-04-back`, camera, selection, and mode restored.
5. `world-05-search`, "verifier" results with plant names.
6. `world-06-contract`, the far end of the OTLP belt.
7. `world-07-levels`, plant, machine, and cutaway.
8. `world-08-states`, all five states.
9. `world-09-provenance`, the verifier popover.
10. `world-10-offline`, stale fields and pending writes.
11. `world-11-bookmark`, a mark recalled with Shift 1.
12. `world-12-arrival`, the sixth row and an unchanged camera.

Assert stable site coordinates. A mouse-only path repeats shots 1, 3, 6, and 10.

## Candidate map.md rows

| ID | Decision | Why | Source |
|---|---|---|---|
| W1 | Kvarm owns coordination. Orrery owns drawing and evidence. | No second coordinator. | h:5, h:208 |
| W2 | A claim is a readback match plus a Kvarm lease. Unmatched claims are labeled. Amends CORE step 4. | One authority per fact. | h:99-102, i:78 |
| W3 | K16 in three layers. Sandbox, pinned runner on a digest, Kvarm required check. Amends K16, D11. | No layer covers it alone. | h:146-152, i:154-168 |
| W4 | Kvarm is opt-in, declared egress, and polled. Refines Q16, V6. | Local-first honesty. | h:168-177, dec:28 |
| W5 | Offline marks remote state stale and holds writes until confirmed. | No false publication. | h:186-191 |
| W6 | Cap at 480 reads and 90 writes per token per minute across the world. | Kvarm rate limits. | h:156-166 |
| W7 | Choice YAML is canonical and published to `record_decision` with sync status. Amends D11. | Git and D1 are not atomic. | h:137-143 |
| W8 | A thin Orrery self-loop runs before Acquit D13, which stays the week-6 demo. | Dogfood day one. | i:108, i:122-132, map.md:328 |
| W9 | A last-good observer watches the candidate build, with a terminal escape hatch. | Safe self-hosting. | i:142-150 |
| W10 | Droid runs as a `stream-jsonrpc` subprocess with content-free GenAI OTel. Amends E1, V3, V4. | Documented interfaces. | i:16-48, i:96-100 |
| W11 | Every semantic mark has a binding, shown on Alt hover. No demo defaults. Refines S1, K4. | The plant cannot lie. | j:195-204, j:232 |
| W12 | Unknown, stale, invalid, zero, and valid are distinct from operational state. Refines V10. | Missing is not idle. | j:208-216 |
| W13 | The v0.1 binding table and its count limits are guesses to test. | Limits are unmeasured. | j:248, j:272-281 |
| W14 | One personal world with authored sites and four semantic levels. Extends K5, replaces the project selector. | No tab switching. | k:114-146, k:210 |
| W15 | One Waiting-on-you queue with `blocks publication` and `recommended hold` rows. | The gate stays advisory. | k:162-179, h:219 |
| W16 | Cross-plant belts show declared contracts only. | No decorative links. | k:148-160 |
| W17 | Ctrl K jumps across plants, with history and bookmarks. Amends V7. | Direct access. | k:195-208 |
| W18 | A10 is one aggregate budget across plants. Amends A10. | Not multiplied per plant. | k:215-226 |
| W19 | Orrery is a canvas of systems, not of agents, terminals, or notes. | Taken category. | f:35, f:40, dec:18 |

## Light mode

Placeholder. `l-light-mode.md` is not ready. Its findings land here, including the five states on light paper.

## Open questions for the user

1. **Should Kvarm enforce the readback before `claim_task`?** [inferred] Orrery cannot stop agents that use Kvarm directly (h:228-229). Default is to label unmatched claims now and add the precondition to Kvarm.
2. **Is Acquit D13 still the week-6 demo with the self-loop first?** [inferred] The handover plan changes (map.md:327-328). Default is yes, with the self-loop in weeks 2 to 4.
3. **May autonomous writes on Windows require WSL2?** [sourced] The Droid sandbox does not document native Windows (i:160), and Windows is the polish target (map.md:144). Default is WSL2 for writes and native read-only readbacks.
