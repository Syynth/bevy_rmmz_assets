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
