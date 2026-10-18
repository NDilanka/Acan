// Fixture for the inspection gate. Every number is scripted for the mock.
// Times are minutes relative to REPLAY.now (16:30 today) in ../acquit.mjs, so the half hour away
// (16:00 to 16:30) is exactly the end of the recorded Replay window and every brief row can link into it.

export const GATE = {
  windowMin: 60,
  presets: [20, 30, 40, 60],
  depart: -30,
  backAt: [
    { key: "1", label: "in 30 min", min: 30 },
    { key: "2", label: "in 1 h", min: 60 },
    { key: "3", label: "in 2 h", min: 120 },
    { key: "4", label: "tomorrow", min: 960 },
  ],

  // Consequence of waiting, worst first. Ranking sorts by this, then by fixture order.
  classes: [
    { id: "harm", label: "live harm" },
    { id: "idle", label: "agent idle" },
    { id: "reversal", label: "undo cost grows" },
    { id: "steady", label: "steady breach" },
    { id: "publish", label: "publication waits" },
    { id: "none", label: "nothing waits" },
  ],

  // before: present at departure (16:00). after: present on return (16:30).
  // A string instead of true means the item is expected but has not arrived, and the string is its forecast.
  items: [
    {
      id: "breaker", title: "Approve the paypal breaker", kind: "ghost", agent: "Droid", territory: "paypal-payouts", on: "breaker", at: -20,
      est: [4, 8], basis: "guess", basisNote: "no reviews of this kind yet", cls: "harm", before: false, after: true,
      cost: "paypal stays at 4.1% errors against a 2% budget. Three retries triple the calls in an outage.",
      safe: "Now. The budget breach is live.",
      meanwhile: "Droid keeps verifying radar-otel.",
      advice: "Droid reads this advice over MCP and decides whether to park.",
      review: { kind: "ghost", id: "breaker" },
    },
    {
      id: "codex", title: "Let Codex run npm test", kind: "permission", agent: "Codex", territory: null, on: "bot-cx", at: -9,
      est: [1, 2], basis: "history", basisNote: "terminal approvals", cls: "idle", before: false, after: true, unforecast: true,
      cost: "Codex is idle at its own approval prompt. 9 min so far.",
      safe: "Now. Each minute is idle agent time.",
      meanwhile: "Codex stopped itself at its terminal prompt. Orrery did not stop it.",
      advice: "Codex cannot read advice. Passive session, no MCP.",
      review: { kind: "session", id: "codex-paypal" },
    },
    {
      id: "traceparent", title: "Queue rows carry traceparent", kind: "choice", agent: "Droid", territory: "radar-otel", on: "api", at: -22,
      est: [2, 4], basis: "history", basisNote: "similar choices", cls: "reversal", before: false, after: true,
      cost: "Droid builds on it. Each commit makes it costlier to undo.",
      safe: "Before radar-otel seals.",
      meanwhile: "Droid keeps building radar-otel.",
      advice: "Droid reads this advice over MCP and decides whether to park.",
      review: { kind: "choice", id: "traceparent-queue-rows" },
    },
    {
      id: "databasesync", title: "Wrap DatabaseSync by hand", kind: "choice", agent: "Droid", territory: "radar-otel", on: "sqlite", at: -14,
      est: [1, 3], basis: "history", basisNote: "similar choices", cls: "reversal", before: false, after: true,
      cost: "Droid builds on it. Each commit makes it costlier to undo.",
      safe: "Before radar-otel seals.",
      meanwhile: "Droid keeps building radar-otel.",
      advice: "Droid reads this advice over MCP and decides whether to park.",
      review: { kind: "choice", id: "wrap-databasesync" },
    },
    {
      id: "radar-choices", title: "Droid's next choices in radar-otel", kind: "forecast", agent: "Droid", territory: "radar-otel", on: "radar",
      est: [3, 7], basis: "history", basisNote: "Droid's last territories", cls: "reversal", before: "within 30 min", after: false,
      cost: "Droid builds on each choice. Undo cost grows from the moment it lands.",
      safe: "Soon after it lands, before Droid builds on it.",
      meanwhile: "Droid has just claimed radar-otel and keeps working.",
      advice: "Droid reads this advice over MCP and decides whether to park.",
      review: null,
    },
    {
      id: "verifier", title: "Raise verifier concurrency to 2", kind: "choice", agent: "Droid", territory: null, on: "verifier",
      est: [8, 15], basis: "guess", basisNote: "no similar reviews", cls: "steady", before: true, after: true,
      cost: "Verifier queue wait stays at 41 s against a 30 s budget.",
      safe: "No deadline. The breach is steady, not growing.",
      meanwhile: "No agent waits on it. The session that recorded it has ended.",
      advice: "The session has ended, so no agent reads this advice.",
      review: { kind: "choice", id: "verifier-concurrency-2" },
    },
    {
      id: "webhook", title: "Exempt the webhook from basic auth", kind: "choice", agent: "Droid", territory: "gate-sessions", on: "gate",
      est: [10, 15], basis: "guess", basisNote: "high reversal cost", cls: "publish", before: true, after: true,
      kvarm: { change: "#212", rule: "deploy/Caddyfile is a protected path. One review required." },
      cost: "Change #212 cannot publish until you review it.",
      safe: "Before you need gate-sessions published. Nothing breaks while it waits.",
      meanwhile: "No agent waits on it. The session that recorded it has ended.",
      advice: "The session has ended, so no agent reads this advice.",
      review: { kind: "choice", id: "gate-webhook-exempt" },
    },
    {
      id: "seal", title: "Seal radar-otel", kind: "seal", agent: "Droid", territory: "radar-otel", on: "radar",
      est: [6, 10], basis: "history", basisNote: "past seals", cls: "publish", before: "in 30 to 50 min", after: "after CI #418 passes",
      kvarm: { change: "#209", rule: "One review required at the exact head." },
      cost: "radar-otel cannot publish until you review the seal.",
      safe: "Before you need radar-otel published.",
      meanwhile: "Droid keeps running checks. 3 of 5 pass so far.",
      advice: "Droid reads this advice over MCP and decides whether to park.",
      review: null,
    },
    {
      id: "retry", title: "Back off retries to x2 with jitter", kind: "choice", agent: "Claude Code", territory: "paypal-payouts", on: "paypal",
      est: [2, 5], basis: "guess", basisNote: "no similar reviews", cls: "none", before: true, after: true,
      cost: "Nothing waits on it. The session that recorded it has ended.",
      safe: "Before anyone claims paypal-payouts.",
      meanwhile: "No agent waits on it.",
      advice: "The session has ended, so no agent reads this advice.",
      review: { kind: "choice", id: "paypal-retry-x2" },
    },
  ],

  // What changed between 16:00 and 16:30, ranked by consequence. t is the Replay minute each row links to.
  brief: [
    {
      cls: "harm", items: ["breaker"], t: -20, sel: "paypal", on: "breaker",
      title: "Droid proposed a paypal breaker",
      what: "paypal held 4.1% errors against its 2% budget all half hour. 212 calls were retried this hour; 38 hit their third try.",
      sources: [["OTel", "paypal spans", "observed"], ["Droid", "ghost on the blueprint", "declared"]],
    },
    {
      cls: "idle", items: ["codex"], t: -9, sel: "bot-cx", on: "bot-cx", unforecast: true,
      title: "Codex is blocked at its own prompt",
      what: "Codex started on paypal.ts at 16:04 and asked to run npm test at 16:21. It is passive, so Orrery could not forecast it.",
      sources: [["Codex hooks", "approval prompt", "observed"]],
    },
    {
      cls: "reversal", items: ["traceparent", "databasesync"], t: -22, sel: "api", on: "api",
      title: "Droid recorded 2 choices in radar-otel",
      what: "Traceparent in queue rows at 16:08, confidence 0.70. DatabaseSync wrapped by hand at 16:16, confidence 0.85. Forecast 3–7 min, arrived 3–7 min.",
      sources: [["Orrery MCP", "record_choice", "declared"]],
    },
    {
      cls: "publish", items: ["webhook"], t: -16, sel: "gate", on: "gate",
      title: "gate-sessions went stale",
      what: "apps/api/src/server.ts changed at 16:14, after its seal. Kvarm still blocks change #212 on your review.",
      sources: [["Kvarm", "get_reviews #212", "authenticated"], ["Orrery", "seal check", "observed"]],
    },
    {
      cls: "none", items: ["seal"], t: -4, sel: null, on: "radar",
      title: "CI #418 is running, 3 of 5",
      what: "#416 failed 1 of 5 at 16:12. #417 failed 3 of 5 at 16:21. The seal review arrives when 5 of 5 pass.",
      sources: [["CI", "local harness", "observed"]],
    },
  ],
  unchanged: "Verifier concurrency 2 and retry x2 still wait. Their costs held steady.",
  heldNothing: "Orrery held nothing back. Droid ran 3 CI runs and recorded 2 choices. Codex stopped at its own prompt.",

  kvarm: {
    origin: "https://git.orrery.local",
    ops: [
      { op: "list_pending_reviews", bytes: 640, reason: "Which acquit changes wait on your review" },
      { op: "get_reviews", bytes: 512, reason: "Rules on change #212, gate-sessions" },
      { op: "get_reviews", bytes: 512, reason: "Rules on change #209, radar-otel" },
    ],
  },

  // The plant at departure. On return the base Acquit fixture is the truth, so these are undone.
  departure: {
    hide: ["breaker", "bot-cx", "crane"],
    hidePins: ["traceparent-queue-rows", "wrap-databasesync"],
    territory: { id: "gate-sessions", state: "passed", label: "GATE-SESSIONS · PASSED" },
    agents: { "droid-radar": ["claimed radar-otel", "8m"], "codex-paypal": ["no session yet", ""] },
  },
};
