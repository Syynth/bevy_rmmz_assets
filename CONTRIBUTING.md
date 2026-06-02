# Contributing

## Workflow

Work is tracked as **issues** on the project board and landed as **one PR per
issue**:

1. Pick (or file) an issue describing the change.
2. Branch from `main` (`feat/…`, `fix/…`, `chore/…`). This repo is a flat
   checkout — use your editor/agent's native git worktrees for parallel work;
   `.claude/worktrees/` is gitignored.
3. Open a PR that closes the issue (`Closes #N`). CI must be green and the PR
   gets an automated review before merging.

## Local checks

CI runs these on Rust **1.96** (pinned in `rust-toolchain.toml`); run them
before pushing:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets -- -D warnings   # default features too
cargo test --all-features
```

The lint profile is strict (pedantic clippy + restriction lints: no
`unwrap`/`expect`/`panic`/`print*` outside tests, etc.). `clippy.toml` relaxes
the `unwrap`/`expect`/`print` rules inside `#[cfg(test)]`. Examples that print
should scope an `#[expect(clippy::print_stdout, …)]`.

## Conventions

- **Edition 2024.** Depend on the individual `bevy_*` sub-crates, not the `bevy`
  umbrella.
- **Data models** mirror the RPG Maker MZ JSON (camelCase via serde) and use
  `#[serde(default)]` for tolerance to version drift across MZ releases.
- **Note parsers** (`NoteParser`) must give a stable, unique `TAG` — it is
  written into baked assets and matched on load.
- Integration tests and examples live in `tests/` and `examples/` and only see
  the crate plus `[dev-dependencies]`.

## Decision log

Architectural decisions and their rationale are recorded in
[`docs/decision-log.md`](docs/decision-log.md). Add an entry when a change
reflects a non-obvious choice worth remembering.
