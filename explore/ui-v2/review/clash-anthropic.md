# Clash on the belt: UX critique

## Verdict

Not ready. The loop runs end to end by keyboard and mouse: pin, card, settle, undo, reopen. The card does not lead with the consequence, the suggested settlement is not argued, and footprint mode is noisy. Two state bugs also need fixing.

## Must fix

1. **Make the consequence the hero, and fix the witness chip.** **Evidence.** clash-16 and clash-24. The title is 15px (`clash.mjs` L305). The consequence is 12px (L320), under two declaration boxes. The red chip `breaks w3 · p-2307 stays pending` is 10px (L321) and hard-coded at L76. It says p-2307 stays pending, while the sentence above says it is marked failed. The chip reads as the outcome, not the broken witness. **Fix.** Put the consequence in a 15px band under the header. Give the broken witness its own row, from its `check` field. Remove the literal `p-2307` from L76.

2. **Argue the suggested settlement, and show settled state by more than position.** **Evidence.** clash-16 and clash-24. The brass border on "Split at the port" comes from `:first-of-type` (L325), so it marks position, not suggestion. The card never explains why split wins for c1. The split detail (keeps unknown as pending) is the only rationale. Settled is a 9px tag (L330-331). **Fix.** Show a one-line `why` above the buttons for each suggested option. Style on `data-suggested`. Give the settled row a filled background and a check.

3. **Show that the blocking partners are not claimed declarations.** **Evidence.** In `core-data.mjs`, d2 and d3 have `order: null`. d3, the breaker write that drives c1, is a proposal. d2, which drives c2, is sourced from history. The Droid row reads "verifying otel radar" (clash-21, clash-22), yet the c1 card gives Droid's breaker write equal weight with Claude Code's declared writes (clash-16). CORE says each claimed agent declares. **Fix.** Tag unclaimed declarations `proposal · not claimed` or `history · not declared`, with a distinct border. Keep history inferences advisory, so they cannot block a settlement.

4. **Close the card on mode change, gate settle to Live, and stop no-op undo entries.** **Evidence (code only, no runtime repro).** `settle()` (L50-58) has no mode check and pushes an undo entry on every call, including re-picking the current option. `setMode()` (`build.mjs` L518-531) does not close the card, and no clash CSS rule (L265-343) is keyed to `data-mode`. Repro: open c1 (which forces Live at L73), press 4, press Shift 2. Replay's watermark says READ ONLY (clash-all-04). Re-picking the same option and pressing Ctrl Z prints "Undid the settlement" and changes nothing. **Fix.** Return early in `settle()` when the kind is unchanged, and unless the mode is live. Close the card on a `body[data-mode]` change.

5. **Shorten the serialize arrow and point it at the right thing.** **Evidence.** clash-07, clash-21, clash-22. The c2 arrow runs from `anchor("radar")` to `anchor("sqlite")` (L189), a diagonal across most of the scene. Its label says `radar-otel → paypal-payouts · after` (L191), but the arrow ends at sqlite. It stays with footprints off (clash-22). **Fix.** Draw a short arc at the sqlite layer ring labelled `radar-otel first`, or drop the arrow and put "after" on the pin.

## Should fix

- **Three stacked top strips.** The hint row (y≈46) is overdrawn by a machine (clash-04, clash-22). The OTEL RADAR chip overlaps "ESC CLOSE" (clash-21, clash-22). The hint row is shell-level (clash-all-01). **Fix.** Merge the concept chip and the toolbar into one row.
- **Footprint labels.** The data holds six writes (d1 3, d2 2, d3 1), not eight. clash-15 shows four. Labels without a free spot are hidden silently (L200-208). **Fix.** Show "6 writes · 4 shown", or scope footprints to the open clash.
- **Claimed plate moves.** L143-158 re-searches a spot on every paint. It sits bottom-centre in clash-05 and clash-17, and top-right in clash-07, clash-21 and clash-22. `.clash-foot-label` is absent from the obstacle list (L147). **Fix.** Pin the plate beside the paypal dock.
- **Waiting on you stays at 7** after every settlement (clash-05, clash-07, clash-21, clash-22). **Fix.** Show the clash count beside it.
- **Three names for c1.** The pin says "retry × breaker" (L111), the card "Retry meets breaker" (L76), the caption "api->paypal" (raw code, clash-04). **Fix.** One name, and a plain caption.
- **Duplicate owner text.** clash-13 repeats the owner on the pin and on the badge. Keep one.
- **Merge is not legible.** The OTEL RADAR chip is unchanged between clash-13 and clash-14. The ground label change (`scene.mjs` L83) cannot be confirmed at this resolution. **Fix.** Draw one outline across both territories.
- **Card covers the plant.** At 720px (L302) it hides the paypal dock and Codex (clash-16, clash-20). **Fix.** About 560px, beside the pin.
- **Verification gaps.** clash-25 is byte-identical to clash-24 (md5 `9b11a24f…`), so Esc top-layer has no evidence. clash-23 is named "escape" but shows the undo toast. Re-shoot both.

## Keep

- The card anchors beside its pin with a leader (clash-04, clash-16).
- Every write carries its source tag, declared, static or history (clash-16).
- Settled pins shrink to a check and keep their label (clash-09, clash-13).
- The port ghost "paypal.result v1 · owner Claude Code" reads clearly (clash-21).
- c3 is a faint dashed outline with no pin or count (clash-02).
- The keyboard path is complete: V, I, Shift 1 to 4, Ctrl Z, Esc, Enter on a pin (clash-24).
- The inactive build is clean (clash-all-01 to clash-all-04).
- `clash.mjs` has no timers (grep found none).
