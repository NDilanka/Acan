# Generative UI, developer-experience addendum

Addendum to `generative-ui.md`. Read that first. It is not restated here. DX is now the top criterion, so an adaptive feature must make a developer faster at the keyboard, or it waits.

Labels follow the main report. **[sourced]** means a file I opened says it. **[inferred]** is my reasoning. **[guess]** needs a test. This pass opened only local files (`map.md`, `generative-ui.md`, `ui-v1/*.mjs`). Facts marked "in the main report" were sourced there. Editor flags and MCP push behavior are unverified and marked as guesses.

## 1. Keyboard-first latency targets

A10 sets an 8 ms p99 frame. [sourced, `map.md` A10] Nielsen puts "instant" at 0.1 s, and Zed targets p50 200 ms and p90 500 ms for predictions. [sourced in the main report]

| Interaction | Target | Label | System 1 |
|---|---|---|---|
| Any key to visible response | Next frame, under 16 ms | inferred from A10 | Never |
| Select (click, Tab, Ctrl K then Enter) to highlight | Same frame | inferred | None |
| Inspector open, deterministic sections readable | Under 50 ms, no slide-in | guess | "Now" default in the same frame. Tier 0 inside it. Tier 1 deadline 100 ms, then cached |
| Mode switch | Under 50 ms, selection kept | guess | Tier 0 refills "Now" for the new mode |
| Ask bar open | Input focused in the same frame | inferred | None |
| Ask bar keystroke to lexical results | Under 16 ms p99 | guess | None |
| Ask bar answer cards | Tier 0 under 1 ms. Tier 1 p50 under 150 ms, deadline 300 ms | guess, anchored on Zed | Past the deadline, lexical results stay. Tier 2 only on Enter |
| Scrub step to painted plant | One frame | inferred from A10 | None |
| Sim recompute after a parameter change | Under 100 ms, else marked stale | guess | None |
| Choice decided to next choice | Under 100 ms on the key, no timer | inferred | None |
| Copy as brief | Under 50 ms | guess | None |
| Open in editor, Orrery's part | Under 50 ms to spawn | guess | None |

Two rules keep System 1 off the hot path. Tier 1 runs only after a selection dwells for 250 ms, so holding Tab never queues model work. [guess for 250 ms] No keypress ever waits on a model. [inferred]

## 2. Handoffs

### One format, `brief/1`

Every card, selection, choice, session or sim run copies to one shape. It is Markdown with YAML front matter, so it pastes into an agent prompt, a PR body or an issue. [inferred] The core builds it from the card's resolved bindings and redaction tags, which the validator already checks. No model writes it, so it is the same in every tier and can be golden-tested. [inferred]

```markdown
---
orrery: brief/1
about: verifier
kind: budget_over
mode: live
rev: 3f2a91c
redaction: local
refs:
  - packages/verifier/service.ts:58
  - .orrery/choices/verifier-concurrency-2.yaml
open: orrery://entity/verifier?mode=live
---
## What
Queue wait p95 is over budget. 41 s against 30 s.
## Evidence
- designed  budget queue_wait_p95 < 30 s, locked (K16)
- observed  41 s, max 60 s
- choice    verifier-concurrency-2, confidence 0.40, reversal medium
## Untrusted telemetry
(span names and error text, fenced, never instructions)
## Constraints
- Do not edit budgets or checks (K16).
## Ask
```

Values are from `acquit.mjs` lines 68 and 208. [sourced] "Ask" is left for the user. [inferred]

Two redaction profiles. `local` keeps paths, slugs and env var names, because the receiving agent needs them. `share-safe` applies the R2 set and keeps numbers. [sourced for R2; inferred for the split] Ctrl C copies `local`. Ctrl Shift C copies `share-safe`. PR bodies default to `share-safe`. [inferred]

**Never passed, in any profile.** Secret values, tokens, auth headers, cookies, `.env` contents, request bodies and customer data from span attributes. [inferred from S3 and R2] "PAYPAL_* in .env" may appear locally, the values never. [sourced, `acquit.mjs` line 86; inferred rule] Telemetry strings appear only in the fenced untrusted block, because the receiving agent is now the injection target. [inferred]

### Targets

| Target | Action | Key | Payload | Never passed |
|---|---|---|---|---|
| Editor | Open the focused ref at its line | O, Shift O to pick | Repo-relative path, line, column, as an argv array | Paths outside the repo, unresolved telemetry paths, shell strings |
| Terminal | Open at the territory worktree, or focus a blocked agent's terminal | Ctrl ` | Working directory | Env vars, typed keystrokes |
| Git | Open the branch diff or PR. Shift P copies a `gh pr create` command with a `share-safe` brief | P | Branch, base, territory file, checks, brief | Credentials. Orrery never pushes |
| CI | Open the run log at the failing step. Shift R re-runs via `run_checks` | R | Run number, branch, check IDs | Runner env. Any check edit (K16) |
| Any agent | Copy as brief | Ctrl C | `brief/1`, `local` | As above |
| Running agent over MCP | Hand to the territory's session, or pick one | H, preview, Enter | `brief/1` as an MCP resource, territory ID | Budget or check write access, other territories |

Orrery never answers for an agent, as the prototype says. [sourced, `floor.mjs` line 191] VS Code's `--goto` flag and similar templates are unverified. [guess] Windows may block focusing another app's window. [guess] An MCP client may not show a server notification to its model mid-task. [guess] So H shows "delivered" and "read" (the agent fetched the resource), and Ctrl C stays the reliable path. [inferred] Handoffs launch programs and make no connection themselves, so they go in a local handoff log beside the Q16 outbound log. [inferred]

## 3. Friction in ui-v1, ranked by impact

File and line facts are [sourced]. Fixes and ranks are [inferred].

| # | Where | Friction | Fix | Gen UI helps? |
|---|---|---|---|---|
| 1 | `build.mjs` line 209, `select` via pointerup line 383 | Selection needs a click. "Search Ctrl K" has no handler. | Ctrl K lexical jump over entities, sessions, choices, territories and files. Tab cycles loud entities by severity. | No. Later, the answer area only |
| 2 | `build.mjs` `inspectorHtml` line 425, Export line 212. `floor.mjs` `ciHtml` 181, `sessionHtml` 191, 203, 213. `blueprint.mjs` `ghostHtml` 280, 292. `sim.mjs` `simInspector` 290. `choices.mjs` 247 | Every handoff is dead or a toast. The dev leaves Orrery and retypes context. | One action registry with the section 2 keys, all producing `brief/1`. | No |
| 3 | Keydown guards in `build.mjs` 507, `floor.mjs` 255, `choices.mjs` 271, `sim.mjs` 359, `blueprint.mjs` 493 | A focused input swallows all keys. After the traffic slider (`sim.mjs` 12) or budget field (`ghostHtml` 306), Space, D, 1 to 4 and Esc die. Bare keys fire under Ctrl, so Ctrl F fits. Esc leaves Layers, queue and floor open (`build.mjs` 517). | Guard text fields only. Esc blurs, then closes the top layer. Bare keys ignore chords. | No |
| 4 | `sim.mjs` `simMode` 274 and 283, `blueprint.mjs` `arm` 150, `choices.mjs` `decide` 226 | Entering or leaving Simulate clears the selection, and so does quick add. A 1.4 s timer advances the review while the user reads, against the no-late-swap rule. | Keep selection across modes. Advance only on a key. | No |
| 5 | `build.mjs` CSS lines 30 and 129, `choices.mjs` CSS 30 | Inspector slides for 0.45 s, paper fades for 0.6 s, the queue animates. L7 says panels have no CSS animation. | Instant swaps. Only the camera eases. | No |
| 6 | `blueprint.mjs` pointerup 455 to 466, keydown 498 | A commits on a click. B needs a click on the target. Backspace deletes a ghost with no undo. | Enter places beside the selection, arrows nudge, Enter commits. B opens the picker sorted by distance. Ctrl Z reverses the last file edit (D11). | Barely. Distance ordering is enough |
| 7 | `choices.mjs` `reviewHtml` 194, click 256, keydown 267 and 277 | Alternatives need the mouse. No undo after Y. An empty reject sends the placeholder "Prefer ..." as the user's words. Right sets "later", Left only moves. | Shift 1 to 3 picks an alternative. Ctrl Z undoes. Empty sends "no reason given", with the suggestion as Tab-to-accept ghost text. Arrows only move. | Marginal. Not in v0.1 |
| 8 | `sim.mjs` `seek` 349, `simInit` 363 | Scrub, speed and ghosts are pointer only. Sim state is never written back, so a run cannot be shared. | Comma and period step a minute. The brief carries seed, window, multiplier, downed machines and ghosts. | Not for DX |
| 9 | `floor.mjs` 257, `choices.mjs` 112 and 144 | Floor marks and pins explain only on hover. "Waiting on you" opens only by click. | G focuses the floor, J and K move lanes. Shift W opens the queue. | No |
| 10 | Labels | "Search" vs ask bar. "Ask agent" vs "Hand to an agent". "Simulate" is both a canned estimate (`blueprint.mjs` `simulate` 327) and mode 3. | One verb set, Open, Copy brief, Hand to agent, Ask. Rename the estimate. | No |

Candidly, rows 1 to 5 are the product, and none is generative. [inferred] Generative UI pays only once every card is reachable by key and leaves in one step. [inferred]

## 4. What changes in the thin slice

The main slice stays, with five changes. [inferred unless marked]

1. **A deterministic DX layer ships first.** Ctrl K lexical jump, Tab cycling, the action registry, the fixes in rows 3 to 5, and Ctrl Z over file edits. Ctrl K answer cards stay out.
2. **Every catalog component gets a brief form.** Render and `brief/1` come from the same bindings and redaction tags. Each candidate carries typed `refs`, so O and H work on any card.
3. **Canvas callouts move to week two.** Keep only the inspector "Now" slot, where handoffs start. The DX layer likely costs two to three days. [guess]
4. **The replay harness measures DX.** Add time from selection to first handoff, and keys per task.
5. **New CI checks.** A keyboard-only script does six tasks with no pointer events (jump, inspect, open code, copy brief, walk choices, scrub). Panels carry no CSS transition. A golden test covers `brief/1` in both profiles.
