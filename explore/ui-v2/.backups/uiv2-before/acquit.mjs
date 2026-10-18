export const ACQUIT = {
  wall: { x: 212, y0: -150, y1: 150, gaps: [[-128, -92], [-58, -22], [22, 58]], text: "PLANT BOUNDARY" },
  entities: [
    { id: "intake", mono: "IN", size: 1, kind: "intake", x: -300, y: 0, state: "ok", name: "Intake", reading: "38 sessions", sub: "web · cli/mcp · paypal webhook · verifier callback" },
    { id: "gate", mono: "CY", size: 1, kind: "gate", params: { layers: ["caddy basic auth", "app sessions", "/paypal/webhook exempt"] }, x: -215, y: 0, state: "ok", variant: "CADDY", name: "Gate", reading: "0.4% denied", sub: "caddy basic auth, then app sessions" },
    { id: "api", mono: "JS", size: 1.08, kind: "assembler", params: { layers: ["adapters · DatabaseSync, spawnSync, fetch", "core · packages/core, packages/house", "routes · apps/api/src/server.ts"] }, x: -90, y: 0, state: "ok", variant: "NODE", name: "acquit-api", reading: "p99 184 ms", sub: "apps/api · :4310" },
    { id: "clock", mono: "30", size: 0.9, kind: "clock", x: -140, y: 88, state: "ok", variant: "30 S", name: "tick + house loop", reading: "every 30 s", sub: "two setInterval timers inside the API" },
    { id: "verifier", mono: "VF", size: 1, kind: "bank", x: 45, y: -96, state: "strain", variant: "DOCKER", params: { cells: 1, busy: 1, queued: 7 }, name: "verifier", reading: "1 busy · 7 queued", sub: "packages/verifier · :4311 · concurrency 1" },
    { id: "sqlite", mono: "SQ", size: 0.9, kind: "tank", params: { layers: ["tables", "effects queue rows", "schema"], files: ["model-calls.jsonl", "house state + pause"] }, x: 12, y: 100, state: "ok", variant: "SQLITE", name: "sqlite", reading: "q p99 3 ms", sub: "data/acquit.db · api + ctl writers" },
    { id: "kiln", mono: "DX", size: 1, kind: "kiln", params: { layers: ["docker · allowlist proxy only", "droid exec · timed from host only"] }, x: 140, y: 40, state: "ok", variant: "DROID EXEC", name: "model calls", reading: "1.9k tok/s", sub: "droid exec in docker · sealed" },
    { id: "paypal", mono: "PP", size: 1, kind: "dock", x: 262, y: -40, state: "fail", variant: "PAYPAL", name: "paypal", reading: "4.1% errors", sub: "sandbox api · retries x3 · token renew" },
    { id: "github", mono: "GH", size: 0.85, kind: "dock", x: 262, y: -110, state: "ok", variant: "GITHUB", name: "github app", reading: "0.3 rps", sub: "rest api + git subprocess" },
    { id: "models", mono: "PX", size: 0.9, kind: "dock", x: 262, y: 40, state: "ok", variant: "PROXY", name: "model hosts", reading: "$1.42/h", sub: "via allowlist proxy" },
    { id: "radar", mono: "OT", size: 1, kind: "radar", x: -92, y: -112, state: "ghost", name: "otel radar", reading: "under construction", sub: "ghost · D13 · Droid building" },
    { id: "scaffold", of: "radar", kind: "scaffold", x: -92, y: -112, state: "ok", params: { w: 52, d: 48, h: 52 }, name: "scaffolding", reading: "branch radar/otel", sub: "open branch for the ghost" },
    { id: "crane", mono: "CI", size: 1, kind: "crane", x: -92, y: -112, state: "run", params: { run: "running", at: 0.3, lift: 0.55, job: "CI #418" }, name: "CI #418", reading: "running · 3 of 5", sub: "branch radar/otel · local harness" },
    { id: "bot", mono: "D", size: 1, kind: "bot", x: -52, y: -124, state: "ok", params: { target: [-84, -112, 30] }, name: "Droid", reading: "verifying · 38m", sub: "agent session" },
    { id: "bot-cx", mono: "CX", size: 1, kind: "bot", x: 178, y: -70, state: "wait", params: { target: [150, -20, 8] }, name: "Codex", reading: "waiting on you · 9m", sub: "passive session" },
    { id: "you", mono: "ME", size: 1, kind: "inspection", x: -222, y: 108, state: "wait", params: { waiting: 7 }, name: "you", reading: "7 waiting", sub: "5 choices · 1 ghost to approve · 1 agent blocked" },
    { id: "retry", mono: "R3", size: 1, kind: "retry", x: 150, y: -20, state: "strain", name: "retry x3", reading: "loop 2/3", sub: "paypal payout retries" },
    { id: "breaker", mono: "BR", size: 1, kind: "breaker", x: 92, y: -20, state: "ghost", name: "paypal breaker", reading: "proposed", sub: "proposed by Droid · waiting on you" },
    { id: "valve", mono: "C1", size: 1, kind: "valve", x: -66, y: -58, state: "strain", name: "concurrency 1", reading: "cap 1", sub: "verifier admits one job" },
  ],
  territories: [
    { id: "radar-otel", entities: ["radar", "api"], state: "verifying", label: "RADAR-OTEL · VERIFYING" },
    { id: "gate-sessions", entities: ["gate"], state: "passed", stale: true, label: "GATE-SESSIONS · STALE" },
    { id: "paypal-payouts", entities: ["retry", "paypal", "breaker"], state: "unclaimed", fog: true, label: "PAYPAL-PAYOUTS · UNCLAIMED" },
  ],
  belts: [
    { from: "intake", to: "gate", pts: [[-280, 0], [-237, 0]] },
    { from: "gate", to: "api", pts: [[-193, 0], [-138, 0]] },
    { from: "api", to: "verifier", pts: [[-66, -34], [-66, -76], [21, -76]] },
    { from: "api", to: "sqlite", pts: [[-74, 34], [-74, 100], [-12, 100]] },
    { from: "clock", to: "api", pts: [[-128, 66], [-128, 43]] },
    { from: "api", to: "kiln", pts: [[-52, 24], [105, 24]] },
    { from: "api", to: "paypal", pts: [[-52, -20], [192, -20], [192, -40], [240, -40]], tone: "red" },
    { from: "verifier", to: "github", pts: [[69, -110], [240, -110]] },
    { from: "kiln", to: "models", pts: [[175, 40], [240, 40]] },
    { from: "api", to: "radar", ghost: true, pts: [[-104, -47], [-104, -94]] },
  ],
};

export const INSPECT = {
  api: {
    budgets: [
      { metric: "p99 latency", unit: "ms", value: 184, budget: 250, max: 320, by: "architect" },
      { metric: "error rate", unit: "%", value: 0.6, budget: 1, max: 1.5, by: "architect" },
    ],
    sources: [
      { src: "designed", ok: true, what: "system.yaml · acquit-api", detail: "role api · lane 0" },
      { src: "declared", ok: true, what: "acquit-api.service · apps/api", detail: "port 4310 · EnvironmentFile" },
      { src: "observed", ok: true, what: "otel service.name acquit-api", detail: "15 rps · via radar (ghost)" },
    ],
    match: { score: 0.94, rules: ["unit name", "package name", "service.name", "port 4310 (weak)"] },
    drift: [
      { sev: "fog", text: "Config also comes from code: /opt/acquit/.env", ref: "apps/api/src/config.ts:14-15" },
      { sev: "drift", text: "Verifier answers by callback, so one job is two requests. Drawn as linked spans.", ref: "apps/api/src/server.ts:362-377" },
    ],
    fog: { observed: 1, tested: 0.5, owned: 1, designed: 1, understood: 1, matched: 1 },
    calibration: { score: 0.82, series: [0.71, 0.74, 0.7, 0.78, 0.8, 0.79, 0.83, 0.82] },
    choices: [
      { text: "Queue rows carry traceparent, with span links", conf: 0.7, cost: "low", risk: 0.12 },
      { text: "Wrap DatabaseSync by hand (no diagnostics channel)", conf: 0.85, cost: "low", risk: 0.06 },
    ],
  },
  verifier: {
    budgets: [
      { metric: "queue wait p95", unit: "s", value: 41, budget: 30, max: 60, by: "architect" },
      { metric: "concurrency", unit: "", value: 1, budget: 1, max: 4, by: "config" },
    ],
    sources: [
      { src: "designed", ok: true, what: "system.yaml · verifier", detail: "role worker pool" },
      { src: "declared", ok: true, what: "acquit-verifier.service · packages/verifier", detail: "port 4311 · /healthz" },
      { src: "observed", ok: false, what: "no spans yet", detail: "docker subjects run --network none" },
    ],
    match: { score: 0.88, rules: ["unit name", "package name", "port 4311 (weak)"] },
    drift: [{ sev: "drift", text: "Queue wait is over budget: 41 s against 30 s", ref: "budget set 3 days ago" }, { sev: "fog", text: "Docker subjects are opaque. Timed from the host only.", ref: "packages/verifier/subject.ts:163" }],
    fog: { observed: 0.5, tested: 1, owned: 1, designed: 1, understood: 1, matched: 1 },
    calibration: { score: 0.64, series: [0.5, 0.55, 0.61, 0.58, 0.6, 0.66, 0.63, 0.64] },
    choices: [{ text: "Raise verifier concurrency to 2", conf: 0.4, cost: "medium", risk: 0.36 }],
  },
  paypal: {
    budgets: [{ metric: "error rate", unit: "%", value: 4.1, budget: 2, max: 6, by: "architect" }],
    sources: [
      { src: "designed", ok: true, what: "system.yaml · dock paypal", detail: "external · sandbox" },
      { src: "declared", ok: true, what: "PAYPAL_* in .env", detail: "loaded in code" },
      { src: "observed", ok: true, what: "fetch → api-m.sandbox.paypal.com", detail: "4.1% 5xx · retries x3" },
    ],
    match: { score: 0.97, rules: ["outbound host", "env key prefix"] },
    drift: [{ sev: "fail", text: "Error rate breaches budget: 4.1% against 2%", ref: "woke the canvas 14:02" }],
    fog: { observed: 1, tested: 0.5, owned: 0, designed: 1, understood: 0.5, matched: 1 },
    calibration: { score: 0.77, series: [0.8, 0.79, 0.77, 0.75, 0.78, 0.76, 0.77, 0.77] },
    choices: [{ text: "Back off retries to x2 with jitter", conf: 0.55, cost: "low", risk: 0.18 }],
  },
};

export const STAGES = [
  { id: "ghost", label: "Ghost", note: "drawn on the blueprint" },
  { id: "simulated", label: "Simulated", note: "replayed on recorded traffic" },
  { id: "building", label: "Under construction", note: "an agent holds the territory" },
  { id: "built", label: "Built", note: "merged, inked in, first light" },
];

export const CATALOG_DEFAULTS = {
  intake: [["sessions", "from traces"]], splitter: [["p99", "1 ms"]], gate: [["p99", "3 ms"], ["deny rate", "1%"]],
  assembler: [["p99", "50 ms"], ["concurrency", "8"]], bank: [["slots", "4"], ["job p95", "30 s"]],
  hopper: [["enqueue p99", "2 ms"], ["depth cap", "1,000"]], tank: [["query p99", "5 ms"], ["writers", "1"]],
  buffer: [["hit p99", "1 ms"], ["hit rate", "0.80"]], kiln: [["p99", "8 s"], ["tokens/s", "2k"]],
  dock: [["p99", "400 ms"], ["errors", "1%"]], clock: [["period", "60 s"]], radar: [["overhead", "1% cpu"], ["export", "OTLP/HTTP"]],
  retry: [["tries", "3"], ["backoff", "exp 200 ms"]], valve: [["cap", "10 rps"]], breaker: [["trip after", "5 fails in 30 s"], ["half-open", "after 60 s"]],
};

export const GHOSTS = {
  radar: {
    stage: "building", by: "you", agent: "Droid", since: "38m", branch: "radar/otel",
    why: "Acquit ships no OpenTelemetry, so its observed source is empty (L3). First real ghost task (D13).",
    checks: [
      { ok: true, text: "OTLP/HTTP JSON export over node:http (E4)" },
      { ok: true, text: "HTTP server and client spans from diagnostics_channel" },
      { ok: true, text: "DatabaseSync wrapped by hand, no channels (L16)" },
      { ok: false, text: "Trace context across the queue and verifier callback (L13)" },
      { ok: false, text: "Budget: export overhead under 2% cpu", locked: true },
    ],
    declared: "branch radar/otel · packages/telemetry", afterReading: "312 spans/s",
  },
  breaker: {
    stage: "proposed", by: "Droid", on: "acquit-api → paypal", conf: 0.6, cost: "low",
    why: "Radar spans show retry storms. Three retries on a 4.1% error rate triple PayPal calls during an outage.",
    evidence: "212 retried calls in the last hour · 38 hit the third try",
  },
};

export const TSTATES = ["unclaimed", "claimed", "building", "verifying", "passed"];

export const TERRITORIES = {
  "radar-otel": {
    goal: "Every Acquit machine has an observed source", file: ".acan/territories/radar-otel.yaml", owner: "Droid",
    entities: ["radar", "api", "sqlite", "verifier"], depends: [], state: "verifying",
    budgets: ["radar.cpu_overhead < 2%", "api.p99_ms < 250"],
  },
  "gate-sessions": {
    goal: "Basic auth in front, app sessions behind, webhook exempt", file: ".acan/territories/gate-sessions.yaml", owner: "Droid",
    entities: ["gate"], depends: [], state: "passed", stale: "apps/api/src/server.ts changed 16 min ago, after the seal",
    budgets: ["gate.p99_ms < 5"],
  },
  "paypal-payouts": {
    goal: "Payouts survive a PayPal outage without retry storms", file: ".acan/territories/paypal-payouts.yaml", owner: null,
    entities: ["paypal", "retry", "breaker"], depends: ["radar-otel"], state: "unclaimed", fog: "Fog 0.42. Nobody owns paypal, and it is half understood. Scout first.",
    budgets: ["paypal.error_rate < 2%"],
  },
};

export const SESSIONS = [
  {
    id: "droid-radar", agent: "Droid", bot: "bot", model: "claude-opus (from genai spans)", goal: "Install radar on Acquit (D13)", state: "verifying",
    now: "run_checks radar-otel · 3 of 5 passed", since: "38m", perm: "can build", territory: "radar-otel",
    via: [["MCP", true, "register_session, claim_territory, record_choice"], ["Native OTel", true, "genai spans per turn, session.id"], ["Hooks", true, "acan-hook relay"], ["Transcript", false, "backfill only"]],
    files: [["packages/core/src/telemetry.ts", "radar", 14], ["packages/core/src/effects.ts", "api", 6], ["apps/api/src/server.ts", "api", 4], ["packages/verifier/service.ts", "verifier", 3], ["packages/core/src/db.ts", "sqlite", 2]],
    spend: [["turns", "64"], ["tool calls", "211"], ["tokens", "412k"], ["cost", "$1.92"]],
    choices: [{ text: "Queue rows carry traceparent, with span links", conf: 0.7, on: "api" }, { text: "Wrap DatabaseSync by hand", conf: 0.85, on: "sqlite" }],
    lane: [[-38, -30, "scout", "scouting · read effects.ts, server.ts"], [-30, -4, "build", "building radar-otel"], [-4, 0, "verify", "run_checks"]],
    marks: [[-30, "claim", "claim_territory radar-otel"], [-22, "choice-low", "choice · traceparent in queue rows · 0.70"], [-18, "fail", "checks 1 of 5"], [-16, "stale", "edit made gate-sessions stale"], [-14, "choice", "choice · wrap DatabaseSync · 0.85"], [-9, "pass", "checks 3 of 5"]],
  },
  {
    id: "codex-paypal", agent: "Codex", bot: "bot-cx", model: "gpt (from OTel)", goal: "Back off PayPal retries with jitter", state: "blocked",
    now: "Waiting for approval in its terminal: run npm test in packages/core", since: "26m", perm: "passive", territory: null,
    via: [["MCP", false, "not connected, so no territory or choices"], ["Native OTel", true, "[otel] table, HTTP"], ["Hooks", true, "trusted by hash"], ["Transcript", false, "not read"]],
    files: [["packages/core/src/paypal.ts", "paypal", 9], ["packages/core/src/retry.ts", "retry", 3]],
    spend: [["turns", "22"], ["tool calls", "58"], ["tokens", "96k"], ["cost", "$0.41"]],
    choices: [],
    lane: [[-26, -9, "build", "editing paypal.ts, retry.ts"], [-9, 0, "wait", "blocked on you · approval prompt"]],
    marks: [[-9, "wait", "asked to run npm test"]],
  },
  {
    id: "droid-gate", agent: "Droid", bot: null, model: "claude-opus", goal: "Harden gate sessions", state: "done",
    now: "Finished 52 min ago. Seal stamped, now stale.", since: "1h 20m", perm: "can build", territory: "gate-sessions",
    via: [["MCP", true, "full loop"], ["Native OTel", true, "genai spans"], ["Hooks", true, ""], ["Transcript", false, ""]],
    files: [["apps/api/src/session.ts", "gate", 11], ["deploy/Caddyfile", "gate", 2]],
    spend: [["turns", "41"], ["tool calls", "133"], ["tokens", "288k"], ["cost", "$1.17"]],
    choices: [{ text: "Webhook path exempt from basic auth", conf: 0.6, on: "gate" }],
    lane: [[-60, -55, "verify", "run_checks gate-sessions"], [-55, -52, "build", "final fixes"]],
    marks: [[-52, "seal", "passed · brass seal"], [-16, "stale", "stale · server.ts changed"]],
  },
];

export const CI = {
  crane: "crane", session: "droid-radar", runner: "local harness · .acan/checks/",
  runs: [
    { n: 414, t0: -58, t1: -53, status: "passed", what: "gate-sessions · 4 of 4" },
    { n: 416, t0: -21, t1: -18, status: "failed", what: "radar-otel · 1 of 5" },
    { n: 417, t0: -12, t1: -9, status: "failed", what: "radar-otel · 3 of 5" },
    { n: 418, t0: -4, t1: 0, status: "running", what: "radar-otel · 3 of 5 so far" },
  ],
};

export const COST = { low: 0.4, medium: 0.6, high: 1 };

export const CHOICES = [
  {
    id: "gate-webhook-exempt", on: "gate", by: "Droid", session: "droid-gate", territory: "gate-sessions", when: "yesterday",
    question: "Should /paypal/webhook skip basic auth?", text: "Exempt the webhook path from basic auth. PayPal can't send the password.",
    conf: 0.6, cost: "high", evidence: ["deploy/Caddyfile:12", "apps/api/src/session.ts:44"],
    alts: [{ text: "Verify the PayPal signature before any auth", note: "keeps auth on, needs the webhook id in env" }, { text: "Move the webhook to its own listener", note: "a second port behind the gate" }],
  },
  {
    id: "verifier-concurrency-2", on: "verifier", by: "Droid", session: "droid-gate", territory: null, when: "yesterday",
    question: "How do we bring verifier queue wait under 30 s?", text: "Raise verifier concurrency from 1 to 2.",
    conf: 0.4, cost: "medium", evidence: ["packages/verifier/service.ts:58", "budget queue wait p95 < 30 s"],
    alts: [{ text: "Keep 1 and shorten docker cold start", note: "no new race on the subject directory" }, { text: "Add a second lane (+10n ports)", note: "costs a full stack copy" }],
  },
  {
    id: "paypal-retry-x2", on: "paypal", by: "Claude Code", session: null, territory: "paypal-payouts", when: "yesterday",
    question: "How many times should a failed payout retry?", text: "Back off retries to x2 with jitter.",
    conf: 0.55, cost: "low", evidence: ["packages/core/src/retry.ts:21"],
    alts: [{ text: "Keep x3, add the breaker ghost", note: "the breaker stops storms instead" }],
  },
  {
    id: "traceparent-queue-rows", on: "api", by: "Droid", session: "droid-radar", territory: "radar-otel", when: "22 min ago",
    question: "How does trace context survive the effects queue?", text: "Queue rows carry traceparent, and spans link across the gap.",
    conf: 0.7, cost: "low", evidence: ["packages/core/src/effects.ts:246", "L13"],
    alts: [{ text: "Start a new trace per tick", note: "simpler, loses the request to job link" }],
  },
  {
    id: "wrap-databasesync", on: "sqlite", by: "Droid", session: "droid-radar", territory: "radar-otel", when: "14 min ago",
    question: "How do we time SQLite calls without a diagnostics channel?", text: "Wrap DatabaseSync by hand.",
    conf: 0.85, cost: "low", evidence: ["packages/core/src/db.ts:9", "L16"],
    alts: [{ text: "Sample with the event-loop delay only", note: "no per-query spans" }],
  },
];

export const SIM = {
  window: "yesterday 14:00 to 15:00", start: 14 * 3600, secs: 3600, dt: 5, seed: 418, baseRps: 15,
  ramp: [0.92, 0.16], burst: { t0: 1140, t1: 1420, add: 0.9 },
  model: {
    gate: { per: 1, S: 0.0008, c: 64, metric: "p99", unit: "ms", budget: 5, cal: 0.91, inputs: [["service time", "0.8 ms", "measured · 4.1k spans"], ["workers", "64", "Caddyfile"]] },
    api: { per: 1, S: 0.045, c: 8, metric: "p99", unit: "ms", budget: 250, err: 0.6, errBudget: 1, cal: 0.82, inputs: [["service time", "45 ms", "measured · 52k spans"], ["in-flight slots", "8", "shared by requests and effects"], ["payout effects", "4% of requests", "measured"]] },
    verifier: { per: 0.0015, S: 19, c: 1, metric: "queue wait p95", unit: "s", budget: 30, cal: 0.64, inputs: [["service time", "19 s", "measured · 212 jobs"], ["concurrency", "1", "config"], ["jobs", "0.15% of requests", "measured"]], unmodeled: "Docker cold start varies from 4 to 30 s. The model uses the mean, so the band is wide." },
    sqlite: { per: 3.2, S: 0.0009, c: 1, metric: "query p99", unit: "ms", cal: 0.9, inputs: [["service time", "0.9 ms", "measured · 180k spans"], ["writers", "1", "SQLite has one writer"]] },
    kiln: { per: 0.025, S: 5, c: 4, metric: "p99", unit: "s", cal: 0.7, inputs: [["service time", "5 s", "timed from the host"], ["containers", "4", "config"], ["calls", "2.5% of requests", "measured"]], unmodeled: "Token counts vary by prompt. Timed from the host only." },
    paypal: { per: 0.04, S: 0.4, metric: "error rate", unit: "%", budget: 2, err: 4.1, retries: 3, timeout: 10, cal: 0.77, inputs: [["call time", "400 ms", "measured · 2.1k calls"], ["errors", "4.1%", "observed"], ["retries", "x3, timeout 10 s", "packages/core/src/retry.ts"]] },
  },
  ghosts: {
    radar: { on: "api", conf: 0.35, effect: "adds 1% to acquit-api service time", from: "catalog default · overhead 1% cpu" },
    breaker: { on: "api", conf: 0.35, effect: "trips after 5 fails in 30 s, then fails fast", from: "catalog default · half-open after 60 s" },
  },
};

export const REPLAY = {
  window: "today 15:30 to 16:30", now: 16.5 * 3600, mins: 60, dt: 10, seed: 1543,
  rps: [[-60, 13.2], [-30, 14.4], [0, 15.4]],
  series: {
    intake: { keys: [[-60, 31], [0, 38]], fmt: "{i} sessions" },
    gate: { keys: [[-60, 0.4], [0, 0.4]], noise: 0.08, fmt: "{1}% denied" },
    api: { keys: [[-60, 178], [-47.1, 182], [-46.5, 640], [-45.2, 2600], [-41, 2300], [-38.5, 900], [-36.4, 240], [-34, 196], [0, 184]], noise: 0.04, unit: "ms", metric: "p99", budget: 250, strain: 250, fail: 1000, fmt: "p99 {ms}",
      err: [[-60, 0.5], [-46.6, 0.6], [-45, 6.2], [-38, 5], [-36.4, 1.1], [-34, 0.6], [0, 0.6]], errBudget: 1, errFail: 5 },
    verifier: { keys: [[-60, 1], [-46, 2], [-44, 6], [-40, 9], [-36, 8], [-20, 6], [0, 7]], step: true, metric: "queued", strain: 3, fail: 12, fmt: "1 busy · {i} queued" },
    sqlite: { keys: [[-60, 3], [0, 3]], noise: 0.1, unit: "ms", metric: "query p99", fmt: "q p99 {i} ms" },
    kiln: { keys: [[-60, 1.8], [-46, 1.8], [-45, 1.2], [-37, 1.2], [-36, 1.9], [0, 1.9]], noise: 0.04, metric: "tokens/s", fmt: "{1}k tok/s" },
    paypal: { keys: [[-60, 0.3], [-47.4, 0.3], [-47.2, 6], [-45.5, 31], [-40, 28], [-37, 9], [-35, 4.6], [-33, 4.1], [0, 4.1]], noise: 0.015, unit: "%", metric: "error rate", budget: 2, strain: 2, fail: 4, fmt: "{1}% errors" },
    retry: { keys: [[-60, 1], [-47, 1], [-46.9, 3], [-36, 3], [-35.9, 2], [0, 2]], step: true, metric: "loop depth", strain: 2, fail: 3, fmt: "loop {i}/3" },
    github: { keys: [[-60, 0.3], [0, 0.3]], noise: 0.1, fmt: "{1} rps" },
    models: { keys: [[-60, 1.38], [-46, 1.38], [-45, 0.9], [-37, 0.9], [-36, 1.42], [0, 1.42]], noise: 0.03, fmt: "${2}/h" },
  },
  changes: [
    { t: -50.5, kind: "deploy", text: "Deploy a41c9e", detail: "apps/api/src/session.ts, deploy/Caddyfile", touches: ["gate", "api"] },
  ],
  extra: [{ t: -20, kind: "ghost", on: "breaker", text: "Droid proposes the paypal breaker" }],
  appear: { radar: -30, breaker: -20 },
  outside: { paypal: "api-m.sandbox.paypal.com" },
  because: {
    retry: "Every failed payout loops to its third try.",
    api: "Three tries of 10 s on each payout hold its 8 in-flight slots.",
    verifier: "Verifier callbacks wait for acquit-api slots.",
  },
  files: ["packages/core/src/paypal.ts", "packages/core/src/retry.ts"],
};

export const FIRST = {
  folder: "~/src/acquit", rev: "3f2a91c",
  readers: [
    { src: "declared", label: "TypeScript packages", detail: "apps/api, packages/core, packages/verifier, packages/house, packages/ctl", files: 212, finds: ["api", "verifier", "sqlite", "valve"] },
    { src: "declared", label: "systemd units", detail: "acquit-api.service, acquit-verifier.service, caddy.service", files: 3, finds: ["gate"] },
    { src: "declared", label: "Caddyfile", detail: "deploy/Caddyfile · basic auth, /paypal/webhook", files: 1, finds: ["intake"] },
    { src: "declared", label: ".env loaded in code", detail: "apps/api/src/config.ts:14-15 · fogged, not read", files: 2, fog: true, finds: [] },
    { src: "declared", label: "docker-compose, Kubernetes", detail: "none in this repo", files: 0, finds: [] },
    { src: "observed", label: "OTLP on 127.0.0.1:4318", detail: "listening · no spans yet", files: 0, waiting: true, finds: [] },
    { src: "designed", label: ".acan/", detail: "none yet · Acan drafts one for you", files: 0, finds: [] },
  ],
  match: {
    intake: "site block in deploy/Caddyfile", gate: "unit caddy.service", api: "unit acquit-api.service, port 4310", verifier: "unit acquit-verifier.service, port 4311",
    sqlite: "file data/acquit.db via DatabaseSync", valve: "concurrency: 1 in packages/verifier/src/queue.ts", kiln: "docker run in packages/house/src/exec.ts",
    paypal: "host api-m.sandbox.paypal.com", retry: "retry(3) in packages/core/src/retry.ts", clock: "setInterval in apps/api/src/server.ts",
    github: "host api.github.com", models: "allowlist in packages/house/proxy.ts",
  },
  auto: { gate: 0.97, api: 0.94, verifier: 0.92, sqlite: 0.9, intake: 0.96, valve: 0.92 },
  tray: [
    { id: "kiln", conf: 0.62, rule: "docker run in packages/house/src/exec.ts", ask: "Its own machine, or work inside acquit-api?", yes: "Place as its own machine", no: "Part of acquit-api", fx: ["machine", "part"] },
    { id: "paypal", conf: 0.71, rule: "fetch to api-m.sandbox.paypal.com in packages/core/src/paypal.ts", ask: "An external service?", yes: "Place as an external dock", no: "Ignore this host", with: ["retry"], fx: ["dock", "ignore"] },
    { id: "clock", conf: 0.58, rule: "two setInterval timers in apps/api/src/server.ts", ask: "Draw the timers as their own machine?", yes: "Place as its own machine", no: "Part of acquit-api", fx: ["machine", "part"] },
    { id: "github", conf: 0.68, rule: "GitHub REST calls and a git subprocess in packages/verifier", ask: "An external service?", yes: "Place as an external dock", no: "Ignore this host", fx: ["dock", "ignore"] },
    { id: "models", conf: 0.55, rule: "allowlist proxy host in packages/house/proxy.ts", ask: "An external service?", yes: "Place as an external dock", no: "Ignore this host", fx: ["dock", "ignore"] },
    { id: "lane1", name: "acquit-api on :4320", mono: "JS", conf: 0.48, rule: "port 4320 = 4310 + 10 · L18 lanes repeat every service", ask: "A second lane of acquit-api?", yes: "Merge as lane 1 of acquit-api", no: "Keep as its own machine", merge: "api", fx: ["lane", "new"] },
    { id: "ctl", name: "packages/ctl", mono: "CT", conf: 0.31, rule: "CLI package · no port, no unit, no listener", ask: "A CLI, not a running machine. Skip it?", yes: "Skip it", no: "Keep as a machine", fx: ["ignore", "new"] },
  ],
  agents: [
    { id: "droid", name: "Droid", found: ".factory/ in the repo · droid on PATH", writes: [
      { to: "~/.zshenv", lines: ['export OTEL_TELEMETRY_ENDPOINT="http://127.0.0.1:4318"', 'export OTEL_TELEMETRY_FORMAT="genai"'] },
      { to: "~/.factory/mcp.json", via: "droid mcp add acan http://127.0.0.1:4319/mcp --type http", lines: ['"acan": { "type": "http", "url": "http://127.0.0.1:4319/mcp" }'] },
    ], note: "Droid reads these at launch, so restart open sessions. What Droid sends to Factory does not change. If your org pins a telemetry endpoint, the first two lines do nothing and Acan tells you.", hooks: "acan-hook relay · optional", bot: "bot", first: 0.6, backfill: 12 },
    { id: "claude", name: "Claude Code", found: "~/.claude/settings.json", writes: [
      { to: "~/.claude/settings.json", lines: ['"env": { "CLAUDE_CODE_ENABLE_TELEMETRY": "1", "OTEL_METRICS_EXPORTER": "otlp", "OTEL_LOGS_EXPORTER": "otlp", "OTEL_EXPORTER_OTLP_PROTOCOL": "http/json", "OTEL_EXPORTER_OTLP_ENDPOINT": "http://127.0.0.1:4318" }'] },
      { to: "~/.claude.json", via: "claude mcp add --transport http --scope user acan http://127.0.0.1:4319/mcp", lines: ['"mcpServers": { "acan": { "type": "http", "url": "http://127.0.0.1:4319/mcp" } }'] },
    ], note: "Claude Code sends metrics and events, not spans, so Acan draws its turns from events. Prompts stay out unless you set OTEL_LOG_USER_PROMPTS.", hooks: "native http hooks · optional", bot: null, first: null, backfill: 4 },
    { id: "codex", name: "Codex", found: "~/.codex/config.toml", writes: [
      { to: "~/.codex/config.toml", lines: ["[otel]", 'exporter = { otlp-http = { endpoint = "http://127.0.0.1:4318/v1/logs", protocol = "json" } }', 'trace_exporter = { otlp-http = { endpoint = "http://127.0.0.1:4318/v1/traces", protocol = "json" } }', "[mcp_servers.acan]", 'url = "http://127.0.0.1:4319/mcp"'] },
    ], note: "codex exec sends no metrics (openai/codex issue 12913), so CI runs show from traces and transcripts.", hooks: "acan-hook relay · optional", bot: "bot-cx", first: 1.1, backfill: 3 },
  ],
};

const S = (observed, tested, owned, designed, understood, matched) => ({ observed, tested, owned, designed, understood, matched });
export const FOG = {
  intake: { s: S(1, 0.5, 1, 1, 1, 1), why: "webhook path has no test" },
  gate: { s: S(1, 0, 1, 1, 0.5, 1), why: "grew back · 6 commits since its seal", churn: 6 },
  api: { s: S(1, 0.5, 1, 1, 1, 1), why: "config also comes from code, /opt/acquit/.env" },
  clock: { s: S(0, 0.5, 1, 0.5, 1, 0.5), why: "timers emit no spans" },
  verifier: { s: S(0.5, 1, 1, 1, 1, 1), why: "docker subjects are opaque" },
  sqlite: { s: S(0, 1, 1, 1, 1, 1), why: "DatabaseSync has no channels yet" },
  kiln: { s: S(0.5, 0, 1, 0.5, 0.5, 0.5), why: "sealed docker, timed from the host only" },
  paypal: { s: S(1, 0.5, 0, 1, 0.5, 1), why: "nobody owns it, half understood" },
  retry: { s: S(1, 0, 0, 0.5, 0.5, 1), why: "no test covers the third try" },
  github: { s: S(0.5, 0, 0, 0.5, 1, 1), why: "git subprocess is not traced" },
  models: { s: S(1, 0, 0.5, 0.5, 0.5, 0.5), why: "the proxy hides which host answers" },
  valve: { s: S(1, 1, 1, 1, 1, 1), why: "" },
};
for (const [id, f] of Object.entries(FOG)) if (INSPECT[id]) INSPECT[id].fog = f.s;
