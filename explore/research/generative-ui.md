# Generative UI with a fast System 1 model

Research note for Acan. Read with `acan/explore/map.md`. This note proposes amendments but does not change the map.

Every claim carries one label:
- **[sourced]** means a page I opened says it. The link is next to it or in the source list.
- **[inferred]** means my reasoning from sourced facts and the map.
- **[guess]** means a number or judgment I could not check. It needs a spike or a test.

"System 1" here always means the fast model. In the map, `S1` is a Stage 3 decision about visual direction. They are unrelated.

## Recommendation

**Bound, not generated.** Acan adopts the System 1 *contract*, not a System 1 vendor. The Rust core builds every possible card, callout and what-if from the twin, with every number held as a data binding. A fast System 1 only *chooses* among those candidates and puts the winners into fixed slots. It never writes a number, a label or a layout. A slower System 2 (the user's own agent over MCP, or an opt-in remote model) can propose new candidate recipes and short notes in the background. It never writes numbers either. System 1 ships first as a deterministic rule scorer that runs in microseconds and works offline. A small local model replaces it only where a replay test shows it picks better. TypeSafe's Jev is an opt-in remote tier that shows on the egress counter. Chrome, layout, budgets, checks, choice order and export redaction are never generated.

Why this shape, in four lines:
1. Senior engineers need a screen that does not move under them. Adaptive menus lose to static ones on speed, and users prefer control they can adjust. [sourced, Findlater and McGrenere 2004; Gajos et al. 2008]
2. A model that cannot leave its schema can still pick the wrong value. Choosing from true candidates bounds the damage to "wrong emphasis," never "wrong number." [sourced, TrueFoundry on Jev; json-render Jev docs] [inferred for the Acan consequence]
3. The fastest real "System 1" product is hosted only, which conflicts with local-first and the zero-egress promise (Q16). [sourced, TypeSafe models page; TrueFoundry] [inferred for the conflict]
4. Full generation of HTML or of new layouts takes seconds to minutes and still makes errors. [sourced, Google Research Generative UI; OUI-1]

## What the field looks like in late 2026

### Three families of generative UI

| Family | How it works | Speed | Fit for Acan |
|---|---|---|---|
| Model writes code | The model writes HTML, CSS and JS per prompt. Google's Generative UI uses Gemini 3 Pro this way. | "A minute or more" in some cases, with "occasional inaccuracies." [sourced, Google Research] | No. Too slow, unbounded, and it would put untrusted script next to live telemetry. [inferred] |
| Model fills a catalog | The model emits a declarative spec using only components the app trusts. A2UI uses a flat list of components with IDs and binds data with JSON Pointer paths (RFC 6901). json-render does the same with zod-typed catalogs, `$state` bindings and streaming. [sourced, a2ui.org; A2UI data binding; json-render repo] | Bounded by text generation speed. OUI-1, a model trained for this, takes about 1.9 s per output on an A100. [sourced, OUI-1 blog] | The right *format*. Too slow and too free if the model writes props. [inferred] |
| Model selects from candidates | The app builds atomic candidates with concrete props. The model decides which to include, their order and their placement. json-render's Jev mode works this way and states that "Jev cannot invent missing prose or data." Raw props and state are not sent to the evaluator. [sourced, json-render Jev docs] | 70 to 500 ms end to end on Jev. [sourced, TypeSafe launch post] | The right *behavior*. [inferred] |

MCP Apps is a fourth path. An MCP server ships an HTML resource (`ui://`) that the host shows in a sandboxed iframe and talks to over postMessage JSON-RPC. Claude, ChatGPT, VS Code and Goose host it. [sourced, MCP Apps overview; MCP Apps blog post] Acan should not host agent-supplied HTML in v0.1. A sandboxed iframe from an arbitrary agent inside a closed, trust-sensitive app is a new attack surface and breaks the visual identity. [inferred]

### "System One" as a product

TypeSafe AI launched "System One models" and Jev on 2026-09-15. [sourced, TypeSafe launch post] The relevant facts:
- Jev returns typed decisions (Choice, Score, a null-able "Noul" and Box types) with probabilities. It does not generate text. [sourced, TypeSafe docs, System One concept page]
- Latency is 70 to 500 ms end to end. Input costs $0.042 per million tokens and output is free. A Choice can have up to 255 options, and one call can ask several questions in parallel. [sourced, TypeSafe launch post]
- Choice confidence is normalized as (pmax − 1/n) / (1 − 1/n). TypeSafe suggests three bands (act, confirm, defer) with thresholds that rise with the cost of a mistake. [sourced, TypeSafe confidence docs]
- Speculative fan-out asks many cheap questions in advance and keeps the answer that turns out to be needed. [sourced, TypeSafe fan-out pattern]
- Known weak spots of jev-1.13 are math, counting, dates, indirection, large state, option-order bias and prompt injection. It cannot generate. [sourced, TypeSafe jaggedness page for jev-1.13]
- jev-1.14 shipped 2026-10-14 with a 64k context. It runs on a cloud endpoint. Zero data retention is enterprise only. [sourced, TypeSafe models page; TypeSafe enterprise page]
- It is hosted only, behind a waitlist, with no weights and no on-premises option. "Can't hallucinate" means it can't leave the schema. It can still pick a wrong value. [sourced, TrueFoundry review]

So the *pattern* fits Acan exactly, and the *product* cannot be the default. Every Jev call is an outbound connection that Q16 must log, and the weak spots (math, large state, injection) are exactly what a telemetry twin is full of. [inferred]

### Fast and slow, split by design

Figure's Helix robot splits control into a 7B "System 2" model at 7 to 9 Hz and an 80M "System 1" policy at 200 Hz. System 2 writes a shared latent asynchronously, and System 1 always acts on the latest one. [sourced, Figure Helix] For Acan the analog is a slow planner that prepares candidate recipes and notes off the hot path, and a fast selector that reads whatever is ready when the user acts. [inferred]

### Latency and predictability

- 0.1 s feels instant and is the limit for direct manipulation. 1 s keeps flow. 10 s is the limit of attention. [sourced, Nielsen]
- Zed's edit prediction targets p50 under 200 ms and p90 under 500 ms. It shows the prediction as a preview behind a modifier key, so it never takes over the Tab key. [sourced, Zed blog]
- In adaptive interfaces, both accuracy and predictability raise satisfaction. Accuracy mattered more. [sourced, Gajos et al. 2008]
- A static menu was faster than an adaptive one, and users preferred an adaptable menu they control. [sourced, Findlater and McGrenere 2004]
- Microsoft's HAX guidelines ask for easy dismissal (G8), easy correction (G9), narrow scope when in doubt (G10), an explanation of why (G11), cautious updates (G14) and global controls (G17). [sourced, HAX toolkit]

### Running models locally on Windows

- Foundry Local is generally available. It has a Rust SDK from cargo, uses WinML on Windows, runs offline, and downloads models on first run and then caches them. Its catalog includes Qwen and Phi models. [sourced, Foundry Local GA post]
- XGrammar shows grammar-constrained decoding with near-zero overhead. A small local model can therefore be forced to emit only a valid candidate ID. [sourced, XGrammar paper]
- tldraw's AI kit has the model emit typed canvas actions that are validated and sanitized before they touch the canvas. [sourced, tldraw AI docs]
- The Vercel AI SDK's generative UI passes tool *results* into React components. The data on screen comes from tools, not model text. [sourced, AI SDK docs]

## Where generative UI appears, surface by surface

The rule is the same everywhere. A surface has a fixed frame and a small number of named slots. Only slots change. [inferred]

| Surface | Generated? | What System 1 decides | What stays fixed |
|---|---|---|---|
| Top bar, mode switcher (1 to 4), Layers, Export, egress pill | **Never** | Nothing | Everything. This is muscle memory. [inferred from Findlater] |
| Canvas layout and positions | **Never** | Nothing | Positions are authored (K5). New entities still land in the unplaced tray. |
| Inspector | **One slot, "Now"** at the top | Which one card best explains this entity right now (budget breach, failing check, pending choice, fog, agent waiting) | Section order below the slot: budgets, sources, match, drift, fog, calibration, choices. These stay deterministic. |
| Canvas callouts | **Up to 3 callouts**, on the annotation layer | Which entities get a callout, and which card each callout shows | Callout anchors follow the entity. Leader-line placement is a deterministic layout pass. No callout appears without a state change (Q15). |
| Overlays and exploded views | **No** in v0.1 | At most a suggestion chip, for example "Show fog" | The overlay itself. Exploded views are out of v0.1 anyway (D1). |
| Ask bar (Ctrl K) | **Answer area only** | The intent (from a closed list) and the entity or territory it is about | Commands, fuzzy search and keyboard behavior. Answers render as catalog cards, never as free prose from System 1. |
| Choice review | **Evidence focus only** | Which evidence card opens first for each choice (budget chart, code reference, alternative) | The queue order, which is risk = (1 − confidence) × reversal cost (K17). System 1 never suggests accept or reject. |
| Simulate | **What-if preset chips** | Which 2 or 3 presets to offer, picked from presets built from choice alternatives and ghosts | Every result number comes from the replay engine (D5). System 1 never predicts an outcome. |
| Agent sessions | **One summary slot per session** | Which fact leads (blocked on you, checks failing, a new choice, finished and stale) | The lane, marks, files and spend rows. |
| Share-safe export | **Never** | Nothing | Export includes pinned views only. Unpinned generated cards and agent notes are left out by default. Redaction (R2) runs on every bound value. |
| Onboarding and the unplaced tray | **Suggestions only** | Which next step to show, and which entity an unplaced item probably belongs near | Placement needs the architect to drag it (K5). Matching stays rule-based (D10). |
| Budgets and checks | **Never** | Nothing | Agents can't edit them (K16), and no model can either. |

## Architecture

### Roles

| Role | Runs where | When | Writes | Never writes |
|---|---|---|---|---|
| **Candidate builder** | Rust core | On every state change (the same events that wake the canvas in Q15) | Candidates, each a catalog component with data bindings, a source rule and feature bins | Nothing it cannot derive from the twin |
| **System 1, selector** | Local by default | On user input (selection, Ctrl K, mode switch) and on wake events | Candidate IDs, order and a confidence band | Numbers, labels, prose, positions |
| **System 2, planner** | The user's agent over MCP, or an opt-in remote model | In the background, seconds to minutes | New candidate recipes (a query, a component and bindings) and short notes that use binding tokens for every number | Raw numbers, budgets, checks, layout |
| **Validator** | Rust core | Before anything renders | Pass, or fall back to the slot default | Nothing |
| **Renderer** | Webview side panels and the canvas annotation layer | On a validated view | DOM for panels, glyphs for callouts | Nothing outside the slot |

[inferred for the whole table; the split follows Helix and json-render, both sourced]

### System 1 has three tiers behind one trait

1. **Tier 0, rule scorer.** Hand-written priors (fail beats drift beats fog, and "blocked on you" beats everything for a session) plus per-user weights learned from pins, dismissals and opens. Weights live in SQLite app state (D9). It runs in microseconds, needs no download and is always present. It ships in v0.1. [inferred]
2. **Tier 1, small local model.** A small instruct model through Foundry Local or llama.cpp, forced by a grammar to output only candidate IDs. It gets features, never raw strings. It ships only if a replay test shows it beats Tier 0 on "picked the card the user then opened" by a clear margin. [inferred] A 0.5B to 1.5B model on a typical Windows laptop CPU probably needs 100 to 400 ms for a short prompt. [guess] That is why it has to earn its place, and why its first job is the Ctrl K intent, where words matter. [inferred]
3. **Tier 2, Jev.** Opt-in, off by default, logged on the egress counter, blocked by the offline switch. It is useful for teams that already have TypeSafe enterprise access with zero data retention. [inferred from the sourced Jev facts]

### Reading the twin without inventing numbers

- The core exposes a read model as a JSON tree. Every card prop that shows a value is a binding like `{"$bind": "/inspect/verifier/budgets/0/value"}`, resolved by the core at render time. This is A2UI's JSON Pointer approach. [sourced for A2UI; inferred for Acan]
- System 1 sees **feature bins**, not values. For example `over_ratio: "1.25-1.5"`, `sev: "drift"`, `stage: "verifying"`, `waiting_on: "you"`. The core does every comparison and count. This avoids Jev's known weak spots in math, counting and large state. [sourced weak spots; inferred design]
- Telemetry strings (span names, file paths, error messages) never go into a System 1 prompt. They can contain injected instructions. [sourced that Jev is injection-susceptible; inferred that telemetry is an injection channel]
- Card text comes from catalog templates with bound holes, or from text the twin already holds, such as drift lines ("Queue wait is over budget: 41 s against 30 s"). [inferred]
- System 2 notes must use binding tokens for numbers. The validator rejects any digit in a note that is not inside a binding token. Notes always show their author ("Droid note"). [inferred]

### Latency budget

| Step | Target | Basis |
|---|---|---|
| Deterministic default for a slot | In the same frame as the selection | Nielsen 0.1 s limit for direct manipulation [sourced] |
| System 1 decision on a cache hit | Under 1 ms | [guess] |
| System 1 decision on a miss, Tier 0 | Under 1 ms | [guess] |
| System 1 decision on a miss, Tier 1 or 2 | Deadline of 100 ms for the inspector, 300 ms for Ctrl K answers | Nielsen [sourced]; Zed p50 target of 200 ms [sourced]; numbers are [guess] |
| System 2 | Seconds or more, never awaited | Helix split [sourced] |

**No late swaps.** If System 1 misses its deadline, the default stays on screen. The late answer is cached and used the next time the user opens that entity. Content never jumps while the user is reading. [inferred from Gajos and HAX G14]

### Caching and determinism

- Cache key = slot, entity, mode, the sorted candidate IDs and the feature bins. A state change that moves a bin invalidates the entry. Bins, not raw values, keep the cache hit rate high when values jitter. [inferred]
- The same twin state gives the same view. Tier 0 is a pure function. Tier 1 runs at temperature 0 with a fixed prompt order. Ties break on stable candidate ID. [inferred]
- To avoid option-order bias, Tier 1 and Tier 2 score each candidate on its own (a Score question per candidate, asked in parallel) instead of one Choice over a list. [sourced bias and parallel questions; inferred remedy]
- Speculative work runs only on wake events. When a state change lands, the core precomputes the "Now" slot for the entities it touched. Nothing runs on a timer, so an idle canvas stays idle (Q15). [sourced fan-out pattern; inferred use]

### Validation

A view renders only if all of these pass, in order:
1. The spec matches the catalog schema.
2. Every component is in the catalog for that slot.
3. The view fits the slot budget (number of cards, rows and characters).
4. Every binding resolves to a value of the declared type and unit.
5. No free text contains a digit outside a binding token.
6. Redaction tags flow from the bound values to the card, so export can redact them (R2).

Any failure renders the slot's default candidate, which always exists and is always deterministic. Failures are counted in a local diagnostics view. [inferred; validation before applying follows tldraw, sourced]

### Degradation

Tier 2 → Tier 1 → Tier 0 → static default order. Offline mode or a missing model drops to Tier 0. The user only notices through "Why this" (below), which names the tier that chose. [inferred]

### Egress

- Tier 0 and Tier 1 send nothing. The pill stays at `local · 0 out`. [inferred]
- Downloading a Tier 1 model is an outbound connection. It is logged with destination, bytes and the reason "model download," and it is listed in the data statement. The offline switch blocks it. [inferred from Q16; Foundry Local downloads on first run, sourced]
- A Tier 2 call sends candidate IDs and feature bins only, never bound values or strings. It is logged per call and counted on the pill. [inferred]
- A System 2 agent over MCP talks to Acan on loopback, which Q16 keeps open in offline mode. What that agent sends to its own vendor is the agent's egress, not Acan's. The data statement should say so plainly. [inferred]

### Stability rules

1. **Fixed slots.** Each surface has named slots of fixed size and position. A slot never grows into its neighbors.
2. **No late swaps.** See the latency section.
3. **Change only on events.** A slot's content changes only after a user action or a wake event, and only if the new winner beats the current one by a margin. At most one change per slot per 30 s, except when a new failure arrives, which always shows at once. [guess for the numbers]
4. **Pin.** Any generated card or view can be pinned. A pin writes `.acan/views/<slug>.yaml` (a proposed new folder for D11) with the component and its bindings. Pinned content always wins its slot, keeps its structure forever, and only its bound values update.
5. **Why this.** Every generated card has a small "why" mark that shows the source rule, the tier that chose it and the confidence band (HAX G11). [sourced guideline]
6. **Dismiss.** Dismissing a card suppresses it for that entity until its state changes, and lowers its Tier 0 weight (HAX G8, G9). [sourced guideline]
7. **One switch.** Settings has an "Adaptive slots" toggle. Turning it off gives the deterministic default everywhere (HAX G17; adaptable beats adaptive in Findlater). [sourced]
8. **No motion.** Swaps are instant replacements with no CSS animation, because one CSS animation cost VS Code 13% idle CPU (L7). [sourced in map L7]

## Component catalog sketch

A first catalog of 13 components. Types are sketches. Real paths come from the Rust read model. The examples below use the `ui-v1/acquit.mjs` fixture shape.

| Component | Props (all bound unless marked literal) | Slots allowed |
|---|---|---|
| `BudgetGauge` | metric, value, budget, max, unit, by | Now, callout |
| `DriftLine` | sev, text, ref | Now, callout |
| `Sparkline` | series, label (literal from template) | Now |
| `SourceRow` | src, ok, what, detail | Now |
| `FogNote` | fog facets, the lowest facet | Now, callout |
| `ChoiceCard` | choice id, question, text, conf, cost, risk | Now, callout, choice review |
| `EvidenceRef` | file and line, or a map reference like L13 | Choice review, Now |
| `WhatIfChips` | 1 to 3 preset ids | Now, Simulate |
| `CompareBars` | replay result A and B for one metric, budget | Simulate, Ctrl K |
| `SessionCard` | agent, state, now, since, territory | Sessions, Ctrl K, callout |
| `CheckList` | checks with ok flags and locks | Sessions, Ctrl K |
| `CIRunRow` | run number, status, what | Sessions, Ctrl K |
| `AgentNote` | author (literal), text with binding tokens | Now (below the card), Ctrl K |

### Example spec for the verifier "Now" slot

Candidates the core built, as System 1 sees them. No values, only bins:

```json
{
  "slot": "inspector.now",
  "entity": "verifier",
  "mode": "live",
  "candidates": [
    { "id": "budget:verifier:queue-wait-p95", "component": "BudgetGauge", "rule": "budget_over", "bins": { "over_ratio": "1.25-1.5", "sev": "drift" } },
    { "id": "choice:verifier-concurrency-2", "component": "ChoiceCard", "rule": "choice_pending", "bins": { "conf_band": "low", "cost": "medium" } },
    { "id": "fog:verifier:observed", "component": "FogNote", "rule": "fog_facet_low", "bins": { "facet": "observed", "level": "half" } }
  ]
}
```

System 1 answer:

```json
{ "pick": ["budget:verifier:queue-wait-p95"], "band": "act", "tier": 0 }
```

The view the core renders after validation. Every number is a binding:

```json
{
  "root": "now",
  "elements": {
    "now":  { "component": "BudgetGauge", "props": {
                "metric": { "$bind": "/inspect/verifier/budgets/0/metric" },
                "value":  { "$bind": "/inspect/verifier/budgets/0/value" },
                "budget": { "$bind": "/inspect/verifier/budgets/0/budget" },
                "max":    { "$bind": "/inspect/verifier/budgets/0/max" },
                "unit":   { "$bind": "/inspect/verifier/budgets/0/unit" } },
              "children": ["why", "next"] },
    "why":  { "component": "DriftLine", "props": { "line": { "$bind": "/inspect/verifier/drift/0" } } },
    "next": { "component": "WhatIfChips", "props": { "presets": [
                { "$bind": "/sim/presets/verifier/0" },
                { "$bind": "/sim/presets/verifier/1" } ] } }
  }
}
```

The `/sim/presets/verifier/*` entries are built by the core from the alternatives of choice `verifier-concurrency-2`. That gives "concurrency 2" and "keep 1, shorten Docker cold start." [inferred; data from `acquit.mjs` lines 206 to 209]

## Three Acquit examples

### 1. The architect clicks the verifier

- The inspector opens with its fixed sections. The "Now" slot shows the deterministic default in the same frame. Here Tier 0 and the default agree, so nothing changes.
- The card is a `BudgetGauge` bound to queue wait p95, showing 41 s against a 30 s budget with a 60 s max (`acquit.mjs` line 68). Under it is the existing drift line and two what-if chips.
- Clicking "concurrency 2" switches to Simulate with that preset loaded. The replay engine computes the result. The model card already warns that Docker cold start varies from 4 to 30 s and calibration is 0.64 (line 237), so the result band is wide. System 1 adds no number of its own.
- Later, a Droid session over MCP adds an `AgentNote` recipe. The note reads "Cold start dominates. {sim.verifier.unmodeled}" with a binding token, and shows "Droid note" with a why mark. It appears the next time the verifier is opened, never mid-read.

[inferred; values sourced from the fixture]

### 2. PayPal breaches its budget at 14:02

- The error rate reaches 4.1% against a 2% budget (line 83). This is a wake event, so the canvas wakes (Q15) and the core builds candidates.
- System 1 ranks callouts across the canvas. It gives one callout to `paypal` with a `BudgetGauge` and a `ChoiceCard` for "Back off retries to x2 with jitter" (conf 0.55, risk 0.18, line 93). The two other callout slots keep their current content, because nothing changed for them.
- The Simulate chips offer "retries x2 with jitter" and "keep x3, add the breaker ghost." Both come from the alternatives of choice `paypal-retry-x2` (lines 212 to 215). The breaker ghost's evidence line, "212 retried calls in the last hour · 38 hit the third try" (line 129), is shown verbatim from the twin.
- The choice review queue order does not change. It is still sorted by K17 risk.

[inferred; values sourced from the fixture]

### 3. "What's blocking radar?" in the ask bar

- Lexical search over entity, territory and session names first narrows the field to `radar`, `radar-otel` and `droid-radar`. Then System 1 picks the intent `blockers_for` from a closed list of about 20 intents. The narrowing step keeps System 1 away from the full 2,000-entity state. [inferred; sourced that large state is a weak spot]
- The answer area shows three cards. A `SessionCard` says Droid is verifying, at 3 of 5 checks, for 38 minutes. A `CheckList` shows the two open checks, "Trace context across the queue and verifier callback (L13)" and the locked overhead budget (lines 117 to 122). A `CIRunRow` shows run #418 running, at 3 of 5 so far (line 192).
- If the user had typed "what's waiting on me," the answer would lead with Codex blocked on an approval prompt for 9 minutes, because "blocked on you" beats every other rule.
- No sentence in the answer was written by a model.

[inferred; values sourced from the fixture]

## Thin slice

One week, after the week-6 demo, at the start of weeks 7 to 9. [guess for the timing]

1. **Catalog of 5 components.** `BudgetGauge`, `DriftLine`, `ChoiceCard`, `SessionCard`, `CheckList`. Rendered in the side panel DOM and as callouts.
2. **4 candidate rules** in the Rust core: budget over, check failing, choice pending, agent waiting on you.
3. **2 slots.** The inspector "Now" slot and the 3 canvas callouts. Nothing else is adaptive.
4. **Tier 0 only.** The rule scorer, plus learning from pins and dismissals.
5. **The validator**, all six checks, with a fallback to the default.
6. **Pin to `.acan/views/`**, the "why" mark, dismiss, and the global off switch.
7. **A replay test harness.** Record selection sessions on Acquit, then measure "the card the user opened next" for Tier 0 against a fixed default. Later, the same harness decides whether Tier 1 earns its place.
8. **Budget checks** in CI. No System 1 work while idle (the Q15 harness), zero egress with adaptive slots on, and the inspector default in the same frame as the selection.

Out of the slice: Tier 1, Tier 2, System 2 recipes, the Ctrl K answer area and Simulate chips.

## Risks and open questions

### Top risks

1. **Instability erodes trust.** If slots change too often, senior engineers stop trusting the inspector and turn the feature off. Adaptive menus are slower than static ones. [sourced, Findlater] Mitigation is fixed slots, change only on events, no late swaps, pins and one off switch. [inferred]
2. **A wrong but valid pick looks authoritative.** Schema-valid does not mean right, and json-render notes Jev confidence is not a calibrated quality threshold there. [sourced, TrueFoundry; json-render Jev docs] A confident card can hide the real problem. Mitigation is that the deterministic sections stay visible below the slot, "why" names the rule, and Tier 0 is easy to reason about. [inferred]
3. **The idle, memory and Windows budgets break.** A local model means a download of hundreds of megabytes, resident memory and CPU on the same laptop that runs the agents. That cuts against the 500 MB memory target (A10) and the idle budget (Q15). [guess for the sizes; inferred for the conflict] Mitigation is that Tier 0 ships alone, Tier 1 loads lazily and unloads after idle, and its memory counts against the soak test.

### Other risks

- **Prompt injection through telemetry.** Span names and error messages are attacker-controlled in some systems. Strings never reach System 1 or System 2 prompts unless they pass through MCP to an agent the user already trusts. [inferred]
- **No central learning.** Acan is local-first and closed, so it cannot learn from other users' clicks without breaking the data statement. Tier 0 priors must be good by design, and the replay harness runs on fixtures. [inferred]
- **Candidate explosion.** At 2,000 machines, candidate building on each wake must be incremental, touching only entities whose bins changed. [inferred]
- **Vendor risk on Jev.** It is on a waitlist, hosted only, and two months old. [sourced, TrueFoundry; TypeSafe models page]

### Open questions

1. Does the Ctrl K answer area belong in v0.1? D1 does not list it. This note assumes v0.2 unless the user wants it in the slice.
2. Should pinned views live in git (`.acan/views/`, shared with the team) or only in app state? This note proposes git, because pins are the architect's authored view, like layout (K5).
3. Who accepts a System 2 recipe? This note proposes the architect accepts each new recipe once, in the spirit of K16, because a recipe decides what the team sees first.
4. May agent notes ever appear in a share-safe export? This note proposes no by default.
5. Which local runtime for Tier 1, Foundry Local or llama.cpp? Foundry Local has a Rust SDK and WinML. [sourced] Its behavior under WebView2 memory pressure is untested. [guess]

## Proposed map.md amendments

For the parent or the user to apply. Not applied here.

- **New A11, generative UI.** "Generated content appears only in named, fixed slots: the inspector 'Now' slot, up to 3 canvas callouts, and later the Ctrl K answer area, Simulate preset chips, session summaries and choice evidence focus. The Rust core builds candidates from the twin with every value as a data binding. A System 1 selector picks among them and never writes numbers, labels, prose or positions. v0.1 ships the deterministic Tier 0 scorer only. Pins write `.acan/views/*.yaml`. One setting turns adaptive slots off."
- **Q16, extended.** "The outbound log and data statement cover model downloads and any remote model call (Tier 2 Jev, opt-in). Remote calls send candidate IDs and feature bins only. The offline switch blocks both."
- **A10 and Q15, extended.** "System 1 runs only on user input and wake events, never on a timer. The inspector default renders in the same frame as the selection. A System 1 answer that misses its 100 ms deadline is cached, not shown. Any local model's memory counts against the 500 MB target and the 24-hour soak."
- **A6, extended.** "MCP adds a `views` resource (candidates and their bins, no strings) and a `propose_view_recipe` tool. Recipes land as files and need the architect's accept once."
- **D11, extended.** "Add `.acan/views/` for pinned views and accepted recipes."
- **S3 and R2, extended.** "Export includes pinned views only. Agent notes are excluded by default. Redaction tags flow from bound values into cards."
- **K5 and K17, reaffirmed.** "Generated callouts never move or place entities. Generated focus never reorders the choice queue."

## Sources opened

- TypeSafe AI, Introducing System One models and Jev. https://typesafe.ai/blog/introducing-system-one-models-and-jev
- TypeSafe docs, introduction. https://docs.typesafe.ai/introduction
- TypeSafe docs, System One concept, fan-out pattern, confidence, jev-1.13 jaggedness and models pages, all under https://docs.typesafe.ai/
- TypeSafe enterprise. https://typesafe.ai/enterprise
- TrueFoundry, review of TypeSafe AI Jev. https://www.truefoundry.com/blog/typesafe-ai-jev
- json-render, Jev composition (experimental). https://json-render.dev/docs/jev
- json-render repository. https://github.com/vercel-labs/json-render
- Thesys OpenUI, OUI-1. https://www.openui.com/blog/oui-1
- Google Research, Generative UI. https://research.google/blog/generative-ui-a-rich-custom-visual-interactive-user-experience-for-any-prompt/
- A2UI. https://a2ui.org/
- Google Developers Blog, Introducing A2UI. https://developers.googleblog.com/introducing-a2ui-an-open-project-for-agent-driven-interfaces/
- A2UI data binding. https://a2ui.org/concepts/data-binding/
- MCP Apps overview. https://modelcontextprotocol.io/extensions/apps/overview
- MCP blog, MCP Apps. https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/
- Vercel AI SDK, generative user interfaces. https://ai-sdk.dev/docs/ai-sdk-ui/generative-user-interfaces
- tldraw AI docs. https://tldraw.dev/docs/ai
- Figure, Helix. https://www.figure.ai/news/helix
- Nielsen Norman Group, response times. https://www.nngroup.com/articles/response-times-3-important-limits/
- Zed, edit prediction. https://zed.dev/blog/edit-prediction
- Gajos et al., predictability and accuracy in adaptive interfaces, CHI 2008. https://www.eecs.harvard.edu/~kgajos/papers/2008/gajos08predictability.shtml
- Findlater and McGrenere, static, adaptive and adaptable menus, CHI 2004. https://dl.acm.org/doi/10.1145/985692.985704
- Microsoft HAX Toolkit guidelines. https://www.microsoft.com/en-us/haxtoolkit/library/
- Microsoft Foundry Local GA. https://devblogs.microsoft.com/foundry/foundry-local-ga/
- XGrammar. https://arxiv.org/abs/2411.15100

A small sketch of the inspector slot rules is in `generative-ui-sketch.html` next to this file.
