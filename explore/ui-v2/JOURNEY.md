# Journey spec: one session through the core loop

This is the build spec for go-bar line G1 in `READINESS.md`. It joins the five concepts into one shell and one continuous session. Read `CORE.md` and `DECISIONS.md` first. Their quality bar still holds, and this spec only adds what the join needs.

The user's calls that shape it are these. An agent claims on its own after its readback matches and Kvarm grants the lease. The gate is advisory. A hold blocks publication only and never stops an agent. Kvarm is opt-in, on your own origin, and every byte to it is declared egress. The world is a canvas of systems, not of agents. Appearance is Light, Dark or System, with F8 per window. Core UX comes first. We dogfood with Droid.

## 1. Shell

**One page, no switching.** `node build.mjs journey` writes `live-journey.html`. The hash `c=journey` is read once at load. After that the URL never changes. There is no hash switching, no `pushState`, and no reload. Back and forward keep an in-memory place history. Optional load flags are `kv=ok|unmatched|stale|offline`, `appearance=light|dark|system`, and `beat=<id>`, which seeds a later beat (see section 6).

**Two levels, one canvas.** The world level is the world concept's canvas of five plants. Focusing Acquit eases the camera into the Acquit site, then swaps the world canvas for the ui-v1 stage, which is Acquit's real plant. The swap is instant, with no transition. The other four plants keep the world concept's own plant stage. Leaving Acquit reverses this.

**Persistent chrome.** These four never unmount and never move.
- **Waiting on you.** One queue for the whole session, as a pill in the bottom bar with its count. U opens it as the left drawer. Its count is the only waiting count on screen. The ui-v1 status panel's `.bl .wait b` shows the same number.
- **Egress pill.** Top bar, right. It reads `local + kvarm · N B out`, where N is the sum of the session ledger (section 5). Click or Shift V opens the network log in the left drawer.
- **Kvarm strip.** Bottom bar, after the queue pill. It shows connection state and the live lease, for example `Kvarm · git.orrery.local · connected · lease g1 · Claude Code · t-41`. Click opens the network log.
- **Appearance switch.** Top bar, before the pill, as a select with System, Light and Dark. F8 cycles System, Light, Dark. The choice is stored in `sessionStorage`, so it holds per window.

**Top bar** (48 px). Back and forward buttons, then the breadcrumb `My world / Acquit / paypal-payouts`, then the jump field `Jump to a place, machine, or decision · Ctrl K`, then the appearance switch and the egress pill.

**Bottom bar** (40 px). Queue pill, Kvarm strip, the one hint line, and the mode chips Live, Blueprint, Simulate, Replay (1 to 4). The mode chips show at plant level only.

**Layout grid.** Every panel lives in one of these zones. Nothing floats outside them.

| Zone | 1280 x 800 | 1440 x 900 | Holds |
|---|---|---|---|
| Top bar | y 0 to 48 | y 0 to 48 | crumbs, jump, appearance, egress |
| Bottom bar | y 760 to 800 | y 860 to 900 | queue pill, Kvarm strip, hint, modes |
| Left drawer | x 12, y 60, 360 wide, 688 tall | x 12, y 60, 380 wide, 788 tall | Waiting on you, network log |
| Dock | x 804, y 60, 464 wide, 688 tall | x 908, y 60, 520 wide, 788 tall | one card at a time |
| Free rect | stage minus open drawer and dock, inset 12 | same | camera subject, pins, chips, `.nav`, replay deck, toasts |

The stage draws behind every zone. The camera fits its subject inside the free rect, with 100 px clear of the dock edge. With drawer and dock both open the free rect is 408 px wide at 1280 and 492 px at 1440. The minimap and the ui-v1 `.bl` panel sit in the free rect's lower corners and hide when the drawer or the dock covers them. Toasts sit centred at the bottom of the free rect, above the replay deck. The jump palette is a 560 px modal at top 96. The concept bar `#cbar` is hidden.

**Cards.** Every card renders in the dock at dock width and fits without scrolling at 1280 x 800. A card about a machine draws a leader from its header to that machine. If the machine is outside the free rect, the camera eases to it, and no other motion happens. This replaces each concept's own anchoring (orders top 100, clash left of nav, gate and bench at left). The stack order is concept card first, then the ui-v1 inspector. Opening a card closes the card below it in the dock. Esc reopens nothing.

## 2. Beats

T0 is 16:30, the end of the recorded hour that every fixture already uses. The journey clock in `journey-data.mjs` stamps each beat. Keys are the keyboard route, and Mouse is the mouse route. Both routes reach the same state. Times in brackets are journey clock stamps.

| # | Beat | Key | Mouse | What appears, and its anchor | Hands off |
|---|---|---|---|---|---|
| 1 | World [16:30] | load | load | Five plants. Queue pill `Waiting on you 3`. Acquit site reads `paypal over budget · 4.1% of 2%`. Kvarm strip `connected`. Pill `local + kvarm · 2304 B out`. No anchor. | queue `breaker, codex, digest`, ledger 4 rows |
| 2 | Intent | Alt 2 | click Acquit | Camera eases in, then the ui-v1 stage replaces the canvas at Replay 15:44:30 with retry selected. Dock shows the orders guide `Point to what matters. Pin real moments. Agents replay their reading before they claim.` It is the dock's resting card until Post. Anchor retry. | `plant: acquit` |
| 3 | Pins | M, Enter, Home, `.` x5, Ctrl K `paypal` Enter, Shift M, Enter | `Retry · 15:44`, `Pin must not`, Keep, `PayPal · 15:35`, `Pin must keep`, Keep | Witness cards `MUST NOT · 15:44 · retry · loop depth 3 of 3` and `MUST KEEP · 15:35 · paypal · 1 try`. Anchor retry, then paypal. | `witnesses w1, w2` |
| 4 | Order [16:31] | P, type, Enter | Post, click sentence, type, Post | Order card with the sentence field, locked checks w1 and w2, and budget `paypal.error_rate < 2%`. Post switches to Live. Kvarm opens issue #41 and task t-41. Anchor paypal. | `sentence` as typed, order r1 open |
| 5 | Readbacks | none | none | bot-cc arrives at 1400 ms and bot-d2 at 2600 ms. The card grows below its fold and keeps focus. The question pin appears at sqlite. Queue gains `p-2307 · keep pending?`. | `arrived claude-payouts, droid-radar` |
| 6 | Answer [16:32] | I, Shift 1 | click the sqlite pin, click option 1 | Question card `PayPal accepted payout p-2307, but the response was lost. What should happen to it?` Answer pins w3 at sqlite. Order is r2. Queue drops p-2307. Anchor sqlite. | `answer 1`, `w3 pending`, r2 |
| 7 | Claim | none | none | Rescoring 600 ms, then 1400 ms of visible evidence, then `claim_task`. Toast `Claude Code · claude-payouts claimed the order. Readback matched 3 of 3.` paypal-payouts unfogs. Kvarm strip shows lease g1. | `claimedBy Claude Code`, lease g1 |
| 8 | Declarations [16:33] | none | none | 1200 ms after the claim, d1, d2 and d3 arrive. Clash pins c1 at the api to paypal belt and c2 at sqlite. Status `2 clashes before code`. Queue gains c1 and c2. Focus stays. | `declarations d1 d2 d3` |
| 9 | Clash | I | click the c1 pin | c1 card `Retry meets breaker`, `Breaks w3 · payout p-2307`. Options Shift 1 Split at the port (suggested), Shift 2 Retry wins, Shift 3 Breaker wins, Shift 4 Serialize, Shift 5 Test on the bench. V shows footprints with their sources. Anchor breaker. | `open c1` |
| 10 | Settle [16:34] | I, Shift 1 | click the c2 pin, click Serialize | c2 card, then `Serialize, radar-otel first`. Toast `Settled before code. Decision logged to Kvarm. No file was edited.` Queue drops c2. c1 stays open. | `settled c2 serialize` |
| 11 | Gate [16:35] | U, Enter, Shift 2 | click the queue pill, click the breaker row, click Advise hold | Drawer header `40 min allowance · 9 to 18 min waiting`. Rows by class are breaker (live harm), c1 (undo cost grows), codex (agent idle), digest (publication waits). Breaker card offers Shift 1 Open review and Shift 2 Advise hold. Toast `Hold advised on the breaker. Droid keeps working. Only publication waits.` Anchor breaker. | `gate.held breaker` |
| 12 | Bench [16:36 to 16:55] | Esc, I, Enter, Shift 5, Enter, Enter, Shift 3, Enter | click the c1 row, click Test on the bench, Fund, Extend, A3, Record | Bench card with the three witness clauses. Fund, then results stream. A2 is disproved by w3. A1 holds. A3 is spent at 300000 tokens, Extend adds 100000 and 5 min, and A3 holds w3. Pick A3, Record. Toast `Recorded A3. Order is now r3. c1 settled as Split at the port.` Queue drops c1. | `bench a3`, r3, `settled c1 split via bench` |
| 13 | Publish or hold [16:58] | U, Shift 4, Shift 1 (or Shift 2) | click the queue pill, click the #184 row, click Approve (or Hold) | 2000 ms after Record, Claude Code pushes and Kvarm opens review #184. Queue gains `Review #184 · blocks publication`. Card `Review #184 · paypal-payouts · Claude Code · 4 files · +96 −20`, with w1, w2 and w3 holding from bench A3. Approve toast `Approved #184 as you. Kvarm publishes on the next train.` Hold toast `Held #184. Claude Code keeps working. Nothing publishes until you approve.` Anchor paypal. | `review 184 approved` or `held` |
| 14 | Step away [17:00] | Shift U, Shift 1 | click Step away, click `in 30 min` | Chooser with `in 30 min`, `in 1 h`, `in 2 h`, `tomorrow`. The stage dims with no animation. Top bar reads `Away until 17:30`. Hint `Shift U I'm back`. | `away backAt 17:30` |
| 15 | Return brief [17:30] | Shift U, I, Esc | click I'm back, click each item, close | Brief card `While you were away · 17:00 to 17:30`. I flies to each item's machine or plant. Items are listed in section 5. Esc closes, and the session stays live. | `returned`, queue updated |

The keyboard pass answers option 1 and approves #184. The mouse pass answers option 1 and holds #184, so both outcomes are covered.

## 3. Key map

One key, one verb. New bindings use only CORE's free keys (I, M, P, U, V, their Shift forms, and Shift with a digit), plus F8 and Alt with a digit or arrow from world. Every other key keeps its ui-v1 meaning. The hint line always starts with the focused target, for example `Clash c1 · Shift 1–5 settle · I next clash · V footprints · Esc close`. A key marked context changes its target with the focused layer, and the hint names that target.

The focused layer is the dock card if one is open, or the left drawer if it was opened last. While a dock card is open, only card keys and global keys act. Plant keys are inert then, as clash does today.

| Key | Does | Context | Replaces |
|---|---|---|---|
| Ctrl K | Jump to a place, machine, or decision. At plant level the results include Acquit's machines. | no | ui-v1 palette and world jump, merged |
| Alt 1 to 5, Alt ← → | Fly to plant N in world order. Back and forward in place history. | no | world |
| U | Waiting on you drawer, opened with row 1 focused. A row leaves when its decision is made (answered, settled, recorded, approved). An advised hold stays listed, marked `hold advised`. | no | gate U queue, world U |
| Shift U | Step away, and I'm back | no | gate P |
| Shift V | Network log | no | orders and clash U panel |
| F8 | Appearance, System then Light then Dark | no | world F8 |
| F | Fit the plant, or fit the world at world level | context | ui-v1 F, world P and Shift P |
| X | Explode the Acquit stage, or cut away another plant | context | ui-v1 X, world X |
| 1 to 4 | Mode of the focused plant. Inert at world level and while a card is open. | context | ui-v1, world |
| M, Shift M | Pin must not, pin must keep, at the selection | plant only | orders. World bookmark moves to the palette as `Mark this place`. Gate allowance is the drawer header button. |
| P | Post, which opens the order card | plant only | orders. Gate step away moves to Shift U. Bench open moves to Shift 5. |
| I | Next question, clash, witness result, review or brief item. With no card open, the next queue row. | context | orders, clash, gate, bench, world I |
| V | Show declared footprints at plant level, contracts at world level | context | clash V, world V. Gate advise hold moves to Shift 2. |
| Enter | The card's primary action. Keep, Post, Open, Fund, Extend, Record. | context | orders, gate, bench |
| Shift 1 to 5 | Pick option N in the focused card. Answer, Settle, Advise hold, Pick, Approve or Hold, Back in. In the drawer it opens row N. | context | orders, clash, gate, bench, world Shift digits |
| Ctrl Z | Undo the last settle while its toast stands | context | clash |
| Esc | Close the top layer only | no | all |

Retired in the journey are bench U (Extend is Enter when the attempt is spent), bench P (Esc closes the bench), and world M. Shift 5 is new, and it appears only on the c1 card.

## 4. Seams and decisions

| Seam | Decision |
|---|---|
| Four egress counters (orders, clash, gate's fixed 1664 B, world) and four pills | One session ledger. Every concept's send appends to it when hosted. The pill shows its sum. The network log lists its rows. |
| Four waiting counts (ui-v1 7, world 5, gate list, clash 9) | One queue model in `journey-data.mjs`. Rows arrive from journey events. |
| Session names differ (`droid-payouts`, `cc-42`, `cx-17`) | Canonical table. `claude-payouts` is Claude Code on bot-cc. `droid-radar` is Droid on bot-d2. `codex-paypal` is Codex on bot-cx, passive. `claude-port` is Claude Code's bench workspace ws-a3. Hosted concepts take aliases from the seed, so standalone fixtures stay as they are. |
| Clash plate uses the placeholder sentence and assumes answer 1 | Clash and bench read sentence, answer and w3 from the orders handoff. |
| Clash seeds the orders end state and its own Kvarm log | Seeding happens only when standalone. Hosted, clash mounts on the live orders state and appends to the ledger. |
| Orders opens in Replay at 15:44:30 | Only on the first entry to Acquit, before any pin. Later entries keep the last mode and time. |
| Gate's away window replays 16:00 to 16:30 | The journey steps away forward, 17:00 to 17:30, from a new away fixture. Gate's past window stays for standalone. |
| World queue has p-2307 and c1 at boot, and an orrery `claim` row where Codex holds g1 | p-2307, c1 and c2 rows come from journey events. The orrery `claim` row is dropped under `kv=ok`. Under `kv=unmatched` it appears on Acquit, where orders' unmatched script puts Codex on g1. |
| Gate items not in the loop (webhook #212, traceparent, DatabaseSync, radar choices, verifier, retry x2) | Not in the journey queue. The retry x2 item conflicts with the live claude-payouts session. |
| World review #184 exists at boot on the Kvarm plant | #184 is created at beat 13 as Claude Code's paypal-payouts change. Its row lives on Acquit. The Kvarm site's condition reads `Review #184 blocks publication` while it waits, and `fresh · nothing waits` before. |
| World Acquit stage art against the ui-v1 stage | The ui-v1 stage is Acquit. World draws no stage for Acquit when hosted. |
| World Acquit revision `55825fe` on `radar/otel` | Override to sheet `3f2a91c` on `main`. |
| c1 can be settled by the clash card or the bench | Both are valid. Bench Record calls clash's settle with `via: "bench"`, which runs the split intents and the contract push but not a second `record_decision`. Ctrl Z on a bench-settled c1 reopens it, and Kvarm keeps both records. |
| Card anchoring differs per concept | One dock and a leader (section 1). Clash at 560 px and the gate brief at 540 px reflow to dock width. |
| Light mode exists only in world | One token sheet in `concepts/shell.mjs`. Light values come from the `l-light-mode.md` table. World's `--w-*` alias to it. Every hard-coded colour in hosted CSS moves to a token. |
| Two palettes on Ctrl K | World jump wins, with Acquit machines merged in at plant level. |
| One hint per concept | One hint line, built from the focused layer's `hint()`. |
| World owns appearance and F8 | The shell owns them. World standalone calls the same shell code. |
| `ordersActive()` checks the hash itself | Moves into `conceptActive()` in `build.mjs`, which reads the hash on each call, so orders' steps that switch hash mid-run keep passing. |
| World coordinates are 1600 x 900 | The canvas keeps its viewBox and the camera fits the free rect. |

## 5. Fixtures that make the concepts agree

New file `concepts/journey-data.mjs`. It imports the other data files and adds only what the join needs. New numbers are marked new.

- **Sessions.** The canonical table from section 4.
- **Clock.** Beat stamps from section 2. Bench result stamps follow `EVENTS.minute` from 16:36, so A3's extension lands at 16:55.
- **Queue rows.** Boot rows `breaker`, `codex` (gate-data) and `digest` (world-data). New row `c2 · two migrations on effects queue rows`, class undo cost grows, 3 to 5 min, guess. Row c1 takes world-data's copy with class undo cost grows. New row `review-184`, class publication waits, 4 to 7 min, from world-data's review row.
- **Plants.** Acquit revision `3f2a91c`, branch `main`. Conditions by state, starting with `paypal over budget · 4.1% of 2%`.
- **Kvarm ops.** Reused ops are world `list_repos 768`, `get_heads 512`, `get_leases 384`, gate `list_pending_reviews 640`, and every orders, clash and bench op as they are. New ops are `advise_hold 212 B` (`Advisory hold on the breaker until c1 settles. Agents keep working.`), `get_review 512 B`, `approve_review 264 B` and `hold_review 232 B`.
- **Timers.** New `declareMs 1200` after the claim. New `reviewMs 2000` after Record.
- **Review #184.** Author Claude Code on claude-payouts, rule `Required reviewer is not the author`, diff from bench A3 `4 files · +96 −20`.
- **Away, 17:00 to 17:30.** Back-at choices from `GATE.backAt`. Items sort by gate class.
  - 17:06 `CI #418 passed 5 of 5. Seal radar-otel is ready for your review.` It was forecast for 17:00 to 17:20. New stamp.
  - Codex, if its row is still open. `Codex has waited 69 min at its own prompt.` Derived from its 16:21 ask.
  - 17:14 `Droid declared the breaker through paypal.result v1. Your hold is still advised.` New stamp.
  - If #184 was approved, 17:10 `Kvarm published #184 on the train.` If held, 17:18 `#184 is still held. Claude Code added 1 commit.` Both new.
  - 17:22 world's arrival row `Check waitlist artifact`. New stamp.
  - Catch-up reads on return are `get_heads`, `get_leases` and `list_pending_reviews`.
- **Offline.** Under `kv=offline`, every Kvarm write is held, at 0 B out, and listed as held. The claim stays `MATCHED · claim waits for connection`. Bench Fund is disabled with `Kvarm records the arena. Fund waits for the connection.` #184 never opens. The brief adds `Kvarm was unreachable all 30 min. N writes are held.`, where N is the count of held rows.

## 6. Architecture

**Compose. Do not merge.** New `concepts/journey.mjs` owns the shell, the key router, the beat machine, the queue, the ledger, the dock and drawer, the brief and `beat=` seeding. New `concepts/shell.mjs` holds what world and the journey share, which is the token sheet, appearance and F8, the ledger, and `dockPlace(card, entity)`. Each concept stays one module with its own steps files.

**Activation** (a shared change in `build.mjs`). `conceptActive(id)` reads the hash on each call. It is true when `c=<id>`, or when `c=journey` and the journey has mounted `id`. A new `conceptKeys(id)` is true only when `c=<id>`. Each concept guards its raw key listener with `conceptKeys` and its logic with `conceptActive`. With no hash, nothing is active, so ui-v1 modes 1 to 4 behave as today. The camera gains `fitRect(rect)` for the free rect.

**What each concept exposes.** Each registers `window.orreryConcepts[id] = { mount(seed), unmount(), act(verb, ...args), state(), hint(), advance(ms) }`. `act` reaches every step the keys reach. `advance` steps its scripted timers synchronously, for `beat=` seeding. Concepts emit `orrery:<event>` DOM events and never call each other.

| Concept | Seed in | Verbs | Events out |
|---|---|---|---|
| orders | sessions, ledger | pin, keep, post, question, answer, next, close | posted, readbacks, answered, claimed, stale |
| clash | order (sentence, rev, claimedBy, lease), witnesses, ledger | footprints, open, next, settle, undo, close | settled (id, kind, via), undone |
| gate | queue, allowance 40, away fixture | queue, next, open, advise, review, stepAway, backIn, return | advised, opened, away, returned |
| bench | order r2, witnesses w1 to w3, c1, ledger | fund, next, extend, pick, record, close | funded, recorded (attempt, kind, rev) |
| world | plant overrides, queue, ledger | focus, back, forward, fit, contracts, jump | focus (plant, level) |

When hosted, each concept hides its own pill, network panel, hint and queue, and renders its card into the dock. World also hides its top bar, footer, bottom-left buttons, queue and network columns. Gate's station becomes the drawer body.

**Shared-file changes needing sign-off.** `build.mjs` for `conceptActive`, `conceptKeys`, `fitRect` and hiding `#cbar`. `drive.mjs` for a `SIZE=WxH` environment variable in place of the fixed 1600 x 900. `concepts/index.mjs` to register journey. Each concept gets the small edits above. Every existing steps file must pass unchanged afterwards.

## 7. Verification

**Runs.** Each prints `OK`.
- `SIZE=1280x800 AUDIT=1 node drive.mjs live-journey.html c=journey concepts/journey.steps.json shots/journey-1280`
- The same at `SIZE=1440x900`, then both again with `c=journey&appearance=light`.
- `concepts/journey-kv-offline.steps.json` with `kv=offline`. Short files for `kv=unmatched` and `kv=stale` that end at the claim beat.
- Every existing concept steps file, and the no-hash build in ui-v1 modes 1 to 4.

**`journey.steps.json` outline.**
1. Boot. Assert `journeyState.beat`, queue `breaker, codex, digest`, pill sum equal to the ledger sum, and `location.href` stored for the end check. Shot `journey-01-world`.
2. Keyboard pass, beats 2 to 15, with one shot per beat (`journey-02-intent` to `journey-15-brief`). At each beat, assert the handoff from section 2, pill sum equal to ledger sum, queue count equal to `.bl .wait b`, and focus unchanged across every timed arrival.
3. `eval location.reload()`, the one reload, outside the journey under test.
4. Mouse pass, beats 2 to 15, clicks only, holding #184. Shots `journey-m02` to `journey-m15`.
5. Seed pass. For each `beat=` id, load and compare `state()` of every mounted concept with the snapshot the keyboard pass took at that beat.
6. End checks. `location.href` is unchanged within each pass, and `performance.getEntriesByType("navigation").length` is 1.

Colour asserts compare against resolved tokens, not rgb literals, so the same file runs in both appearances. `AUDIT=1` runs `audit.mjs` at every shot.

**G6 cold tasks.** Each starts from a seeded load. Each predicate is evaluated 3 s after the agent's last input.

| # | Start | One-line task | Success predicate | Expert steps |
|---|---|---|---|---|
| 1 | `beat=intent` | Post an order for paypal-payouts backed by one thing that must not happen and one that must keep working. | `ordersState.witnesses.some(w=>w.polarity==="must-not") && ordersState.witnesses.some(w=>w.polarity==="must-keep") && ordersState.order.state!=="draft"` | 10 |
| 2 | `beat=readbacks` | The two agents read your order differently. Resolve it so one of them can take the work. | `ordersState.question.answered!==null && ordersState.order.state==="claimed"` | 2 |
| 3 | `beat=declarations` | Two planned changes collide before any code is written. Settle both. | `!!clashState.settled.c1 && !!clashState.settled.c2` | 4 |
| 4 | `beat=gate` | Droid wants to add a paypal breaker while c1 is still open. Decide what happens to it. | `journeyState.gate.held.includes("breaker") \|\| journeyState.gate.opened.includes("breaker")` | 3 |
| 5 | `beat=bench` | Test the fixes for c1 against your witnesses and record the one to build. | `benchState.phase==="recorded" && !!clashState.settled.c1` | 3 |
| 6 | `beat=world` | Find the plant where publication is blocked and open it. | `journeyState.plant==="format"` | 2 |

`window.journeyState` holds `{ beat, clock, plant, level, queue, ledger, gate: { held, opened }, review, away, appearance }`.

## Open questions

1. **Answer 2 on p-2307.** Answering `Mark it failed and refund the client` makes Droid's readback match, so Droid claims, as orders does today. **Default.** Droid on droid-radar holds g1, and its own breaker becomes its plan. With one agent in both territories, c1 and c2 never form. The journey shows `0 clashes before code`, skips the bench, and #184's author is Droid. This adds a `branch.failed` block in `journey-data.mjs` and a short steps file.
2. **Away time.** **Default.** Step away goes forward, 17:00 to 17:30, with the new away fixture in section 5. The alternative is to start the journey with the user just back at 16:30 and reuse gate's recorded window, but then the brief comes before the loop, not after it.
3. **Light scope.** **Default.** All hard-coded colours in the ui-v1 stage and the five concepts move to tokens in this round, so G2 passes in light. The alternative is light for the shell and cards only, with the stage dark, which fails G2 and the contrast audit.
