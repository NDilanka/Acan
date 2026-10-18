# Concurrent same-file editing: prior art for Acan and Kvarm

## Six-line summary

[sourced] Text convergence does not imply correct code; CodeCRDT reports both convergence and semantic conflicts. [Paper](https://arxiv.org/html/2510.18893)  
[sourced] Structural merging reduces reported conflicts, but published accuracy remains imperfect. [IntelliMerge](https://2019.splashcon.org/details/splash-2019-oopsla/53/IntelliMerge-A-Refactoring-Aware-Software-Merging-Technique)  
[sourced] Jujutsu and Pijul preserve unresolved conflicts instead of requiring immediate resolution. [Jujutsu](https://docs.jj-vcs.dev/latest/conflicts/), [Pijul](https://pijul.org/manual/why_pijul.html)  
[sourced] Claude Code and Codex document worktrees as practical parallel-edit isolation. [Claude](https://code.claude.com/docs/en/worktrees), [Codex](https://developers.openai.com/codex/app/worktrees)  
[inferred] Existing agent tool calls need versioned edit transactions, not merely collaborative editor buffers.  
[inferred] Acan's opportunity is cross-harness coordination with explicit conflicts, recoverability, and tested publication.

## Scope and evidence rules

[inferred] The target is several Claude Code, Droid, and Codex processes using one repository. Sharing repository history and sharing writable filesystem bytes are different designs. This report evaluates the latter without assuming it is preferable.

[inferred] Labels cover the claims in their paragraph. “Sourced” means documented or author-reported, not independently replicated here. “Measured” identifies an experiment or observational replay. Suitability and design recommendations are inferences. This is not an implementation audit of `/tmp/kvarm`.

## 1. Text CRDTs, OT, and collaborative IDEs

### The guarantee

[sourced] Text CRDTs merge concurrent insertions and deletions into a convergent sequence. OT instead transforms operations against concurrent operations. Eg-walker's paper describes both approaches and proves its hybrid algorithm correct. [Eg-walker](https://arxiv.org/html/2409.14252v1)

[inferred] This answers “will replicas agree?” It does not answer “will the program parse, compile, pass tests, or preserve both agents' intended behavior?” Two consistent replicas can contain duplicate declarations, incompatible interfaces, or a wrong authorization condition.

### Libraries and research

[sourced] Yjs supplies shared text, arrays, maps, transactions, relative positions, and origin-scoped undo. Its synchronization is network agnostic. Relative positions anchor locations through concurrent edits. Text metadata has storage costs despite compression and garbage-collection optimizations. [Yjs](https://github.com/yjs/yjs)

[sourced] Automerge preserves concurrent sequence insertions and deletions with deterministic ordering. Concurrent assignments to the same property select a deterministic visible value while retaining alternatives through `getConflicts`. That is different from presenting every incompatible edit as a blocking merge conflict. [Automerge conflicts](https://automerge.org/docs/reference/documents/conflicts/)

[sourced] Loro combines text, movable lists, maps, trees, and version history. Its documentation describes Fugue for minimizing text interleaving and Eg-walker for reconstructing merge metadata from edit history. The library still leaves transport, authorization, and persistence integration to applications. [Loro](https://loro.dev/), [algorithm description](https://loro.dev/blog/loro-richtext)

[sourced] Peritext addresses inline formatting intent, including annotations attached to character ranges. It guarantees convergence while explicitly acknowledging that not all author intentions can coexist. Its original scope excludes complete document collaboration and block-level structures. [Peritext](https://www.inkandswitch.com/peritext/)

[inferred] Peritext suggests how Acan could anchor agent provenance or review annotations. It is not a semantic code merger. Likewise, preventing character interleaving does not reconcile two incompatible function implementations.

[sourced] Eg-walker records an event graph and temporarily reconstructs CRDT state for concurrent merges. Measured against selected implementations and editing traces, it reports substantially lower steady-state memory and faster loading. One divergent document took about one hour with the tested OT implementation and 24 milliseconds with Eg-walker. These are text-processing results, not agent throughput measurements. [Paper](https://arxiv.org/html/2409.14252v1)

### Zed and Live Share

[sourced] Zed makes CRDTs its buffer representation. Logical anchors survive concurrent edits. Its collaboration model shares projects, cursors, following, editing, saving, and language-server access. This is a collaborative authoring environment, not merely synchronized files. [CRDT design](https://zed.dev/blog/crdts), [project collaboration](https://zed.dev/features)

[sourced] VS Code Live Share similarly exposes real-time coediting, selections, independent navigation, following, and optional read-only access. Saved edits persist on the host. The cited product documentation does not specify a public correctness proof or justify assuming a particular OT implementation. [Live Share](https://learn.microsoft.com/en-us/visualstudio/liveshare/use/coedit-follow-focus-visual-studio-code)

### Cost and tool-call fit

[inferred] These systems fit agents only when all mutations enter their operation model. Translate a string replacement against the revision the agent actually read. Apply a patch as a transaction with a recorded base. Treat whole-file writes as replacements requiring reconciliation, not harmless save events.

[inferred] A filesystem watcher observes results too late to recover overwritten intent reliably. A CRDT transaction groups operations but is not serializable isolation across agents' reasoning. Supporting shell writes, formatters, generated files, and UTF-16 versus byte offsets adds integration cost beyond embedding Yjs.

## 2. Structured and semantic merge

[sourced] Mergiraf parses three revisions with tree-sitter, matches nodes using GumTree, and merges structural relationships. It checks delete/modify cases and duplicate signatures. Parsing errors cause fallback to line merging. [Architecture](https://mergiraf.org/architecture.html)

[sourced] Its language configuration identifies contexts where ordering is treated as unimportant. It merges independent fields but leaves incompatible statement ordering unresolved. The documentation acknowledges that even Java declaration ordering can matter through reflection. [Examples and limitations](https://mergiraf.org/conflicts.html)

[sourced] GumTree is an AST differencing and matching building block, not a complete concurrent repository protocol. Its edit operations include insertion, removal, update, and move. A measured tuning study improved edit-script length in 21.8% of evaluated cases. That is not a conflict-reduction percentage. [AST differencing study](https://arxiv.org/pdf/2011.10268)

[sourced] SemanticMerge, associated with Plastic SCM, parses language-dependent code trees to handle movement and refactoring. Its introductory guide describes C#, Visual Basic, and Java support and integration with Git and other VCSs. These are product capabilities and examples, not an independent conflict-rate experiment. [SemanticMerge guide](https://www.semanticmerge.com/documentation/intro-guide/semanticmerge-intro-guide)

[sourced] IntelliMerge builds a refactoring-aware graph for Java. Measured on 1,070 scenarios from ten projects, it reported 58.90% fewer conflicts than GitMerge and 11.84% fewer than jFSTMerge. Auto-merging precision was 88.48%, recall 90.22%, and median processing time 539 milliseconds. [OOPSLA results](https://2019.splashcon.org/details/splash-2019-oopsla/53/IntelliMerge-A-Refactoring-Aware-Software-Merging-Technique)

[sourced] Spork combines Java AST merging with formatting preservation. Its 1,740-file benchmark included 255 comparable file merges where at least one tool reported conflicts. Spork produced 227 conflict hunks versus jDime's 376, a 40% reduction. Spork also crashed on 34 file merges; jDime had 16 timeouts and seven crashes. [Spork, section V](https://arxiv.org/html/2202.05329v1)

[sourced] jDime's auto-tuning switches between unstructured and structured merging. An earlier measured study covered eight Java projects and 72 scenarios. Auto-tuning was five times faster on average than fully structured merge, up to twelve times faster. That compares execution strategies, not correctness against Git. [jDime](https://www.se.cs.uni-saarland.de/projects/jdime/)

[sourced] Tree-sitter-based LastMerge and Mergiraf were evaluated on 5,229 Java scenarios. LastMerge reported 15% fewer added false positives than jDime; Mergiraf reported 42% fewer added false negatives than Spork. These are relative disagreement metrics, not absolute error rates or universal conflict reduction. The oracle combines historical merge outputs, structural comparison, and tests; scenarios with tool crashes were excluded. [LastMerge paper](https://arxiv.org/pdf/2507.19687)

[sourced] Difftastic uses tree-sitter for structural comparison. It explicitly does not implement AST merging or emit applicable patches. It can visualize files containing conflict markers. [Difftastic](https://github.com/Wilfred/difftastic)

[inferred] Snapshot-based mergers fit agent patch and whole-file tools better than keystroke protocols. Their costs include parsing, heuristic node identity, language configuration, formatting preservation, and fallback behavior. They need the original base revision, not just the latest file.

[inferred] Structural alignment can follow a moved method, but cannot establish that its new callers still obey its contract. Independent AST nodes can interact semantically across files. Fewer markers can mean better reconciliation or hidden errors. Acan should retain alternatives and attach validation results to the exact merged snapshot.

## 3. Patch theory and first-class conflicts

[sourced] Darcs models changes as patches with context, inversion, and commutation. It supports line hunks and token-replacement patches. Token replacement can commute with edits that textual replacement would unnecessarily entangle. [Patch theory](https://darcs.net/Theory/MergersDocumentation)

[sourced] Historical Darcs formats suffered exponential merge behavior. Its performance FAQ distinguishes Darcs 1 from improvements in Darcs 2. This is a documented historical failure mode, not evidence that every current Darcs merge is slow. [Performance FAQ](https://darcs.net/FAQ/Performance)

[sourced] Pijul represents changes and dependencies so independent changes can be applied in either order. Conflicts remain in the internal representation; both sides can be applied before resolution. Its manual explicitly says this does not guarantee intended program semantics. [Pijul](https://pijul.org/manual/why_pijul.html)

[sourced] Jujutsu stores logical conflicts in commits, not merely conflict-marker strings. Conflicted commits can be rebased, merged, and resolved later. Its operation log stores repository views and supports undo and divergent-operation reconciliation. [Conflicts](https://docs.jj-vcs.dev/latest/conflicts/), [operation log](https://jj-vcs.github.io/jj/latest/operation-log)

[sourced] Jujutsu's concurrency documentation distinguishes repository safety from working-copy state. Concurrent repository copies can leave a working copy mismatched. It also documents backend and distributed-filesystem caveats. Repository-operation concurrency is not a promise that arbitrary editors can safely overwrite one file. [Concurrency](https://jj-vcs.github.io/jj/latest/technical/concurrency)

[sourced] Git `rerere` records manual conflict resolutions and reuses applicable resolutions on recurring conflicts. It neither prevents the initial conflict nor proves the reused result semantically correct. [Git documentation](https://git-scm.com/docs/git-rerere)

[inferred] These ideas fit Kvarm's server boundary particularly well: capture agent proposals, preserve bases and alternatives, and expose conflicts as durable objects. Cost moves into history storage, reconciliation UX, Git interoperability, and validation. They cannot recover an edit overwritten before ingestion.

## 4. Code as structure rather than files

[sourced] Unison identifies definitions by hashes of their syntax trees, with dependencies referenced by hash. Names are separate metadata. Changing a name does not mutate the referenced definition. [Unison's model](https://www.unison-lang.org/docs/the-big-idea/)

[inferred] This removes mutable file placement as the identity of code. Concurrent replacement definitions can coexist. Choosing the desired namespace and updating dependent behavior still needs policy. Importing this model into ordinary TypeScript or Rust repositories would require a substantial representation and tooling layer.

[sourced] JetBrains MPS edits and versions an AST directly. Nodes have unique identifiers and explicit references. Its projectional editor renders that structure instead of repeatedly parsing a character stream. [MPS FAQ](https://www.jetbrains.com/help/mps/mps-faq.html)

[inferred] Stable node identities are attractive coordination units. However, structural well-formedness is weaker than semantic compatibility. Existing string-replace agents would need model operations or a reliable textual projection bridge.

[sourced] Darklang's documented Fluid editor transforms ASTs and sends edit operations to a server, which broadcasts changes to other clients. Its structured editor represents missing expressions as blanks with `Incomplete` values. These documents describe an editing architecture, not a proven general-purpose concurrent merge algorithm. [Edit pipeline](https://docs.darklang.com/contributing/general-concepts), [structured editing](https://docs.darklang.com/discussion/structured-editing)

[sourced] Smalltalk provides image-based development, but teams need not share one mutable image. A practitioner account describes separate developer images, method-level change recording, and Monticello package exchange. Pharo's Iceberg documentation separately explains Git-backed image development. [Team workflow account](https://stackoverflow.com/questions/8244502/version-control-for-smalltalk-seaside), [Iceberg book](https://books.pharo.org/booklet-ManageCode/pdf/2019-03-24-ManageCode.pdf)

[sourced] Hazel gives incomplete programs static and dynamic meaning through typed holes. Holes represent missing terms and membranes around erroneous or conflicted terms. This permits feedback on meaningful program fragments without requiring a complete program. [Hazel](https://hazel.org/)

[inferred] Hazel supplies a better model for unresolved work than poisoning an entire source file with markers. It is not a drop-in compiler feature for existing repositories. Across this family, the migration cost is high: new identities, storage, language services, and agent tools.

## 5. Ownership, locking, leases, and build targets

[sourced] Perforce's `+l` exclusive-open modifier prevents multiple users opening a file for edit. Ordinary `p4 lock` instead allows multiple editors but reserves submission for the lock holder. [Perforce](https://help.perforce.com/helix-core/server-apps/p4guide/current/Content/P4Guide/resolve.lock.exclusive.html)

[sourced] Google's `OWNERS` files assign approval rights over repository subtrees. They govern review and submission, not exclusive live editing. [Software Engineering at Google](https://abseil.io/resources/swe-book/html/ch09.html)

[sourced] Symbol-adjacent locking has research precedent. CoEclipse's dependency-based automatic locking protects selected code regions and related regions. Later hybrid work distinguishes strongly dependent regions requiring locks from weak dependencies receiving awareness highlights. [DAL](https://dl.acm.org/doi/10.1145/2340416.2340420), [hybrid approach](https://doi.org/10.1007/978-3-030-92638-0_7)

[inferred] Expiring symbol leases are a practical adaptation, not an invention of ownership itself. Correct enforcement needs stable identities, fencing tokens, renewal, crash recovery, and atomic acquisition across multi-symbol edits. Advisory prompts alone do not stop shell writes. Dependency-aware leases risk serializing broad portions of the repository.

[sourced] Bazel and Buck coordinate builds through targets and dependency graphs. Targets describe buildable units and their inputs, not mutually exclusive source-edit ownership. [Bazel](https://bazel.build/concepts/dependencies), [Buck](https://buck2.build/docs/concepts/glossary/)

[inferred] Target boundaries can scope tasks and select affected checks. They cannot prove edit independence. The cost is reduced concurrency around hot symbols, plus graph maintenance and waiting. File locks fit ordinary tools easily; symbol leases need write interception and structural mapping.

## 6. Parallel LLM agents: measured evidence versus claims

### CodeCRDT

[sourced] CodeCRDT is direct prior art. It uses Yjs text, a TODO-assignment map, an audit array, and a Hocuspocus relay. An outliner creates TypeScript/React skeletons. Implementation agents observe shared updates and fill TODOs through tools. [Paper](https://arxiv.org/html/2510.18893)

[sourced] Measured across 600 trials, six tasks, and 50 runs per mode using Claude Sonnet 4.5, reported outcomes range from 21.1% speedup to 39.4% slowdown. The paper reports complete text convergence. Its 5–10% semantic-conflict estimate is preliminary inspection. Quality scoring is LLM-based without a human-validation baseline. [Evaluation](https://arxiv.org/html/2510.18893)

[sourced] Its claim protocol writes an assignment, waits 50 milliseconds, then rereads. The safety argument assumes verification observes the converged state. Crashed claims are reclaimed using a timeout. [Protocol and appendix A.5](https://arxiv.org/html/2510.18893)

[inferred] A fixed delay cannot guarantee mutual exclusion under unbounded message delay. Two partitioned agents can each read their own assignment and proceed before eventual convergence. Deterministic final ownership is weaker than at-most-one active worker. Acan should not copy this as a fenced lease protocol.

### Products and integration measurements

[sourced] Claude Code recommends worktrees when agents touch the same files and file partitioning for non-isolated agent teams. Codex documents independent worktrees and warns about parallel write-heavy workflows. These are product recommendations, not measured conflict-elimination rates. [Claude approaches](https://code.claude.com/docs/en/agents), [Codex subagents](https://developers.openai.com/codex/subagents)

[sourced] Cursor reports coordination-file locks becoming bottlenecks and being mishandled by agents. Its later swarm account describes a custom VCS, neutral conflict-resolution agents, and blocking commits while oversized files are decomposed. It reports over 70,000 conflicts in an old run versus fewer than 1,000 in a new run. These are vendor-reported system experiments, not controlled attribution to one mechanism. [Initial experiment](https://cursor.com/blog/scaling-agents), [later system](https://cursor.com/blog/agent-swarm-model-economics)

[sourced] AgenticFlict simulated integration for 107,026 agent-authored PRs and found 29,609 with textual conflicts, 27.67%. Its corpus covers open or closed-unmerged PRs. This is a dataset measurement, not an experiment with agents concurrently writing one checkout. [Dataset paper](https://arxiv.org/html/2604.03551v1)

[sourced] Another observational study replayed 747 co-active PR pairs and reported 41.7% textual conflicts for cross-agent pairs versus 19.8% for same-agent pairs. Temporal PR overlap does not establish simultaneous tool execution. [Study](https://arxiv.org/html/2607.04697v2)

[inferred] None of these percentages directly predicts Acan's conflict rate. Worktree integration, toy shared-buffer generation, and uncontrolled PR histories measure different populations. Observation also costs context and tokens, while worktrees cost environment setup and later integration.

## Comparison

[inferred] This table synthesizes the cited evidence. Guarantees are deliberately narrower than product ambitions. “Semantic safety” means preserving intended combined behavior.

| Family | Unit | Guarantee | Conflict handling | Semantic safety | Agent fit |
|---|---|---|---|---|---|
| Text CRDT / OT | Character operation | Replica convergence | Deterministic transformation or merge | None by itself | Requires versioned operation adapter |
| Zed / Live Share | Buffer and project session | Collaborative editing | Shared state and awareness | No behavioral guarantee | Good inside editor protocol |
| Structural merge | AST/CST node | Heuristic structural reconciliation | Merge or retain markers | Partial syntactic knowledge | Strong fit for based patches |
| Patch theory | Hunk, token, change dependency | Algebraic change composition | Explicit incompatible alternatives | No behavioral guarantee | Strong after ingestion |
| Jujutsu / rerere | Commit conflict / hunk resolution | Durable state / resolution reuse | Defer or replay resolution | No behavioral guarantee | Useful server primitives |
| Unison | Definition hash | Immutable definition identity | Preserve versions; reconcile names | Identity, not intent | Requires language-aware bridge |
| MPS / Dark / Hazel | Node, expression, hole | Structured or meaningful partial state | Model operations and holes | Language-specific, limited | Poor fit without new tools |
| Locks / leases | File, symbol, dependency region | Exclusion if correctly enforced | Wait, reject, expire | Not across unlocked dependencies | Easy by file; harder by symbol |
| OWNERS / build graphs | Path / target | Approval / build dependencies | Review and checks | No edit isolation | Scheduling and validation aids |
| Agent worktrees | Checkout and patch | Filesystem separation | Integrate afterward | No merged-behavior guarantee | Best compatibility baseline |

## What nobody has built yet

[inferred] This heading identifies gaps in the reviewed evidence, not a universal novelty claim. CodeCRDT and Cursor already combine agents with shared-state or specialized VCS coordination. A new CRDT editor alone would not be novel.

[inferred] The missing evaluated package is a local, cross-harness transaction layer for Claude Code, Droid, and Codex. It should accept string replacement, patch, and whole-file proposals against explicit read revisions. Shell and formatter writes need equivalent enforcement or isolation.

[inferred] A second gap is separating speculative collaboration from tested publication. Preserve every proposal and conflict durably. Publish a snapshot only with checks tied to its content hash. Keep test runners isolated from transient mixed files.

[inferred] A third gap is dependency-aware coordination without pretending to prove semantics. Combine short fenced leases, structural merge, affected-target checks, and escalation. Report whether a change was excluded, auto-merged, tested, or merely accepted.

[inferred] A fourth gap is causal feedback for stale agents. Explain which read assumptions changed, which symbols moved, and which competing intent survived. Do not stream every character into every context. Measure useful invalidation versus notification overload.

[inferred] Finally, the field needs a reproducible mixed-agent benchmark with identical tasks, overlap, and budgets. Compare shared checkout, worktrees, leases, CRDTs, and structural merge. Measure lost edits, undetected semantic failures, integration time, token cost, recovery, and final tests. No reviewed study establishes that complete comparison.
