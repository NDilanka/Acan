# Games, software visualization, and agent supervision

[inferred] Acan should borrow games' control loops, not their demand for constant attention.
[inferred] The strongest shift is from issuing prompts to maintaining desired, verified system conditions.
[sourced] Factorio's remote view makes the overview actionable and exposes whether a ghost has an assigned builder [G1].
[sourced] Software visualization studies support overview tasks, but do not establish routine workplace adoption [S1, S2].
[inferred] Surviving tools connect a scoped picture to queries, evidence, or maintained models rather than replace those workflows.
[guess] The best bakeoff candidates are proof-bearing ghosts, outcome request networks, review backpressure, Pareto workshops, and supervision policies.

## Scope and evidence

[sourced] This research uses the supplied K1 to K20 and catalog in `/home/factory-user/acan/explore/map.md:20-72`, plus the two supplied screenshots.
[sourced] The screenshots already show a waiting queue, agent activity, a Floor timeline, checks, budgets, and recorded choices.
[inferred] Adding those controls again would be incremental, so the proposals below change what the supervisor delegates or optimizes.

[sourced] Research date is 2026-10-17, as supplied in the assignment.
[sourced] Both web tools were loaded, but FetchUrl's schema restricts it to user-supplied URLs.
[sourced] Sources below were opened as page text through WebSearch with `text: true`, including targeted URL searches for paper bodies.
[inferred] Game documentation establishes behavior, not its cognitive benefit. Benefits and agent mappings are therefore labeled separately.
[inferred] "Survived" below means a maintained tool or documented workflow exists, not that daily active use has been independently measured.

## Mechanics table

| Game | Mechanic | Problem it solves | Mapping to agent supervision |
| --- | --- | --- | --- |
| Factorio | [sourced] Production statistics pair production and consumption with selectable timescales and item filters [G2]. | [inferred] Rates expose sustained shortages. Filtering prevents a high-volume product from flattening every other series. | [inferred] Pair accepted outcomes with incoming work. Separate agent execution, CI, review, and integration rates. Never substitute tokens for output. |
| Factorio | [sourced] Alt mode reveals information such as assembler recipes in place [G3]. | [inferred] Recognition replaces repeated inspection without changing the spatial model. | [inferred] Hold a key to expose each machine's current contract, branch, evidence age, and next required decision. |
| Factorio | [sourced] Blueprints copy configured arrangements, support parameters, and place ghosts [G4]. | [inferred] Reuse preserves relationships while separating design from construction. | [inferred] Stamp a verified migration pattern with typed inputs, protected checks, allowed files, and rollback requirements. |
| Factorio | [sourced] Ghosts preserve configuration without performing the real entity's function [G5]. | [inferred] Plans remain visible without falsely appearing operational. | [inferred] Keep desired topology separate from code existence and production verification. A merged PR must not automatically make a ghost live. |
| Factorio | [sourced] Construction robots realize supplied ghosts. Remote-view ghost tint indicates that a robot has been assigned [G1, G4]. | [inferred] Delegation needs visible commitment, not only a pending intention. | [inferred] Distinguish unclaimed, reserved, building, verified, and rejected intent. Show missing permissions or prerequisites before dispatch. |
| Factorio | [sourced] Logistic networks connect provider and requester chests within roboport coverage. Charging capacity can limit throughput [G28]. | [inferred] Automatic dispatch works only inside visible capacity and reachability boundaries. More workers cannot fix every shared-resource bottleneck. | [inferred] Show agent permission coverage and shared CI capacity. A queued agent needs a reason, not an instruction to try harder. |
| Factorio | [sourced] Remote view configures real entities and ghosts, including item delivery requests. The map integrates planet and platform switching [G1]. | [inferred] A supervisor cannot visit every worksite. Uniform interaction prevents context-switch errors. | [inferred] Use the same inspect and approve actions across repositories and environments, with an unmistakable environment label. |
| Factorio | [sourced] The map can display pollution. Production statistics also expose pollution production and absorption [G2, G6]. | [inferred] An overlay reveals consequences outside the machine that caused them. | [inferred] Overlay retry amplification, spend, or review demand along affected paths. Preserve the base layout. |
| Factorio, Space Age | [sourced] Vulcanus constrains building around terrain and lava. Fulgora turns scrap into mixed outputs on separated islands [G7, G8]. | [inferred] Different environments force different production strategies instead of repeating a universal template. | [inferred] Treat a greenfield service and a legacy migration as different operating regimes. Reuse controls, not identical agent policies. |
| Factorio, Space Age | [sourced] Gleba rewards throughput because products spoil. Aquilo adds heat distribution to otherwise familiar factory layouts [G9, G10]. | [inferred] Freshness and enabling infrastructure can dominate nominal production capacity. | [inferred] Expire stale test evidence after relevant changes. Show prerequisite permissions, instrumentation, and environments as enabling dependencies. |
| Satisfactory | [sourced] The Blueprint Designer saves bounded building arrangements and organizes named modules into categories [G11]. | [inferred] Bounded modules reduce repetitive construction without requiring a complete factory abstraction. | [inferred] Offer small, inspectable agent-built modules. Make their interfaces and unsupported contexts explicit before reuse. |
| shapez 2 | [sourced] Three building layers, open production buildings, blueprint sharing, and cut, copy, undo, and redo support large factories [G12]. | [inferred] Local transformations stay inspectable while compositions grow. Reversible editing lowers planning cost. | [inferred] Let a supervisor open a component's transformation steps and undo unexecuted plan edits. Undo must never imply that external effects were reversed. |
| Dyson Sphere Program | [sourced] Stations declare demand, supply, or storage separately for local and remote networks, with range and load constraints [G13]. | [inferred] Desired inventory replaces per-trip dispatch while preserving boundaries. | [inferred] Declare "maintain a verified compatibility result for this API" and let eligible agents satisfy demand within repository and permission limits. |
| Oxygen Not Included | [sourced] Distinct overlays expose temperature, oxygen, power, plumbing, automation, and other systems [G14, G15]. | [inferred] One physical layout supports several otherwise invisible causal systems. | [inferred] Make coverage, execution dependencies, ownership, and spend separate lenses. Do not mix all meanings into one machine color. |
| RimWorld | [sourced] Manual work priorities rank task types per colonist, with left-to-right tie-breaking. A high-priority type can exhaust all available work before others [G16]. | [inferred] A policy replaces repeated assignments, but strict priorities can cause starvation. | [inferred] Publish agent role policies with aging and explicit starvation warnings. Keep capability distinct from preference. |
| RimWorld | [sourced] Storytellers vary event pacing using colony state. Difficulty is a separate setting [G17]. | [inferred] The game separates workload rhythm from challenge magnitude. | [inferred] Pace nonurgent review requests around human capacity without changing severity. Never manufacture incidents or postpone safety-critical alerts for drama. |
| RimWorld | [sourced] The Modern Notifications mod describes base-game messages, alerts, and letters scattered across different screen regions [G18]. | [inferred] Durable conditions and transient events need different handling, but scattered channels increase search cost. | [inferred] Use one decision inbox with condition, event, and approval types. A dismissed event must not clear an unresolved condition. |
| Dwarf Fortress | [sourced] Work orders combine recurring checks, inventory predicates, and dependencies. Multiple conditions use logical AND [G19]. | [inferred] Persistent operating rules replace repeated production orders. | [inferred] "Refresh this contract proof when its dependencies change, provided spend remains below the approved cap." Expose the exact failed predicate. |
| Frostpunk | [sourced] Laws expose mutually exclusive choices and can create promises whose nonfulfillment displeases citizens [G20]. | [inferred] An intervention creates obligations, not just an immediate benefit. | [inferred] A temporary risk exception creates an owner, expiry, and compensating check in the choices ledger. Avoid invented morale scores for agents. |
| Mindustry | [sourced] Logic processors can inspect and control buildings and units, read switches, and output messages or displays [G21]. | [inferred] Repeated reactions become inspectable automation rather than manual micro-management. | [inferred] Graduate proven supervision rules into bounded automata. Keep rule inputs, last action, and emergency disable visible. |
| Opus Magnum and Zachtronics | [sourced] Open-ended solutions compete on simplicity, speed, and compactness. Opus Magnum shows cost, cycle, and area histograms [G22, G23]. | [inferred] Correctness becomes an entry condition, not the end of learning. Separate metrics expose tradeoffs. | [inferred] Compare independently verified candidate changes on latency, spend, and maintenance burden. Show alternatives instead of one opaque score. |
| Kerbal Space Program | [sourced] Maneuver nodes alter a projected orbit before execution. The player then aligns and burns against the plan [G24]. | [inferred] Prediction and execution use different representations of the same maneuver. | [inferred] Scrub a proposed migration and inspect predicted capacity or failure exposure, then compare the actual rollout with that projection. |
| StarCraft and RTS macro | [sourced] Mental-checklist practice uses hotkeys and control groups to allocate attention across micro, minimap, money, and supply [G25]. | [inferred] Stable retrieval and repeated checks reduce forgotten work across simultaneous fronts. | [inferred] Save named groups of machines, agents, and checks. Jump directly to a group's unresolved decisions without camera hunting. |
| StarCraft and RTS macro | [sourced] An idle-worker button selects idle workers, individually or together. Macro practice distinguishes useful checks from APM spam [G25, G26]. | [inferred] Exceptions reveal unused capacity, while action counts do not establish useful attention. | [inferred] Separate intentionally idle agents from runnable work without a qualified owner. Optimize accepted outcomes per human interruption, not prompt APM. |
| Football Manager | [sourced] The official delegation guide recommends assigning responsibilities to backroom staff so the manager can focus on selected areas [G27]. | [inferred] The user chooses which managerial responsibilities to retain, not merely who performs today's task. | [inferred] Offer responsibility-level authority such as investigate, propose, implement, and request approval. Retain human control of budgets and checks. |

## Software visualization, failure and survival

### The evidence does not support a universal failure story

[sourced] Sensalire, Ogao, and Telea evaluated 15 corrective-maintenance tools and identified demand for IDE integration, scalability, multiple views, and query support [S2].
[sourced] They reported that high information density can itself make a tool hard to learn, and recommended multiscale methods [S2].
[sourced] Their conclusion leaves open whether users reject 3D and animation or tool developers avoid providing them [S2].
[inferred] It would therefore be wrong to claim that this research proves engineers dislike visual interfaces.
[guess] The size of the adoption gap across the whole field remains unknown because the opened sources do not measure longitudinal daily use.

### CodeCity and software maps

[sourced] CodeCity represents classes as buildings and packages as districts, with code metrics mapped to dimensions [S1].
[sourced] Its controlled experiment involved 41 participants and reported 24% better correctness and 12% less completion time against Eclipse plus Excel [S1].
[sourced] These were comprehension tasks, not a longitudinal development trial, and the authors identify task representativeness as a validity threat [S1].
[sourced] Overview tasks benefited most. Precise lookup tasks did not outperform the baseline [S1].
[inferred] The failure risk is assigning a city the job of a search result, editor, or debugger.

[sourced] Kuhn, Loretan, and Nierstrasz argue that arbitrary layouts make different views hard to compare, and propose consistent thematic maps based on vocabulary similarity [S3].
[inferred] A map loses value when every refresh requires learning its geography again.
[inferred] Acan's authored layout is therefore a functional advantage, provided source updates do not silently relocate machines.
[inferred] Software maps remain plausible for orientation and risk discussion, but these papers do not prove they should occupy the screen all day.

### Gource

[sourced] Gource animates repository history as a tree, with directories as branches, files as leaves, and contributors acting at recorded times [S4].
[sourced] Its site reports a release in March 2026, so describing it as abandoned would be false [S4].
[inferred] Its durable success is a narrow communication task, explaining project evolution.
[inferred] Repository activity cannot by itself establish current runtime behavior, causal bottlenecks, or whether a change achieved its goal.
[inferred] Acan should use replay to explain a selected event or change, rather than turn perpetual contributor animation into a productivity proxy.

### Observability maps

[sourced] Honeycomb explicitly rejected a static map that users would inspect once and forget [S5].
[sourced] Its team tested real customer graphs because its own small architecture was unrepresentative. Early maps became unreadable tangles [S5].
[sourced] Their design tied filtering and highlighting to questions, then allowed drilldown into represented traces [S5].
[inferred] The important survivor is the investigation workflow, not the node-link drawing.

[sourced] Datadog combines dependency maps with team grouping, scoped inspection, monitor status, and pivots into other telemetry [S6].
[sourced] Its documentation also states that missing spans can make a map unavailable [S6].
[sourced] Grafana combines service graphs with rate, error, and duration tables and trace navigation [S7, S8].
[sourced] Its graph processor requires compatible span relationships and must receive both sides of a pair reliably [S7].
[inferred] These tools survive by keeping the picture attached to operational evidence and conventional tables.
[inferred] Missing observation must remain "unknown", not become "no dependency" or a healthy-looking blank area.

### Architecture tools

[sourced] IcePanel reuses model objects across C4 diagrams and uses three diagram levels with links to reality instead of a fourth code-diagram level [S9].
[sourced] Structurizr generates multiple views from a single textual model [S10].
[sourced] C4's author says context and container diagrams suffice for most teams, and recommends only levels that add value [S11].
[inferred] Their useful abstraction is a shared, maintainable model with selective views, not maximal visual detail.

[sourced] Multiplayer discovers components and dependencies from full-stack session recordings, then shows session-filtered maps inside those recordings [S12].
[sourced] Ilograph combines a model-based textual format, automatic layout, and interactive views of flows and components [S13].
[inferred] These are different maintenance strategies, not proof that any vendor solved daily adoption.
[inferred] Acan can distinguish itself by connecting intended changes to agent authority and verification while preserving disagreement between designed, declared, and observed sources.

### 3D and VR research

[sourced] ExplorViz's 2022 study reports preliminary support for collaborative comprehension in VR and AR, alongside readability and performance problems [S14].
[sourced] The authors explicitly call for further research on effectiveness, usability, and acceptance [S14].
[inferred] This supports optional collaborative inspection, not a mandatory immersive workspace.
[inferred] Camera travel and label reading must not become extra work before an engineer can answer a precise question.
[guess] A headset-first Acan would probably impose more switching cost than value for routine supervision, but the opened studies do not measure that product.

## Ideas Acan could steal or invert

### 1. Proof-bearing ghosts

[guess] The user places an intended system change and sees four separate marks for designed, implemented, checked, and observed.
[guess] Clicking an incomplete mark reveals the missing proof and offers a bounded agent request.
[inferred] This solves false completion, where code exists but the promised system behavior does not.
[sourced] Factorio ghosts hold configuration before construction, and assigned ghosts provide progress feedback [G1, G5].
[inferred] The new agentic step is making completion an evidence-backed system transition rather than an agent's assertion or PR status.
[guess] Risk is verification theater. Each mark needs a revision, scope, timestamp, and inspectable result, with protected checks unchanged.

### 2. Outcome request networks

[guess] A machine advertises a desired condition such as a current compatibility proof or a latency budget satisfied under a named workload.
[guess] The user selects eligible agents and authority once. Agents reserve unsatisfied demand instead of receiving a new prompt for every job.
[inferred] This replaces dispatch work with supervision of persistent obligations.
[sourced] Dyson Sphere Program separates supply, demand, and storage across local and remote boundaries [G13].
[inferred] The new agentic step is representing verified engineering outcomes as renewable demand, rather than treating a ticket as the only unit of work.
[guess] Risk is duplicate or endless work. Reservations, invalidation rules, retry limits, and explicit human stop controls are essential.

### 3. Review backpressure

[guess] The Floor shows queues of build, CI, review, and integration, including how long each accepted change waited.
[guess] The supervisor limits new high-risk construction when review demand exceeds an explicit human capacity.
[inferred] This addresses the bottleneck created when agents produce proposals faster than people can responsibly accept them.
[sourced] Factorio pairs production and consumption over time, while Gleba makes stockpiling actively unhelpful [G2, G9].
[inferred] The agentic inversion is to treat unreviewed output as aging inventory, not automatically valuable progress.
[guess] Risk is a misleading throughput target. A rejected unsafe change may represent successful supervision, so outcome classes must stay visible.

### 4. Pareto workshops

[guess] The supervisor requests alternative implementations under the same frozen checks, dataset, and spend cap.
[guess] Acan presents verified candidates as a latency-versus-cost plot, with changed-file scope and maintenance concerns beside each point.
[guess] The user compares traces and diffs, then selects a tradeoff rather than the agent with the most confident explanation.
[sourced] Opus Magnum separates cost, cycles, and area, and its community documents why independent histogram records may belong to different solutions [G23].
[inferred] The new agentic step is making competing experiments a normal design action rather than an informal second opinion.
[guess] Risk is benchmark gaming or incomparable runs. Show failed checks, workload identity, variance, and evaluation cost.

### 5. Supervision policies

[guess] The supervisor chooses responsibility policies per territory, with explicit rules for investigate, propose, implement, and request approval.
[guess] A policy preview explains which current tasks each agent would take, which it would refuse, and why.
[inferred] This reduces repeated prompting while preserving understandable authority.
[sourced] RimWorld exposes work priorities, Football Manager delegates responsibilities, and Mindustry turns repeated control into programs [G16, G21, G27].
[inferred] The agentic difference is a durable delegation contract governing future situations rather than a task-specific instruction.
[guess] Risk is hidden starvation or privilege expansion. Show waiting time and refusal reasons. Agents must never modify the protected budgets or checks.

### 6. One actionable map

[guess] A supervisor selects a territory or environment and acts directly on the selected machine without opening a second, noninteractive preview.
[guess] Control groups save camera position, relevant agents, filters, and outstanding decisions together.
[inferred] This reduces navigation between system understanding and the action it justifies.
[sourced] Factorio removed a planet-preview workflow because users tried to interact with the preview. It integrated switching into remote view [G1].
[sourced] RTS practice uses control groups to retrieve work quickly [G25].
[inferred] The new agentic step is treating a saved view as an executable supervision context.
[guess] Risk is acting in the wrong environment. Display environment and authority beside every consequential action.

### 7. Evidence freshness

[guess] A supervisor sees which checks and assumptions predate the code, workload, or dependency they claim to cover.
[guess] Selecting stale evidence shows the invalidating event and dispatches a focused refresh.
[inferred] This makes uncertainty actionable instead of rendering a generic fog score.
[sourced] Gleba preserves spoilage as an object because disappearing products would hide what happened [G9].
[inferred] The new agentic step is keeping obsolete proof and its invalidation cause visible while agents renew it.
[guess] Risk is invented precision. Evidence expiration must follow explicit dependency rules, not an arbitrary universal half-life.

### 8. Obligations after exceptions

[guess] Approving a risky shortcut draws its compensating check, expiry, and owner beside the affected flow.
[guess] The supervisor can compare leaving the exception in place with assigning an agent to remove it.
[inferred] This prevents temporary decisions from disappearing after the immediate incident.
[sourced] Frostpunk laws can create promises whose fulfillment is tracked separately from signing the law [G20].
[inferred] The new agentic step is attaching follow-through work to approval itself, without permitting agents to relax the objective.
[guess] Risk is treating ethics or operational judgment as a game score. Record the real obligation and evidence instead.

## What to avoid

- [inferred] Avoid a prettier graph with no next action. Honeycomb's own design account rejects the "inspect once" map and documents tangled real-world graphs [S5].
- [inferred] Avoid replacing exact search with spatial hunting. CodeCity's experiment did not beat its baseline on focused lookup tasks [S1].
- [inferred] Avoid changing geography when switching lenses. Consistent-map research identifies incomparable layouts as a comprehension problem [S3].
- [inferred] Avoid motion as evidence of productivity. Gource visualizes recorded activity, not accepted outcomes or runtime correctness [S4].
- [inferred] Avoid maximizing density without progressive disclosure. The maintenance-tool study found both insufficient scalability and high-density learning barriers [S2].
- [inferred] Avoid making VR the price of collaboration. ExplorViz reports readability and performance problems even in preliminary positive studies [S14].
- [inferred] Avoid copying strict work priorities without starvation handling. RimWorld's documented behavior can exhaust one work type before anything lower-ranked [G16].
- [inferred] Avoid rewarding supervisor APM or agent busyness. RTS attention training distinguishes useful checks from repetitive actions [G25].
- [inferred] Avoid hiding missing telemetry behind a complete-looking plant. Datadog and Grafana document concrete conditions under which dependency reconstruction is incomplete or unavailable [S6, S7].

## Sources opened

[sourced] The following 42 source pages supplied the evidence cited above. Primary papers, developer documentation, and game wikis dominate. G18 is a mod author's account of interface friction, G23 is a player-researcher's account, and G24 is a first-hand tutorial on the official forum.

- G1. https://factorio.com/blog/post/fff-380
- G2. https://wiki.factorio.com/Production_statistics
- G3. https://wiki.factorio.com/Shortcut_bar
- G4. https://wiki.factorio.com/Blueprint
- G5. https://wiki.factorio.com/Ghost
- G6. https://wiki.factorio.com/pollution
- G7. https://factorio.com/blog/post/fff-386
- G8. https://www.factorio.com/blog/post/fff-399
- G9. https://factorio.com/blog/post/fff-414
- G10. https://factorio.com/blog/post/fff-432
- G11. https://satisfactory.wiki.gg/wiki/Blueprint_Designer
- G12. https://www.gamespress.com/Factory-building-Favourite-shapez-2-Launches-on-PC-following-700000-Ea
- G13. https://dyson-sphere-program.fandom.com/wiki/Interstellar_Logistics_Station
- G14. https://oxygennotincluded.wiki.gg/wiki/Category:Overlays
- G15. https://oxygennotincluded.wiki.gg/wiki/Temperature_Overlay
- G16. https://rimworldwiki.com/wiki/Work
- G17. https://rimworldwiki.com/wiki/AI_Storytellers
- G18. https://steamcommunity.com/workshop/filedetails/?id=3752932665
- G19. https://dwarffortresswiki.org/Work%20orders
- G20. https://frostpunk.game-vault.net/wiki/Book_of_laws
- G21. https://mindustrygame.github.io/wiki/logic/0-introduction/
- G22. https://www.zachtronics.com/opus-magnum/
- G23. https://biggieblog.com/tracking-the-global-opus-magnum-records/
- G24. https://forum.kerbalspaceprogram.com/topic/194822-how-do-i-properly-use-maneuver-nodes-for-the-mun/
- G25. https://liquipedia.net/starcraft2/Mental_Checklist
- G26. https://liquipedia.net/starcraft2/Unit_Shortcut_Tricks
- G27. https://www.footballmanager.com/the-dugout/delegating-success-football-manager-26
- G28. https://wiki.factorio.com/Logistic_network
- S1. https://wettel.github.io/download/Wettel11a-icse.pdf
- S2. https://webspace.science.uu.nl/~telea001/uploads/PAPERS/SoftVis08/paper2.pdf
- S3. https://arxiv.org/pdf/1209.5490
- S4. https://gource.io/
- S5. https://www.honeycomb.io/blog/interating-toward-service-map
- S6. https://docs.datadoghq.com/tracing/services/services_map/
- S7. https://grafana.com/docs/tempo/latest/metrics-from-traces/service_graphs/
- S8. https://grafana.com/docs/tempo/latest/metrics-from-traces/service_graphs/service-graph-view/
- S9. https://docs.icepanel.io/core-features/diagramming
- S10. https://structurizr.com/
- S11. https://c4model.com/diagrams
- S12. https://www.multiplayer.app/docs/system-dashboard/
- S13. https://www.ilograph.com/
- S14. https://www.sciencedirect.com/science/article/pii/S095058492200132X
