# Parked session: Acan agentic UX (ui-v2)

Resume file for a fresh orchestrator chat. Updated 2026-10-18. No loops. Three subagents running (step 1 below). If this chat is gone, they are dead: launch fresh with the same briefs and tell them about their partial files.

## Hard rules and standing directives

- **No implementation of Acan until Droid gives the go signal.** The go signal requires every line of `ui-v2/READINESS.md` (G1 to G9) to be green, ending with the user's own browser walkthrough (G8). Until then, iterate on UI and UX only.
- **The user wants UI and UX that are excellent, with core features first.**
- **All design builds and critiques go to Opus.** Use pv-opus-high for builds and critiques, and pv-opus-medium for prose. The second lab on review panels is Grok (pv-grok-high or pv-grok-xhigh), or Haiku (pv-haiku-max).
- **One shell.** It is the full-bleed plant with floating glass islands in `ui-v2/shots/orders-kv-unmatched.png`. The user picked it as "the cleanest". The world-style top bar, toolbar row, framed stage and docked inspector (`shots/world-07-levels.png`) are rejected. World and the journey move onto the chosen shell.
- **There is too much text in the UI.** The text system (`TEXT.md`) sets measurable word budgets, and honesty stays one key away.
- **Run drive.mjs runs one at a time.** Parallel drivers cause timing flakes. `ui-v2/run-steps.sh [steps files]` builds and drives under a shared `flock /tmp/acan-drive.lock`, so agents running in parallel still drive one at a time. Every brief tells agents to use it. Env passes through (`AUDIT=1`, `SIZE=WxH`, `OUTROOT`).
- **Verify every agent's work yourself.** Rerun its steps files and look at the key shots before you accept it. Log each decision with `~/.factory/plugins/marketplaces/pvstack/plugins/pvstack/skills/show-me-your-work/scripts/log.sh research/agentic-ux/decisions.tsv <phase> <decision> <why> <evidence> <result>`.
- **Write replies to the user in plain sentences.** No em dashes and no mid-sentence colons. Cite principles where they decide something.
- **Fresh-chat rule (user AGENTS.md).** At every boundary, update this file. Suggest a fresh chat when the chat is heavy and the moment is cheap. Never suggest while a subagent is running or a change is half-made. When you suggest, end with a copy-paste handoff prompt.
- **Open core (user decision, 2026-10-18).** An open core keeps startup options open. The desktop app and the agent adapters are open. A possible team or hosted version stays separate. Keep that boundary clean in the design and the code. Anything team or hosted lives behind its own seam and is never required by the open desktop app.
- **Kvarm** is opt-in to the user's own origin, with declared egress. Agents self-claim after a readback match plus a Kvarm lease. The gate is advisory and blocks publication only. The world is a canvas of systems, not agents.

## Repo and name

- The product was renamed from Orrery to Acan on 2026-10-18, at the user's request. All text files, the folder (`/home/factory-user/orrery` became `/home/factory-user/acan`), the plants (`acan-format`, `acan-site`) and the Kvarm origin (`git.acan.local`) changed. The screenshots and builds were regenerated.
- This is now a git repo on `main`: `3db036b` initial import, `e9cbe5a` rename, `1d7661c` open-core decision, then this update.
- It is pushed to the public repo https://github.com/NDilanka/Acan (`origin`, `main` tracks `origin/main`). Push finished work to `main` after each commit. The repo is public, so never commit secrets.
## Where things stand

- **Done.** Research waves (a to m), syntheses, interviews 1 to 3, and the map proposal (`proposals/ui-v2.md`, including same-file rows W23 to W27).
- **Same-file editing.** Tenure is killed. Kept: a declaration-level advisory clash forecast, read-blob admission as a Kvarm ask, and a narrow Slots pilot (`research/agentic-ux/same-file/00-judging.md`).
- **Five standalone prototypes** in `ui-v2/concepts/`: orders, clash, gate, bench, world.
- **`ui-v2/JOURNEY.md`** is the spec that stitches all five into one session. Its section 1 still describes the rejected bar-and-dock shell.
- **Health check at resume.** 10 of 11 steps files print OK. `clash.steps.json` step 4 fails every time: `topStrips` expected 2, got 3, so the hint, `#cbar` and `#clashToolbar` sit on 3 rows. The cause is the stopped shell agent's partial edits to `build.mjs` and `blueprint.mjs`, which changed since `.backups/shell-before/`. `orders.mjs` changed too. The shell agent should finish this. Do not revert it.
- **Partial files from stopped agents.**
  - `ui-v2/audit.mjs` (41 KB). The AUDIT-gated code in `drive.mjs` is partly there, and plain runs still print OK.
  - `ui-v2/concepts/quiet.mjs` (79 KB), registered in `concepts/index.mjs:13`.
  - `TEXT.md` and `SHELL.md` do not exist yet.
- **Backups.** `ui-v2/.backups/shell-before/` and `ui-v2/.backups/uiv2-before/`.
- **Reviews in.**
  - `review/world-xai-r2.md`: world not ready, 3 must-fix (connector lines, queue clip, arrival message).
  - `review/core4-xai-r2.md`: all four not ready. Badge counts disagree with the rows. Readback rows say pending and failed after the claim. Gate I skips the blocking row. Bench says "8 of 9 replayed". Light mode is unread.
  - `review/journey-spec-xai.md`: 11 must-change. Items 1 and 2 are superseded by the shell. Items 3 to 11 are still open.
- **Dropped.** The Haiku world r2 and core4 r2 reviews. They judged the pre-shell layout. The second-lab review moves to the journey.

## Next steps, in order

1. **Relaunch three agents in parallel**, all pv-opus-high, all told to operate as poteto-agent and load `poteto-delegate` first. Relaunched fresh on 2026-10-18 at /home/factory-user/acan paths. Live task ids: shell 218c3950-0a16-4b2e-95f8-e06c2957d297, text 4a995f7c-9767-4426-9a0e-0deb33948f26, audit 56613b91-6569-48a9-9674-fe05495bb482. The old resume ids below are dead. Agents do not commit; the coordinator verifies and commits.
   - **Shell** (resume id 8f886fb1-0f0c-46f1-b44d-6322cba54ce2).
     - Write `ui-v2/SHELL.md`, the one shell spec, based on the liked orders layout. Give island positions and sizes at 1280x800, 1440x900 and 1600x900.
     - Fold world's extras into the islands. The breadcrumb and Back/Forward go in the top-left island, the search becomes Go to (Ctrl K), and the toolbar items become Layers menu items or palette commands with their keys. Appearance goes in Layers or the top-right island. The inspector becomes a floating card. The minimap goes in the compass slot or is dropped.
     - Fix the orphaned top-left hint line, island alignment, the compass-card collisions, light islands and focus rings. Add no new chrome, and do not rewrite copy.
     - Rewrite JOURNEY.md section 1 to match.
     - Move world onto the shell. Update the world steps files, and keep every world key.
     - Fix the clash step 4 failure.
     - Put side-by-side shots in `shots/shell-*.png`.
     - Every steps file must print OK, run one at a time.
     - It may edit `build.mjs` and the ui-v1 shell CSS, with backups. It must not edit `quiet.mjs`, `TEXT.md` or `audit.mjs`.
   - **Text** (resume id f7310e8b-45ab-4e59-b6da-eb673ee9c7dc).
     - Exhaust the design space: at least four directions, drawing on Factorio and Mini Metro HUDs, Linear, Figma, Things, Tufte and HIG progressive disclosure. Score each on scan speed, honesty and keyboard reach.
     - Write `ui-v2/TEXT.md` with measurable word budgets per surface and per screen, the counting selector logic for the audit, and the honesty rule.
     - Prove it on `concepts/quiet.mjs`, a copy of orders, with `quiet.steps.json` and before and after shots `shots/quiet-*`. Use the kv=unmatched flow and the claimed end state, in dark and light. Put the word-count table in TEXT.md.
     - Do not restyle the shell, and do not edit `orders.mjs`.
   - **Audit** (resume id 956577d1-ed0d-49e4-94d4-80b4ae7de42d).
     - Finish `audit.mjs` behind `AUDIT=1` in `drive.mjs`. The rules are type-floor (11 px), overlap (with a `data-audit-stack` opt-out), offscreen, contrast (4.5:1, and 3:1 for large text, with background blending), focus-ring (as the `{"audit":"focus"}` step), transition (with an allowlist), egress-sum, and focus-steal.
     - Add `SIZE=WxH` to `drive.mjs`.
     - Run it over every steps file in dark and light, with shots to `/tmp/audit-<id>`. Write `review/audit-r1.md`.
     - Edit only `audit.mjs`, the AUDIT and SIZE code in `drive.mjs`, and the review.
2. **When they return**, verify each yourself and show the user the before and after shots. Add the word-budget and island-position rules from TEXT.md and SHELL.md to `audit.mjs`.
3. **Revise `JOURNEY.md`** (Opus). Apply spec-review items 3 to 11, and reconcile it with SHELL.md and TEXT.md.
4. **Build the journey** (Opus). Inputs are the revised JOURNEY.md, SHELL.md, TEXT.md, and every must-fix in `review/*-r2.md`. Verify at 1280x800 and 1440x900, in light, with kv=offline, and with AUDIT=1.
5. **Run a cross-lab journey critique** (G5), then fix rounds.
6. **Run the cold first-use test** (G6): six tasks, two models.
7. **Finish up.** Update the READINESS status column, refresh the proposal's prototype section (G7), do the browser walkthrough with the user (G8), then give the go signal.

## Open questions for the user

None blocking. The user may want the Kvarm panels moved off the compass, and the shell agent's SHELL.md decides that.
