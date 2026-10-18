// Machine check of the ui-v2 quality bar, go-bar lines G3 and G4 in READINESS.md.
// drive.mjs runs `audit` at every shot when AUDIT=1, `auditFocus` on a {"audit":"focus"} step,
// and FOCUS_MARK / FOCUS_CHECK around every scripted wait.
// A finding is { rule, selector, text, detail }. An unknown is a check the audit could not decide.
// Unknowns are reported next to findings and are never counted as passes.

import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { inflateSync } from "node:zlib";

export const RULES = ["type-floor", "overlap", "offscreen", "contrast", "focus-ring", "transition", "egress-sum", "focus-steal"];

// Camera easing runs in JS (build.mjs animate(), blueprint.mjs inkLoop), so no CSS camera selector needs allowing.
// The ink-in moment is the one CSS animation the bar allows: .orders-ink in orders.mjs and .bench-ink in bench.mjs.
export const MOTION_ALLOW = { selectors: [".orders-ink", ".bench-ink"], animations: ["orders-ink", "bench-ink"] };

// Pills show the running byte total; logs list the ops. `section` logs read the rows after the
// "Network log" header up to the next header. `total` logs carry the total in a header and may show
// only the last few rows, so their rows are summed only when the header's op count matches.
export const EGRESS = [
  { pill: ".egress", root: "#orders-kv-panel", section: ".orders-section", row: ".orders-kv-row[data-state]", bytes: "i" },
  { pill: ".egress", root: "#orders-kv-dock", total: "header b", count: "header span", row: ".orders-kv-row", bytes: "i" },
  { pill: ".egress", root: "#orders-kv-card", total: "header b", count: "header span", row: ".orders-kv-row", bytes: "i" },
  { pill: ".egress", root: "#clashKvPanel", section: ".clash-kv-section", row: ".clash-kv-row[data-state]", bytes: "i" },
  { pill: ".egress", root: "#clashKv", total: "header b", count: "header span", row: ".clash-kv-row", bytes: "i" },
  { pill: ".egress", root: "#gateKv", total: "header b", count: "header span", row: ".gate-kvlog-row", bytes: "i" },
  { pill: ".egress", root: ".bench-kvarm", total: "h3 small", row: ".bench-op", bytes: "span" },
  { pill: "#worldEgress", root: "#worldNetworkBody", section: "h3", row: ".world-network-row", bytes: "b > span" },
];

const CONFIG = {
  floorPx: 11,
  overlapPx: 2,
  labelMax: [480, 200],
  surfaces: "[role=dialog],dialog[open],[aria-modal=true],.pop.open,#insp.open,#toast.on,[class*=card],[class*=panel],[class*=plate],[class*=popover],[class*=sheet],[id*=card],[id*=Card],[id*=panel],[id*=Panel],[id*=plate]",
  motion: MOTION_ALLOW,
  egress: EGRESS,
};

// Runs in the page. Everything it needs is inside it, because it is sent as source text.
function pageAudit(cfg) {
  const W = innerWidth, H = innerHeight;
  const findings = [], unknown = [], stats = {};
  const t0 = performance.now();
  const csMemo = new Map();
  const cs = (e) => { let s = csMemo.get(e); if (!s) csMemo.set(e, (s = getComputedStyle(e))); return s; };
  const opMemo = new Map();
  const opacity = (e) => {
    if (!e || e.nodeType !== 1) return 1;
    let v = opMemo.get(e);
    if (v === undefined) opMemo.set(e, (v = +cs(e).opacity * opacity(e.parentElement)));
    return v;
  };
  const anchorOf = (e) => { for (let n = e.parentElement; n && n !== document.body; n = n.parentElement) if (n.id) return n; return null; };
  const part = (n) => {
    if (n.id) return "#" + CSS.escape(n.id);
    let s = n.localName + [...n.classList].slice(0, 2).map((c) => "." + CSS.escape(c)).join("");
    for (const a of ["data-id", "data-key", "data-action", "data-machine", "data-mode", "data-orders-kv", "data-clash"]) if (n.hasAttribute(a)) { s += "[" + a + '="' + n.getAttribute(a) + '"]'; break; }
    return s;
  };
  const sel = (e) => {
    if (!e || e.nodeType !== 1) return String(e);
    if (e.id) return part(e);
    const chain = [part(e)];
    for (let n = e.parentElement, i = 0; n && i < 2 && !n.id && n !== document.body; n = n.parentElement, i++) chain.unshift(part(n));
    const a = anchorOf(e);
    return (a && !chain[0].startsWith("#") ? part(a) + " " : "") + chain.join(" > ");
  };
  const squash = (s) => String(s || "").replace(/\s+/g, " ").trim().slice(0, 70);
  const ctx = new OffscreenCanvas(1, 1).getContext("2d", { willReadFrequently: true });
  const colorMemo = new Map();
  const rgba = (str) => {
    let v = colorMemo.get(str);
    if (v) return v;
    ctx.clearRect(0, 0, 1, 1); ctx.fillStyle = "#010203"; ctx.fillStyle = str;
    const ok = ctx.fillStyle !== "#010203" || /^#010203$/i.test(str.trim());
    ctx.fillRect(0, 0, 1, 1);
    const d = ctx.getImageData(0, 0, 1, 1).data;
    v = ok ? [d[0], d[1], d[2], d[3] / 255] : null;
    colorMemo.set(str, v);
    return v;
  };
  const rendered = (e) => e.getClientRects().length > 0;
  const shown = (e) => rendered(e) && cs(e).visibility === "visible" && opacity(e) > 0.05;
  const inView = (r) => r.right > 0 && r.bottom > 0 && r.left < W && r.top < H && r.width > 0 && r.height > 0;
  const SVG_GEOM = new Set(["path", "rect", "circle", "ellipse", "polygon", "polyline", "line", "image", "use"]);
  const occMemo = new Map();
  const occludes = (e) => {
    let v = occMemo.get(e);
    if (v !== undefined) return v;
    const s = cs(e);
    if (e instanceof SVGElement) v = SVG_GEOM.has(e.localName) && s.fill !== "none" && opacity(e) * +s.fillOpacity > 0.3;
    else {
      const bg = rgba(s.backgroundColor);
      v = /^(img|video|canvas|iframe)$/.test(e.localName) || s.backgroundImage !== "none" || s.backdropFilter !== "none" || (!!bg && bg[3] * opacity(e) >= 0.35);
    }
    occMemo.set(e, v);
    return v;
  };
  const stack = (x, y) => document.elementsFromPoint(x, y);
  const visibleAt = (p, x, y) => {
    for (const e of stack(x, y)) {
      if (e === p || p.contains(e) || e.contains(p)) return true;
      if (occludes(e)) return false;
    }
    return false;
  };

  const SKIP = new Set(["script", "style", "noscript", "template", "title", "desc", "option", "textarea", "metadata"]);
  const groups = new Map();
  const tw = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, { acceptNode: (n) => (/\S/.test(n.data) ? 1 : 3) });
  for (let n = tw.nextNode(); n; n = tw.nextNode()) {
    const p = n.parentElement;
    if (!p || SKIP.has(p.localName) || p.closest("defs,symbol,clipPath,mask,pattern,marker")) continue;
    let g = groups.get(p);
    if (!g) groups.set(p, (g = []));
    g.push(n);
  }
  for (const e of document.querySelectorAll("input,textarea,select")) {
    if (/^(hidden|checkbox|radio|range|color|file|submit|button|reset|image)$/.test(e.type)) continue;
    if (!groups.has(e)) groups.set(e, null);
  }

  const style = document.createElement("style");
  style.textContent = "*,*::before,*::after{pointer-events:auto!important}";
  const items = [];
  document.head.append(style);
  try {
    for (const [p, nodes] of groups) {
      const s = cs(p);
      if (s.visibility !== "visible" || opacity(p) <= 0.05) continue;
      const svg = p instanceof SVGElement;
      let rects = [], text;
      if (nodes) {
        const range = document.createRange();
        for (const n of nodes) { range.selectNodeContents(n); for (const r of range.getClientRects()) if (r.width > 0.5 && r.height > 0.5) rects.push(r); }
        text = squash(nodes.map((n) => n.data).join(" "));
      } else {
        const r = p.getBoundingClientRect(), pl = parseFloat(s.paddingLeft) || 0, pr = parseFloat(s.paddingRight) || 0;
        if (r.width > 0 && r.height > 0) rects.push(new DOMRect(r.left + pl, r.top, Math.max(1, r.width - pl - pr), r.height));
        text = squash(p.value || p.placeholder || (p.selectedOptions && p.selectedOptions[0] && p.selectedOptions[0].textContent));
        if (!text) continue;
      }
      rects = rects.filter(inView);
      if (!rects.length) continue;
      const seen = [];
      for (const r of rects.slice(0, 4)) {
        const y = r.top + r.height / 2;
        const pts = (r.width > 24 ? [0.15, 0.5, 0.85] : [0.5]).map((f) => [r.left + r.width * f, y]).filter(([x]) => x >= 0 && x < W && y >= 0 && y < H);
        if (pts.some(([x, y]) => visibleAt(p, x, y))) seen.push(r);
      }
      if (!seen.length) continue;
      let fontPx = parseFloat(s.fontSize);
      if (svg) {
        const m = p.getScreenCTM();
        if (m) fontPx *= Math.sqrt(Math.abs(m.a * m.d - m.b * m.c));
      } else {
        // Ancestor transforms, not rect/offsetHeight: offsetHeight is rounded to whole pixels.
        for (let n = p; n && n.nodeType === 1; n = n.parentElement) {
          const c = cs(n);
          if (c.transform !== "none") { const m = new DOMMatrix(c.transform); fontPx *= Math.sqrt(Math.abs(m.a * m.d - m.b * m.c)); }
          if (c.scale && c.scale !== "none") { const [x, y = x] = c.scale.split(" ").map(Number); fontPx *= Math.sqrt(Math.abs(x * y)); }
          if (c.zoom && +c.zoom !== 1) fontPx *= +c.zoom;
        }
      }
      items.push({ p, svg, text, rects, seen, fontPx, weight: +s.fontWeight || 400, aria: !!p.closest("[aria-hidden=true]"), placeholder: !nodes && !p.value });
    }
    stats.textItems = items.length;

    for (const it of items) {
      if (it.aria) continue;
      if (it.fontPx < cfg.floorPx - 0.05) findings.push({ rule: "type-floor", selector: sel(it.p), text: it.text, detail: it.fontPx.toFixed(1) + " px" });
    }

    // A label is the box it paints: a small positioned chip, pin or tag with a background or border.
    // Text with no painted box, and SVG text on the iso plant, is measured by its glyph cells,
    // because a skewed text's bounding box covers far more than its letters.
    const paints = (e) => { const c = cs(e), bg = rgba(c.backgroundColor); return (!!bg && bg[3] > 0.1) || c.backgroundImage !== "none" || c.backdropFilter !== "none" || ["Top", "Right", "Bottom", "Left"].some((k) => parseFloat(c["border" + k + "Width"]) > 0 && c["border" + k + "Style"] !== "none"); };
    const glyphs = (t) => {
      const m = t.getScreenCTM(), n = Math.min(t.getNumberOfChars(), 120), out = [];
      if (!m || (Math.abs(m.b) < 0.01 && Math.abs(m.c) < 0.01) || !n) return [t.getBoundingClientRect()];
      for (let i = 0; i < n; i++) {
        let e; try { e = t.getExtentOfChar(i); } catch { continue; }
        const q = [[e.x, e.y], [e.x + e.width, e.y], [e.x, e.y + e.height], [e.x + e.width, e.y + e.height]].map(([x, y]) => [m.a * x + m.c * y + m.e, m.b * x + m.d * y + m.f]);
        const xs = q.map((p) => p[0]), ys = q.map((p) => p[1]);
        out.push(new DOMRect(Math.min(...xs), Math.min(...ys), Math.max(...xs) - Math.min(...xs), Math.max(...ys) - Math.min(...ys)));
      }
      return out;
    };
    const units = new Map();
    for (const it of items) {
      let root = null, rects = null;
      if (it.svg) { root = it.p.closest("text") || it.p; if (units.has(root)) continue; rects = glyphs(root); }
      else {
        for (let a = it.p; a && a !== document.body; a = a.parentElement) {
          const pos = cs(a).position;
          if (pos === "absolute" || pos === "fixed") {
            const r = a.getBoundingClientRect();
            if (r.width <= cfg.labelMax[0] && r.height <= cfg.labelMax[1]) { root = a; rects = paints(a) ? [r] : null; }
            break;
          }
        }
        if (!root) root = it.p;
        if (!rects) { const u = units.get(root); if (u) { if (!u.boxed) u.rects.push(...it.rects); continue; } rects = [...it.rects]; }
        else if (units.has(root)) continue;
      }
      units.set(root, { root, rects, boxed: rects.length === 1 && !it.svg && root !== it.p, text: it.text, stack: root.closest("[data-audit-stack]") });
    }
    for (const u of units.values()) {
      u.rects = u.rects.filter(inView);
      if (!u.rects.length) continue;
      const l = Math.min(...u.rects.map((r) => r.left)), t = Math.min(...u.rects.map((r) => r.top));
      u.rect = new DOMRect(l, t, Math.max(...u.rects.map((r) => r.right)) - l, Math.max(...u.rects.map((r) => r.bottom)) - t);
    }
    const list = [...units.values()].filter((u) => u.rect).sort((a, b) => a.rect.left - b.rect.left);
    stats.labelUnits = list.length;
    stats.stackOptOuts = document.querySelectorAll("[data-audit-stack]").length;
    stats.stackSkips = 0;
    const seen = new Set();
    for (let i = 0; i < list.length; i++) {
      const A = list[i];
      for (let j = i + 1; j < list.length; j++) {
        const B = list[j];
        if (B.rect.left >= A.rect.right - cfg.overlapPx) break;
        let hit = null;
        for (const ra of A.rects) for (const rb of B.rects) {
          const l = Math.max(ra.left, rb.left, 0), r = Math.min(ra.right, rb.right, W), t = Math.max(ra.top, rb.top, 0), b = Math.min(ra.bottom, rb.bottom, H);
          if (r - l > cfg.overlapPx && b - t > cfg.overlapPx && (!hit || (r - l) * (b - t) > (hit.r - hit.l) * (hit.b - hit.t))) hit = { l, r, t, b };
        }
        if (!hit) continue;
        const { l, r, t, b } = hit;
        if (A.root.contains(B.root) || B.root.contains(A.root)) continue;
        if (A.stack && A.stack === B.stack) { stats.stackSkips++; continue; }
        let both = false;
        for (const [fx, fy] of [[0.5, 0.5], [0.25, 0.25], [0.75, 0.25], [0.25, 0.75], [0.75, 0.75]]) {
          let a = false, c = false;
          for (const e of stack(l + (r - l) * fx, t + (b - t) * fy)) {
            const inA = A.root.contains(e), inB = B.root.contains(e);
            a ||= inA; c ||= inB;
            if (a && c) break;
            if (!inA && !inB && occludes(e)) break;
          }
          if (a && c) { both = true; break; }
        }
        if (!both) continue;
        const key = sel(A.root) + "|" + sel(B.root);
        if (seen.has(key)) continue;
        seen.add(key);
        findings.push({ rule: "overlap", selector: sel(A.root), text: A.text, detail: "overlaps " + sel(B.root) + ' "' + B.text + '" by ' + Math.round(r - l) + "x" + Math.round(b - t) + " px" });
      }
    }

    const pillsShown = new Map();
    for (const ps of new Set(cfg.egress.map((g) => g.pill))) {
      const el = document.querySelector(ps);
      if (!el || !shown(el)) continue;
      const r = el.getBoundingClientRect();
      if (!inView(r) || !visibleAt(el, r.left + r.width / 2, r.top + r.height / 2)) continue;
      const m = el.textContent.match(/([\d,]+) B\b/);
      pillsShown.set(ps, { el, bytes: m ? +m[1].replace(/,/g, "") : null, text: squash(el.textContent) });
    }
    stats.egressLogs = 0;
    const bytesOf = (s) => { const m = String(s || "").match(/([\d,]+) B\b/); return m ? +m[1].replace(/,/g, "") : null; };
    for (const g of cfg.egress) {
      const pill = pillsShown.get(g.pill);
      for (const root of document.querySelectorAll(g.root)) {
        if (!shown(root) || !inView(root.getBoundingClientRect())) continue;
        let total = null, rows = [], complete = true, label;
        if (g.section) {
          const head = [...root.querySelectorAll(g.section)].find((h) => /^\s*Network log/i.test(h.textContent));
          if (!head) continue;
          total = bytesOf(head.textContent); label = squash(head.textContent);
          for (let n = head.nextElementSibling; n && !n.matches(g.section); n = n.nextElementSibling) if (n.matches(g.row)) rows.push(n);
        } else {
          const t = root.querySelector(g.total);
          total = t ? bytesOf(t.textContent) : null; label = t ? squash(t.textContent) : "";
          rows = [...root.querySelectorAll(g.row)];
          if (g.count) { const c = root.querySelector(g.count), m = c && c.textContent.match(/(\d+) ops?\b/); complete = !!m && +m[1] === rows.length; }
        }
        stats.egressLogs++;
        const vals = rows.map((n) => bytesOf(n.querySelector(g.bytes) && n.querySelector(g.bytes).textContent));
        const where = sel(root);
        if (pill && pill.bytes !== null && total !== null && pill.bytes !== total)
          findings.push({ rule: "egress-sum", selector: g.pill, text: pill.text, detail: "pill " + pill.bytes + " B, " + where + ' header "' + label + '" says ' + total + " B" });
        if (!complete) continue;
        if (vals.some((v) => v === null)) {
          unknown.push({ rule: "egress-sum", selector: where, text: label, detail: rows.length + " rows, " + vals.filter((v) => v === null).length + " show no byte count (sending, held or a result)" });
          continue;
        }
        const sum = vals.reduce((a, b) => a + b, 0);
        if (total !== null && sum !== total) findings.push({ rule: "egress-sum", selector: where, text: label, detail: "rows sum to " + sum + " B, header says " + total + " B" });
        if (pill && pill.bytes !== null && sum !== pill.bytes) findings.push({ rule: "egress-sum", selector: g.pill, text: pill.text, detail: "pill " + pill.bytes + " B, " + where + " rows sum to " + sum + " B" });
      }
    }
  } finally { style.remove(); }

  const surf = [...document.querySelectorAll(cfg.surfaces)].filter((e) => {
    if (!shown(e)) return false;
    const pos = cs(e).position;
    if (pos !== "absolute" && pos !== "fixed" && e.getAttribute("role") !== "dialog" && e.localName !== "dialog") return false;
    const r = e.getBoundingClientRect();
    return r.width > 0 && r.height > 0 && !(r.width > W * 1.5 && r.height > H * 1.5);
  });
  const surfSet = new Set(surf);
  const tops = surf.filter((e) => { for (let n = e.parentElement; n; n = n.parentElement) if (surfSet.has(n)) return false; return true; });
  stats.surfaces = tops.length;
  for (const e of tops) {
    const r = e.getBoundingClientRect(), out = [];
    if (r.left < -1) out.push("left " + Math.round(-r.left));
    if (r.top < -1) out.push("top " + Math.round(-r.top));
    if (r.right > W + 1) out.push("right " + Math.round(r.right - W));
    if (r.bottom > H + 1) out.push("bottom " + Math.round(r.bottom - H));
    if (out.length) findings.push({ rule: "offscreen", selector: sel(e), text: squash(e.textContent), detail: "clipped by the viewport, " + out.join(", ") + " px" + (inView(r) ? "" : ", fully outside") });
  }

  const allowSel = cfg.motion.selectors.join(",");
  const motion = new Map();
  const sig = (e) => e.localName + [...e.classList].slice(0, 2).map((c) => "." + c).join("") + (e.id ? "#" + e.id : "");
  let scanned = 0;
  const seenMotion = (e) => shown(e) && inView(e.getBoundingClientRect());
  for (const e of document.querySelectorAll("*")) {
    if (!rendered(e)) continue;
    scanned++;
    const s = cs(e);
    if (s.transitionProperty === "none" || !s.transitionDuration.split(",").some((v) => parseFloat(v) > 0)) continue;
    if (e.matches(allowSel) || !seenMotion(e)) continue;
    const k = "transition " + sig(e);
    const m = motion.get(k) || { rule: "transition", selector: sel(e), text: squash(e.textContent).slice(0, 40), detail: "transition: " + s.transitionProperty + " " + s.transitionDuration, n: 0 };
    m.n++; motion.set(k, m);
  }
  stats.motionScanned = scanned;
  for (const a of document.getAnimations()) {
    if (a.playState !== "running") continue;
    const e = a.effect && a.effect.target;
    if (!e || !seenMotion(e)) continue;
    if (a.animationName && cfg.motion.animations.includes(a.animationName)) continue;
    if (e.matches(allowSel)) continue;
    const what = a.animationName ? "animation " + a.animationName : a.transitionProperty ? "running transition " + a.transitionProperty : "script animation";
    const k = what + " " + sig(e);
    const m = motion.get(k) || { rule: "transition", selector: sel(e) + (a.effect.pseudoElement || ""), text: squash(e.textContent).slice(0, 40), detail: what, n: 0 };
    m.n++; motion.set(k, m);
  }
  for (const m of motion.values()) { m.detail += m.n > 1 ? " · " + m.n + " elements" : ""; delete m.n; findings.push(m); }

  const contrast = [], moving = [];
  stats.contrastExempt = 0;
  for (const it of items) {
    if (it.aria || it.p.closest(":disabled,[aria-disabled=true]")) { stats.contrastExempt++; continue; }
    const s = cs(it.p);
    const clip = (() => { for (let n = it.p; n; n = n.parentElement) { const c = cs(n); if (/text/.test(c.backgroundClip + " " + (c.webkitBackgroundClip || ""))) return true; } return false; })();
    const raw = it.svg ? s.fill : it.placeholder ? getComputedStyle(it.p, "::placeholder").color : s.webkitTextFillColor || s.color;
    const fg = /^url\(/.test(raw) || clip ? null : rgba(raw);
    const large = it.fontPx >= 24 || (it.fontPx >= 18.66 && it.weight >= 700);
    const base = { selector: sel(it.p), text: it.text };
    if (!fg) { unknown.push({ rule: "contrast", ...base, detail: "text paint is " + (clip ? "background-clip: text" : raw) }); continue; }
    const alpha = fg[3] * opacity(it.p) * (it.svg ? +s.fillOpacity : 1);
    const box = (r) => [Math.max(0, r.left), Math.max(0, r.top), Math.min(W, r.right), Math.min(H, r.bottom)];
    contrast.push({ ...base, fg, alpha, need: large ? 3 : 4.5, large, rects: it.seen.map(box) });
    moving.push([it.p, it.p.getBoundingClientRect()]);
  }
  window.__orreryAuditMoving = moving;
  stats.contrastItems = contrast.length;
  stats.collectMs = Math.round(performance.now() - t0);
  return { findings, unknown, stats, contrast, view: [W, H] };
}

// Decodes the 8-bit RGB or RGBA PNG that Page.captureScreenshot returns.
function decodePng(buf) {
  let i = 8, w = 0, h = 0, type = 0;
  const idat = [];
  while (i < buf.length) {
    const len = buf.readUInt32BE(i), kind = buf.toString("ascii", i + 4, i + 8), body = buf.subarray(i + 8, i + 8 + len);
    if (kind === "IHDR") { w = body.readUInt32BE(0); h = body.readUInt32BE(4); type = body[9]; if (body[8] !== 8 || (type !== 2 && type !== 6)) throw new Error("unsupported png"); }
    else if (kind === "IDAT") idat.push(body);
    else if (kind === "IEND") break;
    i += 12 + len;
  }
  const bpp = type === 6 ? 4 : 3, stride = w * bpp, raw = inflateSync(Buffer.concat(idat)), px = Buffer.alloc(stride * h);
  for (let y = 0; y < h; y++) {
    const f = raw[y * (stride + 1)], s = y * (stride + 1) + 1, o = y * stride, u = o - stride;
    if (f === 0) raw.copy(px, o, s, s + stride);
    else if (f === 1) for (let x = 0; x < stride; x++) px[o + x] = raw[s + x] + (x >= bpp ? px[o + x - bpp] : 0);
    else if (f === 2) for (let x = 0; x < stride; x++) px[o + x] = raw[s + x] + (y ? px[u + x] : 0);
    else if (f === 3) for (let x = 0; x < stride; x++) px[o + x] = raw[s + x] + (((x >= bpp ? px[o + x - bpp] : 0) + (y ? px[u + x] : 0)) >> 1);
    else for (let x = 0; x < stride; x++) {
      const a = x >= bpp ? px[o + x - bpp] : 0, b = y ? px[u + x] : 0, c = x >= bpp && y ? px[u + x - bpp] : 0;
      const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
      px[o + x] = raw[s + x] + (pa <= pb && pa <= pc ? a : pb <= pc ? b : c);
    }
  }
  return { w, h, bpp, px };
}
const lum = ([r, g, b]) => { const f = (c) => { c /= 255; return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4); }; return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b); };
const ratio = (a, b) => { const x = lum(a), y = lum(b); return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05); };
const over = (fg, a, bg) => [0, 1, 2].map((i) => fg[i] * a + bg[i] * (1 - a));

// The background under a line of text is the most common colour in its box once glyph pixels,
// which sit near the text colour, are set aside. Sampling the shot itself needs no second
// screenshot and no hidden-text repaint, so the audit does not stretch the page's clocks.
function background(img, [l, t, r, b], fg, sx, sy) {
  const buckets = new Map();
  const x0 = Math.floor(l * sx), x1 = Math.ceil(r * sx), y0 = Math.floor(t * sy), y1 = Math.ceil(b * sy);
  const step = Math.max(1, Math.round(Math.sqrt(((x1 - x0) * (y1 - y0)) / 4000)));
  for (let y = y0; y < Math.min(y1, img.h); y += step) for (let x = x0; x < Math.min(x1, img.w); x += step) {
    const k = (y * img.w + x) * img.bpp, c = [img.px[k], img.px[k + 1], img.px[k + 2]];
    if (Math.abs(c[0] - fg[0]) + Math.abs(c[1] - fg[1]) + Math.abs(c[2] - fg[2]) < 24) continue;
    const key = (c[0] >> 3) * 1024 + (c[1] >> 3) * 32 + (c[2] >> 3);
    const e = buckets.get(key) || { n: 0, s: [0, 0, 0] };
    e.n++; e.s[0] += c[0]; e.s[1] += c[1]; e.s[2] += c[2];
    buckets.set(key, e);
  }
  let best = null;
  for (const e of buckets.values()) if (!best || e.n > best.n) best = e;
  return best && best.s.map((v) => v / best.n);
}


// Run just before the screenshot, so the audit can tell which text held still across the shot.
// An element counts as unchanged when one with the same tag, class, own text and box was there at
// the shot, so a re-render that swaps in an identical node keeps its contrast check.
const SHOT_SIG = `const shotSig = (e, r) => { let t = ""; for (const n of e.childNodes) if (n.nodeType === 3) t += n.data; const h = (v) => Math.round(v * 2) / 2; return e.localName + "|" + e.getAttribute("class") + "|" + t.trim().slice(0, 80) + "|" + h(r.left) + "," + h(r.top) + "," + h(r.width) + "," + h(r.height); };`;
export const SHOT_MARK = `(() => { ${SHOT_SIG} const m = new Set(); for (const e of document.querySelectorAll("body *")) m.add(shotSig(e, e.getBoundingClientRect())); window.__orreryShotRects = m; return true; })()`;

const CONTRAST_MOVED = `new Promise((ok) => {
  ${SHOT_SIG}
  const m = window.__orreryAuditMoving || [], shot = window.__orreryShotRects;
  delete window.__orreryAuditMoving; delete window.__orreryShotRects;
  const diff = (a, b) => Math.max(Math.abs(a.left - b.left), Math.abs(a.top - b.top), Math.abs(a.width - b.width), Math.abs(a.height - b.height));
  const say = (d, when) => d > 0.5 ? "text moved " + Math.round(d * 10) / 10 + " px " + when : "";
  const done = () => ok(m.map(([e, a]) => {
    if (shot && !shot.has(shotSig(e, a))) return "text moved or changed between the screenshot and the audit";
    return say(diff(a, e.getBoundingClientRect()), "between frames");
  }));
  requestAnimationFrame(() => requestAnimationFrame(done));
  setTimeout(done, 120);
})`;

export async function audit({ send, evaluate }, shotBase64) {
  const r = await evaluate("(" + pageAudit.toString() + ")(" + JSON.stringify(CONFIG) + ")");
  const { contrast, view } = r;
  delete r.contrast; delete r.view;
  if (!contrast.length) return r;
  const t = Date.now();
  let img = null;
  try { img = decodePng(Buffer.from(shotBase64 || (await send("Page.captureScreenshot", { format: "png" })).data, "base64")); }
  catch (e) { for (const c of contrast) r.unknown.push({ rule: "contrast", selector: c.selector, text: c.text, detail: "screenshot not readable: " + e.message }); return r; }
  const sx = img.w / view[0], sy = img.h / view[1];
  // The shot was taken before the geometry was read. Text that moved since SHOT_MARK, or is still
  // moving (a camera ease, a walking chip), may sit somewhere else in the shot, so its sample
  // would be of whatever was there instead.
  const moved = await evaluate(CONTRAST_MOVED);
  for (const [i, c] of contrast.entries()) {
    if (moved[i]) { r.unknown.push({ rule: "contrast", selector: c.selector, text: c.text, detail: moved[i] }); continue; }
    const fgRgb = c.fg.slice(0, 3);
    let worst = null;
    for (const box of c.rects) {
      const bg = background(img, box, fgRgb, sx, sy);
      if (!bg) continue;
      const shown = over(fgRgb, c.alpha, bg), q = ratio(shown, bg);
      if (!worst || q < worst.q) worst = { q, bg, shown };
    }
    if (!worst) { r.unknown.push({ rule: "contrast", selector: c.selector, text: c.text, detail: "no background pixels apart from the text colour" }); continue; }
    if (worst.q < c.need) r.findings.push({ rule: "contrast", selector: c.selector, text: c.text, detail: worst.q.toFixed(2) + ":1, needs " + c.need + ":1 · text rgb(" + worst.shown.map(Math.round).join(",") + ") on rgb(" + worst.bg.map(Math.round).join(",") + ")" + (c.large ? " · large" : "") });
  }
  r.stats.sampleMs = Date.now() - t;
  return r;
}
function pageFocusList(cap) {
  const W = innerWidth, H = innerHeight;
  const opacity = (e) => { let v = 1; for (let n = e; n && n.nodeType === 1; n = n.parentElement) v *= +getComputedStyle(n).opacity; return v; };
  const all = [...document.querySelectorAll("a[href],button,input,select,textarea,summary,[tabindex],[contenteditable=true]")].filter((e) => {
    if (e.disabled || e.tabIndex < 0 || e.closest("[inert],[hidden]") || (e.type === "hidden")) return false;
    const face = e.closest("label") || e;
    const r = face.getBoundingClientRect();
    if (!face.getClientRects().length || getComputedStyle(face).visibility !== "visible" || opacity(face) <= 0.05) return false;
    return r.right > 0 && r.bottom > 0 && r.left < W && r.top < H;
  });
  window.__orreryFocusTotal = all.length;
  window.__orreryFocusBefore = document.activeElement;
  return all.slice(0, cap);
}

function pageFocusLook() {
  const look = (e, pseudo) => { if (!e) return ""; const s = getComputedStyle(e, pseudo); return [s.outlineStyle, s.outlineWidth, s.outlineColor, s.outlineOffset, s.boxShadow, s.borderTopColor, s.borderRightColor, s.borderBottomColor, s.borderLeftColor, s.borderTopWidth, s.borderBottomWidth].join("|"); };
  const ring = (e) => { const s = getComputedStyle(e); return s.outlineStyle !== "none" && parseFloat(s.outlineWidth) > 0 ? "outline " + s.outlineWidth + " " + s.outlineStyle : ""; };
  const sib = this.nextElementSibling, par = this.parentElement, gp = par && par.parentElement;
  return { parts: [look(this), look(this, "::before"), look(this, "::after"), look(sib), look(par), look(gp)], ring: ring(this) };
}

const focusDescribe = "function(){const p=(n)=>n.id?'#'+n.id:n.localName+[...n.classList].slice(0,2).map(c=>'.'+c).join('');let a=this.parentElement;while(a&&!a.id&&a!==document.body)a=a.parentElement;return{selector:(a&&a.id&&!this.id?p(a)+' ':'')+p(this),text:(this.getAttribute('aria-label')||this.textContent||this.value||'').replace(/\\s+/g,' ').trim().slice(0,60)}}";

// Forces :focus and :focus-visible through CDP instead of pressing Tab, because Tab and focus
// handlers are bound by ui-v1 and the concepts; real focus would change selection mid-flow.
export async function auditFocus({ send, evaluate }, { cap = 80 } = {}) {
  const findings = [], unknown = [];
  await send("DOM.enable"); await send("CSS.enable");
  await send("DOM.getDocument", { depth: 0 });
  const call = async (objectId, fn) => (await send("Runtime.callFunctionOn", { objectId, functionDeclaration: fn, returnByValue: true })).result.value;
  const list = await send("Runtime.evaluate", { expression: "(" + pageFocusList.toString() + ")(" + cap + ")", returnByValue: false });
  const props = await send("Runtime.getProperties", { objectId: list.result.objectId, ownProperties: true });
  const ids = props.result.filter((p) => /^\d+$/.test(p.name)).map((p) => p.value.objectId);
  const parents = async (objectId) => {
    const out = [];
    let cur = objectId;
    for (let i = 0; i < 2; i++) {
      const r = await send("Runtime.callFunctionOn", { objectId: cur, functionDeclaration: "function(){return this.parentElement}", returnByValue: false });
      if (!r.result.objectId) break;
      cur = r.result.objectId;
      out.push((await send("DOM.requestNode", { objectId: cur })).nodeId);
    }
    return out;
  };
  for (const objectId of ids) {
    const who = await call(objectId, focusDescribe);
    try {
      const { nodeId } = await send("DOM.requestNode", { objectId });
      const up = await parents(objectId);
      const plain = await call(objectId, pageFocusLook.toString());
      await send("CSS.forcePseudoState", { nodeId, forcedPseudoClasses: ["focus", "focus-visible"] });
      for (const n of up) await send("CSS.forcePseudoState", { nodeId: n, forcedPseudoClasses: ["focus-within"] });
      const focused = await call(objectId, pageFocusLook.toString());
      await send("CSS.forcePseudoState", { nodeId, forcedPseudoClasses: [] });
      for (const n of up) await send("CSS.forcePseudoState", { nodeId: n, forcedPseudoClasses: [] });
      if (focused.parts.every((v, i) => v === plain.parts[i]))
        findings.push({ rule: "focus-ring", selector: who.selector, text: who.text, detail: "focused style equals unfocused style" + (plain.ring ? " (" + plain.ring + " in both)" : ", no outline, box-shadow or border change") });
    } catch (e) { unknown.push({ rule: "focus-ring", selector: who.selector, text: who.text, detail: e.message }); }
  }
  const total = await evaluate("window.__orreryFocusTotal");
  const moved = await evaluate("(() => { const b = window.__orreryFocusBefore; delete window.__orreryFocusBefore; delete window.__orreryFocusTotal; if (document.activeElement === b) return false; if (b && b.isConnected) b.focus({ preventScroll: true }); return true; })()");
  await send("CSS.disable"); await send("DOM.disable");
  return { findings, unknown, stats: { focusable: total, checked: ids.length, restored: moved } };
}

// A re-render that swaps the focused node for an equal one and refocuses it has not moved focus,
// so the mark keeps a description to compare when the old node is gone.
const FOCUS_SAY = `const d = (e) => !e || e === document.body ? "body|" : (e.id ? "#" + e.id : e.localName + [...e.classList].slice(0, 2).map((c) => "." + c).join("")) + "|" + (e.getAttribute("aria-label") || e.textContent || "").replace(/\\s+/g, " ").trim().slice(0, 60);`;
export const FOCUS_MARK = `(() => { ${FOCUS_SAY} window.__orreryFocusMark = { el: document.activeElement, say: d(document.activeElement) }; return true; })()`;
export const FOCUS_CHECK = `(() => {
  ${FOCUS_SAY}
  const mark = window.__orreryFocusMark, was = mark.el, now = document.activeElement;
  delete window.__orreryFocusMark;
  if (was === now || (mark.say === d(now) && (!was || was === document.body || !was.isConnected))) return null;
  const sayWas = mark.say.split("|")[0];
  const [sel, text] = d(now).split("|");
  // When the focused control is removed, placing focus on its successor keeps keyboard users in
  // place, and the removal is usually the late result of an earlier key. The audit cannot tell, so
  // it reports the move as unknown.
  if (was && was !== document.body && !was.isConnected) return { unknown: true, rule: "focus-steal", selector: sel, text, detail: "focus moved from " + sayWas + " (removed from the page) to " + sel };
  return { rule: "focus-steal", selector: sel, text, detail: "focus moved from " + sayWas + " to " + sel + " with no input" };
})()`;

// node audit.mjs selftest <live-*.html>  plants one violation per rule plus clean controls, and checks each rule fires once.
// node audit.mjs report <outDir>...      prints the concept-by-rule table and the most frequent findings per concept.
const FIXTURE = `(() => {
  document.body.insertAdjacentHTML("beforeend", \`
  <style>#fxKeep{outline:none}#fxKeep:focus-visible{outline:2px solid #fff}</style>
  <div id="fxBack" style="position:fixed;left:560px;top:240px;width:420px;height:300px;background:#000;z-index:99998"></div>
  <div id="fxBox" style="position:fixed;left:580px;top:260px;z-index:99999;font:14px sans-serif;color:#eee">
    <div id="fxTiny" style="position:absolute;left:0;top:0;white-space:nowrap;font-size:9px">tiny text</div>
    <div id="fxDim" style="position:absolute;left:0;top:30px;white-space:nowrap;color:#444">dim text</div>
    <div id="fxGood" style="position:absolute;left:200px;top:0;white-space:nowrap;font-size:13px;color:#ddd">good text</div>
    <div id="fxA" style="position:absolute;left:0;top:60px;white-space:nowrap;background:#222">label A</div>
    <div id="fxB" style="position:absolute;left:24px;top:66px;white-space:nowrap;background:#222">label B</div>
    <div data-audit-stack style="position:absolute;left:200px;top:60px"><div id="fxC" style="position:absolute;white-space:nowrap;background:#222">stack C</div><div id="fxD" style="position:absolute;left:8px;top:6px;white-space:nowrap;background:#222">stack D</div></div>
    <div id="fxMove" style="position:absolute;left:0;top:120px;white-space:nowrap;transition:opacity .3s">moving</div>
    <div style="position:absolute;left:200px;top:120px;transform:scale(.7);transform-origin:0 0"><div id="fxShrunk" style="white-space:nowrap">shrunk text</div></div>
    <div id="fxSlide" style="position:absolute;left:200px;top:200px;white-space:nowrap;color:#333">sliding</div>
    <button id="fxRing" style="position:absolute;left:0;top:160px;outline:none!important;background:#000;color:#fff;border:0;font:14px sans-serif">no ring</button>
    <button id="fxKeep" style="position:absolute;left:200px;top:160px;background:#000;color:#fff;border:0;font:14px sans-serif">kept</button>
    <div id="gateKv" style="position:absolute;left:0;top:200px;background:#000"><header><span>x · 2 ops</span> <b>150 B</b></header><div class="gate-kvlog-row"><b>a</b> <i>100 B</i></div><div class="gate-kvlog-row"><b>b</b> <i>40 B</i></div></div>
  </div>
  <div id="fxCard" role="dialog" style="position:fixed;left:1500px;top:600px;width:300px;height:100px;background:#111;color:#eee;z-index:99999;font:14px sans-serif">card</div>\`);
  document.querySelector(".egress").innerHTML = '<span class="dot"></span>local + kvarm · 100 B out';
  document.querySelector("#fxKeep").focus();
  setTimeout(() => { const o = document.querySelector("#fxKeep"), n = o.cloneNode(true); o.replaceWith(n); n.focus(); }, 100);
  const slide = document.querySelector("#fxSlide");
  (function tick(t) { slide.style.left = 200 + (t / 4) % 100 + "px"; requestAnimationFrame(tick); })(0);
  return true;
})()`;
const SELFTEST_STEPS = [
  { eval: FIXTURE, expect: true },
  { wait: 400 },
  { eval: "setTimeout(() => document.querySelector('#fxRing').focus(), 100); true", expect: true },
  { wait: 400 },
  { eval: "const g = document.createElement('button'); g.id = 'fxGone'; document.body.append(g); g.focus(); setTimeout(() => { g.remove(); document.querySelector('#fxKeep').focus(); }, 100); true", expect: true },
  { wait: 400 },
  { shot: "selftest" },
  { audit: "focus" },
];
const SELFTEST_EXPECT = [
  ["type-floor", /#fxTiny/, 1], ["type-floor", /#fxShrunk/, 1], ["type-floor", /#fxGood/, 0],
  ["contrast", /#fxDim/, 1], ["contrast", /#fxGood/, 0], ["contrast", /#fxSlide/, 0],
  ["overlap", /#fx[AB]\b/, 1], ["overlap", /#fx[CD]\b/, 0],
  ["offscreen", /#fxCard/, 1],
  ["transition", /#fxMove/, 1],
  ["egress-sum", /\.egress|#gateKv/, 3],
  ["focus-steal", /#fxRing/, 1], ["focus-steal", /#fxKeep/, 0],
  ["focus-ring", /#fxRing/, 1], ["focus-ring", /#fxKeep/, 0],
];

function report(dirs) {
  const rows = [], tops = [];
  for (const dir of dirs) {
    const s = JSON.parse(readFileSync(join(dir, "audit-summary.json"), "utf8"));
    const name = basename(dir).replace(/^audit-/, "");
    rows.push("| " + name + " | " + RULES.map((r) => s.summary[r].findings + " (" + s.summary[r].distinct + ")" + (s.summary[r].unknown ? " ?" + s.summary[r].unknown : "")).join(" | ") + " |");
    const seen = new Map();
    for (const a of s.audits) for (const f of a.findings) {
      const k = f.rule + "|" + f.selector + "|" + f.text;
      const e = seen.get(k) || { f, shots: new Set() };
      e.shots.add(a.shot); seen.set(k, e);
    }
    tops.push("\n### " + name + "\n");
    for (const r of RULES) {
      const list = [...seen.values()].filter((e) => e.f.rule === r).sort((a, b) => b.shots.size - a.shots.size).slice(0, 6);
      for (const { f, shots } of list) tops.push("- " + r + " · `" + f.selector + "` · \"" + f.text + "\" · " + f.detail + " · " + shots.size + " shots");
    }
  }
  return "| run | " + RULES.join(" | ") + " |\n|" + "---|".repeat(RULES.length + 1) + "\n" + rows.join("\n") + "\n" + tops.join("\n");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [cmd, ...rest] = process.argv.slice(2);
  if (cmd === "report") console.log(report(rest));
  else if (cmd === "selftest" && rest[0]) {
    const dir = mkdtempSync(join(tmpdir(), "orrery-audit-selftest-")), steps = join(dir, "selftest.steps.json");
    writeFileSync(steps, JSON.stringify(SELFTEST_STEPS));
    const run = spawnSync(process.execPath, [join(dirname(fileURLToPath(import.meta.url)), "drive.mjs"), rest[0], "", steps, dir], { env: { ...process.env, AUDIT: "1" }, encoding: "utf8" });
    const s = JSON.parse(readFileSync(join(dir, "audit-summary.json"), "utf8"));
    const all = s.audits.flatMap((a) => a.findings), shot = s.audits.find((a) => a.shot === "selftest");
    const checks = SELFTEST_EXPECT.map(([rule, re, n]) => [rule + " " + re + " = " + n, all.filter((f) => f.rule === rule && re.test(f.selector)).length === n]);
    checks.push(["data-audit-stack opt-out counted and skipped", shot.stats.stackOptOuts === 1 && shot.stats.stackSkips >= 1]);
    checks.push(["focus restored after its control was removed is unknown, not a finding", shot.unknown.some((u) => u.rule === "focus-steal" && /#fxKeep/.test(u.selector))]);
    checks.push(["moving text is contrast unknown, not a finding", shot.unknown.some((u) => u.rule === "contrast" && /#fxSlide/.test(u.selector) && /moved/.test(u.detail))]);
    checks.push(["drive verdict line printed", /\nOK\n/.test(run.stdout)]);
    for (const [what, ok] of checks) console.log((ok ? "PASS " : "FAIL ") + what);
    console.log("selftest output in " + dir);
    process.exit(checks.every((c) => c[1]) ? 0 : 1);
  } else { console.error("usage: node audit.mjs selftest <live-*.html> | node audit.mjs report <outDir>..."); process.exit(2); }
}
