// Drives a built prototype in headless Chrome: runs steps, saves screenshots, fails on page errors.
// Usage: node drive.mjs <file.html> <hash> <steps.json> <outDir>
// Steps: {key, shift?, ctrl?, alt?} {click: selector} {clickAt: [x, y]} {drag: [x1, y1, x2, y2]}
//        {type: text} {wait: ms} {eval: expr, expect?: value} {shot: name}
import { spawn } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const [file, hash = "", stepsPath, outDir = "shots"] = process.argv.slice(2);
if (!file || !stepsPath) { console.error("usage: node drive.mjs <file.html> <hash> <steps.json> <outDir>"); process.exit(2); }
const steps = JSON.parse(readFileSync(stepsPath, "utf8"));
mkdirSync(outDir, { recursive: true });

const profile = mkdtempSync(join(tmpdir(), "orrery-drive-"));
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
      await sleep(s.wait);
    } else if (s.eval) {
      const v = await evaluate(s.eval);
      const shown = JSON.stringify(v);
      if ("expect" in s && JSON.stringify(s.expect) !== shown) failures.push(`${tag}: expected ${JSON.stringify(s.expect)}, got ${shown}`);
      console.log(`${tag} => ${shown}`);
    } else if (s.shot) {
      const { data } = await send("Page.captureScreenshot", { format: "png" });
      const out = join(outDir, s.shot + ".png");
      writeFileSync(out, Buffer.from(data, "base64"));
      console.log("shot " + out);
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
console.log(errors.length || failures.length ? `NOT OK, ${errors.length} page errors, ${failures.length} failures` : "OK");
process.exit(errors.length || failures.length ? 1 : 0);
