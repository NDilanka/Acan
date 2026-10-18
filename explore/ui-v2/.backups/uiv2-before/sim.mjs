import { SIM } from "./acquit.mjs";

export const SIM_HTML = `
<div class="fl simdeck" id="simdeck">
  <div class="sd-row">
    <button class="sd-play" id="sdPlay" title="Play (Space)"><svg viewBox="0 0 12 12" width="12" height="12"><path id="sdIcon" d="M3 1.5v9l7.5-4.5z" fill="currentColor"/></svg></button>
    <div class="sd-clock"><b id="sdTime">14:00:00</b><small>${SIM.window}</small></div>
    <svg class="sd-scrub" id="sdScrub" viewBox="0 0 300 36" preserveAspectRatio="none"></svg>
    <button class="sd-chip" id="sdSpeed" title="Playback speed">60×</button>
  </div>
  <div class="sd-row sd-ctl">
    <label class="sd-mult">Traffic<input id="sdMult" type="range" min="-1" max="3.3219" step="0.01" value="0"><b id="sdMultV">1.0×</b></label>
    <button class="sd-chip" id="sdGhosts" aria-pressed="true" title="Run the ghosts on catalog defaults">Ghosts on</button>
    <span class="sd-down" id="sdDown"><span class="sd-hint">D takes the selected machine down · , . step a minute</span></span>
  </div>
  <div class="sd-res" id="sdRes"></div>
</div>`;

export const SIM_CSS = `
body[data-mode=simulate] .ent[data-kind=bot],body[data-mode=simulate] .ent[data-kind=crane],body[data-mode=simulate] .ent[data-kind=scaffold],body[data-mode=simulate] .ent[data-kind=inspection],
body[data-mode=simulate] .terr,body[data-mode=simulate] .pins,body[data-mode=simulate] .threads,body[data-mode=simulate] .floor,body[data-mode=simulate] .br,body[data-mode=simulate] .hint{display:none!important}
body[data-mode=simulate] .chip:not(.sim):not(.sel){opacity:0!important}
body[data-mode=simulate] .chip.prop::after{content:none}
body[data-mode=simulate] .chip.sim:not(.loud):not(.hover):not(.sel){opacity:0;translate:0 4px}
body[data-mode=simulate] .chip.sim.loud,body[data-mode=simulate] .chip.sim.hover,body[data-mode=simulate] .chip.sim.sel,body[data-mode=simulate].alt .chip.sim{opacity:1;translate:0 0}
body[data-mode=simulate] .chip.sim .rd{display:inline}
body[data-mode=simulate] .chip[data-state=ok] .mono{background:#cfe0d4}
body[data-mode=simulate] .chip[data-state=idle],body[data-mode=simulate] .chip[data-state=dark]{color:#777}
body[data-mode=simulate] .chip[data-state=dark] .mono{background:transparent;border:1px solid var(--fail);color:var(--fail)}
body[data-mode=simulate] .chip[data-state=idle] .mono{background:#3a3a3a;color:#999}
.chip.first::after{content:"breaks first";position:absolute;left:50%;bottom:calc(100% + 4px);transform:translateX(-50%);font:600 9px var(--mono);letter-spacing:.1em;color:var(--fail);text-transform:uppercase}
body[data-mode=simulate] .belt .iso-line[data-flow]{stroke:var(--anatomy-hi);animation-duration:var(--fd,1600ms)}
body[data-mode=simulate] .belt.warm .iso-line[data-flow]{stroke:var(--strain)}
body[data-mode=simulate] .belt.hot .iso-line[data-flow]{stroke:var(--fail)}
body[data-mode=simulate] .belt.dead .iso-line[data-flow]{animation:none;opacity:.25}

.simdeck{display:none;right:16px;bottom:16px;width:600px;flex-direction:column;align-items:stretch;gap:6px;padding:8px 10px 9px}
body[data-mode=simulate] .simdeck{display:flex}
.has-insp .simdeck{right:420px}
.sd-row{display:flex;align-items:center;gap:10px}
.sd-play{flex:none;width:30px;height:30px;border-radius:8px;display:grid;place-items:center;background:#cfe0d4;color:#0b100d}
.sd-clock{flex:none;display:flex;flex-direction:column;line-height:1.15}
.sd-clock b{font:500 13px var(--mono);color:#eef5f0;font-variant-numeric:tabular-nums}
.sd-clock small{font:9.5px var(--mono);color:var(--dim)}
.sd-scrub{flex:1;height:36px;cursor:ew-resize;overflow:visible}
.sd-scrub .area{fill:rgba(207,224,212,.12);stroke:rgba(207,224,212,.5);stroke-width:.8;vector-effect:non-scaling-stroke}
.sd-scrub .brk{fill:rgba(232,96,74,.28)}
.sd-scrub .ph{stroke:#eef5f0;stroke-width:1.2;vector-effect:non-scaling-stroke}
.sd-scrub .done{fill:rgba(207,224,212,.06)}
.sd-chip{flex:none;height:26px;padding:0 9px;border-radius:7px;border:1px solid var(--line2);font:10.5px var(--mono);color:#cfcfcf;white-space:nowrap}
.sd-chip:hover{background:rgba(255,255,255,.05)}
.sd-chip[aria-pressed=false]{color:var(--faint);border-style:dashed}
.sd-ctl{font-size:11.5px;color:var(--dim)}
.sd-mult{display:flex;align-items:center;gap:8px;flex:none}
.sd-mult input{width:150px;accent-color:#cfe0d4}
.sd-mult b{font:500 11.5px var(--mono);color:#eef5f0;min-width:38px}
.sd-down{display:flex;gap:4px;flex-wrap:wrap;min-width:0}
.sd-down button{height:24px;padding:0 8px;border-radius:7px;border:1px solid rgba(232,96,74,.45);color:#f0b2a6;font:10.5px var(--mono)}
.sd-hint{font:10px var(--mono);color:var(--faint);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.sd-res p{margin:0;flex:1;line-height:1.4}
.sd-res{display:flex;align-items:flex-start;gap:12px;padding:6px 2px 0;border-top:1px solid var(--line);font-size:12px;color:#d6d6d6}
.sd-res b{font-weight:500;color:#f2f2f2}
.sd-res .bad{color:#f0a796}.sd-res .okk{color:#bfe3cf}
.sd-res small{flex:none;text-align:right;font:10px/1.5 var(--mono);color:var(--dim);white-space:nowrap}
.sd-res button{font:inherit;color:#cfe0d4;text-decoration:underline;text-underline-offset:2px}

.sim-now{display:flex;align-items:baseline;gap:10px}
.sim-now b{font:400 24px var(--mono);color:#f2f2f2;font-variant-numeric:tabular-nums}
.sim-now span{font-size:12px;color:var(--dim)}
.sim-now[data-state=strain] b{color:#f1d5a2}.sim-now[data-state=fail] b{color:#f0a796}
.sim-now i{margin-left:auto;font:normal 10.5px var(--mono);color:var(--faint)}
.band{display:block;width:100%;height:46px;overflow:visible;margin-top:4px}
.band .rail{stroke:#3a3a3a}.band .bud{stroke:#8a8a8a;stroke-dasharray:2 2}
.band .unc{fill:rgba(207,224,212,.14)}.band .v{fill:#eef5f0}
.band text{font:9.5px var(--mono);fill:var(--dim)}
.load{display:block;width:100%;height:54px;overflow:visible}
.load path{fill:none;stroke:#cfe0d4;stroke-width:1.1;vector-effect:non-scaling-stroke}
.load .one{stroke:var(--fail);stroke-dasharray:3 3;stroke-width:.8}
.load .ph{stroke:#eef5f0;stroke-width:1;stroke-dasharray:none}
.load text{font:9px var(--mono);fill:var(--faint)}
.facts{display:grid;grid-template-columns:repeat(3,1fr);gap:6px;margin-top:10px}
.facts div{display:flex;flex-direction:column;gap:2px;padding:7px 8px;border-radius:8px;border:1px solid var(--line);font:9.5px var(--mono);color:var(--faint);text-transform:uppercase;letter-spacing:.06em}
.facts b{font:400 14px var(--mono);color:#eee;text-transform:none;letter-spacing:0}
.inp{display:grid;grid-template-columns:1fr auto;gap:1px 10px;padding:6px 0;border-top:1px solid var(--line);font-size:12px;color:#d6d6d6}
.inp:first-of-type{border-top:0}
.inp b{font:500 11.5px var(--mono);color:#eee}
.inp small{grid-column:1/-1;font:10px var(--mono);color:var(--faint)}
.acts button.down{color:#f0b2a6;border-color:rgba(232,96,74,.4)}
`;

export const SIM_SCRIPT = `
const SIMD = ${JSON.stringify(SIM)};
const SM = SIMD.model, ORDER = ["gate", "api", "verifier", "sqlite", "kiln", "paypal"];
const TAIL = { gate: [4, 4.6, 1000], api: [4, 4.6, 1000], verifier: [0, 3, 1], sqlite: [3.3, 4.6, 1000], kiln: [1.6, 4.6, 1] };
const SIMS = { mult: 1, down: new Set(), ghosts: true, t: 0, playing: false, speed: 60, run: null, saved: null, timer: 0, played: false };
const simOn = () => document.body.dataset.mode === "simulate";
function rng(seed) { return () => { seed = (seed + 0x6d2b79f5) | 0; let t = Math.imul(seed ^ (seed >>> 15), 1 | seed); t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t; return ((t ^ (t >>> 14)) >>> 0) / 4294967296; }; }
function arrival(t, r) { const b = SIMD.burst; let c = SIMD.ramp[0] + (SIMD.ramp[1] * t) / SIMD.secs; if (t >= b.t0 && t <= b.t1) c += b.add * Math.sin((Math.PI * (t - b.t0)) / (b.t1 - b.t0)); return c * (1 + (r() - 0.5) * 0.08); }
const wq = (S, c, rho) => (rho >= 1 ? 0 : (S * Math.pow(rho, Math.sqrt(2 * (c + 1)) - 1)) / (c * (1 - rho)));
const clock = (t) => { const s = SIMD.start + Math.round(t); return [s / 3600, (s / 60) % 60, s % 60].map((v) => String(Math.floor(v)).padStart(2, "0")).join(":"); };
const ghostIn = (id) => !!ENT[id] && GHOST[id] && (GHOST[id].stage === "built" || SIMS.ghosts);
function fmtV(id, v) {
  const u = SM[id].unit;
  if (v == null || !isFinite(v)) return "—";
  if (u === "%") return v.toFixed(1) + "%";
  if (u === "ms") return v >= 10000 ? (v / 1000).toFixed(0) + " s" : v >= 1000 ? (v / 1000).toFixed(1) + " s" : Math.round(v) + " ms";
  return v >= 100 ? Math.round(v / 60) + " min" : v >= 10 ? Math.round(v) + " s" : v.toFixed(1) + " s";
}

function simRun() {
  const r = rng(SIMD.seed), dt = SIMD.dt, n = SIMD.secs / dt, down = SIMS.down;
  const radar = ghostIn("radar"), breaker = ghostIn("breaker"), pp = SM.paypal;
  const cache = SCENE.entities.some((e) => e.kind === "buffer" && ghostIn(e.id)), hit = 0.8;
  const B = Object.fromEntries(ORDER.map((id) => [id, 0])), peak = {}, rhoMax = {}, steps = [];
  let tripAt = null, first = null, spend = 0, lastFail = null, okAfter = null;
  const out = ["gate", "api"].find((id) => down.has(id));
  if (out) first = { id: out, t: 0, why: "It is down, so every request fails." };
  for (let i = 0; i <= n; i++) {
    const t = i * dt, m = SIMS.mult * arrival(t, r), rps = SIMD.baseRps * m, M = {};
    const gateUp = !down.has("gate"), apiUp = gateUp && !down.has("api");
    let hold = pp.S * (1 + pp.err / 100 + (pp.err / 100) ** 2), ppErr = pp.err, tripped = false;
    if (down.has("paypal")) {
      ppErr = 100;
      const slow = pp.timeout * (pp.retries + 1) + 1.4;
      if (breaker) { if (tripAt === null && t >= pp.timeout + 5 / Math.max(0.01, rps * pp.per)) tripAt = t; tripped = tripAt !== null; hold = tripped ? 0.002 : slow; }
      else hold = slow;
    }
    let through = apiUp ? rps : 0;
    for (const id of ORDER) {
      const s = SM[id], isDown = down.has(id);
      let lam = id === "gate" ? (gateUp ? rps : 0) : id === "api" ? (apiUp ? rps : 0) : through * s.per * (id === "sqlite" && cache ? 1 - hit : 1);
      const Sx = id === "api" ? s.S * (radar ? 1.01 : 1) * (cache ? 0.95 : 1) : s.S;
      let work = id === "api" ? lam * Sx + lam * pp.per * hold : lam * (Sx || 0);
      const st = { lam, rho: null, val: null, err: 0, state: "ok" };
      if (isDown) {
        if (s.c) B[id] += work * dt;
        st.state = "dark"; st.val = s.c && id === "verifier" ? B[id] / s.c : null; M[id] = st; continue;
      }
      if (!s.c) {
        st.val = ppErr; st.err = ppErr; st.tripped = tripped;
        st.state = ppErr >= 50 ? (breaker && tripped ? "strain" : "fail") : s.budget && ppErr > s.budget ? "strain" : "ok";
      } else {
        const rho = work / s.c;
        B[id] = Math.max(0, B[id] + (work - s.c) * dt);
        const w = B[id] > 0 ? B[id] / s.c : wq(work / Math.max(lam, 1e-9), s.c, rho), [ts, tw, k] = TAIL[id];
        st.rho = rho; st.B = B[id]; st.val = (Sx * ts + w * tw) * k;
        if (s.unit === "ms" && st.val > 30000) { st.val = 30000; st.timeout = true; }
        if (id === "api") {
          st.err = s.err + (down.has("paypal") && !breaker ? pp.per * 100 : 0) + (down.has("kiln") ? SM.kiln.per * 100 : 0) + (down.has("sqlite") ? 100 : 0);
          if (B[id] > s.c * 2) st.err += Math.max(0, (1 - s.c / work) * 100);
          st.err = Math.min(100, st.err);
          through = lam * Math.min(1, s.c / Math.max(work, 1e-9));
        }
        const broken = (rho >= 1 && B[id] > s.c * s.S * 2) || st.err >= 50;
        const over = (s.budget && st.val > s.budget) || (s.errBudget && st.err > s.errBudget);
        st.state = lam === 0 ? "idle" : broken ? "fail" : over || rho >= 0.85 ? "strain" : "ok";
        rhoMax[id] = Math.max(rhoMax[id] || 0, rho);
        if (id === "kiln") spend += Math.min(lam, s.c / s.S) * dt;
      }
      if (st.state !== "dark" && st.state !== "idle") { peak[id] = Math.max(peak[id] || 0, st.val || 0); if (id === "api") peak.apiErr = Math.max(peak.apiErr || 0, st.err); }
      if (!first && st.state === "fail") first = { id, t, why: whyBroke(id, st, breaker) };
      if (first && first.id === id) { if (st.state === "fail") { lastFail = t; okAfter = null; } else if (st.state === "ok" && okAfter === null) okAfter = t; }
      M[id] = st;
    }
    steps.push({ t, rps, M });
  }
  const budgets = [["gate", "p99", peak.gate, SM.gate.budget], ["api", "p99", peak.api, SM.api.budget], ["api", "errors", peak.apiErr, SM.api.errBudget], ["verifier", "queue wait", peak.verifier, SM.verifier.budget], ["paypal", "errors", peak.paypal, SM.paypal.budget]]
    .map(([id, what, v, b]) => ({ id, what, v, b, met: v == null ? null : v <= b }));
  const headroom = Object.fromEntries(Object.entries(rhoMax).map(([id, r2]) => [id, r2 > 0 ? SIMS.mult / r2 : Infinity]));
  const spendH = 1.42 * (spend / SIMD.secs) / (SIMD.baseRps * SM.kiln.per);
  const recovered = first && !out && okAfter !== null ? okAfter : null;
  const backlog = down.has("verifier") ? Math.round(B.verifier / SM.verifier.S) : 0;
  return { cache, steps, first, recovered, backlog, budgets, headroom, spendH, tripAt, radar, breaker };
}
function whyBroke(id, st, breaker) {
  if (id === "api" && SIMS.down.has("paypal") && !breaker) return "Retries to the dead PayPal hold all " + SM.api.c + " slots, 4 tries of 10 s each.";
  if (id === "api" && SIMS.down.has("sqlite")) return "Every request needs the database.";
  if (id === "api" && SIMS.down.has("paypal")) return "Retries fill its " + SM.api.c + " slots until the breaker trips.";
  if (st.err >= 50) return "Errors reach " + Math.round(st.err) + "%.";
  return SM[id].metric + " reaches " + fmtV(id, st.val) + " and the queue keeps growing.";
}
function stepAt(t) { const run = SIMS.run; return run.steps[Math.min(run.steps.length - 1, Math.max(0, Math.round(t / SIMD.dt)))]; }
function readingFor(id, st) {
  if (st.state === "dark") return id === "verifier" && st.val ? "down · backlog " + fmtV(id, st.val) : "down";
  if (st.state === "idle") return "no traffic";
  if (st.timeout) return "timing out · " + Math.round(st.err) + "% err";
  if (id === "paypal") return st.tripped ? "breaker open · payouts deferred" : fmtV(id, st.val) + " errors";
  if (id === "api" && st.err > SM.api.errBudget) return SM.api.metric + " " + fmtV(id, st.val) + " · " + st.err.toFixed(0) + "% err";
  return (id === "verifier" ? "wait " : id === "sqlite" ? "q p99 " : "p99 ") + fmtV(id, st.val);
}

function simApply() {
  if (!simOn() || !SIMS.run) return;
  const step = stepAt(SIMS.t);
  let changed = false;
  for (const id of ORDER) { const e = ENT[id]; if (e && e.state !== step.M[id].state) { e.state = step.M[id].state; changed = true; } }
  if (changed) { dirtyScene = true; animate(); } else { simBelts(); simChips(); }
  $("#sdTime").textContent = clock(SIMS.t);
  drawScrub();
  if (selected && $("#simNow")) paintSimNow(selected);
}
function simChips() {
  const step = stepAt(SIMS.t), run = SIMS.run;
  for (const id of ORDER) {
    const st = step.M[id], chip = $('.chip[data-id="' + id + '"]'); if (!chip) continue;
    chip.dataset.state = st.state; chip.querySelector(".rd").textContent = readingFor(id, st);
    chip.classList.toggle("loud", st.state !== "ok" && st.state !== "idle");
    chip.classList.toggle("first", !!run.first && run.first.id === id && SIMS.t >= run.first.t);
  }
  for (const id of Object.keys(SIMD.ghosts)) {
    const chip = $('.chip[data-id="' + id + '"]'); if (!chip || !ENT[id]) continue;
    const tripped = id === "breaker" && ghostIn(id) && step.M.paypal.tripped;
    chip.querySelector(".rd").textContent = !ghostIn(id) ? "left out of the run" : tripped ? "open · failing fast" : "in the run · confidence " + SIMD.ghosts[id].conf.toFixed(2);
    chip.classList.toggle("loud", tripped);
  }
}
function simBelts() {
  if (!simOn() || !SIMS.run) return;
  const step = stepAt(SIMS.t), base = SIMD.baseRps;
  for (const b of plant.querySelectorAll(".belt")) {
    const to = step.M[b.dataset.to], from = step.M[b.dataset.from], m = to || from;
    const ratio = m ? Math.max(0.25, Math.min(4, (m.lam || 0) / (base * (SM[b.dataset.to] ? SM[b.dataset.to].per : 1) || 1))) : 1;
    b.style.setProperty("--fd", Math.round(1600 / ratio) + "ms");
    const dead = (to && (to.state === "dark" || to.state === "idle")) || (from && (from.state === "dark" || from.state === "idle"));
    b.classList.toggle("dead", !!dead);
    b.classList.toggle("hot", !dead && !!to && to.state === "fail");
    b.classList.toggle("warm", !dead && !!to && to.state === "strain");
  }
}
function simAfterRender() { if (simOn() && SIMS.run) { simBelts(); simChips(); for (const id of [...ORDER, ...Object.keys(SIMD.ghosts)]) { const c = $('.chip[data-id="' + id + '"]'); if (c) c.classList.add("sim"); } } }

function drawScrub() {
  const run = SIMS.run, W2 = 300, H2 = 36, max = Math.max(...run.steps.map((s) => s.rps)) * 1.08, X = (t) => (t / SIMD.secs) * W2;
  const area = "M0 " + H2 + run.steps.map((s) => "L" + X(s.t).toFixed(1) + " " + (H2 - (s.rps / max) * (H2 - 4)).toFixed(1)).join("") + "L" + W2 + " " + H2 + "Z";
  let brk = "", on = null;
  for (const s of run.steps) { const bad = ORDER.some((id) => s.M[id].state === "fail"); if (bad && on === null) on = s.t; if ((!bad || s === run.steps[run.steps.length - 1]) && on !== null) { brk += '<rect class="brk" x="' + X(on).toFixed(1) + '" y="0" width="' + Math.max(1.5, X(s.t) - X(on)).toFixed(1) + '" height="' + H2 + '"/>'; on = null; } }
  $("#sdScrub").innerHTML = brk + '<path class="area" d="' + area + '"/><rect class="done" x="0" y="0" width="' + X(SIMS.t).toFixed(1) + '" height="' + H2 + '"/><line class="ph" x1="' + X(SIMS.t).toFixed(1) + '" x2="' + X(SIMS.t).toFixed(1) + '" y1="-3" y2="' + (H2 + 3) + '"/>';
}
function paintResult() {
  const run = SIMS.run, met = run.budgets.filter((b) => b.met === true).length, all = run.budgets.filter((b) => b.met !== null).length;
  let h;
  if (run.first) h = '<b class="bad">' + ENT[run.first.id].name + " breaks at " + clock(run.first.t).slice(0, 5) + ".</b> " + run.first.why;
  else {
    const tight = Object.entries(run.headroom).filter(([id]) => !SIMS.down.has(id)).sort((a, b) => a[1] - b[1])[0];
    h = '<b class="okk">Nothing breaks at ' + SIMS.mult.toFixed(1) + "×.</b> Tightest is " + ENT[tight[0]].name + ", which saturates at " + tight[1].toFixed(1) + "×.";
  }
  if (run.breaker && SIMS.down.has("paypal") && run.tripAt !== null) h += " The breaker trips at " + clock(run.tripAt) + ", and payouts wait for PayPal.";
  if (run.recovered) h += " It recovers at " + clock(run.recovered) + ".";
  if (run.backlog) h += " " + run.backlog + " jobs wait behind verifier by 15:00.";
  const focus = run.first ? run.first.id : Object.entries(run.headroom).filter(([id]) => !SIMS.down.has(id)).sort((a, b) => a[1] - b[1])[0][0];
  $("#sdRes").innerHTML = "<p>" + h + ' <button data-go="' + focus + '">Show</button></p><small>budgets met ' + met + " of " + all + "<br>model spend $" + run.spendH.toFixed(2) + "/h</small>";
}
function simRecompute() {
  SIMS.run = simRun(); paintResult();
  $("#sdDown").innerHTML = SIMS.down.size ? [...SIMS.down].map((id) => '<button data-up="' + id + '" title="Bring back up">' + ENT[id].name + " down ×</button>").join("") : '<span class="sd-hint">D takes the selected machine down · , . step a minute</span>';
  simApply();
  if (selected && $("#ibody") && (SM[selected] || SIMD.ghosts[selected])) refreshSimInspector();
}

function simTick() {
  SIMS.t = Math.min(SIMD.secs, SIMS.t + SIMS.speed * 0.2);
  if (SIMS.t >= SIMD.secs) simPlay(false);
  simApply();
}
function simPlay(on = !SIMS.playing) {
  if (on && SIMS.t >= SIMD.secs) SIMS.t = 0;
  SIMS.playing = on; clearInterval(SIMS.timer);
  if (on) SIMS.timer = setInterval(simTick, 200);
  $("#sdIcon").setAttribute("d", on ? "M2.5 1.5h2.6v9H2.5zM6.9 1.5h2.6v9H6.9z" : "M3 1.5v9l7.5-4.5z");
}
function simMode(m) {
  if (m === "simulate" && !SIMS.saved) {
    SIMS.saved = Object.fromEntries(SCENE.entities.map((e) => [e.id, e.state]));
    if (selected) select(null);
    SIMS.run = null; simRecompute(); simAfterRender();
    if (!SIMS.played) { SIMS.played = true; simPlay(true); }
  } else if (m !== "simulate" && SIMS.saved) {
    simPlay(false);
    for (const e of SCENE.entities) if (e.id in SIMS.saved) e.state = SIMS.saved[e.id];
    SIMS.saved = null;
    for (const c of document.querySelectorAll(".chip")) { const e = ENT[c.dataset.id]; if (!e) continue; c.classList.remove("sim", "first"); c.dataset.state = e.state; c.querySelector(".rd").textContent = e.reading; }
    for (const b of plant.querySelectorAll(".belt")) b.classList.remove("hot", "warm", "dead");
    if (selected) select(null);
    dirtyScene = true; animate();
  }
}

function simInspector(e) {
  const s = SM[e.id], g = SIMD.ghosts[e.id];
  let h = '<div class="fig" id="fig"><div class="iso" id="figPlant"></div><div class="read" id="simRead">' + (g ? "ghost<small>confidence " + g.conf.toFixed(2) + "</small>" : "") + '</div><div class="tools"><button data-do="open">Open code <kbd>O</kbd></button><button data-do="hand">Hand to agent <kbd>H</kbd></button></div></div>';
  if (g) {
    h += '<div class="sec"><h5>Ghost in the run<i><span class="lowc">confidence ' + g.conf.toFixed(2) + "</span></i></h5><div class=\\"inp\\"><span>Effect</span><b></b><small>" + g.effect + '</small></div><div class="inp"><span>Numbers from</span><b></b><small>' + g.from + "</small></div></div>";
    h += '<div class="note">Ghosts run on catalog defaults until they are built. Their predictions carry low confidence, so nearby bands widen.</div>';
    h += '<div class="acts"><button data-sim="ghosts">' + (SIMS.ghosts ? "Leave ghosts out" : "Put ghosts back in") + "</button></div>";
    return h;
  }
  h += '<div class="sec"><h5>In this run<i id="siT"></i></h5><div id="simNow"></div></div>';
  h += '<div class="sec"><h5>Load over the hour<i>utilization · 1.0 saturates</i></h5><div id="simLoad"></div></div>';
  h += '<div class="sec"><h5>Inputs<i>from the twin</i></h5>' + s.inputs.map(([k2, v, src]) => '<div class="inp"><span>' + k2 + "</span><b>" + v + "</b><small>" + src + "</small></div>").join("") + "</div>";
  if (s.unmodeled) h += '<div class="note">Not modeled. ' + s.unmodeled + "</div>";
  h += '<div class="acts"><button class="down" data-sim="down">' + (SIMS.down.has(e.id) ? "Bring it back up" : "Take it down") + " <kbd>D</kbd></button></div>";
  return h;
}
function uncertainty(id) {
  const u = (1 - SM[id].cal) * 1.2, ghosty = (id === "api" && (SIMS.run.radar || SIMS.run.cache)) || (id === "sqlite" && SIMS.run.cache) || (id === "api" && SIMS.run.breaker && SIMS.down.has("paypal"));
  return { u: ghosty ? Math.max(u, 0.5) : u, ghosty };
}
function paintSimNow(id) {
  const s = SM[id], box = $("#simNow"); if (!s || !box) return;
  const step = stepAt(SIMS.t), st = step.M[id], run = SIMS.run, { u, ghosty } = uncertainty(id);
  $("#siT").textContent = "at " + clock(SIMS.t);
  $("#simRead").innerHTML = readingFor(id, st) + "<small>predicted · ±" + Math.round(u * 100) + "%</small>";
  const word = { ok: "within budget", strain: s.budget && st.val > s.budget ? "over budget" : "near capacity", fail: "breaks · queue keeps growing", dark: "taken down in this run", idle: "no traffic reaches it" }[st.state];
  let h = '<div class="sim-now" data-state="' + st.state + '"><b>' + fmtV(id, st.val) + "</b><span>" + s.metric + " · " + word + "</span><i>±" + Math.round(u * 100) + "%</i></div>";
  if (st.val != null && isFinite(st.val) && s.budget) {
    const max = Math.max(s.budget * 1.7, st.val * (1 + u) * 1.08), X = (v) => 4 + (356 * Math.min(v, max)) / max;
    h += '<svg class="band" viewBox="0 0 364 46"><line class="rail" x1="4" x2="360" y1="20" y2="20"/><rect class="unc" x="' + X(st.val * (1 - u)).toFixed(1) + '" y="14" width="' + Math.max(2, X(st.val * (1 + u)) - X(st.val * (1 - u))).toFixed(1) + '" height="12" rx="2"/><line class="bud" x1="' + X(s.budget) + '" x2="' + X(s.budget) + '" y1="6" y2="34"/><circle class="v" cx="' + X(st.val).toFixed(1) + '" cy="20" r="3"/><text x="' + X(s.budget) + '" y="44" text-anchor="' + (X(s.budget) < 50 ? "start" : X(s.budget) > 314 ? "end" : "middle") + '">budget ' + fmtV(id, s.budget) + "</text></svg>";
  }
  const hr = run.headroom[id];
  h += '<div class="facts"><div>headroom<b>' + (hr == null ? "—" : hr === Infinity ? "∞" : hr.toFixed(1) + "×") + "</b></div><div>calibration<b>" + s.cal.toFixed(2) + "</b></div><div>" + (ghosty ? "ghosts in path<b>low conf</b>" : "peak<b>" + fmtV(id, Math.max(...run.steps.map((x) => (x.M[id].state === "dark" ? 0 : x.M[id].val || 0)))) + "</b>") + "</div></div>";
  box.innerHTML = h;
  const L = $("#simLoad");
  if (L && s.c) {
    const rhos = run.steps.map((x) => (x.M[id].rho == null ? 0 : Math.min(2, x.M[id].rho))), top = Math.max(1.2, ...rhos), Y = (v) => 50 - (v / top) * 46, X2 = (i) => (i / (rhos.length - 1)) * 364;
    L.innerHTML = '<svg class="load" viewBox="0 0 364 54" preserveAspectRatio="none"><line class="one" x1="0" x2="364" y1="' + Y(1) + '" y2="' + Y(1) + '"/><path d="' + rhos.map((v, i) => (i ? "L" : "M") + X2(i).toFixed(1) + " " + Y(v).toFixed(1)).join("") + '"/><line class="ph" x1="' + ((SIMS.t / SIMD.secs) * 364).toFixed(1) + '" x2="' + ((SIMS.t / SIMD.secs) * 364).toFixed(1) + '" y1="0" y2="54"/></svg>';
  } else if (L) L.innerHTML = '<div class="note">External. Orrery sees its errors and call times, not its capacity.</div>';
}
function refreshSimInspector() {
  const e = ENT[selected];
  $("#ibody").innerHTML = simInspector(e);
  $("#ih").dataset.state = e.state; $("#iKind").textContent = "Simulated · queueing model on recorded traffic";
  paintSimNow(selected); requestAnimationFrame(drawCloseup);
}
function simToggleDown(id) {
  if (!SM[id]) return;
  SIMS.down.has(id) ? SIMS.down.delete(id) : SIMS.down.add(id);
  simRecompute();
  toast(SIMS.down.has(id) ? ENT[id].name + " is down for this run. Nothing changes in Live." : ENT[id].name + " is back up.");
}

$("#sdPlay").onclick = () => simPlay();
$("#sdSpeed").onclick = () => { const o = [10, 60, 300]; SIMS.speed = o[(o.indexOf(SIMS.speed) + 1) % o.length]; $("#sdSpeed").textContent = SIMS.speed + "×"; };
$("#sdMult").oninput = (ev) => { SIMS.mult = Math.round(Math.pow(2, +ev.target.value) * 10) / 10; $("#sdMultV").textContent = SIMS.mult.toFixed(1) + "×"; simRecompute(); };
$("#sdGhosts").onclick = () => { SIMS.ghosts = !SIMS.ghosts; $("#sdGhosts").setAttribute("aria-pressed", SIMS.ghosts); $("#sdGhosts").textContent = SIMS.ghosts ? "Ghosts on" : "Ghosts off"; simRecompute(); };
$("#sdDown").addEventListener("click", (ev) => { const b = ev.target.closest("[data-up]"); if (b) simToggleDown(b.dataset.up); });
$("#sdRes").addEventListener("click", (ev) => { const b = ev.target.closest("[data-go]"); if (b) select(b.dataset.go); });
{
  const scrub = $("#sdScrub");
  const seek = (ev) => { const r = scrub.getBoundingClientRect(); SIMS.t = Math.max(0, Math.min(1, (ev.clientX - r.left) / r.width)) * SIMD.secs; simApply(); };
  scrub.addEventListener("pointerdown", (ev) => { simPlay(false); seek(ev); scrub.setPointerCapture(ev.pointerId); scrub.onpointermove = seek; });
  scrub.addEventListener("pointerup", () => { scrub.onpointermove = null; });
}
$("#ibody").addEventListener("click", (ev) => {
  const b = ev.target.closest("[data-sim]"); if (!b || !simOn()) return;
  if (b.dataset.sim === "down") simToggleDown(selected);
  if (b.dataset.sim === "ghosts") $("#sdGhosts").click();
});
addEventListener("keydown", (ev) => {
  if (!simOn() || !bare(ev)) return;
  if (ev.key === " ") { ev.preventDefault(); simPlay(); }
  if (ev.key.toLowerCase() === "d" && selected) simToggleDown(selected);
  const step = { ",": -60, ".": 60, ArrowLeft: -30, ArrowRight: 30 }[ev.key];
  if (step && SIMS.run) { ev.preventDefault(); simPlay(false); SIMS.t = Math.max(0, Math.min(SIMD.secs, SIMS.t + step)); simApply(); }
  if (ev.key === "Home" && SIMS.run) { simPlay(false); SIMS.t = 0; simApply(); }
  if (ev.key === "End" && SIMS.run) { simPlay(false); SIMS.t = SIMD.secs; simApply(); }
});
function simInit(params) {
  if (params.get("mult")) { const v = +params.get("mult"); $("#sdMult").value = Math.log2(v); SIMS.mult = v; $("#sdMultV").textContent = v.toFixed(1) + "×"; }
  if (params.get("down")) for (const id of params.get("down").split(",")) SIMS.down.add(id);
  if (params.get("noghosts")) { SIMS.ghosts = false; $("#sdGhosts").setAttribute("aria-pressed", false); $("#sdGhosts").textContent = "Ghosts off"; }
  if (params.get("t") && document.body.dataset.mode === "simulate") { simPlay(false); SIMS.t = +params.get("t"); simRecompute(); }
  else if (document.body.dataset.mode === "simulate") simRecompute();
  if (params.get("ssel")) select(params.get("ssel"));
}
`;
