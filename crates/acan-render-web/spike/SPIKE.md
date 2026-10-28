# Renderer spike: wgpu to wasm on Windows

This spike tests whether Acan's plant canvas can ship as one Rust `wgpu` renderer compiled to wasm, with panels and labels in plain HTML. The renderer runs on WebGPU and falls back to WebGL2. It was tested in headed Chrome on one Windows laptop.

Verdict: ship it. Both backends render 300 machines and 600 belts above 60 fps at the median. They start in about 0.3 s and draw nothing while idle. The details and the open gaps follow.

## What was built

- `crates/acan-render` is the renderer. It holds a `Scene` of rects (machines) and polylines (belts), a `Camera` with pan, anchored zoom, fit, `world_to_screen` and `screen_to_world`, and a `Renderer` that takes any `wgpu::Surface`. Machines are drawn as one instanced draw call. Belts are triangulated into one vertex buffer and drawn in a second call. The renderer never learns what owns the surface.
- `crates/acan-render-web` is the browser host. It mounts the renderer on a `<canvas>` through `wasm-bindgen` and exposes a small API to plain JavaScript. `www/main.js` renders only when something changes and places the HTML labels from `world_to_screen` in the same frame.

## How to build and run

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129
pwsh -File crates/acan-render-web/build.ps1
cargo run -p acan-server -- serve --repo <repo> --web crates/acan-render-web/www
```

`acan serve --web` serves `www/` at `/` next to the API, so the page needs no JavaScript runtime. The server prints an owner token; open `http://127.0.0.1:7311/#token=<token>` for the live origin, or `http://127.0.0.1:7311/?plant=fixture` for the demo plant with no log.

`build.ps1` runs `cargo build -p acan-render-web --release --target wasm32-unknown-unknown` with `opt-level=s`, LTO, one codegen unit and `panic=abort`, set through `CARGO_PROFILE_RELEASE_*` variables. Then it runs `wasm-bindgen --target web` into `www/pkg` and prints the raw and gzip sizes.

Add `?backend=webgpu` or `?backend=webgl` to choose the backend. Add `&machines=N&belts=M` to change the scene size. Benchmarks run from the console: `runPanBench(10)`, `runRafBaseline(10)`, and `measureIdle(5)`. `window.spike.coldStart` holds the startup timestamps.

## Test machine

- Chrome 156, headed, launched by `agent-browser --headed --webgpu`.
- The GPU in use is Intel UHD Graphics (gen-12lp). The laptop also has an RTX 3050, but Chrome picked the integrated GPU.
- The display runs at 144 Hz. An empty `requestAnimationFrame` loop measured a p50 of 6.9 ms and a p95 of 7.2 ms.
- Host CPU load read 62 to 100 percent during the runs (`Win32_Processor.LoadPercentage`), because other builds were running on the same machine. This is the main source of noise in row 4.

## Pass/fail table

| # | Check | Result | Numbers | What limits it |
|---|---|---|---|---|
| 1 | Builds for `wasm32-unknown-unknown` | Pass | 3,123,370 bytes raw, 1,035,780 bytes gzip | Inferred, not measured: `wgpu-core`, `wgpu-hal` GL and `naga` (WGSL parser plus GLSL output) are compiled in for the WebGL2 tier. A WebGPU-only build should be much smaller, but it was not built. `wasm-opt` was not run because it is not installed. |
| 2 | WebGPU backend in headed Chrome | Pass | Adapter `BrowserWebGpu`, surface `Bgra8Unorm`, Intel gen-12lp | Chrome reports an empty adapter name and description, so only vendor and architecture are known. |
| 3 | WebGL2 with WebGPU forced off (`webgl` feature) | Pass | `WebGL 2.0 (OpenGL ES 3.0 Chromium)`, ANGLE D3D11 on Intel UHD, surface `Rgba8Unorm` | Same scene and same code path; only `wgpu::Backends` differs. |
| 4 | 300 machines and 600 belts, frame time while panning for 10 s | Pass at p50, noisy p95 | WebGPU: interval p50 7.0 to 13.9 ms, p95 14.0 to 48.8 ms over 6 runs. WebGL2: p50 14.1 to 20.8 ms, p95 27.4 to 27.6 ms over 3 runs. Renderer CPU time per frame: WebGPU p50 0.7 to 1.5 ms, WebGL2 p50 1.8 to 2.6 ms. | It is not the scene. 10 machines and 10 belts, and 2,000 machines and 4,000 belts, gave the same intervals as 300 and 600 on both backends (see below). The p95 spread follows host CPU load from other builds. WebGL2 lands on every second vsync at 144 Hz. Inferred cause: the GL path costs more per frame, measured at about 2 ms of CPU time against 0.7 ms on WebGPU, plus Chrome's GPU-process work. CPU time covers only the `render()` call. GPU time was not measured. |
| 5 | Pan and zoom by mouse and keyboard, with HTML labels glued to 3 machines | Pass | A mouse drag of (-80, -60) px moved all three labels by exactly (-80, -60) px. Wheel zoom stays anchored at the pointer. The arrow keys and `-` pan and zoom. Screenshots: `interact-1-wheel-zoom.png`, `interact-2-mouse-drag.png`, `interact-3-keyboard.png`, `webgpu-fit.png`, `webgl2-fit.png`. | Labels are placed in the same frame as the draw, so they cannot lag the canvas. Labels on machines outside the viewport are clipped, not hidden. |
| 6 | Cold start to first frame | Pass | Navigation start to first presented frame, over 3 reloads: WebGPU 259 to 331 ms, WebGL2 316 to 422 ms. On WebGPU, wasm fetch and compile took 68 to 86 ms and `Renderer::new` (adapter, device and pipelines) took 132 to 193 ms. | The local server sends the 3.1 MB wasm with `no-store`, so every reload fetches and compiles it again. The very first launch, including browser startup, took 5,530 ms. |
| 7 | Idle cost | Pass | 5 s with no input after a pan benchmark: 0 `requestAnimationFrame` callbacks and 0 frames on both backends. | Frames are scheduled only by input, resize, or the benchmark. GPU and CPU process usage while idle was not measured. |

### Row 4 runs

Each run lasts 10 s. Interval means the time between `requestAnimationFrame` timestamps. CPU means the time spent in `App::render`. All values are in ms.

| Backend | Scene | Frames | Interval p50 | Interval p95 | Interval max | CPU p50 | CPU p95 |
|---|---|---|---|---|---|---|---|
| WebGPU | 300 / 600 | 587 | 13.9 | 41.6 | 201 | 1.5 | 11.9 |
| WebGPU | 300 / 600 | 849 | 7.1 | 21.0 | 111 | 0.9 | 3.2 |
| WebGPU | 300 / 600 | 449 | 13.8 | 48.8 | 1202 | 1.1 | 4.1 |
| WebGPU | 300 / 600 | 828 | 13.7 | 20.7 | 305 | 0.8 | 2.2 |
| WebGPU | 300 / 600 | 1077 | 7.0 | 14.0 | 28 | 0.7 | 1.6 |
| WebGPU | 300 / 600 | 910 | 7.2 | 20.9 | 83 | 0.7 | 1.8 |
| WebGL2 | 300 / 600 | 594 | 14.1 | 27.5 | 42 | 2.6 | 5.5 |
| WebGL2 | 300 / 600 | 503 | 20.8 | 27.6 | 35 | 1.8 | 3.8 |
| WebGL2 | 300 / 600 | 621 | 14.1 | 27.4 | 70 | 2.4 | 5.8 |
| WebGPU | 10 / 10 | 1034 | 7.1 | 14.1 | 21 | 0.7 | 1.5 |
| WebGL2 | 10 / 10 | 816 | 13.8 | 20.8 | 104 | 2.2 | 4.1 |
| WebGPU | 2000 / 4000 | 918 | 7.1 | 20.8 | 84 | 1.0 | 5.7 |
| WebGL2 | 2000 / 4000 | 772 | 13.8 | 20.8 | 28 | 2.4 | 4.3 |

## Recommendation

Ship A1 as amended by Q14: one `wgpu` renderer crate with a WebGPU tier and a WebGL2 tier. Both tiers passed every row on this machine with the same code, and only the backend flag differs. Even on a loaded host and an integrated GPU, the median frame stays under the 16.7 ms that 60 Hz needs. Going from 10 machines to 2,000 machines did not change frame time, so the plant size is far from the limit. WebGL2 renders at half the WebGPU rate on a 144 Hz display but stays above 60 fps at the median, so "WebGL2 only" would give up WebGPU's headroom for no gain. A Canvas2D fallback is not needed. The real cost is the 1.0 MB gzip wasm. That is fine for a desktop app that loads it from disk, but worth cutting before a web viewer ships.

## Not covered

- Only Chrome was tested. WebView2 inside Tauri, Linux WebKitGTK on WebGL2, and macOS were not tested.
- The p95 frame times come from a host at 62 to 100 percent CPU load, on the integrated GPU. A rerun on an idle host is needed before checking A10's 8 ms p99 target at 2,000 machines.
- GPU time per frame was not measured. That would need timestamp queries.
- A WebGPU-only build size and a `wasm-opt -Oz` pass were not measured.
