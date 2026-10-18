// Fixture for the bench. Times and spend are scripted for the mock, and every number the bench prints comes from here.
import { WITNESSES, QUESTION, ORDER, SHEET, DECLARATIONS, CLASHES } from "./core-data.mjs";

const d1 = DECLARATIONS.find((d) => d.id === "d1");
const d3 = DECLARATIONS.find((d) => d.id === "d3");
const split = CLASHES.find((c) => c.id === "c1").resolutions.find((r) => r.kind === "split");

export const BENCH_WITNESSES = [...WITNESSES, QUESTION.options[0].witness];

export const BENCH = {
  id: "bench-1", clash: "c1", sheet: SHEET.label, rev: SHEET.rev,
  order: { id: ORDER.id, on: ORDER.on, territory: ORDER.territory, sentence: ORDER.placeholder, budget: ORDER.budgets[0], rev: "r2", next: "r3", claimedBy: "Claude Code" },
  question: "Which design keeps payouts honest while PayPal fails?",
  wrongIf: "An attempt is wrong if replay at any pinned moment breaks it.",
  clauses: {
    w1: "a failed payout loops past its second try",
    w2: "p-2291 settles other than exactly once",
    w3: "p-2307 leaves pending before PayPal recovers",
  },
  notProof: "Holding all three means not disproved. It is not proof.",
  environment: "Sealed replay, today 15:30 to 16:30. No call reaches PayPal.",
  budget: { tokens: 300000, minutes: 20, review: 6, extend: { tokens: 100000, minutes: 5 } },
  cannotRun: { agent: "Codex", why: "Passive session, no MCP." },
  kvarm: { origin: "git.orrery.local", arena: "ar-12", change: 191, choice: "choices/bench-1.yaml" },
  zone: { x0: 56, x1: 282, y0: 122, y1: 204 },
};

export const ATTEMPTS = [
  { id: "a1", key: "1", tag: "A1", label: "Retry with jitter", short: "Retry", approach: "retry", agent: "Claude Code", session: "claude-payouts", workspace: "ws-a1", says: d1.asserts, diff: "2 files · +38 −12", x: 92, y: 162 },
  { id: "a2", key: "2", tag: "A2", label: "Circuit breaker", short: "Breaker", approach: "breaker", agent: "Droid", session: "droid-radar", workspace: "ws-a2", says: d3.asserts, diff: "1 file · +54 −3", x: 168, y: 162 },
  { id: "a3", key: "3", tag: "A3", label: "Split at the port", short: "Split", approach: "split", agent: "Claude Code", session: "claude-port", workspace: "ws-a3", says: split.detail, diff: "4 files · +96 −20", x: 244, y: 162 },
];

// What each witness recorded at its moment, and what replay saw for each attempt at that moment.
export const RECORDED = {
  w1: "loop depth 3 of 3 · paypal at 31% errors",
  w2: "p-2291 settled once, on its first try",
  w3: "PayPal accepted p-2307 and the response was lost",
};

// Replay results arrive one at a time after funding. `at` is milliseconds after Fund. `minute` and `tokens` are the attempt's running totals.
export const EVENTS = [
  { at: 700, attempt: "a2", witness: "w1", result: "holds", seen: "loop depth 1 · the open breaker failed fast", minute: 2, tokens: 24000 },
  { at: 1200, attempt: "a1", witness: "w1", result: "holds", seen: "loop depth 2 of 2", minute: 3, tokens: 52000 },
  { at: 1700, attempt: "a2", witness: "w2", result: "holds", seen: "p-2291 settled once, first try", minute: 3, tokens: 41000 },
  { at: 2200, attempt: "a2", witness: "w3", result: "breaks", seen: "p-2307 marked failed at 15:45, not pending", minute: 4, tokens: 61000, stop: "disproved" },
  { at: 2800, attempt: "a3", witness: "w1", result: "holds", seen: "loop depth 2 of 2 · unknown kept pending", minute: 6, tokens: 140000 },
  { at: 3300, attempt: "a1", witness: "w2", result: "holds", seen: "p-2291 settled once, first try", minute: 7, tokens: 118000 },
  { at: 3800, attempt: "a3", witness: "w2", result: "holds", seen: "p-2291 settled once, first try", minute: 12, tokens: 236000 },
  { at: 4300, attempt: "a1", witness: "w3", result: "holds", seen: "p-2307 pending, settled once at 16:05", minute: 11, tokens: 182000, stop: "held" },
  { at: 4900, attempt: "a3", spent: true, minute: 17, tokens: 300000, stop: "spent" },
];

export const EXTENSION = { at: 1300, attempt: "a3", witness: "w3", result: "holds", seen: "p-2307 kept unknown, then pending, settled once at 16:05", minute: 19, tokens: 341000, stop: "held" };

export const KVARM_OPS = {
  fund: [{ op: "create_arena", bytes: 1184, reason: "Brief and 3 candidate workspaces at 3f2a91c. Kvarm spawns nothing." }],
  results: [{ op: "rank_arena", bytes: 412, reason: "Read the duplication report only. Orrery orders attempts by witnesses.", note: "a1 and a3 both edit packages/core/src/retry.ts" }],
  pick: [
    { op: "record_decision", bytes: 736, reason: "Chosen and rejected attempts, each with its reason" },
    { op: "select_arena_winner", bytes: 288, reason: "Opens the final-selection escalation on #191" },
    { op: "resolve_escalation", bytes: 344, reason: "As you, with your note. Agents get HUMAN_REQUIRED." },
  ],
};
