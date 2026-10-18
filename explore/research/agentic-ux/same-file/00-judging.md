# Same-file co-editing: judging the bakeoff

Two model families got the same prompt and the same three research reports. Opus wrote `invent-anthropic.md`. Grok wrote `invent-xai.md`. This note judges them by convergence and states the recommendation.

## What both arms reached independently

| Idea | Opus | Grok | Status |
|---|---|---|---|
| Keyed contributions for registries, imports and route tables | Slots | Stencil | Converged. Both route the registry class here. |
| An edit admitted against the blob the agent read, refused or replayed when the head moved | Turnstile | Loom | Converged on the mechanism. They split on whether it is enough. |
| No live character buffer | not proposed | Glass, rejected | Converged. Both cite m1:24 (convergence is not working code). |
| Conflicts as pins on the plant, settled with Shift keys, checks bound to the exact composed tree | yes | yes | Converged. Both reuse CORE's Settle verb and the train. |
| "Advisory" until exact-tree checks pass | yes | "merely accepted does not publish" | Converged. |

[inferred] Agreement across families is the strongest signal this run has. Slots/Stencil and read-blob admission are the two pieces with the most support.

## Where they split

[inferred] Opus picks Tenure. Ownership of syntax regions is decided before code, on the plant, as a clash. The train splices each region from its owner. Its weakness, by its own account, is that agents drift out of their regions, and a lease cannot prove a push stayed inside one (invent-xai.md, Loom protocol, citing m2:81).

[inferred] Grok picks Loom. Each edit is admitted after the fact against its read blob and woven. Its weakness is the one Opus names for Turnstile. It fixes mechanics and leaves meaning untouched, and the developer has nothing to decide until a conflict already exists (invent-anthropic.md, Turnstile).

[inferred] The two picks fail in complementary places. Tenure is a plan-time decision without write-time proof. Loom is write-time proof without a plan-time decision. CORE's rule is "settled before code", so plan-time ownership is the product. Write-time admission is the enforcement it lacks.

## Recommendation (final, after the replay)

[measured] The history replay (`m4-replay-results.md`) settles the split. Across 1,137 same-file pairs in Kvarm, Hono and tRPC, line merge conflicts on 53.47%. Only 0.18% are region-disjoint and still conflict, far under Tenure's 5% kill line. Line merge already accepts 98.51% of region-disjoint pairs. Of the conflicts, 63.16% sit in commons (imports, registrations, module-level statements), and 36.51% overlap a declaration. I reran the script, and it reproduced `results.json` byte for byte.

[inferred] So the value is not in splicing regions. Line merge already does that. The value is in knowing, before code, which pairs will touch the same declaration. That is where the hard conflicts live, and CORE's rule is "settled before code". The design shrinks to this.

1. **Default stays separate workspaces.** Kvarm's line merge and exact-tree checks already handle disjoint same-file work. Co-editing remains the exception, not the premise (m3).
2. **Declaration-level clash forecast, advisory.** Footprints gain named regions (function, method, class member), anchored by qualified name. Two declarations on the same region raise a clash pin, settled with the existing verbs. Region-disjoint edits in the same file raise no pin, which removes the path-level false alarm (134 of 1,137 pairs here). Commons overlap shows a faint mark, because most of it line-merges or is a keyed list. Acan computes this from `get_file_diff` and the declarations. It needs no Kvarm change.
3. **Read-blob admission stays an upstream ask.** An edit that carries the blob it read, refused or replayed when the head moved, is still the right fence against lost updates. The replay does not test it. It joins the Kvarm gap list beside the readback precondition on `claim_task`.
4. **Slots as a narrow pilot only.** The strict detector found 9 keyed-list candidates, 1.48% of conflicts, but 3 of Kvarm's 10. Pilot it on declared import blocks and Kvarm's registry lists. Do not build a general contribution-file system.

**Killed.** Tenure's region ownership, region leases and region-splice publication. The replay shows they buy nothing over line merge on this corpus.

## What would change it

- [inferred] The corpus is human merge history, weighted by large release merges (one tRPC merge is 20.93% of pairs). Live agents may collide differently. The dogfood self-loop should log same-file pairs with region overlap from week 2, and this verdict should be rechecked on that data.
- [inferred] If live data shows region-disjoint conflicts above 5%, Tenure's splice comes back for review.
- [inferred] If agents' commons conflicts turn out to be mostly keyed additions, Slots widens.
## Biggest risks

- [guess] Agents leave their regions often (a new import, a helper). Opus sets the kill line at 20% of checkpoints refused.
- [inferred] Tinted bays look safer than the code. A behavior change behind an unchanged signature passes every contract check, so a bay may say "checked on tree <sha>" only after exact-tree checks pass.
- [inferred] Writes that skip admission (shell, formatter, direct push) recreate lost updates. If the slice cannot refuse them on a covered path, it does not ship.
