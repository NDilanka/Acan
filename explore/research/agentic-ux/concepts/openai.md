# Acan interaction concepts

17 October 2026.

The engineer should specify behavior, publish boundaries, and control when judgment is needed. Agents should translate those decisions into work.

These are proposals. Every minute below is a scripted HTML mock, not a measured agent performance claim. Tests are proposed kill gates. Research citations name the supplied sibling files and their sections. Novelty means an opportunity within `f-frontier.md`'s reviewed sample, not a claim of invention.

All five concepts keep the authored plant layout, catalog, and four modes. Their controls annotate existing machines, belts, and Floor entities. Budgets and checks remain human-protected. Passive sessions remain observe-only until explicitly connected. Types use branded domain IDs. `ArtifactRef` identifies local evidence by digest, code revision, workload, and environment.

## 1. Trace splicing

Edit a failure into the behavior you want, then make agents demonstrate competing interpretations before they write code.

### A minute on Acquit

At 0 seconds, press `4` for Replay. Click the red API-to-PayPal belt and select a recorded payout whose response timed out. Its event tape opens beneath the belt.

At 10 seconds, drag a "lose response after remote acceptance" fault onto the PayPal dock. The mock shows an accepted payout, a missing response, and another retry. Click the second remote acceptance and press `Delete`. This marks a forbidden outcome. It does not erase evidence.

At 20 seconds, Shift-click a normal payout and choose "Preserve." A predicate strip now says "At most one remote payout per payout ID" and "Normal payouts still settle." These are explicit checker templates, not inferred promises.

At 30 seconds, two scout bots play different readbacks on the same belt. One stops retries forever. The other retries with the same idempotency identity and reconciles the pending local record. The first satisfies the duplicate constraint but leaves a payout unresolved. Acan asks one question beside that SQLite record. Must this case eventually settle, or remain explicitly pending during an outage?

At 45 seconds, click "Pending during outage, settle after recovery." A third tape demonstrates recovery. Press `Enter` to issue the order. Approved predicates become protected checks. The payout territory gains a behavior ribbon. Builders receive the examples, boundaries, and unresolved cases rather than the replay conversation.

### The changed loop

This attacks pain 1. The engineer edits outcomes and resolves consequential ambiguity before implementation. Today's loop discovers the wrong interpretation after a large diff. Here, the agent must expose its interpretation through a witness first.

Unsupported expectations remain prose with a visible verification hole. Acan must not pretend that every gesture produces an executable test.

### Data

```ts
type BehaviorOrder = {
	id: OrderId;
	base: Revision;
	scope: EntityId[];
	examples: {
		tape: ArtifactRef;
		expected: PredicateRef[];
	}[];
	preserve: ArtifactRef[];
	readbacks: { session: SessionId; witness: ArtifactRef }[];
	state:
		| { kind: "draft" }
		| { kind: "issued"; checks: ArtifactRef; approvedBy: HumanId };
};
```

### Evidence and novelty

`d-intent-and-verification.md`, "Counterexample-directed repair" and "Contract gestures on the plant," support examples and explicit contracts. `g-engineering-analogies.md`, "RFIs with an agent readback," supports checking interpretation. The additional proposal is an active teaching loop where disagreement produces the next example. Against `f-frontier.md`, "Objectives attached to the system, not the agent," the distinction is a protected behavioral order, not visual code editing or an MCP drawing.

### What leaves ui-v1

Replace the catalog-first start shown in `bpplace.png` for repair tasks. A breaker can emerge as an implementation proposal after the behavior is issued. The human need not prescribe one upfront.

### Risk and prototype test

The largest risk is teaching the examples rather than the intended rule. Give eight engineers the lost-response repair, a normal payout, and an unseen delayed-recovery case. Compare against the same evidence in chat. Kill the concept if two engineers issue a contract that permits duplicate remote payouts, or if correcting the specification requires no fewer human interventions than the control.

## 2. Split at the port

Publish one executable boundary so agents can build both sides independently instead of waiting for each other's patches.

### A minute on Acquit

At 0 seconds, click Droid's failed radar check for context across the queue and verifier callback. Press `2`, select the API-to-verifier belt, and choose "Split repair here."

At 10 seconds, the belt opens into two aligned port cards. The request includes a job identity and trace context. A dashed return shows the callback. A mismatch marker says that one proposed reader expects a parent span while the producer emits a span link.

At 20 seconds, select "Link across the asynchronous gap." Three fixture buttons demonstrate enqueue, verifier completion, and callback after an API restart. An unknown remains visible if restart behavior lacks a checker. The engineer approves the boundary version, not a generated implementation plan.

At 30 seconds, click "Publish and dispatch." Droid's broad assignment pauses and becomes two issued slices. Two managed bots receive separate worktrees, the same immutable fixtures, and a fake peer. Each can now test its side without the other's implementation.

At 40 seconds, the Floor highlights `apps/api/src/server.ts` as a real shared write target. Click "One integration owner." Workers write separate modules. Only the integrator may change the shared wiring. The existing gate seal remains stale until its checks rerun.

At 55 seconds, the consumer requests a different callback field. A striped mark appears at the port. Unrelated work continues, but integration cannot proceed with mismatched boundary versions. The engineer can publish a revision or keep the existing contract.

### The changed loop

This attacks both pains. Human coordination becomes agreement about a seam and its owner. Agents stop exchanging partial implementations as implicit specifications.

Separate worktrees alone do not establish compatibility. Fake peers remove waiting. Versioned boundary tests and one shared-file writer prevent a clean Git merge from masquerading as a correct integration.

### Data

```ts
type PortSplit = {
	id: SplitId;
	ends: [EntityId, EntityId];
	contract: { revision: Revision; fixtures: ArtifactRef; checks: ArtifactRef };
	workers: {
		session: SessionId;
		side: "producer" | "consumer";
		workspace: WorkspaceId;
		ownedPaths: RepoPath[];
		contractRevision: Revision;
	}[];
	integrator: SessionId;
	sharedPaths: RepoPath[];
};
```

### Evidence and novelty

`a-landscape.md`, "Collision rehearsal," and `g-engineering-analogies.md`, "Semantic clash detection," establish the coordination problem. "Submittals before commitment" supports publishing before dependent construction. Against `f-frontier.md`, "Novelty check," spatial teams and shared drawings are already taken. This proposal makes a system boundary an executable substitute for waiting, with compatibility checked again at integration.

### What leaves ui-v1

Replace whole-territory `depends` as the only expression of waiting. `paypal-payouts` need not wait for all of `radar-otel` if its required boundary evidence is available. The Floor timeline becomes secondary to port readiness and ownership.

### Risk and prototype test

An incomplete boundary can freeze the wrong abstraction. Ask engineers to commission a callback trace repair with incompatible field names in separate files. Kill the concept if the integration gate permits incompatible versions, or if median human coordination interventions do not fall below an isolated-worktree task-board control. Include a mid-task contract revision in both conditions.

## 3. Reissue the blueprint

Change one system rule and require every affected agent to acknowledge its concrete consequences, without re-prompting each session.

### A minute on Acquit

At 0 seconds, click the stale `gate-sessions` seal. The plant shows the basic-auth exemption for `/paypal/webhook`, radar edits to `server.ts`, and the payout path. Press `R` for "Revise issued intent."

At 10 seconds, choose a security predicate at the Gate. "Verify the PayPal signature before a webhook can mutate payout state." The exemption from basic auth remains a separate decision. Acan must not equate the two.

At 20 seconds, approve the new check and inspect an impact preview. It marks the webhook handler and payout mutation path. It leaves the hand-written DatabaseSync timing wrapper outside the known impact. Every mark links to a declared dependency or says "Inferred."

At 30 seconds, click "Issue revision." The old issued drawing stays inspectable. Managed workers lose integration permission for affected deliveries until they acknowledge the new revision. This does not delete their branches or stop unrelated work.

At 40 seconds, one bot returns "Move signature verification before mutation." Another says "My retry test bypasses ingress, so it cannot prove this rule." Their readbacks attach to the affected plant objects. The second bot can continue retry experiments but cannot claim the new security condition.

At 55 seconds, click the newly exposed proof gap to commission an ingress test. Affected acceptance marks remain striped. No agent can turn an old green test into evidence for the revised requirement.

### The changed loop

This attacks both pains. A correction becomes a versioned change to shared intent, not several messages whose delivery and interpretation the human must remember.

The commit gate binds deliveries to an issued revision. Acknowledgment alone proves neither understanding nor correctness. Fresh checks still decide acceptance. Unknown impact loses integration permission rather than receiving an invented all-clear.

### Data

```ts
type IntentRevision = {
	from: Revision;
	to: Revision;
	approvedBy: HumanId;
	changed: ConstraintId[];
	impacts: {
		territory: TerritoryId;
		basis: ArtifactRef[];
		certainty: "declared" | "inferred" | "unknown";
		invalidatedChecks: CheckId[];
		response:
			| { kind: "held" }
			| { kind: "acknowledged"; readback: ArtifactRef };
	}[];
};
```

### Evidence and novelty

`g-engineering-analogies.md`, "Change orders that reopen the right choices" and "Semantic presence with bounded authority," support revision-aware work. `d-intent-and-verification.md`, "The plan is an execution envelope," supports continuing authorization. Against `f-frontier.md`, "A persistent system model that survives a change," the proposed distinction is consequence-based reissuance across running workers. Editing a shared note or drawing does not establish who still works against its obsolete meaning.

### What leaves ui-v1

Replace generic stale status as the endpoint of invalidation. Also replace the need to reopen choices one at a time in `c7-review.png`. The ledger remains risk-ranked, but a revision can reopen the exact affected choices together.

### Risk and prototype test

The largest risk is an incorrect impact graph, followed by acknowledgment theater. Ask engineers to change the webhook rule across three running workers, including one silent worker and one unrelated SQLite task. Kill the concept if two of eight engineers release obsolete evidence as current, or if the unrelated task is needlessly stopped in most trials.

## 4. Book my next decision

Reserve a period of human absence and make agent dispatch respect it through enforceable parking points.

### A minute on Acquit

At 0 seconds, click the inspection station showing seven waiting items. Press `A` and drag the return marker to twenty minutes from now. Choose capacity for two decisions on return.

At 10 seconds, the preview opens in Simulate on graph paper. It shows possible arrivals at the station, not exact completion times. Droid's radar verification has an uncertain arrival window. A gate proof refresh can run without a new architectural decision.

At 20 seconds, select "Tests and isolated edits only." The preview lists the allowed workspaces and commands. Deployment, external writes, budget changes, and protected-check edits are excluded. A row says "Codex is passive. Not covered." Its terminal approval remains unresolved.

At 30 seconds, click "Rehearse absence." The mock advances to a radar failure that would require a schema change. Droid parks before that authority boundary. An independent proof refresh continues. The station shows a parked request rather than an urgent toast.

At 45 seconds, click "Commit window." Managed bots receive those grants and parking rules. Each mission reserves a return slot or must park before requesting another decision. This bounds ready work without predicting every future question. Existing choices retain their risk order.

At 55 seconds, return to Live. The plant becomes quiet and displays "Managed work bounded until your return. One observed-only session excluded." Click that exclusion to open Codex's terminal if desired. Acan makes no promise about an uncontrolled process.

### The changed loop

This attacks pain 2. The engineer decides when judgment is available. Agents decide which authorized work fits around that availability.

The promise is about authority, not success or uptime. Runtime command and integration gates must enforce parking. Duration estimates only inform dispatch. Unknown estimates consume conservative capacity rather than becoming precise countdowns.

### Data

```ts
type AwayWindow = {
	start: Instant;
	returnAt: Instant;
	reviewSlots: number;
	managed: {
		session: SessionId;
		grants: GrantId[];
		parkBefore: EffectKind[];
	}[];
	excluded: SessionId[];
	decisions: {
		choice: ChoiceId;
		due: Instant | "unknown";
		fallback: "park" | "investigate";
	}[];
};
```

### Evidence and novelty

`e-attention.md`, "Safe to leave," "Deliver at my checkpoint," and "Review capacity limits production," provide the basis. `c-supervisory-control.md`, "Safe-to-leave scheduling," distinguishes authority from prediction. Against `f-frontier.md`, "A decision queue that conserves human attention," this moves beyond queue presentation. The human's unavailable time becomes a dispatch constraint across agents.

### What leaves ui-v1

Replace the permanently demanding "Waiting on you" roster in `l-default.png` with a bounded return commitment. Keep the Floor available for investigation, not routine vigilance. Do not add another always-animated schedule.

### Risk and prototype test

The largest risk is false reassurance, especially about passive agents. Ask eight engineers to arrange twenty uninterrupted minutes while radar may request a migration and Codex remains disconnected. Kill the concept if two believe Codex is controlled, if a managed worker crosses its parking boundary, or if uninterrupted work lasts no longer than in a notification-only control.

## 5. Disprove before build

Buy the smallest experiment that could reject an architectural choice before buying its implementation.

### A minute on Acquit

At 0 seconds, open "Raise verifier concurrency from 1 to 2" in the choices ledger. The existing risk is 0.36. Instead of Accept, click "What would make this wrong?"

At 10 seconds, the verifier bank displays a temporary second active slot. Select the shared subject-directory concern already listed beside the alternative. Drag two jobs with the same repository identity into the trial tray.

At 20 seconds, choose "Reject if jobs can corrupt each other's subject state." Set the run limit to two and agent spending to $1. Neither action changes the queue-wait target of thirty seconds or authorizes production concurrency.

At 30 seconds, two scout bots submit probe drafts. One proposes a short overlapping checkout workload. The other notes that different repository identities would miss the suspected collision. The engineer approves the discriminating workload and its rejection condition.

At 40 seconds, click "Run sealed trial." The scripted sandbox run shows a cross-job file change. Click that event to see both job identities, file evidence, and the exact environment. The second slot remains a ghost. No full feature branch has been commissioned.

At 55 seconds, choose "Reject this implementation premise." The ledger attaches the counterexample and reopens the alternatives. A separate-stack lane remains possible. The historical confidence value is retained with its basis, not replaced by a scout's self-rating.

### The changed loop

This attacks both pains. The human supplies a rejection criterion. Agents compete to reduce uncertainty, not to produce the most persuasive plan or largest patch.

A failed refutation is inconclusive. It does not earn a green acceptance mark. Trials remain exploratory evidence until the human approves any new acceptance check. Sealed replay cannot contact PayPal or GitHub.

### Data

```ts
type RefutationTrial = {
	choice: ChoiceId;
	hypothesis: string;
	approvedProbe: ArtifactRef;
	rejectWhen: PredicateRef;
	limits: { runs: number; spendUsd: number };
	environment: "sealed-local";
	result:
		| { kind: "pending" }
		| { kind: "refuted"; counterexample: ArtifactRef }
		| { kind: "inconclusive"; observations: ArtifactRef[] };
};
```

### Evidence and novelty

`a-landscape.md`, "Decision experiment bench," supports buying information before construction. `b-games-softvis.md`, "Pareto workshops," supports common workloads rather than rhetorical comparisons. Against `f-frontier.md`, "Predictions that can admit ignorance," this treats an unsupported concurrency prediction as a request for measurement. It differs from model arenas by testing an architectural claim without commissioning competing complete solutions.

### What leaves ui-v1

Replace Accept as the default next step for a costly uncertainty in `c7-review.png`. Alternative cards become possible experiment targets. Simulate's calibration remains visible, including its inability to predict an unmeasured shared-directory race.

### Risk and prototype test

The largest risk is a persuasive but invalid experiment. Ask engineers whether to authorize concurrency two, with one misleading probe and one valid collision witness. Kill the concept if two of eight miss the seeded race, or if review takes longer than the existing choice card without improving the decision. Cheap experiments must actually save mistaken construction.

## Ranking and combination

1. Trace splicing. It addresses the earliest expensive error, an agent implementing a different meaning of the request.
2. Split at the port. It turns boundary agreement into independent progress rather than coordination chat.
3. Reissue the blueprint. It keeps intent corrections effective after parallel construction has started.
4. Book my next decision. It makes unattended time a bounded operating choice rather than optimism.
5. Disprove before build. It prevents expensive commitments, but only when a cheap discriminating experiment exists.

Trace splicing and Split at the port combine best. First, the engineer teaches the desired end-to-end behavior and resolves ambiguous outcomes. Then the engineer publishes only the boundaries needed for independent work. The same approved examples test both fake peers and the composed candidate. This avoids asking the engineer to design every interface before understanding the behavior. It also prevents local contract success from replacing the system-level promise. Together, the human issues one behavioral order and resolves a few seams instead of writing several prompts, relaying answers, and reconstructing intent from several diffs.
