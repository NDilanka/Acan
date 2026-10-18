# Acan concept bakeoff, Anthropic arm

Dated 2026-10-17. Five concepts that change what the engineer does, not how the plant looks.

**Basis.** I read map.md K1 to K20, the four screenshots, acquit.mjs, ui-v1 DECISIONS.md and actions.mjs, the summary and Ideas sections of all seven slices, and f-frontier.md's novelty and white space sections. Keys reuse ui-v1 where it has them. New keys need checking against actions.mjs:5-10.

## 1. Witnesses instead of prompts

**Pitch.** The engineer stops describing the fix. They pin a real moment on the plant, mark what must never happen again and what must keep working, and Acan issues those marks to the agent as its contract.

**Scenario.** Replay (key 4) on "today 15:30 to 16:30" (acquit.mjs:249). The paypal-payouts territory is unclaimed and fogged at 0.42 (acquit.mjs:146-150).

- 0 to 10 s. Scrub to about 15:44. The retry machine reads "loop 3/3", and the paypal dock reads 31 percent errors (acquit.mjs:256-260). Click the retry machine.
- 10 to 25 s. Press M for must-not. The mock fills a reason from REPLAY.because.retry (acquit.mjs:271). It reads "Every failed payout loops to its third try." Enter accepts. A red pin appears on the retry machine.
- 25 to 35 s. Scrub back to 15:20. Click a payout that succeeded on its first try and press Shift M for must-keep. A green pin appears beside it.
- 35 to 50 s. Tab opens the contract plate on paypal-payouts. It lists both witnesses, the budget paypal.error_rate under 2 percent, and a simulated witness from the breaker ghost at confidence 0.35 (acquit.mjs:242-244). Arrow keys set retry tries to 2, and Enter issues the contract. The plate reads "ready after radar-otel", the dependency the territory records (acquit.mjs:148).
- 50 to 60 s. Droid claims the territory after radar-otel passes. Its candidate replays against both witnesses on the plant. The retry machine must stay green on both pins. Only then does the engineer press O to open the diff.

**What changes on the plant.** Pins sit on machines, and the contract plate sits on the territory. A candidate appears as a ghost copy of the retry machine, with pass or fail on each pin.

**Pain and loop.** Attacks pain 1. Today the engineer types "back off retries with jitter", reads a diff, and judges it by reading. Here the engineer points at a moment and marks its polarity. Acceptance is a replay of that moment, so the diff opens last.

**Data shape.**

```ts
type Witness = {
  id: string;
  polarity: "must-not" | "must-keep";
  entity: string;                        // machine id, e.g. "retry"
  source:
    | { kind: "replay"; at: number }
    | { kind: "simulate"; run: string }  // always labeled as a prediction
    | { kind: "live"; traceId: string };
  observed: { metric: string; value: number; unit: string };  // loop depth 3
  reason: string;                        // one line
  protected: boolean;                    // only a human sets true (K16)
  revision: string;                      // revision the moment belongs to
};

type Contract = {
  territory: string;
  witnesses: Witness[];
  budgets: string[];
  state: "draft" | "issued" | "stale";   // stale when the revision moves
};
```

**Evidence.** d-intent-and-verification.md:74-84 (Idea 1, counterexample-directed repair, where the runtime event "becomes both the instruction and a review obligation") and :107-117 (Idea 4, evidence-bound acceptance, failing closed on revision change). For pain, see f-frontier.md:77-93.

**Novelty against f-frontier.** Partly taken. Trace replay and evals already sit beside CI gates (f-frontier.md:49-51). The new part is a human marking a runtime moment with polarity, so the mark is both the instruction and the acceptance rule for a protected territory. It does not claim the shared-drawing protocol, which f-frontier.md:155 marks as taken.

**What it replaces in ui-v1.** Copy brief and Hand to agent for fix work (actions.mjs:80-112), and Replay's Copy brief (replay.mjs:245-247), which becomes Pin witness. Choice cards stay as decisions, and acceptance cites witnesses.

**Biggest risk and falsifiable test.** Witnesses only capture behavior already in traffic, so a new feature has no moment to pin. Simulated witnesses also inherit model error (K13).

Test. Clickable Replay mock with scripted candidates. Eight to ten developers each get the task "make payouts survive the PayPal outage". Arm A has a text box. Arm B has pins. Each arm gets three scripted candidates. In each arm, one candidate passes a hidden case. The case is a payout that succeeds on its second try, and it must still complete. The concept dies if Arm B does not beat Arm A by at least 25 points on the hidden case, or if fewer than seven of ten participants pin the 15:44 retry within two minutes.

## 2. The inspection gate holds the work

**Pitch.** The engineer sets how much review they can absorb each hour. Acan admits new agent work only inside that allowance. Held work waits on a scaffold in front of the inspection station, so the backlog is physical.

**Scenario.** Live, default plant.

- 0 to 10 s. The inspection station "you" (acquit.mjs:19) carries a brass gauge reading "review 31 of 40 min this hour". It sums the high estimates at the gate.
- 10 to 20 s. Press ] twice to drop the allowance to 30. The gauge goes over. The belt from the verifier turns amber and backs up. The next queued agent job moves onto a scaffold plated "held, review allowance".
- 20 to 40 s. Click the scaffold. Three held proposals list with review-minute ranges and a basis label. Traceparent queue rows take 4 minutes, basis "history" (acquit.mjs:217-222). Verifier concurrency 2 takes 12 minutes, basis "guess" (acquit.mjs:205-210). The webhook exemption takes 15 minutes and has high reversal cost (acquit.mjs:199-204).
- 40 to 50 s. Enter releases traceparent, and the gauge rises to 35. D moves verifier concurrency to the next window. The webhook exemption waits in a red-edged slot, because high-cost jobs are never auto-admitted.
- 50 to 60 s. The traceparent job becomes a live bot on the belt. The gauge reads "next window in 14 min".

**What changes on the plant.** The station gains a gauge. Belts back up in amber. Held work is scaffolding, not a count.

**Pain and loop.** Attacks pain 2, the human as bottleneck. Today the engineer answers cards as they arrive, and Hand to agent (actions.mjs:109-112) starts work with no check on whether the engineer can absorb its output. Here the engineer sets the rate at which agent output may reach them, and starts follow that rate.

**Data shape.**

```ts
type ReviewAllowance = {
  station: string;            // "you" in acquit.mjs
  windowMin: number;          // 60
  allowanceMin: number;       // set by the human
  admittedMin: number;        // high estimates admitted this window
  setAt: number;
};

type HeldJob = {
  id: string;
  session: string;            // "droid-gate", "droid-radar", ...
  territory: string | null;
  reversalCost: "low" | "medium" | "high";
  estReviewMin: { low: number; high: number; basis: "history" | "guess" };
  state: "held" | "admitted" | "deferred";
};

// High reversal cost is never auto-admitted. The allowance never lowers evidence requirements.
const admit = (j: HeldJob, a: ReviewAllowance) =>
  j.reversalCost !== "high" && a.admittedMin + j.estReviewMin.high <= a.allowanceMin;
```

**Evidence.** a-landscape.md:121-130 (Idea 2, review-capacity dispatch, "letting the human bottleneck govern dispatch"). e-attention.md:179-189 (Idea 5, "holding a task is preferable to silently accepting its output"). b-games-softvis.md:137-145 (Idea 3, review backpressure). For pain, see the section "What the measured evidence actually says" in d-intent-and-verification.md. LinearB reports AI-generated PR pickup over 16 hours on average. Those are workflow durations, not reading time.

**Novelty against f-frontier.** Partly taken. Codex has an automation review queue, and cmux shows attention signals (a-landscape.md:121-130). f-frontier.md:175-181 names "a decision queue that conserves human attention" as white space. The open part is admission control, where the human's review allowance decides when agents start, drawn as backpressure on the plant.

**What it replaces in ui-v1.** The passive "Waiting on you" button (build.mjs:236) becomes the gauge. The Later pin (choices.mjs:251) becomes defer to next window. Hand to agent keeps its shortcut and asks the gate first.

**Biggest risk and falsifiable test.** Review-minute estimates are guesses until history exists. The gate can also add delay without adding safety.

Test. Clickable Live mock with a scripted 30-minute stream of 12 proposals from three agents. Four carry planted defects the mock logs. Ten participants run two arms. One is the ui-v1 queue with no allowance. The other is an allowance they set within three minutes. The concept dies if the allowance arm accepts more planted defects than the no-allowance arm, or if half or more of held jobs are released within five minutes of being held, which would mean the gate only delays.

## 3. Footprints clash before anyone writes

**Pitch.** Each agent declares the machines, belts and tables it will change before it writes. Overlaps show as clashes on the plant. The engineer settles them by ordering the work or splitting out an interface, not by repairing a merge later.

**Scenario.** Live, Acquit, with radar-otel verifying and paypal-payouts unclaimed.

- 0 to 10 s. Press F. Each territory's declared footprint draws as a translucent outline. radar-otel covers radar, api, sqlite and verifier (acquit.mjs:137-139). paypal-payouts covers paypal, retry and breaker (acquit.mjs:146-148).
- 10 to 25 s. A brass hatch appears where two writes meet on the sqlite "effects queue rows" layer (acquit.mjs:9) and the api effects path. radar-otel's write is the traceparent column, with evidence at packages/core/src/effects.ts:246 (acquit.mjs:217-222). The breaker's write is the api to paypal belt (acquit.mjs:126-129).
- 25 to 40 s. Press Shift 1 to serialize. The mock offers it as the default, because paypal-payouts already depends on radar-otel. Press Shift 2 to split instead, because the breaker must read the trace column. A contract ghost, "effects row v2", appears on the rows layer, owned by radar-otel.
- 40 to 50 s. Drag the breaker footprint off the rows layer. The hatch clears only if the breaker stops writing rows. Enter confirms the new declaration.
- 50 to 60 s. Both agents receive their footprints and the contract. The clash becomes a sequence arrow and a resolved mark.

**What changes on the plant.** Footprints are outlines. Clashes are hatches. A serial order is an arrow on the dependency. An interface is a ghost port.

**Pain and loop.** Attacks pain 2, conflicts. Today a clash appears after two agents have written diffs, or never appears when Git merges cleanly. Here ownership and order are set before code exists.

**Data shape.**

```ts
type Footprint = {
  session: string;
  territory: string;
  writes: Array<{
    entity: string;
    part?: "tables" | "effects" | "belt" | "budget" | "code";
    path?: string;                 // a file, when known
    source: "declared" | "static" | "history";
  }>;
  revision: string;                // revision the declaration was made against
};

type Clash = {
  id: string;
  at: Array<{ entity: string; part?: string }>;
  sessions: [string, string];
  severity: "blocking" | "advisory";
  resolution?:
    | { kind: "serialize"; first: string }                     // territory id
    | { kind: "interface"; contract: string; owner: string }
    | { kind: "merge"; territory: string };
};
```

**Evidence.** g-engineering-analogies.md:94-103 (Idea 4, semantic clash detection, "declaring intended writes and assumptions before implementation"). a-landscape.md:131-139 (Idea 3, collision rehearsal). g-engineering-analogies.md:159-169 (Idea 10, revision preconditions on contract edits). For pain, see a-landscape.md:81-90.

**Novelty against f-frontier.** Partly taken. f-frontier.md:148 rates "agents shown in the system they change" as partly taken, and f-frontier.md:155 says a shared drawing over MCP is taken. The new part is declared footprints drawn on the plant, with clashes resolved by order or interface before any write.

**What it replaces in ui-v1.** The per-session file list (acquit.mjs:158) and the Floor inspector's "where it works" list become declarations made before work. The Branch diff action (actions.mjs:121) stops being the first place a collision shows.

**Biggest risk and falsifiable test.** Declared footprints are guesses, and static analysis can miss or invent overlaps. The source label must stay visible on every outline.

Test. Clickable mock with three scripted scenarios. A has a true overlap on the sqlite rows layer. B has a decoy that shares a module name but no entity. C overlaps only on a budget. Eight to ten participants classify each as serialize, split or run. The concept dies if more than four of ten call the decoy a clash, or if two or more of ten miss the true overlap in A.

## 4. Leave permits

**Pitch.** The engineer leaves by signing a permit. It fences a territory, lists what agents may do inside it, and sets a return time. The plant stops agents at the fence and explains each stop on return.

**Scenario.** Live. Droid is on radar-otel, verifying, with CI #418 at 3 of 5 (acquit.mjs:154-162, :191).

- 0 to 12 s. Press L. A dotted fence wraps radar, api, sqlite and verifier (acquit.mjs:137-139). The permit card lists actions. Run checks and edit inside the fence are on. Open draft PR is off. Budget and check changes stay locked under K16. The gate is off, because its seal is stale (acquit.mjs:143).
- 12 to 25 s. Arrow keys set the return time, 45 minutes by default. Two stop rules sit beneath. One stops after three failed checks. The other stops above $3 of spend, and spend is $1.92 now (acquit.mjs:159).
- 25 to 35 s. Enter signs. The territory shows a tag, "permit to 15:15". The engineer leaves.
- 35 to 60 s. The absence is compressed, so twenty minutes of work plays in about 20 seconds. CI #418 passes 5 of 5. Droid edits three files inside the fence. At minute 12 it tries to change the api p99 budget. The fence refuses the write, and a red mark appears on the api machine. On return, Enter on the tag opens the return brief. It lists what passed, what changed inside the fence, the one stop with its reason, and the decision still waiting.

**What changes on the plant.** A dotted fence overlays the territory. A tag shows the return time. A stop is a red mark on the refused machine.

**Pain and loop.** Attacks pain 2, waiting on the human and the attention it costs. Today the engineer watches the agent or reads a transcript on return. Here the engineer signs a bounded absence and returns to a checked handover.

**Data shape.**

```ts
type LeavePermit = {
  id: string;
  territory: string;
  fence: string[];                        // machine ids inside the boundary
  allow: Array<"run-checks" | "edit-in-fence" | "open-draft-pr">;
  // No permit can allow a budget or check change (K16).
  stops: Array<
    | { when: "fence-cross" }
    | { when: "protected-write" }
    | { when: "checks-failed"; n: number }
    | { when: "spend-over"; usd: number }
  >;
  returnBy: number;
  state: "active" | "ended" | "stopped";
};

type ReturnBrief = {
  permit: string;
  passed: string[];                       // check runs that passed while away
  changed: Array<{ entity: string; files: number }>;
  stops: Array<{ at: number; entity: string; attempted: string; reason: string }>;
  waiting: string[];                      // choice ids
};
```

**Evidence.** e-attention.md:134-144 (Idea 1, safe to leave, where the promise is "no unapproved consequential action", not "nothing will go wrong"). c-supervisory-control.md:151-160 (Idea 3, which separates enforced rules from advisory instructions). e-attention.md:156-166 (Idea 3, the return brief). For pain, see f-frontier.md:60-76.

**Novelty against f-frontier.** Partly taken. Scheduled autonomous runs already ship, as Antigravity's schedules show (f-frontier.md:43). The new part is a permit as a plant object, with a fence, protected-action exclusions, runtime-enforced stop rules, and a return brief tied to machines. f-frontier.md:175-181 leaves the full workflow open.

**What it replaces in ui-v1.** The Floor timeline as a returning engineer's first stop (build.mjs:240, palette.mjs:57) becomes the return brief. The "Snooze 10 min" control (floor.mjs:193) gives way to a return time set once.

**Biggest risk and falsifiable test.** Permits can be stiff, causing needless stops, or loose, causing silent breaches. The larger risk is false reassurance, where a green tag reads as "nothing happened".

Test. The scripted 20-minute absence has a permitted edit at minute 6, a protected budget write at minute 12, and a third failed check that stops the agent. Ten participants sign, leave and return, and see only the return brief. Each states what the agent did inside the fence, what it tried outside, and what waits. The concept dies if fewer than seven of ten state the refused write within 60 seconds, or if any participant calls an action the permit never allowed "safe".

## 5. Split the path budget into shares

**Pitch.** The engineer states one end-to-end target for a request path and divides it into shares on each machine. Each agent gets its share as a contract, and each proposal shows how much of the path's margin it uses.

**Scenario.** Replay (key 4) on the outage window.

- 0 to 10 s. Press P, then click intake and the paypal dock. The chain lights as a path. Type the target, "payout request p99 under 800 ms".
- 10 to 25 s. The path bar shows gate 5 ms (budget, acquit.mjs:235), api 250 ms (budget, acquit.mjs:46) and paypal 400 ms (measured, acquit.mjs:240). Slack is 145 ms. A red tick sits on the 15:44 outage, where api p99 peaks at 2.6 s (acquit.mjs:254). Three 10-second tries hold api's slots and stretch the paypal tail to 30 s (acquit.mjs:240, :272).
- 25 to 40 s. Set the retry step to 2 tries with the arrow keys. The tail drops to 20 s, but the path stays over target, and the bar marks the breaker as the only fitting that can close the gap. Press B and place the breaker ghost on the api to paypal belt (acquit.mjs:242-244).
- 40 to 55 s. Drag the boundary to give paypal a 300 ms p99 share with a 20 s tail cap, and keep api at 250 ms. Enter issues the shares. paypal-payouts receives its grant, and the breaker goes to the territory as a proposal.
- 55 to 60 s. Droid returns a candidate that reads "uses 40 of 145 ms slack, path p99 under outage 720 ms, replay". The engineer accepts on margin and only then opens the diff.

**What changes on the plant.** A path is a highlighted belt chain with a target line. Shares are segments, breaches are ticks at their replay time, and each candidate carries a margin bar.

**Pain and loop.** Attacks pain 1. The engineer states one number for the whole request and divides it, instead of writing "make payouts fast". Each agent gets a checkable share. Per-machine budgets are checked one at a time today, so a green gauge can sit on a red path.

**Data shape.**

```ts
type PathBudget = {
  id: string;
  path: string[];                    // machine ids in order
  target: { metric: "p99"; ms: number; window: string };
  shares: Array<{
    entity: string;
    ms: number;
    tailMs: number;                  // worst case, e.g. three tries of 10 s
    basis: "measured" | "budget" | "guess";
  }>;
  slackMs: number;                   // target minus the sum of ms
  set: "human";
};

type ShareGrant = {
  territory: string;
  entity: string;
  maxMs: number;
  maxTailMs: number;
  revision: string;
};
```

**Evidence.** g-engineering-analogies.md:105-115 (Idea 5, tolerance budgets, including "Summing component p95 values does not establish the path's p95"). K11 and K16 (map.md:36, :41). Path facts come from acquit.mjs:240 and 270-274.

**Novelty against f-frontier.** Partly taken. f-frontier.md:150 rates budgets as agents' objectives as partly taken. Allocating one path target across machines, with tails and grants to agents, does not appear in the reviewed sources. I did not check operations practice on latency budgets this session, so this claim covers only the reviewed slices.

**What it replaces in ui-v1.** Per-machine budget rows in the inspector (acquit.mjs:45-47, :66-68, :82-83) become path shares. The per-machine Simulate budgets (acquit.mjs:235-240) roll up into the path view.

**Biggest risk and falsifiable test.** Local shares may not compose into path behavior, because queues and retries correlate. If so, the green state misleads, and the central claim fails.

Test. Replay mock with three scripted outage windows from the Acquit fixture. Ten participants split an 800 ms target across the path until the view turns green. The mock then replays each green allocation. The concept dies if the allocations the view marks green breach the replayed target in two or more of the three outages.

## Ranking

1. **Witnesses instead of prompts.** The strongest answer to pain 1, and the most falsifiable, because it extends ui-v1's replay.
2. **The inspection gate holds the work.** The clearest change to who starts work, aimed at the human bottleneck. Partly taken, but the physical backlog is new.
3. **Footprints clash before anyone writes.** Targets conflicts directly. Its value depends on how often territories share machines.
4. **Leave permits.** Strong for attention, but closest to scheduling precedents that already ship.
5. **Path budget shares.** A good idea whose central claim rests on path arithmetic the research warns about.

## Recommended pair

Witnesses and the inspection gate. Witnesses make acceptance a replay, so the review minutes the gate uses come from history, not guesses. Without witnesses, the gate reduces to a count. The gate makes witnesses matter at scale, because a witnessed proposal carries "replay passed, reviewed in minutes", and the human decides how many such proposals enter each window. Footprints can join later, since radar-otel and paypal-payouts already meet on the api effects path.
