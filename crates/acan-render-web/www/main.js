// The Acan product screen. The wasm renderer draws the plant and folds the origin's log into a
// view. This page owns the canvas input, the label and pin overlays, the HTML panels, the owner
// token and the stream. It draws only when something changed: input, a resize, new data, or a
// turn that is still easing. An idle page asks for no frames.
import init, { App } from "./pkg/acan_render_web.js";

const TOKEN_KEY = "acan.owner";
const USAGE_DELAY_MS = 800;
const RECONNECT_MS = 1000;
const ID_PREFIX = { order: "o", clash: "c", question: "q", task: "t" };
const PAN_KEY_PX = 80;
const ZOOM_KEY = 1.25;
const WHEEL_RATE = 0.0015;
const TURN_DEG = 45;
const LABEL_OFFSET_PX = 6;
const LABEL_PAD_PX = 4;
// A press that moves less than this is a click, and a click within PIN_PICK_PX of a clash pin's
// tip opens its card.
const CLICK_PX = 4;
const PIN_PICK_PX = 18;

const params = new URLSearchParams(location.search);
const $ = (id) => document.getElementById(id);
const canvas = $("plant");
const labelsEl = $("labels");
const flagsEl = $("flags");
const card = $("card");
const gate = $("gate");
const link = $("link");
const orderEl = $("order");
const orderLine = $("order-line");
const usagePanel = $("usage");
const usageTable = $("usage-table");
const usageBody = $("usage-body");
const usageNone = $("usage-none");
const repoEl = $("repo");
const noticeEl = $("notice");

const timing = { started: performance.now() };
let app = null;
let token = null;
let view = null;
let renderedVersion = -1;
// The marks as the wasm side last gave them: the flags in order, and the open clash card.
let markState = { flags: [], card: null };
let flagEls = [];
let socket = null;
let usageTimer = null;
let usageRows = [];
let settling = false;
let note = "";
let scheduled = false;
let lastFrame = null;
let drag = null;
let press = null;
let altDown = false;
let labelGeneration = 0;
const labels = new Map();

function requestRender() {
  if (scheduled || app === null) return;
  scheduled = true;
  requestAnimationFrame(drawFrame);
}

// `now` is the frame time. The bench calls this with no argument, which reads as a frame with
// no time elapsed on a turn.
function drawFrame(now = performance.now()) {
  scheduled = false;
  const elapsed = lastFrame === null ? 0 : now - lastFrame;
  lastFrame = now;
  try {
    const turning = app.advance(elapsed);
    if (!app.render()) requestRender();
    if (timing.firstFrame === undefined) timing.firstFrame = performance.now();
    syncView();
    placeLabels();
    if (turning) {
      requestRender();
    } else {
      lastFrame = null;
    }
  } catch (err) {
    console.error(err);
  }
}

// The view is rebuilt by the wasm side after the log changes. The panels are redrawn only when
// the version moves, so a camera move never rebuilds them.
function syncView() {
  const version = app.view_version();
  if (version === 0 || version === renderedVersion) return;
  renderedVersion = version;
  view = JSON.parse(app.view_json());
  paint();
  // The marks ease in from where they start, so the next frame must advance them.
  requestRender();
  scheduleUsage();
  if (socket) setLink(`live · seq ${app.next_seq()}`);
}

// Labels come in priority order, larger and nearer first. A label that would overlap one
// already placed stays hidden, so the plant never shows a pile of names. The flags are placed
// first, so a machine name never hides one.
function placeLabels() {
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  labelGeneration += 1;
  const placedBoxes = [];
  placeFlags(placedBoxes);
  const quads = app.label_layout(altDown);
  for (let k = 0; k < quads.length; k += 4) {
    const index = quads[k];
    const x = quads[k + 1];
    const y = quads[k + 2];
    const opacity = quads[k + 3];
    let el = labels.get(index);
    if (el === undefined) {
      el = makeLabel(index);
      labels.set(index, el);
      labelsEl.appendChild(el);
    }
    el.generation = labelGeneration;
    if (el.size === undefined || el.sizedAlt !== altDown) {
      el.size = [el.offsetWidth, el.offsetHeight];
      el.sizedAlt = altDown;
    }
    const [w, h] = el.size;
    const box = {
      left: x - w / 2 - LABEL_PAD_PX,
      right: x + w / 2 + LABEL_PAD_PX,
      top: y + LABEL_OFFSET_PX - LABEL_PAD_PX,
      bottom: y + LABEL_OFFSET_PX + h + LABEL_PAD_PX,
    };
    const inside = box.right >= 0 && box.left <= width && box.bottom >= 0 && box.top <= height;
    const clear = inside && placedBoxes.every((other) => !overlaps(box, other));
    if (clear) placedBoxes.push(box);
    el.style.visibility = clear ? "visible" : "hidden";
    el.style.opacity = String(opacity);
    el.style.transform = `translate(${x}px, ${y}px) translate(-50%, ${LABEL_OFFSET_PX}px)`;
  }
  for (const [index, el] of labels) {
    if (el.generation !== labelGeneration) {
      el.remove();
      labels.delete(index);
    }
  }
}

function overlaps(a, b) {
  return a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom;
}

function makeLabel(index) {
  const el = document.createElement("div");
  el.className = "label";
  el.textContent = app.machine_name(index);
  const files = app.machine_files(index);
  const count = document.createElement("span");
  count.className = "count";
  count.textContent = `${files} ${files === 1 ? "file" : "files"}`;
  el.appendChild(count);
  return el;
}

function label(kind, id) {
  return `${ID_PREFIX[kind]}-${id}`;
}

function setLink(text) {
  link.textContent = text;
}

function forgetToken() {
  sessionStorage.removeItem(TOKEN_KEY);
  token = null;
  if (socket) {
    const old = socket;
    socket = null;
    old.close();
  }
  gate.hidden = false;
  setLink("sign in");
}

async function get(path) {
  const res = await fetch(path, { headers: { Authorization: `Bearer ${token}` } });
  if (res.status === 401 || res.status === 403) {
    forgetToken();
    throw new Error("the owner token was refused");
  }
  if (!res.ok) {
    throw new Error(`${path} answered ${res.status}`);
  }
  return res.text();
}

async function start() {
  setLink("loading");
  app.set_plant(await get("/api/plant"));
  await reload();
  connect();
  refreshUsage();
}

async function reload() {
  app.load_log(await get("/api/events?from=0"));
  requestRender();
}

function connect() {
  const next = app.next_seq();
  const scheme = location.protocol === "https:" ? "wss" : "ws";
  const ws = new WebSocket(`${scheme}://${location.host}/api/stream?from=${next}`);
  socket = ws;
  ws.onopen = () => {
    ws.send(JSON.stringify({ token }));
    setLink(`live · seq ${app.next_seq()}`);
  };
  ws.onmessage = (message) => {
    try {
      app.ingest(message.data);
      requestRender();
    } catch (err) {
      console.error("the stream could not be applied; reloading the log", err);
      ws.close();
    }
  };
  ws.onclose = (event) => {
    if (socket !== ws) return;
    socket = null;
    if (!token) return;
    if (event.code === 1008) {
      forgetToken();
      return;
    }
    setLink("reconnecting");
    setTimeout(resync, RECONNECT_MS);
  };
}

async function resync() {
  if (!token) return;
  try {
    await reload();
    connect();
  } catch (err) {
    console.error(err);
    setLink("reconnecting");
    setTimeout(resync, RECONNECT_MS * 2);
  }
}

function paint() {
  paintOrderLine();
  paintMarks();
}

function paintOrderLine() {
  const order = view.orders.find((item) => item.phase !== "cancelled" && item.phase !== "landed")
    ?? view.orders.at(-1);
  if (!order) {
    orderEl.hidden = true;
    orderLine.textContent = "";
    return;
  }
  orderEl.hidden = false;
  orderLine.textContent = `${label("order", order.id)} · ${order.machine} · ${order.phase.replace("_", " ")} · ${order.sentence}`;
}

// The marks come from the wasm side: each agent with its activity, the active order, each open
// clash and each question. They are rebuilt when the view changes, and placed every frame.
function paintMarks() {
  markState = JSON.parse(app.marks_json());
  flagsEl.replaceChildren();
  flagEls = markState.flags.map((flag) => {
    const el = flagElement(flag);
    flagsEl.append(el);
    return el;
  });
  paintCard();
}

function flagElement(flag) {
  const el = document.createElement("div");
  el.className = `flag ${flag.kind}`;
  if (flag.hue) {
    const dot = document.createElement("span");
    dot.className = "dot";
    dot.style.background = flag.hue;
    el.append(dot);
  }
  const text = document.createElement("span");
  text.textContent = flag.text;
  el.append(text);
  if (flag.waiting) {
    const amber = document.createElement("span");
    amber.className = "dot wait";
    el.append(amber);
  }
  if (flag.clash !== null) {
    el.addEventListener("click", () => openCard(flag.clash));
  }
  return el;
}

// Each flag is anchored above its point and keeps its place as the bodies and pins move. A
// flag that would overlap one already placed is hidden; flags are placed before the names.
function placeFlags(placedBoxes) {
  const quads = app.flag_layout();
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  for (let k = 0; k < quads.length; k += 4) {
    const el = flagEls[quads[k]];
    if (el === undefined) continue;
    const x = quads[k + 1];
    const y = quads[k + 2];
    if (el.size === undefined) el.size = [el.offsetWidth, el.offsetHeight];
    const [w, h] = el.size;
    const box = {
      left: x - w / 2 - LABEL_PAD_PX,
      right: x + w / 2 + LABEL_PAD_PX,
      top: y - LABEL_OFFSET_PX - h - LABEL_PAD_PX,
      bottom: y - LABEL_OFFSET_PX + LABEL_PAD_PX,
    };
    const inside = box.right >= 0 && box.left <= width && box.bottom >= 0 && box.top <= height;
    const clear = inside && placedBoxes.every((other) => !overlaps(box, other));
    if (clear) placedBoxes.push(box);
    el.style.visibility = clear ? "visible" : "hidden";
    el.style.transform = `translate(${x}px, ${y}px) translate(-50%, calc(-100% - ${LABEL_OFFSET_PX}px))`;
  }
}

function openCard(index) {
  app.open_card(index);
  note = "";
  showCard();
}

function toggleCard() {
  app.toggle_card();
  note = "";
  showCard();
}

function showCard() {
  markState = JSON.parse(app.marks_json());
  paintCard();
  requestRender();
}

// The pin whose tip lies within PIN_PICK_PX of the CSS point, nearest first, or null.
function pinAt(x, y) {
  const quads = app.pin_layout();
  let found = null;
  let best = PIN_PICK_PX;
  for (let k = 0; k < quads.length; k += 3) {
    const distance = Math.hypot(quads[k + 1] - x, quads[k + 2] - y);
    if (distance < best) {
      best = distance;
      found = quads[k];
    }
  }
  return found;
}

// The clash card: the two declarations with their hues, the witness the clash would break, and
// the suggested settlement last. Below them, what each key does. Settled clashes show how they
// were resolved.
function paintCard() {
  const data = markState.card;
  if (!data || !view) {
    card.hidden = true;
    return;
  }
  card.hidden = false;
  const clash = view.clashes[data.clash];
  const unsettled = clash.state === "open" || clash.state === "proposed";

  const head = document.createElement("div");
  head.className = "head";
  const title = document.createElement("h2");
  title.textContent = data.title;
  const close = document.createElement("button");
  close.type = "button";
  close.className = "close";
  close.textContent = "close";
  close.addEventListener("click", toggleCard);
  head.append(title, close);
  const children = [head];

  for (const row of data.rows) {
    if (row.kind === "settle" && !unsettled) continue;
    children.push(cardRow(row, clash));
  }
  if (unsettled && data.keys.length > 0) {
    const list = document.createElement("ol");
    list.className = "keys";
    for (const line of data.keys) {
      const item = document.createElement("li");
      item.textContent = line;
      list.append(item);
    }
    children.push(list);
  }
  if (!unsettled && data.resolved) {
    const resolved = document.createElement("p");
    resolved.className = "resolved";
    resolved.textContent = data.resolved;
    children.push(resolved);
  }
  if (note) {
    const error = document.createElement("p");
    error.className = "note";
    error.textContent = note;
    children.push(error);
  }
  card.replaceChildren(...children);
}

function cardRow(row, clash) {
  const el = document.createElement("p");
  el.className = `row ${row.kind}`;
  if (row.kind === "declaration") {
    const dot = document.createElement("span");
    dot.className = "dot";
    dot.style.background = row.hue;
    const text = document.createElement("span");
    text.textContent = row.text;
    const who = document.createElement("span");
    who.className = "muted";
    who.textContent = row.agent;
    el.append(dot, text, who);
  } else if (row.kind === "witness") {
    el.textContent = row.text;
  } else {
    const key = document.createElement("span");
    key.className = "key";
    key.textContent = String(row.key);
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = row.suggested ? `${row.text} (suggested)` : row.text;
    button.disabled = settling;
    button.addEventListener("click", () => settle(clash, clash.options[row.key - 1]));
    el.append(key, button);
  }
  return el;
}

// Keys 1, 2 and 3 settle the open clash on the card with the matching choice.
function settleByKey(key) {
  const data = markState.card;
  if (!data || !view) return;
  const clash = view.clashes[data.clash];
  if (!clash || (clash.state !== "open" && clash.state !== "proposed")) return;
  const option = clash.options[key - 1];
  if (option) settle(clash, option);
}

async function settle(clash, option) {
  if (!token || settling) return;
  settling = true;
  note = "";
  paintCard();
  try {
    const res = await fetch("/api/commands", {
      method: "POST",
      headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
      body: JSON.stringify({ cmd: "settle_clash", clash: clash.id, choice: option.choice }),
    });
    if (res.status === 401 || res.status === 403) {
      forgetToken();
    } else if (!res.ok) {
      note = `the origin refused: ${(await res.text()).slice(0, 160)}`;
    }
  } catch (err) {
    note = "the settle did not reach the server";
  } finally {
    settling = false;
    paintCard();
  }
}

function scheduleUsage() {
  if (usageTimer) return;
  usageTimer = setTimeout(() => {
    usageTimer = null;
    refreshUsage();
  }, USAGE_DELAY_MS);
}

async function refreshUsage() {
  if (!token) return;
  try {
    renderUsage(JSON.parse(await get("/api/usage")));
  } catch (err) {
    console.error(err);
  }
}

// The usage panel stays hidden until a model has been called. Holding Alt shows the one-line
// `no model calls yet` instead, so the panel is never empty on screen.
function renderUsage(rows) {
  usageRows = rows;
  usageBody.replaceChildren(...rows.map((row) => {
    const tr = document.createElement("tr");
    const cells = [row.role, row.model, row.provider, row.calls, row.prompt_tokens, row.completion_tokens];
    cells.forEach((value, i) => {
      const td = document.createElement("td");
      td.textContent = String(value);
      if (i === 2) td.className = row.provider === "offline" ? "offline" : "live";
      tr.append(td);
    });
    return tr;
  }));
  paintUsage();
}

function paintUsage() {
  const any = usageRows.length > 0;
  usageTable.hidden = !any;
  usageNone.hidden = any;
  usagePanel.hidden = !any && !altDown;
}

// Sets the drawing buffer to the CSS box times the device pixel ratio. The wasm side rounds
// the same product, so the two sizes agree.
function setDrawingSize() {
  const rect = canvas.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;
  canvas.width = Math.max(1, Math.round(rect.width * dpr));
  canvas.height = Math.max(1, Math.round(rect.height * dpr));
  return { rect, dpr };
}

function sizeCanvas() {
  const { rect, dpr } = setDrawingSize();
  if (app !== null) {
    app.resize(rect.width, rect.height, dpr);
    requestRender();
  }
}

function showNotice(text) {
  noticeEl.textContent = text;
  noticeEl.hidden = false;
}

function zoomAtCentre(factor) {
  const rect = canvas.getBoundingClientRect();
  app.zoom_at(rect.width / 2, rect.height / 2, factor);
  requestRender();
}

function turn(degrees) {
  app.turn(degrees);
  requestRender();
}

// Left drag moves the ground under the pointer, so the point you grab stays under the cursor.
// Right drag, or Alt with left drag, orbits and tilts about the ground point under the screen
// centre.
function bindCanvas() {
  canvas.addEventListener("contextmenu", (event) => event.preventDefault());

  canvas.addEventListener("pointerdown", (event) => {
    if (app === null || (event.button !== 0 && event.button !== 2)) return;
    canvas.setPointerCapture(event.pointerId);
    drag = {
      x: event.clientX,
      y: event.clientY,
      orbit: event.button === 2 || event.altKey,
    };
    press = event.button === 0 ? { x: event.clientX, y: event.clientY } : null;
    canvas.classList.add("dragging");
  });

  canvas.addEventListener("pointermove", (event) => {
    if (drag === null) return;
    if (drag.orbit) {
      app.orbit(event.clientX - drag.x, event.clientY - drag.y);
    } else {
      const rect = canvas.getBoundingClientRect();
      app.drag(
        drag.x - rect.left,
        drag.y - rect.top,
        event.clientX - rect.left,
        event.clientY - rect.top,
      );
    }
    drag.x = event.clientX;
    drag.y = event.clientY;
    requestRender();
  });

  // A press that did not move is a click. A click near a clash pin's tip opens its card; the
  // camera stays where it is.
  const endDrag = (event) => {
    const pressed = press;
    press = null;
    drag = null;
    canvas.classList.remove("dragging");
    if (event.type !== "pointerup" || pressed === null) return;
    if (Math.hypot(event.clientX - pressed.x, event.clientY - pressed.y) >= CLICK_PX) return;
    const rect = canvas.getBoundingClientRect();
    const clash = pinAt(event.clientX - rect.left, event.clientY - rect.top);
    if (clash !== null) openCard(clash);
  };
  canvas.addEventListener("pointerup", endDrag);
  canvas.addEventListener("pointercancel", endDrag);

  canvas.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      if (app === null) return;
      const rect = canvas.getBoundingClientRect();
      const lines = event.deltaMode === 1 ? 16 : 1;
      const factor = Math.exp(-event.deltaY * lines * WHEEL_RATE);
      app.zoom_at(event.clientX - rect.left, event.clientY - rect.top, factor);
      requestRender();
    },
    { passive: false },
  );
}

const ARROW_PAN = {
  ArrowLeft: [PAN_KEY_PX, 0],
  ArrowRight: [-PAN_KEY_PX, 0],
  ArrowUp: [0, PAN_KEY_PX],
  ArrowDown: [0, -PAN_KEY_PX],
};

function setAlt(down) {
  if (altDown === down) return;
  altDown = down;
  labelsEl.classList.toggle("alt", down);
  paintUsage();
  requestRender();
}

function bindKeys() {
  window.addEventListener("keydown", (event) => {
    if (event.target instanceof HTMLInputElement) return;
    if (event.key === "Alt") {
      event.preventDefault();
      setAlt(true);
      return;
    }
    if (app === null || event.ctrlKey || event.metaKey) return;
    if (ARROW_PAN[event.key]) {
      event.preventDefault();
      app.pan(...ARROW_PAN[event.key]);
      requestRender();
      return;
    }
    if (event.key === "5" || event.code === "Numpad5") {
      if (!event.repeat) {
        app.toggle_projection();
        requestRender();
      }
      return;
    }
    if (/^[1-3]$/.test(event.key)) {
      settleByKey(Number(event.key));
      return;
    }
    switch (event.key) {
      case "c":
      case "C":
        if (!event.repeat) toggleCard();
        break;
      case "+":
      case "=":
        zoomAtCentre(ZOOM_KEY);
        break;
      case "-":
      case "_":
        zoomAtCentre(1 / ZOOM_KEY);
        break;
      case "0":
      case "f":
        app.fit();
        requestRender();
        break;
      case "q":
      case "Q":
        if (!event.repeat) turn(-TURN_DEG);
        break;
      case "e":
      case "E":
        if (!event.repeat) turn(TURN_DEG);
        break;
      default:
    }
  });

  window.addEventListener("keyup", (event) => {
    if (event.key === "Alt") setAlt(false);
  });
  window.addEventListener("blur", () => setAlt(false));

  $("token-form").addEventListener("submit", (event) => {
    event.preventDefault();
    const value = $("token").value.trim();
    $("token").value = "";
    if (!value) return;
    sessionStorage.setItem(TOKEN_KEY, value);
    token = value;
    gate.hidden = true;
    start().catch((err) => {
      console.error(err);
      setLink("failed");
    });
  });
}

// The owner token arrives as `#token=` from `acan serve`. It is kept for the session only, then
// the fragment is removed so it does not sit in the address bar or the history.
function adoptToken() {
  const fragment = new URLSearchParams(location.hash.slice(1)).get("token");
  if (fragment) {
    sessionStorage.setItem(TOKEN_KEY, fragment);
    history.replaceState(null, "", location.pathname + location.search);
  }
  return sessionStorage.getItem(TOKEN_KEY);
}

async function createApp() {
  const backend = params.get("backend") || (navigator.gpu ? "webgpu" : "webgl");
  try {
    return await App.create(canvas, backend);
  } catch (err) {
    if (backend === "webgl") throw err;
    console.error(err);
    return App.create(canvas, "webgl");
  }
}

// The page loads its plant one of three ways. `?plant=stress&machines=N` builds a generated
// plant for the benchmark. `?plant=fixture` loads the demo plant with no log. Otherwise the
// page reads the live origin, which needs the owner token.
async function loadPlant() {
  const source = params.get("plant");
  if (source === "stress") {
    const machines = Number(params.get("machines") || 2000);
    const started = performance.now();
    app.set_stress(machines);
    timing.load = performance.now() - started;
  } else if (source === "fixture") {
    const response = await fetch("fixtures/demo-plant.json");
    if (!response.ok) throw new Error(`fixture HTTP ${response.status}`);
    app.set_plant(await response.text());
  } else {
    token = adoptToken();
    if (!token) {
      gate.hidden = false;
      setLink("sign in");
      return;
    }
    try {
      await start();
    } catch (err) {
      console.error(err);
      setLink("failed");
    }
  }
  repoEl.textContent = app.repo_name();
}

async function boot() {
  const started = performance.now();
  await init();
  timing.init = performance.now() - started;
  setDrawingSize();
  const createStarted = performance.now();
  app = await createApp();
  timing.create = performance.now() - createStarted;
  sizeCanvas();
  bindCanvas();
  bindKeys();
  window.addEventListener("resize", sizeCanvas);
  await loadPlant();
  requestRender();
  if (params.get("bench") === "1") {
    const { installBench } = await import("./bench.js");
    installBench({ App, app, canvas, timing, redraw: drawFrame, setAlt });
  }
}

boot().catch((error) => {
  console.error(error);
  showNotice(String(error?.message ?? error));
});
