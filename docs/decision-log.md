# Decision Log

This file captures decisions made during development of `bevy_rmmz_assets` —
direction, preferences, and rationale that should inform future work. It is a
data-capture mechanism, not a findings or spec document.

Each entry uses the format:

```
## <short description>
- **WHEN:** <YYYY-MM-DD>
- **PROJECT:** <project/repo name>
- **SYSTEM:** <short tag>
- **SCOPE:** <minor/local | moderate | architectural>
- **WHAT:** <what was decided>
- **WHY:** <rationale — the most important field>
```

---

## Target Bevy 0.19.0-rc.2 via individual sub-crates
- **WHEN:** 2026-05-31
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** cross-system
- **SCOPE:** architectural
- **WHAT:** Target the latest Bevy release candidate (`0.19.0-rc.2`) and depend
  only on the specific sub-crates needed (`bevy_app`, `bevy_asset`, `bevy_ecs`,
  `bevy_reflect`) rather than the `bevy` umbrella crate.
- **WHY:** This is a standalone, reusable, eventually-publishable crate; tracking
  the newest released line keeps it current. Depending on individual sub-crates
  keeps the dependency surface minimal and avoids pulling in renderer/windowing
  features an asset-loading library does not need.

## Layered architecture: assets first, resources on top
- **WHEN:** 2026-05-31
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** cross-system
- **SCOPE:** architectural
- **WHAT:** Database files load into strongly-typed assets as the source of
  truth. Asset loaders and asset types are kept agnostic. A resource layer builds
  indices/lookups on top of the assets. A config resource (plus an `App`
  extension trait offering convenience initializers) determines what is loaded
  and how.
- **WHY:** Keeping loaders/asset types agnostic and pushing policy into a config
  resource makes loading strategies (e.g. how maps are handled) pluggable without
  changing the asset/loader layer. The resource layer gives consumers ergonomic,
  efficient access without coupling them to asset-handle plumbing.

## Maps are optional, not the primary use case
- **WHEN:** 2026-05-31
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** maps
- **SCOPE:** moderate
- **WHAT:** `Map###.json` loading is opt-in (gated behind a `maps` feature and
  configured via the config resource). The asset/loader layer stays agnostic; map
  loading strategy is a config concern.
- **WHY:** The user's primary use case is the core database, not maps. Maps are
  numerous and large, so loading them must be optional and strategy-driven to
  avoid imposing memory/startup costs on consumers who don't need them.

## Flat repo + Claude desktop native worktrees (drop manual worktrees)
- **WHEN:** 2026-05-31
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** process
- **SCOPE:** moderate
- **WHAT:** Use a single flat repository checkout and rely on the Claude desktop
  app's native worktree support, instead of the manual `main` + sibling-worktree
  directory layout used in the `folklore` project.
- **WHY:** The manual worktree layout in folklore predated native worktree support
  in the desktop app. Native worktrees only require a git repo, accepted workspace
  trust, and `.claude/worktrees/` gitignored — simpler than managing worktree
  directories by hand.

## Folklore-style GitHub workflow (project board + labels + issue/PR per task)
- **WHEN:** 2026-05-31
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** process
- **SCOPE:** moderate
- **WHAT:** Adopt the `folklore` project's GitHub workflow: a private Project v2
  board (Status options Todo / In Progress / Done / Icebox), a custom label
  taxonomy, and one Issue → branch → PR per task. The repo itself is public.
- **WHY:** The user wants to reuse an established, working process for tracking and
  executing work in discrete, reviewable units.

## Rust 1.96 toolchain
- **WHEN:** 2026-05-31
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** cross-system
- **SCOPE:** minor/local
- **WHAT:** Pin the toolchain to Rust `1.96` (just released) via
  `rust-toolchain.toml`.
- **WHY:** Start the new crate on the current stable toolchain rather than the
  slightly older 1.95 used elsewhere.

## CodeRabbit as automated PR reviewer
- **WHEN:** 2026-05-31
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** process
- **SCOPE:** moderate
- **WHAT:** Adopt CodeRabbit (free public-repo tier) as the automated PR reviewer,
  configured via `.coderabbit.yaml`: assertive profile, request-changes off (CI
  remains the merge gate), and Rust path-instructions steering it toward
  correctness/API/serde-fidelity rather than lint/format nits already enforced by
  rustfmt + strict clippy.
- **WHY:** Free for public repos with no approval gate and minimal setup; adds
  design/logic review that static lint can't, while avoiding duplicate noise on
  what CI already enforces.

## Data models tolerate version drift via serde(default)
- **WHEN:** 2026-05-31
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** data-model
- **SCOPE:** moderate
- **WHAT:** Apply `#[serde(default)]` + `Default` uniformly across the whole data
  layer so database records still deserialize when fields are missing, rather than
  modeling files strictly. Fixed-size arrays (e.g. `params: [i32; 8]`) keep their
  length check when the field is present.
- **WHY:** Validating the models against a real, long-lived MZ project (deserialized
  its full `data/` — all standard files + 23 maps) revealed version drift: older
  `States.json` entries omit fields added in later engine versions (e.g.
  `releaseByDamage`), which broke strict structs. Real projects upgraded across MZ
  versions are not field-uniform, so tolerance is required for the loader to be
  usable on actual games. Unknown/extra fields are already ignored by serde; this
  covers the missing-field direction.

## Custom/third-party asset types as first-class peers
- **WHEN:** 2026-06-02
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** assets / cross-system
- **SCOPE:** architectural
- **WHAT:** Treat support for consumer/third-party custom asset types and data
  tables (as introduced by RMMZ community plugins) as a first-class design goal —
  a game should be able to register its own tables and have them behave almost as
  peers of the built-in core tables (typed DB access, status aggregation, note
  cache/baking), not merely hand-load them. Design tracked in issue #37.
- **WHY:** Real MZ projects routinely add plugin-specific data files/shapes; if
  the crate only supports the fixed built-in set, those projects can't reuse the
  resource/notes/processing machinery for their own data. Designing for
  extensibility before the table surface ossifies at 0.1 avoids a breaking
  redesign later (and collapses the current ~8-site edit burden for adding a
  table).

## Release memory: own custom assets in the snapshot when `file_watcher` is off
- **WHEN:** 2026-06-02
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** assets / snapshot
- **SCOPE:** moderate
- **WHAT:** When the `file_watcher` feature is **off** (compile-time), custom
  assets are *moved* (not cloned) out of `Assets<A>` into the `RmmzAssets`
  snapshot and their handle is dropped, so release builds store custom data once
  rather than twice. When `file_watcher` is on (dev), keep the handle + clone so
  hot-reload still works. This ships **as part of Phase 4** (alongside baking),
  not a separate pass.
- **WHY:** Custom access goes only through the snapshot, so the `Assets<A>` copy
  is dead weight at runtime; freeing it halves memory for large custom data.
  Gating on `file_watcher`-off ties it to "no hot-reload needed" without adding a
  separate config flag (user's call, over an explicit opt-in). Moving instead of
  cloning avoids even a transient 2× during load. Requires an `owned` flag on the
  registry entry (so `status()` still reports `Loaded` after the handle is
  dropped) and ordering note-caching before the move.

## Load-state reporting: misuse-resistant `ready()` + settle latch
- **WHEN:** 2026-06-02
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** resource (RmmzDatabase)
- **SCOPE:** moderate
- **WHAT:** Add `RmmzDatabase::ready() -> Option<Result<(), DatabaseLoadFailed>>`
  as the recommended readiness check (`None` = loading, `Some(Ok)` = loaded,
  `Some(Err)` = failed). Keep `is_loaded()` as a `== Loaded` convenience and
  `status()`/`DatabaseStatus` as the canonical enum. Latch the aggregate status
  once it settles (Loaded/Failed) so steady-state callers don't re-poll every
  table handle each frame; the latch reflects the **initial** load outcome and is
  not re-evaluated after settling.
- **WHY:** The idiomatic `if !db.is_loaded() { return; }` silently spins forever
  on a failed load. Returning a `Result` forces callers to confront failure
  rather than mistaking it for "still loading." The latch was folded into the
  same change to also resolve the per-frame re-poll cost (audit finding F2).

## Custom asset types: unify built-ins and custom tables on one generic registry
- **WHEN:** 2026-06-02
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** assets / resource (cross-system)
- **SCOPE:** architectural
- **WHAT:** When implementing #37 (custom/third-party tables as first-class
  peers), unify the built-in tables and consumer-registered custom tables onto a
  single generic, type-keyed registry rather than maintaining a parallel path for
  custom tables. Built-in convenience accessors become thin wrappers over the
  generic mechanism.
- **WHY:** Makes custom tables true peers of the built-ins (same status
  aggregation, note cache, baking, and access), and collapses the current
  ~8-site-per-table duplication (audit finding F4) instead of adding a second set
  of parallel code paths to keep in sync. The larger refactor and churn to the
  stable-ish API is accepted as worth the cleaner end state. Approach: a deeper
  requirements interview first, before proposing the API.

## Publish 0.1.0 against Bevy 0.19.0-rc.2 (don't wait for 0.19 final)
- **WHEN:** 2026-06-02
- **PROJECT:** bevy_rmmz_assets
- **SYSTEM:** release/packaging
- **SCOPE:** moderate
- **WHAT:** Publish `0.1.0` to crates.io against Bevy `0.19.0-rc.2` rather than
  waiting for Bevy 0.19 final.
- **WHY:** Staking the crate name and getting early feedback now outweighs the
  downsides of depending on a pre-release (version requirement pins to the rc,
  the rc could be yanked). The README already flags the crate's pre-release
  status, so consumers are warned.
