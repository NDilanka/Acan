export const WORLD = {
  revision: "55825fe",
  origin: "https://git.orrery.local",
  plants: [
    { id: "orrery", name: "Orrery", x: 720, y: 395, env: "local", revision: "71cfe09", branch: "main", worktree: "~/orrery", decisions: 1, condition: "Unmatched claim", freshness: "runtime not reported", binding: "receiver", machines: ["receiver", "telemetry ring", "matcher", "evaluator", "SQLite", "webview renderer"],
      scene: {
        entities: [
          { id: "receiver", kind: "intake", x: -160, y: -10 },
          { id: "telemetry ring", kind: "buffer", x: -40, y: -60, variant: "RING" },
          { id: "matcher", kind: "assembler", size: 0.75, x: -40, y: 40 },
          { id: "evaluator", kind: "assembler", size: 0.75, x: 80, y: 40 },
          { id: "SQLite", kind: "tank", size: 0.8, x: 80, y: -60, variant: "SQLITE" },
          { id: "webview renderer", kind: "assembler", size: 0.6, x: 190, y: -10, variant: "WEBVIEW" }
        ],
        territories: [
          { id: "housing", entities: ["receiver", "telemetry ring", "matcher", "evaluator", "SQLite", "webview renderer"], state: "housing", label: "ONE RUST PROCESS" },
          { id: "g1", entities: ["matcher"], state: "unclaimed", label: "G1 · CODEX · NO READBACK" }
        ],
        belts: [
          { from: "receiver", to: "telemetry ring", pts: [[-135, -60], [-76, -60]] },
          { from: "telemetry ring", to: "matcher", pts: [[-40, -42], [-40, 8]] },
          { from: "matcher", to: "evaluator", pts: [[-4, 40], [44, 40]] },
          { from: "evaluator", to: "SQLite", pts: [[80, 8], [80, -29]] },
          { from: "evaluator", to: "webview renderer", pts: [[116, 40], [190, 40], [190, 16]] }
        ]
      } },
    { id: "acquit", name: "Acquit", x: 345, y: 555, env: "sandbox", revision: "55825fe", branch: "radar/otel", worktree: "~/acquit", decisions: 2, condition: "p-2307 needs an answer", freshness: "OTLP stale · 41 s", binding: "otlp", machines: ["acquit-api", "verifier", "sqlite", "paypal", "retry", "OTLP boundary"], ids: { "acquit-api": "api", "OTLP boundary": "radar" } },
    { id: "kvarm", name: "Kvarm", x: 1150, y: 360, env: "own origin", revision: "8ae91d2", branch: "main", worktree: "~/kvarm", decisions: 1, condition: "Review blocks publication", freshness: "fresh · 8 s", binding: "train", machines: ["repo events", "Worker", "train", "D1", "Artifacts"],
      scene: {
        entities: [
          { id: "repo events", kind: "intake", x: -190, y: 0 },
          { id: "Worker", kind: "assembler", size: 0.85, x: -70, y: 0, variant: "WORKER" },
          { id: "train", kind: "hopper", x: 70, y: 0, params: { depth: 0 } },
          { id: "D1", kind: "tank", x: -70, y: -100, params: { extra: 0 }, variant: "D1" },
          { id: "Artifacts", kind: "buffer", x: -70, y: 92, variant: "ARTIFACTS" }
        ],
        belts: [
          { from: "repo events", to: "Worker", pts: [[-165, 0], [-111, 0]] },
          { from: "Worker", to: "train", pts: [[-29, 0], [34, 0]] },
          { from: "Worker", to: "D1", pts: [[-70, -37], [-70, -69]] },
          { from: "Worker", to: "Artifacts", pts: [[-70, 37], [-70, 74]] }
        ]
      } },
    { id: "format", name: "orrery-format", x: 375, y: 225, env: "local", revision: "e7cb210", branch: "main", worktree: "~/orrery-format", decisions: 1, condition: "Check digest needs review", freshness: "lifecycle · 12 s", binding: "validator", machines: ["schema", "reference validator", "checks"],
      scene: {
        entities: [
          { id: "schema", kind: "crate", x: 0, y: -80, variant: "{ }" },
          { id: "reference validator", kind: "assembler", size: 0.7, x: 0, y: 0, variant: "VALIDATOR" },
          { id: "checks", kind: "gate", x: 0, y: 86, params: { layers: ["protected-input digest", "schema fixtures", "pinned runner"] }, variant: "CHECKS" }
        ],
        belts: [
          { from: "schema", to: "reference validator", pts: [[0, -64], [0, -30]] },
          { from: "reference validator", to: "checks", pts: [[0, 30], [0, 49]] }
        ]
      } },
    { id: "site", name: "orrery-site", x: 1085, y: 665, env: "preview", revision: "09bf732", branch: "waitlist", worktree: "~/orrery-site", decisions: 0, condition: "Preview only", freshness: "simulated · not live", binding: "preview", machines: ["artifact receiver", "static waitlist", "CDN"],
      scene: {
        entities: [
          { id: "artifact receiver", kind: "intake", size: 0.7, x: -112, y: 0 },
          { id: "static waitlist", kind: "buffer", x: -10, y: 0, variant: "STATIC" },
          { id: "CDN", kind: "splitter", size: 0.8, x: 100, y: 0, variant: "CDN" }
        ],
        belts: [
          { from: "artifact receiver", to: "static waitlist", pts: [[-94, 0], [-46, 0]] },
          { from: "static waitlist", to: "CDN", pts: [[26, 0], [66, 0]] }
        ]
      } }
  ],
  bindings: {
    receiver: { entity: "Orrery / receiver", channel: "runtime.active", value: null, state: "unknown", sourceClass: "observed", locator: "otel://local/receiver/active", revision: "71cfe09", age: null, unit: "requests", meaning: "Runtime not reported", policy: "5 s heartbeat", reason: "No collector attached" },
    otlp: { entity: "Acquit / OTLP boundary", channel: "runtime.rate", value: 12, state: "stale", sourceClass: "observed", locator: "otel://sandbox/acquit/export", revision: "55825fe", age: 41, unit: "spans/s", meaning: "Last rate, not a current rate", policy: "stale after 5 s" },
    train: { entity: "Kvarm / train", channel: "train.queue_depth", value: 0, state: "zero", sourceClass: "authenticated", locator: "kvarm://own-origin/train/queue", revision: "8ae91d2", age: 8, unit: "changes", meaning: "Queue depth is known zero", policy: "stale after two missed polls", remote: true },
    validator: { entity: "orrery-format / validator", channel: "process.lifecycle", value: "stopped", state: "stopped", sourceClass: "declared", locator: "local://validator/exit/0", revision: "e7cb210", age: 12, unit: "lifecycle", meaning: "Process exited, code 0", policy: "lifecycle event" },
    preview: { entity: "orrery-site / preview", channel: "simulation.waitlist", value: 24, state: "simulated", sourceClass: "designed", locator: "scenarios/waitlist.yaml:8", revision: "09bf732", age: 0, unit: "visits", meaning: "SIM · generated visits, not traffic", policy: "scenario, seed 24" },
    verifier: { entity: "Acquit / verifier", channel: "declared.concurrency", value: 1, state: "valid", sourceClass: "declared", locator: "deploy/runtime.env:12", revision: "55825fe", age: 8, unit: "cell", meaning: "one cell", policy: "revision bound", runtime: "runtime concurrency not reported", text: "one cell · declared concurrency 1 · deploy/runtime.env:12 · fixture revision 55825fe · runtime concurrency not reported" }
  },
  queue: [
    { id: "claim", plant: "orrery", title: "Unmatched claim", why: "Codex holds g1 without a readback", cost: "Territory remains fogged", safe: "Before the next lease", estimate: "1–2 min", kind: "recommended hold", entity: "matcher" },
    { id: "p-2307", plant: "acquit", title: "p-2307 · keep pending?", why: "Two readbacks disagree at sqlite", cost: "Payout retry cannot be witnessed", safe: "Before the next retry", estimate: "2–4 min", kind: "question", entity: "sqlite", departure: "You left p-2307 unanswered. Two readings disagree on whether a failed payout stays pending. Next, compare w3 at sqlite." },
    { id: "c1", plant: "acquit", title: "c1 · retry meets breaker", why: "Two declared writes share one belt", cost: "The proposal can break w3", safe: "Before any code is edited", estimate: "3–5 min", kind: "recommended hold", entity: "retry" },
    { id: "review", plant: "kvarm", title: "Review #184", why: "Required reviewer is not the author", cost: "Publication waits on approval", safe: "Before the train publishes", estimate: "4–7 min", kind: "blocks publication", entity: "train" },
    { id: "digest", plant: "format", title: "Check protected-input digest", why: "Pinned runner needs your digest", cost: "The required check cannot pass", safe: "Before publishing v0.3.2", estimate: "1–3 min", kind: "blocks publication", entity: "checks" }
  ],
  arrival: { id: "artifact", plant: "site", title: "Check waitlist artifact", why: "Preview differs from the pinned build", cost: "Delivery remains a local proposal", safe: "Before the next delivery", estimate: "1–2 min", kind: "recommended hold", entity: "artifact receiver" },
  contracts: [
    { id: "otlp", from: "acquit", to: "orrery", fromEntity: "OTLP boundary", toEntity: "receiver", kind: "runtime", label: "OTLP / HTTP", detail: "sandbox · acquit@55825fe → local · orrery@71cfe09", evidence: "declared · runtime rate unknown", path: "M445 503 L550 503 L550 420 L615 420", labelAt: [533, 482] },
    { id: "rest", from: "orrery", to: "kvarm", fromEntity: "matcher", toEntity: "Worker", kind: "egress", label: "REST · declared egress", detail: "local · orrery@71cfe09 → own origin · kvarm@8ae91d2", evidence: "opt-in connection · scripted fixture", path: "M820 385 L1045 385", labelAt: [943, 365] },
    { id: "package", from: "format", to: "orrery", fromEntity: "schema", toEntity: "evaluator", kind: "package", label: "package · v0.3.2", detail: "orrery-format@e7cb210 → orrery@71cfe09", evidence: "declared dependency · compatibility not reported", path: "M463 260 L550 260 L550 340 L620 340", labelAt: [505, 248] },
    { id: "artifact", from: "orrery", to: "site", fromEntity: "webview renderer", toEntity: "artifact receiver", kind: "artifact", label: "artifact · web bundle", detail: "orrery@71cfe09 → orrery-site@09bf732", evidence: "declared delivery · preview only", path: "M800 415 L900 415 L900 620 L1040 620", labelAt: [975, 540] }
  ],
  operations: [
    { op: "list_repos", bytes: 768, reason: "Read the three connected repository identities" },
    { op: "get_heads", bytes: 512, reason: "Bind plant summaries to repository revisions" },
    { op: "get_leases", bytes: 384, reason: "Check g1 before presenting an unmatched claim" }
  ],
  pending: [{ op: "record_decision", path: "choices/p-2307.yaml", bytes: 248, reason: "Scripted local choice, not published" }],
  bots: [{ principal: "Codex", session: "cx-17", evidence: "authenticated", note: "holds g1 · no Orrery readback" }, { principal: "Claude Code", session: "cc-42", evidence: "declared", note: "readback · Orrery · 3 of 3" }]
};
