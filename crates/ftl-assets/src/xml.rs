//! FTL's XML files, read with quick-xml and serde.
//!
//! FTL's XML is not always well-formed. [`scrub`] repairs it the way
//! `DatParser` of the Java editor does before it parses:
//!
//! 1. Several roots: most files are a flat list of elements without one root
//!    (FTL 1.5) or inside an `<FTL>` root (1.6). The XML declaration and every
//!    `<FTL>` / `</FTL>` tag are removed and the rest is wrapped in one
//!    synthetic root, so both forms parse alike (`xmlDeclPtn`, `rootTagPtn`).
//! 2. Stray text between elements (seen in the fixture form, tolerated by
//!    FTL) is ignored: the root structs have no `$text` field.
//! 3. Repeated elements: some blueprints repeat an element that FTL reads
//!    once (`PLAYER_SHIP_FED` has two `droneList`, `DRONE_MISSILE` two
//!    `short`). Such fields are read as lists and the first one counts,
//!    assuming FTL takes the first child of a name; the Java editor (JAXB)
//!    keeps the last. `crewCount` repeats on purpose, once per race.
//! 4. Encoding: the Java editor guesses the charset; FTL 1.6 writes UTF-8,
//!    and a byte that is no UTF-8 becomes U+FFFD instead of an error. A UTF-8
//!    byte-order mark is dropped.
//!
//! The Java fixes for FTL 1.02/1.03 (`img="rebel_long_hard"`, a second
//! `<title>` in `SYSTEM_CASING`) are not needed: those texts are absent in
//! 1.6.13, and the image name is not read here.

use std::collections::HashMap;

use serde::Deserialize;

use crate::Error;

/// Repairs `bytes` into one well-formed document with the root `<root>`.
pub(crate) fn scrub(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let mut out = String::with_capacity(text.len() + 16);
    out.push_str("<root>");
    let mut rest = text;
    while let Some(start) = rest.find("<?xml") {
        out.push_str(&rest[..start]);
        rest = match rest[start..].find("?>") {
            Some(end) => &rest[start + end + 2..],
            None => "",
        };
    }
    out.push_str(rest);
    let mut out = out.replace("<FTL>", "").replace("</FTL>", "");
    out.push_str("</root>");
    out
}

/// Parses one file of the package into `T`, naming the file in an error.
pub(crate) fn parse<T: for<'de> Deserialize<'de>>(
    inner_path: &str,
    bytes: &[u8],
) -> Result<T, Error> {
    quick_xml::de::from_str(&scrub(bytes)).map_err(|e| Error::Parse {
        inner_path: inner_path.to_owned(),
        detail: e.to_string(),
    })
}

/// A text element: either inline text or an `id` into the text lookup
/// (`DefaultDeferredText` in the Java editor).
#[derive(Debug, Default, Deserialize)]
pub(crate) struct Text {
    #[serde(rename = "@id")]
    id: Option<String>,
    #[serde(rename = "$text", default)]
    text: String,
}

/// `<text name="...">value</text>` from the `text_*.xml` files.
#[derive(Debug, Default)]
pub(crate) struct TextLookup(HashMap<String, String>);

#[derive(Deserialize)]
struct NamedTexts {
    #[serde(rename = "text", default)]
    texts: Vec<NamedText>,
}

#[derive(Deserialize)]
struct NamedText {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "$text", default)]
    text: String,
}

impl Text {
    /// The id into the text lookup, when the element has one.
    pub(crate) fn id_of(texts: &[Text]) -> Option<&str> {
        texts.first().and_then(|t| t.id.as_deref())
    }
}

impl TextLookup {
    /// The text of `id`, line breaks resolved like [`TextLookup::resolve`].
    pub(crate) fn get(&self, id: &str) -> Option<String> {
        self.0.get(id).map(|t| t.replace("\\n", "\n"))
    }

    pub(crate) fn add_file(&mut self, inner_path: &str, bytes: &[u8]) -> Result<(), Error> {
        let file: NamedTexts = parse(inner_path, bytes)?;
        for t in file.texts {
            self.0.insert(t.name, t.text);
        }
        Ok(())
    }

    /// The looked-up text when the element has a known `id`, else its own
    /// text. The lookup files write line breaks as a literal `\n`; the game
    /// shows a break, so it becomes one. (The Java editor means to do the same
    /// but its `replaceAll("\\n", "\n")` matches a real newline: a no-op.)
    pub(crate) fn resolve(&self, texts: &[Text]) -> String {
        let Some(text) = texts.first() else {
            return String::new();
        };
        match text.id.as_ref().and_then(|id| self.0.get(id)) {
            Some(found) => found.replace("\\n", "\n"),
            None => text.text.trim().to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct BlueprintsFile {
    #[serde(rename = "shipBlueprint", default)]
    pub ships: Vec<ShipXml>,
    #[serde(rename = "crewBlueprint", default)]
    pub crew: Vec<CrewXml>,
    #[serde(rename = "systemBlueprint", default)]
    pub systems: Vec<SystemXml>,
    #[serde(rename = "weaponBlueprint", default)]
    pub weapons: Vec<WeaponXml>,
    #[serde(rename = "droneBlueprint", default)]
    pub drones: Vec<DroneXml>,
    #[serde(rename = "augBlueprint", default)]
    pub augments: Vec<AugXml>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ShipXml {
    #[serde(rename = "@name")]
    pub id: String,
    #[serde(rename = "@layout", default)]
    pub layout: String,
    #[serde(rename = "@img", default)]
    pub img: String,
    #[serde(default)]
    pub class: Vec<Text>,
    #[serde(default)]
    pub name: Vec<Text>,
    #[serde(default)]
    pub desc: Vec<Text>,
    #[serde(rename = "systemList", default)]
    pub system_list: SystemListXml,
    #[serde(rename = "weaponSlots")]
    pub weapon_slots: Option<u32>,
    #[serde(rename = "droneSlots")]
    pub drone_slots: Option<u32>,
    #[serde(rename = "weaponList")]
    #[serde(default)]
    pub weapon_list: Vec<WeaponListXml>,
    #[serde(rename = "aug", default)]
    pub augs: Vec<NameAttr>,
    #[serde(rename = "droneList")]
    #[serde(default)]
    pub drone_list: Vec<DroneListXml>,
    #[serde(default)]
    pub health: Vec<Amount>,
    #[serde(rename = "maxPower")]
    #[serde(default)]
    pub max_power: Vec<Amount>,
    /// One element per race: a mixed crew repeats `crewCount`.
    #[serde(rename = "crewCount", default)]
    pub crew_count: Vec<CrewCountXml>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SystemListXml {
    #[serde(rename = "$value", default)]
    pub entries: Vec<SystemEntry>,
}

/// One child of `systemList`; the tag names the system.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SystemEntry {
    Pilot(RoomXml),
    Doors(RoomXml),
    Sensors(RoomXml),
    Medbay(RoomXml),
    Oxygen(RoomXml),
    Shields(RoomXml),
    Engines(RoomXml),
    Weapons(RoomXml),
    Drones(RoomXml),
    Teleporter(RoomXml),
    Cloaking(RoomXml),
    Artillery(RoomXml),
    Clonebay(RoomXml),
    Hacking(RoomXml),
    Mind(RoomXml),
    Battery(RoomXml),
}

impl SystemEntry {
    pub fn split(self) -> (&'static str, RoomXml) {
        use SystemEntry::*;
        match self {
            Pilot(r) => ("pilot", r),
            Doors(r) => ("doors", r),
            Sensors(r) => ("sensors", r),
            Medbay(r) => ("medbay", r),
            Oxygen(r) => ("oxygen", r),
            Shields(r) => ("shields", r),
            Engines(r) => ("engines", r),
            Weapons(r) => ("weapons", r),
            Drones(r) => ("drones", r),
            Teleporter(r) => ("teleporter", r),
            Cloaking(r) => ("cloaking", r),
            Artillery(r) => ("artillery", r),
            Clonebay(r) => ("clonebay", r),
            Hacking(r) => ("hacking", r),
            Mind(r) => ("mind", r),
            Battery(r) => ("battery", r),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RoomXml {
    #[serde(rename = "@power", default)]
    pub power: u32,
    #[serde(rename = "@max")]
    pub max: Option<u32>,
    #[serde(rename = "@room", default)]
    pub room: u32,
    #[serde(rename = "@start")]
    pub start: Option<bool>,
    #[serde(rename = "@img")]
    pub img: Option<String>,
    /// `<slot>`: where crew mans the system.
    pub slot: Option<SlotXml>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SlotXml {
    pub direction: Option<String>,
    pub number: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WeaponListXml {
    #[serde(rename = "@missiles", default)]
    pub missiles: u32,
    #[serde(rename = "weapon", default)]
    pub weapons: Vec<NameAttr>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DroneListXml {
    #[serde(rename = "@drones", default)]
    pub drones: u32,
    #[serde(rename = "drone", default)]
    pub drones_list: Vec<NameAttr>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NameAttr {
    #[serde(rename = "@name")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Amount {
    #[serde(rename = "@amount")]
    pub amount: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CrewCountXml {
    #[serde(rename = "@amount")]
    pub amount: u32,
    #[serde(rename = "@class")]
    pub class: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CrewXml {
    #[serde(rename = "@name")]
    pub id: String,
    #[serde(default)]
    pub title: Vec<Text>,
    #[serde(default)]
    pub short: Vec<Text>,
    #[serde(default)]
    pub desc: Vec<Text>,
    pub cost: Option<u32>,
    #[serde(rename = "colorList", default)]
    pub color_list: ColorListXml,
}

/// `<colorList>` of a crew blueprint: one `<layer>` per tint layer.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ColorListXml {
    #[serde(rename = "layer", default)]
    pub layers: Vec<ColorLayerXml>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ColorLayerXml {
    #[serde(rename = "color", default)]
    pub colors: Vec<ColorXml>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ColorXml {
    #[serde(rename = "@r")]
    pub r: f32,
    #[serde(rename = "@g")]
    pub g: f32,
    #[serde(rename = "@b")]
    pub b: f32,
    #[serde(rename = "@a", default = "opaque")]
    pub a: f32,
}

fn opaque() -> f32 {
    1.0
}

/// `data/names.xml`: the crew names FTL draws from.
#[derive(Debug, Deserialize)]
pub(crate) struct NamesFile {
    #[serde(rename = "nameList", default)]
    pub lists: Vec<NameListXml>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NameListXml {
    #[serde(rename = "@race", default)]
    pub race: String,
    #[serde(rename = "@sex", default)]
    pub sex: String,
    #[serde(rename = "@language")]
    pub language: Option<String>,
    #[serde(rename = "name", default)]
    pub names: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SystemXml {
    #[serde(rename = "@name")]
    pub id: String,
    #[serde(default)]
    pub title: Vec<Text>,
    #[serde(default)]
    pub desc: Vec<Text>,
    #[serde(rename = "startPower", default)]
    pub start_power: u32,
    #[serde(rename = "maxPower", default)]
    pub max_power: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WeaponXml {
    #[serde(rename = "@name")]
    pub id: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub title: Vec<Text>,
    #[serde(default)]
    pub short: Vec<Text>,
    #[serde(default)]
    pub desc: Vec<Text>,
    #[serde(rename = "weaponArt")]
    pub weapon_art: Option<String>,
    /// Reactor bars the weapon draws; a repeated element keeps the first.
    #[serde(default)]
    pub power: Vec<u32>,
    /// Seconds between shots; a repeated element keeps the first.
    #[serde(default)]
    pub cooldown: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DroneXml {
    #[serde(rename = "@name")]
    pub id: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub title: Vec<Text>,
    #[serde(default)]
    pub short: Vec<Text>,
    #[serde(default)]
    pub desc: Vec<Text>,
    /// Base name of the drone sprites under `img/ship/drones/`.
    #[serde(rename = "droneImage")]
    pub drone_image: Option<String>,
    /// Reactor bars the drone draws; a repeated element keeps the first.
    #[serde(default)]
    pub power: Vec<u32>,
    /// Milliseconds between shots; a repeated element keeps the first.
    #[serde(default)]
    pub cooldown: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AugXml {
    #[serde(rename = "@name")]
    pub id: String,
    #[serde(default)]
    pub title: Vec<Text>,
    #[serde(default)]
    pub desc: Vec<Text>,
}

/// `data/<layout>.xml`: the hull image rectangle, the floor offset and the
/// weapon mounts.
#[derive(Debug, Deserialize)]
pub(crate) struct ChassisXml {
    pub img: Option<Rect>,
    pub offsets: Option<OffsetsXml>,
    #[serde(rename = "weaponMounts")]
    pub weapon_mounts: Option<WeaponMountsXml>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WeaponMountsXml {
    #[serde(rename = "mount", default)]
    pub mounts: Vec<MountXml>,
}

/// One `<mount x y rotate mirror gib slide/>`. FTL compares `rotate` and
/// `mirror` with `"true"` and reads `slide` with `Globals::GetDirection`.
#[derive(Debug, Deserialize)]
pub(crate) struct MountXml {
    #[serde(rename = "@x")]
    pub x: i32,
    #[serde(rename = "@y")]
    pub y: i32,
    #[serde(rename = "@rotate", default)]
    pub rotate: String,
    #[serde(rename = "@mirror", default)]
    pub mirror: String,
    #[serde(rename = "@slide")]
    pub slide: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Rect {
    #[serde(rename = "@x")]
    pub x: i32,
    #[serde(rename = "@y")]
    pub y: i32,
    #[serde(rename = "@w")]
    pub w: i32,
    #[serde(rename = "@h")]
    pub h: i32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OffsetsXml {
    pub floor: Option<Point>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Point {
    #[serde(rename = "@x")]
    pub x: i32,
    #[serde(rename = "@y")]
    pub y: i32,
}

/// `animations.xml`: the sheets and the weapon animations that name them.
#[derive(Debug, Deserialize)]
pub(crate) struct AnimationsFile {
    #[serde(rename = "animSheet", default)]
    pub sheets: Vec<AnimSheetXml>,
    #[serde(rename = "weaponAnim", default)]
    pub weapon_anims: Vec<WeaponAnimXml>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AnimSheetXml {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@w")]
    pub w: u32,
    #[serde(rename = "@fw")]
    pub fw: u32,
    #[serde(rename = "@fh")]
    pub fh: u32,
    /// Path below `img/`.
    #[serde(rename = "$text")]
    pub path: String,
}

/// `achievements.xml`: id, name, description, image and ship of each achievement.
#[derive(Debug, Deserialize)]
pub(crate) struct AchievementsFile {
    #[serde(rename = "achievement", default)]
    pub achievements: Vec<AchievementXml>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AchievementXml {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(default)]
    pub name: Vec<Text>,
    #[serde(default)]
    pub desc: Vec<Text>,
    /// Path below `img/`.
    pub img: Option<String>,
    /// The player ship of a ship achievement.
    pub ship: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WeaponAnimXml {
    #[serde(rename = "@name")]
    pub name: String,
    pub sheet: String,
    /// Where the weapon sits on its mount, in pixels of one frame.
    #[serde(rename = "mountPoint")]
    pub mount_point: Option<Point>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrub_removes_declarations_and_ftl_roots() {
        let s = scrub(
            "\u{feff}<?xml version=\"1.0\"?>\n<FTL>\n<a/>\n</FTL>\n<?xml version=\"1.0\"?><b/>"
                .as_bytes(),
        );
        assert_eq!(s, "<root>\n\n<a/>\n\n<b/></root>");
    }

    #[test]
    fn resolve_prefers_the_lookup_and_turns_literal_breaks_into_newlines() {
        let mut lookup = TextLookup::default();
        lookup
            .add_file("t.xml", br#"<FTL><text name="a">one\ntwo</text></FTL>"#)
            .unwrap();
        let by_id = Text {
            id: Some("a".into()),
            text: String::new(),
        };
        let unknown_id = Text {
            id: Some("zz".into()),
            text: " own ".into(),
        };
        assert_eq!(lookup.resolve(&[by_id]), "one\ntwo");
        assert_eq!(lookup.resolve(&[unknown_id]), "own");
        assert_eq!(lookup.resolve(&[]), "");
    }
}

/// `data/sector_data.xml`: the sector lists the sector tree draws from.
#[derive(Debug, Deserialize)]
pub(crate) struct SectorDataFile {
    #[serde(rename = "sectorType", default)]
    pub types: Vec<SectorTypeXml>,
    #[serde(rename = "sectorDescription", default)]
    pub descriptions: Vec<SectorDescriptionXml>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SectorTypeXml {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "sector", default)]
    pub sectors: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SectorDescriptionXml {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@minSector", default)]
    pub min_sector: i32,
    #[serde(rename = "@unique", default)]
    pub unique: bool,
    #[serde(rename = "nameList", default)]
    pub name_list: Vec<SectorNameListXml>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SectorNameListXml {
    #[serde(rename = "name", default)]
    pub names: Vec<SectorNameXml>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SectorNameXml {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@short")]
    pub short: Option<String>,
    #[serde(rename = "$text", default)]
    pub text: String,
}
