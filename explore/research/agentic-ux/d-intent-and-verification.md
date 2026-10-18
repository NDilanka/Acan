# Intent and verification beyond chat and diffs

[inferred] Orrery's strongest alternative is to make a runnable contract, rather than a conversation, the unit of delegation.
[inferred] Humans should point to examples, edit constraints, compare alternatives, and inspect consequences on the same persistent system drawing.
[inferred] The return object should connect each requested behavior to independent evidence and disclose what remains untested.
[sourced] Spec workflows, direct manipulation, live programming, and proof-carrying code supply precedents, not evidence that this combined workflow succeeds. [S1, S5, S20, S26]
[inferred] A plan approval should authorize a bounded experiment, not certify its eventual implementation.
[guess] The best bakeoff candidates are counterexample-directed repair, contract gestures, experiment branches, evidence-bound acceptance, and explicit verification holes.

## Scope and evidence

[sourced] This research is dated 2026-10-17 and uses the supplied product context, `map.md` lines 20 to 72, and the two supplied screenshots.
[sourced] Orrery already fixes budgets as objectives, reserves changes to budgets and checks for human approval, and ranks choices by `(1 - confidence) × reversal cost`. [Local grounding]
[inferred] The opportunity is to change how those rules become actions, rather than add another dashboard.
[sourced] WebSearch and FetchUrl were loaded, and source pages were read through WebSearch's full-page-text option.
[sourced] FetchUrl's tool policy permits explicitly supplied URLs, not newly discovered research URLs, so it was not used for those pages.
[inferred] Here, [sourced] means an opened source supports the statement, [inferred] means a synthesis or proposed design, and [guess] means an unvalidated prediction.

## Intent

| Approach | What the human does | Precedent | Fit for Orrery |
|---|---|---|---|
| Spec-driven development | [sourced] Read and edit requirements, architecture, and tasks, then approve stages or execute selected tasks. | [sourced] Kiro maintains `requirements.md`, `design.md`, and `tasks.md`. [S1] [sourced] GitHub Spec Kit separates specification, clarification, planning, tasks, implementation, and convergence. [S2] | [inferred] Attach these artifacts to a territory and expose requirement-to-machine links, rather than reproduce Markdown tabs beside a chat box. |
| Programming by example | [sourced] Supply input-output pairs and answer questions about ambiguous cases. | [sourced] Gulwani's spreadsheet synthesis ranks candidate programs and asks for outputs on inputs with competing interpretations. [S3] | [inferred] Select an actual request, supply its desired response, and mark behaviors that must remain unchanged. |
| Programming by demonstration | [sourced] Perform a browser task once, inspect its recorded steps, and substitute reusable values. | [sourced] CoScripter records and replays web macros, with variable substitution and shared scripts. [S4] | [inferred] Record an end-to-end user journey as an executable example, then ask an agent to preserve or change a specific step. |
| Sketch-based and output-directed programming | [sourced] Draw objects, establish relationships, or manipulate rendered output while the system proposes corresponding program changes. | [sourced] Sketch-n-Sketch combines output-directed transformations, bidirectional evaluation, and structured code editing. [S5] | [inferred] Connect explicit blueprint ports or insert a breaker fitting to propose a code change. [inferred] Moving a machine for layout must remain a separate operation. |
| Properties and counterexamples | [sourced] Write an invariant over generated inputs and inspect the concrete input that falsifies it. | [sourced] Hypothesis generates inputs and reports falsifying cases. [S6] | [inferred] A queue can carry "delivery never causes a second charge" and show the failing request sequence, not merely a red test badge. |
| Objectives and architectural fitness | [sourced] Choose architectural qualities, resolve their conflicts, and encode objective measurements in checks. | [sourced] Fitness function-driven development extends feedback beyond features to architectural aims such as resilience and operability. [S7] | [inferred] Human-owned meters define the feasible region for agents. [inferred] Latency improvements cannot compensate for broken correctness or access controls. |
| Goal trees | [sourced] Refine a higher-level goal into requirements and assign responsibility for satisfying them. | [sourced] Van Lamsweerde's goal-oriented requirements work connects objectives, refinements, and agents. [S8] | [inferred] Expand "checkout survives provider failure" into bounded subgoals attached to belts and machines, each with an owner and evidence obligation. |
| Immediate feedback and abstraction control | [sourced] Change a parameter, scrub time, inspect trajectories, and move between individual examples and aggregate behavior. | [sourced] Inventing on Principle connects code edits to visible results. [S9] [sourced] Ladder of Abstraction makes time and parameters directly explorable. [S10] | [inferred] Let users compare retry policies across saved incidents, then descend from a distribution to the request responsible for its tail. |
| Shared, explorable models | [sourced] Spread out physical computational materials, point to a figure, scrub its video, and discuss models with other people. | [sourced] Dynamicland's documented scientific presentation uses dynamic papers, table models, and a persistent wall timeline. [S12] [sourced] Seeing Spaces asks for visibility inside systems, across time, and across possibilities. [S11] | [inferred] Keep the system, alternatives, and evidence visible together during review. [inferred] A desktop drawing borrows this principle, not Dynamicland's physical or communal medium. |
| Versioned dynamic documents | [sourced] Explore variants and edit a plan that contains structured references, formulas, maps, and other views. | [sourced] Patchwork investigates version control for creative work. [S13] [sourced] Embark progressively enriches informal notes with computations and views. [S14] | [inferred] A choice can retain its assumptions, experiments, and rejected alternatives, with live links to the affected machines. |
| Malleable, locally owned tools | [inferred] Duplicate an inspector, edit its source or ask an agent to modify it, and save the adapted tool locally. | [sourced] Litt argues for user-modifiable software, Appleton for locally useful home-cooked software, and Ink and Switch for collaboration with ownership. [S15, S16, S17] | [inferred] Let an architect inspect a domain-specific tool without surrendering ownership to its service provider. [inferred] Do not let agent-authored inspectors redefine protected acceptance checks. |
| Semantic controls instead of repeated prompts | [inferred] Drag a selected object, change a named parameter, and watch dependent results rather than retype the whole request. | [sourced] Lee explores interfaces beyond raw completion and conversation. [S18] [sourced] Wattenberger demonstrates directly moving itinerary objects while related information changes. [S19] | [inferred] Select a named retry policy, pin its invariants, and compare parameter alternatives without rewriting the entire task. |
| Live incomplete programs | [sourced] Continue inspecting meaningful results while unfinished or erroneous parts remain explicit holes. | [sourced] Hazel assigns static and dynamic meaning to incomplete programs. [S20] | [inferred] A ghost machine may expose a typed, unimplemented port without pretending that its behavior has been verified. |
| Reactive notebooks and moldable development | [sourced] Change a notebook control and rerun dependent computations, or start with a live domain object and add contextual views and actions. | [sourced] marimo synchronizes code and dependent outputs. [S21] [sourced] Glamorous Toolkit builds on Pharo Smalltalk and domain-specific inspectors. [S22] | [inferred] A selected machine opens an executable investigation with request examples, measurements, and source links. [inferred] The maintainer inherits a rerunnable investigation, not a screenshot alone. |

## Verification

| Approach | What the human does | Precedent | Fit for Orrery |
|---|---|---|---|
| Structural and semantic summaries | [sourced] Inspect syntax changes without treating reformatting as equivalent to a logic change. | [sourced] Difftastic compares syntax, rather than lines, and falls back conservatively when parsing fails. [S23] | [inferred] Show changed interfaces, dependencies, permissions, and data flows first, with source diffs underneath. [inferred] Syntax awareness does not establish behavioral equivalence. |
| Behavioral comparison | [sourced] Send the same traffic to current and candidate versions and inspect response differences. | [sourced] Microsoft's shadow-testing guide describes parallel environments and response comparison. [S24] | [inferred] Scrub synchronized runs and inspect changed outputs, effects, timings, and errors on the corresponding belts. |
| Visual regression | [sourced] Compare a candidate screenshot with an approved baseline and explicitly update the baseline for intended changes. | [sourced] Playwright documents screenshot assertions, pixel thresholds, and environment-sensitive rendering. [S25] | [inferred] Use before, after, and difference views for UI work, alongside interaction and accessibility checks. |
| Tests and evals as the review object | [sourced] Read task inputs, expected outcomes, actual outcomes, and grader decisions, then inspect failed cases. | [sourced] Anthropic distinguishes code-based, model-based, and human graders, including fail-to-pass and pass-to-pass tests. [S28] | [inferred] Review evidence by requirement, including preserved behavior, instead of reading an agent's success narrative. |
| Proof-carrying changes | [sourced] Validate a producer-supplied machine-checkable proof against a consumer's safety policy. | [sourced] Necula and Schneck separate proof production from verification and discuss the trusted computing base. [S26] | [inferred] Use formal proofs only for properties and components with a defined model. [inferred] Call ordinary test attachments an evidence bundle, not a proof. |
| Provenance and attestation | [sourced] Verify which builder produced which artifact from which inputs. | [sourced] SLSA build provenance records build definitions, dependencies, builder identity, and artifacts. [S27] | [inferred] Bind checks to the exact candidate revision, protected contract, environment, and fixtures. [inferred] Provenance establishes origin, not product correctness. |
| Review the plan before execution | [sourced] Examine a specification and implementation plan before starting the implementation stage. | [sourced] Kiro and Spec Kit expose staged review and approval. [S1, S2] | [inferred] Approve touched territory, intended changes, forbidden effects, and verification obligations. [inferred] Reopen approval when implementation exceeds that envelope. |
| Risk-ranked review | [sourced] Orrery already orders choices by uncertainty and reversal cost. [Local grounding] | [inferred] Evidence can inform confidence, while deletion, migration, and permission changes inform reversal cost. | [inferred] Open the highest-risk unresolved claim directly on its machine, with missing evidence and rollback options visible. |
| Explorable runtime explanation | [sourced] Inspect live domain objects through contextual views instead of starting with a generic code editor. | [sourced] Glamorous Toolkit recommends starting with objects and incrementally building tools that answer domain questions. [S22] | [inferred] An agent returns a reproducible inspector showing why the change works, including counterexamples and drill-down to source. |

### What the measured evidence actually says

[sourced] LinearB's 2026 article reports analysis of 8.1 million pull requests across 4,800 teams in 42 countries. [S31]
[sourced] It reports AI-generated PR pickup exceeding 16 hours on average, compared with roughly 200 minutes for unassisted work, and a subsequent review cycle of about 194 versus 252 minutes. [S31]
[inferred] These are workflow durations, not measured minutes of uninterrupted human reading, and the article does not establish a universal labor cost per agent PR.
[sourced] LinearB separately distinguishes AI-assisted and fully agentic changes, so its broad AI-generated figures should not be silently relabeled as agent-only results. [S31]

[sourced] Faros compares periods of lowest and highest adoption within organizations, across 22,000 developers and more than 4,000 teams. [S32]
[sourced] It reports average time spent in review increasing 199.6% and median time in review increasing 441.5%. [S32]
[inferred] These observational vendor measurements support a bottleneck hypothesis, not a causal estimate for Orrery's users.

[sourced] A 2026 AIDev study finds no recorded review activity for 20,621 of 33,596 agent-authored PRs, or 61.38%. [S33]
[sourced] The authors explicitly warn that absence of recorded interaction does not establish absence of human inspection. [S33]
[inferred] The important design gap is demonstrable oversight, not a comment-count target.

[sourced] METR's early-2025 randomized study found a 19% slowdown across 246 tasks completed by 16 experienced open-source developers. [S34]
[sourced] Its February 2026 follow-up says selection effects and concurrent agent work make the newer productivity estimate unreliable. [S35]
[inferred] Neither study supplies a clean number for review minutes per autonomous PR.
[inferred] Orrery should measure active inspection, waiting, interruptions, corrections, and escaped failures separately before claiming that its review workflow saves time.

## Ideas Orrery could steal or invert

### 1. Counterexample-directed repair

[inferred] The user selects a failed checkout request, scrubs to its duplicate charge, and marks the second charge as forbidden.
[inferred] Beside it, they pin a successful request whose behavior must remain unchanged.
[inferred] The agent returns a candidate, the original replay, generated neighboring cases, and any remaining failures.
[inferred] This replaces an ambiguous bug description with concrete behavior plus preservation requirements.
[sourced] Flash Fill asks humans to resolve competing interpretations, and Hypothesis reports falsifying inputs. [S3, S6]
[inferred] The new element relative to prompt-to-diff development is that the selected runtime event becomes both the instruction and a review obligation.
[guess] This may reduce repeated clarification for incident repair.
[inferred] The risk is overfitting to recorded traffic, so acceptance must include varied inputs and explicitly unknown cases.

### 2. Contract gestures on the plant

[inferred] The user selects a belt, inserts a breaker fitting, chooses a latency ceiling, and pins "no duplicate external writes."
[inferred] Orrery shows the exact contract and proposed affected territory before an agent starts.
[inferred] The user approves that contract with its checks, not the agent's implementation recipe.
[sourced] Sketch-n-Sketch translates output manipulation into program changes, while architectural fitness functions make nonfunctional aims executable. [S5, S7]
[inferred] The new element is that a gesture creates a bounded engineering obligation, rather than directly rewriting infrastructure.
[inferred] This makes architecture editing operational while preserving Orrery's human-owned budgets.
[inferred] The risk is hidden interpretation, especially confusing authored layout with system topology.
[inferred] Only explicit port and fitting edits should change design intent.

### 3. Experiment branches before implementation

[inferred] The user duplicates a territory into alternatives such as retries, caching, and a queue, then pins identical traffic and correctness requirements.
[inferred] They scrub scenarios and compare latency, cost, failure behavior, and uncertainty side by side before funding one build.
[sourced] Patchwork investigates variant exploration, and Victor's Ladder compares behavior across parameter choices. [S13, S10]
[inferred] The problem is premature commitment to the first plausible plan.
[inferred] The new element is that agents compete on inspectable consequences and constraint satisfaction, not persuasive prose.
[guess] Side-by-side tradeoffs may produce better architectural choices than sequential chat.
[inferred] The risk is treating a fast model as reality.
[inferred] Simulation results must remain labeled predictions, with calibration and assumptions visible, until independent runtime measurements exist.

### 4. Evidence-bound acceptance

[inferred] The user opens an inspection station showing requirement rows with before behavior, candidate behavior, check identity, revision, environment, and evidence.
[inferred] They click an anomalous row to inspect a replay, screenshot difference, counterexample, or source change.
[inferred] Acceptance fails closed when the revision or protected contract changes.
[sourced] Proof-carrying code separates the producer's work from the consumer's checker, and SLSA binds artifact production to recorded inputs. [S26, S27]
[inferred] The new element is applying that separation to every agent delivery, while preserving distinct labels for tests, observations, predictions, and formal proofs.
[inferred] This solves stale evidence and unsupported completion claims.
[inferred] The risk is an attractive collection of weak checks.
[inferred] The user must see omitted obligations, grader limitations, and who can alter each checker.

### 5. Verification holes that remain visible

[inferred] A ghost machine shows an empty port where behavior has no implementation and a separate empty evidence slot where implemented behavior has not been tested.
[inferred] The user clicks either hole to supply an example, commission an experiment, or accept a documented risk.
[sourced] Hazel preserves meaning around incomplete programs through typed holes. [S20]
[inferred] The problem is a false binary between green and broken.
[inferred] The new element is making incomplete understanding a durable delegation target, rather than burying it in a completion message.
[guess] Such holes may make fog of war actionable without encouraging arbitrary confidence percentages.
[inferred] The risk is visual overload, so the distant plant should show aggregate uncertainty while inspection exposes individual obligations.

### 6. The plan is an execution envelope

[inferred] Before work, the user approves intended behavior, permitted territory, forbidden effects, stop conditions, and a verification plan.
[inferred] During work, a proposed migration or permission expansion appears as a new choice at the affected machine.
[inferred] The user compares the new action with the approved envelope and authorizes or rejects that specific expansion.
[sourced] Kiro and Spec Kit provide specification and planning stages before implementation. [S1, S2]
[inferred] The new element is checking subsequent work against a continuing authorization boundary.
[inferred] This prevents "review the plan" from becoming an excuse never to inspect implementation.
[inferred] The risk is interruption fatigue.
[inferred] Reversible implementation details can stay delegated, while irreversible effects and protected-check changes require human approval.

### 7. A maintainable investigation, not a generated explanation

[inferred] The agent returns a saved investigation attached to the machine, with concrete inputs, dependent computations, traces, and contextual views.
[inferred] The user changes an input, reruns the evidence, and inspects the view's source when its explanation seems wrong.
[sourced] marimo maintains dependent outputs, Embark keeps computations inside planning documents, and Glamorous Toolkit builds contextual tools around domain objects. [S21, S14, S22]
[inferred] The problem is that a prose summary cannot be rerun when the code changes.
[inferred] The new element is treating the review instrument as part of the deliverable that the next maintainer inherits.
[inferred] The risk is allowing that instrument to become an unreviewed source of authority.
[inferred] Protected tests must run independently of the agent's custom views.

## What to avoid

[inferred] Avoid replacing a short prompt with mandatory document production for every change.
[sourced] Kiro recommends quick fixes for straightforward changes and offers a lighter spec path when standard review feels too heavy. [S1]
[inferred] This documents workflow friction, not a controlled failure rate for specification-driven development.
[inferred] Orrery should require more explicit intent as ambiguity and reversal cost rise.

[inferred] Avoid a single reward that lets an agent trade correctness for speed.
[sourced] Anthropic's reward-hacking experiment includes bypassing a test harness with `sys.exit(0)` to appear successful. [S29]
[sourced] The researchers deliberately selected vulnerable training environments and supplied knowledge of such hacks. [S29]
[inferred] This establishes a possible failure mechanism, not the frequency of cheating in ordinary coding sessions.
[inferred] Protected checks, artifact binding, and independent execution matter more than another success badge.

[inferred] Avoid assuming clean-looking output needs less scrutiny.
[sourced] CodeRabbit reports 10.83 findings per AI-coauthored PR versus 6.45 for its human-only group across 470 PRs. [S30]
[sourced] Its authorship labels rely on detectable signals and may misclassify undetected AI assistance. [S30]
[inferred] Vendor findings are not independently confirmed production defect counts, but they challenge appearance-based approval.

[inferred] Avoid treating baseline updates as harmless cleanup.
[sourced] Playwright allows screenshot baselines to be regenerated and warns that operating systems, hardware, and rendering environments change pixels. [S25]
[inferred] A user should approve intended differences under a controlled environment, rather than let the implementation agent erase regression evidence.

[inferred] Avoid replaying real writes twice.
[sourced] Shadow testing duplicates requests into a candidate environment. [S24]
[inferred] A second payment or notification can still happen unless effects are isolated, substituted, or blocked.
[inferred] Recorded responses and OTel timings alone cannot prove the behavior of unexecuted candidate code.

[inferred] Avoid presenting research prototypes as solved product foundations.
[sourced] Embark reports hardcoded functions, implicit schemas, collaborative-editing problems, and concrete formula arguments that can become stale. [S14]
[inferred] Preserve its inspectable computations without inheriting invisible coupling or promising general-purpose extensibility.

[inferred] Avoid silently generalizing one demonstration.
[sourced] The CoScripter field study found users inventing workarounds for repetition, generalization, and reuse. [S4]
[inferred] Ask for discriminating examples and show the inferred rule before applying it broadly.

[inferred] Avoid calling a syntax diff semantic proof, a signed build a correct feature, or a quiet review queue successful oversight.
[sourced] Difftastic compares syntax, SLSA describes production provenance, and the AIDev study measures recorded interactions. [S23, S27, S33]
[inferred] Each answers a different question, and Orrery should keep those questions separate.

## Sources opened

[sourced] The evidence set contains the following 35 opened sources.

- S1. https://kiro.dev/docs/specs/best-practices/
- S2. https://github.github.com/spec-kit/quickstart.html
- S3. https://www.microsoft.com/en-us/research/publication/automating-string-processing-spreadsheets-using-input-output-examples/
- S4. http://www.cs.cmu.edu/~Compose/CoscrStudy-20080304-2200.pdf
- S5. https://ravichugh.github.io/sketch-n-sketch/
- S6. https://hypothesis.readthedocs.io/en/latest/quickstart.html
- S7. https://www.thoughtworks.com/en-us/insights/articles/fitness-function-driven-development
- S8. https://webperso.info.ucl.ac.be/~avl/files/RE01.pdf
- S9. https://jamesclear.com/great-speeches/inventing-on-principle-by-bret-victor
- S10. https://worrydream.com/LadderOfAbstraction/
- S11. https://vimeo.com/97903574
- S12. https://dynamicland.org/2023/Improvising_cellular_playgrounds_in_Realtalk/
- S13. https://www.inkandswitch.com/project/patchwork/
- S14. https://www.inkandswitch.com/embark/
- S15. https://www.inkandswitch.com/essay/local-first/
- S16. https://www.geoffreylitt.com/2023/03/25/llm-end-user-programming.html
- S17. https://maggieappleton.com/home-cooked-software
- S18. https://thesephist.com/posts/latent/
- S19. https://wattenberger.com/thoughts/ai-is-still-in-its-terminal-era/
- S20. https://hazel.org/
- S21. https://github.com/marimo-team/marimo
- S22. https://book.gtoolkit.com/how-to-get-started-with-moldable-developme-lor2271nbpvdqfx46d0prlk2
- S23. https://github.com/Wilfred/difftastic
- S24. https://microsoft.github.io/code-with-engineering-playbook/automated-testing/shadow-testing/
- S25. https://playwright.dev/docs/test-snapshots
- S26. https://people.eecs.berkeley.edu/~necula/Papers/sfpol_isss02.pdf
- S27. https://slsa.dev/spec/v1.2/build-provenance
- S28. https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents
- S29. https://www.anthropic.com/research/emergent-misalignment-reward-hacking
- S30. https://www.coderabbit.ai/blog/state-of-ai-vs-human-code-generation-report
- S31. https://linearb.io/blog/8-million-prs-engineering-productivity
- S32. https://www.faros.ai/blog/ai-acceleration-whiplash-takeaways
- S33. https://arxiv.org/html/2605.02273v1
- S34. https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/
- S35. https://metr.org/blog/2026-02-24-uplift-update/

[sourced] Local grounding consists of `/home/factory-user/orrery/explore/map.md:20-72`, `/home/factory-user/orrery/explore/ui-v1/shots/l-default.png`, and `/home/factory-user/orrery/explore/ui-v1/shots/p5-floor.png`.
