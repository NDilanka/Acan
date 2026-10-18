# Plant fidelity to the system

## Summary

[inferred] Acan should derive meaningful machine parts from inspectable evidence, while retaining the fixed catalog and authored layout.
[inferred] Structure belongs in repeated parts and connections. [inferred] Live state belongs in bounded fills, gauges, and evidence-driven motion.
[inferred] Unknown, stale, zero, stopped, and simulated must remain visibly different.
[inferred] The useful fidelity target is a truthful abstraction, not a miniature of every implementation detail.
[inferred] Start with Acquit's single verifier slot, separate queues, lane identity, actual timers, and real outbound paths.
[inferred] Give every semantic channel a binding, expose it through Alt+hover, and test both its values and its rendered pixels.

## 1. Research and its implications

### Glyphs have a limited perceptual budget

[sourced] Bertin systematized visual variables in *Sémiologie graphique* in 1967, including position, size, shape, value, hue, orientation, and texture. [Visual variables](https://en.wikipedia.org/wiki/Visual_variable).
[sourced] Munzner separates marks, which represent items or links, from channels, which vary their appearance with attributes. [Marks and channels](https://www.cs.ubc.ca/~tmm/talks/vad/VAD-marks.pdf).
[sourced] Her expressiveness principle matches channel type to data type, while effectiveness considers accuracy, discriminability, separability, and popout. [Munzner](https://www.cs.ubc.ca/~tmm/talks/vad/VAD-marks.pdf).
[inferred] Acan's catalog silhouette should encode categorical role, while count and length encode quantities.
[inferred] A language change should change a plate, not make a service twice as wide.

[sourced] Cleveland and McGill's experiments found position judgments more accurate than length and angle judgments, while warning that their ordering is guidance rather than a complete design prescription. [Graphical perception, 1984](https://www.statsclass.com/dsci310/Notes/Cleveland_McGill_EPT.pdf).
[inferred] A mounted dial can communicate proximity to a limit, but exact comparisons need a number or an aligned scale in Inspect.
[inferred] Tank volume, smoke opacity, and furnace brightness are poor substitutes for a precise latency comparison.

[sourced] Chernoff faces map multiple variables to facial features, but those features differ in perceived importance. [Chernoff faces](https://en.wikipedia.org/wiki/Chernoff_face).
[sourced] Naveh-Benjamin and Pachella found that distinctive irrelevant features interfered with similarity judgments even when observers were instructed to ignore them. [1982 experiment](https://doi.org/10.1177/001872088202400102).
[sourced] Morris, Ebert, and Rheingans found serial rather than preattentive processing for the tested face features. [Experimental analysis](https://engineering.purdue.edu/~ebertd/papers/Chernoff_990402.PDF).
[inferred] An assembler with twelve independently changing details risks becoming a mechanical Chernoff face.
[inferred] The warning concerns unequal salience and interference, not a blanket ban on expressive glyphs.

### Ecological displays expose relationships

[sourced] Ecological interface design makes domain constraints perceptually evident through an abstraction hierarchy and a skills, rules, and knowledge framework. [EID overview](https://en.wikipedia.org/wiki/Ecological_interface_design).
[sourced] Its five abstraction levels distinguish functional purpose, abstract function, generalized function, physical function, and physical form. [Abstraction hierarchy](https://en.wikipedia.org/wiki/Ecological_interface_design).
[inferred] These are means-end relationships, not merely five zoom levels.
[inferred] Acquit's display can connect verified payment as purpose, payout correctness and delivery budgets as constraints, verification and escrow as functions, queue and callback behavior as mechanisms, and processes and files as implementation.
[inferred] Selecting a failing payout should expose that chain instead of merely enlarging the PayPal dock.

[sourced] Bennett and Flach argue that displays must support both relationships among variables and focused readings of individual variables. [Graphical displays, 1992](https://doi.org/10.1177/001872089203400502).
[sourced] Their account makes task meaning essential, rather than treating an attractive global shape as sufficient. [Bennett and Flach](https://doi.org/10.1177/001872089203400502).
[inferred] A full waiting tray beside one occupied verifier cell can expose pressure relative to capacity.
[inferred] The developer still needs queue depth, oldest waiting age, and the configured concurrency separately.
[inferred] Do not connect unrelated metrics into a dramatic polygon and call its distortion a system constraint.

### SCADA favors a quiet normal state

[sourced] ISA-101 covers the HMI lifecycle and display structures, with supporting reports on philosophy, usability, and performance. [ISA's series overview](https://www.isa.org/standards-and-publications/isa-standards/isa-101-standards).
[sourced] High-performance HMI practice uses neutral gray normal equipment, contextual ranges, and attention colors for abnormal states. [Design guide](https://wartens.com/us/news/high-performance-hmi-isa-101).
[sourced] That guide distinguishes this practice from a mandatory ISA palette and recommends less decorative animation. [Same guide](https://wartens.com/us/news/high-performance-hmi-isa-101).
[inferred] Acan can retain brass and graphite without painting every healthy machine green.
[inferred] Reserve saturated amber and red for actionable deviation, and pair them with shape and text.
[inferred] Blue blueprint ghosts are a mode and provenance convention, not evidence of runtime health.

### Isotype makes repetition accountable

[sourced] Isotype represents larger quantities with more equal-sized pictograms rather than larger pictograms. [Isotype Revisited](https://isotyperevisited.org/2012/08/introduction.php).
[inferred] Worker cells and deployment copies are good applications because each repeated unit can have a declared meaning.
[inferred] A 200-worker pool should not become one enormous worker or 200 unreadable pistons.
[inferred] An aggregate glyph must disclose its grouping and retain the exact count.

### Factorio's tells work because they have causes

[sourced] Factorio supports working and idle animations, machine-speed-dependent animation, recipe icons in alt-mode, and working visualizations with lights. [Crafting machine](https://lua-api.factorio.com/latest/prototypes/CraftingMachinePrototype.html), [working visualizations](https://lua-api.factorio.com/latest/types/WorkingVisualisations.html), and [lights](https://lua-api.factorio.com/latest/types/WorkingVisualisation.html).
[sourced] Its smoke is an explicitly configured animated entity, not a universal failure signal. [Smoke prototype](https://lua-api.factorio.com/latest/prototypes/SmokePrototype.html).
[sourced] Belt throughput is speed multiplied by item density, and backed-up items expose bottlenecks. [Belt transport](https://wiki.factorio.com/Belt_transport_system).
[inferred] Borrow the connection between state and tell, not every game's visual meaning.
[inferred] Acan's failure smoke needs an explicit failure binding, while moving cargo needs evidence of traffic.
[inferred] An alt-mode plate for operation, protocol, and runtime is more honest than inventing a unique silhouette for every framework.

### Procedural variety is not evidential fidelity

[sourced] Factorio's map generator combines a seed with separate controls for resource frequency, patch size, richness, and terrain. [Map generator](https://wiki.factorio.com/Map_generator).
[sourced] Dwarf Fortress generates terrain through interacting fields and then simulates history, according to Tarn Adams's account. [World generation interview](https://www.polygon.com/2014/7/23/5926447/dwarf-fortress-will-crush-your-cpu-because-creating-history-is-hard/).
[sourced] Townscaper selects compatible prebuilt architectural pieces from player placements and surrounding structure. [Creator interview and technical account](https://www.gamedeveloper.com/game-platforms/how-townscaper-works-a-story-four-games-in-the-making).
[sourced] No Man's Sky combines a generation pipeline with tools for artistic control. [Generation architecture](https://www.gdcvault.com/play/1024265/Continuous_World_Generation_in__No_Man_s_Sky_) and [art direction](https://www.gdcvault.com/play/1021805/Art-Direction-Bootcamp-How-I).
[inferred] These examples separate input parameters, compatibility constraints, simulation, and authored art.
[inferred] Acan should generate compatible machine parts from evidence, but never generate unknown architecture to make the plant look complete.
[inferred] A stable seed can place cosmetic scratches, not decide ports or traffic.

### Software cities demonstrate the trade-off

[sourced] CodeCity maps classes to buildings and packages to districts, with method count driving height and attribute count driving base dimensions. [CodeCity paper](https://wettel.github.io/download/Wettel08b-wasdett.pdf).
[sourced] EvoStreets' history-aware layout aims to evolve smoothly and preserve spatial stability. [Representing development history](https://www.researchgate.net/publication/220955860_Representing_development_history_in_software_cities).
[sourced] A 34-participant EvoStreets experiment did not establish the listed performance advantages of richer viewing environments. [ICPC 2019](https://ieeexplore.ieee.org/document/8813284/).
[inferred] Metric-derived geometry is feasible, but more dimensions and realism do not establish better comprehension.
[inferred] Exact scaling can make large machines occlude small ones, while continuous resizing can disrupt authored spatial memory.

[sourced] Research used WebSearch full text on 2026-10-17, with no rate-limit errors or retries in this session's search results. `/home/factory-user/.factory/artifacts/scripts/4b4a8595-9831-44d3-bf55-9eda2ab10195/`.
[sourced] The retrieved ISA page offers the standard for purchase, and the Bennett-Flach page exposes the abstract with full access gated, so the claims above use those public materials rather than inaccessible chapters. [ISA overview](https://www.isa.org/standards-and-publications/isa-standards/isa-101-standards) and [Bennett-Flach abstract](https://doi.org/10.1177/001872089203400502).
[inferred] No claim of ISA compliance or measured benefit for Acan follows from this research.

## 2. A fidelity model for machines

### Preserve the catalog and derive its semantic parts

[sourced] K4 requires designed, declared, and observed sources, K5 preserves authored layout, and K6 preserves catalog recognition at distance. `/home/factory-user/acan/explore/map.md:29-31`.
[sourced] S1 retains mounted gauges, while R1 requires a flat plan in every mode. `/home/factory-user/acan/explore/map.md:128-142`.
[inferred] Use a catalog shell, evidence-bound parts, and a separate uncertainty overlay.
[inferred] Draw tank layers as plan-view sectors or nested bands, with stacked layers available only in section or exploded views.
[inferred] Represent stacked deployment copies with offset plan outlines, not compulsory perspective.

[sourced] The asset sheet already shares geometry across SQLite, PostgreSQL, and MySQL variants and lists seven universal states. `/home/factory-user/attachments/ae5adbbb-90cc-467f-9499-0ca9036a8e29-mvb27mpz-7e279d60.png`.
[sourced] The supplied scene shows a multi-cell verifier and an elevated assembly. `/home/factory-user/acan/explore/ui-v1/shots/l-default.png`.
[inferred] Treat those images as visual studies, not measurements or amendments to the Acquit corrections.

[sourced] `ASSETS` separates metadata from `build` functions, and the bank derives its width from `cells` and its raised parts from `busy`. `/home/factory-user/acan/explore/ui-v1/assets.mjs:95-110`.
[sourced] Its defaults invent four cells, one busy cell, and seven waiting jobs, while the hopper hardcodes five rings and the tank hardcodes three layers. `/home/factory-user/acan/explore/ui-v1/assets.mjs:98-150`.
[inferred] The generator mechanism is useful, but demo defaults cannot enter a live rendering path.
[inferred] Changing `layers` labels does not establish a binding to actual schema structure.

### Concrete channel assignments

1. [inferred] Bind catalog type to a designed role and declared entity evidence, with observed role evidence retained separately.
   [guess] Recompute on blueprint or source revision, show an unresolved crate when role is unknown, and keep one shell per entity.
   [sourced] Acquit's House is a library inside the API, not a separate process. `/home/factory-user/acan/explore/map.md:195`.
   [inferred] House therefore belongs in the API cutaway unless the architect explicitly models a logical component.

2. [inferred] Bind bank cell count to declared concurrency, with runtime-reported effective concurrency as observed evidence.
   [guess] Show one cell per slot through eight, then a compact bank engraved with the exact count, including `200 slots`.
   [guess] Update structure on configuration revision and occupancy every second, with fogged unknown capacity and no guessed busy cells.
   [sourced] Acquit's production verifier concurrency is one, and its queue is in memory and lost on restart. `/home/factory-user/acan/explore/map.md:190`.
   [inferred] One active span proves activity, not that every other slot is idle.

3. [inferred] Bind tray cargo and hopper fill to observed queue depth, not cumulative submissions.
   [guess] Update once per second, draw exact tokens through seven, then show the exact number beside a bounded fill.
   [guess] Divide fill by declared capacity or a labeled designed warning threshold, never by a hidden rolling maximum.
   [inferred] An unbounded queue needs a warning scale rather than a fictitious capacity.
   [inferred] Missing measurements fog the tray instead of emptying it.
   [sourced] Acquit's database effects queue and verifier's volatile queue are distinct mechanisms. `/home/factory-user/acan/explore/map.md:176,190`.

4. [inferred] Bind tank bands to one explicitly selected schema grouping, with declared migrations compared against observed catalog metadata.
   [guess] Refresh on schema revision or a 60-second metadata check, draw at most six groups, and engrave the remaining count.
   [inferred] Unknown schema gets fogged internals, not three default tables.
   [inferred] SQLite side vessels require observed file existence or declared journaling intent, with those provenances distinguished.
   [sourced] Acquit has one SQLite file per lane and separate model-call and House files. `/home/factory-user/acan/explore/map.md:180-181`.
   [inferred] Draw separate stores rather than relabeling every file as a relational schema.

5. [inferred] Bind repeated outlines to declared replicas or lane instances, and observed readiness to each instance.
   [guess] Refresh on configuration changes and five-second discovery, show three outlines before a count, and fog unknown membership.
   [inferred] Keep replicas and environments separate in the legend.
   [sourced] Acquit identity is role plus lane, with repeated offset ports, not port alone. `/home/factory-user/acan/explore/map.md:181`.
   [inferred] Two lanes must not imply load-balanced replicas.

6. [inferred] Bind mounted openings to declared logical interfaces, with observed listeners and connections recorded independently.
   [guess] Update on source revision and five-second discovery, show up to six grouped openings, then an exact interface count.
   [inferred] Missing discovery leaves declared openings visible but unverified, rather than inventing port numbers.
   [inferred] Protocol sockets, HTTP routes, and database connections need different interface classes.
   [sourced] Acquit needs webhook, redirect, and verifier callback routes, and its production web frontend is Caddy-served static files. `/home/factory-user/acan/explore/map.md:191-193`.

7. [inferred] Bind engraved runtime plates to declared manifests and launch commands, with observed process version as a separate line.
   [guess] Refresh on revision or process restart, show two short plate lines, and move additional languages into Inspect with a count.
   [inferred] Unknown versions show `version unknown`, never a customary Node release.
   [sourced] The Acquit fixture records TypeScript executed directly by Node 24. `/home/factory-user/acan/explore/map.md:250`.

8. [inferred] Bind belt width to a named throughput statistic only in an explicit traffic overlay.
   [guess] Use three labeled width bins from a fixed per-unit scale, update a rolling 60-second rate every five seconds, and use uniform width with fog when unavailable.
   [inferred] Keep authored paths and endpoint positions fixed as width changes.
   [inferred] Requests per second, jobs per second, and bytes per second cannot share an unlabeled scale.
   [inferred] A wider belt means more measured flow, not greater capacity.

9. [inferred] Bind cargo births to sampled observed events, with direction, collection window, and sampling policy inspectable.
   [guess] Cap visible cargo at twelve groups and show a number above that limit, while the clock respects the existing 20 fps cap.
   [inferred] Unknown traffic has no moving cargo, and known zero has an explicit zero reading.
   [inferred] Sampled traces alone cannot establish a true zero or total throughput without suitable counting evidence.
   [sourced] Acquit's linked spans cross an effects queue and a later callback. `/home/factory-user/acan/explore/map.md:176`.
   [inferred] Preserve those handoffs instead of animating one continuous HTTP request.

10. [inferred] Bind return-loop geometry to declared retry policy and motion to observed retry attempts.
    [guess] Refresh policy on revision and activity each second, draw one loop per flow, and collapse policy complexity into a retry count.
    [inferred] Unknown policy gets a fogged fitting, and a policy without retry evidence stays still.
    [sourced] Acquit has different MCP PayPal, API PayPal, and callback retry rules. `/home/factory-user/acan/explore/map.md:188`.
    [inferred] Verify whether each reported count means attempts or retries before engraving a multiplier.

11. [inferred] Bind a timeout fuse's length to a declared or designed deadline and its burn fraction to observed elapsed time divided by that deadline.
    [guess] Update selected in-flight calls at the capped motion rate, show one fuse for one selected call, and replace concurrent-call detail with a count and timeout rate.
    [inferred] Without a deadline, show elapsed time and a fogged fuse scale.
    [inferred] A percentile latency cannot honestly burn like an individual request.
    [sourced] The prototype currently defers the Fuse drawing to a belt budget dimension. `/home/factory-user/acan/explore/ui-v1/assets.mjs:6-13`.

12. [inferred] Bind cam period to declared scheduling and motion to observed timer events.
    [guess] Refresh on revision and each event, show two mounted cams before a schedule count, and fog phase when timing evidence is absent.
    [sourced] Acquit has two 30-second API timers, while `sweep` is manual. `/home/factory-user/acan/explore/map.md:187`.
    [inferred] A configured interval proves intention, not an on-time heartbeat.

[inferred] Give the mounted gauge one named metric, unit, window, and scale, with its threshold notch from a designed budget.
[guess] Refresh gauges each second, replace missing readings with an empty bezel, and use an overflow numeral rather than silently clipping.
[inferred] Keep K13 calibration in a separate Inspect comparison with backtest period, sample count, and error definition.
[inferred] No backtest means unknown calibration, not a reassuring confidence percentage.

## 3. Rules that stop the plant lying

### Every semantic mark has an inspectable binding

[inferred] Store each binding with entity and lane identity, channel ID, semantic meaning, source class, source locator, revision, timestamp, unit, aggregation, transform, limits, and freshness policy.
[inferred] Include telemetry coverage, sample policy, and collector health when a binding uses observed events.
[inferred] Preserve designed, declared, and observed values separately rather than coalescing them into an anonymous value.
[inferred] Feed the gauge, geometry, label, and export from the same evaluated binding result.
[sourced] This extends the anatomy rule that every displayed number comes from the model that moves its parts. `/home/factory-user/acan/explore/map.md:14`.

[inferred] Alt+hover should identify the selected part and show its full derivation.
[guess] A hypothetical verifier explanation reads `one cell; declared concurrency 1; deploy/runtime.env:12; fixture revision 55825fe; runtime concurrency not reported`.
[inferred] Add a keyboard-accessible Inspect equivalent and links to the actual blueprint field, source line, or telemetry query.
[inferred] Cosmetic screws and seams still need a catalog-definition binding labeled non-semantic, so their count cannot masquerade as instance data.

### Missing evidence and drift remain visible

[inferred] Model unknown, stale, invalid, known zero, and valid positive values as distinct states.
[guess] Mark one-second runtime bindings stale after five seconds without their expected heartbeat, with per-source overrides for slower collectors.
[inferred] Keep the last reading with its age in Inspect, but stop live motion and withdraw current-value claims.
[inferred] Loss of telemetry is not proof that a process stopped.
[inferred] Working requires observed active work, while idle requires verified liveness and a trustworthy zero-active reading.
[inferred] Strained requires a named budget violation, and failing requires an observed failure rule with an inspectable evaluation window.
[inferred] A designed-only ghost has a dashed shell and empty bezel, without live occupancy or traffic.
[inferred] The dark state requires lifecycle evidence or an explicit operator declaration, while fog denotes missing knowledge.
[inferred] Failure and fog may coexist, so provenance quality must not overwrite operational state.

[inferred] Compare only compatible facts from the same entity, lane, time context, unit, and definition.
[inferred] Two busy workers against a declared single-slot limit is drift only if the observed workers belong to that pool and the observations are comparable.
[inferred] In Live, show observed state over the declared structure with a drift bracket and inspectable alternatives.
[inferred] In Blueprint, emphasize designed structure without silently changing the declared configuration.
[inferred] In Replay, use historical source revisions and event time.
[inferred] In Simulate, identify generated activity, model version, and calibration separately from live traffic.
[sourced] K4 explicitly requires disagreement to remain visible as drift. `/home/factory-user/acan/explore/map.md:29`.
[inferred] Never auto-relayout existing machines to resolve drift, and send newly discovered entities to K5's unplaced tray.

### Test the contract and the picture

[inferred] Property-based tests should generate machine specs across missing values, zero, overflow, restart, sampling gaps, revision changes, and conflicting sources.
[inferred] Assert literal expected behaviors at boundaries, including eight versus nine slots and seven versus eight waiting jobs.
[inferred] Verify monotonic fill under a fixed scale, stable role silhouettes, identical gauge and label values, and no motion without qualifying evidence.
[inferred] Test that a missing field never activates a demo default.
[inferred] Test input permutations so source arrival order cannot change a resolved shape.

[inferred] Semantic scene-node assertions alone are insufficient because the pixels can still obscure or contradict the binding.
[inferred] Render deterministic-clock screenshots for every catalog state, flat-plan projection, overflow label, and fog combination.
[inferred] Check dimensions and pixel regions against independent expected values, then run recognition tasks with developers.
[sourced] Existing layout rules place readings left of gauges, lift failure readings above smoke, and keep title blocks and connectors away from labels. `/home/factory-user/acan/explore/map.md:137`.
[inferred] Include those rules in screenshot checks.
[inferred] Replay the corrected Acquit fixture to verify one verifier slot, two mounted timers, separate queues, lane identities, and sealed Docker internals.
[inferred] Redaction tests must cover provenance popovers and exported metadata as well as visible labels.

## 4. What to avoid

[inferred] Avoid comprehensive clutter at the overview scale.
[sourced] Chernoff interference research shows that distinctive irrelevant details can disrupt judgments. [Naveh-Benjamin and Pachella](https://doi.org/10.1177/001872088202400102).
[inferred] Prefer a role shell, a primary gauge, and a few causal tells, with detailed anatomy on demand.
[guess] The proposed count limits need developer testing and are not perceptual constants established by the cited studies.

[inferred] Avoid decorative traffic and perpetual healthy-state motion.
[sourced] Q15 caps belts at 20 fps and requires stillness after two minutes without input or state change. `/home/factory-user/acan/explore/map.md:229`.
[inferred] Show a motion-paused indicator and current static readings when motion is disabled, so stationary cargo does not imply zero traffic.
[inferred] Never use sine-wave gauges, random smoke, or warm-up animation as live measurements.

[inferred] Avoid treating software as conserved physical material.
[inferred] Requests fan out, retries duplicate work, caches suppress downstream calls, and callbacks cross time boundaries.
[sourced] Acquit specifically needs linked spans across delayed queue and callback work. `/home/factory-user/acan/explore/map.md:176`.
[inferred] Cargo is a disclosed event representation, not proof that every upstream token must emerge downstream.

[inferred] Avoid false precision from opaque components.
[sourced] Acquit's Docker subjects are timed from the host and drawn as sealed housings. `/home/factory-user/acan/explore/map.md:182`.
[inferred] Host elapsed time cannot justify invented internal stages, token counts, or model progress.
[inferred] Likewise, absent traces cannot justify an empty queue, and five decorative hopper rings cannot justify five partitions.

[inferred] Avoid confusing detail with comprehension.
[sourced] EvoStreets comparisons do not establish a general performance advantage for richer views. [ICPC experiment](https://ieeexplore.ieee.org/document/8813284/).
[inferred] Test whether developers can find a bottleneck, distinguish a missing sensor from a stopped service, and explain a shape's evidence.
[inferred] Keep a binding only when that task benefit outweighs clutter and maintenance.

## Recommended v0.1 bindings

| Binding | Source | Update | Missing evidence | Bounded drawing | Acquit application |
|---|---|---|---|---|---|
| [guess] Catalog shell and runtime plate | Designed role, declared code and command, observed version separately | Revision or restart | Crate for unknown role, fogged plate fields | One shell, two plate lines | API process, House internal, Caddy static web |
| [guess] Worker cells and occupancy | Declared concurrency, observed pool gauge | Revision and 1 second | Unknown cell count or occupancy is fogged | Eight cells, then exact count | One declared verifier slot, never four by default |
| [guess] Queue tray and depth | Observed queue gauge, declared capacity or designed warning | 1 second | Fogged tray, never empty by assumption | Seven tokens, then number and bounded fill | Separate effects and volatile verifier queues |
| [guess] Ports and lane copies | Declared interfaces and lanes, observed discovery | Revision and 5 seconds | Unverified openings, unknown membership fog | Six groups, three copy outlines, then counts | Role plus lane, callback distinct from HTTP listener |
| [guess] Gauge and abnormal state | Observed named metric, designed budget | 1 second | Empty bezel, aged last value in Inspect | One gauge, exact overflow number | Verifier waiting age or measured service time |
| [guess] Belts and return loops | Declared edge and retry policy, observed sampled events | Events and 1-second activity rollup | No cargo motion, fogged unknown activity | Twelve cargo groups, one loop, then counts | Queue handoff, callback, distinct PayPal retry policies |
| [guess] Mounted cams | Declared schedule, observed events | Revision and event | Period may remain declared, phase fogged | Two cams, then schedule count | Two 30-second API timers, no sweep clock |
| [inferred] Defer richer quantitative geometry | Schema layers, throughput widths, and per-call fuse need dedicated bindings | Implement after source coverage is verified | Retain fogged anatomy and existing belt budget treatment | No fabricated internals | Keep SQLite and Docker truthful before adding detail |
