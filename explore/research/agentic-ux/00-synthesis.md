# Agentic UX research synthesis

Date 2026-10-17. This file merges the seven slices in this folder (a to g). Bracketed labels are the ones each slice gave its claim. Pointers name the slice file and section heading, with line numbers where one line matters. URLs live in the slices.

## Summary

1. All seven slices agree that an agent result is a claim with evidence. The agent's assertion, a passing check, and observed runtime behavior stay separate, never one green light.
2. Six slices agree that the human's review capacity, not agent count or execution slots, is the real limit, and that it should throttle new work.
3. Six slices agree that motion, tokens, and agent busyness must not read as progress.
4. For pain 1 (intent into instructions), the evidence favors bounded contracts drawn on the plant, intent expressed through examples and counterexamples, and questions pinned where an ambiguity has high reversal cost.
5. For pain 2 (many agents), it favors catching semantic clashes on shared interfaces before code exists, one decision queue that says why each item needs you, and a return brief that makes leaving safe.
6. The game-like agent fleet, "mission control", agent canvases, and MCP-shared drawings are taken. Acan's remaining distinction is one persistent system entity linking intent, agent work, and measured runtime consequence.
7. Evidence is documentation, human-factors theory, and vendor or first-person numbers. Nothing measures how many agents one person can supervise, and no proposed interaction has been tested.

## Convergences

Source bases differ. Slice a covers product docs, b games and visualization papers, c human factors, d programming-languages and HCI research, e interruption psychology and operations, f competitors and developer testimony, and g construction and systems engineering. Agreement between c and g weighs more than between a and f, which share product sources. Some sources recur and weaken independence. METR appears in a, d, and e, Faros in a and d, and the September 2026 delegation preprint in c and f.

Nearly every row is an [inferred] design conclusion in each slice that reached it. The [sourced] parts are the precedents underneath.

| # | Idea in plain words | Slices | Strongest evidence pointer |
|---|---|---|---|
| 1 | Completion is a claim with evidence. Keep agent assertion, checks, and observed behavior as separate states. | a, b, c, d, e, f, g (7) | `d-intent-and-verification.md`, "4. Evidence-bound acceptance"; `c-supervisory-control.md`, "Design rules" (line 37) |
| 2 | Human review capacity limits throughput and should throttle new work. | a, b, c, d, e, f (6) | `e-attention.md`, "5. Review capacity limits production"; `c-supervisory-control.md`, "Fan-out is an attention budget" |
| 3 | Motion, tokens, and agent count are not progress. Measure verified outcomes and human minutes. | a, b, c, e, f, g (6) | `a-landscape.md`, "Throughput claims need a human denominator"; `f-frontier.md`, "Busy agents without progress" |
| 4 | Missing observation stays visibly unknown, never healthy-looking blank space. | a, b, d, e, f, g (6) | `b-games-softvis.md`, "Observability maps" (line 95); `d-intent-and-verification.md`, "5. Verification holes that remain visible" |
| 5 | Agents never set or weaken their own acceptance criteria (confirms K16). | a, b, c, d, f, g (6) | `d-intent-and-verification.md`, "What to avoid" (lines 156 to 160) |
| 6 | Delegate through a bounded contract with authority levels, stop conditions, and evidence obligations. | a, b, c, d, g (5) | `c-supervisory-control.md`, "3. Executable delegation plays"; `g-engineering-analogies.md`, "1. Issued-for-agents drawing sets" |
| 7 | One decision queue, separate from progress. Each item says why you, the cost of waiting, and the latest safe response. | a, b, c, e, f (5) | `e-attention.md`, "6. Decisions with an interruption deadline"; `c-supervisory-control.md`, "5. An alarm system with an audit trail" |
| 8 | Evidence binds to a revision and expires when what it covers changes. | a, b, d, f, g (5) | `b-games-softvis.md`, "7. Evidence freshness" |
| 9 | Predictions carry workload, calibration, and valid range, and stay labeled as predictions. | a, c, d, f, g (5) | `g-engineering-analogies.md`, "8. Prediction warrants and failure rehearsals" |
| 10 | A durable system object, not a conversation, carries intent across design, code, and runtime. | a, c, d, f, g (5) | `f-frontier.md`, "A persistent system model that survives a change" |
| 11 | The map must be actionable and tied to evidence. Topology alone is a picture. | b, c, e, f, g (5) | `b-games-softvis.md`, "Observability maps"; `c-supervisory-control.md`, "Why the abstraction hierarchy matters" |
| 12 | Compare alternatives under identical frozen checks and workloads before funding a build. | a, b, c, d (4) | `b-games-softvis.md`, "4. Pareto workshops"; `a-landscape.md`, "4. Decision experiment bench" |
| 13 | Return through a checked handover of what changed, not transcript replay. | a, c, e, g (4) | `e-attention.md`, "3. The return brief" |
| 14 | Unattended time is the goal. Stay calm, and let the human choose when to be interrupted. | b, c, e, f (4) | `e-attention.md`, "1. Safe to leave"; `f-frontier.md`, "Attention, waiting, and locating work" |
| 15 | No single confidence number. K17 confidence needs an evidence basis, with harm and urgency shown separately. | c, e, f, g (4) | `c-supervisory-control.md`, "Where the current design follows or breaks the rules" (lines 111 to 113) |
| 16 | Approving a plan is not oversight, and silence is not consent. | a, c, d, e (4) | `c-supervisory-control.md`, "What to avoid" (line 207) |
| 17 | Show mode and authority persistently in text and shape, not color or paper alone. | b, c, e, g (4) | `c-supervisory-control.md`, "Control-room and cockpit lessons" |
| 18 | Scale formality with ambiguity and reversal cost. Ceremony on every edit fails. | c, d, e, g (4) | `d-intent-and-verification.md`, "What to avoid" (lines 151 to 154); `g-engineering-analogies.md`, "A second model-maintenance job" |
| 19 | Ask at a consequential ambiguity, with concrete alternatives, before the large patch exists. | c, d, g (3) | `g-engineering-analogies.md`, "2. RFIs with an agent readback"; `c-supervisory-control.md`, "6. Preference checkpoints" |
| 20 | Agents request scope or authority changes as explicit change orders, never silently. | c, d, g (3) | `g-engineering-analogies.md`, "6. Change orders that reopen the right choices" |
| 21 | Separate worktrees do not prove independence. Detect semantic clashes on shared interfaces before implementation. | a, b, g (3) | `g-engineering-analogies.md`, "4. Semantic clash detection"; `a-landscape.md`, "3. Collision rehearsal" |
| 22 | Repeated corrections become reusable, previewable plays, proposed rather than applied automatically. | a, b, c (3) | `c-supervisory-control.md`, "Design rules" (line 43); `b-games-softvis.md`, "5. Supervision policies" |

Rows 1, 4, 6, 9, and 21 are the strongest signals because their precedents share no sources, for example proof-carrying code (d), as-built drawings (g), Factorio ghosts (b), and trust research (c). Row 2 is broad but rests on vendor and first-person reports.

## What is taken

These facts come from `f-frontier.md`, "Competitors and prior art" and "Novelty check".

- [sourced] AgentCraft, VibeCraft, Pixel Agents, and Agent Town already give agents game-like bodies and places (line 4). [inferred] "Factorio for agents" is a category, not a novelty claim (line 3).
- [sourced] Maestri combines local agents, a persistent spatial layout, notes, diagrams, and cross-agent connections (line 5). [inferred] It is the closest competitor for local-first spatial orchestration (line 40).
- [sourced] tldraw, Figma, and Archyl expose shared visual models through MCP (line 6), which the novelty table marks Taken (line 155).
- [sourced] The Codex app and Google Antigravity present themselves as homes for parallel agents. [inferred] "Mission control" cannot distinguish Acan (lines 42 to 43). Slice a documents the same session model across Cursor, Claude Code, Factory Missions, GitHub Agent HQ, and others (`a-landscape.md`, "Comparison of the tools").
- [sourced] Comper ties fog of war to software maps (line 53), Archyl claims drift detection as vendor marketing (line 52), Palladio and StrongDM cover what-if analysis (line 55), and Autoresearch and Chi cover agents optimizing a measured objective under a spending cap (line 56).
- The table marks system-as-factory, agents in the system they change, blueprint ghosts, budgets as objectives, fog of war, three-source drift, and calibrated simulation as Partly taken. Only the risk-ranked choices ledger is Open (lines 147 to 154).

For positioning, this means the following.

- [inferred] Do not pitch the aesthetic, the fleet view, the canvas, MCP access, or a "first" command center (`a-landscape.md`, lines 166 to 167; f, line 204).
- [inferred] Pitch one entity that persists from intent through code, deployment, and measurement, with designed, declared, and observed disagreements on one stable model (f, "A persistent system model that survives a change"). That is a possible distinction, not a cleared first (f, line 152).
- [inferred] The reversal-cost decision queue is the most open idea, and it holds only if confidence rests on evidence (f, lines 177 to 181). Per-machine forecast-versus-outcome history and objectives attached to the system come next (f, "White space").
- [sourced] Terragon shut its hosted service, and Vibe Kanban became community-maintained local software (a, lines 180 to 183). [inferred] This makes K9 local-first a trust argument as well as a privacy one.
- [inferred] f proposes a bakeoff against a Maestri-like canvas and an RTS office on the same change (f, "What the bakeoff should prove"). [guess] g proposes a planted cross-agent contract conflict as the decisive test (g, line 210).

## Contradictions and tensions

**Visible internals versus unnecessary takeovers.** Slices b, d, and g put evidence, constraints, and alternatives on the plant. [sourced] c reports that an ATC solution-space display improved fault detection but also prompted takeovers unrelated to faults (c, lines 34 and 211). I would keep the distant plant quiet and aggregate, open detail on selection, and count unnecessary interventions in the bakeoff.

**Contracts versus ceremony.** Slices c, d, and g add envelopes, submittals, RFIs, and readbacks. d warns against mandatory documents (d, lines 151 to 154) and g against a second bureaucracy (g, "A second model-maintenance job"). I would let reversal cost decide the gate, as d and g already suggest (d, line 137; g, line 92). Reversible details stay delegated. Public schemas, destructive migrations, new dependencies, and permission changes go through a submittal or change order.

**Automatic dispatch versus negotiated authority.** [guess] b's outcome request networks let agents claim open demand without a new prompt. [sourced] c reports that the few direct comparisons favor automation the human assigns over automation the system adapts (c, line 29). [sourced] a reports that Cursor's shared locks and dedicated integrator both became bottlenecks (a, lines 69 to 70). I would use human-set policies with a preview of what each agent would take and refuse (b, "5. Supervision policies"). Automatic containment is allowed, and silent expansion of permission is not.

**Grouping alarms versus hiding a second cause.** c and e both group shared failures into one decision and both flag this risk (c, line 180; e, line 175). I would show a grouping as a hypothesis unless the dependency is established, let the user split it, and never merge security or data-loss concerns.

**Whether K17 is enough.** d uses the K17 ordering as is ("Risk-ranked review" row). c says it omits harm and urgency and is unvalidated (c, lines 111 to 113). e makes delivery timing a separate policy (e, line 197), g keeps severity separate (g, line 145), and f warns about uncalibrated confidence (f, line 180). I would keep K17 as the review order, show urgency and harm as separate fields, derive confidence from evidence, and show the formula so users can challenge it.

**How many agents.** [sourced] e reports practitioners at two or three, five, and up to eight (e, lines 108 to 116), which e reads as different work arrangements (line 112). I would ship no default and learn a personal review budget instead (e, line 130).

**Simulated comparison versus real effects.** a, b, and d want matched runs of alternatives. d warns that shadow replays can repeat a payment (d, lines 171 to 174), and c warns that a false causal diagram breeds confident mistakes (c, line 68). I would isolate or stub effects, label every simulated result as a prediction, and promote only after independent measurement.

**Faros numbers.** [sourced] a found the Faros report gated with no results or method (a, line 98). [sourced] d cites Faros review-time increases of 199.6% and 441.5% from a different Faros blog page (d, lines 59 to 60). I would treat them as vendor observations of unknown method, used only to motivate the bottleneck hypothesis, as d does (d, line 61).

## The two user pains

### Pain 1. Turning intent into instructions agents follow

1. **A durable object on the plant is the unit of intent.** [inferred] The territory carries the contract, and conversations are disposable (`a-landscape.md`, "The conversation remains the default work record"; `d-intent-and-verification.md`, summary). [sourced] BCF, SysML v2, and boundary objects show shared models with stable IDs across viewpoints (`g-engineering-analogies.md`, "Artifacts and rituals").
2. **Issue bounded contracts, not prose.** [inferred] A contract carries protected checks, scope, tools, required evidence, and a safe abort state. The agent may change its plan but must request changes to its authority (`c-supervisory-control.md`, "3. Executable delegation plays"). [inferred] Authority splits into inspect, propose, edit, and merge (c, line 27). [inferred] The UI separates enforced rules from advisory ones (c, line 160), because [sourced] Cursor says its allowlists are not security boundaries (`a-landscape.md`, "What to avoid").
3. **Express intent with examples and counterexamples.** [inferred] The user selects a failing request, marks the forbidden effect, and pins a request that must stay unchanged (`d-intent-and-verification.md`, "1. Counterexample-directed repair"). [inferred] Ask for discriminating examples before generalizing one demonstration (d, lines 180 to 182).
4. **Gestures on the plant create obligations.** [inferred] Inserting a breaker or pinning a latency ceiling creates a contract the user approves, while the recipe stays with the agent (d, "2. Contract gestures on the plant"). Only port and fitting edits change intent, never layout moves (d, line 94). [inferred] Tolerance budgets show which end-to-end margin a local change consumes (g, "5. Tolerance budgets instead of isolated targets").
5. **Ask at consequential ambiguity, before the patch.** [inferred] An RFI pinned to the port names the competing readings and what can continue safely (g, "2. RFIs with an agent readback"). [inferred] Preference checkpoints include a "not specified" marker and accept an option the agent did not offer (c, "6. Preference checkpoints").
6. **Submittals and change orders for high reversal cost.** [inferred] An agent submits a schema, dependency, or migration plan before dependent work (g, "3. Submittals before commitment"). [inferred] Work beyond the approved envelope appears as a new choice at the affected machine (d, "6. The plan is an execution envelope").
7. **Close the loop against the same contract.** [inferred] Acceptance shows each requirement with check identity, revision, and environment, and fails closed when either changes (d, "4. Evidence-bound acceptance"; g, "7. As-built acceptance").
8. **Learn from corrections without silent hardening.** [inferred] Corrected instructions become proposed reusable assets (c, line 43).

[sourced] Willison names the same two skills, confident instruction and confident verification (`f-frontier.md`, "Verification overload and unreliable evidence").

### Pain 2. Coordinating many agents at once

1. **Catch semantic clashes before code exists.** [inferred] Agents declare intended writes and interface assumptions. The floor marks a shared interface where proposals conflict even when files differ, and the architect reconciles, serializes, or reassigns (`g-engineering-analogies.md`, "4. Semantic clash detection"; `a-landscape.md`, "3. Collision rehearsal"). [inferred] Worktrees prevent simultaneous writes but not semantic incompatibility (a, line 86).
2. **Reservations with expiry and visible authority.** [inferred] Each bot shows the model revision it read, its assumptions, intended writes, and territory (g, "10. Semantic presence with bounded authority"). [guess] Reservations need expiry because they can deadlock or outlive crashed agents (g, line 169). [inferred] Budgets and accepted contracts never resolve by last write wins (g, line 199).
3. **Review capacity governs dispatch.** [inferred] The user sets a review budget, and new high-risk work parks when demand exceeds it while verification continues (a, "2. Review-capacity dispatch"; b, "3. Review backpressure"; e, "5. Review capacity limits production"). [inferred] Never lower evidence requirements to fit the budget (e, line 187).
4. **Every wait states its reason.** [inferred] A queued agent needs a reason, not an instruction to try harder (b, mechanics table, logistic network row). [sourced] One developer says unpredictable waits make it impossible to choose between waiting and switching (f, line 68). [inferred] Each mission shows its next intervention window and safe parking point, with unknowns shown as unknown (c, "2. Safe-to-leave scheduling"). [inferred] Priority policies need starvation warnings (b, RimWorld row).
5. **One queue of decisions.** [guess] Five bots hitting one expired credential produce one card (e, "4. One symptom, one decision"). [guess] Each card shows why you, what happens if it waits, and the latest safe response (e, "6. Decisions with an interruption deadline"). [inferred] Shelving needs an owner, reason, and expiry (c, "5. An alarm system with an audit trail").
6. **Deliver at the human's checkpoint.** [guess] Non-urgent decisions wait until the user finishes the current review, with a visible maximum delay (e, "2. Deliver at my checkpoint"). [sourced] Iqbal and Bailey's deferral reduced frustration (e, line 68).
7. **Review results, not conversations.** [inferred] A dailies playlist of changed paths, clashes, regressions, and risky choices replaces reading each bot's account (g, "9. Dailies of system changes"), and a return brief replaces replay (e, "3. The return brief").
8. **Do not answer coordination with a manager agent.** [sourced] Cursor's shared locks cut twenty agents to the throughput of two or three, and a dedicated integrator became another bottleneck (a, lines 69 to 70). [inferred] Coordination belongs in the shared model, not another agent or a shared checklist (a, line 71).

## Gaps and evidence quality

- **Retrieval method.** Six slices say FetchUrl was limited to user-supplied URLs, so they read pages through WebSearch full text (b, line 18; c, lines 13 to 14; d, lines 15 to 16; e, line 20; f, line 18; g, lines 18 to 19). Slice a says it used "WebSearch full-text results and FetchUrl" (`a-landscape.md`, line 16), which conflicts with the others and the parent's account. Treat a as WebSearch-read until checked.
- **Rate limits.** c hit HTTP 429 errors, verified no CSCW paper, and calls its coverage a sample (`c-supervisory-control.md`, lines 15 to 19). f hit 429 errors on System Initiative and a CHI 2026 preprint, did not recover the requested X threads, and has no CHI 2026 coverage (`f-frontier.md`, lines 9, 23 to 26, and 140 to 141).
- **No capacity measure.** No source measures how many agents one person can supervise (a, line 4; e, line 8; c, lines 75 to 81). c's fan-out bound is an idealized intuition (c, lines 78 to 81).
- **Partial reading.** Some sources were abstracts or partial scans (c, lines 32 and 41; e, line 21).
- **No hands-on testing.** Product claims are documentation, not installed behavior (a, line 16; f, line 28). Every proposed interaction is [inferred] or [guess].
- **Vendor and first-person numbers dominate.** The only randomized developer study is METR's early-2025 work, and its follow-up calls newer estimates unreliable (a, lines 93 to 95).

Claims that look overstated or need care.

- `a-landscape.md:16`. The method claim names FetchUrl, which the other slices say was not allowed for discovered URLs.
- `e-attention.md:8`. A [sourced] summary range of "three or four sustainable threads to eight" rests on two first-person posts. The body also reports two or three and five (lines 108 and 114). It is testimony, not a measured range.
- `e-attention.md:121`. An Anthropic report dated one day before the research date says sessions spend about 80% of wall time waiting for humans. It is one company's figure, derived by subtraction (line 122). The slice caveats it, but the number invites misuse.
- `d-intent-and-verification.md:60`. The Faros percentages come from a blog page whose underlying report a found gated (`a-landscape.md:98`).
- `f-frontier.md:153`. "Open" for the risk-ranked ledger rests on a search that rate limits cut short (f, line 26). Read it as "not found", as f's own definition says (f, line 27).
- `f-frontier.md:3`. "Already a category" rests mostly on hobby repositories and product pages f did not install (f, line 28). The point that the aesthetic is not defensible stands, but "category" overstates market maturity.
- `a-landscape.md:69`. The Cursor result is one vendor's internal experiment, which a says does not transfer to the product (a, line 72). It is the only source for the manager-bottleneck claim.
- `c-supervisory-control.md:43` and `f-frontier.md:135` cite the same preprint, still under review. Their agreement counts as one source.

## What the plant must become

1. **A plant of obligations, not activity.** Each machine and ghost shows designed, implemented, checked, and observed as separate marks. Moving belts and typing bots stop meaning progress, and the age of the latest independent result becomes the progress signal (rows 1, 3, and 4; `f-frontier.md`, line 118).
2. **A place where intent is drawn as contracts.** Selecting a territory issues a work package with protected checks, authority levels, stop conditions, and examples. Agents reply with questions, submittals, and change orders pinned to the exact port, and formality scales with reversal cost (rows 6, 18, 19, and 20).
3. **A coordination model for shared interfaces.** Agents declare intended writes and the revision they read. Clashes appear on the shared belt or port before code exists, and reservations expire (row 21).
4. **Output limited by the reviewer.** Review demand and capacity are visible. Risky new work parks when the human is full, and every waiting agent says why (row 2).
5. **Safe to leave.** The plant is quiet by default, sends one queue of decisions with why you, the cost of waiting, and a deadline, and greets a return with a brief of what changed (rows 7, 13, and 14).
6. **Honest about what it does not know.** Fog names its cause and the action that would clear it. Simulations carry their warrant, K17 confidence comes from evidence, and urgency and harm show separately (rows 4, 9, and 15).
