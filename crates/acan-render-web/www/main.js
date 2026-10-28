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

const params = new URLSearchParams(location.search);
const $ = (id) => document.getElementById(id);
const canvas = $("plant");
const labelsEl = $("labels");
const pins = $("pins");
const card = $("card");
const gate = $("gate");
const link = $("link");
const orderEl = $("order");
const orderLine = $("order-line");
const usageBody = $("usage-body");
const repoEl = $("repo");
const noticeEl = $("notice");

const timing = { started: performance.now() };
let app = null;
let token = null;
let view = null;
let renderedVersion = -1;
let selectedClash = null;
let socket = null;
let usageTimer = null;
let settling = false;
let placed = [];
let note = "";
let scheduled = false;
let lastFrame = null;
let drag = null;
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
    placePins();
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
  scheduleUsage();
  if (socket) setLink(`live · seq ${app.next_seq()}`);
}

// Labels come in priority order, larger and nearer first. A label that would overlap one
// already placed stays hidden, so the plant never shows a pile of names.
function placeLabels() {
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  labelGeneration += 1;
  const quads = app.label_layout(altDown);
  const placedBoxes = [];
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
  if (selectedClash === null) {
    const open = view.clashes.find((clash) => clash.state === "open" || clash.state === "proposed");
    if (open) selectedClash = open.id;
  }
  paintOrderLine();
  paintPins();
  paintCard();
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

// Pins sit on the plant's ground plan, at the same coordinates as the labels. Machine names
// are the labels, so they get no pin here.
function paintPins() {
  placed = [];
  pins.replaceChildren();
  const machines = new Map(view.machines.map((machine) => [machine.name, machine]));

  for (const order of view.orders) {
    const home = machines.get(order.machine);
    if (!home || !order.at) continue;
    const x = home.min[0] + home.size[0] / 2;
    addPin("pin order", x, home.min[1], `${label("order", order.id)} ${order.phase.replace("_", " ")}`, { dy: -22, dx: -40 });
  }
  for (const agent of view.agents) {
    if (!agent.body) continue;
    const [x, y] = agent.body;
    addPin("pin flag", x, y, `${agent.id} · ${agent.activity}`, { dy: -22, color: agent.color });
  }
  for (const question of view.questions) {
    if (!question.at) continue;
    addPin("pin question", question.at[0], question.at[1], `${label("question", question.id)} ${question.text}`, { dy: -14 });
  }
  for (const clash of view.clashes) {
    if (!clash.at) continue;
    const el = addPin(`pin clash ${clash.state === "open" || clash.state === "proposed" ? "" : "resolved"}`, clash.at[0], clash.at[1], clash.label, { diamond: true });
    if (clash.id === selectedClash) el.classList.add("selected");
    el.addEventListener("click", () => {
      selectedClash = clash.id;
      note = "";
      paint();
      requestRender();
    });
  }
}

function addPin(className, wx, wy, text, options = {}) {
  const el = document.createElement("div");
  el.className = className.trim();
  if (options.diamond) {
    const diamond = document.createElement("span");
    diamond.className = "diamond";
    el.append(diamond);
  }
  const span = document.createElement("span");
  span.className = options.diamond ? "text" : "";
  span.textContent = text;
  el.append(span);
  if (options.color) el.style.color = options.color;
  pins.append(el);
  placed.push({ el, wx, wy, dx: options.dx ?? 0, dy: options.dy ?? 0 });
  return el;
}

function placePins() {
  for (const pin of placed) {
    const [x, y] = app.world_to_screen(pin.wx, pin.wy);
    pin.el.style.transform = `translate(${x + pin.dx}px, ${y + pin.dy}px)`;
  }
}

function currentClash() {
  if (!view) return null;
  return view.clashes.find((clash) => clash.id === selectedClash) ?? null;
}

function paintCard() {
  const clash = currentClash();
  if (!clash) {
    card.hidden = true;
    return;
  }
  card.hidden = false;
  const unsettled = clash.state === "open" || clash.state === "proposed";
  const title = document.createElement("h2");
  title.textContent = clash.label;

  const kind = document.createElement("p");
  kind.className = "muted";
  kind.textContent = `${clash.kind.replace(/_/g, " ")} between ${label("task", clash.a)} and ${label("task", clash.b)}`;

  const body = document.createElement("p");
  body.textContent = clash.consequence ?? "explaining the overlap";
  if (clash.consequence === null) body.className = "muted";

  const children = [title, kind, body];
  if (clash.breaks) {
    const breaks = document.createElement("p");
    breaks.textContent = `breaks ${clash.breaks}`;
    children.push(breaks);
  }

  if (unsettled) {
    const list = document.createElement("ol");
    clash.options.forEach((option) => {
      const item = document.createElement("li");
      const key = document.createElement("span");
      key.className = "key";
      key.textContent = String(option.key);
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = option.label;
      button.disabled = settling;
      button.addEventListener("click", () => settle(clash, option));
      item.append(key, button);
      list.append(item);
    });
    children.push(list);
    const hint = document.createElement("p");
    hint.className = "hint";
    hint.textContent = "press 1, 2 or 3 to settle";
    children.push(hint);
  } else {
    const resolved = document.createElement("p");
    resolved.textContent = clash.resolved ?? clash.state;
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

function renderUsage(rows) {
  if (rows.length === 0) {
    const empty = document.createElement("tr");
    const cell = document.createElement("td");
    cell.className = "muted";
    cell.colSpan = 6;
    cell.textContent = "no model calls yet";
    empty.append(cell);
    usageBody.replaceChildren(empty);
    return;
  }
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

  const endDrag = () => {
    drag = null;
    canvas.classList.remove("dragging");
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
      const clash = currentClash();
      if (!clash || (clash.state !== "open" && clash.state !== "proposed")) return;
      const option = clash.options[Number(event.key) - 1];
      if (option) settle(clash, option);
      return;
    }
    switch (event.key) {
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
