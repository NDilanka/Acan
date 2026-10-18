export const SH_HTML = `
<div class="pop expop" id="ex" role="dialog" aria-label="Export">
  <h4>Export<i id="exWhat"></i></h4>
  <div class="seg" id="exFmt"><button data-fmt="png" aria-pressed="true">PNG</button><button data-fmt="svg" aria-pressed="false">SVG</button><button data-fmt="frames" aria-pressed="false">Frames</button></div>
  <label>Share-safe <small>R2 set</small><input type="checkbox" id="exSafe" checked><span class="sw"></span></label>
  <div class="exlist" id="exList"></div>
  <div class="exsize" id="exSize"></div>
  <button class="exgo" id="exGo">Save PNG <kbd>Enter</kbd></button>
  <div class="exnote">Saved on this machine. Nothing is uploaded.</div>
</div>
<div class="exframe" id="exFrame"><div class="tblock" id="exTitle"></div></div>
<div class="pop netpop" id="net" role="dialog" aria-label="Network">
  <h4>Outbound since launch<i id="netSince"></i></h4>
  <div class="netlog" id="netLog"></div>
  <h4>Loopback<i>not counted · never leaves</i></h4>
  <div class="netlb">
    <div><b>OTLP in</b><span>127.0.0.1:4318</span><em>3 agents, 1 app</em></div>
    <div><b>MCP</b><span>127.0.0.1:4319</span><em>2 clients</em></div>
  </div>
  <label>Offline <small>blocks all egress</small><input type="checkbox" id="netOff"><span class="sw"></span></label>
  <h4>What Orrery can ever send</h4>
  <div class="netst">
    <div><b>Update check</b><span>version number only · off until you turn it on</span></div>
    <div><b>Crash report</b><span>stack trace, no paths · asks each time</span></div>
    <div><b>Remote suggestions</b><span>Jev, opt-in · share-safe briefs only</span></div>
    <small>Nothing else. Telemetry, code and transcripts stay on this machine.</small>
  </div>
</div>`;

export const SH_CSS = `
.pop h4 i{float:right;font-style:normal;letter-spacing:.02em;text-transform:none;color:var(--faint)}
.expop{width:300px;right:150px}
.seg{display:flex;gap:2px;padding:3px;margin:0 6px 6px;border-radius:9px;background:rgba(255,255,255,.04)}
.seg button{flex:1;height:24px;border-radius:7px;font:11px var(--mono);color:var(--dim)}
.seg button[aria-pressed=true]{background:rgba(255,255,255,.12);color:#fff}
.exlist{margin:0 6px 6px;padding:6px 8px;border-radius:8px;border:1px solid var(--line);display:grid;grid-template-columns:1fr auto;gap:3px 10px;font-size:12px;color:#cfcfcf}
.exlist b{font:500 11px var(--mono);color:#f2f2f2;text-align:right}
.exlist small{grid-column:1/-1;font:10px var(--mono);color:var(--faint)}
.exlist.off{opacity:.45}
.exsize{margin:0 6px 8px;font:10.5px var(--mono);color:var(--dim)}
.exgo{display:flex;align-items:center;justify-content:center;gap:8px;width:calc(100% - 12px);margin:0 6px;height:32px;border-radius:9px;background:#e8e8e8;color:#0a0a0a;font-size:12.5px;font-weight:500}
.exgo kbd{color:#555;border-color:rgba(0,0,0,.2)}
.exnote{margin:6px 6px 2px;font:10px var(--mono);color:var(--faint)}
.exframe{display:none;position:fixed;left:16px;top:62px;right:332px;bottom:16px;border:1px dashed rgba(255,255,255,.28);border-radius:4px;pointer-events:none;z-index:4}
body.exporting .exframe{display:block}
body.exporting .bl,body.exporting .br,body.exporting .nav,body.exporting .hint,body.exporting .tc,body.exporting .watermark,body.exporting .floor,body.exporting .rpdeck,body.exporting .simdeck{display:none!important}
.tblock{position:absolute;right:12px;bottom:12px;display:grid;grid-template-columns:auto auto;gap:2px 14px;padding:8px 10px;border:1px solid rgba(255,255,255,.3);background:rgba(10,10,10,.8);font:10px var(--mono);color:#bdbdbd;letter-spacing:.06em;text-transform:uppercase}
.tblock b{grid-column:1/-1;font-weight:600;font-size:11px;color:#f2f2f2;letter-spacing:.14em}
.tblock span:nth-child(odd){color:var(--faint)}
.netpop{width:330px;right:16px}
.netlog{margin:0 6px 8px;padding:8px;border-radius:8px;border:1px solid var(--line);font-size:12px;color:#cfcfcf}
.netlog small{display:block;margin-top:3px;font:10px var(--mono);color:var(--faint)}
.netlb,.netst{display:flex;flex-direction:column;gap:4px;margin:0 6px 8px}
.netlb div{display:grid;grid-template-columns:62px 1fr auto;gap:8px;align-items:baseline;font-size:12px;color:#cfcfcf}
.netlb span{font:10.5px var(--mono);color:var(--dim)}.netlb em{font:normal 10.5px var(--mono);color:var(--ok)}
.netst div{display:flex;flex-direction:column;font-size:12px;color:#cfcfcf}.netst div span{font:10.5px var(--mono);color:var(--dim)}
.netst small{font:10.5px var(--mono);color:var(--faint);margin-top:2px}
.egress{cursor:pointer}.egress:hover{color:#fff;border-color:var(--line2)}
body.offline .egress{color:#f2f2f2;border-color:rgba(255,255,255,.3)}
body.offline .egress .dot{background:#8a8a8a;box-shadow:none}
body.offline .netlb em{color:var(--ok)}
`;

export const SH_SCRIPT = `
const EX = { fmt: "png", safe: true, saved: null };
const exOpen = () => $("#ex").classList.contains("open");
const FMT = { png: ["Save PNG", "2× · 2880 × 1800 px"], svg: ["Save SVG", "vector · labels as text · fonts outlined"], frames: ["Save frames", "60 PNG frames at 1 s"] };
function exScope() { const m = document.body.dataset.mode; return m === "replay" ? "replay at " + rpClock(RP.t) : m === "simulate" ? "simulated run" : m === "blueprint" ? "blueprint" : "live plant"; }
function exRedactions() {
  const drawn = SCENE.entities.filter((e) => e.kind !== "scaffold" && e.name);
  const named = drawn.filter((e) => ASSETS[e.kind].family !== "fitting").length, orig = EX.saved ? new Map(EX.saved.ents) : null, vOf = (e) => (orig ? orig.get(e) : e.variant);
  return [
    ["Names", named + SCENE.territories.length, "machines and territories become their kind"],
    ["Hostnames", drawn.filter((e) => e.kind === "dock").length, "dock faces: paypal, github, model hosts"],
    ["Product labels", drawn.filter((e) => vOf(e) && e.kind !== "dock").length, "plinth text such as caddy, sqlite"],
    ["Revision", 1, ""],
  ];
}
function exGeneric() {
  const n = {}, out = {};
  for (const e of SCENE.entities) { if (!e.name || e.kind === "scaffold") continue; const l = ASSETS[e.kind].label; n[l] = (n[l] || 0) + 1; out[e.id] = { name: l + " " + n[l], mono: String(Object.keys(out).length + 1) }; }
  return out;
}
function exApply() {
  const on = exOpen() && EX.safe;
  if (on && !EX.saved) {
    const g = exGeneric();
    window.__redacting = true;
    EX.saved = { ents: SCENE.entities.map((e) => [e, e.variant]), terr: SCENE.territories.map((t) => [t, t.label]), chips: [] };
    for (const e of SCENE.entities) if (e.variant) e.variant = e.kind === "dock" ? "EXT" : undefined;
    SCENE.territories.forEach((t, i) => { t.label = "TERRITORY " + (i + 1) + " · " + t.state.toUpperCase(); });
    for (const c of document.querySelectorAll(".chip")) {
      const r = g[c.dataset.id]; if (!r) continue;
      EX.saved.chips.push([c, c.querySelector(".mono").textContent, c.childNodes[1].textContent]);
      c.querySelector(".mono").textContent = r.mono; c.childNodes[1].textContent = r.name;
    }
    dirtyScene = true; animate();
  } else if (!on && EX.saved) {
    for (const [e, v] of EX.saved.ents) e.variant = v;
    for (const [t, l] of EX.saved.terr) t.label = l;
    for (const [c, m, nm] of EX.saved.chips) { c.querySelector(".mono").textContent = m; c.childNodes[1].textContent = nm; }
    EX.saved = null; window.__redacting = false; dirtyScene = true; animate();
  }
  const red = exRedactions(), total = red.reduce((s, r) => s + r[1], 0);
  $("#exList").classList.toggle("off", !EX.safe);
  $("#exList").innerHTML = red.map(([k2, v, d]) => "<span>" + k2 + "</span><b>" + v + "</b>" + (d ? "<small>" + d + "</small>" : "")).join("") + "<small>Not in this view: paths, ports, env and key names, headers, cookies. Numbers are kept.</small>";
  $("#exWhat").textContent = exScope();
  $("#exSize").textContent = FMT[EX.fmt][1];
  $("#exGo").innerHTML = FMT[EX.fmt][0] + (EX.safe ? " · share-safe" : "") + " <kbd>Enter</kbd>";
  $("#exTitle").innerHTML = "<b>" + (EX.safe ? "[project]" : "Acquit") + " · " + exScope() + "</b><span>rev</span><span>" + (EX.safe ? "[rev]" : REV) + "</span><span>drawn</span><span>" + new Date().toISOString().slice(0, 10) + "</span><span>redaction</span><span>" + (EX.safe ? "share-safe · " + total + " redacted" : "none · local only") + "</span>";
}
function exToggle(on = !exOpen()) {
  if (on && frOn()) return toast("Export opens once setup is done.");
  $("#net").classList.remove("open");
  $("#ex").classList.toggle("open", on); document.body.classList.toggle("exporting", on);
  if (on && selected) select(null);
  exApply();
}
function exSave() {
  const name = "acquit-" + document.body.dataset.mode + (EX.safe ? "-share-safe" : "") + "." + (EX.fmt === "frames" ? "frames/" : EX.fmt);
  const total = exRedactions().reduce((s, r) => s + r[1], 0);
  handoff("file", { path: "~/Downloads/" + name, fmt: EX.fmt, safe: EX.safe, redactions: EX.safe ? total : 0 }, "Saved ~/Downloads/" + name + (EX.safe ? " with " + total + " redactions." : ". Not share-safe, keep it local."));
  exToggle(false);
}
$("#exFmt").addEventListener("click", (ev) => { const b = ev.target.closest("[data-fmt]"); if (!b) return; EX.fmt = b.dataset.fmt; for (const x of $("#exFmt").children) x.setAttribute("aria-pressed", x === b); exApply(); });
$("#exSafe").onchange = (ev) => { EX.safe = ev.target.checked; exApply(); };
$("#exGo").onclick = exSave;
$("#exportBtn").onclick = (ev) => { ev.stopPropagation(); exToggle(); };

const NET = { since: "09:12", off: false };
function netToggle(on = !$("#net").classList.contains("open")) {
  if (on) { $("#ex").classList.remove("open"); if (exOpen()) exToggle(false); }
  $("#net").classList.toggle("open", on);
  $("#netSince").textContent = "since " + NET.since;
  $("#netLog").innerHTML = NET.off ? "Offline. Every outbound connection is blocked.<small>The OTLP and MCP listeners keep working on loopback.</small>" : "Nothing has left this machine.<small>Each connection would show here with its destination, bytes and reason.</small>";
}
function setOffline(v) {
  NET.off = v; $("#netOff").checked = v; document.body.classList.toggle("offline", v);
  $(".egress").lastChild.textContent = v ? "offline" : "local · 0 out";
  $(".egress").title = v ? "Offline. All egress blocked." : "Outbound connections since launch";
  if ($("#net").classList.contains("open")) netToggle(true);
}
$(".egress").addEventListener("click", (ev) => { ev.stopPropagation(); netToggle(); });
$("#netOff").onchange = (ev) => { setOffline(ev.target.checked); toast(ev.target.checked ? "Offline. Orrery can't reach anything outside this machine." : "Online. Still nothing is sent unless you turn it on."); };
addEventListener("click", (ev) => {
  if (!ev.target.closest("#net,.egress")) $("#net").classList.remove("open");
  if (exOpen() && !ev.target.closest("#ex,#exportBtn,#stage")) exToggle(false);
});
addEventListener("keydown", (ev) => {
  if (ev.key === "Escape" && !typing(ev)) {
    if (exOpen()) { exToggle(false); ev.stopImmediatePropagation(); return; }
    if ($("#net").classList.contains("open")) { netToggle(false); ev.stopImmediatePropagation(); return; }
  }
  if (exOpen() && ev.key === "Enter" && !typing(ev)) { ev.preventDefault(); ev.stopImmediatePropagation(); exSave(); }
}, true);
function shInit(params) {
  if (params.get("export")) { if (params.get("export") === "raw") { EX.safe = false; $("#exSafe").checked = false; } exToggle(true); }
  if (params.get("net")) netToggle(true);
  if (params.get("offline")) setOffline(true);
}
`;
