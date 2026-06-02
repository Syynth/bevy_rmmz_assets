//! An extensible registry that turns [`NoteTokens`] into typed metadata.
//!
//! Register a [`NoteParser`] for each metadata type you care about; each parser
//! maps a note's tokens to an optional value of its `Output` type. The
//! note-metadata cache runs every registered parser once per record at load
//! time (see [`crate::notes::cache`]), so lookups never re-parse.
//!
//! Parser outputs must be `Serialize + DeserializeOwned` so the parsed metadata
//! can be baked into the processed binary ahead of time (feature `process`) and
//! deserialized back at runtime instead of re-parsing.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use bevy_ecs::resource::Resource;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::notes::NoteTokens;

/// Produces a piece of typed metadata from a note's tokens.
pub trait NoteParser: Send + Sync + 'static {
    /// The metadata type this parser produces. Must be serializable so it can be
    /// baked into the processed binary.
    type Output: Serialize + DeserializeOwned + Send + Sync + 'static;

    /// A stable, unique tag identifying this metadata in baked assets.
    ///
    /// It is written into processed binaries and matched on load, so it must be
    /// stable across builds (do **not** use `type_name`, which can change with
    /// the toolchain or a type rename) and unique among registered parsers.
    const TAG: &'static str;

    /// Parses the tokens into the output type, or `None` if the note carries no
    /// such metadata.
    fn parse(&self, tokens: &NoteTokens) -> Option<Self::Output>;
}

/// Parses tokens into a boxed output, hiding the parser type.
type ParseFn = Arc<dyn Fn(&NoteTokens) -> Option<Box<dyn Any + Send + Sync>> + Send + Sync>;
/// Parses then serializes, yielding `(type tag, bytes)` for baking.
type BakeFn = Arc<dyn Fn(&NoteTokens) -> Option<(String, Vec<u8>)> + Send + Sync>;
/// Deserializes baked bytes back into a boxed output.
type UnbakeFn = Arc<dyn Fn(&[u8]) -> Option<Box<dyn Any + Send + Sync>> + Send + Sync>;

/// A registry of [`NoteParser`]s, keyed by the [`TypeId`] of their output.
///
/// Registering two parsers with the same output type replaces the earlier one.
#[derive(Resource, Default)]
pub struct NoteRegistry {
    parsers: HashMap<TypeId, ParseFn>,
    /// Bake closures (parse + serialize), keyed by output type.
    bakers: HashMap<TypeId, BakeFn>,
    /// Unbake closures (deserialize), keyed by type tag, with the target type id.
    unbakers: HashMap<String, (TypeId, UnbakeFn)>,
}

impl NoteRegistry {
    /// Registers a parser. Prefer
    /// [`RmmzAppExt::register_note_parser`](crate::ext::RmmzAppExt::register_note_parser).
    pub fn register<P: NoteParser>(&mut self, parser: P) {
        let type_id = TypeId::of::<P::Output>();
        let tag = P::TAG.to_owned();
        let parser = Arc::new(parser);

        let parse = Arc::clone(&parser);
        self.parsers.insert(
            type_id,
            Arc::new(move |tokens| {
                parse
                    .parse(tokens)
                    .map(|out| -> Box<dyn Any + Send + Sync> { Box::new(out) })
            }),
        );

        let bake = Arc::clone(&parser);
        let bake_tag = tag.clone();
        self.bakers.insert(
            type_id,
            Arc::new(move |tokens| {
                let out = bake.parse(tokens)?;
                let bytes = postcard::to_stdvec(&out).ok()?;
                Some((bake_tag.clone(), bytes))
            }),
        );

        self.unbakers.insert(
            tag,
            (
                type_id,
                Arc::new(|bytes| {
                    postcard::from_bytes::<P::Output>(bytes)
                        .ok()
                        .map(|out| -> Box<dyn Any + Send + Sync> { Box::new(out) })
                }),
            ),
        );
    }

    /// Whether a parser producing `T` is registered.
    pub fn has<T: Send + Sync + 'static>(&self) -> bool {
        self.parsers.contains_key(&TypeId::of::<T>())
    }

    /// Runs the parser registered for `T` against `tokens`.
    pub fn parse<T: Send + Sync + 'static>(&self, tokens: &NoteTokens) -> Option<T> {
        let parser = self.parsers.get(&TypeId::of::<T>())?;
        parser(tokens)?.downcast::<T>().ok().map(|b| *b)
    }

    /// Runs every registered parser against `tokens`, collecting the results
    /// into a [`ParsedNote`] type map.
    pub fn parse_all(&self, tokens: &NoteTokens) -> ParsedNote {
        let mut note = ParsedNote::default();
        for (type_id, parser) in &self.parsers {
            if let Some(value) = parser(tokens) {
                note.insert_raw(*type_id, value);
            }
        }
        note
    }

    /// Runs every parser and serializes the matches into `(type tag, bytes)`
    /// pairs for baking into a processed asset. Output is sorted by tag so baked
    /// assets are reproducible regardless of registration order.
    pub fn bake(&self, tokens: &NoteTokens) -> Vec<(String, Vec<u8>)> {
        let mut baked: Vec<(String, Vec<u8>)> = self
            .bakers
            .values()
            .filter_map(|bake| bake(tokens))
            .collect();
        baked.sort_by(|a, b| a.0.cmp(&b.0));
        baked
    }

    /// Reconstructs a [`ParsedNote`] from previously [`baked`](Self::bake)
    /// `(type tag, bytes)` pairs, deserializing rather than re-parsing.
    pub fn unbake(&self, baked: &[(String, Vec<u8>)]) -> ParsedNote {
        let mut note = ParsedNote::default();
        for (tag, bytes) in baked {
            if let Some((type_id, unbake)) = self.unbakers.get(tag)
                && let Some(value) = unbake(bytes)
            {
                note.insert_raw(*type_id, value);
            }
        }
        note
    }

    /// Snapshots the current bakers into a [`NoteBaker`] for use by the
    /// processing transformer (which runs without `World` access). Parsers
    /// registered after this call are not included.
    pub fn baker(&self) -> NoteBaker {
        NoteBaker {
            bakers: Arc::new(self.bakers.values().cloned().collect()),
        }
    }
}

/// A cloneable, `World`-free snapshot of the registry's bake closures, used by
/// the processing transformer to bake notes ahead of time.
#[derive(Clone, Default)]
pub struct NoteBaker {
    bakers: Arc<Vec<BakeFn>>,
}

impl NoteBaker {
    /// Whether any parsers were captured (none ⇒ baking would erase metadata).
    pub fn is_empty(&self) -> bool {
        self.bakers.is_empty()
    }

    /// Bakes every matching parser's output for `tokens` into `(tag, bytes)`,
    /// sorted by tag for reproducible output.
    pub fn bake(&self, tokens: &NoteTokens) -> Vec<(String, Vec<u8>)> {
        let mut baked: Vec<(String, Vec<u8>)> =
            self.bakers.iter().filter_map(|bake| bake(tokens)).collect();
        baked.sort_by(|a, b| a.0.cmp(&b.0));
        baked
    }
}

/// Typed metadata parsed from a single note: a map from metadata type to value.
#[derive(Default)]
pub struct ParsedNote {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ParsedNote {
    /// Returns the parsed metadata of type `T`, if present.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Whether metadata of type `T` was produced for this note.
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }

    /// Whether no metadata was produced.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn insert_raw(&mut self, type_id: TypeId, value: Box<dyn Any + Send + Sync>) {
        self.map.insert(type_id, value);
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::{NoteParser, NoteRegistry};
    use crate::notes::NoteTokens;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Element(String);

    struct ElementParser;
    impl NoteParser for ElementParser {
        type Output = Element;
        const TAG: &'static str = "element";
        fn parse(&self, tokens: &NoteTokens) -> Option<Element> {
            tokens.value("element").map(|v| Element(v.to_owned()))
        }
    }

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Boss;

    struct BossParser;
    impl NoteParser for BossParser {
        type Output = Boss;
        const TAG: &'static str = "boss";
        fn parse(&self, tokens: &NoteTokens) -> Option<Boss> {
            tokens.has_flag("boss").then_some(Boss)
        }
    }

    fn registry() -> NoteRegistry {
        let mut reg = NoteRegistry::default();
        reg.register(ElementParser);
        reg.register(BossParser);
        reg
    }

    #[test]
    fn parses_single_type() {
        let reg = registry();
        let tokens = NoteTokens::parse("<element:fire>");
        assert_eq!(
            reg.parse::<Element>(&tokens),
            Some(Element("fire".to_owned()))
        );
        assert_eq!(reg.parse::<Boss>(&tokens), None);
    }

    #[test]
    fn parse_all_collects_present_metadata() {
        let reg = registry();
        let parsed = reg.parse_all(&NoteTokens::parse("<element:ice> <boss>"));
        assert_eq!(parsed.get::<Element>(), Some(&Element("ice".to_owned())));
        assert_eq!(parsed.get::<Boss>(), Some(&Boss));

        let empty = reg.parse_all(&NoteTokens::parse("nothing here"));
        assert!(empty.is_empty());
    }

    #[test]
    fn bake_round_trips_through_unbake() {
        let reg = registry();
        let baked = reg.bake(&NoteTokens::parse("<element:fire> <boss>"));
        assert_eq!(baked.len(), 2);

        // Unbaking reconstructs the same typed metadata without re-parsing.
        let note = reg.unbake(&baked);
        assert_eq!(note.get::<Element>(), Some(&Element("fire".to_owned())));
        assert_eq!(note.get::<Boss>(), Some(&Boss));
    }

    #[test]
    fn unregistered_type_yields_none() {
        let reg = NoteRegistry::default();
        assert!(!reg.has::<Element>());
        assert_eq!(
            reg.parse::<Element>(&NoteTokens::parse("<element:fire>")),
            None
        );
    }
}
