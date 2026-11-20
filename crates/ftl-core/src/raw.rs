//! The raw view of a saved game for the Expert area and `ftled dump`: every
//! decoded field as a dotted path with its type, range and value.
//!
//! The view comes from the serde derives of the model, not from a list per
//! field: [`entries`] serializes the [`SavedGame`] into a [`Node`] tree, and
//! a raw edit ([`set`]) changes one number or flag in that tree and
//! deserializes the game again. A field the codec decodes tomorrow shows up
//! here without a change. The path segments are the serde names of the
//! model (`player_ship.crew.0.health`); an `Option` is transparent, a list
//! counts from 0.
//!
//! What a raw edit can change: a number (`i32`, `u32`) within its type and a
//! flag (`bool`, 0 or 1). Texts, enum variants and the length of a list stay
//! as they are, so the shape of the file never changes; the verify step of
//! a save (encode, decode, compare) still guards the file. A state var
//! (`state_vars.<n>.value`) is edited by its name (`state.<id>`), because
//! its index is no stable name. [`SavedGame::undecoded`] shows as its length
//! here and as hex in [`hex_lines`]; it is never edited.

use std::fmt;

use serde::de::value::{MapDeserializer, SeqDeserializer, StrDeserializer};
use serde::de::{self, IntoDeserializer};
use serde::ser::{self, Impossible};
use serde::{Deserialize, Serialize};

use crate::edit::Field;
use crate::model::SavedGame;

/// The type of a raw entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RawKind {
    /// A struct of the model; its fields follow as children.
    Record,
    /// A list; its items follow as children.
    List,
    I32,
    U32,
    Bool,
    Text,
    /// A variant of an enum without data, such as the difficulty.
    Enum,
    /// An `Option` that is `None`.
    Absent,
}

impl RawKind {
    /// The name `ftled dump` prints and the app shows.
    pub fn name(self) -> &'static str {
        match self {
            RawKind::Record => "record",
            RawKind::List => "list",
            RawKind::I32 => "i32",
            RawKind::U32 => "u32",
            RawKind::Bool => "bool",
            RawKind::Text => "text",
            RawKind::Enum => "enum",
            RawKind::Absent => "none",
        }
    }
}

/// One field of the saved game in the raw view, in the order of the model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RawEntry {
    /// Dotted path, for example `player_ship.resources.scrap`.
    pub path: String,
    /// 0 for a field of the saved game itself, 1 for a field below it, ...
    pub depth: usize,
    pub kind: RawKind,
    /// The value as text: digits, `true`/`false`, the text, the variant;
    /// the item count of a list; empty for a record.
    pub value: String,
    /// The edit of this entry: `raw.<path>` for a number or flag,
    /// `state.<id>` for the value of a state var; `None` when it is not
    /// editable.
    pub field: Option<Field>,
    /// The inclusive range of an editable entry: its type's range, as an
    /// `i32` ([`crate::Value::Int`]).
    pub range: Option<(i32, i32)>,
    /// Whether the codec does not know what the field means (`unknown_*`).
    pub unknown: bool,
}

/// Every field of `game` in the raw view, depth first.
pub fn entries(game: &SavedGame) -> Vec<RawEntry> {
    let mut out = Vec::new();
    if let Node::Record(fields) = tree(game) {
        for (key, node) in &fields {
            walk(key, 0, node, game, &mut out);
        }
    }
    out
}

/// The current value of the number or flag at `path`; `None` when the path
/// names no number or flag of this game.
pub fn get(game: &SavedGame, path: &str) -> Option<i32> {
    match find(&tree(game), path)? {
        Node::I32(v) => Some(*v),
        Node::U32(v) => i32::try_from(*v).ok(),
        Node::Bool(b) => Some(i32::from(*b)),
        _ => None,
    }
}

/// The range of the number or flag at `path`; `None` as in [`get`].
pub fn range(game: &SavedGame, path: &str) -> Option<(i32, i32)> {
    find(&tree(game), path).and_then(range_of)
}

/// Writes `value` to the number or flag at `path` and answers the old
/// value; `None` as in [`get`]. The caller checks the [`range`].
pub(crate) fn set(game: &mut SavedGame, path: &str, value: i32) -> Option<i32> {
    let mut root = tree(game);
    let old = match find_mut(&mut root, path)? {
        Node::I32(v) => std::mem::replace(v, value),
        Node::U32(v) => {
            let old = i32::try_from(*v).ok()?;
            *v = u32::try_from(value).ok()?;
            old
        }
        Node::Bool(b) => i32::from(std::mem::replace(b, value != 0)),
        _ => return None,
    };
    let mut next = SavedGame::deserialize(root).expect("a raw edit keeps the shape of the model");
    next.undecoded = std::mem::take(&mut game.undecoded);
    *game = next;
    Some(old)
}

/// The undecoded bytes as hex lines of 16 bytes: offset from the first
/// undecoded byte, the bytes, the printable ASCII.
pub fn hex_lines(bytes: &[u8]) -> Vec<String> {
    bytes
        .chunks(16)
        .enumerate()
        .map(|(i, chunk)| {
            let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02x}")).collect();
            let ascii: String = chunk
                .iter()
                .map(|&b| {
                    if b.is_ascii_graphic() || b == b' ' {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
            format!("{:08x}  {:<47}  {ascii}", i * 16, hex.join(" "))
        })
        .collect()
}

fn walk(key: &str, depth: usize, node: &Node, game: &SavedGame, out: &mut Vec<RawEntry>) {
    let leaf = key.rsplit('.').next().unwrap_or(key);
    let (kind, value) = match node {
        Node::Record(_) => (RawKind::Record, String::new()),
        Node::List(items) => (RawKind::List, items.len().to_string()),
        Node::I32(v) => (RawKind::I32, v.to_string()),
        Node::U32(v) => (RawKind::U32, v.to_string()),
        Node::Bool(b) => (RawKind::Bool, b.to_string()),
        Node::Text(t) => (RawKind::Text, t.clone()),
        Node::Variant(v) => (RawKind::Enum, (*v).to_owned()),
        Node::None => (RawKind::Absent, String::new()),
    };
    let range = range_of(node);
    let field = range.map(|_| match state_var_id(key, game) {
        Some(id) => Field::StateVar(id),
        None => Field::Raw(key.to_owned()),
    });
    out.push(RawEntry {
        path: key.to_owned(),
        depth,
        kind,
        value,
        field,
        range,
        unknown: leaf.starts_with("unknown"),
    });
    match node {
        Node::Record(fields) => {
            for (name, child) in fields {
                walk(&format!("{key}.{name}"), depth + 1, child, game, out);
            }
        }
        Node::List(items) => {
            for (i, child) in items.iter().enumerate() {
                walk(&format!("{key}.{i}"), depth + 1, child, game, out);
            }
        }
        _ => {}
    }
}

/// The id of the state var whose value `path` names (`state_vars.<n>.value`).
fn state_var_id(path: &str, game: &SavedGame) -> Option<String> {
    let n: usize = path
        .strip_prefix("state_vars.")?
        .strip_suffix(".value")?
        .parse()
        .ok()?;
    game.state_vars.get(n).map(|v| v.id.clone())
}

fn range_of(node: &Node) -> Option<(i32, i32)> {
    match node {
        Node::I32(_) => Some((i32::MIN, i32::MAX)),
        // A Value is an i32; a u32 above i32::MAX cannot be written.
        Node::U32(_) => Some((0, i32::MAX)),
        Node::Bool(_) => Some((0, 1)),
        _ => None,
    }
}

fn find<'n>(root: &'n Node, path: &str) -> Option<&'n Node> {
    path.split('.').try_fold(root, |node, part| match node {
        Node::Record(fields) => fields.iter().find(|(k, _)| *k == part).map(|(_, n)| n),
        Node::List(items) => items.get(index(part)?),
        _ => None,
    })
}

fn find_mut<'n>(root: &'n mut Node, path: &str) -> Option<&'n mut Node> {
    path.split('.').try_fold(root, |node, part| match node {
        Node::Record(fields) => fields.iter_mut().find(|(k, _)| *k == part).map(|(_, n)| n),
        Node::List(items) => items.get_mut(index(part)?),
        _ => None,
    })
}

fn index(s: &str) -> Option<usize> {
    s.bytes()
        .all(|b| b.is_ascii_digit())
        .then(|| s.parse().ok())
        .flatten()
}

fn tree(game: &SavedGame) -> Node {
    game.serialize(NodeSerializer)
        .expect("the model holds only i32, u32, bool, text, enums, options, lists and records")
}

/// The model as serde sees it. Keys and variant names are the static names
/// of the derives.
#[derive(Debug, Clone, PartialEq)]
enum Node {
    I32(i32),
    U32(u32),
    Bool(bool),
    Text(String),
    Variant(&'static str),
    None,
    List(Vec<Node>),
    Record(Vec<(&'static str, Node)>),
}

#[derive(Debug)]
struct RawError(String);

impl fmt::Display for RawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RawError {}

impl ser::Error for RawError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        RawError(msg.to_string())
    }
}

impl de::Error for RawError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        RawError(msg.to_string())
    }
}

fn unsupported(what: &str) -> RawError {
    RawError(format!("the raw view has no {what}"))
}

struct NodeSerializer;

struct SeqSerializer(Vec<Node>);

struct RecordSerializer(Vec<(&'static str, Node)>);

impl ser::Serializer for NodeSerializer {
    type Ok = Node;
    type Error = RawError;
    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = Impossible<Node, RawError>;
    type SerializeMap = Impossible<Node, RawError>;
    type SerializeStruct = RecordSerializer;
    type SerializeStructVariant = Impossible<Node, RawError>;

    fn serialize_bool(self, v: bool) -> Result<Node, RawError> {
        Ok(Node::Bool(v))
    }
    fn serialize_i8(self, _: i8) -> Result<Node, RawError> {
        Err(unsupported("i8"))
    }
    fn serialize_i16(self, _: i16) -> Result<Node, RawError> {
        Err(unsupported("i16"))
    }
    fn serialize_i32(self, v: i32) -> Result<Node, RawError> {
        Ok(Node::I32(v))
    }
    fn serialize_i64(self, _: i64) -> Result<Node, RawError> {
        Err(unsupported("i64"))
    }
    fn serialize_u8(self, _: u8) -> Result<Node, RawError> {
        Err(unsupported("u8"))
    }
    fn serialize_u16(self, _: u16) -> Result<Node, RawError> {
        Err(unsupported("u16"))
    }
    fn serialize_u32(self, v: u32) -> Result<Node, RawError> {
        Ok(Node::U32(v))
    }
    fn serialize_u64(self, _: u64) -> Result<Node, RawError> {
        Err(unsupported("u64"))
    }
    fn serialize_f32(self, _: f32) -> Result<Node, RawError> {
        Err(unsupported("f32"))
    }
    fn serialize_f64(self, _: f64) -> Result<Node, RawError> {
        Err(unsupported("f64"))
    }
    fn serialize_char(self, v: char) -> Result<Node, RawError> {
        Ok(Node::Text(v.to_string()))
    }
    fn serialize_str(self, v: &str) -> Result<Node, RawError> {
        Ok(Node::Text(v.to_owned()))
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<Node, RawError> {
        Err(unsupported("bytes"))
    }
    fn serialize_none(self) -> Result<Node, RawError> {
        Ok(Node::None)
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Node, RawError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Node, RawError> {
        Ok(Node::None)
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Node, RawError> {
        Ok(Node::None)
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Node, RawError> {
        Ok(Node::Variant(variant))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Node, RawError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Node, RawError> {
        Err(unsupported("externally tagged enum"))
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<SeqSerializer, RawError> {
        Ok(SeqSerializer(Vec::with_capacity(len.unwrap_or(0))))
    }
    fn serialize_tuple(self, len: usize) -> Result<SeqSerializer, RawError> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<SeqSerializer, RawError> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RawError> {
        Err(unsupported("externally tagged enum"))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RawError> {
        Err(unsupported("map"))
    }
    fn serialize_struct(self, _: &'static str, len: usize) -> Result<RecordSerializer, RawError> {
        Ok(RecordSerializer(Vec::with_capacity(len)))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RawError> {
        Err(unsupported("externally tagged enum"))
    }
}

impl ser::SerializeSeq for SeqSerializer {
    type Ok = Node;
    type Error = RawError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RawError> {
        self.0.push(value.serialize(NodeSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Node, RawError> {
        Ok(Node::List(self.0))
    }
}

impl ser::SerializeTuple for SeqSerializer {
    type Ok = Node;
    type Error = RawError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RawError> {
        ser::SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Node, RawError> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SeqSerializer {
    type Ok = Node;
    type Error = RawError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RawError> {
        ser::SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Node, RawError> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeStruct for RecordSerializer {
    type Ok = Node;
    type Error = RawError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RawError> {
        self.0.push((key, value.serialize(NodeSerializer)?));
        Ok(())
    }
    fn skip_field(&mut self, _: &'static str) -> Result<(), RawError> {
        Ok(())
    }
    fn end(self) -> Result<Node, RawError> {
        Ok(Node::Record(self.0))
    }
}

impl<'de> IntoDeserializer<'de, RawError> for Node {
    type Deserializer = Node;
    fn into_deserializer(self) -> Node {
        self
    }
}

impl<'de> de::Deserializer<'de> for Node {
    type Error = RawError;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, RawError> {
        match self {
            Node::I32(v) => visitor.visit_i32(v),
            Node::U32(v) => visitor.visit_u32(v),
            Node::Bool(v) => visitor.visit_bool(v),
            Node::Text(v) => visitor.visit_string(v),
            Node::Variant(v) => visitor.visit_str(v),
            Node::None => visitor.visit_none(),
            Node::List(items) => {
                let mut seq = SeqDeserializer::new(items.into_iter());
                let value = visitor.visit_seq(&mut seq)?;
                seq.end()?;
                Ok(value)
            }
            Node::Record(fields) => {
                let mut map = MapDeserializer::new(fields.into_iter());
                let value = visitor.visit_map(&mut map)?;
                map.end()?;
                Ok(value)
            }
        }
    }

    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, RawError> {
        match self {
            Node::None => visitor.visit_none(),
            node => visitor.visit_some(node),
        }
    }

    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, RawError> {
        match self {
            Node::Variant(v) => visitor.visit_enum(StrDeserializer::<RawError>::new(v)),
            Node::Text(v) => visitor.visit_enum(v.into_deserializer()),
            node => node.deserialize_any(visitor),
        }
    }

    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, RawError> {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct
        identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_lines_show_offset_bytes_and_ascii() {
        let lines = hex_lines(b"FTL\x00\x01abcdefghijklmnop");
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("00000000  46 54 4c 00 01 61"));
        assert!(lines[0].ends_with("FTL..abcdefghijk"));
        assert!(lines[1].starts_with("00000010  6c 6d 6e 6f 70"));
    }

    #[test]
    fn index_takes_digits_only() {
        assert_eq!(index("12"), Some(12));
        assert_eq!(index("+1"), None);
        assert_eq!(index("a"), None);
    }
}
