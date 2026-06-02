# Design: custom/third-party asset types as first-class peers (#37)

**Status:** design complete (interview + all open questions resolved 2026-06-02); ready for implementation
**Issue:** [#37](https://github.com/Syynth/bevy_rmmz_assets/issues/37)
**Related decisions:** "Custom asset types as first-class peers"; "unify built-ins
and custom tables on one generic registry" (see `docs/decision-log.md`).

## Motivation & real examples

Validated against a real project (`codetta-rpg-mz`). Its custom `data/*.json`
files are **not** MZ-style id arrays — they are whole-document typed configs:

- **`AnimationMap.json`** — string-keyed map: `{ "furnitureBreak": { backend,
  config }, … }`.
- **`RankStats.json`** — `{ "actors": { "1": { name, stats: { hp:[10], … } } } }`
  (actor-id-keyed map of per-rank stat arrays).
- **`text_profiles.json`** — nested config: `schemaVersion`, `globalDefaults`,
  `punctuationStyles`, `profiles` (character-keyed), punctuation chars as keys.

So the extension need is **"load `data/<Name>.json` into a type I define and hand
me the whole thing,"** closer to the `System.json` singleton than to `Items.json`.

## Requirements (from interview)

1. **Two shapes, both first-class:** id-indexed tables (`Table<R>`) *and*
   whole-document singletons (arbitrary user-defined `R`). Singletons dominate the
   real data, but custom arrays must work too.
2. **Fully typed:** consumers define Rust structs with serde derives.
3. **Minimal access:** hand back the typed asset; the consumer navigates it. No
   cross-reference helpers (e.g. no "rank stats for actor N" conveniences).
4. **Notes are a generic, orthogonal facility:** any type — built-in or custom —
   that carries note-bearing records opts into the *same* note parsing / cache /
   AOT baking. No built-in-vs-custom distinction.
5. **Unify:** built-ins and custom tables flow through one type-keyed registry;
   built-in convenience accessors become thin wrappers. Collapses the current
   ~8-site-per-table duplication (audit F4).
6. **Access via a snapshot resource** (see the constraint below).

## The hard constraint

`RmmzDatabase` is a `SystemParam` with **static** fields (`Res<Assets<ItemsAsset>>`,
…). A generic `db.asset::<R>()` for a type the crate has never heard of would need
`Res<Assets<R>>` for arbitrary `R`, which a `SystemParam` cannot enumerate. The
data must therefore live somewhere `RmmzDatabase` *can* reach for the generic path.
Chosen resolution: a **snapshot resource**.

## Design sketch

### Registration (one mechanism, two ergonomic front-ends)

There is **no table-vs-singleton split at the registration layer** — every file is
just "load `data/X.json` into a type `R`." A macro declares the binding (file +
optional `notes`); registration is one generic call. The table-vs-singleton choice
is purely *which `R` you pick*, surfaced as two convenience macros:

```rust
// Singleton: load the file into the type as-is (struct, HashMap newtype, …).
rmmz_asset!(TextProfiles, file = "text_profiles.json");
rmmz_asset!(AnimationMap, file = "AnimationMap.json");   // e.g. a HashMap<String, _> newtype

// Table: opt the record type into MZ's 1-based, null-padded int-id convention.
// Sugar for registering `R = Table<Quest>`; `notes` requires Quest: HasNote + HasId.
rmmz_table!(Quest, file = "QuestLog.json", notes);

app.register_rmmz::<TextProfiles>();   // reads `const FILE` stamped by the macro
app.register_rmmz::<Table<Quest>>();
```

`Table<R>` is **not** a special path — it is a provided collection *type* (the MZ
int-id, null-padded convention) that **any** record type may opt into as its `R`,
built-in or custom. A custom record gets the same `.get(id)`/`.iter()`/`.count()`
and (with `notes`) note indexing simply by registering as `Table<R>`. String-keyed
maps and config structs are just "your `R`," iterated with stdlib methods.

The registry records the type, filename, and whether it carries notes. A startup
system loads every registered path; a type-keyed handle map replaces the hardcoded
`RmmzHandles` fields. `status()`, notes, and baking all read the registry.

### Access (RmmzDatabase)

```rust
db.asset::<TextProfiles>()  -> Option<&TextProfiles>   // singleton
db.table::<Quest>()         -> Option<&Table<Quest>>   // table
db.record::<Quest>(id)      -> Option<&Quest>          // table by 1-based id
```

These work uniformly for built-ins *and* custom types via **manual
specialization** — a trait with per-type impls (no blanket impl, so it compiles
on stable; real specialization is nightly-only):

```rust
pub trait RmmzFetch: Sized {
    fn table(db: &RmmzDatabase) -> Option<&Table<Self>>;
}

// Built-in: crate-generated impl reads the static `Res<Assets<…>>` field — zero copy.
impl RmmzFetch for Item { fn table(db) -> _ { db.items.get(...) } }

// Custom: a one-line declarative macro generates the impl, reading the snapshot.
rmmz_table!(Quest, file = "QuestLog.json");
//  => impl RmmzFetch for Quest { fn table(db) { db.snapshot::<Quest>() } }

pub fn table<R: RmmzFetch>(&self) -> Option<&Table<R>> { R::table(self) }
```

`db.table::<Item>()` resolves to the zero-copy static-field read; `db.table::<Quest>()`
resolves to the snapshot read — **one interface, per-type efficient impl**. No
overlap (no blanket impl), so it's legal on stable; custom types impl the foreign
trait on their local type (orphan rule allows it) via the macro. The macro is
**declarative** (`macro_rules!`), not a proc-macro, so it adds no `syn`/`quote`
weight, and it stamps `const FILE` onto the type so registration is just
`app.register_rmmz::<Quest>()`. Built-ins keep their named accessors
(`db.item(1)`, `db.system()`) as thin wrappers over the same trait.

### Snapshot resource

```rust
// type-keyed: TypeId -> Arc<dyn Any + Send + Sync>
#[derive(Resource, Default)]
pub struct RmmzAssets { map: HashMap<TypeId, Arc<dyn Any + Send + Sync>> }
```

A generic `snapshot_asset::<A>` system, registered per registered type, updates
the entry on `AssetEvent::{Added,Modified,LoadedWithDependencies}` (and clears on
removal). Storing **`Arc<A>`** means one ref-bump per load/reload, not a deep copy
per access, and the generic path shares rather than re-duplicates.

**Memory note:** snapshotting clones the asset into the `Arc` once per load, so the
data exists in both `Assets<A>` and the snapshot. Trivial for config files; real
for big built-ins (Maps, Troops). Proposed mitigation: built-ins keep their
zero-copy `Res<Assets<…>>` named accessors and need not be snapshotted; large
built-ins (Maps) stay handle-based and out of the generic snapshot path. (Open —
see below.)

### Generic note facility

The note core (`NoteRegistry`, `RmmzNoteCache` keyed by `(TypeId, id)`,
`bake`/`unbake`) is already type-generic. Only the *list* of wired types is
hardcoded today. Unifying means: registering a note-bearing type also registers
`cache_table_notes::<R>` and its baking processor through the same path — so notes
become orthogonal to built-in-vs-custom for free.

### Status aggregation

`aggregate_status` iterates the registry's handle map instead of the hardcoded 14,
so custom tables participate in `status()` / `ready()` / the settle latch.

### Processing / baking

`register_rmmz_processing` derives its type list from the registry rather than a
hardcoded set; note-bearing types bake, others get data-only processing (which
also gives custom configs a compact binary for release for free).

## Open questions

1. ~~**Snapshot scope for built-ins.**~~ **RESOLVED:** option-2 storage (built-ins
   zero-copy via static fields; custom via the snapshot resource, big built-ins
   stay handle-based), presented through *one* uniform generic interface via the
   `RmmzFetch` manual-specialization trait + declarative macro (see Access above).
   Uniform surface, no doubled Map memory, no nightly.
2. ~~**Notes auto vs opt-in.**~~ **RESOLVED:** opt-in via a `notes` flag on the
   registration macro (`rmmz_table!(Quest, file = "…", notes)`), **type-enforced**
   — declaring `notes` requires `R: HasNote + HasId` or it's a hard compile error.
   Truly-automatic isn't feasible on stable (can't branch generic wiring on an
   optional trait bound — the same specialization limit as Q1). Built-ins declare
   `notes` on the 9 note-bearing types via their crate-internal macro; the other 6
   omit it. No behavior change from today.
3. ~~**A concrete custom *array* file.**~~ **RESOLVED:** the table-vs-singleton
   split dissolves — registration is one mechanism; the real axis is how a
   collection is *keyed* (int `Table<R>` vs string-keyed map vs nested config), and
   that's just "which `R`." No unified iteration/lookup trait (`Vec`/`HashMap`
   iterate natively; key types differ). Custom types **can** opt into the int-id
   `Table<R>` convention via `rmmz_table!`, but the real custom data here is
   string-keyed maps — so `AnimationMap.json` is the natural custom fixture, not a
   fabricated int-id table.
4. ~~**Config/selection.**~~ **RESOLVED:** registration = loaded, for everyone.
   `TableSelection` survives only as sugar for *which built-ins auto-register*
   (`add_rmmz` = all 14; `add_rmmz_with(...with_tables([…]))` = a subset). Custom
   types are registered individually and always load — no separate selection.
   Custom files resolve under the shared `data_path` via `RmmzConfig::path()`; a
   per-type path override is YAGNI for now.
5. ~~**Filename ↔ type binding & collisions.**~~ **RESOLVED:** type→file is 1:1 by
   construction (`const FILE` on the type), and the registry is `TypeId`-keyed, so
   there are no structural collisions. Re-registering the same type is an
   idempotent no-op (`debug_assert` in dev, never `panic!`). Two *different* types
   may bind the same file (alternate typed views — Bevy keys typed loads by
   `(path, type)`); allowed, no warning. No stricter shadowing checks.

## Rough phases

1. Unified type-keyed registry + handle map; port built-ins onto it (named
   accessors preserved). Status aggregation reads the registry.
2. Snapshot resource + generic `db.asset`/`db.table`/`db.record`.
3. Note facility wired through registration (generalize the per-type plugin wiring).
4. Processing/baking driven from the registry.
5. Docs + a worked custom-type example (using a codetta config as the fixture).
