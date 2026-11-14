# Test cases and run records

Start here to see what is being tested, how to reproduce it, and what remains
open. The [acceptance plan](../acceptance-plan.md#4-acceptance-cases) defines the
expected behavior and the original case IDs. The [M2 case index](m2.md) maps work
to those IDs, executable tests, and evidence. The [M3 plan](../m3-plan.md)
describes future cases; runtime trials have not started.

## Latest runs

| Date | Run | Outcome |
| --- | --- | --- |
| 2026-11-14 | [Windows 11 restart and desktop checks](runs/2026-11-14-windows-11.md) | Restart, default suite and pinned tool subcases PASS; benign Node/npm BLOCKED by partial coverage |
| 2026-11-06 | [Four package launchers and benign Node/npm](../validation/launcher-regression-2026-11-06.json) | Five passing subcases on the earlier Windows 11 build |

Earlier trials remain in the [validation archive](../validation/README.md).
Executable integration tests stay in [`tests/`](../../tests/), and Rust unit
tests stay beside the code they exercise. This folder holds the human-readable
case index and dated run records.

## How to maintain this folder

For each change, add or update the case in its milestone index. Reuse the
acceptance ID where one exists; give distinct variants their own subcase labels.
List the setup, command or test entry point, and expected outcome. Mark an
active check as **IN PROGRESS** until it has a recorded outcome, then link its
dated run in [`runs/`](runs/).

For each run, record its date, exact revision, whether the checkout was clean,
OS release/build and inspection mode when relevant, expected and actual
results, and what the evidence covers. Use **PASS**, **FAIL**, **BLOCKED**,
**NOT RUN**, or **NOT APPLICABLE** with a reason. A partial subcase or fixture
pass does not close its parent desktop row. Add a new run when rerunning a case
so earlier results remain traceable to their original build and source.

Publish only synthetic inputs and minimized evidence. Raw reports, logs,
participant paths/arguments and private checkpoint data stay outside Git.
