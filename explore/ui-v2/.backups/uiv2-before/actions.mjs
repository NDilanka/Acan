export const ACT_SCRIPT = `
const REV = "3f2a91c";
const SEVERITY = { fail: 0, strain: 1, wait: 2, ghost: 3, run: 4, ok: 5, idle: 6, dark: 6 };
const ACTIONS = {
  open: { label: "Open code", key: "O" },
  hand: { label: "Hand to agent", key: "H" },
  brief: { label: "Copy brief", key: "C" },
  "brief-safe": { label: "Copy share-safe brief", key: "Shift C" },
  terminal: { label: "Open its terminal", key: "" },
  log: { label: "Open session log", key: "" },
  runlog: { label: "Open run log", key: "R" },
  rerun: { label: "Re-run checks", key: "Shift R" },
  branch: { label: "Open branch diff", key: "" },
};
let handoffs = [];

function context() {
  if (review && review.i >= 0) return { kind: "choice", c: CH[review.i], id: CH[review.i].id };
  if (session) return { kind: "session", s: SESSIONS.find((x) => x.id === session), id: session };
  if (selected) return { kind: "entity", id: selected };
  if (rpOn() && RP.first) return { kind: "entity", id: RP.first };
  return null;
}
const uniq = (a) => [...new Set(a)];
function refsFor(ctx) {
  if (!ctx) return [];
  if (ctx.kind === "choice") return ctx.c.evidence.filter((r) => r.includes("/"));
  if (ctx.kind === "session") return ctx.s.files.map((f) => f[0]);
  const id = ctx.id, d = INSPECT[id];
  return uniq([...(d ? d.drift.map((x) => x.ref).filter((r) => r.includes("/")) : []), ...SESSIONS.flatMap((s) => s.files.filter((f) => f[1] === id).map((f) => f[0])), ...CH.filter((c) => c.on === id).flatMap((c) => c.evidence.filter((r) => r.includes("/")))]);
}
function ownerOf(ctx) {
  const live = SESSIONS.filter((s) => s.state !== "done");
  if (ctx.kind === "session") return ctx.s;
  if (ctx.kind === "choice") return live.find((s) => s.id === ctx.c.session) || null;
  const mine = live.filter((s) => (s.territory && TERR[s.territory].entities.includes(ctx.id)) || s.files.some((f) => f[1] === ctx.id));
  return mine.find((s) => s.via[0][1]) || mine[0] || null;
}
const nameOf = (ctx) => (ctx.kind === "entity" ? ENT[ctx.id].name : ctx.kind === "session" ? ctx.s.agent : "choice " + ctx.c.id);

function whatLines(ctx) {
  const m = document.body.dataset.mode;
  if (ctx.kind === "choice") { const c = ctx.c; return [c.question, c.by + " chose: " + c.text, "Confidence " + c.conf.toFixed(2) + ", reversal cost " + c.cost + ", status " + c.status + "."]; }
  if (ctx.kind === "session") { const s = ctx.s; return [s.agent + " · " + s.goal + ".", STATE_LABEL[s.state] + " for " + s.since + ". " + s.now]; }
  const id = ctx.id, e = ENT[id];
  if (m === "replay" && RP.frames.length) return story().what.concat(RPD.series[id] && id !== RP.first ? ["At " + rpClock(RP.t) + " " + e.name + " reads " + rpReading(id, rpFrame().M[id]) + "."] : []);
  if (m === "simulate" && SIMS.run && SM[id]) {
    const res = $("#sdRes p") ? $("#sdRes p").textContent.replace(/ Show$/, "") : "";
    return ["Simulated at " + clock(SIMS.t) + " on " + SIMD.window + ", traffic " + SIMS.mult.toFixed(1) + "×" + (SIMS.down.size ? ", down " + [...SIMS.down].join(", ") : "") + (SIMS.ghosts ? ", ghosts in" : "") + ", seed " + SIMD.seed + ".", e.name + ": " + readingFor(id, stepAt(SIMS.t).M[id]) + " (predicted).", res];
  }
  const d = INSPECT[id], out = [e.name + ": " + e.reading + ". State " + e.state + "."];
  if (d) for (const b of d.budgets) out.push(b.metric + " " + b.value + " " + b.unit + " against a budget of " + b.budget + " " + b.unit + (b.value > b.budget ? ", over." : ", within."));
  if (GHOST[id]) out.push("Ghost, stage " + GHOST[id].stage + ". " + (GHOST[id].why || ""));
  return out;
}
function evidenceLines(ctx) {
  if (ctx.kind === "choice") return [...ctx.c.evidence.map((r) => "- ref       " + r), ...ctx.c.alts.map((a) => "- weighed   " + a.text + " (" + a.note + ")")];
  if (ctx.kind === "session") return [...ctx.s.files.map(([f, id, n]) => "- edits     " + f + " → " + ENT[id].name + ", " + n), ...ctx.s.via.map(([n, ok, d]) => "- via       " + n + (ok ? " on" : " off") + (d ? ", " + d : ""))];
  const id = ctx.id, d = INSPECT[id], out = [];
  if (document.body.dataset.mode === "replay" && RP.frames.length) return [...RPD.changes.map((c) => "- change    " + rpClock(toSec(c.t)) + " " + c.text + " (" + c.detail + ")"), ...RP.events.filter((x) => x.on === id).slice(0, 8).map((x) => "- event     " + rpClock(x.sec) + " " + x.text)];
  if (d) { for (const s of d.sources) out.push("- " + s.src.padEnd(9) + " " + s.what + " (" + s.detail + ")"); for (const x of d.drift) out.push("- " + x.sev.padEnd(9) + " " + x.text + " (" + x.ref + ")"); }
  for (const c of CH.filter((c) => c.on === id)) out.push("- choice    " + c.id + ", confidence " + c.conf.toFixed(2) + ", reversal " + c.cost + ", " + c.status);
  for (const s of SESSIONS.filter((s) => s.files.some((f) => f[1] === id))) out.push("- agent     " + s.agent + " · " + s.goal + " (" + STATE_LABEL[s.state].toLowerCase() + ")");
  return out;
}
function replaceCI(t, n, r) {
  let out = "", i = 0, low = t.toLowerCase(), k = n.toLowerCase(), j;
  while ((j = low.indexOf(k, i)) >= 0) { out += t.slice(i, j) + r; i = j + k.length; }
  return out + t.slice(i);
}
function redact(t) {
  const names = [...SCENE.entities.map((e) => [e.name, "[" + e.mono + "]"]), ...Object.keys(TERR).map((id) => [id, "[territory]"]), ...CH.map((c) => [c.id, "[choice]"]), ["acquit", "[project]"]]
    .filter(([n]) => n.length > 3).sort((a, b) => b[0].length - a[0].length);
  t = t.replace(/\\b[a-z0-9-]+(\\.[a-z0-9-]+)+\\.(com|net|io|org|dev)\\b/gi, "[host]");
  t = t.replace(/[\\w.@-]*(\\/[\\w.@*-]+)+(:\\d+(-\\d+)?)?/g, "[path]");
  t = t.replace(/\\b[A-Z][A-Z0-9]*_[A-Z0-9_*]+\\b/g, "[env]").replace(/:\\d{4,5}\\b/g, ":[port]").replace(/\\bport \\d{2,5}\\b/gi, "port [port]").replace(new RegExp(REV, "g"), "[rev]");
  for (const [n, r] of names) t = replaceCI(t, n, r);
  return t;
}
function brief(ctx, safe = false) {
  const m = document.body.dataset.mode, refs = refsFor(ctx), owner = ownerOf(ctx);
  const kind = ctx.kind === "entity" ? (GHOST[ctx.id] ? "ghost" : { fail: "failing", strain: "over_budget", wait: "waiting" }[ENT[ctx.id].state] || "machine") : ctx.kind;
  const open = "acan://acquit/" + ctx.kind + "/" + ctx.id + "?mode=" + m + (m === "replay" ? "&t=" + rpClock(RP.t) : m === "simulate" ? "&t=" + Math.round(SIMS.t) + "&mult=" + SIMS.mult + "&seed=" + SIMD.seed + (SIMS.down.size ? "&down=" + [...SIMS.down].join(",") : "") + (SIMS.ghosts ? "" : "&noghosts=1") : "");
  const cons = ["- Do not edit budgets or checks (K16)."];
  if (owner && ctx.kind !== "session") cons.push("- " + (owner.territory ? owner.territory + " is claimed by " + owner.agent + "." : owner.agent + " is editing these files.") + " Coordinate before you change them.");
  if (m === "simulate") cons.push("- Values are model predictions, not measurements.");
  if (m === "replay") cons.push("- Values are recorded. The plant has moved on since.");
  const head = ["---", "acan: brief/1", "about: " + (safe && ctx.kind === "entity" ? "[" + ENT[ctx.id].mono + "]" : ctx.id), "kind: " + kind, "mode: " + m, "rev: " + REV, "redaction: " + (safe ? "share-safe" : "local"), "refs:" + (refs.length ? "" : " []"), ...refs.map((r) => "  - " + r)];
  if (!safe) head.push("open: " + open);
  const body = [...head, "---", "## What", ...whatLines(ctx).filter(Boolean), "## Evidence", ...(evidenceLines(ctx).length ? evidenceLines(ctx) : ["- none mapped yet"]), "## Constraints", ...cons, "## Ask", ""].join("\\n");
  return safe ? body.split("\\n").map((l, i) => (i === 1 ? l : redact(l))).join("\\n") : body;
}
function copyText(t) {
  window.__copied = t;
  try { navigator.clipboard.writeText(t).catch(() => {}); } catch (err) {}
}
function handoff(target, payload, msg) { handoffs.push({ target, payload, at: Date.now() }); window.__handoff = handoffs[handoffs.length - 1]; if (msg) toast(msg); }

function runAction(id, ctx = context()) {
  if (!ctx) return toast("Select a machine, session or choice first. Ctrl K finds one.");
  const nm = nameOf(ctx);
  if (id === "brief" || id === "brief-safe") { const b = brief(ctx, id === "brief-safe"); copyText(b); return toast((id === "brief" ? "Brief" : "Share-safe brief") + " for " + nm + " copied · " + b.split("\\n").length + " lines. Paste it into any agent."); }
  if (id === "open") {
    const r = refsFor(ctx)[0]; if (!r) return toast("No code is mapped to " + nm + " yet.");
    const [path, line] = r.split(":");
    return handoff("editor", ["code", "--goto", path + ":" + ((line || "1").split("-")[0])], "Opened " + r + " in your editor" + (refsFor(ctx).length > 1 ? " · " + (refsFor(ctx).length - 1) + " more in Ctrl K" : "") + ".");
  }
  if (id === "hand") {
    const o = ownerOf(ctx), b = brief(ctx);
    if (o && o.via[0][1]) return handoff("mcp", { session: o.id, brief: b }, "Handed to " + o.agent + " over MCP. It reads the brief on its next turn.");
    copyText(b);
    return handoff("clipboard", { brief: b }, o ? o.agent + " has no MCP link. Brief copied, paste it into its terminal." : "No agent works on " + nm + ". Brief copied, paste it into any agent.");
  }
  if (id === "terminal") { const s = ctx.s || ownerOf(ctx); return handoff("terminal", { session: s && s.id }, s && s.perm === "passive" ? "Focused " + s.agent + "'s terminal. Its prompt is waiting there." : "Opened a terminal in the " + ((s && s.territory) || "repo") + " worktree."); }
  if (id === "log") { const s = ctx.s || ownerOf(ctx); return handoff("log", { session: s && s.id }, s ? "Opened " + s.agent + "'s session log." : "No session here."); }
  if (id === "runlog" || id === "rerun") {
    const r = CI.runs[CI.runs.length - 1];
    if (id === "runlog") return handoff("ci", { run: r.n }, "Opened the log for CI #" + r.n + " at its first unfinished check.");
    return handoff("ci", { run: r.n, rerun: true }, "Re-running " + r.what.split(" ")[0] + " checks through run_checks. Nothing in .acan/checks/ changes.");
  }
  if (id === "branch") return handoff("git", { branch: "radar/otel", base: "main" }, "Opened the radar/otel diff against main.");
}

function tabOrder() {
  const hidden = rpOn() ? hiddenAt(RP.t / 60 - RPD.mins) : [];
  return SCENE.entities.filter((e) => !["scaffold", "inspection"].includes(e.kind) && !hidden.includes(e.id) && (document.body.classList.contains("ghosts") || e.state !== "ghost"))
    .sort((a, b) => (SEVERITY[a.state] ?? 5) - (SEVERITY[b.state] ?? 5) || a.x - b.x).map((e) => e.id);
}
let tabAt = -1;
function tabStep(d) {
  const ids = tabOrder(), cur = session ? SESSIONS.find((s) => s.id === session).bot : selected;
  const i = ids.indexOf(cur), j = ((i < 0 ? (d > 0 ? -1 : 0) : i) + d + ids.length) % ids.length;
  tabAt = j; select(ids[j]);
}

document.addEventListener("click", (ev) => {
  const b = ev.target.closest("[data-do]"); if (!b) return;
  if (b.dataset.do === "palette") return palOpen();
  runAction(b.dataset.do);
});
addEventListener("keydown", (ev) => {
  const k = ev.key.toLowerCase();
  if ((ev.ctrlKey || ev.metaKey) && k === "k") { ev.preventDefault(); palToggle(); return; }
  if ((ev.ctrlKey || ev.metaKey) && k === "c" && !typing(ev) && !String(getSelection()).trim() && context()) { ev.preventDefault(); runAction(ev.shiftKey ? "brief-safe" : "brief"); return; }
  if (ev.key === "Tab" && !typing(ev) && !ev.ctrlKey && !ev.altKey && !ev.metaKey) { ev.preventDefault(); tabStep(ev.shiftKey ? -1 : 1); return; }
  if (typing(ev) || ev.ctrlKey || ev.metaKey || ev.altKey) return;
  if (k === "c") runAction(ev.shiftKey ? "brief-safe" : "brief");
  if (k === "o") runAction("open");
  if (k === "h") runAction("hand");
  if (k === "r" && selected === "crane") runAction(ev.shiftKey ? "rerun" : "runlog");
});
`;
