# Supervisory control for many coding agents

[inferred] Orrery should increase safe unattended time, not maximize the number of visible agents.
[inferred] Its plant becomes an ecological interface only when it exposes constraints and means-ends relationships, not merely topology.
[inferred] Delegate bounded missions with explicit authority, stopping conditions, and evidence obligations.
[inferred] Make returning to a task cheap enough that the engineer can leave it unattended.
[sourced] Recent studies find that participation can improve outcomes without calibrating trust, and that generation probabilities can be poor review cues [S17, S19].
[inferred] The strongest bakeoff candidates are a constraint-first plant, attention scheduling, executable delegation, evidence-linked review, and rationalized alarms.

## Scope and evidence

[sourced] This research is dated 2026-10-17 and uses K1 through K20 and the catalog in `/home/factory-user/orrery/explore/map.md:20-70`, plus the two requested screenshots.
[sourced] WebSearch and FetchUrl were loaded, but FetchUrl's tool contract permits only user-supplied URLs, not discovered research URLs.
[sourced] Sources were therefore opened through WebSearch's full-page-text option.
[sourced] Later requests returned HTTP 429 despite backoff, which prevented some targeted follow-ups.
[sourced] The source inventory contains 35 nonempty page responses, including abstracts, bibliographic pages, and incidental results.
[inferred] An abstract supports its stated result, not an unreported effect size or proof that the result transfers to coding agents.
[sourced] No CSCW-specific empirical paper was verified before retrieval stopped.
[inferred] Treat the recent-work coverage as a selected CHI, UIST, ToCHI, and arXiv sample, not a systematic review.

## Design rules

[inferred] Evidence strength below distinguishes experiments, reviews, conceptual models, and standards guidance rather than treating citation counts as validation.

| Rule | Source and evidence | Evidence strength | Implication for Orrery |
|---|---|---|---|
| [inferred] Assign authority separately for sensing, analysis, decisions, and execution. | [sourced] Sheridan and Verplank describe supervisory control and graded allocation [S1]. [sourced] Parasuraman, Sheridan, and Wickens distinguish four automation functions [S2]. | [sourced] Foundational models, not a coding-agent trial. | [inferred] Show "may inspect," "may propose," "may edit," and "may merge" separately. [inferred] One autonomy slider hides important differences. |
| [inferred] Design the handback before permitting unattended execution. | [sourced] Bainbridge identifies problems left with the human responsible for abnormal conditions [S3]. [sourced] The 2024 GenAI synthesis describes the shift from production to evaluation [S14]. | [sourced] Classic analysis and cross-study synthesis. | [inferred] Every delegation needs a safe stopping state, resumable context, and an intervention path that does not require reading the whole transcript. |
| [inferred] Prefer negotiated authority changes over silent adaptation. | [sourced] Calhoun distinguishes human-assigned adaptable automation from system-assigned adaptive automation [S4]. [sourced] The few direct comparisons favor adaptable automation, while comparative research remains scarce [S4]. | [sourced] Mini-review with limited direct comparisons. | [inferred] Suggest a narrower or broader delegation, explain the trigger, and record acceptance. [inferred] Preauthorize automatic containment, not silent permission expansion. |
| [inferred] Measure attention demand instead of promising a fixed span of control. | [sourced] Olsen and Wood relate fan-out to independent robot activity and interaction time and report experimental validation [S5]. | [sourced] Experiments in robot tasks, with domain limits. | [inferred] Measure orientation, review, intervention, and switching time. [inferred] Agent count alone is not capacity. |
| [inferred] Delegate reusable plays, not isolated commands. | [sourced] Calhoun describes extensions of Miller and Parasuraman's Playbook delegation for multiple unmanned vehicles and single-pilot operation [S4]. | [sourced] Review of prototypes and experiments. | [inferred] Share a task model containing goal, roles, constraints, checkpoints, and authority. [inferred] A named play must remain inspectable and editable. |
| [inferred] Support perception, comprehension, and projection. | [sourced] Endsley's model separates these three levels of situation awareness [S6]. | [sourced] Foundational theory with only headings and limited body text extracted from the retrieved scan. | [inferred] Show what changed, why it matters, and what happens next if nobody intervenes. [inferred] A moving bot covers only part of the first level. |
| [inferred] Represent the work domain's constraints, not a prescribed procedure alone. | [sourced] Vicente and Rasmussen's abstraction hierarchy connects system purpose to implementation through means-ends relationships [S7]. | [sourced] Theory with preliminary empirical support. | [inferred] Make goals, tradeoffs, capabilities, components, and evidence navigable together. |
| [inferred] Show the feasible alternatives behind automated advice. | [sourced] A sixteen-participant ATC simulation improved fault detection and diagnosis with a solution-space display [S8]. [sourced] It also prompted takeovers unrelated to faults [S8]. | [sourced] Controlled simulation, not operational ATC evidence. | [inferred] Show why a proposed change lies inside or outside constraints. [inferred] Test whether visibility causes unnecessary intervention. |
| [inferred] Reserve alarms for conditions requiring a response. | [sourced] EEMUA describes alarms as operator support for abnormal situations [S9]. [sourced] ISA distinguishes alarms from non-alarm notifications and requires rationalization [S10, S11]. | [sourced] Industrial guidance and standards summaries. | [inferred] Completion, narration, and ordinary CI progress belong in history or an inbox, not the urgent queue. |
| [inferred] Manage alarm quality throughout its life. | [sourced] ISA includes monitoring, maintenance, change management, dynamic alarming, and shelving [S10]. [sourced] ABB describes overload and nuisance alarms [S11]. | [sourced] Standards guidance and practitioner evidence. | [inferred] Track interrupt rate, stale requests, repeated alarms, and response cost. [inferred] Suppression needs a reason and expiry. |
| [inferred] Calibrate reliance instead of maximizing trust. | [sourced] Lee and See frame the objective as appropriate reliance [S20]. [sourced] The CHI daily-assistant experiment found that involvement did not calibrate trust to outcomes [S17]. | [sourced] Foundational review framing and a 248-person experiment. | [inferred] Separate agent assertions, independent checks, observed outcomes, and untested claims. [inferred] "Human approved" is not evidence of correctness. |
| [inferred] Do not treat expertise or reminders as protection against automation bias. | [sourced] Parasuraman and Manzey's review finds complacency and omission or commission errors among novices and experts [S12]. [sourced] Simple practice or instructions do not eliminate them [S12]. | [sourced] Empirical literature review. | [inferred] Reduce monitoring competition and provide independent evidence. [inferred] "Review carefully" is not a sufficient safety mechanism. |
| [inferred] Ask only questions whose answer changes the action. | [sourced] Horvitz considers uncertainty, interruption cost, useful dialog, direct termination, and doing less under uncertainty [S13]. | [sourced] Design principles and research prototype. | [inferred] Offer a default, alternatives, consequence, and deadline. [inferred] Preserve direct pause, edit, and cancellation. |
| [inferred] Measure the human work after acceptance. | [sourced] CUPS studied 21 programmers and documents verification and editing after suggestions were accepted [S15]. | [sourced] Small observed study of code completion. | [inferred] Track time to verified integration, not accepted suggestions, tokens, or agent "done" events. |
| [inferred] Stop at consequential ambiguity, not every tool call. | [sourced] Morae pauses UI agents for user choices and reports better task completion and preference matching for blind and low-vision users [S16]. | [sourced] UIST system study, read through its abstract. | [inferred] Ask about public APIs, data retention, or dependency tradeoffs before committing to one interpretation. |
| [inferred] Target review cues at likely human correction, not token surprise. | [sourced] A 30-programmer study found benefits from predicted-edit highlighting, but none from generation-probability highlighting [S19]. | [sourced] Mixed-methods study of code completion. | [inferred] Treat uncertainty labels as task-specific instruments. [inferred] Do not use raw model probability as K17's confidence. |
| [inferred] Support the whole supervisory loop. | [sourced] Park and colleagues identify Plan, Monitor, Wait, Review, Teach, Manual Fix, and Update Assets from 19 experienced developers [S18]. | [sourced] September 2026 preprint, short observations and qualitative analysis. | [inferred] A session needs explicit review and learning states. [inferred] Preserve corrected instructions as proposed reusable assets, not automatic permanent policy. |

## Why the abstraction hierarchy matters

[sourced] Vicente and Rasmussen distinguish functional purpose, abstract function, generalized function, physical function, and physical form [S7].
[sourced] Their hierarchy is not a spatial zoom ladder, and its exact levels can vary by domain [S7].
[inferred] Zooming from a repository to a service to a file therefore does not, by itself, produce an ecological interface.
[inferred] Orrery needs movement between "why does this exist?" and "how can that purpose be achieved?"

[inferred] For a payment path, a useful translation would contain the following levels.

1. [inferred] Functional purpose is collecting authorized payments while preserving customer trust.
2. [inferred] Abstract function contains outcome measures and constraints, such as duplicate-charge prevention, latency, cost, and acceptable failure.
3. [inferred] Generalized function contains authorization, deduplication, settlement, retry, and reconciliation.
4. [inferred] Physical function contains the service, queue, database, external payment API, and their capacities.
5. [inferred] Physical form contains the implementation locations, deployment identity, ports, and observed instances represented by catalog machines.

[sourced] EID aims to support skill-based, rule-based, and knowledge-based behavior without forcing a higher cognitive level than the task requires [S7].
[inferred] Routine inspection should allow direct perception of a constraint violation, while diagnosis should expose the underlying relationship.
[inferred] A retry loop should show the connection between arrival rate, retry amplification, queue growth, and the threatened payment objective.
[inferred] The operator should not have to reconstruct that relationship from six disconnected gauges.

[inferred] Software requires caution when borrowing process-control representations.
[inferred] An accounting identity, a human policy, a measured correlation, and a queueing approximation have different authority.
[inferred] Display the relationship's type, assumptions, provenance, and validity period.
[guess] An attractive but false causal diagram could produce more confident mistakes than an ordinary metrics dashboard.
[sourced] The ATC study explicitly identifies unreliable underlying data as a challenge for ecological displays [S8].

## Fan-out is an attention budget

[sourced] Olsen and Wood define fan-out as the number of robots one person can operate and associate higher fan-out with greater neglect tolerance and lower interaction time [S5].
[sourced] Their references include Crandall and Goodrich's work on human-robot interaction efficiency [S5].
[sourced] Crandall, Goodrich, Olsen, and Nielsen's 2005 paper was located, but the retrieved publisher page did not expose its substantive body.
[inferred] No numerical capacity claim here should be attributed to that inaccessible paper.

[inferred] A simple idealized bound is `N <= 1 + NT / IT`, where NT is tolerable unattended time and IT is the attention needed to restore useful independent operation.
[inferred] The bound follows by fitting the other `N - 1` interactions into one agent's unattended interval.
[inferred] It assumes similar independent tasks, predictable service times, and no switching cost.
[inferred] It is an intuition for scheduling, not a claim that a particular engineer can supervise a particular number of agents.

[inferred] Coding agents can often wait safely at a checkpoint, unlike vehicles with time-critical intervention requirements.
[inferred] That makes intentional parking a capacity tool.
[inferred] Conversely, agents sharing a repository, dependency, or test service can produce correlated demands that invalidate independent-task assumptions.
[inferred] Count orientation, evidence inspection, conflict resolution, and manual repair inside IT.
[guess] Orrery's practical capacity limit may be review bursts rather than concurrent execution.

## Control-room and cockpit lessons

[sourced] Parasuraman, Sheridan, and Wickens identify reduced situation awareness, complacency, skill loss, and mode confusion among automation's human-performance costs [S2].
[inferred] For Orrery, the cockpit equivalent is a persistent indication of execution authority beside the system's data mode.
[inferred] "Replay, cannot write" and "Live, may edit branch, merge requires approval" should remain distinguishable while the engineer inspects a machine.
[inferred] Show armed actions separately from actions already underway, together with the condition that starts them.
[inferred] A change of paper cannot communicate all those distinctions.

[sourced] The 2024 GenAI synthesis explicitly relates contemporary usability problems to Bainbridge and Endsley's 2023 Ironies of Artificial Intelligence [S14].
[sourced] It identifies evaluation burden, workflow restructuring, interruptions, and making difficult tasks harder as recurring problems [S14].
[inferred] The implication for a second-monitor application is to preserve orientation without demanding constant reading.
[inferred] A stable overview, a short account of changes since inspection, and drill-down to evidence fit that requirement better than a permanent narration feed.

## Where the current design follows or breaks the rules

[sourced] K4 separates designed, declared, and observed sources and displays disagreements as drift.
[inferred] This is a strong foundation for evidence provenance and detection of a false system model.
[sourced] K5 preserves an architect-authored layout, and K6 supplies consistent machine silhouettes.
[inferred] Those decisions support a stable place to return to after attention shifts.

[sourced] K11 makes budgets agent objectives, K13 requires simulation calibration, and K16 prevents agents from changing budgets or checks without approval.
[inferred] Together they support bounded delegation, but optimizing one budget can still violate an unstated global constraint.
[sourced] K17 orders choices by `(1 - confidence) × reversal cost`.
[inferred] That order omits explicit harm magnitude, urgency, and evidence about whether confidence predicts error.
[inferred] Keep the rule visible and challengeable rather than presenting it as a validated safety measure.

[sourced] `/home/factory-user/orrery/explore/ui-v1/shots/p5-floor.png` shows agent activity, mapped files, CI attempts, checks, a territory progression, and recorded choices.
[inferred] Those links are more useful for supervisory comprehension than a transcript-only interface.
[sourced] The screenshots show "Waiting on you" queues containing three and seven items respectively.
[inferred] Neither screenshot visibly explains which request can wait safely, how long review takes, or what harm follows delay.
[inferred] This is a visible gap, not proof that no other screen supplies the information.

[sourced] `/home/factory-user/orrery/explore/ui-v1/shots/l-default.png` gives most screen space to machines and belts, with compact status information.
[inferred] The image communicates topology more strongly than purpose, constraints, or predicted consequences.
[inferred] Its small, low-contrast labels also deserve testing at second-monitor viewing distance.
[inferred] No contrast-compliance or usability measurement was performed here.
[sourced] K15 differentiates Live, Blueprint, Simulate, and Replay through their paper.
[inferred] Add persistent textual mode and data-age indicators, since those modes do not explain an agent's execution authority.

## Ideas Orrery could steal or invert

### 1. The constraint-first plant

[inferred] The user sees customer outcomes above the plant and constraint relationships attached to the relevant flows.
[inferred] Selecting "no duplicate charge" reveals the deduplication function, responsible components, current evidence, and pending changes that could break it.
[inferred] The user delegates an improvement inside that constraint rather than assigning edits to a machine.
[inferred] This solves the separation between architectural intent and local agent optimization.
[sourced] Its precedent is EID's means-ends hierarchy and ATC solution-space displays [S7, S8].
[inferred] Its contribution to agentic development is making the architecture an inspectable delegation and acceptance model, rather than a map beside the real workflow.
[guess] The main risk is costly or misleading constraint modeling, especially when inferred relationships look authoritative.

### 2. Safe-to-leave scheduling

[inferred] Every mission shows its next intervention window, estimated review effort, safe parking point, and reason it can remain unattended.
[inferred] The user chooses a focus period, and Orrery proposes which missions can run, park, or start without an overlapping review burst.
[inferred] Unknown estimates appear as unknown, not precise countdowns.
[inferred] This solves overload before the urgent queue grows.
[sourced] Fan-out research supplies the attention-time precedent, while Horvitz supplies interruption timing [S5, S13].
[inferred] Its contribution is scheduling the engineer's future judgment alongside agent execution, instead of treating human review as unlimited capacity.
[guess] The risk is false reassurance from unstable duration estimates or hidden shared dependencies.
[inferred] Use measured ranges and reveal the assumptions.

### 3. Executable delegation plays

[inferred] The user selects "replace dependency," "repair regression," or an authored play and edits a visible mission contract.
[inferred] The contract includes protected checks, scope, permitted tools, mandatory evidence, review gates, and a safe abort state.
[inferred] The agent may alter its implementation plan but must request changes to delegated authority.
[inferred] This solves repeated prompting and ambiguous responsibility.
[sourced] Miller and Parasuraman's Playbook approach, described in Calhoun's review, supplies the precedent [S4].
[inferred] Its contribution is binding the human-readable play to enforced execution permissions and system-level acceptance evidence.
[guess] The risk is a polished contract that the runtime does not enforce.
[inferred] Distinguish enforced rules from advisory instructions in the UI.

### 4. The evidence-linked review station

[inferred] Returning to a mission opens a compact packet of intent, changes since last inspection, claimed outcomes, independent checks, remaining unknowns, and reversal options.
[inferred] Selecting a claim reveals the diff, relevant system path, exact check revision, execution result, and evidence age.
[inferred] The user approves a supported consequence, requests a missing witness, or rejects the change.
[inferred] This solves the cost of recovering context and reviewing fluent but unsupported summaries.
[sourced] CUPS documents post-acceptance verification, while the 2026 supervisory framework identifies Review and maintaining understanding as distinct work [S15, S18].
[inferred] Its contribution is organizing review around system claims and evidence rather than the agent's conversational narrative.
[guess] The risk is correlated implementation and verification errors hidden behind a reassuring packet.

### 5. An alarm system with an audit trail

[inferred] The user sees separate streams for required decisions, abnormal conditions, completions, and ordinary history.
[inferred] A shared CI failure groups related requests while retaining independently actionable failures.
[inferred] Shelving requires an owner, reason, expiry, and visible unresolved count.
[inferred] This solves notification floods without silently discarding obligations.
[sourced] EEMUA and ISA supply rationalization, notification separation, monitoring, and shelving precedents [S9, S10, S11].
[inferred] Its contribution is treating agent requests as an engineered operational system with measured attention cost, not a collection of notifications.
[guess] The main risk is incorrect causal grouping that hides a separate urgent problem.
[inferred] Display grouping as a hypothesis unless the dependency is established.

### 6. Preference checkpoints

[inferred] At an architectural fork, the agent offers a few concrete alternatives with consequences and an explicit "not specified" marker.
[inferred] The user decides between retaining compatibility, accepting migration cost, or narrowing scope before the agent generates a large patch.
[inferred] This solves expensive execution against unstated preferences.
[sourced] Morae supplies proactive choice pauses, and Horvitz supplies cost-sensitive clarification [S16, S13].
[inferred] Its contribution is promoting consequential software tradeoffs into first-class controls rather than discovering them during code review.
[guess] The risk is endless consultation or the agent presenting only alternatives it already prefers.
[inferred] Allow direct editing of the constraint and an alternative the agent did not propose.

### 7. Rehearsal before wider authority

[inferred] Replay pauses before a consequential event and asks the engineer to predict the next state or choose a containment action.
[inferred] The user compares the prediction with recorded behavior and can narrow a mission's authority afterward.
[inferred] This addresses loss of system understanding while execution becomes more automated.
[sourced] Bainbridge's analysis and the GenAI synthesis identify the problem of shifting humans into evaluation and difficult exception handling [S3, S14].
[inferred] Replay and calibrated simulation in K15 and K13 provide an existing place to test this idea.
[guess] Its novelty and value for coding-agent supervision remain unvalidated.
[guess] The risk is performative training that increases interruption without improving intervention.
[inferred] Compare actual recovery performance, not quiz completion.

## What to avoid

- [inferred] Avoid a watch wall of animated agents that requires continuous attention. [sourced] The complacency review links monitoring failures to competing multitask demands, including among experts [S12]. [inferred] Test error detection after an interruption, not whether users enjoy watching activity.
- [inferred] Avoid equating plan approval with calibrated oversight. [sourced] The 248-person CHI study found better task performance with involvement but not calibrated trust, and identifies plausible plans as a possible contributor [S17].
- [inferred] Avoid a single numerical "confidence" badge. [sourced] Generation-probability highlighting provided no benefit in the 30-programmer study [S19]. [inferred] Unknown evidence quality must remain distinct from measured historical error.
- [inferred] Avoid treating every event as an alarm. [sourced] ABB describes how cheap computerized alarm configuration produced nuisance-heavy systems and warns that overload can cause important alarms to be missed or ignored [S11]. [sourced] Its historical summary gives benchmarks below one alarm per ten minutes in steady operation and under ten in the first ten minutes of an upset [S11]. [inferred] These are industrial reference points, not validated notification limits for coding work.
- [inferred] Avoid invisible authority transitions and color-only mode communication. [sourced] Calhoun describes untimely or unpredictable adaptive changes as threats to awareness and performance [S4]. [inferred] The glass-cockpit design lesson is to expose current control mode, its trigger, and the next transition.
- [inferred] Avoid assuming that more visible internals always improve teamwork. [sourced] The ecological ATC display also encouraged takeovers regardless of fault condition [S8]. [inferred] Evaluate unnecessary interventions and downstream harm alongside detection.
- [inferred] Avoid claiming that safety-critical evidence proves a coding interface safe. [sourced] The 2026 developer study observed roughly forty-minute tasks and explicitly calls for longitudinal work [S18]. [inferred] The bakeoff should measure verified integration, recovery after absence, missed consequential choices, and human attention consumed per completed mission.

## Sources opened

[sourced] S1 through S20 are the evidence set. Additional pages are listed separately so bibliographic access and incidental search results are not mistaken for substantive evidence.

- S1. [sourced] Sheridan and Verplank, 1978, report text. https://www.academia.edu/45510118/Human_and_Computer_Control_of_Undersea_Teleoperators
- S2. [sourced] Parasuraman, Sheridan, and Wickens, 2000, author-uploaded paper text. https://www.researchgate.net/publication/11596569_A_model_for_types_and_levels_of_human_interaction_with_automation
- S3. [sourced] Bainbridge, 1983, abstract. https://www.sciencedirect.com/science/article/abs/pii/0005109883900468
- S4. [sourced] Calhoun, 2022, full review. https://journals.sagepub.com/doi/full/10.1177/00187208211037457
- S5. [sourced] Olsen and Wood, CHI 2004, abstract and references. https://dl.acm.org/doi/10.1145/985692.985722
- S6. [sourced] Endsley, 1995, scan with limited extracted text. https://www.digishape.nl/wp-content/uploads/2024/11/endsley_mr_toward_a_theory_of_situation_awareness_.pdf
- S7. [sourced] Vicente and Rasmussen, 1992, full paper. https://backend.orbit.dtu.dk/ws/files/158017888/SMC.PDF
- S8. [sourced] Ecological interface design for supervisory ATC, 2017, full paper. https://link.springer.com/article/10.1007/s10111-017-0438-y
- S9. [sourced] EEMUA 191, official publication summary, not the purchased standard. https://www.eemua.org/products/publications/digital/eemua-publication-191
- S10. [sourced] ISA-18, official standards summaries. https://www.isa.org/standards-and-publications/isa-standards/isa-18-series-of-standards
- S11. [sourced] ABB, Alarm Management for SCADA control rooms, white paper. https://library.e.abb.com/public/72f20c70c7b44d889d463db81df5c38d/SCADA%20Alarm%20Management%20White%20Paper.pdf
- S12. [sourced] Parasuraman and Manzey, 2010, abstract. https://pubmed.ncbi.nlm.nih.gov/21077562/
- S13. [sourced] Horvitz, CHI 1999, full paper. https://erichorvitz.com/chi99horvitz.pdf
- S14. [sourced] Ironies of Generative AI, 2024, full preprint. https://arxiv.org/html/2402.11364v1
- S15. [sourced] Mozannar and colleagues, CHI 2024, full paper. https://dl.acm.org/doi/fullHtml/10.1145/3613904.3641936
- S16. [sourced] Peng and colleagues, Morae, UIST 2025, abstract. https://arxiv.org/abs/2508.21456
- S17. [sourced] Plan-Then-Execute, CHI 2025, full paper. https://dl.acm.org/doi/full/10.1145/3706598.3713218
- S18. [sourced] Park and colleagues, September 2026, full preprint. https://arxiv.org/html/2609.24234v1
- S19. [sourced] Vasconcelos and colleagues, ToCHI 2025, author-hosted abstract. https://www.microsoft.com/en-us/research/publication/generation-probabilities-are-not-enough-exploring-the-effectiveness-of-uncertainty-highlighting-in-ai-powered-code-completions/
- S20. [sourced] Lee and See, 2004, title and bibliographic page only. https://journals.sagepub.com/doi/10.1518/hfes.46.1.50_30392

### Other opened pages, not used as independent evidence

[sourced] These fifteen responses include duplicate access routes, secondary summaries, access-limited pages, and off-topic results.

- https://satechnologies.com/publications/
- https://ieeexplore.ieee.org/document/844354
- https://www.researchgate.net/publication/23882567_Human_and_Computer_Control_of_Undersea_Teleoperators
- https://www.semanticscholar.org/paper/A-model-for-types-and-levels-of-human-interaction-Parasuraman-Sheridan/14ae6f2231e09e226b99002aa04b5c70f3c59f2b
- https://andrewclark.co.uk/all-media/a-model-for-types-and-levels-of-human-interaction-with-automation
- https://en.wikipedia.org/wiki/Ironies_of_Automation
- https://medium.com/@craigtrim/the-ironies-of-automation-0f302343bf7d
- https://ferd.ca/notes/paper-ironies-of-automation.html
- https://journals.sagepub.com/doi/10.1518/001872095779049543
- https://dl.acm.org/doi/10.1145/1121241.1121249
- https://ntrs.nasa.gov/api/citations/20170011326/downloads/20170011326.pdf
- https://pubmed.ncbi.nlm.nih.gov/35358047/
- https://ieeexplore.ieee.org/document/1453692/
- https://discovery.ucl.ac.uk/id/eprint/10212907/1/1149_25_Promoting%20and%20Protecting%20Teacher%20Agency_Cukurova.pdf
- https://discovery.ucl.ac.uk/id/eprint/10056144/1/Cohen_et%20al%20JRI%20rewrite.pdf
