# How coding agents collide inside one file

## Six-line summary

[inferred] Sharing a pathname is not the same as needing shared mutable state.
[sourced] [Claude's current Edit rules](https://code.claude.com/docs/en/tools-reference#edit-tool-behavior) already permit some exact, disjoint edits after external changes.
[inferred] A write broker can prevent lost updates, but cannot reconcile incompatible plans.
[sourced] [Agent PR research](https://arxiv.org/html/2607.04697v2) finds substantial textual conflict rates, not a measured benefit from live co-editing.
[inferred] Registries often need keyed additions, generated files need one producer, and shared functions usually need one implementation owner.
[guess] Acan's best first experiment is narrower overlap classification, not a universal multiplayer file editor.

## How the agents edit files

### Claude Code: exact replacement, with version-sensitive guards

[sourced] [The official tools reference](https://code.claude.com/docs/en/tools-reference#edit-tool-behavior) describes `Edit(file_path, old_string, new_string, replace_all)`. Matching is exact, including whitespace. The old string must occur once, unless `replace_all` is enabled. A changed target can therefore produce a failed match. Repeated boilerplate can produce an ambiguity failure.

[sourced] The same reference qualifies “must read first.” Before v2.1.208, unread files and files changed since the previous read were refused. Current behavior permits newer models to edit unread or externally changed files under documented conditions. The replacement must match current content exactly and unambiguously, and reading must not require another permission prompt. Older models retain stricter read requirements. Acan must inspect the deployed version and model, not hard-code historical behavior.

[inferred] Under strict guards, another agent changing a distant function can invalidate the whole-file read. Under relaxed guards, that distant edit need not block this replacement. Neither policy proves the agent's reasoning remains valid. An unchanged call site can still depend on a changed contract elsewhere.

[sourced] Historical `MultiEdit` bundled sequential replacements within one file. A [community reconstruction](https://github.com/moltis-org/moltis/issues/657) describes all-or-none batch application. This is weaker evidence than current vendor documentation. An [Anthropic-repository report for v2.0.32](https://github.com/anthropics/claude-code/issues/11125) says MultiEdit was missing. An [earlier report](https://github.com/anthropics/claude-code/issues/3513) records Edit and MultiEdit refusing files “modified since read.” Treat this as historical behavior, not a current availability guarantee.

[inferred] Batch atomicity can prevent half a rename from landing within that batch. It does not establish locking against a second process, repository-wide transactions, or semantic agreement.

### Codex: contextual patches

[sourced] OpenAI's [Codex prompting guide](https://developers.openai.com/cookbook/examples/gpt-5/codex_prompting_guide.md) documents the `apply_patch` interface and patch grammar. The [apply-patch guide](https://developers.openai.com/api/docs/guides/tools-apply-patch) describes create, update, and delete operations, contextual diffs, `Invalid Context` errors, and recovery by rereading. The API guide separately says the integrating harness chooses atomicity. Do not transfer an API example's guarantees wholesale to the CLI.

[inferred] A patch built from a stale read can still apply if its target and required context survive. Changed context can make it fail even when intended edits concern different syntax elements. Successful matching does not detect a conflicting assumption outside the patch. Expected text is a local precondition, not a revision check over everything the model considered.

### Droid: inspect the actual tool surface

[sourced] Factory's [custom-droid documentation](https://docs.factory.ai/cli/configuration/custom-droids) lists `Create`, `Edit`, and `ApplyPatch`. It automatically includes ApplyPatch when Edit is selected with OpenAI models. Its [Exec documentation](https://docs.factory.ai/droid-exec/overview) provides model-specific `--list-tools` and tool restrictions.

[inferred] This session exposes ApplyPatch and instructs reading existing files before updates. Public pages found here do not specify Droid Edit's matching algorithm, stale-read enforcement, or cross-process atomicity. “Droid edits are concurrency-safe” is therefore unestablished. Acan needs a capability record per installed harness. String replacement, contextual patching, and whole-file replacement are different operations, even when their UI labels all say Edit.

### Whole-file writes and the actual race

[sourced] Claude's [reference](https://code.claude.com/docs/en/tools-reference#edit-tool-behavior) distinguishes full replacement with Write from exact replacement. It also notes that Edit ultimately saves the whole decoded file. A small model-generated edit is not necessarily a small filesystem write.

[inferred] Suppose A and B read revision zero. A adds logging. B writes its complete revised copy from revision zero. Without a freshness check or merge, A's logging disappears silently. Whole-file shell writes, generators, formatters, and replacement tools all need coverage. It would be wrong to claim every Claude Write bypasses harness guards.

[inferred] Even precise edit tools can lose updates if two separate processes read current bytes, compute replacements, then overwrite each other's results. A broker must make validation and publication one guarded operation. An atomic rename prevents a torn file, not a lost update. A watcher that notices writes afterwards is not such a broker.

[inferred] Finally, an agent may see another agent's unfinished code, diagnose its temporary failure, and “repair” it away. File convergence does not prevent this feedback loop. Agents need published checkpoints and dependency intent, not just fresh bytes.

## What empirical conflict studies actually measure

[inferred] Three populations must stay separate. Same-file co-change includes independent edits that merge cleanly. Textual conflicts concern overlapping or adjacent merge regions, plus file-level structural cases. Semantic conflicts concern incompatible behavior, with or without text overlap. These populations do not form a clean three-way partition.

[sourced] [Brindescu et al.](https://ics.uci.edu/~iftekha/pdf/J4.pdf) analyzed 143 projects and 36,122 merge commits. They report 19.32% conflicting merges and say 75.23% of conflicts required reflection on program logic. Their AST-based categories must not be read as a measurement of “compiles but breaks.” Requiring reasoning during textual resolution and observing a runtime integration failure are different findings.

[sourced] [Ghiotto et al.](https://leomurta.github.io/papers/ghiotto2018.pdf), published in TSE in 2020 after its 2018 acceptance, studied 2,731 Java projects. Among 175,805 conflicting chunks, 94% had at most 50 lines in each side, 68% at most ten, and about half at most five. In the 616-chunk manual sample, 81% of resolutions used existing lines without adding or editing them. Small conflicts need not be independent conflicts. Selecting which side survives can require substantial context.

[sourced] [Brun et al.](https://people.cs.umass.edu/~brun/pubs/pubs/Brun13tse.pdf), Figure 4 and Section 4.2, report approximately 16% textual failures, 1% build failures, and 6% test failures in the three projects with runnable test suites. Across those projects, 266 textual conflicts represented 67% of detected conflicts. Thus roughly one third of detected conflicts were higher-order. This does not mean one third of all clean merges fail. Test failures also undercount semantic problems not covered by tests.

[sourced] The [LastMerge evaluation](https://arxiv.org/html/2507.19687v1) replayed 5,229 scenarios involving 13,675 mutually modified files. It compared generic structured tools with Java-specific counterparts. Mergiraf had 42% fewer added false negatives than Spork in that comparison. This is not “42% of Git conflicts disappear.” Its relative accuracy measures do not estimate the benefit of sharing a live file.

[sourced] [Mergiraf's examples](https://mergiraf.org/conflicts.html) explain the relevant distinction better than a headline percentage. Independent syntax elements can merge despite sharing a line. Unordered member additions can merge. Statement additions remain conflicted when order matters. Duplicate JSON keys can be detected even when Git's line merge succeeds.

[inferred] No retrieved study supplies a comparable fraction for same-line, same-file-different-region, and semantic collisions across these three agents. Git conflict corpora condition on conflicts, so they omit many harmless same-file edits. Runtime defects require different detection. A credible Acan estimate needs all attempted overlapping tasks as its denominator.

## Hotspot files and why they attract collisions

[inferred] Routers and registries centralize independent additions at one insertion point. Barrel exports and import blocks do the same near the top or bottom of a module. Their text overlap can be accidental. Their semantics are not always commutative: duplicate names, route precedence, side-effect imports, and initialization order can matter.

[sourced] Ghiotto's [automated analysis](https://leomurta.github.io/papers/ghiotto2018.pdf), Table 8, identifies import-only conflicts as approximately 11% of classified conflict kinds. This supports specialized import handling, not the claim that most conflicts are imports.

[inferred] Shared configuration centralizes both independent keys and mutually exclusive decisions. Different keys can still interact. A database URL and a selected driver can be individually valid and jointly incompatible. Migrations introduce ordering and schema invariants across separate files. Splitting migration files alone cannot remove that coupling.

[sourced] A [practitioner account](https://haacked.com/archive/2026/03/25/resolve-merge-conflicts/) treats lockfiles and migrations as distinct resolution categories. It recommends regenerating lockfiles through the package manager rather than deciding their lines manually. This is operational advice, not prevalence evidence.

[inferred] Generated clients, schemas, snapshots, and lockfiles should usually have one publication step after source inputs are integrated. Splitting their outputs among writers creates unnecessary shared state. Routers, registries, barrels, config, migrations, and generated files are a useful inspection list, not a verified universal ranking.

[sourced] The [agent PR-pair study](https://arxiv.org/html/2607.04697v2) finds 84.4% of conflicted files were source files. Nearly 42% of its classified conflicts were structural, principally modify/delete and add/add. Those findings oppose a design aimed only at lockfile noise or overlapping text spans.

## Multi-agent coding reports and measured rates

[sourced] Isolation is documented practice across the products. [Codex worktrees](https://developers.openai.com/codex/app/worktrees) provide separate file copies for independent tasks. [Droid's worktree flag](https://docs.factory.ai/reference/cli-reference) gives parallel sessions dedicated checkouts and branches. [Claude agent-team guidance](https://code.claude.com/docs/en/agent-teams#avoid-file-conflicts) explicitly warns of overwrites and recommends different file ownership.

[inferred] Parallel subagents are a context-management feature, not automatically a filesystem-isolation feature. Worktrees remove immediate clobbering and exposure to unfinished peer edits. They defer integration conflicts rather than eliminate them.

[sourced] Anthropic's [parallel C-compiler report](https://www.anthropic.com/engineering/building-c-compiler) describes 16 agents converging on the same Linux-kernel blocker, fixing the same bug, and overwriting each other's changes. Its remedy was a GCC oracle and a harness that exposed separable failures. This is direct evidence that discovering independent work can matter more than improving write arbitration. The report provides no normalized collision rate.

[sourced] Cursor's [initial scaling report](https://cursor.com/blog/scaling-agents) describes a shared coordination file whose locks stalled twenty agents to the effective throughput of two or three. Agents held locks too long, failed to release them, and sometimes wrote without locks. Optimistic concurrency helped mechanics but not the deeper work-allocation problem. This was a coordination-state hotspot, not a controlled test of source-region leases.

[sourced] Cursor's later [swarm report](https://cursor.com/blog/agent-swarm-model-economics) reports workers overwriting peer changes or abandoning their own during conflict resolution. Its redesigned system uses a neutral conflict resolver and splits bloated files. One old run accumulated over 70,000 conflicts before being paused. A new run logged fewer than 1,000 across four hours. The hottest old file collected 7,771 conflicts; the hottest new file saw 47.

[inferred] These are workload-specific counts, not conflicts per attempted merge. Several harness changes happened together. They support investigating architectural hotspots, but do not isolate the causal benefit of file splitting.

[sourced] [AgenticFlict](https://arxiv.org/html/2604.03551v1) processed 107,026 open or closed-unmerged agent PRs from a larger collected corpus. It found 29,609 conflicting PRs, or 27.67%, and 336,380 conflict regions. Its comparison is PR versus base-branch integration, not simultaneous agents editing one local file. Its selection excludes already merged PRs.

[sourced] The [separate PR-pair study](https://arxiv.org/html/2607.04697v2) collected 33,596 agent PRs across 2,807 repositories and replayed 747 co-active pairs. Textual conflicts occurred in 19.8% of intra-agent pairs and 41.7% of cross-agent pairs. “Intra-agent” means the same agent product, not necessarily one running process. Cross-agent pairs were rare in the underlying corpus. These observational rates do not prove vendor mixing causes conflicts, or predict a supervised Acan session's rate.

## Taxonomy and the best-fitting remedy

[inferred] The table gives design judgments, not measured treatment effects. “Live co-edit” means convergent text operations. “Region lease” means enforced, revision-aware ownership of a stable region, not a promise in a prompt.

| Collision | Best first remedy | What same-file machinery removes | What remains |
| --- | --- | --- | --- |
| [inferred] Independent registry, import, or barrel additions | Structural split into feature-owned contributions; otherwise a keyed append-merge driver | Shared insertion-point text conflicts and duplicate mechanical work | Duplicate keys, incompatible names, side effects |
| [inferred] Ordered routes or middleware additions | Sequential micro-turns for registration; publish precedence rules | Simultaneous insertion races | Which route or effect must run first |
| [inferred] Disjoint functions within a cohesive file | None if existing patches compose safely; otherwise region leases | False whole-file exclusion and guarded-write races | Shared helpers, formatting churn, stale assumptions |
| [inferred] Independent syntax changes on the same line | Structural merging, possibly live co-edit with semantic checks | Line-based false conflicts | Behavior not represented by syntax |
| [inferred] Two agents changing one function's algorithm | Split work into implementation and review; otherwise sequential micro-turns | Lost writes and interleaved publication | Competing intent and duplicate reasoning |
| [inferred] Cross-file semantic coupling | Split at an agreed contract; assign integration ownership | No file-only remedy | Schema, protocol, error handling, and invariant disagreement |
| [inferred] Lockfiles and generated outputs | One producer after input integration | Avoid concurrent output editing entirely | Dependency resolution and generator correctness |
| [inferred] Schema migrations | Split independent schema work; one ordering owner | Filename and publication races | Data compatibility and operation ordering |

[sourced] Git's [union-driver documentation](https://git-scm.com/docs/gitattributes) warns that taking both sides can leave added lines in random order. Append-merge must therefore mean a validated operation over a known collection, not “enable union everywhere.”

## Attack the premise before building it

[inferred] Ask what mutable object must be shared. Two feature agents may need the same registration capability without needing to rewrite the same registry file. Let each contribute a separate manifest. Let a generator or integrator own publication. Two agents addressing the same failing function may instead need one implementer and one independent investigator.

[inferred] Same-file work is defensible when the file is cohesive, edits are independently specified, contracts are stable, and extracting modules would create worse indirection. Examples include separate methods of a deliberately compact class or independent corrections in a specification-mandated single file. A short publication turn can preserve parallel reasoning without allowing simultaneous writes.

[inferred] Live co-edit becomes worth considering when necessary same-file work repeatedly loses time to harmless region overlap. It is not justified merely because multiple agents can reach the file. Text convergence neither updates model reasoning nor makes half-finished combined code a stable test target.

[inferred] Acan already distinguishes incompatible declarations before code in [CORE.md](/home/factory-user/acan/explore/ui-v2/CORE.md) and [00-judging.md](/home/factory-user/acan/explore/research/agentic-ux/concepts/00-judging.md). Keep that semantic layer. Classify each overlap as shared insertion, disjoint regions, shared implementation, or dependency contract. A same-file exception should narrow a clash, not erase it.

[guess] Instrument failed matches, rereads, lost updates, conflict-resolution time, duplicate fixes, and post-integration failures. Compare worktrees alone, structural splits, and narrowly brokered shared-file edits. Count writer tools, formatters, and generators. Without that baseline, a shared editor can make coordination look smoother while shipping the same contradictions faster.

## Verdict

[inferred] Do not make “multiple agents edit one file simultaneously” the product premise. Make safe independent progress the premise. Structural splits remove artificial central objects. Ownership and contract splits remove duplicated or incompatible work. A write broker removes publication races. None substitutes for the others. Co-editing is a justified exception for cohesive files with composable changes, not a cure for agents pursuing the same algorithm or disagreeing about behavior. Acan should first prove which clashes are merely physical, then optimize those without weakening its before-code decisions.
