# Clash on the belt: UX critique

## Verdict

The loop is right and the plant language holds: brass pins, a red chip for the broken witness, a ghost port only after the split. It fails the bar at the moment the loop is decided. The card leads with two declarations and buries the consequence, and a settled clash cannot say what was decided. Reopening the card covers the mark. Settling before code only works if the settlement stays readable on the machine.

## Must fix

1. The consequence is a caption under the declarations. In `clash-04-belt-card.png` the card opens on two grey panels, "Retry x2 with jitter" against "Breaker trips after 5 fails in 30 s", and the sentence that matters sits below in 12px grey: the open breaker marks p-2307 failed instead of pending. The red chip `breaks w3 · p-2307 stays pending` is the third thing down, and four buttons push Shift 3 and Shift 4 below the fold. The audit in `clash.steps.json` records `card scrolls` and still expects no issues. `clash-06-layer-card.png` is worse: both sides write the same path in the same grey, and the only difference, a `payout_state` column against a `traceparent` column, is a 10px line. "Serialize, radar-otel first" is justified by "paypal-payouts already depends on radar-otel" (`core-data.mjs`, c2), but nothing on the plant shows that dependency, so the suggestion reads as a guess. Fix: put the consequence and the breaks chip in the header, in the fail red, and collapse the declarations. Lead each with the word that differs. Name the dependency, or drop "suggested". Size the card so Shift 1 to 4 show without a scrollbar. The card is one string at `clash.mjs:74`, so this is a reorder.

2. A settled clash is illegible, and reopening it hides the evidence. After Split, `clash-05-split.png` shows a 9px teal pin labeled "Split at the port" with a leader to a ghost box the user has to hunt, while the order plate floats mid-screen on its own leader. After Retry wins, `clash-10-retry-wins.png` dims the breaker to 20% and parks "breaker · waits" at the top left, so the machine that yielded is the one the user cannot read. After Serialize, `clash-12-serialize-belt.png` puts the "after" label at the top left, off the belt. Merge joins the territory names but leaves the mark reading "Merge the territories" (`clash-14-merge.png`), the verb rather than the result. Reopen makes it worse. `open()` calls `flyTo` (`clash.mjs:96`), which frames the anchor for a 420px inspector (`choices.mjs`). The card is 720px, so in `clash-09-reopen.png` and `clash-24-keyboard-reopen.png` it covers the port plate, the pin and the belt. Fix: one short noun on the machine, "port v1", "waits", "after radar", "owner Droid", and do not dim the thing the decision names. On reopen, frame the anchor just left of the card and keep the current mark visible beside it.

3. The top of the screen is four stacked bars, and the keyboard path lies about one of them. `clash-01-claimed.png` stacks the mode switch, the hint, the concept switch and a toolbar at top 139 that repeats the same keys. The hint has no separator after V, so "V NEXT CLASH" reads as one verb. The toolbar is a second hint sitting on the plant. Worse, `open()` calls `setMode("live")` while the card is open (`clash.mjs:73`), and the handler only stops Shift-digit and Esc (`clash.mjs:238`). Keys 2, 3 and 4 change mode and leave the card up. `clash-25-top-layer.png` is the same frame as `clash-24`: L is swallowed, so Layers never opens, and Esc is claimed by the card first (`clash.mjs:235` returns only when Layers is already open). The shot that should prove top-layer Esc proves the card ate the key. Fix: one hint line, no toolbar. Put the count on the pin, "2 before code", and turn it teal only when both are settled. While the card is open, stop every mode and panel key.

## Should fix

Ctrl Z undoes and closes without reopening the card, so the user undoes blind (`clash-08-undo.png`). The stack keeps the previous kind (`clash.mjs:53`), so a second undo restores a settlement. Reopen the undone card, and say so in the toast.

I only opens the next unsettled clash (`next()`, `clash.mjs:44`). The toast tells the user to click a settled mark, which leaves no keyboard path back. Let I cycle settled marks once nothing is open.

The advisory clash is a dashed outline on acquit-api (`clash.mjs:174`) with its text in an SVG title, so no shot names it. One chip, "advisory · server.ts, different routes", matches the spec without a third pin.

Footprint labels name the agent, not the machine. In `clash-02-footprints.png` the same label sits on retry and sqlite. Put the machine first. The plate also quotes the placeholder as the posted sentence (`clash.mjs:125`). Store the posted text.

`clash-all-02-blueprint.png` still says LIVE in the status bar. That is ui-v1, and the inactive concept leaves no chrome, so it is a note.

## Keep

The pins read as part of the machine, and brass against the teal mark is the right polarity. "Codex cannot see orders. Passive session, no MCP." survives every shot. The source chips, declared, static, history, show on every write. The toast "Settled before code. No file was edited." is the sentence the concept exists to earn, and the ghost port is the right answer to Split. Spend the same care on the other three.
