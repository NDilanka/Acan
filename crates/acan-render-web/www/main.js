// The Acan plant page. The wasm App draws; this file owns the canvas size, input, the HTML
// labels and the token. A frame is drawn only after input, resize, new data or while a turn
// eases. An idle page asks for no frames.
import init, { App } from './pkg/acan_render_web.js';

const PAN_KEY_PX = 80;
const ZOOM_KEY = 1.25;
const WHEEL_RATE = 0.0015;
const TURN_DEG = 45;
const TOKEN_KEY = 'acan.token';
const LABEL_OFFSET_PX = 6;
const LABEL_PAD_PX = 4;

const params = new URLSearchParams(location.search);
const canvas = document.getElementById('plant');
const labelsEl = document.getElementById('labels');
const repoEl = document.getElementById('repo');
const noticeEl = document.getElementById('notice');

const timing = { started: performance.now() };
let app = null;
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
  const turning = app.advance(elapsed);
  if (!app.render()) requestRender();
  if (timing.firstFrame === undefined) timing.firstFrame = performance.now();
  placeLabels();
  if (turning) {
    requestRender();
  } else {
    lastFrame = null;
  }
}

// Labels come in priority order, larger and nearer first. A label that would overlap one
// already placed stays hidden, so the plant never shows a pile of names.
function placeLabels() {
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  labelGeneration += 1;
  const quads = app.label_layout(altDown);
  const placed = [];
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
    const clear = inside && placed.every((other) => !overlaps(box, other));
    if (clear) placed.push(box);
    el.style.visibility = clear ? 'visible' : 'hidden';
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
  const el = document.createElement('div');
  el.className = 'label';
  el.textContent = app.machine_name(index);
  const files = app.machine_files(index);
  const count = document.createElement('span');
  count.className = 'count';
  count.textContent = `${files} ${files === 1 ? 'file' : 'files'}`;
  el.appendChild(count);
  return el;
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

// The token arrives as `#token=` from `acan serve`. It is kept, then the fragment is removed
// so it does not sit in the address bar or the history.
function takeToken() {
  const match = /(?:^|[#&])token=([^&]+)/.exec(location.hash);
  if (match) {
    const token = decodeURIComponent(match[1]);
    localStorage.setItem(TOKEN_KEY, token);
    history.replaceState(null, '', location.pathname + location.search);
    return token;
  }
  return localStorage.getItem(TOKEN_KEY);
}

async function loadLive() {
  const token = takeToken();
  if (!token) {
    showNotice('No token · run acan serve');
    return;
  }
  const response = await fetch('/api/plant', {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!response.ok) {
    showNotice(`Could not load the plant (HTTP ${response.status})`);
    return;
  }
  app.set_plant(await response.text());
}

async function loadPlant() {
  const source = params.get('plant');
  if (source === 'stress') {
    const machines = Number(params.get('machines') || 2000);
    const started = performance.now();
    app.set_stress(machines);
    timing.load = performance.now() - started;
  } else if (source === 'fixture') {
    const response = await fetch('fixtures/demo-plant.json');
    if (!response.ok) throw new Error(`fixture HTTP ${response.status}`);
    app.set_plant(await response.text());
  } else {
    await loadLive();
  }
  repoEl.textContent = app.repo_name();
}

async function boot() {
  const started = performance.now();
  await init();
  timing.init = performance.now() - started;
  setDrawingSize();
  const createStarted = performance.now();
  app = await App.create(canvas, params.get('backend') || 'webgpu');
  timing.create = performance.now() - createStarted;
  const { rect, dpr } = setDrawingSize();
  app.resize(rect.width, rect.height, dpr);
  await loadPlant();
  requestRender();
  if (params.get('bench') === '1') {
    const { installBench } = await import('./bench.js');
    installBench({ App, app, canvas, timing, redraw: drawFrame, setAlt });
  }
}

window.addEventListener('resize', sizeCanvas);

// Left drag moves the ground under the pointer, so the point you grab stays under the cursor.
// Right drag, or Alt with left drag, orbits and tilts about the ground point under the screen
// centre.
canvas.addEventListener('contextmenu', (event) => event.preventDefault());

canvas.addEventListener('pointerdown', (event) => {
  if (app === null || (event.button !== 0 && event.button !== 2)) return;
  canvas.setPointerCapture(event.pointerId);
  drag = {
    x: event.clientX,
    y: event.clientY,
    orbit: event.button === 2 || event.altKey,
  };
  canvas.classList.add('dragging');
});

canvas.addEventListener('pointermove', (event) => {
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

function endDrag() {
  drag = null;
  canvas.classList.remove('dragging');
}
canvas.addEventListener('pointerup', endDrag);
canvas.addEventListener('pointercancel', endDrag);

canvas.addEventListener(
  'wheel',
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

const ARROW_PAN = {
  ArrowLeft: [PAN_KEY_PX, 0],
  ArrowRight: [-PAN_KEY_PX, 0],
  ArrowUp: [0, PAN_KEY_PX],
  ArrowDown: [0, -PAN_KEY_PX],
};

function setAlt(down) {
  if (altDown === down) return;
  altDown = down;
  labelsEl.classList.toggle('alt', down);
  requestRender();
}

window.addEventListener('keydown', (event) => {
  if (event.key === 'Alt') {
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
  if (event.key === '5' || event.code === 'Numpad5') {
    if (!event.repeat) {
      app.toggle_projection();
      requestRender();
    }
    return;
  }
  switch (event.key) {
    case '+':
    case '=':
      zoomAtCentre(ZOOM_KEY);
      break;
    case '-':
    case '_':
      zoomAtCentre(1 / ZOOM_KEY);
      break;
    case '0':
      app.fit();
      requestRender();
      break;
    case 'q':
    case 'Q':
      if (!event.repeat) turn(-TURN_DEG);
      break;
    case 'e':
    case 'E':
      if (!event.repeat) turn(TURN_DEG);
      break;
    default:
  }
});

window.addEventListener('keyup', (event) => {
  if (event.key === 'Alt') setAlt(false);
});

window.addEventListener('blur', () => setAlt(false));

boot().catch((error) => {
  console.error(error);
  showNotice(String(error?.message ?? error));
});
