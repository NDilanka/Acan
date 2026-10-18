# Core four: UX critique, round 2

Reviewed as a first-time engineer running Claude Code, Droid and Codex. Driven alone, in dark, from `live-<id>.html`. Official steps printed `OK` for orders, clash, bench, and all four Kvarm scripts. Gate's first run failed step 95 (station HTML changed across 300 ms with no input) and passed on a solo rerun, so that one is a flake, not a bug. Probes are in `/tmp/xai-probes/`.

The three things the lead saw all matter. Two of them are the same bug.

## Orders

### Verdict

`not ready`

The loop is the right one and the plant language holds: red and teal pins, a brass plate, a question pinned on the sqlite tank, fog that clears when the lease is real. It fails at the moment the loop is decided. After the claim, the readback rows still lead with the words the agents chose before the answer. The floor list and the chip above it disagree about how much is waiting. A hint line floats on the plant.

### Must fix

1. After the claim, the readback rows still say "pending" and "failed". Confirmed on the keyboard path, the mouse path and the official script. In `/tmp/xai-probes/out/orders-keys-claimed.png` the dock flag reads `CLAIMED · Claude Code · lease g1`, the status line reads `3 of 3 · matches · claimed`, and the row still opens with `pending`. Droid's row opens with `failed` next to `2 of 3 · yields`. Same words in `/tmp/xai-orders/orders-10-claim.png` and in the inspect card at `/tmp/xai-orders/orders-11-inspect.png`. The mouse probe read the card text as `pending · Claude Code 3 of 3 · matches · claimed` and `failed · Droid · second session 2 of 3 · yields`. `ordersReadback` always sets `choice` from `r.answers.q1` (`concepts/orders.mjs:279`), and both `ordersReadingHtml` (`concepts/orders.mjs:307`) and the dock renderer (`concepts/orders.mjs:527`) print it with no guard for a settled question. Those two words were the disagreement. Once the engineer answered, they are a closed question, and leaving them in front of "matches" makes the claim look unfinished. Drop the choice word once `question.answered` is set. Keep it, in amber, only while the question is open.

2. The Waiting-on-you badge says 7 while the panel lists 3 rows. It is not a concept bug, and it is worse than a stale fixture. `build.mjs` hardcodes `<b>7</b>` on the button and `floor.mjs` paints three agent rows (Droid, Codex, Claude Code). `setWaitingCount` (`build.mjs`) writes the badge and the "you" chip, and orders only adds the open question and the unmatched holder on top of 7 (`concepts/orders.mjs:529`). So the badge is 7 at rest, 8 while the question is open, and 7 again after the claim, against the same three rows. The unmatched script makes it 9, and the fourth row it adds ("Codex holds g1 · no readback") is not in `.bl .agent`, so the badge and the list diverge by six (`/tmp/xai-orders-kv-unmatched/orders-kv-unmatched.png`). The "you" chip repeats the lie: `7 waiting`. Count the rows the panel actually draws, or stop printing a number the panel cannot explain.

3. A hint line floats mid-canvas at the top left. After Post, the guide hides and `#orders-hint` shows at `left:20px; top:130px` (`concepts/orders.mjs:13`, text set at `concepts/orders.mjs:515-516`). In `/tmp/xai-orders/orders-10-claim.png` it reads `P inspect · click a pin · U Kvarm` with no panel behind it, sitting on the plant between the mode switch and the machines. The concept hint is already in the top bar. One hint, in the bar. Delete the floater.

### Should fix

The keyboard path to the second pin is not the path the guide describes. Shift M pins must-keep only when paypal is selected at 15:35. The guide says "select paypal at 15:35" and never says how. Comma and period seek by 60 seconds (`replay.mjs`), so nine presses of `.` from 15:44 lands at 15:39, still on retry, and Shift M does nothing (`/tmp/xai-probes/out/orders-keys-scrubbed.png`). The official steps jump with Ctrl K and the word "paypal". A first-time engineer will not invent that. Put the two Replay jumps the guide already has ("Retry · 15:44", "PayPal · 15:35") on keys, and say so.

P both posts and inspects, and I both jumps to the question and is listed after the question is gone. The hint at rest says `I question` while no question exists (`/tmp/xai-probes` first frame, `conceptHint: "P inspect · I question · U Kvarm"`). After the claim the floater drops I and the top bar keeps it. A key that does nothing should not be advertised.

The guide still ships type below the bar. "Work orders · from witnesses", the two "Replay" labels, the key captions and the gate line all measure 10 px (`orders-first` probe). `READINESS.md` G3 says nothing visible below 11 px.

"Droid · second session" is an explanation, not a name. It is appended in `ordersAgent` (`concepts/orders.mjs:220`) and then repeated by the floor row, which writes "Droid second session · yields payout order". The session id `droid-payouts` is the honest disambiguator and it already appears on the chip. Use it.

The unmatched card is honest and ugly. `UNMATCHED · Codex holds g1 · no readback` is the right sentence, and the territory stays fogged. The card then spends a third of its height on a five-row egress log (`/tmp/xai-orders-kv-unmatched/orders-kv-unmatched.png`), and the witness pins are clipped to `W2 MUST KEEP · 15:3`. The stale dock has the same clip, plus a heartbeat row that overflows into the byte count (`/tmp/xai-orders-kv-stale/orders-kv-stale.png`). The log belongs behind the pill.

`theme=light` does nothing. The hash is unread. Body background stays `rgb(10, 10, 10)` and the guide heading stays `rgb(238, 238, 238)` (`orders-light` probe). G2 asks for light. This shell has no light appearance at all, so it is a shared gap, noted once here.

### Ideas

When the claim lands, rewrite the two rows as a result, not a transcript. "Claude Code · matched · claimed" and "Droid · yields · wanted failed". The choice word earned its place only while the question was open.

Let the engineer pin the second witness from the guide without learning Replay. The two buttons already seek. Pressing one should select the machine too, so the next key is Shift M.

Show the lease as a clock on the plate only, and let it be the one moving thing. Ten minutes of countdown in a corner nobody is watching is noise. Ten minutes on the plate the bot just walked onto is the claim.

## Clash

### Verdict

`not ready`

The loop is right and the plant language holds. The card now leads with the consequence, in the fail red, and the settlement stays readable on the machine: `port v1`, `after radar-otel`. It fails the same floor-list lie as orders, and it fails the bar on type size and on a count chip that is not a count.

### Must fix

1. The same Waiting-on-you lie, plus two. At rest the badge reads 9 against the same three rows, because clash adds `waitingBase + 2` (`concepts/clash.mjs:291-294`) on top of the hardcoded 7. The panel lists Droid, Codex and Claude Code. The "you" chip says `9 waiting`. After both settlements the badge falls back to 7, still against three rows (`clash-keys` probe, `/tmp/xai-clash/clash-01-claimed.png`). The small "2 clashes" note inside the button is the honest number, and it is smaller than the badge it contradicts.

2. The count chip is a sentence, and the top of the screen is three bars. `#clashCount` reads `2 clashes before code`, then `0 clashes · 2 settled before code`, in a toolbar that also holds Footprints, Next clash and Undo (`concepts/clash.mjs:598`). Above it sits the concept hint, which repeats V, I, Shift 1-4 and Ctrl Z. Below the mode switch sits the concept switch. Three strips, one job. The chip should be a number on the pin, `2`, turning teal at `0`. The toolbar buttons duplicate keys the hint already lists.

3. Type is below the bar in the place the engineer reads. The advisory note is 10 px (`#clashAdvisory`, `concepts/clash.mjs:528`). The order plate's "Codex cannot see orders" line is 9 px (`.clash-order>small`, `concepts/clash.mjs:553`). The Kvarm strip on the plate is 9 px (`.clash-kv`, `concepts/clash.mjs:603`). The footprint labels are 9.5 px. These are the lines that say where a number came from. G3 says 11.

### Should fix

The layer card calls itself advisory and then offers Settle. c2 is drawn `data-advisory="true"` because one side is history (`concepts/clash.mjs:213`), so the header says `POSSIBLE CONSEQUENCE · HISTORY IS ADVISORY` and a note says "This overlap does not block code" (`/tmp/xai-clash/clash-06-layer-card.png`). The same card then says `Settle · before any file is edited` and counts c2 in `2 clashes before code`. Pick one. If history does not block, do not count it and do not use the Settle verb. If it blocks, do not call it advisory.

"Why · paypal-payouts already depends on radar-otel" is still a sentence with no picture. Nothing on the plant shows that dependency, so the suggestion reads as a guess. Name it on the belt, or drop "suggested".

Footprints are honest and then too many. Six labels, each `machine · agent · source`, and the overlap resolver walks the whole canvas to seat them (`concepts/clash.mjs:360-369`). In `/tmp/xai-clash/clash-02-footprints.png` they read, and they also cover the plant. Show the two that clash. The other four belong in the declaration, which already lists them.

The offline pin says `port v1` and hides `decision pending` in an 8.5 px `::after` (`concepts/clash.mjs:620`). Pending is the state the engineer has to act on. Put it in the label, in amber, at the same size as the result.

### Ideas

One pin, one number, one verb. The brass pin already says "Retry meets breaker". Let the chip on it be `2`, and let I be the only way in. Footprints, undo and Kvarm stay on keys. The toolbar can go.

When a settlement changes the plant, say the result in the noun and nowhere else. `port v1` on the belt is the sentence. The order plate repeating `CLAIMED · Claude Code · lease g1` beside it is a second product.

Draw the dependency the suggestion cites. A thin `after` line from radar-otel to paypal-payouts, labelled with the sheet revision, would make "suggested" mean something.

## Gate

### Verdict

`not ready`

This is the one that looks finished. The forecast is a range, the queue is ranked, "Blocks publication" is a separate section, and the return brief is a brief rather than a timeline. It never stops an agent, and it says so on the card. It fails because the next review is the wrong review, and because the number on the badge is not the number in the queue.

### Must fix

1. I skips the thing that blocks publication. The station opens with "Exempt the webhook from basic auth" under `Blocks publication · Kvarm`, and the first advisory row is "Approve the paypal breaker". Pressing I opens the breaker (`gate-keys` probe: `open: "breaker"`). `nextItem` walks `present()`, which sorts by cost class and then fixture order (`concepts/gate.mjs:16, 128-132`), and `harm` outranks `publish` (`concepts/gate-data.mjs:13-20`). So the keyboard path the hint calls "next review" never lands on the only row that blocks publication unless the engineer already knows to press I seven more times. The mouse path is fine: clicking the row opens it. A first-time engineer on keys will review the breaker and leave, believing the queue is done. Lead with the blocking row, or make the first I open it.

2. The badge says 7 and the queue has 8 rows, and neither is the floor list. `setWaitingCount(present().length)` (`concepts/gate.mjs:230`) writes 7, which is the present items, not the 8 rows on screen (7 present plus "Seal radar-otel", which is expected). The floor panel still lists 3 agents. The "you" chip says `7 waiting`. Three numbers, three meanings, one label. The badge should be the count of the section it names. If "Waiting on you" means the floor list, count the floor list. If it means the gate queue, say "7 reviews" on the station and leave the floor badge alone.

3. "Blocks publication" and "Advisory" share a card ending. The webhook card says `Blocks publication · Kvarm #212` and then ends `Advisory. Droid decides whether to park; Orrery never stops an agent.` (`gate-mouse` probe, `.gate-note` at `concepts/gate.mjs:117`). Both sentences are true and they read as a contradiction. The block is on publication, not on the agent. Say that in the note when `i.kvarm` is set: "Blocks publication. The agent is not stopped." The advisory note belongs on advisory cards only.

### Should fix

The section headers are 9.5 px and the row meta is 9.5 px (`.gate-sec`, `.gate-row-main small`, `concepts/gate.mjs:361, 378`). The demand note is 10 px. The row meta is where "guess" and "history" live, which is the honesty the card depends on. 11 px.

The return brief covers the plant and the queue at once. In `/tmp/xai-gate/gate-08-brief.png` the brief sits over the right half of the factory and the station is still open on the left, so the engineer gets two lists of the same half hour. Close the station while the brief is open, or let the brief replace it.

"Slips first: Seal radar-otel" names an item that is not in the queue yet. The note is right that it slips at the high estimate. It should say "expected, not here" so the engineer does not go looking for a row.

Step away changes the floor list to "Droid claimed radar-otel" and "Codex no session yet", and the badge drops to 3 (`gate-keys-away`). That is the departure fixture, and it is correct. Nothing on the station says the floor list just changed underneath it. One line, "Floor shows 16:00", would stop the engineer trusting a list that is now a recording.

The allowance buttons read `⇧120 min` with no gap (`gate-keys` probe). The key and the value run together.

### Ideas

Rank the blocking row outside the cost sort, always first, always with a verb that is not Review. "Clear to publish" is the action. "Advise hold" stays on the advisory rows. Two verbs, two sections, no shared button.

Let the gauge be the forecast and the list be the queue. Today both say the same thing in different units. The gauge already shows 34–62 against an allowance of 40. The list does not need to repeat the arithmetic in a 10 px note.

On return, open the brief and nothing else. Five rows, ranked, each with one button. The station comes back when the brief closes.

## Bench

### Verdict

`not ready`

The idea is the best of the four. Three attempts, sealed, replayed against the witnesses, one disproved, one out of budget, a human pick, the order bumped to r3. Keyboard and mouse both get there. It fails on a number the fixture does not have, and on witness pins that have left their machines.

### Must fix

1. The bench claims 9 replays when 8 happened. After the pick, `#benchZoneLabel` reads `bench-1 8 of 9 replayed · picked` (`bench-mouse` probe, `/tmp/xai-probes/out/bench-mouse-closed.png`). The grid shows 8 cells with a result and one "not replayed": A3 never replayed w3, because its budget ran out. `replayedCount` sums `s.replayed` (`concepts/bench.mjs:36, 334`), and A2's w3 is a `breaks`, which counts as replayed, so the sum is 8. The label then prints `attempts.length * WIDS.length`, which is 9, as if the ninth were merely pending. A break is a replay. An unreplayed cell is not. `8 of 8` is the truth, or `8 replayed · 1 not replayed`. Printing 9 invents a replay the fixture does not have. G4.

2. The witness pins have moved onto the rigs. w1, w2 and w3 are Replay moments on retry, paypal and sqlite. In `/tmp/xai-bench/bench-05-results.png` and `/tmp/xai-probes/out/bench-mouse-closed.png` the three pins sit on the sealed rigs inside the bench zone, and the paypal dock has no pin at all. The rigs already carry A1, A2, A3 lamps. Putting the witness names on the rigs says the witnesses are part of the experiment. They are the criterion. Keep the pins on the machines they were pinned to, and let the rigs carry only the attempt.

3. The order plate says the clash is open after the bench has settled it. The plate keeps `design open · c1 unsettled` through funding and results (`/tmp/xai-bench/bench-01-empty.png`, still there in `bench-05-results.png`), while the pin says `Retry meets breaker · on bench-1` and the grid has a disproved attempt. After the pick the plate finally says `design Retry with jitter` and the pin says `retry wins · bench-1`. Until that moment the plate and the pin disagree about whether anything has been decided. While the bench is running, the plate should say `bench running`, not `c1 unsettled`.

### Should fix

U means two things. On the bench it extends a budget. On orders, clash and gate it opens Kvarm or the queue. The shared bar says one verb everywhere. Extend is a new verb and it has taken a key the other three concepts already spent. Pick a free key, or call the action `Fund` again and keep U for Kvarm.

The zone label collides with itself. The steps file expects `Benchempty · frame an experimentP` (`concepts/bench.steps.json`), which is `Bench` and `empty · frame an experiment` and `P` rendered with no gap. Same collision after the pick: `bench-19 of 9 replayed · pickedP` in the official shot name `bench-10-closed`. The CSS is a flex with a gap (`concepts/bench.mjs:548`) and the text still runs together. Give the count its own element.

"Fund the bench" is a third verb for a concept whose job is Pick. The hint says `Enter fund` and `Shift 1-3 pick`. Fund is real (it spends the budget and opens the arena), and it is one more word than the bar allows. The button can say `Start`, which is not a domain verb, and leave Pick as the only decision.

The results card is the right artifact and it is dense. Status, spend, diff and a Kvarm log share the card with the grid (`/tmp/xai-bench/bench-05-results.png`). The grid is the decision. Spend and diff can collapse under the column. The log belongs behind the pill, as on the other three.

`theme=light` is unread here too. Same shell gap as orders.

### Ideas

A witness pin that stays on its machine, with the three attempt results as small lamps on the pin, would make the bench readable from the plant. The card becomes the place you pick, not the place you learn.

When an attempt is disproved, strike it on the rig and leave it. A2's rig going quiet is the sentence. The card repeating "cannot be picked" is the footnote.

Record the pick as a revision the other three concepts would recognise. r3 with a design is right. Say `order revised` in the toast, not `Pick recorded`, so the bench ends in the same language the order started in.

## Cross-concept

### Do they look like one product?

The materials do. Brass, graphite glass, IBM Plex Mono for readings, Inter for prose, the same fail red and ok teal, the same isometric plant. A card from orders would sit next to a card from gate and nobody would flinch. That part is won.

The behaviour is four products sharing a shell.

The verbs do not hold. Pin, Post, Answer, Claim and Settle are the five the bar names. Orders uses Pin, Post, Answer, and Claim happens alone, which is correct. Clash uses Settle, and also Footprints, Next clash, Undo. Gate uses none of the five. Its verbs are Step away, Advise hold, Open review, Return now. Bench uses Fund, Pick, Record, Extend. Claim is a flag on three of them and an event on one. An engineer who learned orders cannot guess gate.

The keys do not hold. P posts an order, steps away from the gate, and opens the bench. I jumps to a question, the next clash, the next review, and the next witness. U opens Kvarm, toggles the queue, and extends a budget. M pins a witness and sets an allowance. The hint bar teaches whichever concept is active and unteaches the last one.

The floor list is a third product. Every concept rewrites `.bl .agent` text and `.bl .wait b`, and none of them owns the three rows the panel was built with. The badge is 7, 8 or 9 depending on which concept last wrote it. The "you" chip mirrors the badge. The panel mirrors neither.

Kvarm is the one shared idea that survived. The pill, the held-write count, the reconnect, the "checked heads and leases" line, and the refusal to draw an unmatched lease as CLAIMED are the same sentence in orders and clash. Gate and bench only show the pill. That is enough. Do not invent a fourth network panel.

### What the three shots mean

The badge is a real bug, and it is one bug across all four. The hardcoded 7 is the root. Each concept then adds its own count, so the lie changes shape per concept and never goes away.

The stale readback words are a real bug, and they are the worst of the three, because they sit on the claim. A claim the engineer did not approve has to be legible. `pending` next to `matches` is not legible.

The floating hint is a real bug, and it is orders only. Clash, gate and bench put their hint in the top bar. Orders puts a second one on the plant after Post. Delete it.

### Are they ready against the bar?

No. G1 is still red and I could not test it: there is no journey shell, so the four concepts never run as one session. G2 is unmet on light, because `theme=light` is read by world and by nothing in this shell. G3 is unmet on type size in every concept. G4 is unmet on the bench's `8 of 9` and on the waiting badge, which is a number no fixture row adds up to. G5 is this review, and it is not "ready, no must-fix".

The plant is ready. The loop is ready. The chrome around them is not.

### What would make it one product

One floor list, owned by nobody but the shell, counting the rows it draws. Concepts append a row or they do not. They never write the badge.

One hint bar, and the five verbs only. Anything else is a key with no label, or it is a new verb added to the bar in all four concepts on the same day.

One place for Kvarm. The pill opens the log. Cards stop embedding the log.

Pins stay on the machine they name. Orders does this. Clash does this. Bench does not. Gate's station is a panel, and that is fine, because a queue is not a machine.

Until those four are true, shipping one of these concepts teaches the engineer a dialect the next concept will contradict.
