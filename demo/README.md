# Demo take

This folder stages one live take of the Acan demo. Each take clones the demo repository into a fresh run folder, starts the Acan server with the UI, and posts one order. Two worker slots take the tasks from the split. The split can produce three tasks that all change `src/middleware.ts`. When a second task declares that file while the first task is still open, the origin raises a clash and holds the second task until a person settles it. The run then continues through build, gate, and land.

## Run a take

Requires PowerShell 7, Bun, and a built worktree. The keys in `crates/acan-llm/.env` and the routing in `crates/acan-llm/llm.toml` must exist. Both files are git-ignored.

1. Build the UI bundle once per worktree. Run `crates/acan-render-web/build.ps1`.
2. Start the take. Run `pwsh demo/run.ps1`. The script returns after the server, the demo app, and both worker slots are running.
3. Open the UI. Open the URL in `<run folder>\logs\ui-url.txt`. The URL holds the owner token in its fragment, so do not share it.
4. Settle the clash in the UI with 1, 2, or 3. Or run `pwsh demo/watch.ps1 -Settle 3` in another shell to settle each clash with option 3 (serialize) without the UI.
5. Stop the take. Run `pwsh demo/stop.ps1`. It stops only the processes the take started.

The default run folder is `D:\dev\acan-run`. Starting a new take renames the previous folder to `D:\dev\acan-run-prev-<timestamp>`. Nothing is deleted, so the earlier logs stay.

Each worker slot runs `acan-worker` again after every task, because one worker process builds one task and exits. A slot therefore builds every task its agent can claim. `worker-slot.ps1` runs the loop, and each run writes its own log pair, such as `worker-1.2.err.log`.

## Read the result

`demo/watch.ps1` writes three files under `<run folder>\logs`.

- `timeline.txt` lists each origin event with its time from the order post.
- `events.jsonl` holds the raw events from `GET /api/events`.
- `summary.json` holds the outcome, the step times, and the usage totals.

The clash rules in `summary.json` are these.

- `clash_before_write`: the first clash was raised before any task wrote its contested file. This is the rule the demo needs.
- `clash_before_blocked_write`: the first clash was raised before the task it held back, the one that declared second, wrote the file.
- `contested_file`, `blocked_task`, and `declared_ms` name the file, the held task, and each task's declaration time in milliseconds.

The outcome is one of these values.

- `clash_before_write_landed`: the first clash was raised before any write to its file, and every task landed.
- `clash_before_write_not_landed`: the first clash was raised before any write to its file, and some task did not land.
- `clash_after_write`: a task wrote the contested file before the first clash was raised.
- `no_clash_landed` and `no_clash_incomplete`: the split produced no blocking overlap.

The write times come from the worker logs. Each write logs one line when the write returns. The clash times come from the origin.

## Why a clash can be missed

A clash needs the second declaration to arrive while the first task is still open, which means before the first task lands. The first task is open from its declaration until its gate passes and it lands. Its own edit and gate decide that window. The second task's readback and review come after its worker starts, so it can arrive after the first task has already landed. Then the origin sees no overlap and the take has no clash.

## Layout

- `lib.ps1`: shared helpers for the scripts.
- `run.ps1`: clones the demo, writes the tokens, starts the processes, registers the agents, and posts the order.
- `worker-slot.ps1`: runs `acan-worker` for one agent again after each task.
- `watch.ps1`: follows the event log, settles clashes when asked, and writes the result.
- `stop.ps1`: stops the processes recorded in `<run folder>\logs\run.json`.

## Tokens

`run.ps1` creates one owner token and two agent tokens and writes them to `<run folder>\.acan\tokens.json`. The server reads that file at start, so it mints no token of its own. The owner token is never printed to the console.
