# Many plants on one canvas

## Summary

[inferred] Put the developer's active plants in one persistent world, but preserve direct jumps that cost no more gestures than switching tabs.
[inferred] Stable places support recognition. [inferred] Semantic zoom must change the information shown, not shrink twenty unreadable plants.
[inferred] Keep one focused plant, one inspector, and one global "Waiting on you" queue.
[inferred] Review capacity belongs to the person across all plants. [sourced] The inspection gate remains advisory under `/home/factory-user/acan/explore/research/agentic-ux/decisions.tsv:15`.
[inferred] Show actual cross-project contracts, not decorative connections between nearby repositories.
[guess] Target 5 to 20 plants through summaries and bounded detail, not twenty copies of the current renderer and telemetry allowance.

## 1. Research and evidence

### Grounding and limits

[sourced] Research uses the assignment's date of 2026-10-17, also recorded in `/home/factory-user/acan/explore/research/agentic-ux/00-synthesis.md:3`.
[sourced] K1 to K20 establish architect-owned layouts, designed, declared, and observed evidence, protected checks, repo-local YAML, and risk-ranked choices in `/home/factory-user/acan/explore/map.md:26-45`.
[sourced] A1, A4, and A10 specify the Rust renderer, orthographic views, WebGPU and WebGL2 tiers, and an 8 ms p99 frame at 2,000 machines in `/home/factory-user/acan/explore/map.md:51-60`.
[sourced] U2, Q15, L7, and L8 require second-monitor use, capped motion, idle control, and recovery from device loss in `/home/factory-user/acan/explore/map.md:115,164-165,229`.

[sourced] The supplied screenshots show one project selector, one mode strip, and a small decision queue, with a right inspector and bottom Floor in the expanded view at `/home/factory-user/acan/explore/ui-v1/shots/l-default.png` and `/home/factory-user/acan/explore/ui-v1/shots/p5-floor.png`.
[inferred] Those regions can become shared chrome rather than repeat around every plant.
[sourced] The synthesis converges on limited human review capacity, one decision queue, quiet peripheral awareness, and evidence-bound completion in `/home/factory-user/acan/explore/research/agentic-ux/00-synthesis.md`, "Convergences".

[sourced] External research uses WebSearch full-text results, not FetchUrl or installed product tests, with calls logged under `/home/factory-user/.factory/artifacts/scripts/62de3077-ca4a-4838-8efd-41285e169859/`.
[sourced] The search logs under `/home/factory-user/.factory/artifacts/scripts/62de3077-ca4a-4838-8efd-41285e169859/` contain no rate-limit errors, so no retries or pauses were needed.
[inferred] Vendor documentation establishes available interactions, not improved developer performance.
[sourced] Raskin coverage uses a [secondary summary](https://en.wikipedia.org/wiki/The_Humane_Interface), and Cockburn and McKenzie coverage uses their [abstract](https://dl.acm.org/doi/10.1145/503376.503413).
[sourced] The [Dwarf Fortress article](https://dwarffortresswiki.org/index.php/Civilization_and_World_Info) warns that its migrated content may be outdated.
[inferred] No reviewed study establishes that twenty software projects on one canvas outperform tabs.

### ZUIs promise continuity, not free navigation

[sourced] Perlin and Fox's 1993 Pad places objects on a shared two-dimensional plane and uses portals to view distant regions, as described in this [institutional paper analysis](http://www.cs.umd.edu/~bederson/classes/cmsc838b/papers/pad.html).
[sourced] Bederson and Hollan's 1994 [Pad++ paper](http://www.cs.columbia.edu/graphics/courses/csw4170/resources/bedersonHollanUIST94.pdf) distinguishes semantic zoom from geometric scaling, so an object can change representation with its displayed size.
[sourced] The [Pad++ paper](http://www.cs.columbia.edu/graphics/courses/csw4170/resources/bedersonHollanUIST94.pdf) also describes spatial indexing, clipping, level of detail, lazy loading, and eviction of unseen objects.
[inferred] Acan should inherit meaningful representations and bounded work, not the promise that infinite space removes organization.

[sourced] Raskin's ZoomWorld argument replaces document hierarchies with navigation over a common plane, where zooming opens detail, according to this [summary of The Humane Interface](https://en.wikipedia.org/wiki/The_Humane_Interface).
[inferred] The useful argument is continuity between overview and work, rather than a requirement to replace every list, search box, and application boundary.
[sourced] Bederson's later [reflection on ZUIs](https://scispace.com/pdf/the-promise-of-zoomable-user-interfaces-2tl9dx0mvx.pdf) says the grand zoomable-desktop vision did not broadly materialize and notes limited examples of ordinary daily tasks in early demonstrations.
[inferred] A convincing fly-through is therefore insufficient evidence for removing fast nonspatial navigation.

### Existing canvases retain structure and direct access

[sourced] [tldraw pages](https://tldraw.dev/sdk-features/pages) are independent scene roots with per-page cameras and selection state, while its [camera API](https://tldraw.dev/sdk-features/camera) supports bounded navigation and zoom fitting.
[inferred] A single document can preserve several places without rendering or presenting them all at full detail.
[sourced] [Figma's navigation panel](https://help.figma.com/hc/en-us/articles/360039831974-View-layers-and-pages-in-the-left-sidebar) retains separate page canvases and a nested layer list for frames, groups, and objects.
[inferred] Frames organize a canvas, but do not make named navigation obsolete.

[sourced] [Miro frames](https://help.miro.com/hc/en-us/articles/360018261813-Frames) structure and navigate a board, while [Miro search](https://help.miro.com/hc/en-us/articles/360012140959-How-to-search-in-Miro) finds boards and Spaces and can open matching content at its location.
[sourced] Muse, now [Allume](https://allume.com/), uses nested boards, search, and quick-jump, and explicitly rejects endless canvas zooming.
[inferred] Nested boards provide hierarchy and focus rather than literal simultaneous visibility of everything.

[sourced] [Heptabase's shortcuts](https://wiki.heptabase.com/keyboard-shortcuts) expose a whiteboard map, global search, within-whiteboard search, history, tabs, and side panels.
[sourced] [Kinopio's minimap and table of contents](https://kinopio.club/help/posts/minimap/) support click-to-jump navigation and a persistent minimap toggle.
[sourced] [Kinopio](https://kinopio.club/) also supports links between Spaces.
[sourced] [Obsidian Canvas](https://help.obsidian.md/plugins/canvas) stores separate `.canvas` files, groups related cards, and offers "Go to source" and "Go to target" on distant connections.
[sourced] Obsidian's separate [Bookmarks plugin](https://obsidian.md/help/plugins/bookmarks) provides named, grouped shortcuts to frequently used items.

[inferred] Search answers "where is this named thing", a minimap answers "where am I", and bookmarks answer "take me back there".
[inferred] Acan needs all three functions, although they need not occupy three permanent panels.
[inferred] None of this evidence justifies assigning a minimap, bookmark system, or universal content search to every listed product.

### Spatial memory helps when places remain learnable

[sourced] Robertson and colleagues' [Data Mountain study](https://www.microsoft.com/en-us/research/publication/data-mountain-using-spatial-memory-for-document-management/) found reliable advantages over Internet Explorer Favorites for its document-management task using freely arranged thumbnails on an inclined plane.
[inferred] That result supports learnable locations and recognizable landmarks, not arbitrary three-dimensional navigation for software systems.
[sourced] [Cockburn and McKenzie](https://dl.acm.org/doi/10.1145/503376.503413) found deteriorating retrieval performance as freedom to place items in the third dimension increased, with greater perceived clutter.
[sourced] [Scarr and colleagues](https://inria.hal.science/hal-02862475/document) found faster selection after view transformations when layouts preserved spatial consistency rather than reflowing.
[inferred] Keep plant positions stable across days and keep the working view flat, even if machines use three-dimensional geometry.

[sourced] Jul and Furnas define [desert fog](https://www.researchgate.net/publication/2806120_Critical_Zones_in_Desert_Fog_Aids_to_Multiscale_Navigation) as a view with no information for navigation and report problems for novice and experienced Pad++ users.
[inferred] Empty travel, scale changes, and missing landmarks add navigation work that a direct tab selection avoids.
[inferred] Removing tabs removes one window-management action, not the mental cost of changing tasks.
[guess] Keyboard jumps with restored views may beat tabs for some users, while mandatory pan-and-zoom travel may be slower.

### Games separate strategic awareness from local work

[sourced] [Factorio's remote-view account](https://factorio.com/blog/post/fff-380) describes configuring entities remotely and identifies the cumbersome planet-dialog, platform-dialog switching sequence as a problem.
[sourced] Factorio adds [map search, pins, and planet-grouped alerts](https://www.factorio.com/blog/post/fff-400), plus [back and forward history](https://www.factorio.com/blog/post/fff-397).
[inferred] Its strongest precedent is direct remote access with return paths, not rendering every planet as one continuous physical factory.

[sourced] [Stellaris](https://stellaris.paradoxwikis.com/Main_interface) distinguishes galaxy and system views, supplies an outliner, and offers ownership, diplomacy, intelligence, and opinion map modes.
[sourced] [Civilization VI lenses](https://civilization.fandom.com/wiki/Lens_(Civ6)) overlay selected strategic information on the same world.
[inferred] Acan can borrow overview versus inspection and task-specific overlays without changing the identity or position of plants.

[sourced] [Dwarf Fortress's world information](https://dwarffortresswiki.org/index.php/Civilization_and_World_Info) distinguishes sites, holdings, missions, and news beyond the local fortress.
[sourced] Original [Frostpunk scouting](https://www.neoseeker.com/frostpunk/Scout_Locations) opens exploration after constructing a Beacon, with discovered locations revealing further destinations.
[sourced] [Frostpunk 2's Frostland](https://frostpunk-2.game-vault.net/wiki/Frostland) uses a regional map with territories, outposts, colonies, and transport connections.
[sourced] [Anno's multisession design](https://www.anno-union.com/multisession-gameplay-in-anno-1800/) connects separate worlds through goods transfer and explicitly considers players who prefer a home session.
[sourced] [RimWorld's colony documentation](https://rimworldwiki.com/wiki/Colony) describes multiple simultaneous colonies and warns that several can suffer raids at once.
[inferred] The common lesson is stable sites, explicit links, local detail, and global demands, not a mandate to keep all sites equally busy or equally visible.

### Switching research and IDEs preserve useful boundaries

[sourced] [Czerwinski and colleagues' large-display study](https://www.microsoft.com/en-us/research/publication/toward-characterizing-the-productivity-benefits-of-very-large-displays/) reports benefits and preference for a prototype large display during complex office work.
[sourced] Their [task-switching diary study](https://www.microsoft.com/en-us/research/publication/a-diary-study-of-task-switching-and-interruptions/) describes difficulty interleaving work and recovering complex projects.
[inferred] More persistent visual context can help recovery, but neither study proves that more simultaneous detail improves attention.

[sourced] `/home/factory-user/acan/explore/research/agentic-ux/e-attention.md:44-69` separates interruption stress, delayed return, memory cues, ready-to-resume plans, and breakpoint-based delivery.
[sourced] That review reports greater stress under interruption in Mark's experiment and useful departure and retrieval cues in Altmann and Trafton's model at `/home/factory-user/acan/explore/research/agentic-ux/e-attention.md:46-60`.
[sourced] It also reports reduced attention residue with Leroy and Glomb's ready-to-resume plan and reduced frustration with Iqbal and Bailey's deferred notifications at `/home/factory-user/acan/explore/research/agentic-ux/e-attention.md:64-69`.
[inferred] A plant jump should restore the unfinished question and next action, not merely its camera.
[inferred] There is no defensible universal switching penalty to subtract from Acan's claimed savings.

[sourced] [VS Code multi-root workspaces](https://code.visualstudio.com/docs/editing/workspaces/multi-root-workspaces) combine related folders while preserving folder identity, settings scope, and disambiguated names.
[sourced] Its [container documentation](https://code.visualstudio.com/remote/advancedcontainers/connect-multiple-containers) permits one connected container per window, with multiple windows or explicit switching for several containers.
[sourced] [JetBrains' Go guide](https://www.jetbrains.com/guide/go/tips/better-view-of-attached-projects/) describes attached projects and keyboard navigation, without establishing identical behavior across every JetBrains IDE.
[inferred] Shared presentation must not imply shared execution authority, environment, or repository state.

## 2. Proposed world design

### One personal world with stable plant sites

[inferred] Display every explicitly added active plant in one personal world, with a test target of 5 to 20 rather than an arbitrary five-project product limit.
[inferred] A plant represents a software system, not necessarily one repository, so a monorepo can contain several plants and one plant can span repositories.
[inferred] Each plant retains its own model and local coordinates, while the world stores its placement and camera bookmarks in local workspace state.
[inferred] A plant's worktree and environment are explicit identity fields, so switching branches does not silently redirect another plant's evidence or edits.
[inferred] World placement must not rewrite another project's `.acan/` files.

[inferred] Acan suggests initial nonoverlapping sites, but the architect accepts placement and controls subsequent moves, extending K5.
[inferred] New plants arrive in the unplaced tray, with an explicit "place suggested site" action rather than automatic rearrangement.
[inferred] Reserve expansion space around each site, and ask before growth overlaps a neighbor.
[inferred] Preserve positions across launches, branch changes, agent activity, and sorting of the decision queue.
[inferred] An archived plant keeps its saved site and remains searchable without occupying live detail.
[inferred] Physical distance expresses the user's grouping only, never latency, trust, or dependency strength.

### Four semantic levels with persistent identity

[inferred] World level shows each plant's name, recognizable footprint, environment, freshness, unknown-observation mark, decision count, and most consequential current condition.
[inferred] Show shared contracts as quiet routes between boundary ports, but hide machine labels, bot avatars, packet motion, and internal belt detail.
[inferred] A small plant must retain a readable name and status symbol instead of shrinking into an indistinguishable dot.

[inferred] Plant level shows catalog silhouettes, internal topology, territories, blueprint ghosts, budget breaches, and separate designed, declared, and observed marks.
[inferred] Show only a few important machine labels by default, with all labels available through an explicit reveal.
[inferred] The plant's title and boundary remain recognizable when neighboring plants enter the viewport.

[inferred] Machine level exposes the selected machine's ports, fittings, named metrics, evidence freshness, checks, affected work orders, and connected flows.
[inferred] Only the focused plant accepts edits, while neighboring plants retain simpler representations.
[inferred] A machine selection can open the shared inspector without requiring microscopic text on the canvas.

[inferred] Cutaway level opens one selected machine's anatomy, internal stages, bound evidence, and calibration history where applicable.
[inferred] Cutaways require an explicit action, remain read-only until an appropriate edit command, and close back to the same machine.
[guess] Initial transition thresholds should use projected screen size with hysteresis, so minor wheel movements do not flicker between representations.
[inferred] Preserve the plant outline and selected object's position through transitions.

### Cross-project belts represent contracts

[inferred] Give every cross-plant endpoint a qualified plant, entity, port, environment, and revision identity.
[inferred] Distinguish runtime requests, package dependencies, and artifact delivery with line pattern and text, rather than implying that every connection carries measured traffic.
[inferred] A library repository feeding another plant shows package version, consumer revision, compatibility check, and unresolved drift.
[inferred] Runtime links show observation age and environment, with unknown rate left unknown.

[inferred] Represent a shared service once in its owning plant or an explicitly created shared-services plant.
[inferred] Consumers receive labeled references, not editable duplicate machines.
[inferred] Bundle multiple contracts between the same plants at world scale and reveal individual routes on selection.
[inferred] Inside a plant, a boundary connector names the distant endpoint and supports "jump to other end" with Back.
[inferred] Drawing a proposed connection creates blueprint intent, not a deployed integration or permission to edit both repositories.
[inferred] Cross-plant changes require explicit scope and approval for each affected owner, with protected budgets and checks unchanged under K16.

### One attention queue and one human review budget

[inferred] Retain one fixed "Waiting on you" instrument across the world, including demands from offscreen plants.
[inferred] Each decision names its plant, why the human is needed, consequences of waiting, latest safe response, evidence, and estimated review effort.
[inferred] Order choice review using K17, while showing urgency and harm separately rather than hiding them inside confidence.
[inferred] Confidence without an evidence basis remains unknown.

[inferred] Group reports only when they require the same human decision, preserve affected plants underneath, and label uncertain correlations.
[inferred] Never merge distinct security or data-loss concerns because their timestamps coincide.
[inferred] Keep aging and deferred decisions visible so a noisy plant cannot indefinitely displace quiet work.
[guess] Add per-plant defer controls and a review checkpoint that flags every overdue decision, rather than using a fixed quota that could conceal urgent harm.
[inferred] Routine progress does not enter this queue, animate the world, steal focus, or move the camera.

[sourced] The user's gate decision is advisory in `/home/factory-user/acan/explore/research/agentic-ux/decisions.tsv:15`.
[inferred] Therefore show one personal review allowance across plants and recommend holding new high-review work when demand exceeds it.
[inferred] Agents may voluntarily park at declared safe points, but the UI must distinguish recommended holds from enforced authority limits.
[inferred] Do not claim that the advisory gate prevents independent agents from starting work.
[inferred] Overrides expose extra review demand and never weaken checks or grant protected authority.
[guess] Start review estimates as editable ranges, then compare estimates with actual review time rather than presenting precise invented minutes.

### Shared chrome, explicitly scoped modes

[inferred] Keep one inspector on the right and one collapsible Floor below, following the screenshot regions.
[inferred] The inspector always names plant, environment, revision, and whether the object is editable.
[inferred] The Floor defaults to the focused plant, with an explicit "All plants" filter for cross-plant work and clashes.
[inferred] Every row in the all-plants Floor includes the plant name.

[inferred] Keep Live, Blueprint, Simulate, and Replay in one mode strip scoped to the focused plant.
[inferred] World overview stays a live summary, with per-plant mode chips identifying any retained local analysis state.
[inferred] The focused plant changes paper within its boundary, preserving K15 without recoloring the whole world.
[inferred] A jump restores local inspection state but does not restore an armed edit gesture.
[inferred] Replay timestamps and simulation scenarios never silently apply to neighboring plants or combine into a supposedly synchronized world.

### Navigation without compulsory travel

[inferred] Ctrl K becomes global jump and command search across plants, entities, work orders, bookmarks, and decisions.
[inferred] Results always include plant and environment, with an explicit current-plant filter and a visible target for commands that write.
[inferred] Empty-query results offer recent places, pinned places, and waiting decisions.
[inferred] Enter restores the target's saved view, while Escape dismisses search without moving.
[inferred] Restored views include the departure note, unfinished review, and next action alongside the camera.

[guess] Use Ctrl Tab for recent-plant cycling and Ctrl Shift Tab for reverse cycling, with the candidate visible before release.
[guess] Offer Alt PageDown and Alt PageUp for stable world-order cycling, plus remappable Alt 1 through Alt 9 bookmarks.
[inferred] Disable canvas shortcuts while typing in editors.
[inferred] Home fits the active world, F fits the focused plant, and back and forward restore camera, selection, mode, and Floor filter.
[inferred] Distant jumps need no mandatory flight animation, and reduced-motion users receive immediate relocation.
[inferred] The minimap shows labeled plant sites, viewport bounds, and decision markers without moving or resizing sites by urgency.

[inferred] Replace the project selector with a world breadcrumb and focused plant name, not another row of hidden tabs.
[inferred] At world scope the title block names the world and reports the number of stale plants, with no fictional global git revision.
[inferred] At plant scope it shows that plant's model revision, repository or repository set, worktree, branch, dirty state, and environment.
[inferred] Multi-repository plants expand to per-repository revisions instead of claiming one SHA covers everything.

### Performance is one aggregate budget

[sourced] A10 requires 8 ms p99 at 2,000 machines, telemetry-to-pixel below 100 ms, 100,000 spans per second below one core, cached open below one second, and memory below 500 MB with one hour of telemetry at `/home/factory-user/acan/explore/map.md:60`.
[inferred] Do not multiply those allowances by plant count.
[inferred] Test 5 plants with 400 machines, 10 with 200, and 20 with 100 as equal-total fixtures.
[inferred] Add a stress fixture of 20 plants with 2,000 machines each to verify virtualization, without claiming 40,000 full-detail machines satisfy A10.

[inferred] Render world summaries for all sites, cull offscreen geometry, and cap detailed visible machines at 2,000 across the viewport.
[inferred] Use projected-size level of detail when several plants are visible, and keep only one cutaway.
[inferred] Keep ingestion and bounded history in the Rust core, with compact deltas to one renderer rather than one webview or frame loop per plant.
[inferred] Cache summaries and navigation metadata while evicting unused geometry, textures, and high-detail working state.
[inferred] Check cached-plant entry against A10's one-second limit, and disclose loading or stale detail rather than displaying an old scene as current.
[guess] Begin frame allocation with 2 ms for CPU preparation, 4 ms for GPU work including text, and 2 ms contingency, then measure end-to-end p99 rather than adding component percentiles.

[guess] A provisional 480 MB envelope reserves 100 MB for the shell and core, 200 MB for aggregate history, 100 MB for rendering including wasm and attributable GPU resources, 40 MB for summaries, and 40 MB for scratch space.
[inferred] These are proposed caps, not measured feasible allocations, and machine counts alone do not bound geometry or telemetry cardinality.
[inferred] Retain bounded one-hour aggregate rings and separately capped span samples, with cardinality limits shared across plants.
[inferred] Preserve failure records and expose retention loss if admission or sampling policies reduce telemetry detail.

[sourced] Q15 caps flowing belts at 20 fps and stops motion after two minutes without input or state change in `/home/factory-user/acan/explore/map.md:229`.
[inferred] Offscreen plants need no animation, while visible summary changes can invalidate a frame without starting continuous rendering.
[guess] Coalesce routine world-summary changes for at most 50 ms, with consequential changes bypassing that window.
[inferred] Apply A10's under-100-ms telemetry-to-pixel target to both visible summaries and detailed views, with ingestion, batching, and rendering included.
[guess] Start true-idle acceptance below 1% of one logical core over ten minutes, with no recurring GPU submissions after settling.
[inferred] Sustained telemetry is not true idle and needs its own power test.
[inferred] Run 24-hour flat-memory, burst-ingestion, sleep-wake, monitor-hotplug, and device-loss tests on both renderer tiers before claiming twenty-plant support.

## 3. What to avoid

[inferred] Avoid automatic relayout by activity, risk, or agent count, because it destroys the stable locations supported by [Scarr's study](https://inria.hal.science/hal-02862475/document) and contradicts K5.
[inferred] Avoid orbital navigation and compulsory three-dimensional travel, given [Cockburn and McKenzie's retrieval findings](https://dl.acm.org/doi/10.1145/503376.503413).
[inferred] Avoid empty-space wandering without Home, history, named search, and a minimap, given [desert fog](https://www.researchgate.net/publication/2806120_Critical_Zones_in_Desert_Fog_Aids_to_Multiscale_Navigation).

[inferred] Avoid accidental cross-plant edits by making world view read-only, requiring explicit plant focus, and clearing drag, placement, and destructive command state on every jump.
[inferred] A drag crossing a boundary must cancel rather than migrate an entity, and cross-plant selection must not enable bulk writes.
[inferred] Bind each write to its named plant and expected revision so a late inspector response cannot edit the newly focused plant.
[inferred] These safeguards preserve the project boundaries retained by [multi-root IDEs](https://code.visualstudio.com/docs/editing/workspaces/multi-root-workspaces).

[inferred] Avoid camera theft, flashing fleets, and agent-activity rankings, because attention is already the shared limit in `/home/factory-user/acan/explore/research/agentic-ux/00-synthesis.md`, "Convergences".
[inferred] Avoid hiding quiet decisions under one noisy plant by retaining aging, defer-until, and latest-safe-response fields in the global queue.
[inferred] Avoid confusing the advisory inspection gate with an execution lock, given `/home/factory-user/acan/explore/research/agentic-ux/decisions.tsv:15`.
[inferred] Avoid twenty independent histories and render loops masquerading as one canvas, because A10 and Q15 remain aggregate constraints.

[guess] Validate the proposal against tabs and against a named plant list using the same interruptions, return tasks, and planted cross-project mistake.
[inferred] Measure time to the correct object, wrong-plant actions, review omissions, recovery after a day away, fatigue, frame latency, and idle power before removing existing navigation.

## Recommended v0.1 scope

- [inferred] Demonstrate five persistent plant sites and test twenty summary sites, with one focused editable plant.
- [inferred] Ship world and plant representations, existing machine inspection, one explicit cutaway, global Ctrl K, history, and bookmarks.
- [inferred] Keep one global decision queue, an advisory shared review allowance, and scoped inspector, Floor, modes, and title block.
- [inferred] Support declared cross-plant references and endpoint jumps, but defer automatic contract discovery and cross-repository editing.
- [inferred] Require equal-total A10 fixtures, idle and device-loss tests, and a navigation comparison before promising twenty fully live plants.
