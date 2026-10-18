// Shared fixture for the two core concepts. Times are minutes relative to REPLAY.now (16:30 today),
// matching REPLAY.series in ../acquit.mjs. Every number here is scripted for the mock.

export const SHEET = { rev: "3f2a91c", sheet: 1, label: "sheet 1 · 3f2a91c" };

// A witness is a real moment pinned by the human. Polarity decides whether it must never recur or must keep working.
export const WITNESSES = [
  {
    id: "w1", polarity: "must-not", entity: "retry", t: -45.5, clock: "15:44",
    observed: { metric: "loop depth", value: 3, unit: "of 3" },
    context: "paypal at 31% errors",
    reason: "Every failed payout loops to its third try.",
    check: "replay 15:44 · retry loop depth ≤ 2",
  },
  {
    id: "w2", polarity: "must-keep", entity: "paypal", t: -55, clock: "15:35",
    observed: { metric: "tries", value: 1, unit: "try" },
    context: "payout p-2291",
    reason: "Payout p-2291 succeeded on its first try and settled once.",
    check: "replay 15:35 · p-2291 settles exactly once",
  },
];

// Raised when two readbacks disagree. The answer becomes a third witness and a locked check.
export const QUESTION = {
  id: "q1", entity: "sqlite", layer: "effects queue rows", t: -45.2, clock: "15:45",
  text: "PayPal accepted payout p-2307, but the response was lost. What should happen to it?",
  options: [
    {
      key: "1", label: "Keep it pending, settle once after PayPal recovers",
      witness: { id: "w3", polarity: "must-keep", entity: "sqlite", t: -45.2, clock: "15:45", observed: { metric: "state", value: "pending", unit: "" }, context: "payout p-2307", reason: "A payout whose response was lost stays pending and settles once after recovery.", check: "replay 15:45 · p-2307 pending, then settled once" },
    },
    {
      key: "2", label: "Mark it failed and refund the client",
      witness: { id: "w3", polarity: "must-keep", entity: "sqlite", t: -45.2, clock: "15:45", observed: { metric: "state", value: "failed", unit: "" }, context: "payout p-2307", reason: "A payout whose response was lost is marked failed and refunded.", check: "replay 15:45 · p-2307 failed, refund issued" },
    },
  ],
};

export const ORDER = {
  id: "ord-payouts-1", on: "paypal", territory: "paypal-payouts",
  placeholder: "Payouts survive a PayPal outage without retry storms, and never charge twice.",
  budgets: ["paypal.error_rate < 2%"],
  sheet: SHEET.label,
  cannotSee: [{ agent: "Codex", why: "passive session, no MCP, so it cannot read orders" }],
};

// Readbacks arrive after the order is posted. Each is the agent's interpretation, replayed against the witnesses before any code.
export const READBACKS = [
  {
    session: "claude-payouts", agent: "Claude Code", bot: "bot-cc", arriveMs: 1400,
    text: "Stop at 2 tries with jitter. If PayPal accepted a payout but the response was lost, keep it pending and reconcile it by payout id after recovery.",
    conflict: "keep it pending",
    replay: { w1: "pass", w2: "pass" }, answers: { q1: "1" },
    footprint: ["retry", "paypal", "sqlite"],
  },
  {
    session: "droid-payouts", agent: "Droid", bot: "bot-d2", arriveMs: 2600,
    text: "Stop retrying after the first failure and mark the payout failed.",
    conflict: "mark the payout failed",
    replay: { w1: "pass", w2: "pass" }, answers: { q1: "2" },
    footprint: ["retry", "paypal"],
  },
];

// Declarations are what each claimed agent says it will change, before it writes. Clash on the belt starts from here.
export const DECLARATIONS = [
  {
    id: "d1", session: "claude-payouts", agent: "Claude Code", territory: "paypal-payouts", order: "ord-payouts-1", sheet: SHEET.label,
    touches: ["retry", "paypal", "sqlite"], belts: ["api->paypal"],
    writes: [
      { entity: "retry", part: "code", path: "packages/core/src/retry.ts", source: "declared" },
      { entity: "sqlite", part: "effects queue rows", path: "packages/core/src/effects.ts", source: "declared", detail: "adds payout_state column" },
      { entity: "api", part: "code", path: "apps/api/src/server.ts", source: "static" },
    ],
    asserts: "Retry x2 with jitter. A lost response marks the effect row pending.",
  },
  {
    id: "d2", session: "droid-radar", agent: "Droid", territory: "radar-otel", order: null, sheet: SHEET.label,
    touches: ["radar", "api", "sqlite"], belts: ["api->radar"],
    writes: [
      { entity: "sqlite", part: "effects queue rows", path: "packages/core/src/effects.ts:246", source: "history", detail: "adds traceparent column" },
      { entity: "api", part: "code", path: "apps/api/src/server.ts", source: "history" },
    ],
    asserts: "Queue rows carry traceparent, and spans link across the gap.",
  },
  {
    id: "d3", session: "droid-radar", agent: "Droid", territory: "paypal-payouts", order: null, sheet: SHEET.label, proposal: "breaker",
    touches: ["breaker"], belts: ["api->paypal"],
    writes: [{ entity: "breaker", part: "code", path: "packages/core/src/paypal.ts", source: "declared" }],
    asserts: "Breaker trips after 5 fails in 30 s, then fails fast.",
  },
];

export const CLASHES = [
  {
    id: "c1", severity: "blocking", at: { belt: "api->paypal", entity: "breaker" }, decls: ["d1", "d3"],
    consequence: "When the breaker is open, the retry's second try fails fast, so payout p-2307 is marked failed instead of pending.",
    breaks: "w3",
    suggested: "split",
    resolutions: [
      { kind: "split", key: "1", label: "Split at the port", detail: "Publish the contract paypal.result v1 as ok, failed or unknown. An open breaker returns unknown, and the retry keeps unknown as pending.", owner: "Claude Code" },
      { kind: "left", key: "2", label: "Retry wins", detail: "The breaker ghost waits until the payout order passes." },
      { kind: "right", key: "3", label: "Breaker wins", detail: "Claude Code yields the belt and replans around a fail-fast breaker." },
      { kind: "serialize", key: "4", label: "Serialize", detail: "The breaker starts after the payout order passes its checks." },
    ],
  },
  {
    id: "c2", severity: "blocking", at: { entity: "sqlite", layer: "effects queue rows" }, decls: ["d1", "d2"],
    consequence: "Two migrations change the effects table. The second one to land must be rewritten on top of the first.",
    breaks: null,
    suggested: "serialize",
    resolutions: [
      { kind: "serialize", key: "1", label: "Serialize, radar-otel first", detail: "paypal-payouts already depends on radar-otel. Claude Code builds its migration on the traceparent column." },
      { kind: "owner", key: "2", label: "One migration owner", detail: "Droid writes both columns in one migration. Claude Code reads the schema it publishes." },
      { kind: "merge", key: "3", label: "Merge the territories", detail: "One territory, one agent, one migration." },
    ],
  },
  {
    id: "c3", severity: "advisory", at: { entity: "api" }, decls: ["d1", "d2"],
    consequence: "Both edit apps/api/src/server.ts, in different routes. No shared contract changes.",
    breaks: null, suggested: null, resolutions: [],
  },
];

// Scripted Kvarm fixture. Kvarm is an opt-in connection to the user's own origin, so every op here is declared egress.
// Bytes, latencies and ages are invented for the mock. Each concept inlines the part it uses, and nothing calls the network.
// The hash flag kv=ok|unmatched|stale|offline picks the script. No flag is kv=ok.
export const KVARM = {
  origin: "https://git.orrery.local", host: "git.orrery.local", repo: "acquit",
  staleAfter: "two missed polls, or at once on disconnect",
  orders: {
    issue: 41, task: "t-41", generation: "g1", leaseS: 600, heartbeatS: 120,
    ops: {
      create_issue: { bytes: 612, ms: 180, reason: "Mirror order r1 and its witness ids" },
      dispatch_issue: { bytes: 148, ms: 260, reason: "Open one claimable fix task" },
      get_task: { bytes: 96, ms: 160, reason: "Poll the lease on t-41" },
      comment_issue: { bytes: 296, ms: 200, reason: "Add w3 and bump the order to r2" },
      claim_task: { bytes: 184, ms: 140, reason: "own token · readback matched r2" },
      heartbeat_task: { bytes: 88, ms: 160, reason: "Keep lease g1 live" },
      check_heads_and_leases: { bytes: 384, ms: 300, reason: "Read heads and leases before any held write" },
    },
    checked: "sheet head 3f2a91c unchanged · no live lease on t-41",
    unmatched: { holder: "Codex", session: "codex-paypal", evidence: "authenticated", pollMs: 700 },
    stale: { heartbeatMs: 1600, code: "STALE_GENERATION" },
  },
  clash: {
    lease: { holder: "Claude Code", session: "claude-payouts", generation: "g1", evidence: "authenticated", task: "t-41" },
    reads: [
      { op: "get_intent", bytes: 412, reason: "Read declared intents for paypal-payouts" },
      { op: "forecast", bytes: 538, reason: "Forecast path overlap for d1 and d3" },
    ],
    cachedAgeS: 12,
    forecast: {
      c1: { ageS: 12, text: "no path overlap", detail: "retry.ts and paypal.ts are different files" },
      c2: { ageS: 12, truncated: true, detail: "stopped at 2 import hops" },
      c3: { ageS: 12, text: "overlap", detail: "apps/api/src/server.ts" },
    },
    ops: {
      record_decision: { bytes: 356, ms: 260 },
      set_intent: { bytes: 228, ms: 200 },
      git_push: { bytes: 1180, ms: 320 },
      check_heads_and_leases: { bytes: 384, ms: 300, reason: "Read heads and leases before any held write" },
    },
    checked: "sheet head 3f2a91c unchanged · lease g1 live",
    choices: { c1: "choices/c1.yaml", c2: "choices/c2.yaml" },
    settle: {
      c1: {
        split: { push: "contracts/paypal.result.v1.yaml", intents: [["d1", "Claude Code writes through paypal.result v1"], ["d3", "breaker returns unknown through v1"]] },
        left: { intents: [["d3", "breaker waits for the payout order"]] },
        right: { intents: [["d1", "Claude Code yields the belt"]] },
        serialize: { pin: true },
      },
      c2: {
        serialize: { pin: true },
        owner: { intents: [["d1", "Claude Code drops its migration"]] },
        merge: { push: "blueprint/territories.yaml" },
      },
    },
  },
};
