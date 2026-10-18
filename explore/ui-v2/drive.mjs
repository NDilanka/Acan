// Drives a built prototype in headless Chrome: runs steps, saves screenshots, fails on page errors.
// Usage: node drive.mjs <file.html> <hash> <steps.json> <outDir>
// Steps: {key, shift?, ctrl?, alt?} {click: selector} {clickAt: [x, y]} {drag: [x1, y1, x2, y2]}
//        {type: text} {wait: ms} {eval: expr, expect?: value} {shot: name}
//        {audit: "focus"} runs the focus-ring rule; it does nothing without AUDIT=1.
// AUDIT=1 runs audit.mjs at every shot, writes <outDir>/<shot>.audit.json, checks focus across every wait,
// prints findings per rule at the end, and exits non-zero on any finding.
import { spawn } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const [file, hash = "", stepsPath, outDir = "shots"] = process.argv.slice(2);
if (!file || !stepsPath) { console.error("usage: node drive.mjs <file.html> <hash> <steps.json> <outDir>"); process.exit(2); }
const steps = JSON.parse(readFileSync(stepsPath, "utf8"));
const AUDIT = process.env.AUDIT === "1";
const auditLib = AUDIT ? await import("./audit.mjs") : null;
const audits = [], auditCarry = [];
mkdirSync(outDir, { recursive: true });

const profile = mkdtempSync(join(tmpdir(), "acan-drive-"));
const chrome = spawn("google-chrome", ["--headless=new", "--remote-debugging-port=0", "--user-data-dir=" + profile, "--no-first-run", "--hide-scrollbars", "--window-size=1600,900", "about:blank"], { stdio: ["ignore", "ignore", "pipe"] });
const port = await new Promise((ok, no) => {
  let buf = "";
  chrome.stderr.on("data", (d) => { buf += d; const m = buf.match(/DevTools listening on ws:\/\/[^:]+:(\d+)\//); if (m) ok(m[1]); });
  setTimeout(() => no(new Error("chrome did not start: " + buf.slice(-400))), 15000);
});
const page = (await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()).find((t) => t.type === "page");
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((ok) => ws.addEventListener("open", ok, { once: true }));

let seq = 0;
const pending = new Map(), waiters = [];
const errors = [], failures = [];
ws.addEventListener("message", (ev) => {
  const msg = JSON.parse(ev.data);
  if (msg.id && pending.has(msg.id)) { const { ok, no } = pending.get(msg.id); pending.delete(msg.id); msg.error ? no(new Error(msg.error.message)) : ok(msg.result); return; }
  if (msg.method === "Runtime.exceptionThrown") errors.push("exception: " + (msg.params.exceptionDetails.exception?.description || msg.params.exceptionDetails.text));
  if (msg.method === "Runtime.consoleAPICalled" && msg.params.type === "error") errors.push("console.error: " + msg.params.args.map((a) => a.value ?? a.description).join(" "));
  if (msg.method === "Log.entryAdded" && msg.params.entry.level === "error") errors.push("log: " + msg.params.entry.text);
  for (const w of waiters.splice(0)) if (w.method === msg.method) w.ok(); else waiters.push(w);
});
const send = (method, params = {}) => new Promise((ok, no) => { const id = ++seq; pending.set(id, { ok, no }); ws.send(JSON.stringify({ id, method, params })); });
const once = (method) => new Promise((ok) => waiters.push({ method, ok }));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const evaluate = async (expression) => {
  const r = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
  if (r.exceptionDetails) throw new Error("eval failed: " + (r.exceptionDetails.exception?.description || r.exceptionDetails.text));
  return r.result.value;
};
const mouse = async (x, y, type) => send("Input.dispatchMouseEvent", { type, x, y, button: "left", buttons: type === "mouseReleased" ? 0 : 1, clickCount: 1, pointerType: "mouse" });

await send("Page.enable"); await send("Runtime.enable"); await send("Log.enable");
await send("Emulation.setDeviceMetricsOverride", { width: 1600, height: 900, deviceScaleFactor: 1, mobile: false });
const loaded = once("Page.loadEventFired");
await send("Page.navigate", { url: "file://" + resolve(file) + (hash ? "#" + hash.replace(/^#/, "") : "") });
await loaded; await sleep(900);

for (const [i, s] of steps.entries()) {
  const tag = `step ${i + 1} ${JSON.stringify(s)}`;
  try {
    if (s.key) {
      const modifiers = (s.alt ? 1 : 0) | (s.ctrl ? 2 : 0) | (s.shift ? 8 : 0);
      const text = s.key.length === 1 ? s.key : s.key === "Enter" ? "\r" : undefined;
      const code = s.key.length === 1 ? (/[a-z]/i.test(s.key) ? "Key" + s.key.toUpperCase() : /\d/.test(s.key) ? "Digit" + s.key : "") : s.key;
      await send("Input.dispatchKeyEvent", { type: "keyDown", key: s.key, code, text, modifiers });
      await send("Input.dispatchKeyEvent", { type: "keyUp", key: s.key, code, modifiers });
      await sleep(s.after ?? 250);
    } else if (s.click) {
      const c = await evaluate(`(() => { const e = document.querySelector(${JSON.stringify(s.click)}); if (!e) return null; const r = e.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
      if (!c) throw new Error("no element " + s.click);
      await mouse(c[0], c[1], "mouseMoved"); await mouse(c[0], c[1], "mousePressed"); await mouse(c[0], c[1], "mouseReleased");
      await sleep(s.after ?? 300);
    } else if (s.clickAt) {
      const [x, y] = s.clickAt;
      await mouse(x, y, "mouseMoved"); await mouse(x, y, "mousePressed"); await mouse(x, y, "mouseReleased");
      await sleep(s.after ?? 300);
    } else if (s.drag) {
      const [x1, y1, x2, y2] = s.drag;
      await mouse(x1, y1, "mouseMoved"); await mouse(x1, y1, "mousePressed");
      for (let t = 1; t <= 10; t++) await mouse(x1 + ((x2 - x1) * t) / 10, y1 + ((y2 - y1) * t) / 10, "mouseMoved");
      await mouse(x2, y2, "mouseReleased");
      await sleep(s.after ?? 300);
    } else if (s.type !== undefined) {
      await send("Input.insertText", { text: s.type }); await sleep(s.after ?? 200);
    } else if (s.wait) {
      if (AUDIT) await evaluate(auditLib.FOCUS_MARK);
      await sleep(s.wait);
      if (AUDIT) { const f = await evaluate(auditLib.FOCUS_CHECK); if (f) auditCarry.push({ ...f, detail: f.detail + " · step " + (i + 1) + " wait " + s.wait + " ms" }); }
    } else if (s.eval) {
      const v = await evaluate(s.eval);
      const shown = JSON.stringify(v);
      if ("expect" in s && JSON.stringify(s.expect) !== shown) failures.push(`${tag}: expected ${JSON.stringify(s.expect)}, got ${shown}`);
      console.log(`${tag} => ${shown}`);
    } else if (s.shot) {
      if (AUDIT) await evaluate(auditLib.SHOT_MARK);
      const { data } = await send("Page.captureScreenshot", { format: "png" });
      const out = join(outDir, s.shot + ".png");
      writeFileSync(out, Buffer.from(data, "base64"));
      console.log("shot " + out);
      if (AUDIT) {
        const t = Date.now();
        const r = await auditLib.audit({ send, evaluate }, data);
        r.stats.ms = Date.now() - t;
        for (const c of auditCarry.splice(0).reverse()) c.unknown ? (delete c.unknown, r.unknown.unshift(c)) : r.findings.unshift(c);
        writeFileSync(join(outDir, s.shot + ".audit.json"), JSON.stringify({ shot: s.shot, step: i + 1, ...r }, null, 1));
        audits.push({ shot: s.shot, ...r });
        console.log(`audit ${s.shot}: ${r.findings.length} findings, ${r.unknown.length} unknown, ${r.stats.ms} ms`);
      }
    } else if (s.audit) {
      if (AUDIT) {
        if (s.audit !== "focus") throw new Error("unknown audit " + s.audit);
        const name = "step-" + (i + 1) + "-focus";
        const r = await auditLib.auditFocus({ send, evaluate });
        writeFileSync(join(outDir, name + ".audit.json"), JSON.stringify({ shot: name, step: i + 1, ...r }, null, 1));
        audits.push({ shot: name, ...r });
        console.log(`audit ${name}: ${r.findings.length} findings, ${r.stats.checked} of ${r.stats.focusable} controls`);
      }
    } else throw new Error("unknown step");
  } catch (e) { failures.push(`${tag}: ${e.message}`); }
}

ws.close();
// Chrome keeps writing to its profile until it exits, so removing it early races with ENOTEMPTY.
const exited = new Promise((ok) => chrome.once("exit", ok));
chrome.kill();
await Promise.race([exited, sleep(5000)]);
try { rmSync(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 }); } catch (e) { console.log("cleanup skipped: " + e.message); }
for (const e of errors) console.log("PAGE ERROR " + e);
for (const f of failures) console.log("FAIL " + f);
let auditFindings = 0;
if (AUDIT) {
  if (auditCarry.length) audits.push({ shot: "after-last-shot", findings: auditCarry.filter((c) => !c.unknown), unknown: auditCarry.filter((c) => c.unknown), stats: {} });
  const by = Object.fromEntries(auditLib.RULES.map((r) => [r, { findings: 0, distinct: new Set(), unknown: 0 }]));
  for (const a of audits) {
    for (const f of a.findings) { by[f.rule].findings++; by[f.rule].distinct.add(f.selector + "|" + f.text + "|" + f.detail.replace(/[\d.]+/g, "#")); }
    for (const u of a.unknown) by[u.rule].unknown++;
  }
  const summary = Object.fromEntries(Object.entries(by).map(([r, v]) => [r, { findings: v.findings, distinct: v.distinct.size, unknown: v.unknown }]));
  const optOuts = Math.max(0, ...audits.map((a) => a.stats.stackOptOuts || 0));
  writeFileSync(join(outDir, "audit-summary.json"), JSON.stringify({ file, hash, steps: stepsPath, summary, optOuts, audits }, null, 1));
  for (const [r, v] of Object.entries(summary)) console.log(`AUDIT ${r.padEnd(11)} ${v.findings} findings, ${v.distinct} distinct, ${v.unknown} unknown`);
  console.log("AUDIT data-audit-stack opt-outs " + optOuts);
  auditFindings = Object.values(summary).reduce((n, v) => n + v.findings, 0);
  console.log(auditFindings ? `AUDIT NOT OK, ${auditFindings} findings in ${audits.length} audits` : `AUDIT OK, ${audits.length} audits`);
}
console.log(errors.length || failures.length ? `NOT OK, ${errors.length} page errors, ${failures.length} failures` : "OK");
process.exit(errors.length || failures.length || auditFindings ? 1 : 0);
