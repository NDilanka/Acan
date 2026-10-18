# Work orders prototype: UX critique

Scope: `CORE.md`, `00-judging.md`, `concepts/orders.mjs` (read in full), `core-data.mjs`. Frames viewed this session: `orders-06` to `10`. Frames `01` to `05` and `11` to `16` returned no pixels. Findings resting on them are marked "code".

## Verdict

The loop reads as a plant: witnesses on machines, a plate that says OPEN, the question at sqlite, the claim on the dock. It fails at the moment the concept exists for. A losing readback is painted in the success colour, and the disagreement is named in grey. The claim fires 500 ms after the answer with no evidence on the plant, and the mouse path ends with the floor list contradicting it. Fix disagreement legibility and claim evidence first.

## Must fix (ranked)

1. **A losing reading is shown in the success colour.** In `orders-10`, the claimed plate (about x 1270-1580, y 622-762) shows "matches · claimed" and "yields" in the same teal. `orders-08` and `orders-09` show "p-2307 differs" in teal too. Cause: `orders.mjs` L69 sets `.orders-mini small{color:var(--ok)}` for every state. L153 renders the p-2307 token as `data-result="pending"` for both agents, so Droid's "failed" in `orders-07` is amber. **Fix.** Add a `data-state` to each dock row. Colour matches ok, differs and pending wait, yields and failed fail. At L153 use `"fail"` for the Droid reading.

2. **The claim is not earned on the plant, and the floor contradicts it.** `orders-09` shows "Answer pinned as w3. Readbacks re-scored." while the plate still reads OPEN. L277 fires `ordersClaim` 500 ms later, so the re-score is never on screen. The 3 of 3 result appears only in the toast (`orders-10`). L289 updates only the Claude Code floor row, so a Droid claim leaves "Droid verifying otel radar" unchanged (code). **Fix.** Hold the claim about 1 s and show "3 of 3 · claiming" on the plate. Update the claimant's own row by `winner.agent`. Name the session on the plate and in the toast.

3. **An empty sentence posts the fixture text.** Code only. L235 replaces an empty sentence with `ORDER.placeholder`, which is also the draft placeholder (`core-data.mjs` L42). L178 enables Post once both witnesses exist. **Fix.** Disable Post until `sentence.trim()` is non-empty. Delete the fallback at L235.

4. **The draft sentence hides its start (known issue).** Code only. L176 renders a single-line `<input>`. **Fix.** Use `<textarea rows="2">` with wrapping. Enter still posts (L383).

5. **The question repeats its verb and hides the stakes.** In `orders-08`, "Shift 1 Answer · Keep it pending, settle once after PayPal recovers" wraps under the kbd tag. L169 builds "Answer ·" into every option, and which agent reads which way lives only in the footnote. **Fix.** Options read "Keep it pending · Claude Code reads this" and "Mark it failed and refund · Droid reads this".

6. **The conflicting clause is not located.** In `orders-07`, Claude Code says "keep it pending" and Droid says "mark the payout failed". Only tokens show the conflict. **Fix.** Add a `conflict` substring per READBACK in `core-data.mjs` (shared file, so report it to the builder) and wrap it in `<mark data-result="fail">`.

7. **The guide is generic dashboard chrome.** The "Point to what matters." panel (x 20-308, y 130-482) stays on screen in Live and after the claim (`orders-06` to `10`). CSS L17 has no hide condition, and its key list repeats the hint that CSS L10 hides. **Fix.** Show it only while the order is a draft.

## Should fix

- **The plate disappears while the card is open** (code L183). Make the plate the card header.
- **The question pin overlaps a chip.** In `orders-07`, the QUESTION pin (about x 467-681, y 388-411) covers the corner of the MUST NOT chip (x 667-786, y 372-393). CORE forbids overlaps. Call `layoutChips()` after placing it.
- **Camera framing is decided by code.** `ordersFocusMachine` (L122-135) pans on every open, including witness pins. Pan only when the anchor is off screen.
- **Off-screen anchors snap to a corner** (code L313-314, L318-319). Show an edge arrow with the machine name.
- **"Waiting on you 7" stays 7** while the question is open (`orders-08`). Increment it.
- **Answer pins look like replay pins.** The w3 check reads "replay 15:45" (`core-data.mjs` L31). Use "answered at sqlite".
- **The pin gate is toast-only.** L220 needs the exact minute and a selected machine, yet the button looks enabled. Show the reason inline.
- **Keyboard and mouse Post differ.** L303 refuses P without a paypal selection. L351 selects paypal on click.

## Keep

- Witness chips: red MUST NOT and teal MUST KEEP, with clock, on the machine (`orders-07`, `orders-10`).
- Locked checks in the card, with the reason field the only editable text before Post.
- Pass and fail tokens in the card (`w1 pass`, `w3 fail`).
- The footnote and "Codex cannot see orders. Passive session, no MCP."
- Keyboard: `ev.code` Digit1 and Digit2 (L388), capture listener, Esc layering (L382, L387).
- Scoped CSS under `body[data-concept=orders]`, so the inactive build is untouched (CSS L11).

## Open

- Not viewed: `orders-01` to `05`, `11` to `16`. The Droid claim path rests on code (L289, L290).
