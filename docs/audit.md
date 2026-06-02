# Project audit — `bevy_rmmz_assets`

**Date:** 2026-06-02
**Scope:** whole crate (~4,600 LOC Rust), all features. Reviewed against ease of
use, understandability, documentation, performance, security, and
maintainability.
**State at audit:** `cargo test --all-features` green (52 unit + 1 integration +
3 doc tests); `cargo clippy --all-features --all-targets -D warnings` clean under
a deliberately strict lint profile.

## Verdict

Unusually disciplined work for the size: a genuinely layered architecture that
matches its documented design, strict lints held throughout (not retrofitted), no
dead code, no `TODO` debt, no half-finished features, and a clean
one-issue→one-branch→one-PR history with automated review on each PR. The
findings below are almost entirely **polish and publish-readiness**, not
correctness. There is one latent correctness footgun (F1), and it is niche.

## Architecture (as built)

Four cleanly separated layers, matching `docs/decision-log.md`:

1. **Data models** (`src/data/`) — serde structs mirroring MZ camelCase JSON,
   uniformly `#[serde(default)]` for version-drift tolerance (validated against a
   real MZ project; regression test at `src/data/mod.rs:297`).
2. **Asset layer** (`src/asset.rs`, `src/loader.rs`) — generic `Table<T>` asset +
   a file-agnostic JSON loader resolved by asset *type* rather than extension
   (necessary since every MZ database file is `.json`).
3. **Resource layer** (`src/database.rs`) — `RmmzDatabase` SystemParam for cheap
   id lookups, reading live `Assets<…>` so hot-reload is free.
4. **Notes subsystem** (`src/notes/`) — tokenizer + type-erased parser registry +
   parse-once cache, with ahead-of-time baking into the processed binary.

Optional `maps` and `process` features, an `RmmzAppExt` convenience trait, a
runnable example, and an end-to-end disk integration test.

## Dimension notes

- **Ease of use — strong.** One-liner setup, builder config, clean prelude. The
  awkward map-request-needs-a-separate-system constraint is documented. `note_meta`
  turbofish is slightly clumsy; `is_loaded()` returning `false` for both loading
  and failure is a foot-tap (documented, with `status()`/`is_failed()`). See F2.
- **Understandability — strong.** Real layering, module docs explain the *why*,
  repetitive surface partly tamed by macros (`table_accessors!`, `impl_has_id!`).
- **Documentation — strong, with publish gaps.** Excellent inline coverage +
  README + CONTRIBUTING + decision log + compiling doc examples. Gaps: F5, F6, F7.
- **Performance — appropriate.** Parse-once + cache; release builds skip JSON and
  parsing via baked binaries; O(1) id lookups. Minor scaling notes: F2, F3.
- **Security — low risk.** `unsafe_code = "deny"`, zero unsafe. Inputs are local
  game assets / build-time binaries; serde_json recursion limit bounds nesting
  DoS. Trust boundary (trusted `.rmmzbin`/JSON) should be stated in docs.
- **Maintainability — strong, one structural smell.** Strict lints + green CI +
  CodeRabbit gate; tests exercise the real async asset pipeline. See F4.

## Findings

| # | Severity | Area | Finding |
|---|----------|------|---------|
| **F1** | **Medium** | Correctness | Duplicate `NoteParser::TAG` across two *different* output types is undetected. `unbakers` is keyed by tag and silently overwritten (`src/notes/registry.rs:88`); baking emits two same-tag entries that both unbake as the last-registered type → silent metadata corruption/loss. Docs say TAG "must be unique" but nothing enforces it. Add a `debug_assert!`/return-error on tag collision. |
| F2 | Low | Ergonomics / perf | `is_loaded()` returns `false` for both "loading" and "failed"; and `status()` re-polls all 14 handles every call with no loaded-latch, though the example calls it per frame. (`src/database.rs:154`) |
| F3 | Low | Perf | `cache_map_notes` reverse-maps `asset_id → map_id` by linear-scanning `handles` per event — O(maps) per event. Fine for tens of maps; wants a reverse index at thousands. (`src/maps.rs:145`) |
| F4 | Design | Extensibility / maint | Adding a table touches ~8 parallel sites (`CoreTable`, `RmmzHandles`, `load_core_tables`, `status()`, `RmmzDatabase` fields, `table_accessors!`, `plugin::build`, `register_rmmz_processing`, prelude) — silent-drift risk. More importantly, there is no path for a *consumer* (an MZ game with custom plugins / custom asset types) to register their own tables/assets as first-class peers. Worth a design pass. |
| F5 | Low | Docs/publish | No `[package.metadata.docs.rs]` (`all-features = true`, `--cfg docsrs`); the `maps`/`process` modules won't render on docs.rs. |
| F6 | Low | Docs | The `process` workflow (`register_rmmz_processing`, per-asset `.meta`, `asset_processor`) lives only in the `processing` module doc, not the README, despite being a headline feature. |
| F7 | Low | Docs/maint | `missing_docs` isn't enforced; coverage is excellent today but unguarded. Add `#![warn(missing_docs)]`. |
| F8 | Low | CI coverage | CI runs clippy/test only with `--all-features`. The default-feature (no `maps`) path — incl. `#[cfg(not(feature = "maps"))]` at `src/processing.rs:288` — is never compiled in CI, though CONTRIBUTING tells humans to. Add a default-features job. |
| F9 | Trivial | Publish | No CHANGELOG; depends on Bevy `0.19.0-rc.2`. Expected for unpublished `0.1`; track before release. |

## Tracking

- Umbrella issue: quick wins (F1, F3, F5–F9) + this document.
- Standalone: F2 (`is_loaded()` ergonomics).
- Standalone (design): F4 (custom/peer asset types & tables).
