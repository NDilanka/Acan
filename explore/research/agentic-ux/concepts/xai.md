# Interaction concepts for Orrery

Six concepts for the Acquit plant. Each one changes what the engineer does, not how the brass looks. The plant stays. The prompt box, the per-session waiting list, and the panel that asks the engineer to accept an agent's sentence do not.

Today the engineer types a goal into a terminal, a bot edits files, and the engineer accepts or rejects a diff. On this plant that shows up as Codex on the PayPal belt, "waiting on you, 9m," because its terminal asked to run `npm test`. It also shows up as the review panel whose question is "How do we bring verifier queue wait under 30 s?" with the agent's own answer already filled in.

`f-frontier.md` says the factory drawing is not open ground. Gas City, VibeCraft, Figma's agent canvas, and tldraw's MCP app already exist. What those sources do not establish is a decision queue ranked by evidence rather than model self-rating, an objective attached to a machine instead of to an agent, and one identity that survives from intention to measurement.

## 1. The standing order

A machine carries a desired condition, and any agent that can satisfy it claims the work. The engineer stops writing a prompt per job.

**Pain.** Both. The order is the instruction, so pain 1 shrinks to one sentence typed on a machine. Agents stop waiting for a fresh prompt, which is the waiting half of pain 2.

**Sixty seconds on Acquit.** It is 15:24. PayPal reads 4.1% errors against a 2% budget, the retry fitting is on loop 2 of 3, and paypal-payouts is unclaimed with fog 0.42. The engineer presses `G` then `P`, clicks the empty brass plate under the dock's gauge, and types "Payouts complete once, or fail closed, while PayPal is down." They drag two payouts up from the Replay scrubber, one charged once and one charged twice at 15:41 yesterday, and mark the second forbidden. Enter. The plate inks in and prints three locked checks. No second charge on the pinned pair. Error rate under 2%, measured rather than predicted. The breaker ghost may not edit either check. An "open" flag appears. Codex has sat idle for nine minutes waiting on a terminal prompt. Its permit says "may claim standing orders," so it turns and writes one line. "I will add jitter, stop at two tries, and not touch the breaker." The engineer does not answer. The readback matches, Codex claims the territory, and the fog clears to a claimed tint. Droid stays on radar-otel, since its current order outranks an open plate. The engineer typed one sentence and pinned two traces.

**What it replaces.** The goal string on a session, `SESSIONS[].goal` set to "Back off PayPal retries with jitter," and the habit of retyping it for each new session. A session starts by claiming a plate, not from prose.

**Data.**

```ts
type StandingOrder = {
  id: string
  on: string              // entity id, "paypal"
  text: string            // the one sentence the engineer typed
  pins: { replayId: string; spanId: string; verdict: "keep" | "forbid" }[]
  checks: { id: string; expr: string; locked: true }[]  // K16, agents read, never write
  open: boolean
  claimedBy: string | null
  readback: { agent: string; text: string; matches: boolean } | null
}
```

**Evidence and novelty.** `b-games-softvis.md`, "Outcome request networks," supplies the demand side. A machine advertises a condition and agents reserve it. `d-intent-and-verification.md`, "Counterexample-directed repair," supplies the instruction side, where a selected runtime event becomes the instruction. Against `f-frontier.md` this sits in "Objectives attached to the system, not the agent." Autoresearch and Chi hillclimb a fixed score. Neither pins the objective and a forbidden example on the machine being edited. Gas City records versioned work, not a condition a machine keeps posting after the session that wrote it has exited.

**Risk and the test that kills it.** Agents will satisfy the pinned traces and miss the intent. That is the overfitting risk named in `d-intent-and-verification.md`. In the mock, the engineer posts the payout order and watches two agents claim it. The concept dies if both readbacks look plausible and the engineer cannot tell which matches the order without opening a diff. A plate that still needs a paragraph of clarification is a prompt box drawn on a machine.

## 2. The shift

The engineer sets the time they will be away, and the plant schedules agent work against the decisions that will be waiting when they return.

**Pain.** Pain 2, the waiting half. Agents block on the human, or finish at a moment the human cannot use. The shift makes absence a plan instead of a risk.

**Sixty seconds on Acquit.** The inspection station shows seven items waiting. The engineer presses `Shift`+`L` and a brass card rises with three ranges, not countdowns. Codex wants to run tests any minute, marked safe to refuse. Droid's check 4 of 5 is a trace-context choice, likely inside 20 minutes, low reversal cost. The verifier queue offers nothing until concurrency is raised, choice 2 of 5. The engineer drags the return dial to 16:30 and presses Enter. The title block reads "back at 16:30." The shift answers Codex's terminal prompt with "no," and the bot turns to the open plate on the PayPal dock. Droid keeps verifying, since no remaining check needs a person. The red belt gains a parked notch at the breaker ghost. At 16:30 the title block flips to four lines. What changed, which check moved, what is asked, what was refused. The engineer presses `J` for the held breaker decision and reads neither transcript.

**What it replaces.** The "Waiting on you" list at the lower left of `l-default.png`, which counts sessions rather than decisions, and the Floor timeline in `p5-floor.png`, which records the past. The shift is the same timeline run forward, with the human's absence as an input.

**Data.**

```ts
type Shift = {
  awayUntil: string                     // ISO time the engineer picked
  held: { choiceId: string; why: string; latestSafe: string | null }[]
  refused: { sessionId: string; ask: string; reason: string }[]
  promise: "no unapproved consequential action"  // never "nothing will go wrong"
  brokenBy: { at: string; reason: string } | null
}
```

**Evidence and novelty.** `e-attention.md`, "Safe to leave" and "Deliver at my checkpoint," supplies the mechanism. Holding delivery to a breakpoint beat ad hoc interruption in the Iqbal and Bailey studies that report cites. `c-supervisory-control.md`, "Safe-to-leave scheduling," adds the rule that an unknown estimate stays unknown. `f-frontier.md` does not list scheduling a human absence as taken or open. No reviewed source dispatches agents against a stated return time while promising only "no unapproved consequential action."

**Risk and the test that kills it.** A shift that says "safe" while an agent takes an irreversible action is worse than no shift. The mock task is to set a 40-minute shift while a bot's script tries to drop the `effects` table. The concept dies if the return strip lists that drop as something that happened during the shift, instead of as the event that ended the shift early with a reason.

## 3. The clash pin

Before any agent writes code, it declares the contracts it intends to change. When two declarations disagree, the plant shows the disagreement on the shared belt, and the engineer resolves it with one gesture.

**Pain.** Pain 2, the conflict half. Textual merge conflict is the late, cheap symptom. The expensive one is two agents holding incompatible ideas about the same flow.

**Sixty seconds on Acquit.** The belt from acquit-api to the PayPal dock carries two ghost fittings. Droid's breaker trips after 5 fails in 30 seconds. Codex's retry drops to x2 with jitter. Each is legal alone. Together they double-charge yesterday's pinned payout, because both fire on the same 5xx. Orrery draws one brass pin through both fittings. The engineer clicks it and a card opens on the belt, not in a side panel, with the two declarations and one line. "Both fire on the same 5xx. The pinned payout charges twice." Three buttons follow. "Breaker wins." "Retry wins." "Serialize." The engineer clicks the first. Codex's fitting dims with a "yields" tag, Droid's breaker stays proposed, and Codex's bot turns to the next open plate. No file was edited. The conflict ended in one click, while both changes were still declarations.

**What it replaces.** Discovery of the conflict in a pull request, or in the choices ledger after both agents have built their version. The ledger still ranks the resulting choice. The pin stops the choice from being created twice.

**Data.**

```ts
type Declaration = {
  id: string
  agent: string
  touches: string[]          // entity or belt ids
  asserts: string            // "breaker trips after 5 fails in 30s"
  status: "open" | "yielded" | "building"
}
type Clash = {
  id: string
  belt: string               // "api->paypal"
  decls: [string, string]
  consequence: string        // one sentence, tied to a pinned trace when one exists
  resolution: "left" | "right" | "serialize" | null
}
```

**Evidence and novelty.** `g-engineering-analogies.md`, "Semantic clash detection," is the source, grounded in BCF issues between discipline models. `a-landscape.md`, "Collision rehearsal," makes the same point from the other side. A clean worktree is not evidence of compatible intent. `f-frontier.md` marks "Designed versus declared versus observed drift" as partly taken, by Archyl and Multiplayer. The pin is a different comparison. Drift sets three sources against one thing. A clash sets two agents' intentions against one belt, before either becomes code. No row in the novelty table shows that.

**Risk and the test that kills it.** The detector will invent clashes from weak inference, and the engineer will learn to dismiss the pin. In the mock, two agents work paypal-payouts for one hour. The concept dies if more than one pin in three cites a consequence the engineer judges false on inspection. A pin the engineer does not trust is noise, and `e-attention.md` is clear that noise trains people to ignore the channel.

## 4. The margin stack

The engineer sets one number on a path. Every machine shows the share it consumes, and an agent that spends another machine's share is refused.

**Pain.** Both. Pain 1, because the instruction becomes a single path budget instead of a per-machine prompt. Pain 2, because agents stop optimizing against each other without knowing it.

**Sixty seconds on Acquit.** The engineer clicks the intake, then shift-clicks the PayPal dock. The belts between them lift and a stack opens, one brass segment per hop. Gate shows 1 ms of 3 ms, the API 184 ms of 250 ms, PayPal 400 ms call time and 4.1% errors against 2%. The verifier segment is hatched, not numbered. The fixture says Docker cold start varies from 4 s to 30 s and the model uses the mean, so the hatch prints that range instead of a false-precise figure. The engineer drags the path total from 250 ms to 200 ms. The API segment turns red, since 184 ms no longer fits. Droid, on the radar order, drops a change that would add the radar's 1% service-time overhead. It does not ask. The segment gains a mark, "Droid held, overhead 1% does not fit." A number the agent cannot edit blocked its local win.

**What it replaces.** Per-machine budgets as the only objective an agent sees. The fixture already has `api.p99_ms < 250` and `paypal.error_rate < 2%` as separate strings. Nothing tells an agent that shaving the API's latency by retrying PayPal harder spends the PayPal budget. The stack is that missing relation.

**Data.**

```ts
type PathBudget = {
  id: string
  hops: string[]                  // ["intake", "gate", "api", "paypal"]
  metric: string                  // "p99 latency"
  total: number                   // the one number the engineer set
  shares: { entity: string; used: number; allowed: number }[]
  unmodeled: { entity: string; range: [number, number]; why: string }[]
  held: { agent: string; change: string; overBy: number }[]
}
```

**Evidence and novelty.** `g-engineering-analogies.md`, "Tolerance budgets instead of isolated targets," is the source. It warns that summing component p95 values does not give the path's p95, and the hatched segment keeps that warning on screen. `f-frontier.md` marks "Budgets as agents' objectives" as partly taken, by Autoresearch and Chi. The part it calls unestablished is a latency objective on a system entity the agent cannot edit. The stack adds one step. The objective sits on the path, and a local improvement that spends another hop's share is refused.

**Risk and the test that kills it.** A visible decomposition invites false confidence, which the analogies report states directly. The mock task is to tighten the payout path to 200 ms and ask the engineer what the verifier's queue wait does to that number. The concept dies if the engineer answers with a point value. The honest answer is "it is outside the model, between 4 s and 30 s," and the stack has to make that the easiest answer to give.

## 5. The bench

A choice opens as a small set of funded experiments on the plant, all under the same frozen checks. The engineer funds the cheapest one that can change their mind, and picks a result instead of an agent's sentence.

**Pain.** Pain 1. The prompt-then-diff loop asks the engineer to accept an interpretation. The bench asks them to fund a measurement and then accept a row in a table.

**Sixty seconds on Acquit.** The engineer presses `J` and lands on choice 2 of 5, risk 0.36, the highest on the ledger. The panel in `c7-review.png` would show "Raise verifier concurrency from 1 to 2" as text beside two more sentences. Here the plant dims to the verifier bank and its "concurrency 1" valve, and three ghost copies of the valve line up, labeled from the fixture. "Keep 1, shorten cold start." "Raise to 2." "Second lane, plus 10 ports." Each carries a cost chip. A replay of yesterday's 212 jobs, the same replay plus a missing race test, and a full stack copy. The engineer clicks the first chip and presses `F`. A crane replays the recorded traffic and the gauge grows a second needle, landing at 28 s against the 30 s budget, inside the wide band the fixture warns about. The middle chip changes from a guess to "would not beat this unless cold start is under 6 s." The engineer presses `1`. The other ghosts fold into the tray, and the choice closes with the replay attached instead of an accepted sentence.

**What it replaces.** The alternatives list in `c7-review.png`, where each option is a sentence the agent weighed and the engineer accepts on trust. The bench keeps the sentences as labels and puts a measurement under each one.

**Data.**

```ts
type Bench = {
  choiceId: string
  checksFrozen: string[]          // ids, agents cannot edit, K16
  arms: {
    id: string
    label: string                 // from CHOICES[].alts
    cost: { kind: "replay" | "stack-copy" | "live"; estimate: string }
    result: { metric: string; value: number; band: [number, number] } | null
    assumption: string | null     // "cold start under 6s", or null
  }[]
  funded: string | null
}
```

**Evidence and novelty.** `a-landscape.md`, "Decision experiment bench," frames this as buying information before funding construction. `d-intent-and-verification.md`, "Experiment branches before implementation," adds the rule that agents compete on constraint satisfaction rather than prose. `b-games-softvis.md`, "Pareto workshops," supplies the comparison form. `f-frontier.md` marks "Calibrated simulation of a change" as partly taken, by Palladio and StrongDM, and marks the risk-ranked choices ledger as open. The bench joins the two. An entry ranked by `1 - confidence` times reversal cost opens onto a funded comparison whose confidence is a replay with a named band, not the agent's `conf: 0.4`.

**Risk and the test that kills it.** A weak contract makes every arm look successful, which `a-landscape.md` names as the failure mode. The mock task is to fund the concurrency bench with the budget check removed. The concept dies if the engineer can still press `1`, `2`, or `3` and close the choice. With no frozen check, the bench is three ghost valves and a guess, and it should refuse to close.

## 6. The issued sheet

The engineer issues a territory as a revisioned sheet with stable ids, and an agent's question comes back as a readback pinned to one id. Work proceeds against the issued revision, so a later edit to the drawing cannot silently change what an agent was told.

**Pain.** Both. Pain 1, because the instruction is a set of ids and constraints the engineer issues once, not prose rewritten per session. Pain 2, because two agents can be handed the same revision and their disagreements surface against it, instead of against whatever each one remembers.

**Sixty seconds on Acquit.** Radar-otel is verifying under Droid, two of five checks failing. The engineer presses `I`. The territory boundary thickens and stamps "rev 3" at its corner, listing four entity ids, two locked budgets, and one open question. "How does trace context survive the effects queue?" A pin appears on the belt from the API to the verifier, the one that crosses the queue, with two readings. "Reading A, queue rows carry traceparent. Reading B, a new trace starts per tick." The engineer clicks Reading A and presses Enter. The pin inks "rev 3, reading A, acknowledged," and Droid's readback prints beneath it. "Rows carry traceparent, spans link across the gap, rev 3." It matches, so Droid continues check 4 without a second confirmation. The engineer then drags the radar ghost in the blueprint. That lands as rev 4, unissued, and Droid keeps building against rev 3. The corner shows both numbers until rev 4 is issued. The assignment and the new idea no longer share a silent current state.

**What it replaces.** The territory as one live YAML file that both the human and the agent read and write, which is what `TERRITORIES` and K19 imply today. Issuing splits it into a revision the agent holds and a draft the engineer edits. The choice panel's free-text question becomes a pin against one id on the issued sheet.

**Data.**

```ts
type IssuedSheet = {
  territory: string
  rev: number
  issuedAt: string
  entities: string[]              // stable ids
  budgets: string[]               // locked, K16
  openQuestions: { id: string; on: string; text: string }[]
}
type Readback = {
  sheetRev: number
  questionId: string
  agent: string
  chosenReading: string
  acknowledged: boolean           // true only when the reading matches the issued answer
}
```

**Evidence and novelty.** `g-engineering-analogies.md`, "Issued-for-agents drawing sets," grounded in NASA configuration baselines, and "RFIs with an agent readback," grounded in Autodesk's split between a reviewer response and an official response. Clark and Brennan, cited there, treat common ground as updated coordination rather than shared possession of a file. `f-frontier.md` marks "Shared drawing exposed through MCP" as taken, by tldraw, Figma, and Archyl. A shared drawing is exactly what this concept refuses to treat as enough. The unestablished part is an issued revision plus a readback that gates action, so an agent cannot proceed on a reading the engineer did not choose.

**Risk and the test that kills it.** Mandatory readbacks on routine edits become ceremony, which `g-engineering-analogies.md` flags. In the mock, the engineer changes one word of a label inside radar-otel. The concept dies if issuing rev 4 demands a readback for that change. Readbacks belong on questions with reversal cost. A sheet that demands one for every edit will be bypassed, and a bypassed sheet is a YAML file with extra steps.

## Ranking

1. **The standing order.** A machine posts the condition, and agents claim it instead of waiting for a prompt.
2. **The clash pin.** Two agents' intentions meet on the shared belt before either becomes code.
3. **The bench.** The engineer funds a measurement and accepts a result, not a sentence.
4. **The issued sheet.** Work is pinned to a revision, so a later edit cannot silently retarget an agent.
5. **The margin stack.** One path number refuses local wins that spend another machine's share.
6. **The shift.** The engineer schedules their absence, and agents plan against the decisions waiting at return.

The standing order leads because it deletes the most repeated action, typing the goal. The clash pin is second because it catches the most expensive failure at the cheapest moment, two agents building incompatible systems. The shift is last because it depends on the others. A shift can only hold decisions the plant already knows how to name.

## The pair to build

Build the standing order and the clash pin together. The order lets agents act without a fresh prompt. The pin stops two of them acting on contradictory readings of the same belt, which removes the conflict. Neither works alone. Orders without pins reproduce the PayPal failure, where Droid's breaker and Codex's retry each look correct and together double-charge a payout. Pins without orders leave the engineer writing the prompt the pin was meant to replace. The bench, the sheet, the stack, and the shift attach later. Each assumes the plant already knows what an agent is trying to do and what it must not break.
