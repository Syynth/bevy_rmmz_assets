//! Tokenizer for the RPG Maker `note` convention.
//!
//! Community plugins embed structured metadata in the free-text `note` field
//! using angle-bracket tags. [`NoteTokens::parse`] recognizes three shapes and
//! ignores the surrounding prose:
//!
//! - `<key>` — a [flag](NoteTag::Flag).
//! - `<key:value>` — a [key/value](NoteTag::Value) pair.
//! - `<key> … </key>` — a [block](NoteTag::Block) with multi-line body.
//!
//! Keys and values are matched case-sensitively and trimmed of surrounding
//! whitespace. This layer only tokenizes; turning tags into typed metadata is
//! the job of the note-parser registry.

/// A single recognized tag within a note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteTag {
    /// A bare `<key>` with no value and no matching close tag.
    Flag {
        /// The tag key.
        key: String,
    },
    /// A `<key:value>` pair.
    Value {
        /// The tag key.
        key: String,
        /// The (trimmed) value text.
        value: String,
    },
    /// A `<key> … </key>` block.
    Block {
        /// The tag key.
        key: String,
        /// The (trimmed) body between the open and close tags.
        body: String,
    },
}

impl NoteTag {
    /// The key of this tag.
    pub fn key(&self) -> &str {
        match self {
            NoteTag::Flag { key } | NoteTag::Value { key, .. } | NoteTag::Block { key, .. } => key,
        }
    }
}

/// The parsed tags of a single note, plus the original text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NoteTokens {
    raw: String,
    tags: Vec<NoteTag>,
}

impl NoteTokens {
    /// Tokenizes a note string into its recognized tags. Text outside of tags is
    /// ignored. Stray/unmatched closing tags and empty `<>` are skipped.
    pub fn parse(note: &str) -> Self {
        let mut tags = Vec::new();
        let mut cursor = 0usize;

        while let Some(open_rel) = note[cursor..].find('<') {
            let open = cursor + open_rel;
            let Some(close_rel) = note[open + 1..].find('>') else {
                break;
            };
            let close = open + 1 + close_rel;
            let inner = note[open + 1..close].trim();
            cursor = close + 1;

            if inner.is_empty() || inner.starts_with('/') {
                continue;
            }

            if let Some((key, value)) = inner.split_once(':') {
                tags.push(NoteTag::Value {
                    key: key.trim().to_owned(),
                    value: value.trim().to_owned(),
                });
                continue;
            }

            // A bare `<key>`: a block if a matching `</key>` follows, else a flag.
            let key = inner;
            let closing = format!("</{key}>");
            if let Some(body_rel) = note[cursor..].find(&closing) {
                let body_end = cursor + body_rel;
                let body = note[cursor..body_end].trim().to_owned();
                tags.push(NoteTag::Block {
                    key: key.to_owned(),
                    body,
                });
                cursor = body_end + closing.len();
            } else {
                tags.push(NoteTag::Flag {
                    key: key.to_owned(),
                });
            }
        }

        Self {
            raw: note.to_owned(),
            tags,
        }
    }

    /// The original, untokenized note text.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// All recognized tags, in source order.
    pub fn tags(&self) -> &[NoteTag] {
        &self.tags
    }

    /// Whether any tag (of any kind) uses `key`.
    pub fn contains(&self, key: &str) -> bool {
        self.tags.iter().any(|t| t.key() == key)
    }

    /// Whether a bare `<key>` flag is present.
    pub fn has_flag(&self, key: &str) -> bool {
        self.tags
            .iter()
            .any(|t| matches!(t, NoteTag::Flag { key: k } if k == key))
    }

    /// The value of the first `<key:value>` tag with `key`.
    pub fn value(&self, key: &str) -> Option<&str> {
        self.values(key).next()
    }

    /// All values of `<key:value>` tags with `key`, in source order.
    pub fn values<'a>(&'a self, key: &str) -> impl Iterator<Item = &'a str> {
        // Own the key so the returned iterator borrows only `self`, not `key`.
        let key = key.to_owned();
        self.tags.iter().filter_map(move |t| match t {
            NoteTag::Value { key: k, value } if *k == key => Some(value.as_str()),
            _ => None,
        })
    }

    /// The body of the first `<key> … </key>` block with `key`.
    pub fn block(&self, key: &str) -> Option<&str> {
        self.tags.iter().find_map(|t| match t {
            NoteTag::Block { key: k, body } if k == key => Some(body.as_str()),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::NoteTokens;

    #[test]
    fn parses_value_flag_and_block_amid_prose() {
        let note = "Intro prose.\n<element:fire> some text <boss>\n<lore>\nAncient relic.\nForged in flame.\n</lore>\ntrailing";
        let t = NoteTokens::parse(note);

        assert_eq!(t.value("element"), Some("fire"));
        assert!(t.has_flag("boss"));
        assert_eq!(t.block("lore"), Some("Ancient relic.\nForged in flame."));
        assert_eq!(t.tags().len(), 3);
        assert!(t.contains("element"));
        assert!(t.contains("lore"));
        assert!(!t.contains("missing"));
    }

    #[test]
    fn trims_keys_and_values() {
        let t = NoteTokens::parse("<  speed : 12  >");
        assert_eq!(t.value("speed"), Some("12"));
    }

    #[test]
    fn collects_repeated_values() {
        let t = NoteTokens::parse("<drop:1><drop:2><drop:3>");
        let drops: Vec<_> = t.values("drop").collect();
        assert_eq!(drops, ["1", "2", "3"]);
        assert_eq!(t.value("drop"), Some("1"));
    }

    #[test]
    fn flag_vs_block_disambiguation() {
        // `<note>` with no close is a flag; with a close it's a block.
        let flag = NoteTokens::parse("<sticky>");
        assert!(flag.has_flag("sticky"));
        assert_eq!(flag.block("sticky"), None);

        let block = NoteTokens::parse("<sticky>body</sticky>");
        assert!(!block.has_flag("sticky"));
        assert_eq!(block.block("sticky"), Some("body"));
    }

    #[test]
    fn empty_value_is_allowed() {
        let t = NoteTokens::parse("<hidden:>");
        assert_eq!(t.value("hidden"), Some(""));
    }

    #[test]
    fn ignores_empty_and_stray_closing_tags() {
        let t = NoteTokens::parse("<></foo>plain text");
        assert!(t.tags().is_empty());
        assert_eq!(t.raw(), "<></foo>plain text");
    }

    #[test]
    fn empty_note_has_no_tags() {
        let t = NoteTokens::parse("");
        assert!(t.tags().is_empty());
    }
}
