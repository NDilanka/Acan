# acan-core design

`acan-core` is Acan's world model. It holds the rules for orders, readbacks, claims, declarations, clashes, gates and landing. The `acan` server and the browser renderer both use it. It does no I/O, reads no clock, and builds for `wasm32-unknown-unknown`.

The tests in `tests/` pin the rules below: one file per domain, plus a determinism test that replays a full scripted loop through a replica.

## The model

Each world has one append-only event log. The **origin** is the only process that appends to it. Everyone else (the browser, a desktop joined to a team origin, the hosted replay) keeps a **replica** that folds the same events with the same code.

- A solo desktop runs the origin in its own process.
- A team origin is the same `acan` binary on a shared host.
- A joined desktop sends `HumanCmd` over HTTP and receives `Stamped` events over WebSocket. It does not act while its origin is unreachable.

The server stores each `Stamped` event as JSON and streams the same JSON. A client resumes from `Replica::next_seq`.

Model calls, witness runs and merges are **jobs**. `World::jobs` derives them from state, so a crashed server restarts them by asking again. The server runs each job and reports the result as a `SystemCmd`. A result for a job that is no longer derived is rejected.

Clash detection is the pure function `detect`. The origin runs it inside `execute` and records the blocking results as `ClashFound` and `ClashDissolved` events. Replicas never run detection and never need the code index.

## Entry points

| Item | Who calls it | What it does |
| --- | --- | --- |
| `Origin::open`, `Origin::replay` | server | Start a world, or rebuild it from the stored log. |
| `Origin::execute(&Ctx, Command)` | server | Decide a command. Returns the stamped events to persist and broadcast, or a `Rejection`. A rejected command changes nothing. |
| `Replica::from_log`, `Replica::ingest` | browser, joined desktop | Fold a fetched log, then apply streamed events. Reports `ReplayError::SeqGap` when events were missed. |
| `World` read methods | everyone | `order`, `tasks_of`, `clashes_for`, `phase`, `land_queue`, `jobs`, `settle_right`, `usage`. |
| `detect(&[&Declaration], &CodeIndex)` | origin, tests | Pure overlap classification. |
| `Command`, `HumanCmd`, `AgentCmd`, `SystemCmd` | server, MCP adapter | The write vocabulary. `HumanCmd` is the HTTP body; `AgentCmd` is the MCP tool arguments. |

```rust
use acan_core::{AgentCmd, Command, Ctx, Origin, Rejection, Stamped, Timestamp};

// Server write path: one lock around the origin, persist before acknowledging.
fn claim(
    origin: &mut Origin,
    index: &acan_core::CodeIndex,
    now: Timestamp,
    agent: acan_core::AgentId,
    task: acan_core::TaskId,
) -> Result<Vec<Stamped>, Rejection> {
    let ctx = Ctx { now, index };
    let events = origin.execute(&ctx, Command::Agent { agent, cmd: AgentCmd::Claim { task } })?;
    // store.append(&events)?; hub.broadcast(&events);
    Ok(events)
}

// Browser or joined desktop: follow the stream.
fn follow(replica: &mut acan_core::Replica, streamed: &[Stamped]) -> Result<(), acan_core::ReplayError> {
    for event in streamed {
        replica.ingest(event)?; // on SeqGap, refetch from replica.next_seq()
    }
    Ok(())
}
```

`Command` is not deserializable. The server builds it from an authenticated session, so a request can never choose its own principal, and `SystemCmd` can never arrive over the wire.

## Modules

| Module | Owns |
| --- | --- |
| `ids` | Typed ids. Counter ids (`OrderId`, `TaskId`, ...) are minted by the origin. Names (`MemberId`, `FilePath`, `CodeRev`, ...) are validated once at the boundary. `Seq`, `OrderRev` and `Generation` are distinct types. |
| `event` | `Command`, `Event`, `Stamped`, `Rejection`, `ReplayError`. This is the stored and streamed schema. |
| `world` | `World` (the fold), `Origin`, `Replica`, `Ctx`, derived reads, and `execute` dispatch. |
| `order` | Orders, witnesses, tasks, readbacks, questions, and their decisions. |
| `lease` | Leases, fences, claim, renew, release and expiry. |
| `clash` | Declarations, `detect`, clashes, settlement menus, and the build-clear rule. |
| `gate` | Gate runs, verdicts and the land queue. |
| `member` | Members, agent principals, and `SettleRight`. |
| `index` | `CodeIndex`, the input to detection. |
| `job` | `Job`, the work the server derives and runs. |

Each domain module owns both its decide functions and the state they change. Every module is private; `lib.rs` re-exports the public surface.

## State machines

**Task.** A claim cannot exist without both a lease and the readback it was granted on (`Claim { lease, readback }`).

```text
Open --LeaseGranted--> Claimed{Declaring}
Claimed --LeaseEnded(any)--> Open
Claimed{Declaring|Declared|Building} --Declared--> Claimed{Declared}
Claimed{Declared} --BuildCleared--> Claimed{Building}
Claimed{Building} --BuildReported--> Claimed{Gating}
Claimed{Gating} --GateClosed(Failed)--> Claimed{Building}
Claimed{Gating} --GateClosed(Passed)--> Queued        (lease ends as Delivered)
Queued --Landed--> Landed
Queued --LandFailed--> Open
any but Landed --OrderCancelled--> Dropped
```

**Lease.** Every grant bumps the task's generation, which never resets. Every fenced write whose generation is not the live one is rejected with no event.

```text
Open --Claim: registered agent, readback matched at current revision, no open question--> Claimed
Claimed --Renew(fence)--> Claimed, same generation, later expiry
Claimed --Release | expiry | question answered | gate passed | order cancelled--> lease ended
```

A lease does not expire while its task is gating, because the worker is waiting on witnesses it does not run. A failed gate renews the lease from the moment it closes.

**Clash.**

```text
(none) --ClashFound--> Open
Open --SettleClash, member may only propose--> Proposed
Open|Proposed --SettleClash, member may settle--> Settled
Open|Proposed --redetection no longer finds the overlap--> Dissolved
```

The owner settles any clash. A member settles a choice whose yielding task is held by one of their own agents, and may only propose otherwise.

A blocking clash dissolves when either party stops holding a declaration (its lease ends, it narrows, or its order is cancelled). A `Contract` clash records which task owns the interface (`Clash::contract`), because its menu offers to split at that interface.

**Order phase** is derived by `World::phase`, not stored. The order stores only `Splitting`, `Split` or `Cancelled`. Everything after the split comes from its tasks, its open question and its clashes, so the phase cannot drift from them.

**Gate verdict** is derived from a run's results. A witness that errors never counts as a pass. A run's witness set is frozen when it starts.

## The code index contract

Detection reads a `CodeIndex` that the server builds from its scanner.

- `rev: CodeRev` is an opaque content revision. Equal revisions must mean equal contents.
- `files` maps each repo-relative `FilePath` to its machine and its symbols.
- Each symbol has a name, a half-open byte range in the file at `rev`, and whether it is exported.
- `belts` lists import edges between machines. `from -> to` means `from` imports `to`.

Declarations name symbols by file and name, not by range, so they survive a rescan. A symbol that does not resolve counts as a whole-file write.

## What the crate does not do

It does not open sockets, touch git, run processes, call models, read the clock, or store anything. The server supplies `now` and the index in `Ctx`, owns the event table, runs jobs, and logs its own outbound traffic.
