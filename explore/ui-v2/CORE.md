# Core concepts spec: work orders and clash on the belt

Throwaway prototypes that decide how Orrery's core loop feels. Evidence and reasoning are in `../research/agentic-ux/00-synthesis.md` and `../research/agentic-ux/concepts/00-judging.md`. The user's calls are these. Agents claim open orders on their own, after a readback. The review gate is advisory (wave 2, not here). Developer experience must be excellent, and core features come first.

Round 3 adds three calls. Kvarm is an opt-in connection to your own origin, and every byte to it is drawn as declared egress. A claim needs both a matching Orrery readback and a live Kvarm lease of the same generation; a lease holder with no matching readback is drawn as UNMATCHED, never as CLAIMED. We ask Kvarm to enforce that readback match server-side as a precondition on claim_task. The review gate stays advisory.

## The loop these two concepts make

Today the engineer types a prompt into each agent and reads a diff at the end. In Orrery the engineer does this instead.

1. Point at real moments in Replay and pin them as **must not** or **must keep**. These pins are witnesses.
2. Post one sentence plus the witnesses as an **order** on the machine. The witnesses become locked checks (K16).
3. Agents send a **readback**, which is their interpretation replayed against the witnesses before any code. If two readbacks disagree, Orrery asks the engineer one question at the place where they disagree. The answer becomes another witness.
4. An agent whose readback matches every check **claims** the order on its own. Nobody assigns it.
5. Each claimed agent **declares** what it will change. Where two declarations meet on a belt or a layer, a **clash pin** appears with its consequence. The engineer settles it with one key, before any file is edited.

Concept `orders` covers steps 1 to 4. Concept `clash` covers step 5 and starts from the state that `orders` ends in.

## Shared rules

**Files.** Each concept is one module, `concepts/orders.mjs` or `concepts/clash.mjs`, that default-exports `{ id, label, hint, css, html, script }` (see `concepts/index.mjs`). `script` declares `function <id>Init(params)`. Import fixture data from `concepts/core-data.mjs` and inline it into `script` with `JSON.stringify`. Edit only your own module and your own steps file. Do not edit `build.mjs`, `drive.mjs`, `core-data.mjs`, other concepts, or the ui-v1 modules. If you need a change in a shared file, stop and report it.

**Build and run.** `node build.mjs <id>` writes `live-<id>.html`. The concept is active with the hash `c=<id>`, for example `live-orders.html#c=orders&mode=replay`. `node build.mjs` with no argument builds all concepts into `live.html`. Yours must work there too, inactive unless its hash is set.

**Runtime globals you may use** (from `build.mjs` and the ui-v1 modules): `SCENE`, `ENT`, `$`, `select(id)`, `selected`, `renderPlant()`, `apply()`, `flyTo(id)`, `setMode(m)`, `toast(text)`, `wake()`, `layoutChips()`, `bare(ev)`, `typing(ev)`, `conceptActive(id)`, the replay state in `replay.mjs`, and the territory and session data in `floor.mjs`. Read those modules before you use them. To add a bot or a ghost, push to `SCENE.entities` and `ENT`, then call `renderPlant()`. Prefix every global you add with your concept id (`orders`, `clash`).

**Keys.** Taken by ui-v1, so do not rebind them: 1 to 4, A, B, C, D, E, F, G, H, J, K, L, N, O, Q, R, S, T, W, X, Y, Z, Tab, Enter, Esc, arrows, `[ ] , .`, Space, Home, End, Delete, Backspace, Alt, Ctrl K. Free: I, M, P, U, V, and Shift with a digit (Shift 1 to 4 already picks an alternative in the choices review, so use it the same way, for picking among options in an open card). Guard every handler with `conceptActive("<id>")` and `bare(ev)`. When your card is open and handles a key, call `ev.preventDefault()` and `ev.stopImmediatePropagation()` so mode keys do not also fire. Register your listener with `{ capture: true }` for that reason.

**Quality bar.** These come from `../ui-v1/DECISIONS.md` and `../research/generative-ui-dx.md`, and they are not optional.
- Every step works from the keyboard alone, and every step also works with the mouse.
- A key gets a visible response on the next frame. Panels and cards appear and disappear instantly, with no CSS transitions. Only the camera eases. The one allowed animation is the existing "ink-in" moment, where a ghost turns solid.
- Nothing moves under the user. An agent event that arrives later (a readback, a claim) is appended and announced, and it never steals focus or advances a card.
- Esc closes the top layer only. Text fields swallow letter keys, and Esc blurs them first.
- Cards that are about a machine are anchored on the plant next to that machine, not in a side panel. Labels and chips never overlap, so call `layoutChips()` after you add chips.
- Copy is short and plain. One verb per action, the same verb everywhere: Pin, Post, Answer, Claim, Settle. Numbers come from the fixture. Never print a number that the fixture does not have.
- Show uncertainty honestly. A declaration's source (declared, static or history) is visible. A passive agent that cannot see orders says so.
- Each concept has a one-line hint in `hint` that lists its keys, in the same style as the ui-v1 hint.
- Match the visual language: brass `#c9a45c`, graphite glass panels (`var(--glass)`, `var(--line)`), IBM Plex Mono for readings, Inter for prose, and the existing state colours (`var(--wait)` amber, the fail red, the ok teal). Look at `../ui-v1/shots/p5-floor.png` and `../ui-v1/shots/c7-review.png` before you design a card.

## Concept `orders`: work orders from witnesses

Start state: `#c=orders&mode=replay`. The replay window is today 15:30 to 16:30. The paypal-payouts territory is unclaimed and fogged.

1. **Pin a must-not.** Scrub to 15:44 with `[` and `]` or `,` and `.`, or open at `t=-45.5` if replay supports a start time (check `replay.mjs`). Select the retry machine. Press **M**. A red pin appears on the retry machine, labeled `MUST NOT · 15:44`. An inline editor shows the reason from `WITNESSES[0].reason`, editable. Enter keeps it.
2. **Pin a must-keep.** Scrub to 15:35. Select paypal. Press **Shift M**. A teal pin appears, labeled `MUST KEEP · 15:35`.
3. **Post the order.** Press **P** with paypal or any machine in paypal-payouts selected. The order plate opens anchored at the paypal dock. It shows a one-line sentence field (placeholder from `ORDER.placeholder`), the witnesses as rows with their checks, the locked budget `paypal.error_rate < 2%`, the sheet revision, and the line "Codex cannot see orders. Passive session, no MCP." Enter posts the order. The plate inks in and becomes a brass plate on the plant, flagged `OPEN`. The order switches the view to Live (key 1) so the plant shows now.
4. **Readbacks arrive.** At `arriveMs` after posting, each readback appears as a row under the plate. Each row has the agent, its one-line interpretation, and pass or fail per witness. A bot for each agent appears near the dock (add `bot-cc` and `bot-d2` to `SCENE`). The two readbacks agree on w1 and w2 and disagree on p-2307, so a question pin appears on the sqlite tank's "effects queue rows" layer with `QUESTION.text`. Focus does not move. A toast says "Two readings differ on p-2307. Answer at sqlite." **I** jumps to the open question.
5. **Answer.** With the question open, **Shift 1** or **Shift 2** answers, or a click does. The answer becomes witness w3, and its row is added to the plate as a locked check. The readbacks re-score. Claude Code now matches 3 of 3. Droid's second session matches 2 of 3 and shows `yields`.
6. **Claim.** Claude Code claims on its own. The plate flag changes to `CLAIMED · Claude Code · lease g1` with a lease countdown. The paypal-payouts territory changes from unclaimed to claimed, its fog clears, and the bot moves onto the dock. A toast says "Claude Code claimed the order. Readback matched 3 of 3." If the user picked Shift 2, Droid's second session claims instead, with the same rules. Nothing asks the user to approve the claim.
7. **Inspect.** Selecting the order plate (click it, or **P** again with paypal selected) opens it read-only. It shows the claim, the witnesses, the checks and the readback that matched.

End state: expose `window.ordersState` with `{ witnesses, order: { state, claimedBy }, question: { answered }, rev, kvarm }` for the steps file.

## Concept `clash`: settle clashes before code

Start state: `#c=clash`, Live. Seed the end state of `orders` directly. The order is claimed by Claude Code with witnesses w1, w2 and w3 (option 1), and `bot-cc` sits at the dock. Draw the order plate in its claimed state. Do not import code from `orders.mjs`. Duplicate the small drawing you need.

1. **See footprints.** Press **V**. Each declaration in `DECLARATIONS` draws as a translucent outline over the machines it touches, in a colour per agent, with a small label `Claude Code · paypal-payouts · declared`. The source label (declared, static or history) is visible on every write. V again hides the outlines. Clash pins stay visible either way.
2. **See clashes.** A brass clash pin sits at each clash location: c1 on the api to paypal belt at the breaker, and c2 on sqlite's "effects queue rows" layer. c3 is advisory, so it is a faint dashed mark on acquit-api with no pin and no count. A chip near the top shows `2 clashes before code`.
3. **Open one.** **I** jumps to the next unsettled clash and opens its card on the plant, next to the pin. A click on the pin does the same. The card shows the two declarations side by side (agent, territory, the assertion, and the writes with their source), the one-line consequence, and the witness it breaks, if any, as a red chip `breaks w3 · p-2307 stays pending`. The suggested resolution is first and marked `suggested`.
4. **Settle.** **Shift 1** to **Shift 4** picks a resolution, as does a click. The plant answers each one in its own way.
   - **Split at the port.** A ghost port plate `paypal.result v1` appears on the belt, owned by Claude Code. Both declarations now point to it.
   - **Retry wins.** The breaker ghost dims and gains a `waits` tag.
   - **Breaker wins.** Claude Code's outline on the belt dims, and its bot shows `replanning`.
   - **Serialize.** An arrow from the first territory to the second appears, labeled `after`.
   - **One migration owner.** The sqlite layer shows `owner Droid`.
   - **Merge.** The two territory labels join.
   The card closes, the pin becomes a small settled mark (click it to reopen and change the decision), and a toast says "Settled before code. No file was edited."
5. **Done.** When both clashes are settled, the chip reads `0 clashes · 2 settled before code`. **Ctrl Z** undoes the last settlement, the same way the blueprint undoes.

End state: expose `window.clashState` with `{ footprints: bool, open: id|null, settled: { c1: kind|null, c2: kind|null }, kv }`.

## Kvarm and evidence (wave 3)

Kvarm is an opt-in connection to the user's own origin (`git.orrery.local`), so every op to it is declared egress. Each concept inlines its own scripted fixture from `concepts/core-data.mjs` (`KVARM`) and makes no network requests. A strip under the order plate logs each op with destination, bytes, and reason. The egress pill (top right) adds the bytes and opens the network log. The hash flag `kv=ok`, `kv=unmatched`, `kv=stale`, or `kv=offline` picks the script. No flag is `kv=ok`, and the default stays as calm as before.

**Orders.**
- Post writes order revision `r1` locally, then logs `create_issue` and `dispatch_issue`. The flag reads `POSTED LOCALLY` until both return, then `OPEN · kvarm #41`.
- Readback rows say `readback · Orrery`, because Kvarm stores no readback.
- An answer that creates w3 bumps the order to `r2` and logs `comment_issue`. Readbacks show `re-scored on r2`.
- A claim logs `claim_task` from the agent's own token. The flag reads `CLAIMED · Claude Code · lease g1` with a countdown. The other matched agent shows `yields`.
- With `kv=unmatched`, Codex holds the Kvarm lease with no readback. The flag reads `UNMATCHED · Codex holds g1 · no readback` in amber, the territory stays fogged, and a Waiting-on-you row appears. Never draw this state as `CLAIMED`.
- With `kv=stale`, a heartbeat fails. The flag reads `LEASE STALE · g1 expired`, the bot shows `stopped · STALE_GENERATION`, and the territory refogs.
- With `kv=offline`, Post shows `POSTED LOCALLY · not in Kvarm` and a matching readback shows `MATCHED · claim waits for connection`. Writes are held and counted on the pill. Reconnect shows `checked heads and leases`, and Enter sends the held writes in order.

**Clash.**
- Footprint labels name their source, for example `declared · kvarm intent` or `static`.
- Each clash card shows the Kvarm `forecast` evidence with age, or `forecast truncated`. c3 stays advisory.
- Every settlement writes a choice file and logs `record_decision` with the choice path and the decision id. Split adds a contract commit (`git_push` of `contracts/paypal.result.v1.yaml`). Split, a winning side, and one migration owner each log `set_intent` for every narrowed side. Serialize shows `after · pin waits for upstream revision` and logs no `add_dependency`. Merge logs only the blueprint commit.
- The settled mark shows `decision synced` or `decision pending`. Ctrl Z logs a superseding decision and marks the old one `superseded`, because Kvarm cannot delete a decision idempotently.

**Bindings and provenance.**
- Every semantic mark reads from a binding record with channel, value, state, source class, locator, revision, and age. The mark, its label, and its popover all read the same record.
- Alt shows labels, as in ui-v1. Alt plus hover on a mark opens its provenance. The order card lists the same bindings under Evidence bindings for keyboard users.
- Draw five distinct states. Unknown is fog hatch, an empty bezel, and no motion. Stale is a dimmed last value with age and no motion. Zero is an explicit `0` with the needle at rest. Stopped is a dark shell with a lifecycle source. Simulated is graph-paper hatch with a `SIM` tag. Invalid draws as unknown with a reason.
- A missing field never falls back to an `assets.mjs` demo default. Acquit's verifier draws one declared cell with unknown occupancy.
- Failure and fog can show together.
- Bot badges show principal, session, and evidence level: `declared`, `authenticated`, or `verified`.

## Verification contract

Write `concepts/<id>.steps.json` for `drive.mjs`. It drives the whole flow by keyboard, with an `eval` and `expect` after each beat, and a `shot` named `<id>-NN-<beat>` at each beat into `shots/`. Add a second, shorter mouse-only path in the same file. Each kv script has its own steps file, `concepts/<id>-kv-<script>.steps.json`, driven with the hash flag, for example `node drive.mjs live-orders.html 'c=orders&kv=stale' concepts/orders-kv-stale.steps.json shots`. Run the command below, and it must print OK with no page errors.

    node build.mjs <id> && node drive.mjs live-<id>.html c=<id> concepts/<id>.steps.json shots

Also run `OUT=live-all-<id>.html node build.mjs` (all concepts, written to your own file so parallel builders never share one) and drive that file with no hash through modes 1 to 4, to prove that ui-v1 still works with your module loaded and inactive. Read every screenshot yourself and check the bar above. Look for overlaps, clipped text, cards off screen, wrong anchoring, and anything that moves without input. Fix what you find before you report.

## Out of scope for this round

The review gate, the bench, a witness taken from Simulate for a ghost (a predicted moment, labeled as a prediction), real persistence, and any change to ui-v1 modules.
