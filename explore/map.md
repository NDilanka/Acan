# Acan: Map of Unknowns

> A working model of your system.

Status: **Walk complete (all four quadrants).** Handed over on 2026-10-13. Nothing is OPEN that blocks the spec. The items to confirm before coding are listed in "Before coding" at the end.

How to read it: entries are kept as decided, and later amendments are marked inline with **→ amended by …**. The current answer is always the latest amendment. The end of the file holds the handover: the before-coding facts, a tweakable build plan, and the prompt to paste next.
Method: explore-unknowns quadrant walk (https://github.com/dzhng/skills/blob/main/skills/engineering/explore-unknowns/SKILL.md), feeding write-spec for v0.1.

## Territory

- Greenfield. The workspace (`/home/factory-user`) is empty and is not a git repository. No prior code or attempts exist.
- References read during the brainstorm:
  - anatomy (https://github.com/wheresryan22/anatomy): isometric SVG + WebGL figures. Load-bearing rule: "Every number the figure shows comes from the same small model that moves its parts." Verifies its own output with a geometry audit and headless-Chrome screenshots.
  - dzhng/skills (https://github.com/dzhng/skills): "fog of war" framing, territories as independently verifiable slices, a `choices.md` ledger ranked least-confident first, a renderer skill with "one frame orchestrator, single owners, a resource registry, depth and compositing contracts, and verification that looks at pixels."
  - The "delete layers of abstraction" post: Rust compiled to wasm, HTML and raw WebGPU, custom validation harnesses that let agents hillclimb.

---

## Quadrant 1: Known knowns

### Locked (decided by the user)

| # | Decision | Why |
|---|---|---|
| K1 | The product is **Acan**, "a working model of your system". | Distinctive, accurate: a precise moving model of a system. |
| K2 | Primary users: solo developers running many agents, and tech leads of small teams. Positioned as a professional tool for senior engineers who work like architects. | Stated by the user. |
| K3 | One canvas combines three modes of work: observe (live agents and system), analyze (delivery bottlenecks), design (blueprints built by agents). | The user wants all three, plus a live preview of the system being built. |
| K4 | The drawing merges three sources from day one: **designed** (blueprint), **declared** (code and infra files), **observed** (runtime telemetry). Disagreements between them are shown as drift. | Chosen over a single source of truth. |
| K5 | Layout is authored by the architect. Acan suggests a first draft. New entities land in an **unplaced tray** instead of being auto-placed. | Stable, meaningful layouts; the layout is the architect's mental model. |
| K6 | Visual vocabulary: a **fixed machine catalog** at distance, plus **custom anatomy-style metaphors** for close zoom, cutaways, Inspect mode and promoted hero components. Custom machines must declare ports, standard states and metric bindings. | Readable at scale like Factorio, expressive up close like anatomy. |
| K7 | v0.1 catalog: **15 machines, 6 fittings, 4 Floor entities** as listed below. | Accepted as is. |
| K8 | Rate limiter, circuit breaker, retries, timeout, validation and budget probes are **fittings on belts**, not machines. | They act on flows, not beside them. |
| K9 | Desktop app, **local first**. | Privacy, speed, no account needed. |
| K10 | Engineering philosophy: delete abstraction layers on hot paths (rendering, ingestion, queries, simulation). Rust across the codebase. | The user's performance goal. |
| K11 | **Budgets are the agents' hillclimbing objectives.** Architects set them on the drawing; agents read them and iterate. | Core idea, confirmed. |
| K12 | **Fog of war, territories and choices** are first-class layers. | Core idea, confirmed. |
| K13 | **Calibrated simulation**: each machine shows how well past predictions matched reality. | Core idea, confirmed. |
| K14 | **Open file format** for blueprints and machine definitions, plus **drawing diffs on PRs**. **→ amended by S3:** PR drawing diffs move to v0.2; v0.1 ships share-safe exports. | Core idea, confirmed. |
| K15 | **The paper changes with the mode**: vellum (Live), cyanotype (Blueprint), graph paper (Simulate), sepia (Replay). **→ amended by S2:** dark versions of all four (graphite, deep cyanotype, dark graph, dark sepia) with shared brass accents. | Confirmed. |
| K16 | **Agents can never edit budgets or checks** without human approval. | Prevents agents from moving the target they are graded against. |
| K17 | Choices are reviewed in order of **risk = (1 − confidence) × reversal cost**. | Confirmed. |
| K18 | The first sample system is **Acan itself**. | Dogfooding and the launch demo. |
| K19 | Blueprint lives as **YAML in the repo** (`.acan/`). | The user confirmed this direction. |
| K20 | Process: explore-unknowns walk, then write-spec, then a long autonomous build against harnesses. | Chosen by the user. |

### Settled unless the user says otherwise (proposed by the agent, not explicitly confirmed)

| # | Assumption |
|---|---|
| A1 | Canvas architecture: raw WebGPU in the system webview (Tauri), renderer in Rust on `wgpu` compiled to wasm, panels in plain HTML, native Rust core. The renderer does not know its host, so a native-window host can be added later. Final decision is made by the week-one spike's pass/fail table. **→ amended by Q14:** one wgpu renderer crate with two capability tiers, WebGPU (Windows, macOS 26+) and WebGL2 (Linux WebKitGTK, older macOS). |
| A2 | Core sends compact **binary deltas of rates**, not raw events, to the renderer. Same protocol over IPC (desktop) and WebSocket (web viewer). |
| A3 | Built-in OpenTelemetry (OTLP) receiver on `localhost:4317`. **→ amended by E2 and E5:** OTLP/HTTP on 4318 comes first and gRPC on 4317 follows. Loopback only, with exclusive bind, conflict detection and configurable ports. |
| A4 | Renderer: real 3D scene, orthographic cameras per view, four-tone flat shading, edges as real geometry, GPU text (MSDF) for in-canvas labels, ID-buffer picking, drivable clock for deterministic screenshots, SVG export. |
| A5 | Visual identity: brass-and-vellum palette, drafting line weights, title block with git revision, IBM Plex Sans and Mono, mechanical easing, "ink-in" ghost-to-solid moment. To be tested with mockups in Stage 3. **→ amended by U4, S1, S2, R1:** direction D, brass instruments on dark paper. Catalog silhouettes with mounted gauges. Plan view flat in every mode. Reference: `acan/explore/stage3-refined.html`. |
| A6 | MCP server as proposed: resources (`system`, `entity`, `territory`), query/design/build/session tools, notifications, three permission levels. Non-MCP agents are detected passively from session logs and hooks. **→ amended by E1:** passive observation goes through each agent's native OTel export first, then opt-in hooks, then transcripts. |
| A7 | Fog score from six signals: observed, tested, owned, designed, understood, matched. Fog grows back with code churn. |
| A8 | Territory and choice file schemas as proposed in the brainstorm. |
| A9 | Business model: free local app; Pro (about $25/month); Team (about $50/seat/month) with web viewer and PR bot. Bring your own agents and API keys. No per-gigabyte fees. Prices are guesses to test. |
| A10 | Performance targets: 8 ms p99 frame at 2,000 machines; under 100 ms telemetry-to-pixel; 100k spans/s on under one core; under 1 s to open a cached project; under 500 MB memory with 1 hour of telemetry. **→ extended by U2 and Q15:** belts animate at most 20 fps and the canvas goes still after 2 minutes idle. Idle CPU/GPU budget and a 24-hour soak with flat memory, including the wasm high-water mark. |

### v0.1 catalog (K7)

Machines: Intake (clients), Splitter (load balancer / gateway), Gate (auth), Assembler (service), Assembler bank (worker pool), Fold-out cell (serverless), Hopper + conveyor (queue), Pneumatic exchange (pub/sub), Tank (relational database), Buffer rack (cache), Silo (object storage), Card catalog (search / vector index), Kiln (LLM call), Dock (external API), Cam clock (scheduler).

Fittings: Valve (rate limiter), Breaker (circuit breaker), Return loop (retries), Fuse (timeout), Strainer (validation), Meter (budget probe).

Floor: Construction bot (agent session), Scaffolding (open PR or branch), Crane (CI), Inspection station (human reviewer).

Universal states: idle, working, strained, failing, ghost, dark, fogged.

---

## Quadrant 2: Known unknowns

Ordered by architectural blast radius. Each closes as answered by the user, answered by the territory, or recorded as OPEN.

| # | Question | Status |
|---|---|---|
| Q1 | What is the scope boundary of v0.1? | Closed (D1) |
| Q2 | Acan is a single desktop process, so drawing itself exercises in-process machines, not the service catalog. What second fixture is needed? | Closed (D2) |
| Q3 | Which operating systems must v0.1 support? | Closed (D3) |
| Q4 | Who builds it, and on what timeline? | Closed (D4) |
| Q5 | How much simulation is in v0.1? | Closed (D5) |
| Q6 | Which languages does code analysis support in v0.1? | Closed (D6, agent) |
| Q7 | Which infra sources does v0.1 read? | Closed (D7, agent) |
| Q8 | Which agents are supported in v0.1, and how (MCP or passive)? | Closed (D8, agent; fact closed in Stage 4 by E1) |
| Q9 | Telemetry storage: embedded DuckDB or a custom columnar store? | Closed (D9, agent) |
| Q10 | How are entities matched across sources in v0.1? | Closed (D10, agent) |
| Q11 | Blueprint schema details: versioning, and how the app and humans both edit the same file safely. | Closed (D11, agent; fact closed in Stage 4 by E3) |
| Q12 | Licensing: is the app open source, or only the format? | Closed (D12) |
| Q13 | How does Acquit get OpenTelemetry instrumentation, given its minimal-dependency style? (Raised by L3.) | Closed (D13) |

**Stage 2 closed.** All 13 questions closed in front of the user. The facts left OPEN inside decisions were closed in Stage 4: agent log and hook formats (D8, see E1), comment-preserving YAML editing in Rust (D11, see E3), and Acquit's SQLite driver and Node 24 diagnostic channels (D13, see L16 and E4).

### Landmine disclosed in Stage 1

**L1 (filed under Q2):** Acan drawing itself is a strong demo, but Acan is one desktop process made of threads, channels and embedded stores. Most of the catalog (Splitter, Gate, Tank, Buffer rack, Hopper as a queue service, Pneumatic exchange) would never appear. Dogfooding alone would not test the service-level view that most users need.

---

## Quadrant 3: Unknown knowns

Complete.

- Artifact: `acan/explore/stage3-directions.html` (self-contained, no external requests). It draws the same Acquit plant (structure inferred from its README at rev 55825fe; numbers invented) four ways: **A** drafting plan on vellum with a paper switcher for the four modes, **B** factory floor with 3/4 extrusion, **C** cyanotype schematic with a parts list, **D** brass instruments. Every state appears: working, strained (verifier), failing (PayPal, with the retry loop), idle (sweep), ghost (OTel radar), fogged (`packages/house`). Steal/skip chips and A/B questions assemble the user's reply.
- Screenshot: `acan/explore/shots/stage3-full.png`.
- Context probes asked: who consumes the drawing, where it runs day to day, what "done" means for v0.1.

### Extracted (answered by the user)

| # | Fact | What it reshapes |
|---|---|---|
| U1 | **Audience: everyone.** Teammates reviewing PRs, clients and stakeholders (for example Acquit's customers), agents through MCP, and the public through screenshots, demos and social posts. | Drawings leave the machine, so **sharing must be safe**: secrets, internal hostnames, env var names and customer data must be redactable before export. D1 left exports and PR diffs out of v0.1; that needs revisiting (S3). |
| U2 | **Always open on a second monitor.** | Idle cost matters: render on demand, very low CPU/GPU when nothing changes, no memory growth over days. Signals must be readable from a distance and noticeable in peripheral vision. Adds a multi-day soak harness and an idle-power harness. |
| U3 | **v0.1 is done when there is a launch video and a public waitlist.** | v0.1 does not require outside users to install it. That questions D3 (three operating systems for v0.1) and points the week-6 and week-12 milestones at a cinematic demo (S4). |
| U4 | **First reaction: direction D, brass instruments.** | A5 (vellum-first identity) and K15 (light papers per mode) were designed for light paper, so they need dark equivalents (S2). In D every machine is a round gauge, which conflicts with the catalog rule of a unique silhouette per machine type (K6/K7) (S1). |

### Follow-ups raised by U1 to U4

- S1: How do D's gauges coexist with distinct machine silhouettes?
- S2: What does each mode's paper look like in a dark, brass-first identity?
- S3: Does v0.1 include share-safe exports (and PR drawing diffs)?
- S4: Given "done = launch video + waitlist", keep three operating systems for v0.1?

| # | Decision | Closed by |
|---|---|---|
| S1 | **Every machine keeps its unique catalog silhouette**, drawn in brass and graphite, **with a gauge mounted on it** as its primary dial. Ghosts carry an empty dashed bezel. | User |
| S2 | **Dark versions of all four mode papers:** graphite (Live), deep cyanotype (Blueprint), dark graph paper (Simulate), dark sepia (Replay). Brass accents are shared by all four. This replaces the light-first assumption in A5 and refines K15. | User |
| S3 | **Share-safe export in v0.1:** PNG, SVG and video frames, with a redaction layer for secrets, env var names, internal paths, hostnames, ports and customer data. **PR drawing diffs move to v0.2.** This amends D1. | User |
| S4 | **Polish on the user's daily OS first. The other two operating systems must pass the week-one spike and build and run in CI**, but are not polished for v0.1. This amends D3. **OPEN fact:** which OS is the user's daily OS. **→ closed by R3: Windows.** | User |

### Refined mockup (direction D after S1 to S4)

- Artifact: `acan/explore/stage3-refined.html`. The Acquit plant with catalog silhouettes plus mounted gauges, all four dark papers, a flat or 3/4 depth toggle, a share-safe redaction toggle (labels, paths, ports, key names and the revision become generic and are counted in the title block), and a motion toggle. URL hash parameters select a variant, for example `#paper=blueprint&depth=34&redact=1`.
- Screenshots: `acan/explore/shots/refined-default.png` (Live, flat), `refined-blue.png` (Blueprint, 3/4, redacted), `refined-sim.png` (Simulate, 3/4).
- Layout rules learned while fixing overlaps (carry into the renderer's label placement): a dial reading sits to the left of its mounted gauge, right-aligned, never centered over it; a failing machine's reading lifts above its smoke; the title block is never placed over a label; ghost connectors route around labels; boundary labels sit at the top of the boundary line.
- Reaction (user): silhouettes with mounted gauges **read right**; **flat everywhere** by default (3/4 stays a drawing-set view, not the plan default); redaction hides **the right amount** (names, paths, ports, key names, revision; numbers stay); **daily OS is Windows**.

| # | Decision | Closed by |
|---|---|---|
| R1 | Plan view is **flat in every mode**. 3/4 extrusion belongs to the iso and exploded drawing-set views. | User |
| R2 | The default share-safe redaction set is names, paths, ports, env var and key names, hostnames and the revision. Numbers are kept. | User |
| R3 | **Windows is the polish target for v0.1** (closes the S4 open fact). The renderer spike must pass on WebView2 first. macOS (WKWebView) and Linux (WebKitGTK) must pass the spike and CI, but are not polished. See L6. | User |

Stage 3 complete.

## Quadrant 4: Unknown unknowns

Complete. Acan has no code yet, so the sweep covered what v0.1 stands on:
1. **The Acquit codebase** at rev 55825fe: a full clone of all 558 commits at `/tmp/acquit-sweep`, read without running anything.
2. **The renderer and desktop platform:** WebView2, WKWebView, WebKitGTK, Tauri 2, GPU CI, and Windows distribution.
3. **The agent sources and the blueprint format:** Droid, Claude Code and Codex logs, hooks and OTel export, plus comment-preserving YAML crates in Rust.

Evidence is cited by file:line or URL. [V] means verified, [I] means inferred.

### Landmine cards (worst first)

**Platform and renderer**

| # | Finding | Evidence | Why it bites | What it changes |
|---|---|---|---|---|
| L6 | **WebKitGTK has no WebGPU, and nobody is building it.** WKWebView has WebGPU only on **macOS 26+**, with no flag for older versions. WebView2 on Windows x64 has it on by default; on ARM64 it is still behind a flag. | gpuweb Implementation-Status wiki (Oct 2026); tauri#6381; Apple forum 770862; WebKitGTK 2.54 notes | The "raw WebGPU in the webview" path (K-level choice) cannot run on Linux or on macOS before 26. A native wgpu layer under the webview flickers on Linux and was closed as not planned (tauri#9220). | Needs a decision: capability tiers (see Q14). |
| L7 | **An always-on window conflicts with animated belts.** WebView2 rAF throttling cannot be configured (WebView2Feedback#1172). The Tauri `backgroundThrottling` setting works only on macOS. A window visible on a second monitor is never occluded, so a continuous loop runs for days. VS Code saw 13% idle CPU from one CSS animation. | Sources above; VS Code#22900 | U2 (always open) against the Factorio-style moving items. | Needs a decision: motion budget (Q15). In any case the side panels have no CSS animation, and ingest lives in the Rust core so webview suspension never loses data. |
| L8 | **Device loss over days is certain.** Driver resets, sleep and wake, and monitor hot-plugs all lose the device. Chrome blocks a page after a 2nd GPU crash within 2 minutes. An open Chromium bug (557281760) loses the device when a worker-owned OffscreenCanvas is resized. | toji.dev device-loss guide | A black canvas on the second monitor. | Handle `device.lost` by rebuilding from core state. Keep the canvas on the main thread with 1 frame in flight. Show a reload screen after the second loss. Add a device-loss injection harness. |
| L9 | **OTLP ports 4317 and 4318 are shared with other tools.** Aspire Dashboard, the OTel Collector and Jaeger all default to them. On Windows, a specific-address bind can succeed beside a wildcard holder and silently split traffic. Binding 0.0.0.0 triggers a firewall prompt; 127.0.0.1 does not. | aspire.dev; Microsoft SO_EXCLUSIVEADDRUSE docs; SO 40492842 | Partial telemetry with no error. | Bind 127.0.0.1 and ::1 with SO_EXCLUSIVEADDRUSE. Detect conflicts and show them. Make the ports configurable and expose the endpoint through MCP. Remote OTLP is opt-in only. |
| L10 | **Windows signing.** Azure Artifact Signing costs $9.99/month and is open to individuals in the US or Canada, or to organizations in the US, Canada, EU or UK. EV certificates no longer skip SmartScreen. Smart App Control blocks unsigned binaries. | Microsoft docs; itwriting.com | Waitlist users get warnings or blocks. | Sharp edge for v0.2 (outside users install it). v0.1 is done at the launch video and waitlist (U3), so it needs no signing. **OPEN fact:** eligibility (Q17). |
| L11 | **Long-running staleness.** A running WebView2 keeps the old runtime after an Evergreen update. Wasm linear memory never shrinks. | Microsoft WebView2 distribution docs; WebAssembly/design#1397 | Memory creep across a multi-day soak. | Bound renderer buffers and keep history in the core. Track the wasm memory high-water mark in the soak harness. Prompt a restart on `NewBrowserVersionAvailable`. |
| L12 | **DPI and text.** `devicePixelContentBoxSize` is missing in Safari 26. Text renders differently in each webview. | caniuse | Blurry canvases when a window moves between monitors; golden images that differ by platform. | Fall back to `devicePixelRatio`. Draw canvas text from Acan's own glyph atlas (IBM Plex) so goldens are deterministic. No transparent windows. |
| L4 | **GPU CI (closed by plan).** GitHub GPU runners are organization-only and cost $0.052 to $0.102 per minute. | GitHub billing docs; wgpu CI | — | **Plan:** Tier 1, every PR: native wgpu golden tests on WARP (Windows), lavapipe (Linux) and Metal (macOS-14), compared with nv-flip. Tier 2, every PR: Chromium WebGPU smoke test on SwiftShader under Xvfb, checking pixel statistics because `--disable-vulkan-surface` is otherwise a false pass. Tier 3, nightly: a self-hosted runner on the user's Windows machine for frame time, idle CPU, a 24-hour soak and device-loss injection, plus a manual macOS 26 production-bundle check. |

**Acquit (fixture) and the D13 instrumentation task**

| # | Finding | Evidence | What it changes |
|---|---|---|---|
| L13 | **Trace context is lost twice.** Work goes through a database effects queue and a 30-second tick. The API and verifier then talk by HTTP, with a later callback that arrives as a new request. | `packages/core/src/effects.ts:246,402`; `acquit.ts:255-259`; `packages/verifier/service.ts:146-147,209-219`; `apps/api/src/server.ts:362-377` | D13 stores `traceparent` in each queue row and uses span links. It sends `traceparent` as a header to the verifier; this is safe because the signature covers only timestamp, nonce and body (`service.ts:125`). The verifier echoes it on the callback. Acan must draw linked spans as a belt through a tank, not as one request. |
| L14 | **Tests stub `fetch`, assert exact outbound call lists, and freeze `Date.now`.** | `packages/ctl/test/server.test.ts:20-27,56-60,211-212,252-253` | The exporter uses `node:http`, never `fetch`. It is off unless an endpoint is set. Timestamps come from `performance.timeOrigin + performance.now()`. |
| L15 | **stdout is a protocol.** MCP uses JSON-RPC over stdio, and the control CLI prints exactly one JSON object. | `packages/mcp/src/main.ts:49`; `packages/ctl/test/cli.test.ts:66` | The module never writes to stdout. Diagnostics go to stderr only. |
| L16 | **Channels cannot see sync work.** `node:sqlite` `DatabaseSync` has no diagnostics channel in Node 24.21, and `spawnSync` emits no `child_process` events [V by probe]. `fetch` fires `undici:*` channels, not `http.client.*`. | `packages/core/src/store.ts:25-28`; `packages/acquit-cli/src/run.ts:189-191`; `packages/verifier/judge.ts:32,63`, `subject.ts:178,208,240` | D13 wraps database and `spawnSync` calls by hand and adds an event-loop-delay histogram. Acan must treat built-in channel names as unstable across Node versions and cover them with a fixture test. |
| L17 | **One SQLite file per lane has many writers.** The API opens two connections. The control CLI `sweep` and `seed-db`, the seed script and tests also write. There are separate file stores: `model-calls.jsonl` and the House state and pause files. | `acquit.ts:200`; `server.ts:56`; `ctl/src/commands.ts:127,405`; `core/src/model.ts:142-148`; `apps/api/src/house.ts:44-55` | The tank has pipes from short-lived processes, not just services. Draw the extra tanks. |
| L18 | **Lanes repeat every service on offset ports** (`4310+10n`, `5173+10n`, `4311+10n`, one database each). An ownership preload requires the nonce to be the last argument. | `packages/ctl/src/state.ts:47-49`; `ctl/src/ownership-preload.cjs:10`; `process.ts:393` | Identity is role plus lane, never port. The plant is drawn once, with lanes as stacked copies. D13 loads with `--import` before the script, or with a static import. |
| L19 | **Docker is opaque.** Verifier subjects get only `PATH` and `--network none`. Runner containers reach only model hosts through an allowlist proxy. | `packages/verifier/subject.ts:163,274`; `packages/runner/allowlist.mjs:16-19` | Containers are drawn as sealed housings, timed from the host only. |
| L20 | **The "zero dependency" premise is partial.** `packages/mcp` depends on the MCP SDK and `@paypal/agent-toolkit`, and `@opentelemetry/api` 1.9.0 is already in the lockfile through `ai`. Nothing enforces no-deps, and there is no AGENTS.md, linter or CI. | `packages/mcp/package.json`; lockfile; `tsconfig.json` (`erasableSyntaxOnly`) | D13's module lives in `packages/core`, uses only erasable TypeScript syntax and `.ts` import extensions, and keeps the API and verifier dependency-free. |
| L21 | **Declared config is incomplete.** The API and verifier also load `/opt/acquit/.env` in code, outside the systemd units. | `apps/api/src/config.ts:14-15`; `packages/verifier/config.ts:11` | The systemd reader (L2) must also read the `EnvironmentFile`s and any `.env` loaded in code, or show "config also comes from code" as fog. |

**Corrections to the Acquit mapping** (supersede the "Likely mapping" paragraph above):
- **Clock:** `sweep` is a manual CLI command and nothing schedules it (`ctl/src/commands.ts:121-138`). The real clocks are two 30-second `setInterval` timers inside the API, the tick and the House loop (`server.ts:737-740`). The catalog needs a cam clock mounted inside a machine.
- **Retry loops:** "renew the token and retry once" is in the MCP process's PayPal reader (`packages/mcp/src/paypal.ts:57-73`). The API's PayPal client retries GETs three times and drops its token on a 401 (`core/src/paypal.ts:220-243`). The verifier callback retries three times.
- **Kiln:** model calls are `droid exec` (or claude-code or OpenRouter) inside Docker, started by the API's House loop and by operators' `acquit run` (`core/src/model.ts:40-42`; `acquit-cli/src/run.ts:473`). They are not HTTP calls from Node.
- **Worker bank:** the queue is in memory, concurrency is 1 in production, and runs are lost on restart (`deploy/runtime.env:12`; `service.ts:87-88`).
- **Gate:** two layers. Caddy basic auth exempts `/paypal/webhook` (`deploy/Caddyfile.template:11-14`), and app sessions sit behind it (`server.ts:100-121,274-330`).
- **Intake:** add three routes: the PayPal webhook, the PayPal return and cancel redirects, and the verifier callback.
- **Web:** in production it is static files served by Caddy (80/443, plus 127.0.0.1:8080 and admin 2019). In development it is Vite on 5173. The web app polls every 1 or 4 seconds; there are no websockets or SSE.
- **Docks:** the verifier also calls GitHub, by API and by `git` subprocess.
- **`packages/house`** is now known (fog cleared): it is the platform's built-in operator, a library driven by the API's House loop. It is not a process.

**Redaction set found in Acquit** (confirms R2 and extends it to headers, cookies and database columns):
- Secret files: `/etc/acquit/*-secrets.env`, `/opt/acquit/.env`, `/tmp/acquit-deploy*/…`, and `~/.config/acquit/…`.
- Env var names such as `PAYPAL_CLIENT_SECRET`, `ACQUIT_VERIFIER_*_SECRET`, `ACQUIT_GITHUB_APP_PRIVATE_KEY` and `FACTORY_API_KEY`.
- Headers `Authorization`, `PayPal-Auth-Assertion` and `x-acquit-*`, the `acquit_session` cookie, the `cli_codes.token` column, and the VM host name with its IP (`deploy/runtime.env:5`).

**Agent sources (D8 facts closed)**

| Agent | Transcripts | Stable? | Hooks | Native OTel |
|---|---|---|---|---|
| Droid | `~/.factory/sessions/<cwd-slug>/<uuid>.jsonl` [V on Linux]. Line types: `session_start`, `message` (Anthropic-style blocks), `todo_state`, `compaction_state`, `agent_turn_outcome`. | Undocumented. The docs' example path differs from the real path, so read `transcript_path` from the hook payload. | `hooks.json`, command type only. Droid warns on external edits; orgs can lock hooks. | Yes. OTLP over **HTTP only**, using `OTEL_TELEMETRY_ENDPOINT`. `OTEL_TELEMETRY_FORMAT=genai` gives per-turn spans with `session.id`. |
| Claude Code | `~/.claude/projects/<cwd>/<session>.jsonl`. **Deleted after 30 days by default.** | "Internal… changes between versions" (docs). | 25+ events, including **native `http` hooks** to localhost. | Yes, the richest. Protocol must be set explicitly; beta traces are available. |
| Codex CLI | `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl` | "Not a stable interface" (docs). | command or `mcp_tool` handlers. **Every hook must be trusted by hash.** | Yes, through the `[otel]` table, HTTP or gRPC. |

Sharp edges: transcripts are plaintext and contain whatever a tool read, including `.env` files, so share-safe redaction must cover transcript-derived data. A global `OTEL_EXPORTER_OTLP_ENDPOINT` would make every OTel-instrumented program report to Acan, so each agent is configured through its own setting. Parsers ignore unknown records, count them, and show a "format drift" warning.

**Blueprint YAML (D11 fact closed):** there is no battle-tested lossless YAML editor in Rust. `serde_yaml` is archived, and `serde_yml` is unsound (RUSTSEC-2025-0068). `yamlpatch` (from zizmor) does best-effort, comment-preserving path edits. `yaml-edit` is a lossless rowan tree but has one maintainer and a 0.x API. `toml_edit` is the only proven lossless option.

### Decisions taken under delegation (P3 real systems, P4 performance/craft; disclosed)

| # | Decision | Why |
|---|---|---|
| E1 | **Agents are observed through their native OTel export first.** Acan's Connect flow writes per-agent settings (Droid `OTEL_TELEMETRY_ENDPOINT` with `genai` format, Claude `env` in user settings, Codex `[otel]`). Hooks are an opt-in second source, using an `acan-hook` relay for Droid and Codex and native http hooks for Claude. Transcripts are the third source, used for backfill and replay, with drift warnings. Message content export stays off. Amends D8. | It is the only documented, stable interface of the three. Transcripts are explicitly unstable, and Claude deletes them after 30 days. |
| E2 | **The OTLP/HTTP receiver on 4318 (protobuf and JSON) is first-class in v0.1. gRPC on 4317 follows** in the same release, if the week-one spike leaves time. | Droid exports only over HTTP. The D13 module can only write OTLP/HTTP JSON without dependencies. |
| E3 | **Blueprint YAML uses a canonical subset** enforced by `acan fmt`: block style, one document per file, no anchors, aliases, tags or flow collections, and own-line or end-of-line comments. Reading uses `serde-saphyr`, app edits use `yamlpatch`, and formatting uses `pretty_yaml`. A one-day bake-off against `yaml-edit` runs on a golden round-trip corpus. TOML stays the escape hatch if the corpus fails. | Every editor is reliable on the subset, and a small custom editor stays possible. Keeps the human-friendly format chosen in D11. |
| E4 | **D13 specifics:** the module lives in `packages/core` and exports OTLP/HTTP JSON over `node:http`. It is off without an endpoint, never writes to stdout, and loads through `--import`. It propagates `traceparent` through queue rows (with span links) and through signed verifier calls. It hand-wraps `DatabaseSync` and `spawnSync`, and records event-loop delay. | L13 to L20. |
| E5 | **The receiver binds 127.0.0.1 and ::1 only**, with exclusive-address use, conflict detection and configurable ports. Ingest and history live in the Rust core. The renderer handles device loss by rebuilding from core state and draws text from its own glyph atlas. | L8, L9, L11, L12. |

### Decisions closed by the user

| # | Question | Decision | Closes |
|---|---|---|---|
| Q14 | Renderer coverage | **Two capability tiers in one wgpu-based renderer crate.** The full tier is WebGPU (Windows WebView2, macOS 26+). The downlevel tier is WebGL2 (`downlevel_webgl2_defaults`, no compute) for WebKitGTK on Linux and macOS before 26. The Linux spike passes only if it renders correctly on Intel, AMD and NVIDIA, including with `WEBKIT_DISABLE_DMABUF_RENDERER=1`, and it warns when performance suggests software rendering. If WebGL2 performance fails, the fallback is a separate native winit + wgpu canvas window on Linux, never a layer under the webview. The Tauri 3 CEF runtime is a long-term option only. Amends K-level "raw WebGPU in the webview" to "wgpu with WebGPU and WebGL2 backends." | L6 |
| Q15 | Motion budget | **Belts animate at a capped 20 fps while data flows, and the canvas goes fully still after 2 minutes without input or state change.** A state change (failure, ghost landing, budget breach) wakes it. Interaction runs at the display rate. Harness: an idle-CPU and idle-GPU budget on the Windows self-hosted runner, plus a 24-hour soak. | L7 |
| Q16 | Closed-app trust | **All three proofs ship in v0.1:** an in-app log of every outbound connection (destination, bytes, reason), a hard offline switch that blocks all egress (the OTLP and MCP listeners keep working on loopback), and a published data statement listing everything that is ever sent. The ingest and network layer stays closed. | L5 |
| Q17 | Windows signing eligibility | **Deferred to v0.2.** v0.1 ends at the launch video and the waitlist, so it needs no signed distribution. Before v0.2: confirm Azure Artifact Signing eligibility or buy a certificate, sign every binary with timestamps, and expect weeks of SmartScreen warnings. | L10 (parked) |

Stage 4 complete. Every finding is decided, parked with a trigger, or recorded as a sharp edge.

### Territory findings for Q2

From https://github.com/open-telemetry/opentelemetry-demo (read during the walk):
- The OpenTelemetry Astronomy Shop is "a microservice-based distributed system intended to illustrate the implementation of OpenTelemetry in a near real-world environment." Apache-2.0. Latest release 3.1.0 (Sep 18, 2026).
- Runs with Docker or Kubernetes.
- Polyglot: TypeScript, Elixir, Python, Go, C#, Java and others.
- Has feature flags for failure scenarios (commit history mentions a `recommendationCacheFailure` flag and a cart-service demo flag, plus a flagd UI) and a load generator.
- Commit history mentions `agent`, `chatbot` and `mcp` components, so it likely includes AI parts. Not verified in detail.
- Dozens of observability vendors fork it to show their integrations, and the README lists them.
- Not yet verified: exactly which catalog machines it exercises (queue, cache, database), and its memory needs on a laptop.

### Territory findings: Acquit (the user's SaaS)

From https://github.com/NDilanka/acquit (README and root `package.json`, read during the walk):
- "A marketplace where AI-agent operators deliver verified coding work, and PayPal escrow pays them once the work is proven." MIT. 558 commits, most co-authored by factory-droid (built with Droid).
- TypeScript (about 89%), Node.js 24 running `.ts` directly. Root `package.json` has no runtime dependencies; workspaces `packages/core`, `packages/verifier`, `packages/acquit-cli`, `packages/mcp`. Tests also cover `packages/ctl`, `packages/house`, `packages/runner`, `apps/api`, `apps/web`.
- Services: a `node:http` API (port 4310), a web app (port 5173), a verifier (port 4311, `/healthz`) that "judges with the Docker subject by default".
- Storage: SQLite (`DATABASE_PATH=./data/acquit.db`).
- External: PayPal (sandbox API, escrow, payouts), a GitHub App (creates and deletes client-repo forks), a model key (`FACTORY_API_KEY` in `/etc/acquit/model-secrets.env`).
- Agent interfaces: a control CLI that prints JSON, and an MCP server over stdio with "exactly the six approved tools" (in progress per the latest commits).
- Background job: `sweep` of expired visitors, described as safe to run from cron.
- Retries: the payout reader "renews its token and retries a refused read once".
- Deployment: `deploy/` holds systemd unit templates (`acquit-api`, verifier) and a `configure.sh` for a VM.
- Verification lanes: parallel copies of the whole stack on offset ports, with memory-based admission.

**→ superseded by "Corrections to the Acquit mapping" in Quadrant 4.** Original likely mapping (inferred): web clients and agent CLI/MCP → Intake; API → Assembler; verifier with Docker subjects → Assembler bank; SQLite → Tank; PayPal and GitHub → Docks; model calls → Kiln; sweep → Cam clock; sign-in → Gate; payout retry → Return loop fitting. Verification lanes could be drawn as parallel copies of the plant.

Hints for queued questions (to be shown when asked): Q6, the fixtures are Rust (Acan) and TypeScript (Acquit), plus the polyglot Astronomy Shop. Q8, Acquit is built with Droid, so Droid sessions are a real first source for the Floor layer.

### Landmines

- **L2 (affects Q7):** Acquit's infra is **systemd units and a shell script on a VM**, not docker-compose or Kubernetes. If v0.1 only reads compose and Kubernetes, Acquit has no "declared" infra.
- **L3 (raises Q13):** Acquit's root manifest shows **no OpenTelemetry**. Per-package manifests were not checked. If Acquit isn't instrumented, its "observed" source is empty on day one. This is also the best first real ghost task: "instrument Acquit," done by an agent, which clears fog on a real product.

- **L4 (raised by D3):** GPU harnesses on three operating systems need GPUs in CI. Hosted CI runners usually have none, so golden screenshots would run on software rasterizers (for example Mesa's lavapipe on Linux or WARP on Windows), which can differ pixel by pixel from real GPUs. Real-GPU frame-time checks need self-hosted or GPU runners. **Decided:** three-tier CI plan, see the L4 card in Quadrant 4.

- **L5 (raised by D12):** with a closed app, engineers can't read the source to confirm their telemetry stays local. Trust needs other proof, for example an in-app network log of every outbound connection, a hard offline mode, and a published statement of what is ever sent. **Decided by Q16:** all three ship in v0.1.

## Preference delegation

Confirmed by the user after Q5:
- P1: On depth, take recommendations that make a part thin but complete.
- P2: On reach, go wider (more platforms, more real systems).
- P3: Real systems over toy examples.
- P4: Performance and craft first; no unnecessary layers.

Scope: the agent answers Q6 to Q11 on the user's behalf, each disclosed and correctable. Q12 (licensing) and Q13 (Acquit instrumentation) stay with the user. Delegation lasts for this walk only and does not authorize implementation.

## Decision ledger

| # | Question | Decision | Why | Closed by |
|---|---|---|---|---|
| D1 | Q1: v0.1 scope | **The full ghost-to-live loop, in thin form:** all three sources, plan view only, ghosts, MCP, simulation by replaying recorded traces. Only the catalog machines the sample systems need are built. Other views (elevation, section, isometric, exploded) come later. **→ amended by S3:** share-safe PNG, SVG and video-frame export is in v0.1, and PR drawing diffs move to v0.2. **→ R1:** the plan view is flat. | The ghost-to-live loop is the product's core idea and its demo; thinness keeps it buildable. | User |
| D2 | Q2: sample systems | **Three fixtures:** Acan itself (in-process view, dogfooding), the **OpenTelemetry Astronomy Shop** (polyglot microservices, Docker/Kubernetes, failure flags, load generator), and **Acquit**, the user's real SaaS (TypeScript, systemd on a VM, SQLite, PayPal, GitHub App, MCP). | Acan alone doesn't exercise the service catalog (L1). The Astronomy Shop is realistic and widely recognized. Acquit is a real product with real users, which keeps v0.1 honest. | User |
| D3 | Q3: operating systems | **macOS, Linux and Windows** in v0.1. The week-one spike must pass on all three. Any OS whose system webview fails the WebGPU checks uses the native `wgpu` host (`host-native`) for the canvas instead. **→ amended by S4, R3, Q14:** Windows (WebView2) is polished. macOS and Linux pass the spike and CI only. Linux and macOS before 26 use the WebGL2 tier, and a separate native window is the Linux fallback (never a layer under the webview, L6). | The user wants all three. The host-independent renderer (A1) makes a per-OS fallback possible without a rewrite. | User |
| D4 | Q4: team and timeline | **Solo developer with agents, about 3 months to v0.1.** Milestone: a demo-able ghost-to-live loop on Acquit around week 6. | Large scope for one person; long agent runs against strong harnesses make it realistic. The week-6 milestone keeps the core loop ahead of breadth. | User |
| D5 | Q5: simulation in v0.1 | **Recorded traffic replayed through a queueing model.** Each machine uses its measured service times and concurrency limits. Ghosts use catalog default values, marked low confidence. What-if controls: a traffic multiplier and taking a machine down. A nightly backtest produces each machine's calibration score. The scenario library and preview environments come later. This refines D1: pure replay alone could not evaluate ghosts. | Smallest simulation that makes the "Simulated" step of the ghost-to-live loop real. | User |
| D6 | Q6: code languages | **Deep analysis for Rust and TypeScript/JavaScript** (modules, entry points, HTTP/gRPC handlers and outbound calls, imports). **Shallow analysis for Go, Python, Java and C#** (packages, imports, entry points). **Protobuf `.proto` files parsed** for service contracts in any language. Everything else appears only from infra and runtime, and is fogged on "understood". | P1 + P3: the two fixtures you own (Acan in Rust, Acquit in TypeScript) get full depth. P2: shallow coverage reaches the polyglot Astronomy Shop, whose `pb` folder makes `.proto` contracts a cheap cross-language source. | Agent on the user's behalf |
| D7 | Q7: infra sources | **docker-compose files, Kubernetes manifests (Helm charts read through their rendered `helm template` output, not by parsing templates), and systemd unit files.** Acan's own in-process structure comes from its Cargo workspace. Terraform comes later. **→ extended by L21:** the systemd reader also reads `EnvironmentFile`s and flags configuration loaded in code (Acquit's `/opt/acquit/.env`) as fog. It also reads the Caddyfile, because Caddy is Acquit's real front door. | P3 + L2: Acquit deploys with systemd units, and the Astronomy Shop ships compose files and a Helm chart. P2: three formats cover all three fixtures. P1: rendered Helm output avoids writing a template engine. | Agent on the user's behalf |
| D8 | Q8: agent support | **The MCP server works with any MCP-capable agent. Passive detection for Droid, Claude Code and Codex** from their session logs or hooks. Droid is the first one built and tested. **OPEN fact:** the exact session-log and hook formats of each agent must be verified before the spec fixes them; unblocked by reading each agent's documentation. **→ fact closed and amended by E1** (native OTel first, then hooks, then transcripts; see the agent table in Quadrant 4). | P2: three agents. P3: Acquit is built with Droid (its commits are co-authored by factory-droid and it keeps skills in `.factory/skills/`), so Droid sessions are a real source on day one. | Agent on the user's behalf |
| D9 | Q9: telemetry storage | **Custom storage on the hot path:** per-entity, per-second rollups in memory, raw spans in a ring buffer, older data spilled to append-only columnar segment files owned by Acan. **SQLite for app state** (window layout, caches); the blueprint stays the source of truth in git. No embedded DuckDB. | P4: the query set is small and known (entity windows, backtest arrivals, replay buckets), so an SQL engine would be a large layer on the hottest path. SQLite is small, reliable and off the hot path. | Agent on the user's behalf |
| D10 | Q10: entity matching | **Deterministic rules with a confidence score**: OTel `service.name`, compose service names, Kubernetes labels, systemd unit names, package names (`package.json`, `Cargo.toml`), directory paths, image names and ports. High-confidence matches apply automatically; uncertain ones wait in the unplaced tray for the architect. Confirmed matches are saved in the blueprint's `match:` field. Agent-proposed matches come later. **Harness:** precision and recall on hand-labeled matches for all three fixtures. | P1: rules plus human confirmation is the thin, complete version. P4: a measured harness instead of guesswork. Ports work as a signal for Acquit, whose env file and units name them (4310, 4311). **→ amended by L18:** a port is a weak signal. Identity is role plus lane, because Acquit lanes repeat every service at `+10n`. | Agent on the user's behalf |
| D11 | Q11: blueprint schema | **Versioned (`acan: 1` at the top), split into files:** `system.yaml` (entities, flows, budgets), `layout.yaml` (positions only, so layout changes don't clutter semantic diffs), `territories/*.yaml`, `choices/*.yaml` (one file per choice so agents writing at the same time don't conflict), `machines/`, and `checks/` (protected, per K16). Strict YAML subset (no anchors or aliases). Stable slug IDs, with `renamed_from` for renames. **The file is the truth:** the app writes each edit immediately and reloads external edits live. **`acan fmt`** gives one canonical formatting. A JSON Schema is published for editor autocompletion. **OPEN fact:** whether a Rust library can edit YAML while preserving comments and order; if not, `acan fmt` canonical output is the fallback. Unblocked by a library check during the spike week. **→ fact closed by E3:** canonical YAML subset, `serde-saphyr` to read, `yamlpatch` for edits, `pretty_yaml` for `acan fmt`, and a golden round-trip corpus. | P4: clean diffs, no hidden state, and conflict-free agent writes. K14: a published schema supports the open format. | Agent on the user's behalf |
| D12 | Q12: licensing | **Open format only; the desktop app is closed source.** The open part is the Acan spec: the blueprint, territory, choice and machine-definition formats and their JSON Schema. **Settled unless the user says otherwise:** a small reference validator for the format (used by `acan fmt` and CI checks) is published with the spec, because a format without a reference validator rarely becomes a standard. | The user's choice. Trade-off accepted: "you can read the code" is no longer the privacy argument, so local-first trust needs other proof (see L5). | User |
| D13 | Q13: Acquit instrumentation | **A small zero-dependency module inside Acquit** that listens to Node's built-in diagnostic events (`node:diagnostics_channel` for HTTP server, HTTP client and `fetch`) and sends **standard OTLP** to Acan. Explicit spans only where built-in events can't see: SQLite queries, verifier runs, and any PayPal or GitHub calls not made through `fetch`. **Built by an agent as Acan's first real ghost task** ("install radar on Acquit") and used as the week-6 demo. **OPEN facts:** which SQLite driver Acquit uses, and the exact diagnostic channel names on Node 24; unblocked by reading Acquit's code. **→ facts closed by L16, amended by E4:** `node:sqlite` `DatabaseSync` has no channels, so database calls are wrapped by hand. The export is OTLP/HTTP JSON over `node:http`, not `fetch` (L14). Context propagates across the queue and the verifier callback (L13). | Matches Acquit's no-dependency style (P4). Standard OTLP keeps it usable with any backend. Doing it through Acan's own loop proves the product on a real system (P3). | User |

---

## Handover

### Before coding: facts to confirm

These are small, but each one can bend a spec line. Confirm them in week one.

1. **Droid and Codex paths on Windows.** `%USERPROFILE%\.factory\sessions\…` and `%USERPROFILE%\.codex\sessions\…` are inferred [I]. Check them on the user's Windows machine. Read `transcript_path` from hook payloads rather than hardcoding a path.
2. **Droid `genai` OTel content.** Confirm, on the user's Droid version, that `OTEL_TELEMETRY_FORMAT=genai` spans carry the tool name and the edited file paths with message content off. The Floor layer needs those to place a construction bot on a machine.
3. **One wasm build with both wgpu backends.** Confirm that the current wgpu release compiles the `webgpu` and `webgl` features into one wasm binary and selects the backend at runtime. If it can't, the renderer ships two wasm builds.
4. **WebView2 with WARP in CI.** Low confidence. Try it in the spike, but do not depend on it (Tier 1 native goldens already cover Windows).
5. **The YAML bake-off.** Run `yamlpatch` against `yaml-edit` for one day on a golden round-trip corpus (E3). If both fail, switch to TOML with `toml_edit`.
6. **Hardware for budgets.** Note the user's Windows GPU (integrated or discrete) and confirm it can act as the Tier 3 self-hosted runner. Find a macOS 26 machine for the manual production-bundle check.
7. **The Astronomy Shop on a laptop.** Check which catalog machines it exercises (queue, cache, database) and its memory needs (from D2, unverified).
8. **Node channel names.** Pin the Node 24.x patch version used by Acquit and add a fixture test listing the built-in channels D13 subscribes to (L16).
9. **Where the week-6 demo runs.** Recommendation: a local Acquit lane (`ACQUIT_LANE`), not the production VM. Confirm with the user before an agent touches the VM.

### Tweakable build plan

Sorted by how likely you are to tweak each item, not by execution order. Judgment calls come first with their alternatives. The mechanical work is collapsed at the bottom.

**Judgment calls (alternatives in brackets)**

| Call | Current choice | Alternatives |
|---|---|---|
| Week-6 demo target | A local Acquit lane, with Droid installing the D13 "radar" ghost | [the production VM] [the Astronomy Shop first] |
| Fixture order | Acquit, then Acan itself, then the Astronomy Shop | [the Astronomy Shop first, for breadth] |
| Motion numbers | 20 fps belts, still after 2 minutes | [10 fps] [still after 30 s] |
| gRPC receiver | After HTTP, if time allows in v0.1 | [HTTP only in v0.1] |
| Blueprint format | Canonical YAML subset | [TOML if the bake-off fails] |
| Linux canvas | WebGL2 tier in the webview | [a separate native window from day one] |
| Redaction default | Names, paths, ports, env and key names, hostnames, revision, headers, cookies; numbers kept | [also round numbers] |
| Trust proofs | Network log, offline switch and data statement in v0.1 | [statement only at waitlist; the rest in v0.2] |

**Milestones**
- **Week 1, spike.** The renderer pass/fail table on Windows WebView2 (WebGPU), macOS 26 (WebGPU), and Linux WebKitGTK plus macOS before 26 (WebGL2). Also the OTLP/HTTP receiver with exclusive loopback bind, the YAML bake-off, and the before-coding facts above.
- **Weeks 2 and 3, core.** Telemetry store (D9), blueprint read and write (D11, E3), readers for code, compose, Kubernetes and systemd (D6, D7, L21), and rule matching with the unplaced tray (D10, L18).
- **Weeks 4 to 6, the loop.** Plan view in the direction D identity, all universal states, ghosts, the MCP server, and Droid observed through native OTel (E1). Droid builds the D13 module on Acquit as the first real ghost task. **Week-6 demo.**
- **Weeks 7 to 9, depth.** Queueing-model replay with a traffic multiplier and machine-down (D5), the nightly calibration backtest, the Astronomy Shop fixture, Claude Code and Codex connectors, share-safe export with redaction (S3, R2), and the trust features (Q16).
- **Weeks 10 to 12, polish and launch.** Windows polish, device-loss recovery (L8), idle and soak budgets (Q15), the Acan self-drawing fixture, the launch video, and the public waitlist (U3).

**Mechanical (I trust you on that part):** the Tauri shell and IPC delta protocol (A2); the glyph atlas and MSDF text (A4, L12); CI Tiers 1 and 2 (L4); JSON Schema and the reference validator (D12); the `acan fmt` and `acan-hook` relay binaries; port conflict detection UI (L9); and the WebView2 runtime check and restart prompt (L11).

### Copyable next prompt

```
Write the Acan v0.1 spec from acan/explore/map.md. Treat the map as the source of truth: use the latest "→ amended by" value for every entry, and do not reopen decided items. Structure: goals and non-goals (from D1, U3), system overview and architecture (A1 as amended by Q14, A2, D9, E2, E5), the blueprint format (D11, E3) with a JSON Schema sketch, the plan view and visual identity (direction D: S1, S2, R1, with stage3-refined.html as the reference), sources and matching (D6, D7, D10, L18, L21), agents and MCP (A6, D8, E1, K16), simulation (D5), sharing and trust (S3, R2, Q16), the Acquit fixture and the D13 ghost task (E4, Acquit corrections), performance and idle budgets (A10, Q15) with the harness for each, CI (L4), and the 12-week milestone plan. End with the "Before coding" list as open verification tasks. Write it to acan/spec/v0.1.md.
```

After the walk, keep `acan/explore/notes.md` during the build. Each time the code forces a change from this map, log what the map said, what the code showed, and the conservative call made. Tag entries that need the user's judgment, and fold them back into the map.
