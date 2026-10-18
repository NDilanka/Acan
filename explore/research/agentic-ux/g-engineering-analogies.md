# Engineering disciplines that direct work through a shared model

[inferred] Orrery's strongest precedent is a coordinated construction model, not a dashboard of agent activity.
[inferred] The drawing should issue bounded work, record questions, expose incompatible plans, and accept completed work against independent evidence.
[inferred] Steal submittals, semantic clash detection, tolerance analysis, and as-built handover before adding more agent controls.
[inferred] Review changes to the system through dailies rather than reading transcripts from every contractor.
[inferred] Keep the shared model small, but make its claims, revisions, and authority explicit.
[guess] The largest risk is creating a second engineering bureaucracy that agents maintain while the real system changes elsewhere.

## Scope and evidence

[sourced] Research date is 2026-10-17, as supplied in the assignment.
[sourced] Orrery already separates designed, declared, and observed sources, protects budgets and checks, and stores blueprints in repository YAML in `/home/factory-user/orrery/explore/map.md:26-45`.
[sourced] The catalog includes construction bots, scaffolding, a CI crane, and an inspection station in `/home/factory-user/orrery/explore/map.md:62-70`.
[sourced] The supplied screenshots show a plant overview, agent locations, a floor timeline, and an agent inspector with checks and files in `/home/factory-user/orrery/explore/ui-v1/shots/l-default.png` and `/home/factory-user/orrery/explore/ui-v1/shots/p5-floor.png`.
[inferred] The proposals below change the contract between architect and agents rather than expand that activity inspector.

[sourced] WebSearch ran with `text=true` to retrieve source page bodies, including primary-source abstracts.
[sourced] FetchUrl was loaded, but its tool policy permits only user-supplied URLs and Factory documentation, which excludes the discovered research URLs.
[inferred] This is a retrieval limitation, not evidence that search snippets alone establish the claims.
[inferred] "New" below means a different agent-development workflow, not a claim that nobody has previously implemented the idea.

## Artifacts and rituals

| Discipline | Artifact or ritual | Problem it solves | Orrery translation |
|---|---|---|---|
| Construction | [sourced] Drawing baselines and revision control establish the approved configuration. [S11] | [sourced] Later modifications need an identifiable, verified baseline. [S11] | [inferred] Issue a revision-bound drawing set for a territory, with ports, budgets, and accepted choices. |
| Construction | [sourced] BIM Collaboration Format links issues to model element GUIDs, views, and coordinates. [S1] | [sourced] Different BIM applications need to exchange model-based issues without proprietary workflows. [S1] | [inferred] Every question and review comment references stable machine and belt IDs through both UI and MCP. |
| Construction | [sourced] BCF supports coordination issues between discipline models. [S1] | [inferred] Individually plausible designs can conflict at their interfaces. | [inferred] Compare agents' intended contracts before their branches merge, not just their edited files. |
| Construction | [sourced] RFIs have reviewers, an official response, closure, and an activity log. [S2] | [sourced] Builders need authoritative clarification rather than scattered answers. [S2] | [inferred] Pin an unresolved question to a port and name the human who can settle it. |
| Construction | [sourced] Submittals present proposed products and methods for approval before installation. [S3] | [sourced] Late rejection causes rework after fabrication or delivery. [S3] | [inferred] An agent submits its schema, dependency, or migration plan before expensive implementation. |
| Construction | [sourced] Change orders record agreed changes to contracted scope and price. [S4] | [sourced] Extra work needs explicit agreement beyond the original contract. [S4] | [inferred] An agent requests a scope change with affected budgets and reversal cost, without changing the budgets itself. |
| Construction | [sourced] Rolling punch lists assign defects during work and site walkthroughs. [S5] | [sourced] Deferring inspection creates a closeout scramble. [S5] | [inferred] Walk a request path, pin failed acceptance evidence, and dispatch bounded correction work. |
| Construction | [sourced] As-built drawings record differences between design and final construction. [S6] | [sourced] Future operators need the actual installation after original participants leave. [S6] | [inferred] Preserve designed intent while accepting a revision-bound declared and observed handover. |
| Mechanical CAD | [sourced] Sketch constraints express relationships, and a constraint manager locates errors. [S7] | [inferred] Editing independent dimensions can violate design intent elsewhere. | [inferred] Draw relationships between capacity, concurrency, and flow budgets, then show which requirement blocks a change. |
| Electronics | [sourced] KiCad DRC checks missing connections, clearances, widths, and custom rules. [S8] | [sourced] A visually plausible board can violate connectivity or manufacturing constraints. [S8] | [inferred] Mark incompatible ports, missing timeouts, or forbidden flows directly on belts. |
| Mechanical design | [sourced] Tolerance analysis supports worst-case, statistical, and root-sum-square methods. [S9] | [sourced] Parts must fit together despite permitted dimensional variation. [S9] | [inferred] Show how local latency and resource allowances consume an end-to-end budget, with explicit statistical assumptions. |
| Manufacturing | [sourced] A manufacturing bill of materials lists components, quantities, and assembly information. [S10] | [sourced] Production planning needs the complete assembly rather than isolated parts. [S10] | [inferred] Generate a territory inventory of services, dependencies, credentials required, and deployment resources without exposing secret values. |
| PLM | [sourced] BOM software manages versions and engineering change orders. [S10] | [inferred] A substitution must remain connected to the assembly revision that uses it. | [inferred] A changed library or external API contract identifies affected machines, checks, and outstanding agent work. |
| Aerospace | [sourced] PDR and CDR are staged design reviews in NASA's lifecycle. [S11] | [sourced] Requirements and designs mature before final fabrication and integration. [S11] | [inferred] Review a risky territory's architecture before implementation and its evidence before acceptance, without gating every small edit. |
| Safety engineering | [sourced] Software FMEA records failure modes, effects, severity, detection, and compensation. [S12] | [sourced] Component failures must be assessed through their consequences on higher levels. [S12] | [inferred] Ask what a Tank or Dock failure does to the complete request path, then require detection and recovery evidence. |
| Systems engineering | [sourced] SysML v2 provides textual and graphical representations plus a standard model API. [S13] | [sourced] Tools need consistent access to the same underlying model. [S13] | [inferred] Human drawing edits and MCP operations modify typed semantic objects rather than separate explanations. |
| Systems engineering | [sourced] NASA uses requirements verification matrices and requirements management. [S11] | [inferred] A requirement without linked evidence can disappear between specification and delivery. | [inferred] Follow a budget through assigned work, a protected check, its result, and the accepted system revision. |
| Manufacturing twins | [sourced] NIST calls for verification, validation, and uncertainty quantification throughout twin lifecycles. [S14] | [sourced] Simulations need credibility for their intended decisions. [S14] | [inferred] Display prediction error, applicable workload, calibration age, and unsupported scenarios beside each simulation. |
| SCADA and HMI | [sourced] ISA's high-performance HMI guidance uses display hierarchy, embedded trends, and redundant alarm coding. [S15] | [sourced] Raw numbers and color misuse make abnormal conditions harder to interpret. [S15] | [inferred] Keep brass quiet during normal operation and expose violated limits with text, shape, and color, not motion alone. |
| Film and games | [sourced] Flow Production Tracking tracks shots, assets, tasks, resources, and versions. [S16] | [inferred] Many specialists need to know which output is ready and what depends on it. | [inferred] A shot-board-style territory board shows deliverables and review readiness rather than session duration. |
| Film production | [sourced] Dailies Smart Playlists dynamically select versions awaiting review. [S17] | [sourced] Reviewers otherwise spend time collecting the day's new work. [S17] | [inferred] Queue changed system behaviors and contested decisions into a reviewable playback. |
| Asset pipelines | [sourced] ShotGrid integrations publish design work for handoff between departments. [S18] | [inferred] Work in progress and reusable approved output need different status. | [inferred] A schema or machine contract becomes consumable by other agents only after its declared publication gate. |
| Software architecture | [sourced] C4 supplies hierarchical abstractions, while Structurizr defines a model and views. [S19] [S20] | [inferred] Different readers need different detail without independently maintained drawings. | [inferred] Render architect, implementer, and operator projections of identical machine IDs. |
| Software architecture | [sourced] IcePanel reuses model objects, propagating changes across references. [S21] | [sourced] Duplicate diagram objects otherwise increase maintenance. [S21] | [inferred] One service appears in several territories without acquiring separate contracts. |
| Software architecture | [sourced] ADRs preserve decision context, status, and consequences. [S22] | [sourced] Newcomers otherwise blindly accept or reverse earlier decisions. [S22] | [inferred] Attach the choices ledger to affected objects and retain superseded decisions. |
| Evolutionary architecture | [sourced] Fitness functions assess architectural characteristics through tests, metrics, or monitors. [S23] | [sourced] Architectural intent needs continual validation. [S23] | [inferred] Protected drawing budgets become agents' objectives and independent acceptance evidence. |
| Communication research | [sourced] Clark and Brennan distinguish coordination of content from coordination of process. [S24] | [sourced] Common ground must change as joint activity proceeds. [S24] | [inferred] An agent's readback names what it believes, what it will change, and which revision it understands. |
| Cooperative work | [sourced] Star and Griesemer describe boundary objects that retain identity across different viewpoints. [S25] | [sourced] Heterogeneous participants need cooperation without identical perspectives. [S25] | [inferred] Preserve shared IDs and constraints while letting humans see a plant and agents receive structured contracts. |
| Human-AI research | [sourced] Holstein and Satzger distinguish domain, information-processing, and complementarity-awareness mental models. [S26] | [inferred] Knowing the system is different from knowing how an agent reasons and where its limits lie. | [inferred] Separate system evidence, agent claims, and human authority instead of calling all three "understanding." |
| Multiplayer design | [sourced] Figma resolves property conflicts using server ordering in its described architecture. [S27] | [sourced] Concurrent editors need convergent document state. [S27] | [inferred] Permit independent changes but reject stale semantic writes that would silently replace another actor's decision. |
| Collaborative documents | [sourced] Google Docs' published protocol tracks revisions, pending edits, sent edits, and acknowledgments. [S28] | [sourced] Concurrent editing needs both transformed edits and a coordination protocol. [S28] | [inferred] Distinguish an agent's proposed, persisted, validated, and accepted model changes. |
| BIM collaboration | [sourced] Revit worksets and element borrowing assign editable portions of a shared model. [S29] | [sourced] Functional work divisions and protected elements reduce accidental changes. [S29] | [inferred] Territories advertise authority and bounded reservations, while shared interfaces require explicit coordination. |

## Ideas Orrery could steal or invert

### 1. Issued-for-agents drawing sets

[inferred] The architect selects a territory and issues its current drawing as a work package with stable IDs, port contracts, protected budgets, and required evidence.
[inferred] Agents see the issued revision, permitted changes, and unresolved assumptions before they start.
[inferred] This replaces open-ended task prose with an inspectable contract and keeps a later drawing edit from silently changing an agent's assignment.
[sourced] NASA's configuration baselines connect specifications, drawings, parts lists, and verification. [S11]
[sourced] SysML v2 supplies complementary text and graphics over the underlying model. [S13]
[inferred] The agentic change is that delegation starts from selected model objects, and completion returns evidence against those same objects.
[guess] Freezing too much would obstruct exploration, so issuance should distinguish hard constraints from questions the agent may investigate.

### 2. RFIs with an agent readback

[inferred] A bot pins a question to the exact belt or port where the drawing leaves a consequential ambiguity.
[inferred] The card states the conflicting interpretations, the evidence needed, and which work can continue safely.
[inferred] The architect gives an official answer, and the agent returns a structured readback of the resulting contract before acting.
[inferred] This addresses confident implementation of an interpretation the human never intended.
[sourced] Autodesk RFIs distinguish reviewer responses from an official response and preserve their activity history. [S2]
[sourced] Clark and Brennan treat common ground as continually updated coordination, not merely possession of the same information. [S24]
[inferred] The agentic change is making acknowledged interpretation a model artifact rather than assuming that sending context establishes understanding.
[guess] Mandatory readbacks for routine edits would become empty ceremony, so trigger them on ambiguity with material reversal cost.

### 3. Submittals before commitment

[inferred] Before introducing an external dependency, public schema, or destructive migration, an agent submits a small proposed implementation attached to the affected machine.
[inferred] The human compares alternatives, sees consumers and reversal cost, and approves the proposal or requests revision.
[inferred] Cheap investigation continues, but expensive dependent work waits for publication of the contract.
[inferred] This targets premature commitment rather than review after the agent has already built the wrong system.
[sourced] Construction submittals review proposed products and methods before fabrication and installation. [S3]
[sourced] ShotGrid separates working data from published work handed to other teams. [S18]
[inferred] The agentic change is an approval point before a bot's speculative choice becomes assumptions for several other bots.
[guess] The risk is a human approval queue, so only high-reversal choices belong at this gate.

### 4. Semantic clash detection

[inferred] The floor shows a shared interface where two agents propose incompatible contracts, even when their patches touch different files.
[inferred] The architect opens the clash and chooses to reconcile, serialize, or reassign the work.
[inferred] Examples include incompatible event versions, competing database ownership, and two territories allocating the same spare capacity.
[inferred] This solves conflicts of intent that a clean Git merge cannot settle.
[sourced] BCF supports coordination issues between discipline models and identifies affected elements. [S1]
[sourced] Revit recommends functional worksets that avoid overlapping functionality. [S29]
[inferred] The agentic change is declaring intended writes and assumptions before implementation, so coordination can happen before wasted construction.
[guess] Inferred intent may be wrong, so show the conflicting assertions and evidence rather than blocking work on an opaque collision score.

### 5. Tolerance budgets instead of isolated targets

[inferred] The architect selects a request path and sees its end-to-end latency allowance distributed across machines, belts, queues, and external calls.
[inferred] An agent proposes a local change and sees which downstream margin it consumes.
[inferred] The user can compare a worst-case bound with a calibrated replay instead of treating every green local gauge as a green system.
[sourced] Mechanical tolerance analysis evaluates whether component variation still permits assembly fit. [S9]
[sourced] Onshape makes relationships and constraint errors inspectable. [S7]
[inferred] This translates local agent objectives into an explicit system constraint rather than rewarding improvements that transfer cost elsewhere.
[inferred] Summing component p95 values does not establish the path's p95, and mechanical root-sum-square calculations must not be copied without their statistical assumptions.
[guess] A visible decomposition may create false confidence unless correlations, queueing, and unmeasured segments remain visible.

### 6. Change orders that reopen the right choices

[inferred] When a task exceeds its territory, the bot proposes a drawing change with affected consumers, invalidated checks, expected cost, and rollback conditions.
[inferred] The architect reviews that change beside the existing choices ledger, then edits protected requirements personally if needed.
[inferred] This separates failure to meet a target from an authorized decision that the target no longer fits the product.
[sourced] Construction change orders explicitly amend scope and agreed price. [S4]
[sourced] Nygard's ADR pattern retains proposed, accepted, deprecated, and superseded decisions. [S22]
[sourced] Orrery orders choices by risk = (1 - confidence) x reversal cost in `/home/factory-user/orrery/explore/map.md:42`.
[inferred] The agentic change is that agents cannot quietly redefine success, but they can request a reasoned change to their assignment.
[guess] A model-generated cost estimate could appear more certain than it is, so retain its assumptions and label unsupported estimates.

### 7. As-built acceptance instead of "agent done"

[inferred] An agent hands over a machine with the declared implementation revision, protected check results, observed traffic where available, and known gaps.
[inferred] The architect inspects the designed, declared, and observed layers separately before accepting the territory.
[inferred] The punch list keeps missing telemetry or unresolved behavior open without pretending the entire machine does not exist.
[inferred] This makes completion a claim with evidence rather than a session status.
[sourced] As-built documentation preserves the actual construction and changes from the design. [S6]
[sourced] Rolling punch lists assign unresolved work throughout construction. [S5]
[inferred] The agentic change is tying the handover to the same drawing that issued the work, with independent checks the agent cannot weaken.
[guess] Telemetry can omit rare paths, so observed traffic must never certify the absence of unobserved dependencies.

### 8. Prediction warrants and failure rehearsals

[inferred] Simulate shows a proposed change together with the workload it was calibrated against, recent prediction error, and assumptions that the proposed change violates.
[inferred] The architect can require a measurement or failure rehearsal instead of accepting an attractive but unsupported prediction.
[inferred] A failure card names its mode, system-level effect, detection mechanism, and recovery behavior.
[sourced] NIST requires credibility assessment through verification, validation, and uncertainty quantification. [S14]
[sourced] NASA software FMEA connects failure modes to effects, detection, and compensation. [S12]
[inferred] Keep failure severity separate from the choices ledger's uncertainty and reversal-cost ordering.
[inferred] The agentic change is making an agent's forecast carry a bounded reason to trust it before the forecast directs other agents.
[guess] One universal confidence score would hide different failure mechanisms, so keep calibration evidence and extrapolation warnings separate.

### 9. Dailies of system changes

[inferred] The architect starts a short review playlist of changed request paths, unresolved clashes, budget regressions, and high-risk choices.
[inferred] Each item plays a before-and-after trace or demonstration and opens the exact drawing objects affected.
[inferred] The human accepts the result, pins a correction, or requests a new experiment instead of reading every bot's account.
[sourced] ShotGrid Smart Playlists collect new versions awaiting review, including a documented previous-24-hours workflow. [S17]
[sourced] Flow Production Tracking connects tasks, assets, version history, and review tools. [S16]
[inferred] The agentic change is supervising results across many bots through evidence selected by system impact rather than supervising their conversation volume.
[guess] A replay can omit the difficult case, so the playlist must include failed checks and unreviewed risk, not only agent-selected demonstrations.

### 10. Semantic presence with bounded authority

[inferred] Each bot shows the model revision it read, the assumptions it relies on, its intended writes, and the territory where it has authority.
[inferred] The human can inspect stale assumptions and resolve overlapping reservations without following animated cursors.
[inferred] Layout edits may merge independently, while contract edits require revision preconditions and revalidation.
[sourced] Figma's published design orders property writes, and Google Docs' protocol distinguishes pending, sent, and acknowledged changes. [S27] [S28]
[sourced] Boundary objects retain identity across different participants' viewpoints. [S25]
[inferred] The agentic change is that presence describes anticipated semantic work, not only the current cursor location.
[inferred] A shared external model does not prove that a human and a neural agent share an internal world model.
[sourced] Embodied-agent research describes world modeling as perception, planning, reasoning, and memory. [S32]
[guess] Reservations can deadlock or outlive crashed agents, so they need expiry and visible recovery without weakening human-only controls.

## What to avoid

### A second model-maintenance job

[sourced] NBS's 2018 survey summary names missing in-house expertise as the main BIM adoption barrier, with training, time, and client demand also obstacles. [S30]
[inferred] This historical finding does not establish today's adoption rates, but it contradicts the assumption that an attractive shared model creates its own adoption.
[sourced] Henderson's MBSE study reports upfront setup, tool costs, training demands, and models neglected under project pressure. [S31]
[inferred] Orrery should create useful coordination artifacts from real work, then ask humans only for intent and judgment that cannot be derived.
[guess] Requiring a complete plant before the first useful delegation would reproduce the adoption failure.

### A universal formal language as the admission price

[sourced] Henderson reports cases where specialist engineers found models difficult to interpret despite training and still needed traditional documents. [S31]
[sourced] Autodesk's ShotGrid adoption guidance warns against capturing the entire process at once and recommends beginning with one concrete challenge. [S18]
[inferred] Keep repository YAML and stable IDs, but do not require users to master SysML or a PLM lifecycle before assigning one territory.
[inferred] Generate audience-specific views of shared semantics instead of forcing every participant to consume the same drawing.

### A beautiful plant that hides abnormal conditions

[sourced] ISA's guidance warns against alarm colors used for ordinary states, color-only status, and screens dominated by raw values. [S15]
[sourced] It recommends embedded trends with desirable ranges and progressive exposure of detail. [S15]
[inferred] Orrery's brass machines should retain their identity while making diagnosis possible without interpreting decorative animation.
[inferred] During an incident, show the violated limit, affected path, direction of change, and permitted actions before agent theatrics.

### Convergence mistaken for agreement

[sourced] Figma's described property-conflict behavior retains the last server-ordered value for the same property. [S27]
[inferred] That guarantees a selected document value, not an authorized architectural decision.
[inferred] Do not silently last-write-win budget changes, accepted contracts, or choices with expensive reversals.
[sourced] BuildingSMART warns that file-based BCF roundtripping depends on maintaining the shared file's integrity rather than circulating multiple copies. [S1]
[inferred] A successful YAML merge likewise does not prove semantic consistency among several agent plans.

### Rubber-stamp understanding and false certainty

[sourced] Holstein and Satzger's framework is conceptual research, not proof that a shared canvas improves coding-agent outcomes. [S26]
[sourced] Clark and Brennan describe costs of grounding, including formulation, understanding, delay, and repair. [S24]
[inferred] Repeating the drawing back in polished prose is not sufficient evidence of correct understanding.
[inferred] Test consequential interpretations through an explicit contract, example, or check.
[inferred] Keep unknowns unresolved when the evidence is missing, and never let agents alter their own acceptance criteria.
[guess] For the bakeoff, the decisive experiment is whether one architect catches a planted cross-agent contract conflict and accepts a correct handover with less effort than through chat and PRs alone.

## Sources opened

[sourced] The first 32 entries supply the cited evidence, including the original Star and Griesemer abstract and the NIST publication abstract rather than claims from unread full papers.
[sourced] Full-text search returned substantive page bodies for 69 unique URLs, including alternate versions and incidental results.
[sourced] The additional 37 URLs appear below for retrieval auditing and do not supply evidence for the argument.

[S1]: https://technical.buildingsmart.org/standards/bcf/
[S2]: https://help.autodesk.com/view/BUILD/ENU/?guid=Process_RFI
[S3]: https://www.autodesk.com/blogs/construction/submittals-template/
[S4]: https://www.autodesk.com/blogs/construction/change-order/
[S5]: https://www.autodesk.com/blogs/construction/what-is-a-punch-list/
[S6]: https://www.autodesk.com/blogs/construction/as-built-drawings/
[S7]: https://cad.onshape.com/help/Content/constraints.htm
[S8]: https://docs.kicad.org/9.0/en/pcbnew/pcbnew.html
[S9]: https://www.autodesk.com/solutions/tolerance-analysis-workflow
[S10]: https://www.ptc.com/en/technologies/plm/bill-of-materials/mbom
[S11]: https://www.nasa.gov/wp-content/uploads/2018/09/nasa_systems_engineering_handbook_0.pdf
[S12]: https://swehb.nasa.gov/display/SWEHBVC/8.5+-+SW+Failure+Modes+and+Effects+Analysis?desktop=true&macroName=show-if
[S13]: https://www.omg.org/news/releases/pr2025/07-21-25.htm
[S14]: https://www.nist.gov/publications/credibility-consideration-digital-twins-manufacturing
[S15]: https://blog.isa.org/the-high-performance-hmi
[S16]: https://www.autodesk.com/products/flow-production-tracking/overview
[S17]: https://help.autodesk.com/view/SGSUB/ENU/?guid=SG_Tutorials_tu_screening_room_dailies_html
[S18]: https://help.autodesk.com/cloudhelp/ENU/SG-Automotive/files/am-getting-started/SG_Automotive_am_getting_started_am_before_getting_started_html.html
[S19]: https://c4model.com/abstractions
[S20]: https://docs.structurizr.com/dsl/language
[S21]: https://docs.icepanel.io/core-features/modelling
[S22]: https://www.cognitect.com/blog/2011/11/15/documenting-architecture-decisions
[S23]: https://www.thoughtworks.com/en-us/radar/techniques/architectural-fitness-function
[S24]: https://worrydream.com/refs/Clark_H_1991_-_Grounding_in_Communication.pdf
[S25]: https://worldpece.org/archive/artifact-pdf-document/star-susan-leigh-and-james-r-griesemer-institutional-ecology
[S26]: https://arxiv.org/pdf/2510.08104
[S27]: https://www.figma.com/blog/how-figmas-multiplayer-technology-works/
[S28]: https://drive.googleblog.com/2010/09/whats-different-about-new-google-docs.html
[S29]: https://help.autodesk.com/cloudhelp/2015/ENU/Revit-Collaborate/files/GUID-6054CEF3-C4E6-4334-BA43-568DF5DF5702.htm
[S30]: https://www.thenbs.com/knowledge/the-national-bim-report-2018
[S31]: https://incose.onlinelibrary.wiley.com/doi/full/10.1002/sys.21717
[S32]: https://arxiv.org/html/2506.22355v1

- [sourced] S1. BuildingSMART, BIM Collaboration Format introduction. https://technical.buildingsmart.org/standards/bcf/
- [sourced] S2. Autodesk, Process RFI. https://help.autodesk.com/view/BUILD/ENU/?guid=Process_RFI
- [sourced] S3. Autodesk, Construction submittals. https://www.autodesk.com/blogs/construction/submittals-template/
- [sourced] S4. Autodesk, Change orders. https://www.autodesk.com/blogs/construction/change-order/
- [sourced] S5. Autodesk, Construction punch lists. https://www.autodesk.com/blogs/construction/what-is-a-punch-list/
- [sourced] S6. Autodesk, As-built drawings. https://www.autodesk.com/blogs/construction/as-built-drawings/
- [sourced] S7. Onshape, Working with constraints. https://cad.onshape.com/help/Content/constraints.htm
- [sourced] S8. KiCad 9, PCB editor. https://docs.kicad.org/9.0/en/pcbnew/pcbnew.html
- [sourced] S9. Autodesk, Tolerance analysis workflow. https://www.autodesk.com/solutions/tolerance-analysis-workflow
- [sourced] S10. PTC, Manufacturing bill of materials. https://www.ptc.com/en/technologies/plm/bill-of-materials/mbom
- [sourced] S11. NASA, Systems engineering handbook. https://www.nasa.gov/wp-content/uploads/2018/09/nasa_systems_engineering_handbook_0.pdf
- [sourced] S12. NASA, Software FMEA. https://swehb.nasa.gov/display/SWEHBVC/8.5+-+SW+Failure+Modes+and+Effects+Analysis?desktop=true&macroName=show-if
- [sourced] S13. OMG, SysML v2 final adoption. https://www.omg.org/news/releases/pr2025/07-21-25.htm
- [sourced] S14. NIST, Digital twin credibility abstract. https://www.nist.gov/publications/credibility-consideration-digital-twins-manufacturing
- [sourced] S15. ISA, High-performance HMI. https://blog.isa.org/the-high-performance-hmi
- [sourced] S16. Autodesk, Flow Production Tracking. https://www.autodesk.com/products/flow-production-tracking/overview
- [sourced] S17. Autodesk, Screening Room dailies. https://help.autodesk.com/view/SGSUB/ENU/?guid=SG_Tutorials_tu_screening_room_dailies_html
- [sourced] S18. Autodesk, ShotGrid adoption guidance. https://help.autodesk.com/cloudhelp/ENU/SG-Automotive/files/am-getting-started/SG_Automotive_am_getting_started_am_before_getting_started_html.html
- [sourced] S19. C4, Abstractions. https://c4model.com/abstractions
- [sourced] S20. Structurizr DSL, Language reference. https://docs.structurizr.com/dsl/language
- [sourced] S21. IcePanel, Modelling. https://docs.icepanel.io/core-features/modelling
- [sourced] S22. Michael Nygard, Documenting architecture decisions. https://www.cognitect.com/blog/2011/11/15/documenting-architecture-decisions
- [sourced] S23. Thoughtworks, Architectural fitness functions. https://www.thoughtworks.com/en-us/radar/techniques/architectural-fitness-function
- [sourced] S24. Clark and Brennan, Grounding in communication. https://worrydream.com/refs/Clark_H_1991_-_Grounding_in_Communication.pdf
- [sourced] S25. Star and Griesemer, Original paper abstract. https://worldpece.org/archive/artifact-pdf-document/star-susan-leigh-and-james-r-griesemer-institutional-ecology
- [sourced] S26. Holstein and Satzger, Mental-model framework. https://arxiv.org/pdf/2510.08104
- [sourced] S27. Figma, Multiplayer technology. https://www.figma.com/blog/how-figmas-multiplayer-technology-works/
- [sourced] S28. Google, Docs collaboration protocol. https://drive.googleblog.com/2010/09/whats-different-about-new-google-docs.html
- [sourced] S29. Autodesk, Planning Revit worksets. https://help.autodesk.com/cloudhelp/2015/ENU/Revit-Collaborate/files/GUID-6054CEF3-C4E6-4334-BA43-568DF5DF5702.htm
- [sourced] S30. NBS, National BIM Report 2018 summary. https://www.thenbs.com/knowledge/the-national-bim-report-2018
- [sourced] S31. Henderson, MBSE adoption experiences. https://incose.onlinelibrary.wiley.com/doi/full/10.1002/sys.21717
- [sourced] S32. Embodied AI Agents, Modeling the World. https://arxiv.org/html/2506.22355v1

### Additional returned pages, not cited

- [sourced] https://www.makerlessons.com/drafting/cad/onshape/sketching-constraints
- [sourced] https://control.com/technical-articles/going-gray/
- [sourced] https://hidekazu-konishi.com/entry/architecture_decision_records_templates_and_operations.html
- [sourced] https://developers.google.com/workspace/meet/api/guides/overview
- [sourced] https://www.researchgate.net/publication/372019617_BIM_and_BPMN_20_Integration_for_Interoperability_Challenge_in_Construction_Industry
- [sourced] https://arxiv.org/abs/2407.17489
- [sourced] https://docs.hedgedoc.org/dev/ot/
- [sourced] https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions.html
- [sourced] https://www.deloitte.com/us/en/insights/industry/technology/ai-and-vr-model-for-human-ai-collaboration.html
- [sourced] https://www.sciencedirect.com/science/article/pii/S0166497222001961
- [sourced] https://www.researchgate.net/publication/258032891_Boundary_Objects_Social_Meanings_and_the_Success_of_New_Technologies
- [sourced] https://www.autodesk.com/blogs/construction/construction-terms-glossary/
- [sourced] https://www.thenbs.com/knowledge/reports
- [sourced] https://help.autodesk.com/cloudhelp/ENU/SG-Tutorials/files/SG_Tutorials_tu_screening_room_dailies_html.html
- [sourced] https://en.wikipedia.org/wiki/Grounding_in_communication
- [sourced] https://groups.google.com/g/closure-library-discuss/c/GhiycGt0ZH4
- [sourced] https://www.loremine.com/blogs/operational-transformation-algorithm-behind-google-docs
- [sourced] https://www.constructionspecifier.com/nbs-publishes-its-national-bim-report-and-survey/
- [sourced] https://www.researchgate.net/publication/235255679_Revisiting_the_notion_of_boundary_object
- [sourced] https://iitsec.xcdacademy.com/index.cfm/item/39203
- [sourced] https://dev.to/matfrana/common-ground-a-framework-for-human-ai-collaboration-516l
- [sourced] https://help.autodesk.com/cloudhelp/2023/ENU/VRED-ShotGrid/files/VRED-ShotGrid-Workflows/VRED_ShotGrid_for_Automotive_Design_VRED_ShotGrid_Worflows_Working_with_ShotGrid_html.html
- [sourced] https://www.buildingsmart.org/standards/bsi-standards/bim-collaboration-format/
- [sourced] https://www.buildingsmart.org/about/openbim/
- [sourced] https://www.figma.com/blog/multiplayer-editing-in-figma/
- [sourced] https://www.figma.com/blog/making-multiplayer-more-reliable/
- [sourced] https://www.nist.gov/programs-projects/digital-twins-advanced-manufacturing
- [sourced] https://www.nist.gov/digital-twins/validating-and-advancement
- [sourced] https://docs.kicad.org/8.0/en/pcbnew/pcbnew.html
- [sourced] https://docs.kicad.org/10.0/en/pcbnew/pcbnew.html
- [sourced] https://explorers.larc.nasa.gov/2023ESE/pdf_files/NASA04.%20SysEng%20Handbook%20(NASA_SP-2016-6105%20Rev%202).pdf
- [sourced] https://www.omg.org/sysml/sysmlv2/
- [sourced] https://www.omg.org/sysml/news-articles.htm
- [sourced] https://construction.autodesk.com/tools/construction-rfi-tracking/
- [sourced] https://help.autodesk.com/view/BUILD/ENU/?guid=RFIs_Overview
- [sourced] https://plcprogramming.io/blog/high-performance-hmi-isa-101
- [sourced] https://ifactoryapp.com/blog/hmi-design-best-practices
