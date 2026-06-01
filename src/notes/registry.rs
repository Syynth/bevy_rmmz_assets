//! An extensible registry that turns [`NoteTokens`] into typed metadata.
//!
//! Register a [`NoteParser`] for each metadata type you care about; each parser
//! maps a note's tokens to an optional value of its `Output` type. Look the
//! results up by type with [`NoteRegistry::parse`] (single type) or
//! [`NoteRegistry::parse_all`] (every registered parser into a [`ParsedNote`]).
//!
//! Parsing is **on demand** rather than baked into the asset at load time: the
//! asset types stay pure data, there is no per-record storage to keep in sync,
//! and because parsing reads the live note text, results automatically reflect
//! hot-reloads.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use bevy_ecs::resource::Resource;

use crate::notes::NoteTokens;

/// Produces a piece of typed metadata from a note's tokens.
pub trait NoteParser: Send + Sync + 'static {
    /// The metadata type this parser produces.
    type Output: Send + Sync + 'static;

    /// Parses the tokens into the output type, or `None` if the note carries no
    /// such metadata.
    fn parse(&self, tokens: &NoteTokens) -> Option<Self::Output>;
}

/// A function that produces a boxed output from tokens, hiding the parser type.
type ErasedParser = Box<dyn Fn(&NoteTokens) -> Option<Box<dyn Any + Send + Sync>> + Send + Sync>;

/// A registry of [`NoteParser`]s, keyed by the [`TypeId`] of their output.
///
/// Registering two parsers with the same output type replaces the earlier one.
#[derive(Resource, Default)]
pub struct NoteRegistry {
    parsers: HashMap<TypeId, ErasedParser>,
}

impl NoteRegistry {
    /// Registers a parser. Prefer
    /// [`RmmzAppExt::register_note_parser`](crate::ext::RmmzAppExt::register_note_parser).
    pub fn register<P: NoteParser>(&mut self, parser: P) {
        let erased: ErasedParser = Box::new(move |tokens| {
            parser
                .parse(tokens)
                .map(|out| -> Box<dyn Any + Send + Sync> { Box::new(out) })
        });
        self.parsers.insert(TypeId::of::<P::Output>(), erased);
    }

    /// Whether a parser producing `T` is registered.
    pub fn has<T: Send + Sync + 'static>(&self) -> bool {
        self.parsers.contains_key(&TypeId::of::<T>())
    }

    /// Runs the parser registered for `T` against `tokens`.
    ///
    /// Returns `None` if no parser produces `T`, or if that parser found no
    /// matching metadata in the note.
    pub fn parse<T: Send + Sync + 'static>(&self, tokens: &NoteTokens) -> Option<T> {
        let parser = self.parsers.get(&TypeId::of::<T>())?;
        let boxed = parser(tokens)?;
        boxed.downcast::<T>().ok().map(|b| *b)
    }

    /// Runs every registered parser against `tokens`, collecting the results
    /// into a [`ParsedNote`] type map.
    pub fn parse_all(&self, tokens: &NoteTokens) -> ParsedNote {
        let mut map: HashMap<TypeId, Box<dyn Any + Send + Sync>> = HashMap::new();
        for (type_id, parser) in &self.parsers {
            if let Some(value) = parser(tokens) {
                map.insert(*type_id, value);
            }
        }
        ParsedNote { map }
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
}

#[cfg(test)]
mod tests {
    use super::{NoteParser, NoteRegistry};
    use crate::notes::NoteTokens;

    #[derive(Debug, PartialEq, Eq)]
    struct Element(String);

    struct ElementParser;
    impl NoteParser for ElementParser {
        type Output = Element;
        fn parse(&self, tokens: &NoteTokens) -> Option<Element> {
            tokens.value("element").map(|v| Element(v.to_owned()))
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Boss;

    struct BossParser;
    impl NoteParser for BossParser {
        type Output = Boss;
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
        let tokens = NoteTokens::parse("<element:ice> <boss>");
        let parsed = reg.parse_all(&tokens);
        assert_eq!(parsed.get::<Element>(), Some(&Element("ice".to_owned())));
        assert_eq!(parsed.get::<Boss>(), Some(&Boss));
        assert!(parsed.contains::<Element>());

        let empty = reg.parse_all(&NoteTokens::parse("nothing here"));
        assert!(empty.is_empty());
        assert_eq!(empty.get::<Element>(), None);
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

    #[test]
    fn re_registering_replaces_parser() {
        // A second parser for the same Output wins.
        struct Always;
        impl NoteParser for Always {
            type Output = Element;
            fn parse(&self, _: &NoteTokens) -> Option<Element> {
                Some(Element("default".to_owned()))
            }
        }

        let mut reg = NoteRegistry::default();
        reg.register(ElementParser);
        reg.register(Always);
        assert_eq!(
            reg.parse::<Element>(&NoteTokens::parse("no tags")),
            Some(Element("default".to_owned()))
        );
    }
}
