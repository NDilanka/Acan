# Concept bakeoff: judging

Three model families got the same brief: Anthropic (`anthropic.md`), OpenAI (`openai.md`) and xAI (`xai.md`). None of the three read the others' files. This page groups their concepts by mechanism, counts the agreement between arms, and picks the shortlist. The evidence base is `../00-synthesis.md`.

## Clusters

| Cluster | Anthropic | OpenAI | xAI | Arms | Best rank | Pain |
|---|---|---|---|---|---|---|
| A. Intent as pinned runtime examples | Witnesses instead of prompts (1) | Trace splicing (1) | The standing order (1) | 3 | 1, 1, 1 | 1 |
| B. Clashes settled before code | Footprints clash (3) | Split at the port (2) | The clash pin (2) | 3 | 2 | 2 |
| C. Human attention as a dispatch limit | Inspection gate (2), Leave permits (4) | Book my next decision (4) | The shift (6) | 3 | 2 | 2 |
| D. Revisioned, issued intent | (contract `state: stale` only) | Reissue the blueprint (3) | The issued sheet (4) | 2 | 3 | 1, 2 |
| E. Experiments before build | none | Disprove before build (5) | The bench (3) | 2 | 3 | 1 |
| F. Path budget shares | Path budget shares (5) | none | The margin stack (5) | 2 | 5 | 1 |

All three arms ranked cluster A first, each on its own. That is the strongest signal in the bakeoff. It also matches the synthesis's pain 1 evidence: examples, counterexamples and protected acceptance (`../00-synthesis.md`, "Pain 1").

Cluster B shows up in all three arms with three different ways to resolve a clash. Anthropic serializes the work or splits out an interface. xAI has one side yield in a single click. OpenAI publishes the port contract so both sides can build at once against fake peers. These fit together as one resolution menu.

Cluster C shows up in all three arms but ranks low in two of them. The synthesis ranks "review capacity throttles new work" as its second-strongest convergence (6 of 7 slices). The arms rank it lower because it depends on A and B. A gate can only weigh work the plant can already name.

Cluster D is not a separate screen. A and B need it underneath. An order or a footprint is meaningless if the drawing can change under the agent without anyone noticing. It becomes a mechanism inside the shortlist.

Clusters E and F are good second-wave ideas. E needs a working sandbox replay, and F's central claim rests on path arithmetic the research warns about (`../g-engineering-analogies.md`, "Tolerance budgets").

## Shortlist

1. **Work orders from witnesses (A + D).** You pin a real moment in Replay as must-keep or must-not. You add one sentence and post it as an order on the machine. Agents read the order back, which shows how they interpret it, before they claim it. The order is tied to a drawing revision. Its pins become locked checks (K16).
2. **Clash on the belt (B + D).** Before writing, agents declare the machines and belts they will touch and what they will assert about them. When two declarations disagree on a belt, a pin appears there with the consequence, tied to a witness when one exists. You settle it with one of four actions: one side wins, serialize, split at the port (publish a contract), or merge territories.
3. **The inspection gate (C).** You set how many review minutes you can spare per hour, or a time you will be back. New agent work is admitted only within that limit, and held work waits as scaffolding at the inspection station. When you come back, you get a return brief instead of a timeline.

Order of build: 1, 2, 3. Each depends on the one before it.

## Gaps the arms share

- Witnesses only cover behavior that already runs. A new feature has no moment to pin. The prototype for shortlist item 1 has to show what an order looks like for a ghost (a Simulate run, labeled as a prediction).
- Every kill test is a user study. None of them can run inside this session. The prototypes will be built so a study can run against them, and every number in them is scripted.
- No arm measured anything. Review-minute estimates, readback match rates and clash precision are all guesses until real sessions exist.
