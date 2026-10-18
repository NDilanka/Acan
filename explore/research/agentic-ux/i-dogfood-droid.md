# Develop Acan inside Acan

## Summary

[inferred] Ship a small self-development loop before the broad service catalog or simulation.
[inferred] Acan starts local Droid candidates, but agents choose and claim orders after checked readbacks.
[inferred] Keep coordination in authenticated MCP calls, live activity in process events, and passive observation in OTel.
[inferred] Enforce K16 outside the editable checkout, including the runner and configuration that interpret checks.
[inferred] Run a pinned last-good observer against a separate candidate build, with a plain-terminal escape hatch.
[sourced] V4 needs revision because [Claude documents beta spans](https://code.claude.com/docs/en/monitoring-usage) and [Codex's cited bug closed](https://github.com/openai/codex/issues/12913).

## 1. Droid integration from day one

### What Factory exposes

[sourced] `droid exec` supports headless tasks, `--cwd`, prompt files, JSON results, session continuation, forks, tags, and worktrees, according to [Droid Exec](https://docs.factory.ai/droid-exec/overview.md). [sourced] [Droid Exec](https://docs.factory.ai/droid-exec/overview.md) defaults to read-only, while `--auto medium` permits ordinary local edits, builds, tests, and commits.

[sourced] Factory documents a long-lived subprocess using `--input-format stream-jsonrpc --output-format stream-jsonrpc`, with session initialization, turns, notifications, permission requests, interruption, and cleanup, in [Droid Exec](https://docs.factory.ai/droid-exec/overview.md). [sourced] [Droid Exec](https://docs.factory.ai/droid-exec/overview.md) deprecates the older `stream-json` input.

[inferred] Use that subprocess directly from Rust rather than introduce a Node service just to host an SDK. [inferred] Pin the CLI version and test its protocol against captured fixtures before depending on exact request shapes.

[sourced] The [TypeScript SDK](https://docs.factory.ai/sdk/typescript.md) offers `createSession`, `resumeSession`, streamed turns, structured output, permission handlers, and session discovery. [inferred] It is a useful implementation reference, not a requirement to replace Acan's Rust core.

[sourced] Droid reads MCP configuration from `~/.factory/mcp.json` and project or ancestor `.factory/mcp.json` files, according to [MCP](https://docs.factory.ai/harness/mcp.md). [sourced] [MCP](https://docs.factory.ai/harness/mcp.md) documents `droid mcp add acan <url> --type http` for user configuration and `${NAME}` expansion in HTTP headers.

[inferred] Connect should preview and merge the Acan entry without replacing other servers. [inferred] Configure Kvarm separately with an expiring repository-scoped token, an environment-backed authorization header, and OAuth disabled for its header-authenticated endpoint.

[sourced] Kvarm's agent guide has no verified Droid-specific setup yet, although its generic client requires Streamable HTTP and a custom authorization header. See `/tmp/kvarm/docs/agents/README.md` and `/tmp/kvarm/docs/agents/generic.md`. [inferred] Treat a real Droid `get_repo` call against Kvarm as a day-one acceptance test, not as established compatibility.

### Observation is not coordination

[sourced] [Telemetry setup](https://docs.factory.ai/enterprise/telemetry/index.md) documents Droid's OTLP HTTP export through `OTEL_TELEMETRY_ENDPOINT`, with `OTEL_TELEMETRY_FORMAT=genai` selecting GenAI spans and metrics. [sourced] That page documents organization-pinned overrides and a separate Factory metrics export at [telemetry setup](https://docs.factory.ai/enterprise/telemetry/index.md).

[inferred] For Acan-launched children, set `OTEL_TELEMETRY_ENDPOINT=http://127.0.0.1:4318`, `OTEL_TELEMETRY_FORMAT=genai`, and message-content export off in the child environment. [inferred] Do not invent a Droid `settings.json` `env` block or globally redirect every instrumented application through shell-wide OTLP defaults.

[sourced] GenAI export produces an `invoke_agent droid` root per turn with model and tool children, according to the [data reference](https://docs.factory.ai/enterprise/telemetry/data-reference.md). [sourced] The [data reference](https://docs.factory.ai/enterprise/telemetry/data-reference.md) lists tool names, call IDs, `session.id`, and `gen_ai.conversation.id` independently of content, while arguments and results require content export.

[inferred] Content-free OTel identifies activity but does not establish edited file paths. [inferred] Resolve bot placement from explicit declarations, minimal path-only hook events, and actual git diffs, with each source named.

[sourced] GenAI subagent traces are separate roots without the legacy parent-session linkage, and GenAI durations use seconds rather than legacy milliseconds. See the [data reference](https://docs.factory.ai/enterprise/telemetry/data-reference.md). [inferred] Preserve child relationships through launch records and subagent events rather than infer them from trace parentage.

[sourced] [Hooks](https://docs.factory.ai/harness/hooks.md) are command handlers in user or project `hooks.json`, including `SessionStart`, `PreToolUse`, `PostToolUse`, `Stop`, `SubagentStop`, and `SessionEnd`. [sourced] [Hook payloads](https://docs.factory.ai/harness/hooks.md) include `session_id`, `cwd`, and `transcript_path`.

[inferred] Install an opt-in `acan-hook` relay at an absolute, protected path. [inferred] Emit session lifecycle, normalized changed paths, tool outcome, and sequence IDs, but discard prompt bodies, file contents, and shell arguments.

[sourced] `PreToolUse` can deny calls, while post-tool hooks run after the operation, according to [hooks](https://docs.factory.ai/harness/hooks.md). [sourced] [Hooks](https://docs.factory.ai/harness/hooks.md) are snapshotted at startup and external edits cause warnings. [inferred] A reporting relay must use bounded timeouts and remain independent of policy enforcement, so a dead viewer does not disable protection.

[inferred] Use subprocess notifications for immediate progress, OTel first for passive observation, hooks second, and transcripts only for backfill. [inferred] Deduplicate by session, tool-call identity, and source rather than count each observation as another action.

[sourced] The map records Linux Droid JSONL sessions under `~/.factory/sessions/<cwd-slug>/<uuid>.jsonl`, but calls the format undocumented and Windows paths unverified. See `/home/factory-user/acan/explore/map.md:202-210,305-308`. [inferred] Read `transcript_path` from hooks, tolerate unknown records, and display format drift instead of fabricating complete history.

[sourced] Interactive `/sessions`, `droid --resume`, and headless `--session-id` are documented recovery routes. See the [CLI reference](https://docs.factory.ai/droid-cli/cli-reference.md). [inferred] Persist the session ID beside each launch so a crash does not make the work inaccessible.

### How an order becomes work

[sourced] `/home/factory-user/acan/explore/ui-v2/CORE.md` requires witnesses, readbacks before code, autonomous claims, declarations, and clash pins before edits. [sourced] `/home/factory-user/acan/explore/ui-v2/CORE.md` makes the later review gate advisory and excludes it from these prototypes.

1. [inferred] Posting creates an immutable order revision containing machine IDs, the request, witness references, check and budget digests, and the repository base SHA. [inferred] A human confirms executable witness assertions when posting. [inferred] Unsupported assertions remain explicitly uncheckable and cannot silently pass the claim gate.

2. [inferred] Acan's local launcher wakes a bounded, user-enabled agent pool with available orders, not an assignment to implement one. [inferred] Start with one Droid candidate, which can read, decline, or request a claim. [inferred] A user-started Droid with the same MCP connection can participate without Acan spawning it.

3. [inferred] The launcher prepares an isolated candidate checkout at the recorded base before readback, then keeps that working directory through implementation. [inferred] The candidate returns a structured readback with expected witness outcomes, a proposed transformation, assumptions, and intended files. [inferred] Initially run `droid exec --cwd <checkout> --output-format json -f <readback-prompt>` without editing autonomy. [inferred] The trusted adapter validates the readback inside the JSON result against Acan's schema, then imports it and the claim request on that session's behalf.

4. [inferred] A deterministic evaluator replays the candidate's supported interpretation against the locked witness cases. [inferred] Self-reported green checks do not count as evaluator results. [inferred] Conflicting interpretations produce the single anchored question described in CORE, whose human answer creates a new witness revision.

5. [inferred] A matching candidate claims through an atomic compare-and-swap over order revision, check digest, and available ownership. [inferred] The adapter executes the agent's claim request without another human assignment click. [inferred] A lease generation fences stale agents after expiry, restart, or takeover.

6. [inferred] Before edits, the claimant declares machines, ports, layers, and paths it expects to change. [inferred] Acan compares active declarations and holds the implementation phase at unsettled clash pins. [inferred] An empty overlap result is not proof that the changes are compatible.

7. [inferred] After settlement, associate the candidate checkout with its Kvarm workspace and continue the Droid session with `--auto medium` in the same directory. [inferred] Include the order revision, lease generation, and approved declaration in its implementation prompt. [inferred] Scope growth requires a revised declaration before work on the new area.

8. [inferred] Independent checks run on the candidate tree, then again on Kvarm's composed integration tree. [inferred] A trusted publication step pushes only the checked workspace head and submits that SHA to Kvarm, without granting Droid high autonomy. [inferred] Distinguish finished agent work, passed checks, published code, and observed live behavior. [inferred] Only the relevant runtime witness can establish that an operational change became live.

[inferred] Expose the day-one coordination contract through MCP with separate operations for order listing, readbacks, claim requests, declarations, heartbeats, and evidence. [inferred] These are proposed Acan tools, not existing Factory or Kvarm tool names. [inferred] The JSON adapter initially submits on the agent's behalf. [inferred] Direct agent submission needs a read-only candidate tool policy plus permission for narrowly scoped coordination calls, not general edit permission.

[sourced] Kvarm coordinates agents but calls no model itself, and its task claims use expiring, generation-fenced leases. See `/tmp/kvarm/README.md` and `/tmp/kvarm/docs/agents/workflow.md`. [inferred] Therefore Acan, an external runner, or the user starts Droid, not Kvarm task dispatch alone.

[inferred] Keep Acan authoritative for witness matching and claims on day one, and mirror accepted work into Kvarm workspaces. [inferred] If task dispatch later owns claims, move ownership there explicitly rather than require an impossible atomic transaction across two authorities.

### Instructions, specs, and missions

[sourced] [AGENTS.md](https://docs.factory.ai/harness/agents-md.md) carries repository commands and conventions, [skills](https://docs.factory.ai/harness/skills.md) supply reusable procedures, and [custom droids](https://docs.factory.ai/harness/subagents.md) add separate contexts with enforced tool policies. [sourced] The [skills reference](https://docs.factory.ai/harness/skills.md) explicitly says `allowed-tools` metadata is not a sandbox.

[inferred] Put subsystem commands and the Kvarm workflow in Acan's `AGENTS.md`. [inferred] Put readback and declaration procedures in `.factory/skills/`, and use a read-only custom droid for independent inspection only after the single-agent loop works.

[sourced] [Spec Mode](https://docs.factory.ai/autonomy-and-safety/specification-mode.md) plans before implementation and normally requests approval through `ExitSpecMode`. [sourced] [Missions](https://docs.factory.ai/missions/reference.md) orchestrate workers and inherit integrations, while [Droid Exec](https://docs.factory.ai/droid-exec/overview.md) requires high autonomy for headless missions.

[inferred] Treat a spec as planning evidence, not an approved check change or an automatic claim. [inferred] Defer missions until every worker can be associated with the same claim, isolated writes, declarations, and protected checks.

[sourced] Settings include autonomy defaults, `blockOnMcpLoad`, permission rules, hook controls, and `cloudSessionSync`, whose documented default is true. See [settings](https://docs.factory.ai/droid-cli/settings.md). [inferred] Connection setup must disclose session sync and model-network use separately from Acan's local storage promise.

### Claude Code and Codex

[sourced] V3 proposes Claude's user-settings `env` configuration and user-scope MCP, while current Claude docs confirm MCP configuration lives in `~/.claude.json`. See `/home/factory-user/acan/explore/proposals/ui-v1.md:11` and [Claude MCP](https://code.claude.com/docs/en/mcp).

[inferred] Retain V3's Claude telemetry selectors with OTLP logs and metrics, an explicit `http/json` protocol, and the local endpoint. [sourced] Unlike V4's blanket no-spans statement, current Claude docs describe opt-in beta tracing through `CLAUDE_CODE_ENHANCED_TELEMETRY_BETA` and `OTEL_TRACES_EXPORTER`. See [monitoring](https://code.claude.com/docs/en/monitoring-usage). [inferred] Default to event-derived activity and expose beta spans as a capability, not a universal assumption.

[sourced] Codex's official sample documents `[otel]`, separate exporters, and HTTP protocol values `binary` and `json`, resolving V3's protocol uncertainty. See [sample configuration](https://developers.openai.com/codex/config-sample). [sourced] Kvarm verifies Codex HTTP MCP using `bearer_token_env_var` in `~/.codex/config.toml`. See `/tmp/kvarm/docs/agents/codex.md`.

[sourced] V4's cited issue #12913 closed as completed with a telemetry fix, but issue #33668 reports missing headless OTLP token metrics in Codex 0.144.3. See [#12913](https://github.com/openai/codex/issues/12913) and [#33668](https://github.com/openai/codex/issues/33668). [inferred] Probe the installed version and each signal rather than advertise either guaranteed parity or permanent absence.

[inferred] Both adapters retain the same readback, claim, declaration, and protected-check contract. [inferred] A passively observed session without coordination access must say it cannot see orders, as CORE requires.

## 2. The minimum self-hosting loop

### Draw actual boundaries inside the desktop app

[sourced] K18 makes Acan the first fixture, but the handover places its self-drawing late and explicitly allows fixture order to change. See `/home/factory-user/acan/explore/map.md:43,319-341`. [inferred] Move a thin self-drawing and Droid order loop before Acquit instrumentation without dropping Acquit or Astronomy Shop from v0.1.

[sourced] `/home/factory-user/acan/explore/map.md:98` says Acan's threads, channels, and embedded stores do not exercise the distributed service catalog. [sourced] `/home/factory-user/acan/explore/map.md:293` derives its structure from the Cargo workspace. [inferred] Draw logical machines within one Acan housing rather than pretend each crate is a network service.

[inferred] The first drawing needs a Rust-core assembly containing the OTLP receiver, matcher, order evaluator, and delta producer. [inferred] Show its bounded channels as internal belts and its telemetry ring and segments as embedded storage.

[sourced] D9 distinguishes the custom telemetry store from SQLite app state, while A1 and A2 distinguish the Rust core, wasm renderer, webview, and binary IPC deltas. See `/home/factory-user/acan/explore/map.md:51-52,295`. [inferred] Draw SQLite separately from telemetry history, the IPC boundary as a labeled port, and the renderer inside the webview housing.

[inferred] Expose renderer frame time, IPC backlog, ingest drops, and durable-order state as gauges only when measured. [inferred] Show webview and GPU details as unknown when unavailable, and never manufacture service spans from module names.

[inferred] Put Droid's construction bot, the Kvarm workspace, the CI crane, and the human inspection station on the development floor. [inferred] Keep model endpoints and Kvarm outside the Acan process boundary so offline limits remain visible.

### A testable dogfood ladder

1. [inferred] Milestone zero establishes repair without Acan. [inferred] Pin the toolchain and Droid version, record build and test commands, configure the Kvarm repository, and retain a standalone check runner. [inferred] Pass when a terminal-only change can build, test, and reach Kvarm without launching the desktop app.

2. [inferred] Milestone one makes Acan visible to itself. [inferred] A minimal canvas reads its blueprint, Cargo declarations, and explicitly instrumented internal spans. [inferred] Pass when a known ingest event lights the correct internal belt and an unknown module stays unplaced. [inferred] Promote this binary to last-good before adding agent control.

3. [inferred] Milestone two observes plain Droid. [inferred] Connect telemetry, capture a session ID, and map a declared path to a machine. [inferred] Pass when a real terminal Droid edit appears once, raw message content stays absent, and disconnecting telemetry shows unknown activity rather than idle success.

4. [inferred] Milestone three posts one durable order and imports a real readback. [inferred] Add witness pinning, locked checks, a JSON readback evaluator, autonomous claim requests, and declarations for one Droid. [inferred] Pass when a mismatching readback cannot claim, a matching one can, and restarting Acan preserves the order and ownership.

5. [inferred] Milestone four closes a self-development order. [inferred] The first request is "Keep ingest running and preserve this order when the webview reloads." [inferred] Capture a controlled reload as a real witness, pin continued ingest as must-keep, and pin lost or duplicated order state as must-not. [inferred] If the build already passes, inject a labeled development fault before recording a witness rather than invent a production incident.

6. [inferred] Complete that order in a separate candidate checkout while last-good watches. [inferred] Pass when Droid produces a patch, the independent runner reproduces red then green, Kvarm checks the exact integration tree, and the promoted build survives the same reload. [inferred] This is the first usable self-hosting build, not completion of all v0.1 features.

7. [inferred] Milestone five adds a second agent and an actual pre-code clash. [inferred] Have two agents declare changes to the IPC contract and settle ownership before granting writes. [inferred] Pass when simultaneous claims produce one valid owner per order, both declarations remain inspectable, and no patch touches an unsettled area.

8. [inferred] Milestone six widens coverage to Acquit and Astronomy Shop, then simulation, missions, and richer passive adapters. [sourced] Those external fixtures are required because dogfooding misses most service machines. See `/home/factory-user/acan/explore/map.md:98,288`. [inferred] Pass when both an in-process IPC failure and a distributed service failure can drive the same order lifecycle.

## 3. Safeguards against self-hosting failures

### Keep the repair path independent

[inferred] Run last-good and candidate as different binaries with distinct data directories, instance IDs, and receiver ports. [inferred] Never hot-replace the observer during an active order, and never let a candidate migrate last-good's state in place.

[inferred] Snapshot state before promotion and keep the old binary, schema-compatible snapshot, CLI version, and check-runner digest together. [inferred] Test rollback after a deliberately broken candidate, including a failure before its window opens.

[inferred] Provide an export containing the order, witnesses, check digests, base SHA, declaration, workspace ref, session ID, and exact verification commands. [inferred] The escape hatch is plain `droid exec` or `droid --resume` in a terminal against the preserved checkout, with the independent runner and Kvarm CLI still usable.

[inferred] On observer restart, mark unfinished runs disconnected, then reconcile persisted process identity, git state, session identity, and lease generation. [inferred] Do not respawn from telemetry silence alone, because delayed export and process death are different facts.

[inferred] Support cancellation independently of the canvas and terminate the owned process tree, not arbitrary matching processes. [inferred] A renewed claim requires fresh ownership checks before further writes.

### K16 must cover the whole grading path

[sourced] K16 forbids agent edits to checks or budgets without human approval, and D11 includes protected `.acan/checks/`. See `/home/factory-user/acan/explore/map.md:41,297`. [inferred] Protection must also include budget-bearing blueprint fields, witness assertions, check loaders, CI commands, executable helpers, policy files, and dependency pins that determine check behavior.

[inferred] Mount protected inputs read-only into agent environments, retain an approved digest outside the checkout, and run verification from a pinned trusted runner. [inferred] An agent may propose a check change in a separate artifact, but only a human action can approve a new grading revision.

[inferred] When budgets share `.acan/system.yaml` with editable design, protect direct writes to that file and accept non-budget design patches through a validating broker. [inferred] Compare protected fields structurally, not through a text pattern that whitespace or key movement can evade.

[sourced] Droid's [sandbox](https://docs.factory.ai/autonomy-and-safety/sandbox.md) checks file tools, confines shell execution, propagates policy to subagents, and supports deny-write paths. [sourced] The [sandbox reference](https://docs.factory.ai/autonomy-and-safety/sandbox.md) names macOS and Linux or WSL2 host mechanisms, without establishing native Windows support. [inferred] On Windows, prove equivalent protection in the launch environment or use a dedicated WSL2 environment or VM before enabling autonomous writes.

[inferred] Hooks and instructions improve feedback but cannot be the only K16 boundary, since a shell command or writable policy could bypass a path-only edit guard. [inferred] Keep publication credentials and human policy approval outside the worker process.

[sourced] Kvarm offers protected-path human escalations, required checks, exact-head approvals, and a bounded `kvarm/guard` scan whose completeness is advisory by default. See `/tmp/kvarm/README.md:62-68`. [inferred] Configure these protections explicitly and add a dedicated K16 check rather than treat the generic guard as complete enforcement.

[inferred] A change to the checker itself must run the old pinned checker and a human-approved checker test suite before promotion. [inferred] Agents cannot bless their own replacement runner, rewrite saved evidence, or approve a weaker threshold.

[inferred] Test K16 with direct file edits, shell writes, parent-directory renames, symlinks, replacement CI scripts, altered check imports, and a forged human-approval request. [inferred] Each attempt must leave the approved grading inputs unchanged and produce a visible denial. [inferred] Replaying a previously valid claim after takeover must also fail before any new edit.

### Keep evidence and telemetry trustworthy

[inferred] Require run-scoped authentication for coordination and fence every mutating request with the claim generation. [inferred] A loopback address alone does not prove that a request came from the right agent.

[inferred] Separate agent-declared progress, observed tool outcomes, and independently verified results in storage and UI. [inferred] Treat repository text, transcript contents, and tool results as untrusted evidence, never as authority to change policy.

[sourced] Droid content export is raw, unredacted, and can contain prompts, tool arguments, results, and file contents. See [telemetry privacy](https://docs.factory.ai/enterprise/telemetry/privacy.md). [inferred] Keep content export off, redact imported transcripts, cap retention, and disclose that local-first Acan does not make cloud model calls local.

[inferred] Filter Acan's own ingestion diagnostics from the stream they diagnose to prevent recursive telemetry growth. [inferred] Bound internal queues, display dropped evidence, and distinguish actual replay from simulation so an observer bug cannot silently certify its own behavior.

## 4. Precedents and their limits

[sourced] Rust bootstraps a new compiler using a precompiled stage-zero compiler, builds further stages from source, and describes an optional same-result stage-three test. See [Rust bootstrapping](https://rustc-dev-guide.rust-lang.org/building/bootstrapping/what-bootstrapping-does.html). [inferred] Acan's equivalent is an independently retained last-good host, a candidate produced under it, and replayed acceptance evidence before promotion.

[inferred] Acan is not a compiler, so binary equality is not its acceptance criterion. [inferred] Compare durable state, witness outcomes, event loss, and visible behavior across old and candidate builds.

[sourced] VS Code's contributor guide describes editing its repository, launching a distinct Code OSS development build, debugging it with VS Code, and running tests directly from a terminal. See [How to contribute](https://github.com/microsoft/vscode/wiki/How-to-Contribute). [inferred] Copy the separation between working editor and tested editor, including separate profiles and a conspicuous candidate identity.

[sourced] `/tmp/kvarm/README.md:249-266` states that its only automated gate is its own CI, using `.kvarm/ci.json` for typecheck, lint, tests, and CLI checks. [sourced] `/tmp/kvarm/AGENTS.md` names `scripts/dogfood-ci.mjs` as the driver for a change through that system. [inferred] Copy the rerunnable end-to-end gate and standalone commands, not the assumption that self-hosting proves every infrastructure failure recoverable.

[inferred] These precedents support using the tool on its own development while retaining an older working tool and independent tests. [inferred] They do not justify letting the candidate replace its observer, policy, and evidence in one step.

## Recommended day-one scope

- [inferred] Ship the internal Acan drawing, one measured witness, one durable order, one Droid readback, an autonomous claim, and a declaration.
- [inferred] Use Rust-owned local process launch, JSON results first, authenticated MCP coordination, and content-free GenAI OTel.
- [inferred] Require a verified Droid-to-Kvarm connection, isolated workspace writes, protected checks, and exact-tree CI.
- [inferred] Keep a pinned last-good observer, independent runner, rollback snapshot, and terminal escape hatch.
- [inferred] Prove one two-agent clash next, then widen Claude and Codex support through per-version capability probes.
- [inferred] Defer missions and full simulation until the self-development loop survives crashes and adversarial check edits.
