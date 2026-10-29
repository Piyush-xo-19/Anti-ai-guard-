# Contributing to Anti-AI Guard

The project is implementing M2 in small chunks; M2.1 provides the CLI foundation and M2.2 adds [shared contracts and diagnostic evaluation](docs/shared-contracts.md), and M2.3 adds [bounded Windows installation inventory](docs/installation-inventory.md). Start with the [README](README.md), [public status](PROJECT_STATUS.md), [decision log](docs/decisions.md), [M2 plan](docs/m2-plan.md), and [development instructions](docs/development.md). Agree on a bounded outcome before implementing a subsystem.

## One reviewable chunk at a time

For each change, record:

- **Problem:** what concrete gap does this address?
- **Scope:** what will this chunk deliver?
- **Acceptance:** how will we know it works?
- **Validation:** which checks were run, and on which platform?
- **Limits:** which supported cases remain incomplete or untested?

Keep unrelated refactors and speculative features out of the same change. Use a short-lived branch and a pull request for collaborative work. A small foundational documentation change may go directly to `main` when the maintainer authorizes it.

## Commit convention

Use a short imperative subject with a prefix:

| Prefix | Use |
| --- | --- |
| `docs:` | Documentation and design decisions |
| `feat:` | A working capability |
| `fix:` | A behavior correction |
| `test:` | Meaningful acceptance or regression coverage |
| `chore:` | Repository or build maintenance |

Make a commit when a coherent chunk has passed its relevant checks. Push completed work and explain any remaining gaps. Never claim that an untested collector supports an operating-system configuration.

## Policy and privacy are part of correctness

- Installed inactive tools are inventory findings, not execution violations.
- Separate ambiguous identity evidence from confirmed supported identity.
- Report missing permissions, collector failures, and event loss explicitly.
- Preserve normal terminal, compiler, package-manager, and legitimate AI-development use.
- Do not add automatic process termination, uninstall, file deletion, or disqualification.
- Use fixtures and synthetic data. Never commit real participant telemetry or raw command lines.
- Get an architecture decision reviewed before changing a trust boundary, privilege requirement, data-collection scope, or event policy.

## Validation

For documentation changes, check links, terminology, Markdown structure, and whether claims match the actual implementation stage.

For Rust changes, run the formatting, strict Clippy, locked build and tests in the [development instructions](docs/development.md). Build/run and test each feature before pushing; independent testers can then open reproducible issues. Cosmetic and non-blocking issues may be scheduled later. Build failures, data-loss/privacy risks, incorrect violation reports and bugs that block dependent work must be resolved before building on them. OS collector changes require platform-specific evidence; unit tests alone do not establish Windows/Linux telemetry coverage.

## Maintainer's local progress tracker

The maintainer uses `PROJECT_STATUS.local.md` for detailed progress and handoff notes. It is ignored by Git and must not be staged or force-added. Contributors do not need that file or any private chat context. Public progress belongs in [PROJECT_STATUS.md](PROJECT_STATUS.md), issues, pull requests and commits; decisions belong in the [decision log](docs/decisions.md).

Update the local tracker after each chunk with completed work, current work, next steps, unresolved decisions, validation, and commit references. Check that it remains excluded before every commit.

When completing a milestone chunk or accepting a design decision, update the relevant public status/decision/plan document in the same focused change. Include checks and known gaps. Do not mark proposed behavior as implemented or unexecuted tests as passed.

## License

Contributions are made under this repository's [MIT license](LICENSE). Avoid adding third-party material without compatible permission and attribution.
