# World prototype critique, round 2

## Verdict

not ready

A first-time engineer can tell in 3 seconds which plants need them, and why. The world level now ranks trouble, the plants are distinct, and the Kvarm states are honest. It is not ready because the plant level draws its connectors through labels and buildings, the queue lies about what is below the fold and clips a row the moment you scroll, and an arriving message erases the message you just caused.

## Must fix

1. **Connector lines cross labels and buildings on four of the five plant stages.** The line is drawn as a straight segment from the machine anchor to a point on the edge label (`concepts/world.mjs:308-313`), and nothing checks what that segment passes through. `declutter` only keeps labels off each other (`concepts/world.mjs:316-341`). Measured by sampling each connector and calling `elementFromPoint` (`/tmp/xai-world-steps/plants.steps.json`).
   - Kvarm. The REST line crosses the Artifacts label and the Artifacts building. `/tmp/xai-world/xai-kvarm-plant.png`, `#worldStage .world-connectors [data-line=rest]`. The probe returned `["rest","Worker"]` and `["rest","Artifacts"]`.
   - Acquit. The OTLP line crosses the OTLP boundary label. `/tmp/xai-world/world-07a-plant.png`, `#worldStage .world-machine[data-machine="OTLP boundary"]`. The same crossing is in `/tmp/xai-world/xai-light-acquit.png`.
   - Orrery. The package line crosses the matcher label and the matcher building, and the REST line crosses the telemetry ring. `/tmp/xai-world/xai-orrery-plant.png`. The probe returned eleven hits, including `["package","matcher"]`, `["rest","telemetry ring"]`, and `["rest","SQLite"]`.
   - orrery-format. The package line crosses the schema crate, whose face reads `{ }`. `/tmp/xai-world/xai-format-plant.png`. The probe returned `["package","{ }"]`.
   - orrery-site. The artifact line ends on top of the artifact receiver label. `/tmp/xai-world/xai-site-plant.png`. The probe returned `["artifact","artifact receiver"]`.
   G3 forbids two labels overlapping, and a line through a label is the same failure. The shipped overlap check never sees it, because it only compares `.world-machine`, `.world-edge`, and `.world-stage-title` boxes (`concepts/world.steps.json`, the eval before `world-07a-plant`).

2. **The queue clips its top row as soon as it scrolls, and it reports "1 new below" while that row is fully on screen.** `#worldQueueRows` scrolls, and `.world-queue-row` has no scroll padding, so the row above the fold is cut through its text. After setting `scrollTop` to 70, the claim row measured `top: -68` against the rows box and was clipped. `/tmp/xai-world/xai-queue-scrolled.png`, `#worldQueueRows [data-queue=claim]`. At the bottom, the last row ends exactly on the box edge (`bottom 802` against `box.bottom 802`), so the cost line has no breathing room. `/tmp/xai-world/xai-queue-bottom.png`.
   The "below" count is also wrong at rest. It counts a fresh row whose bottom is past the rows box (`concepts/world.mjs:351-353`). The artifact row measured `top 521, bottom 628` against a rows box of height 559, so 3 pixels of its padding counted as below, and the button read "1 new below ↓" while the whole row, including "Check waitlist artifact", was readable. `/tmp/xai-world/xai-queue-6.png` and `/tmp/xai-world/xai-light-queue.png`, `#worldQueueBelow`. A control that says a row is below the fold, when the row is not, fails the honesty bar.

3. **The arrival message replaces whatever message the user just caused.** `announce` writes one string and clears it after 2.6 seconds (`concepts/world.mjs:114-118`). The arrival fires on its own 3 second timer and calls `announce` again (`concepts/world.mjs:793-797`), so it overwrites the message for the action the user just took. Pressing X inside that window should say "Cutaway open. Declared shell, bound evidence." It said "orrery-site needs an artifact check. Added to Waiting on you. Your view has not moved." (`/tmp/xai-world-steps/plants.steps.json`, step 23). One status line cannot carry two facts. Queue the second, or let the user's own message finish.

## Should fix

- **Tab leaves the world.** The first Tab from boot focused the ui-v1 body, whose class string is `traffic ghosts agents territories choices used has-insp`. That body is `display:none` while the concept is active (`concepts/world.mjs`, the `body[data-world=active]>:not(#worldApp)` rule), and it is still in the tab order. `/tmp/xai-world/xai-tab-1.png`. Six more Tabs landed on hidden ui-v1 territory labels, "RADAR-OTEL · VERIFYING". A keyboard user who does not already know Ctrl K gets no focus ring anywhere in the world. Give `#worldApp` a focus scope, or set `inert` on the hidden shell.

- **Arrow keys do not pan.** With `#worldCanvas` focused, Left, Right, and Up left the camera at `{"x":0,"y":0,"s":1}` (`/tmp/xai-world-steps/edges.steps.json`). The canvas is reachable by drag only. Keyboard-only navigation still works through Alt 1-5, Ctrl K, and the queue, so this does not block the go bar. It does leave the canvas itself with no keyboard move.

- **The decision count is missing from the world level.** The brief asks the world level to show name, freshness, decision count, and top condition (`../research/agentic-ux/01-synthesis-wave3.md`, semantic levels). Name, freshness, and the top condition are on the site. The decision count is not. `WORLD.plants[].decisions` is 1, 2, 1, 1, and 0 (`concepts/world-data.mjs:5-70`), and none of those numbers reach the canvas.

- **Light mode has one contrast miss.** The queue count `6` measured 4.21:1 at 11 px in light, under the 4.5:1 body floor in G3. Selector `#worldQueueTitleCount`. Every other measured text passed, and nothing rendered below 11 px.

- **A contract line has no halo where it crosses a building.** The route text has a paper stroke so it stays readable (`concepts/world.mjs`, `.world-route text`). The connector paths do not. A paper halo, or a route that bends around labels, would fix must-fix 1 without hiding the building.

- **The plant stage is mostly empty margin.** On orrery-format and orrery-site the stage is a small cluster in a 1100 by 620 frame, with the neighbor labels doing the work. `/tmp/xai-world/xai-format-plant.png`, `/tmp/xai-world/xai-site-plant.png`. Fit the cluster to the frame, and the connectors get shorter and cross less.

- **Shift 1 to 4 do nothing unless the queue or the bookmarks card is open.** From the world, Shift 1 changed nothing (`/tmp/xai-world-steps/keys.steps.json`, `xai-shift1-no-card`). The hint says U opens the queue and then Shift 1 jumps. Say that in the footer, since the footer currently lists neither.

- **`kv=offline` starts with a held write the user never made.** Booting with `kv=offline` clones `D.pending`, so the pill reads "Kvarm offline · 1 write held · 40 s" before any action (`concepts/world.mjs:124`). The panel is honest about the reason, "Scripted local choice, not published". The pill is not. A cold reader thinks the prototype is holding their own write.

- **The viewport cannot be changed.** `drive.mjs` hardcodes `--window-size=1600,900` and `Emulation.setDeviceMetricsOverride` at 1600 by 900 (`drive.mjs:22`, `drive.mjs:49`). No env var sets it. The small-viewport probe was skipped for that reason. The only responsive rule is a topbar squeeze below 1200 px (`concepts/world.mjs`, the `max-width:1200px` block), so a laptop width is untested.

## Ideas

- **Lead with the two plants that block publication.** The scope line already says "2 block publication". Make those two sites the first thing the eye hits, by pulling them forward on the grid or by opening on a framed pair. Today the eye lands on orrery-format because it sits top left, which happens to be right, and on Orrery because it sits in the center, which is not. The center plant is a recommended hold.

- **One reason per plant, said once.** Kvarm currently says "blocks publication" in the chip, again in the condition, and a third time if you open the row. Keep the chip as the rank and let the condition carry the reason only. The same cut on Acquit removes "? 1 question +1" plus "p-2307 · keep pending?" stacked on one site.

- **Show the other end of a contract on the line, not in a card.** Clicking a belt end opens a card whose main action is "Jump to other end". Draw the far plant's name and revision at the far end of the line and let the click go there. The card then only has to show the evidence, which is the part a card is good at.

- **Let the queue say which plant, and only jump on Enter.** Hovering a row already outlines the site. Extending that to the keyboard, so arrowing through the queue moves the brass outline without changing the view, keeps the canvas as the map and the queue as the index. Enter is the jump. That is the interaction a five-repo engineer already expects from a build list.

- **Age the arrival.** "1 new" stays on the button until the row is opened. Stamp the arrival with the time it landed and let the mark fade to the ordinary row style after it has been on screen, so a later arrival can still say new.

## Earlier must-fixes

1. **The world level does not rank trouble. Fixed.** Chips carry the worst row kind. Kvarm and orrery-format read "✕ 1 blocks publication" in fail ink, Acquit reads "? 1 question +1", Orrery reads "◇ 1 recommended hold" in a dashed chip, and orrery-site reads "nothing waiting". The scope line reads "5 plants · 2 block publication · 5 waiting · 1 stale", and it becomes 6 waiting after the arrival. The bezel sits on the freshness line, not on the condition. `/tmp/xai-world/world-01-overview.png`, `/tmp/xai-world/xai-first-impression.png`. The shipped steps assert the chip kinds and the scope line, and both passed.

2. **Footprints are not recognizable. Fixed.** Five different scenes render at one projection. Orrery is a six-machine housing, Acquit is the catalog plant, Kvarm has a Worker, a D1 tank, an Artifacts shelf, and a train, orrery-format is a vertical stack ending in a gate, and orrery-site is an intake, a rack, and a CDN splitter. Grayscale at thumbnail size, all five stay distinct. `/tmp/xai-world/world-01-overview.png`.

3. **The plant level is zoom, not semantics. Fixed.** The plant stage is its own drawing, not a scaled world thumbnail. The summary is gone, strokes do not scale, and neighbors are edge labels with a direction mark ("01 Orrery ↗"). `/tmp/xai-world/world-07a-plant.png`, `/tmp/xai-world/xai-kvarm-plant.png`. What remains is the connector routing in must-fix 1, which is a new defect, not this one.

4. **The queue covers the trouble. Partly fixed.** The queue is docked in the right column and the world refits to the free width, so no plant label sits under it. `/tmp/xai-world/world-02-queue.png`. The arrival row carries a "new" mark, and so does the button. `/tmp/xai-world/world-12-arrival.png`. The part that is not fixed is the scrolling. The top row clips, and "1 new below" shows while the new row is on screen. That is must-fix 2.

5. **Collisions. Fixed, with one new collision class.** The minimap reads five separate names, Orrery, Acquit, Kvarm, format, and site, at the 10 by 10 scale. `/tmp/xai-world/world-01-overview.png`. The package label no longer touches its chip. The contract list is gone. The inspector, the queue, and the network panel share one right column and never stack. The new collision is the connector against the label, tracked as must-fix 1.

6. **Announcements never clear. Partly fixed.** The timer clears the line after 2.6 seconds. The boot toast is gone by the first shot, and "Place restored" does not linger. `/tmp/xai-world/xai-first-impression.png`, `/tmp/xai-world/xai-after-toast.png`. Panning no longer announces "Cross-boundary edits are cancelled". The part that is not fixed is the single slot. The arrival message replaces the message for the action you just took. That is must-fix 3.

7. **The Kvarm pill flips a hard switch. Fixed.** Clicking the pill opens the network panel and changes nothing. Going offline from the panel with no prior send creates exactly one held write, and a second offline cycle does not add another. The pill reads "Kvarm offline · 1 write held · 40 s" at boot and "Kvarm stale · 2 missed polls · 31 s" under `kv=stale`. `/tmp/xai-world/xai-kv-boot.png`, `/tmp/xai-world/xai-kv-panel.png`. Ctrl K offers "Reconnect Kvarm" while offline and "Go offline" while connected. The shipped steps confirm the held write is not doubled, and the mouse path confirms a click leaves `pending` empty until an action.

8. **Key and history bugs. Fixed.** Esc closes Ctrl K and returns focus to the opener. Enter on a focused result jumps. Back and forward restore a pan (`concepts/world.steps.json`, the drag then Alt Left check, which passed). A fifth bookmark is refused, and the toast says "Four places are already marked. Nothing new was saved." `/tmp/xai-world/world-17-bookmarks-full.png`. Scroll changes one level per gesture and uses the site under the pointer. The shipped wheel steps passed, including the one that stays on the plant when the fling is too small. `paint` restores focus by key instead of rebuilding the focused node.

## What I drove

- `node build.mjs world` wrote `live-world.html`.
- `node drive.mjs live-world.html "c=world" concepts/world.steps.json /tmp/xai-world` printed OK, 0 page errors.
- Probes in `/tmp/xai-world-steps/`, shots in `/tmp/xai-world/`. `plants.steps.json` and `queue.steps.json` failed on the defects above. `first.steps.json`, `keys.steps.json`, `mouse.steps.json`, `light.steps.json` (hash `c=world&theme=light`), `kv.steps.json` at `kv=offline` and at `kv=stale`, and `edges.steps.json` printed OK.
- `drive.mjs` has no viewport env var. The small-viewport probe was skipped.
