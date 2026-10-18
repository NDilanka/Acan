# World prototype critique

## Verdict

Not ready. The plumbing is sound, and the canvas is clearly about systems, not agents. The world level fails its main job, though. You cannot tell at a glance where the trouble is. Four footprints are generic, and the plant level is a magnified thumbnail. Your notes hold up, except that light mode works. Its problem is type size, not color.

## Must fix (ranked)

1. **The world level does not rank trouble.** In `world-01-overview` and `world-13-light-overview`, every site shows the same "N waiting" chip. Kvarm and orrery-format block publication, and nothing on the canvas says so. The condition bezel draws an unrelated binding. Kvarm shows a teal "0" next to "Review blocks publication", which reads as healthy. Fix: let the chip carry the worst row kind, so "1 blocks publication" gets fail ink and a cross, and holds get an outlined chip. Move the bezel to the freshness line. Change the scope line to "5 plants · 2 block publication · 6 waiting · 1 stale".

2. **Footprints are not recognizable.** One generator draws four plants (`world.mjs:23-60`). Orrery and Kvarm differ only by a raised slab. Acquit uses another projection (azimuth 35, not 42), so it reads as the only real plant. Fix: give each plant an authored `renderScene` scene with catalog assets, all at one projection and scale. Kvarm gets a Worker, a D1 tank, an Artifacts shelf, and the train as a belt with cars. Check that grayscale 64 px thumbnails stay distinct.

3. **The plant level is zoom, not semantics.** `plantView` scales the world DOM by 2.2 (`world.mjs:200-203`, `603-609`). In `world-07a-plant`, the summary fills the bottom 40% of the frame. In `world-15-light-provenance`, "01 Orrery" sits under the toolbar. In `world-mouse-06-contract`, scaled belts cut through the contract list. Fix: at the plant and machine levels, hide the summary, use non-scaling strokes, and clamp neighbors to edge labels ("01 Orrery ↗").

4. **The queue covers the trouble.** In `world-02-queue`, it covers Acquit and orrery-format. In `world-12-arrival`, the new row lands below the fold with no "new" mark. Fix: dock the queue in the inspector's right column, and fit the world to the remaining width when U opens it. Hovering a row should outline its site.

5. **Collisions.** The minimap shows "OrreryKvarm" because x is divided by 16 and y by 10 (`world.mjs:150`, `179`). Divide both by 10. The format chip touches "package · v0.3.2" (`world-01`), so move that label to 505,248. The contract list reads as part of Acquit, so delete it. The inspector hides the canvas tools (`world-03-jump`) and clips "Agent evidence" (`world-07-levels`). The network panel stacks on the inspector (`world-10-offline`). Use one right column.

6. **Announcements never clear.** `announce` (`world.mjs:114`) has no timer. The F8 toast lingers in `world-01`, and "Place restored" lingers in `world-13`. Every pan claims "Cross-boundary edits are cancelled" (`world.mjs:469`). Fix: clear after 2.6 s, as `toast()` does, and drop the pan message.

7. **The Kvarm pill flips a hard switch.** One click on the readout goes offline (`world-mouse-10-offline`). The pill hides held writes, ages stay frozen at "8 s", and "3 repos" appears twice. Going offline always clones `D.pending` (`world.mjs:270`), so a second cycle sends `record_decision` twice. Ctrl K still offers "Go offline" while offline (`world.mjs:316`). Fix: the pill opens the panel and reads "Kvarm offline · 1 write held · 40 s". Only actions create pending writes.

8. **Key and history bugs.**
   - With the network panel open, every Enter is eaten (`world.mjs:443`), so its buttons cannot be pressed with Enter.
   - Esc in Ctrl K only blurs the field (`world.mjs:418`). The palette has no focus trap, and Enter ignores the focused result.
   - Back and forward skip `save()` (`world.mjs:384-385`, `414`), so Forward loses pans and mode changes.
   - A fifth bookmark is dropped, yet the toast says "Place marked" (`world.mjs:282-283`).
   - Scrolling in always opens Acquit (`world.mjs:454`), and one trackpad fling chains through levels. Use the site under the pointer, one level per gesture.
   - `paint()` rebuilds the sites on every key (`world.mjs:136`), which drops focus.

   F8 with the search field focused works.

## Should fix

- Delete the hero ("Your world."). It is marketing, and k:209 gives the title block that job.
- Jumping costs more than a tab. The empty Ctrl K query lists no plants (`world.mjs:306-309`), and search never returns contracts. Add Plants 01 to 05. Alt 1 to 5 as direct jumps needs your key approval.
- The stale strike-through reads as deleted, so show the dimmed value with its age. The teal "0" reads as healthy, so use neutral ink. The stopped detail should say "stopped · exit 0".
- In light mode, mono labels run 8 to 10 px. Faint ink passes at 5.21:1, but l asks for bigger microtype. Set an 11 px floor.
- The cutaway in `world-07-levels` is a box with a plate. It needs its internal stages and a bound gauge.
- The verifier draws as fog while the inspector says "valid". Label both on the canvas.
- The contract far end does not highlight the receiver, and the card covers it (`world-mouse-06-contract`).

## Keep

- There are no agents or terminals on the canvas. Bots live in the inspector, which keeps clear of Maestri and VibeCraft.
- Sites stay fixed, and the arrival neither moves the camera nor steals focus.
- Belts use distinct dash patterns with text and declared evidence.
- The provenance text matches the binding exactly (`world-09-provenance`).
- The network panel shows the origin, held writes, bytes with reasons, and a reconnect that waits for Enter.
- Back restores the camera, selection, and mode.
- The light palette follows l, and the drafted look should stay.
