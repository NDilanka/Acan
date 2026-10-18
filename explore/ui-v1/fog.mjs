import { FOG } from "./acquit.mjs";

export const FOG_CSS = `
.fogl{position:absolute;left:0;top:0;pointer-events:none;overflow:visible;display:none}
body.fogged .fogl{display:block}
.fogl text{font:600 9px var(--mono);letter-spacing:.08em;fill:#a9a9a9;text-anchor:middle}
.fogl text.w{font-size:8.5px;font-weight:400;letter-spacing:0;fill:#7a7a7a}
body.far .fogl text{display:none}
body.far .chip{font-size:0;gap:0;padding:2px}
body.far .chip .rd,body.far .chip::after{display:none!important}
body.far .chip::before{height:10px}
`;

export const FOG_SCRIPT = `
const FOGD = ${JSON.stringify(FOG)};
const fogScore = (id) => { const s = Object.values(FOGD[id].s); return 1 - s.reduce((a, b) => a + b, 0) / s.length; };
const fogMissing = (id) => Object.entries(FOGD[id].s).filter(([, v]) => v < 1).map(([k2, v]) => (v ? "half " : "not ") + k2);
function fogAfterRender() {
  let svg = $("#fogl");
  if (!svg) {
    world.insertAdjacentHTML("afterbegin", '<svg class="fogl" id="fogl" width="' + W + '" height="' + H + '"><defs><radialGradient id="fogG"><stop offset="0" stop-color="#9a9a9a" stop-opacity=".55"/><stop offset=".6" stop-color="#8a8a8a" stop-opacity=".28"/><stop offset="1" stop-color="#8a8a8a" stop-opacity="0"/></radialGradient><pattern id="fogDots" width="3" height="3" patternUnits="userSpaceOnUse"><circle cx="1.5" cy="1.5" r=".45" fill="#c8c8c8"/></pattern><filter id="fogEdge" x="-30%" y="-30%" width="160%" height="160%"><feTurbulence type="fractalNoise" baseFrequency=".035" numOctaves="2" seed="7"/><feDisplacementMap in="SourceGraphic" scale="14"/></filter></defs><g id="foglg"></g></svg>');
    svg = $("#fogl");
  }
  if (!document.body.classList.contains("fogged")) return;
  const hidden = new Set(($("#rpHide").textContent + $("#frHide").textContent).match(/data-id="([^"]+)"/g)?.map((m) => m.slice(9, -1)) || []);
  let h = "";
  for (const id of Object.keys(FOGD)) {
    const e = ENT[id]; if (!e || hidden.has(id) || e.state === "ghost") continue;
    const sc = fogScore(id); if (sc < 0.05) continue;
    const a = ASSETS[e.kind], s = e.size || 1, hw = (a.w * s) / 2 + 8, hd = (a.d * s) / 2 + 8, pts = [];
    for (const sx of [-1, 1]) for (const sy of [-1, 1]) for (const z of [0, a.top * s]) pts.push(k.iso([e.x + sx * hw, e.y + sy * hd, z], P));
    const xs = pts.map((p) => p[0]), ys = pts.map((p) => p[1]), cx = (Math.min(...xs) + Math.max(...xs)) / 2, cy = (Math.min(...ys) + Math.max(...ys)) / 2;
    const rx = (Math.max(...xs) - Math.min(...xs)) / 2 * 1.05, ry = (Math.max(...ys) - Math.min(...ys)) / 2 * 1.05, op = Math.min(0.95, 0.3 + sc * 1.3);
    h += '<g opacity="' + op.toFixed(2) + '" filter="url(#fogEdge)"><ellipse cx="' + cx.toFixed(1) + '" cy="' + cy.toFixed(1) + '" rx="' + rx.toFixed(1) + '" ry="' + ry.toFixed(1) + '" fill="url(#fogG)"/><ellipse cx="' + cx.toFixed(1) + '" cy="' + cy.toFixed(1) + '" rx="' + (rx * 0.8).toFixed(1) + '" ry="' + (ry * 0.8).toFixed(1) + '" fill="url(#fogDots)" opacity="' + (0.15 + sc * 0.5).toFixed(2) + '"/></g>';
    if (sc < 0.2) continue;
    const ty = Math.max(...ys) + 10;
    h += '<text x="' + cx.toFixed(1) + '" y="' + ty.toFixed(1) + '">FOG ' + sc.toFixed(2) + '</text><text class="w" x="' + cx.toFixed(1) + '" y="' + (ty + 10).toFixed(1) + '">' + FOGD[id].why + "</text>";
  }
  $("#foglg").innerHTML = h;
}
function fogStrip(id) {
  const f = FOGD[id]; if (!f) return "";
  return '<div class="sec"><h5>Fog<i>' + fogScore(id).toFixed(2) + (f.why ? " · " + f.why : "") + '</i></h5><div class="fog">' + Object.entries(f.s).map(([n, v]) => '<div data-v="' + v + '"><i></i>' + n + "</div>").join("") + "</div></div>";
}
function fogToggle(on = !document.body.classList.contains("fogged")) {
  document.body.classList.toggle("fogged", on);
  const i = document.querySelector('[data-layer="fogged"]'); if (i) i.checked = on;
  fogAfterRender();
  if (on) { const worst = Object.keys(FOGD).filter((id) => ENT[id]).sort((a, b) => fogScore(b) - fogScore(a)).slice(0, 3); toast("Fog on. Thickest: " + worst.map((id) => ENT[id].name + " " + fogScore(id).toFixed(2)).join(", ") + "."); }
}
`;
