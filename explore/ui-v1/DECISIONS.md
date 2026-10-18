# Acan v1 prototype · decision log (pages 8 to 10)

Each entry gives the call, why, and what would change it. Guesses are labelled.

## First run (`live.html#first=1`, `first.mjs`)

1. **Three steps in one left panel: Read, Place, Connect.** The plant stays in the middle the whole time, so you watch it fill in. The inspector side (right) stays free.
2. **Confident matches go straight onto the plant in grey (D10).** They look idle because no spans have arrived yet. Uncertain ones go to the unplaced tray (K5) as cards with confidence, the rule that fired, one question, and two answers.
3. **Y always means "agree with Acan's guess".** N picks the other answer. Each tray item has an outcome pair (`fx`), for example `["dock", "ignore"]`, so one table drives the plant, the yaml, and the wording. ctl and the :4320 lane are phrased so Y is still the guess.
4. **The focused guess is drawn as a ghost where it would go.** You see the consequence before you answer. Items with no position (the CLI, the lane) say "not drawn until you place it".
5. **Nothing is written until you continue.** The system.yaml draft updates live under the tray, and new lines are highlighted. Enter agrees with the focused guess. Once every guess is answered, Enter writes. Shift Enter writes early and leaves the rest under `tray:` in the yaml.
6. **Connect shows the exact lines it adds to each agent's settings file**, with a backup and "message content export stays off" (E1). Enter writes, C copies the lines instead, S skips. Each card also says what that agent can and cannot send. An earlier version said Codex has no MCP client. That was wrong, and Codex now gets an `[mcp_servers.acan]` table.
7. **The payoff is the first span.** The plant lights up with real states, so paypal is already red the moment Droid connects. The toast says so.
8. **Setting keys checked against the vendor docs** (proposal V3). Droid's `settings.json` has no `env` key, so its telemetry lines go in the shell profile, and MCP goes through `droid mcp add`. Claude gained `OTEL_METRICS_EXPORTER`, and its MCP server moved to `claude mcp add`, which writes `~/.claude.json`. Codex gained `trace_exporter` and `protocol`. The Codex `protocol` values and the Claude MCP file are still unverified.

## Friction pass 2 (DX rows 6 to 10)

9. **Blueprint from the keyboard.** A opens the catalog beside the selection and finds a free spot. Arrows nudge (Shift for bigger steps), Enter places, Esc cancels. B opens a belt picker with the nearest machine first, and clicking a machine still works.
10. **Ctrl Z on the blueprint** undoes the last file edit: adding a ghost, a belt, a reverse, or a removal. Delete now says "Ctrl Z brings it back".
11. **Choices.** Shift 1 to 3 picks an alternative. Ctrl Z undoes the last decision, including a reopened seal. Arrows only move. "Later" moved to D. An empty reject with Tab uses the suggested reason.
12. **Shift W opens the queue. J and K walk the floor lanes when the floor is open.** Mode keys ignore Shift, which was eating Shift 2.
13. **Simulate and Replay step with comma and period (a minute) and arrows (30 s).** The simulate brief's open link now carries the seed, the downed machines and the ghost switch, so a run can be shared.
14. **One verb set.** Open, Copy brief, Hand to agent. The blueprint's canned "Simulate" became "Quick estimate", so Simulate means only mode 3.
15. Bug found on the way: the review's back arrow used the class `nav`, which is the compass column, so it was fixed in place behind the compass. Renamed.

## Share-safe export (S3, R2)

16. **Export previews the redaction on the plant itself.** Names become kind plus number, dock faces become EXT, plinth text drops, territories become numbered, and the title block counts what was redacted. Toggle share-safe off and the real names come back.
17. **Counts only what is drawn.** Paths, ports and env names are not in this view, and the panel says so rather than showing zeros as if they were checked.
18. Saving is local. The note under the button says nothing is uploaded.

## Trust (Q16)

19. **The egress pill opens the network log.** Empty state: "Nothing has left this machine." Loopback listeners are listed apart, as not counted. The offline switch turns the pill to "offline". The data statement lists the three things Acan could ever send, all off or asking by default.
20. **Update checks are off until you turn them on** (proposal V6). This keeps the pill honest at "0 out" from first launch. The cost is that people stay on old builds until they opt in.

## Fog and far zoom (K12, A7, L21)

21. **Fog is a layer, off by default, in Layers and Ctrl K.** Each machine gets a mist patch scaled by its fog score (1 minus the mean of the six signals). Patches under 0.2 stay faint with no label, so the plant does not shout about small gaps.
22. **Fog grows back.** gate's seal is stale after 6 commits, so it is fogged again.
23. **Far zoom (under 55%) collapses chips to their badges** and hides fog captions. The zoom readout says "far".

24. **Replay briefs now cite only recorded evidence**: the change log and the events on that machine. They no longer mix in live inspector data.

## Labels

25. **Chips never overlap** (proposal V9). Every frame, visible chips are placed in order: selected first, then failing, then strained, then front to back. A chip that would collide lifts on a longer leader line in half-height steps, up to four chip heights. A chip that only shows on hover goes last, so hovering moves that chip and not its neighbours.

## Calls made after the first review

26. **Proposal location.** `acan/explore/proposals/ui-v1.md`, in map.md's row format, so map.md stays untouched and each row can be pasted in.
27. **D1 is amended, R1 is not** (proposal V1, V2). v0.1 ships the iso working view as the default and the flat plan behind T. R1 already says the plan is flat, so it only needs Top to reach a true 90°.
28. **One first-run layout, no competing variant** (proposal V5). The open questions there are copy and order, and a second layout would not answer them.
29. **Ctrl K answer area is v0.2** (proposal V7). v0.1 keeps the lexical jump.
