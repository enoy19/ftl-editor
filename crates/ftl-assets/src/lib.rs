//! Reads FTL's resource package `ftl.dat` into plain values.
//!
//! [`GameData::open`] reads the package once (PKG since FTL 1.6.1, DAT
//! before, both through the crate `ftldat`), parses the blueprint XML, the
//! text lookup, the ship layouts and the animation sheets, copies the PNG
//! bytes it can answer later, and closes the file. There is no global state:
//! the caller owns the [`GameData`] and passes it where it is needed.
//!
//! The file list follows `DefaultDataManager` of Vhati's Java editor
//! (<https://github.com/Vhati/ftl-profile-editor>): the base blueprint files first, then the
//! DLC files, a later definition of an id replacing an earlier one.

mod font;
mod frames;
mod layout;
mod package;
mod slice;
mod xml;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub use font::{BitmapFont, FontError, Glyph};
pub use frames::{CREW_FRAME, Rect};
pub use layout::{ShipLayout, ShipLayoutDoor, ShipLayoutRoom, Slide, WeaponMount};
pub use slice::{Insets, measure_insets};

/// Base blueprint files, read in this order.
pub const STD_BLUEPRINT_FILES: [&str; 3] = ["blueprints.xml", "autoBlueprints.xml", "bosses.xml"];
/// Advanced Edition blueprint files, read after the base files.
pub const DLC_BLUEPRINT_FILES: [&str; 3] = [
    "dlcBlueprints.xml",
    "dlcBlueprintsOverwrite.xml",
    "dlcPirateBlueprints.xml",
];
/// Text lookup files (FTL 1.6.1+; `misc.xml` for 1.5.x). A missing file is skipped.
pub const TEXT_FILES: [&str; 8] = [
    "misc.xml",
    "text_achievements.xml",
    "text_blueprints.xml",
    "text_events.xml",
    "text_misc.xml",
    "text_sectorname.xml",
    "text_tooltips.xml",
    "text_tutorial.xml",
];
/// Translations of the text lookup by language code (FTL 1.6.1+). They hold
/// the same names as `text_*.xml`. Only the languages the app speaks are read.
pub const LOCALIZED_TEXT_FILES: [(&str, &str); 1] = [("de", "text-de.xml")];
/// Animation files that hold the weapon art sheets. A missing file is skipped.
pub const ANIMATION_FILES: [&str; 2] = ["animations.xml", "dlcAnimations.xml"];

/// The ten player ship families in the order of the hangar (the list of
/// `DefaultDataManager`; FTL hard-codes it too). Each has a Type-B variant
/// `<id>_2` and, except Crystal and Lanius, a Type-C variant `<id>_3`
/// (`dlcBlueprints.xml` keeps a Lanius Type-C inside a comment).
pub const PLAYER_SHIP_BASE_IDS: [&str; 10] = [
    "PLAYER_SHIP_HARD",
    "PLAYER_SHIP_STEALTH",
    "PLAYER_SHIP_MANTIS",
    "PLAYER_SHIP_CIRCLE",
    "PLAYER_SHIP_FED",
    "PLAYER_SHIP_JELLY",
    "PLAYER_SHIP_ROCK",
    "PLAYER_SHIP_ENERGY",
    "PLAYER_SHIP_CRYSTAL",
    "PLAYER_SHIP_ANAEROBIC",
];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{path}: cannot read the package: {detail}")]
    Package { path: PathBuf, detail: String },
    #[error("{inner_path} is missing in the package")]
    Missing { inner_path: String },
    #[error("{inner_path}: {detail}")]
    Parse { inner_path: String, detail: String },
    #[error("{inner_path}: {detail}")]
    Image { inner_path: String, detail: String },
    #[error("no sprite for {id}")]
    NoSprite { id: String },
}

/// One ship blueprint, names already resolved through the text lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipBlueprint {
    pub id: String,
    /// Base name of `data/<layout>.txt` and `data/<layout>.xml`.
    pub layout: String,
    /// Base name of the hull images under `img/ship/`.
    pub img: String,
    pub class_name: String,
    pub name: String,
    pub description: String,
    pub systems: Vec<ShipSystem>,
    /// Absent in enemy blueprints.
    pub weapon_slots: Option<u32>,
    pub drone_slots: Option<u32>,
    pub weapons: Vec<String>,
    pub drones: Vec<String>,
    pub augments: Vec<String>,
    pub missiles: u32,
    pub drone_parts: u32,
    pub hull: u32,
    /// Initial reactor power.
    pub max_power: u32,
    /// The starting crew, one entry per race.
    pub crew: Vec<CrewCount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrewCount {
    /// Crew blueprint id; FTL's default when the attribute is missing is `human`.
    pub race: String,
    pub amount: u32,
}

/// One entry of a ship's `systemList`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipSystem {
    /// The tag name, equal to the system blueprint id (`shields`, `pilot`, ...).
    pub system: String,
    pub power: u32,
    /// The ship's own cap, overriding the system blueprint's `maxPower`.
    pub max: Option<u32>,
    pub room: u32,
    /// Installed at start (FTL's default is true).
    pub start: bool,
    /// Base name of the room interior under `img/ship/interior/`, if the
    /// blueprint names one.
    pub img: Option<String>,
    /// `<slot>` of the entry: the square crew mans the system from and the
    /// direction it faces (`up`, `down`, `left`, `right`); `None` without
    /// one, then FTL picks a default per system.
    pub slot: Option<(i32, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrewBlueprint {
    pub id: String,
    pub title: String,
    pub short: String,
    pub description: String,
    pub cost: u32,
    /// `<colorList>`: per tint layer its colors. FTL draws layer `k` (from 1)
    /// as `img/people/<race>_layer<k>.png` tinted with the chosen color.
    pub color_layers: Vec<Vec<Rgba>>,
}

/// A color of a crew tint layer: `r`, `g`, `b` 0 to 255, `a` 0 to 255
/// (the blueprint's 0.0 to 1.0 times 255, rounded).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemBlueprint {
    pub id: String,
    pub title: String,
    pub description: String,
    pub start_power: u32,
    pub max_power: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponBlueprint {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub short: String,
    pub description: String,
    /// Id of the `weaponAnim` that draws this weapon.
    pub weapon_art: Option<String>,
    /// Reactor bars the weapon draws.
    pub power: u32,
    /// Time between shots in milliseconds: the seconds of `<cooldown>`
    /// times 1000 in `f32`, cut to an integer, as `FileHelper::writeFloat`
    /// stores a cooldown (FTL 1.6.13). `None` for the weapons of drones,
    /// which have no tag.
    pub cooldown_ms: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DroneBlueprint {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub short: String,
    pub description: String,
    /// Base name of the sprites under `img/ship/drones/`.
    pub drone_image: Option<String>,
    /// Reactor bars the drone draws.
    pub power: u32,
    /// Milliseconds between shots (`<cooldown>`); `None` for the drones
    /// without the tag (repair, anti-personnel, boarding, hacking).
    pub cooldown: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AugmentBlueprint {
    pub id: String,
    pub title: String,
    pub description: String,
}

/// The sprite strip of a weapon: a row of `frames` frames, each
/// `frame_width` x `frame_height` pixels; `mount_point` is the pixel of a
/// frame that sits on the mount (`<mountPoint>` of its `weaponAnim`, 0, 0
/// without one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponArt<'a> {
    pub png: &'a [u8],
    pub frame_width: u32,
    pub frame_height: u32,
    pub frames: u32,
    pub mount_point: (i32, i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AnimSheet {
    path: String,
    width: u32,
    frame_width: u32,
    frame_height: u32,
}

/// A `weaponAnim`: its sheet and its mount point.
#[derive(Debug, Clone, PartialEq, Eq)]
struct WeaponAnim {
    sheet: AnimSheet,
    mount_point: (i32, i32),
}

/// Everything the editor reads from `ftl.dat`.
#[derive(Debug, Clone, Default)]
pub struct GameData {
    ships: BTreeMap<String, ShipBlueprint>,
    crew: BTreeMap<String, CrewBlueprint>,
    systems: BTreeMap<String, SystemBlueprint>,
    weapons: BTreeMap<String, WeaponBlueprint>,
    drones: BTreeMap<String, DroneBlueprint>,
    augments: BTreeMap<String, AugmentBlueprint>,
    layouts: BTreeMap<String, ShipLayout>,
    /// weaponAnim id -> its sheet and mount point.
    weapon_anims: BTreeMap<String, WeaponAnim>,
    /// Inner path -> PNG bytes, only for the sprites the accessors answer.
    images: BTreeMap<String, Vec<u8>>,
    /// The achievements in the order of `achievements.xml`, then the ship
    /// quests and victories.
    achievements: Vec<Achievement>,
    /// Language code -> achievement id -> translated name and description.
    achievement_texts_in: BTreeMap<String, BTreeMap<String, (String, String)>>,
    /// Blueprint id -> text id of its name (title, or name of a ship).
    title_ids: BTreeMap<String, String>,
    /// Ship blueprint id -> text id of its class (`Kestrel Cruiser`).
    class_ids: BTreeMap<String, String>,
    /// Language code -> blueprint id -> translated name.
    names_in: BTreeMap<String, BTreeMap<String, String>>,
    /// The English human crew names of `data/names.xml`, in file order.
    male_names: Vec<String>,
    female_names: Vec<String>,
    /// The sector lists of `data/sector_data.xml`.
    sectors: SectorData,
    /// Language code (`en` and those of [`LOCALIZED_TEXT_FILES`]) -> text
    /// id of a sector name -> its text.
    sector_texts: BTreeMap<String, BTreeMap<String, String>>,
}

/// The sector lists of `data/sector_data.xml` (`sectorType`, with the
/// `OVERRIDE_` lists, and `sectorDescription`), as far as the sector tree
/// reads them. Empty without the file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SectorData {
    pub types: BTreeMap<String, Vec<String>>,
    pub descriptions: BTreeMap<String, SectorInfo>,
}

/// One `sectorDescription`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SectorInfo {
    pub min_sector: i32,
    pub unique: bool,
    /// The `nameList`, in file order.
    pub names: Vec<SectorNameInfo>,
}

/// One `<name>`: text ids (`id`, `short`) or a literal (`is_id` false).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SectorNameInfo {
    pub text: String,
    pub short: String,
    pub is_id: bool,
}

/// What an achievement is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AchievementKind {
    /// Any ship: `ACH_SECTOR_5`.
    General,
    /// One of the three of a player ship; two unlock its Type-B.
    Ship,
    /// The unlock quest of a ship, `PLAYER_SHIP_<X>_QUEST`.
    Quest,
    /// A win with a ship, `PLAYER_SHIP_<X>_VICTORY`.
    Victory,
}

/// One achievement of `ftl.dat`, its texts in English.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Inner path of the image.
    pub image: Option<String>,
    /// The player ship of a ship, quest or victory achievement.
    pub ship: Option<String>,
    pub kind: AchievementKind,
}

/// The ships with an unlock quest. `achievements.xml` lists no quest and no
/// victory: FTL hard-codes them, and so does the Java editor
/// (`DefaultDataManager`). Their names are `quest_ach_title` and
/// `victory_ach_title` of `text_misc.xml` with the ship class, their images
/// the hangar markers `S_Q_on` and `S_V_on`.
pub const QUEST_SHIP_IDS: [&str; 7] = [
    "PLAYER_SHIP_STEALTH",
    "PLAYER_SHIP_MANTIS",
    "PLAYER_SHIP_FED",
    "PLAYER_SHIP_JELLY",
    "PLAYER_SHIP_ROCK",
    "PLAYER_SHIP_ENERGY",
    "PLAYER_SHIP_CRYSTAL",
];
const QUEST_IMAGE: &str = "img/achievements/S_Q_on.png";
const VICTORY_IMAGE: &str = "img/achievements/S_V_on.png";

impl GameData {
    /// Reads `ftl.dat` (PKG or DAT, told apart by the `PKG\n` signature).
    /// A panic of ftldat on a corrupt package becomes [`Error::Package`].
    pub fn open(path: impl AsRef<Path>) -> Result<GameData, Error> {
        let path = path.as_ref();
        without_panic(path, || load(&package::open(path)?))
    }

    pub fn ships(&self) -> impl Iterator<Item = &ShipBlueprint> {
        self.ships.values()
    }
    pub fn ship(&self, id: &str) -> Option<&ShipBlueprint> {
        self.ships.get(id)
    }
    pub fn crew_races(&self) -> impl Iterator<Item = &CrewBlueprint> {
        self.crew.values()
    }
    pub fn crew(&self, id: &str) -> Option<&CrewBlueprint> {
        self.crew.get(id)
    }
    pub fn systems(&self) -> impl Iterator<Item = &SystemBlueprint> {
        self.systems.values()
    }
    pub fn system(&self, id: &str) -> Option<&SystemBlueprint> {
        self.systems.get(id)
    }
    pub fn weapons(&self) -> impl Iterator<Item = &WeaponBlueprint> {
        self.weapons.values()
    }
    pub fn weapon(&self, id: &str) -> Option<&WeaponBlueprint> {
        self.weapons.get(id)
    }
    pub fn drones(&self) -> impl Iterator<Item = &DroneBlueprint> {
        self.drones.values()
    }
    pub fn drone(&self, id: &str) -> Option<&DroneBlueprint> {
        self.drones.get(id)
    }
    pub fn augments(&self) -> impl Iterator<Item = &AugmentBlueprint> {
        self.augments.values()
    }
    pub fn augment(&self, id: &str) -> Option<&AugmentBlueprint> {
        self.augments.get(id)
    }

    /// The player ships in hangar order: each family, then its Type-B and
    /// Type-C variants. Ids the package does not define are skipped.
    pub fn player_ships(&self) -> Vec<&ShipBlueprint> {
        PLAYER_SHIP_BASE_IDS
            .iter()
            .flat_map(|base| [base.to_string(), format!("{base}_2"), format!("{base}_3")])
            .filter_map(|id| self.ships.get(&id))
            .collect()
    }

    /// The name the game shows for a blueprint id of any kind: the ship name,
    /// else the title of a crew race, system, weapon, drone or augment.
    pub fn display_name(&self, id: &str) -> Option<&str> {
        let name = self
            .ships
            .get(id)
            .map(|s| &s.name)
            .or_else(|| self.crew.get(id).map(|c| &c.title))
            .or_else(|| self.systems.get(id).map(|s| &s.title))
            .or_else(|| self.weapons.get(id).map(|w| &w.title))
            .or_else(|| self.drones.get(id).map(|d| &d.title))
            .or_else(|| self.augments.get(id).map(|a| &a.title))?;
        Some(name.as_str())
    }

    /// [`GameData::display_name`] in `language` (`de`): the translation of
    /// [`LOCALIZED_TEXT_FILES`] when the blueprint names its text by id and
    /// the file has it, else the English name.
    pub fn display_name_in(&self, id: &str, language: &str) -> Option<&str> {
        self.names_in
            .get(language)
            .and_then(|names| names.get(id))
            .map(String::as_str)
            .or_else(|| self.display_name(id))
    }

    /// The highest level `system` can reach on ship `ship_id`: the ship's own
    /// `max` attribute, else `maxPower` of the system blueprint (the rule of
    /// the Java editor's floorplan panel). `None` when the ship lacks the system.
    pub fn max_system_level(&self, ship_id: &str, system: &str) -> Option<u32> {
        let entry = self
            .ships
            .get(ship_id)?
            .systems
            .iter()
            .find(|s| s.system == system)?;
        entry
            .max
            .or_else(|| self.systems.get(system).map(|b| b.max_power))
    }

    /// The layout by its id (`ShipBlueprint::layout`).
    pub fn layout(&self, layout_id: &str) -> Option<&ShipLayout> {
        self.layouts.get(layout_id)
    }

    pub fn ship_layout(&self, ship_id: &str) -> Option<&ShipLayout> {
        self.layout(&self.ships.get(ship_id)?.layout)
    }

    /// `img/people/<race>_base.png`, the grey base sprite sheet of a crew race
    /// (`female` for a female human; `ghost` uses the human sheet).
    pub fn crew_sprite(&self, race: &str) -> Option<&[u8]> {
        self.image(&crew_sprite_path(race))
    }

    /// The standing frame ([`CREW_FRAME`]) of a crew race as PNG bytes.
    pub fn crew_frame(&self, race: &str) -> Result<Vec<u8>, Error> {
        let path = crew_sprite_path(race);
        let sheet = self.image(&path).ok_or_else(|| Error::NoSprite {
            id: race.to_owned(),
        })?;
        frames::crop_png(&path, sheet, CREW_FRAME)
    }

    /// The standing frame of a crew race with its visible pixels centered
    /// in the 35x35 cell, for a portrait. The floorplan keeps
    /// [`GameData::crew_frame`]: FTL anchors the cell, not the art.
    pub fn crew_portrait(&self, race: &str) -> Result<Vec<u8>, Error> {
        frames::center_opaque(&crew_sprite_path(race), &self.crew_frame(race)?)
    }

    /// The English human crew names of `data/names.xml` (the lists without
    /// a `language`), male or female, in file order.
    pub fn crew_names(&self, male: bool) -> &[String] {
        if male {
            &self.male_names
        } else {
            &self.female_names
        }
    }

    /// The portrait of a crew member as FTL draws its look: the standing
    /// frame of the race sheet ([`crew_sheet`]: female humans wear
    /// `female`), each tint layer `<sheet>_layer<k>.png` multiplied with the
    /// color `tints[k-1]` picks from layer `k` of the blueprint's
    /// `colorList` and laid over it (`CrewAnimation::SetupStrips`, FTL
    /// 1.6.13), centered like [`GameData::crew_portrait`]. A missing tint
    /// takes color 0; a layer without a sheet or color is left out.
    pub fn crew_look(&self, race: &str, male: bool, tints: &[i32]) -> Result<Vec<u8>, Error> {
        let frame = self.crew_look_frame(race, male, tints)?;
        frames::center_opaque(&crew_sprite_path(crew_sheet(race, male)), &frame)
    }

    /// [`GameData::crew_look`] anchored as FTL anchors the cell, for the
    /// floorplan (the art is not centered in its cell).
    pub fn crew_look_frame(&self, race: &str, male: bool, tints: &[i32]) -> Result<Vec<u8>, Error> {
        let sheet = crew_sheet(race, male);
        let base_path = crew_sprite_path(sheet);
        let base = self.image(&base_path).ok_or_else(|| Error::NoSprite {
            id: race.to_owned(),
        })?;
        let blueprint_race = if race == "ghost" { "human" } else { race };
        let colors = self
            .crew
            .get(blueprint_race)
            .map_or(&[][..], |b| b.color_layers.as_slice());
        let mut layers = Vec::new();
        for (k, choices) in colors.iter().enumerate() {
            let index = tints.get(k).copied().unwrap_or(0).max(0) as usize;
            let (Some(color), Some(png)) = (
                choices.get(index).or(choices.first()),
                self.image(&crew_layer_path(sheet, k)),
            ) else {
                continue;
            };
            layers.push((crew_layer_path(sheet, k), png, *color));
        }
        frames::tinted_frame(&base_path, base, &layers, CREW_FRAME)
    }

    /// The first frame of a weapon's strip as PNG bytes (the weapon at rest).
    pub fn weapon_frame(&self, weapon_id: &str) -> Result<Vec<u8>, Error> {
        let no_sprite = || Error::NoSprite {
            id: weapon_id.to_owned(),
        };
        let anim = self
            .weapons
            .get(weapon_id)
            .and_then(|w| w.weapon_art.as_ref())
            .ok_or_else(no_sprite)?;
        let sheet = &self.weapon_anims.get(anim).ok_or_else(no_sprite)?.sheet;
        let png = self.image(&sheet.path).ok_or_else(no_sprite)?;
        frames::crop_png(
            &sheet.path,
            png,
            (0, 0, sheet.frame_width, sheet.frame_height),
        )
    }

    /// `img/icons/s_<system>_overlay.png`, the white system icon.
    pub fn system_icon(&self, system: &str) -> Option<&[u8]> {
        self.image(&system_icon_path(system))
    }

    /// The sprite strip of a weapon blueprint, through its `weaponArt` animation.
    pub fn weapon_art(&self, weapon_id: &str) -> Option<WeaponArt<'_>> {
        let anim = self.weapons.get(weapon_id)?.weapon_art.as_ref()?;
        let WeaponAnim { sheet, mount_point } = self.weapon_anims.get(anim)?;
        Some(WeaponArt {
            png: self.image(&sheet.path)?,
            frame_width: sheet.frame_width,
            frame_height: sheet.frame_height,
            frames: sheet.width.checked_div(sheet.frame_width).unwrap_or(0),
            mount_point: *mount_point,
        })
    }

    /// The hull image of a ship blueprint, in the order to try:
    /// `img/ship/<img>_base.png` (player ships), then
    /// `img/ships_glow/<img>_base.png` (enemy ships since FTL 1.5.4), as
    /// `ShipBundle` of the Java editor. Not kept in memory: read them with
    /// [`read_entry`].
    pub fn ship_image_paths(&self, ship_id: &str) -> Option<[String; 2]> {
        let img = &self.ships.get(ship_id)?.img;
        Some([
            format!("img/ship/{img}_base.png"),
            format!("img/ships_glow/{img}_base.png"),
        ])
    }

    /// The sprite of a drone blueprint, in the order to try:
    /// `img/ship/drones/<droneImage>_base.png`, then `..._off.png` (the shield
    /// drone of FTL 1.6.13 has no `_base` sprite).
    pub fn drone_image_paths(&self, drone_id: &str) -> Option<[String; 2]> {
        let image = self.drones.get(drone_id)?.drone_image.as_ref()?;
        Some(["base", "off"].map(|suffix| format!("img/ship/drones/{image}_{suffix}.png")))
    }

    /// The image of an achievement (`img/achievements/...`): from
    /// `achievements.xml`, for a ship quest or victory FTL's hangar marker.
    pub fn achievement_image_path(&self, achievement_id: &str) -> Option<&str> {
        self.achievement(achievement_id)?.image.as_deref()
    }

    /// Every achievement: `achievements.xml` in file order, then the ship
    /// quests and victories (see [`QUEST_SHIP_IDS`]).
    pub fn achievements(&self) -> &[Achievement] {
        &self.achievements
    }

    /// The sector lists of `data/sector_data.xml`.
    pub fn sectors(&self) -> &SectorData {
        &self.sectors
    }

    /// The text of a sector name id in `language` (`en`, `de`), else in
    /// English; `None` for an id no text file names.
    pub fn sector_text_in(&self, id: &str, language: &str) -> Option<&str> {
        let get = |l: &str| self.sector_texts.get(l).and_then(|t| t.get(id));
        get(language).or_else(|| get("en")).map(String::as_str)
    }

    pub fn achievement(&self, id: &str) -> Option<&Achievement> {
        self.achievements.iter().find(|a| a.id == id)
    }

    /// Name and description of an achievement in `language` (`de`), else in
    /// English.
    pub fn achievement_text_in(&self, id: &str, language: &str) -> Option<(&str, &str)> {
        if let Some((name, desc)) = self
            .achievement_texts_in
            .get(language)
            .and_then(|t| t.get(id))
        {
            return Some((name, desc));
        }
        let a = self.achievement(id)?;
        Some((&a.name, &a.description))
    }

    fn image(&self, inner_path: &str) -> Option<&[u8]> {
        self.images.get(inner_path).map(Vec::as_slice)
    }
}

/// Reads one entry of the package at `path`, for the images [`GameData`]
/// does not keep (ship hulls, drones, achievements, any inner path). Each call
/// opens the package again; a caller that asks often keeps a cache.
pub fn read_entry(path: impl AsRef<Path>, inner_path: &str) -> Result<Vec<u8>, Error> {
    let path = path.as_ref();
    without_panic(path, || package::open(path)?.require(inner_path))
}

/// Every inner path of the package at `path`, in package order.
pub fn entry_paths(path: impl AsRef<Path>) -> Result<Vec<String>, Error> {
    let path = path.as_ref();
    without_panic(path, || Ok(package::open(path)?.inner_paths()))
}

/// Runs `f`, turning a panic of ftldat into [`Error::Package`].
///
/// ftldat panics instead of failing on some corrupt packages (unchecked
/// slices, `expect`); the default panic hook still prints one line to stderr.
fn without_panic<T>(path: &Path, f: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|panic| {
        let detail = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap_or("unknown panic");
        Err(Error::Package {
            path: path.to_owned(),
            detail: format!("corrupt package: {detail}"),
        })
    })
}

/// A color channel of `ftl.dat`, cut to 0..=255.
fn channel(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// The tint layer sheets `<sheet>_layer<k>.png` the editor keeps per race
/// sheet: FTL 1.6.13 has at most two (human and female).
const MAX_TINT_LAYERS: usize = 4;

fn crew_layer_path(sheet: &str, layer: usize) -> String {
    format!("img/people/{sheet}_layer{}.png", layer + 1)
}

/// The sheet FTL draws for a race and sex: `female` for a female human,
/// `human` for a ghost.
pub fn crew_sheet(race: &str, male: bool) -> &str {
    match race {
        "human" if !male => "female",
        "ghost" => "human",
        _ => race,
    }
}

/// Sheets without a crew blueprint of their own: `female` (female humans).
const EXTRA_CREW_SHEETS: [&str; 1] = ["female"];

fn crew_sprite_path(race: &str) -> String {
    // Ghost crew are humans with reduced opacity (DefaultSpriteImageProvider).
    let sheet = if race == "ghost" { "human" } else { race };
    format!("img/people/{sheet}_base.png")
}

fn system_icon_path(system: &str) -> String {
    format!("img/icons/s_{system}_overlay.png")
}

fn load(pkg: &package::Package) -> Result<GameData, Error> {
    let mut text = xml::TextLookup::default();
    for name in TEXT_FILES {
        let path = format!("data/{name}");
        if let Some(bytes) = pkg.bytes(&path) {
            text.add_file(&path, &bytes)?;
        }
    }

    let mut g = GameData::default();
    for (i, name) in STD_BLUEPRINT_FILES
        .iter()
        .chain(&DLC_BLUEPRINT_FILES)
        .enumerate()
    {
        let path = format!("data/{name}");
        // blueprints.xml is required; the others came with later versions or the DLC.
        let bytes = match pkg.bytes(&path) {
            Some(b) => b,
            None if i == 0 => return Err(pkg.require(&path).unwrap_err()),
            None => continue,
        };
        let file: xml::BlueprintsFile = xml::parse(&path, &bytes)?;
        g.add_blueprints(file, &text);
    }

    // Achievement id -> text ids of its name and description, for the translations.
    let mut achievement_text_ids = Vec::new();
    let achievements = "data/achievements.xml";
    if let Some(bytes) = pkg.bytes(achievements) {
        let file: xml::AchievementsFile = xml::parse(achievements, &bytes)?;
        for a in file.achievements {
            achievement_text_ids.push((
                a.id.clone(),
                xml::Text::id_of(&a.name).map(str::to_owned),
                xml::Text::id_of(&a.desc).map(str::to_owned),
            ));
            let kind = if a.ship.is_some() {
                AchievementKind::Ship
            } else {
                AchievementKind::General
            };
            g.achievements.push(Achievement {
                name: text.resolve(&a.name),
                description: text.resolve(&a.desc),
                image: a.img.map(|img| format!("img/{}", img.trim())),
                ship: a.ship.map(|s| s.trim().to_owned()),
                kind,
                id: a.id,
            });
        }
    }
    g.add_ship_achievements(|id| text.get(id));

    let sector_data = "data/sector_data.xml";
    if let Some(bytes) = pkg.bytes(sector_data) {
        let file: xml::SectorDataFile = xml::parse(sector_data, &bytes)?;
        g.add_sectors(file);
    }
    let english = g
        .sector_text_ids()
        .filter_map(|id| Some((id.clone(), text.get(id)?)))
        .collect();
    g.sector_texts.insert("en".to_owned(), english);

    for (language, name) in LOCALIZED_TEXT_FILES {
        let path = format!("data/{name}");
        let Some(bytes) = pkg.bytes(&path) else {
            continue;
        };
        let mut lookup = xml::TextLookup::default();
        lookup.add_file(&path, &bytes)?;
        let names = g
            .title_ids
            .iter()
            .filter_map(|(id, text)| Some((id.clone(), lookup.get(text)?)))
            .collect();
        g.names_in.insert(language.to_owned(), names);
        let mut texts: BTreeMap<String, (String, String)> = achievement_text_ids
            .iter()
            .filter_map(|(id, name, desc)| {
                let name = lookup.get(name.as_deref()?)?;
                let desc = desc
                    .as_deref()
                    .and_then(|d| lookup.get(d))
                    .unwrap_or_default();
                Some((id.clone(), (name, desc)))
            })
            .collect();
        texts.extend(g.ship_achievement_texts(|id| lookup.get(id)));
        g.achievement_texts_in.insert(language.to_owned(), texts);
        let sector_names = g
            .sector_text_ids()
            .filter_map(|id| Some((id.clone(), lookup.get(id)?)))
            .collect();
        g.sector_texts.insert(language.to_owned(), sector_names);
    }

    let layout_ids: std::collections::BTreeSet<String> =
        g.ships.values().map(|s| s.layout.clone()).collect();
    for id in layout_ids {
        let txt = format!("data/{id}.txt");
        // 1.6.13 has a layout for every blueprint; a modded package may not.
        let Some(bytes) = pkg.bytes(&txt) else {
            continue;
        };
        let mut layout = layout::parse_txt(&txt, &bytes)?;
        let chassis = format!("data/{id}.xml");
        if let Some(bytes) = pkg.bytes(&chassis) {
            layout::add_chassis(&mut layout, &chassis, &bytes)?;
        }
        g.layouts.insert(id, layout);
    }

    let mut sheets = BTreeMap::new();
    let mut weapon_anims = Vec::new();
    for name in ANIMATION_FILES {
        let path = format!("data/{name}");
        let Some(bytes) = pkg.bytes(&path) else {
            continue;
        };
        let file: xml::AnimationsFile = xml::parse(&path, &bytes)?;
        for s in file.sheets {
            let sheet = AnimSheet {
                path: format!("img/{}", s.path.trim()),
                width: s.w,
                frame_width: s.fw,
                frame_height: s.fh,
            };
            sheets.insert(s.name, sheet);
        }
        weapon_anims.extend(file.weapon_anims);
    }
    for a in weapon_anims {
        if let Some(sheet) = sheets.get(a.sheet.trim()) {
            let mount_point = a.mount_point.map_or((0, 0), |p| (p.x, p.y));
            let anim = WeaponAnim {
                sheet: sheet.clone(),
                mount_point,
            };
            g.weapon_anims.insert(a.name, anim);
        }
    }

    let sheets: Vec<&str> = g
        .crew
        .keys()
        .map(String::as_str)
        .chain(EXTRA_CREW_SHEETS)
        .collect();
    let layers = sheets
        .iter()
        .flat_map(|sheet| (0..MAX_TINT_LAYERS).map(|k| crew_layer_path(sheet, k)));
    let wanted: Vec<String> = sheets
        .iter()
        .copied()
        .map(crew_sprite_path)
        .chain(layers)
        .chain(g.systems.keys().map(|s| system_icon_path(s)))
        .chain(g.weapon_anims.values().map(|a| a.sheet.path.clone()))
        .collect();
    for path in wanted {
        if let Some(bytes) = pkg.bytes(&path) {
            g.images.insert(path, bytes);
        }
    }

    let names = "data/names.xml";
    if let Some(bytes) = pkg.bytes(names) {
        let file: xml::NamesFile = xml::parse(names, &bytes)?;
        for list in file.lists {
            if list.race != "human" || list.language.is_some() {
                continue;
            }
            let into = if list.sex == "female" {
                &mut g.female_names
            } else {
                &mut g.male_names
            };
            into.extend(list.names.into_iter().map(|n| n.trim().to_owned()));
        }
    }
    Ok(g)
}

impl GameData {
    /// Adds one blueprint file; an id defined again replaces the earlier one.
    /// Remembers the text id of a blueprint's name; a later definition
    /// without one drops it.
    /// Name and description of each ship quest and victory, from the text
    /// templates of `text_misc.xml` (`\1` is the ship class) in the language
    /// of `lookup`. A ship without a blueprint or a template is left out.
    fn ship_achievement_texts(
        &self,
        lookup: impl Fn(&str) -> Option<String>,
    ) -> Vec<(String, (String, String))> {
        let mut out = Vec::new();
        for ship in PLAYER_SHIP_BASE_IDS {
            let Some(class) = self
                .class_ids
                .get(ship)
                .and_then(|id| lookup(id))
                .or_else(|| Some(self.ships.get(ship)?.class_name.clone()))
            else {
                continue;
            };
            let fill = |key: &str| Some(lookup(key)?.replace("\\1", &class));
            let mut kinds = vec![("VICTORY", "victory_ach_title", "victory_ach")];
            if QUEST_SHIP_IDS.contains(&ship) {
                kinds.insert(0, ("QUEST", "quest_ach_title", "quest_ach"));
            }
            for (suffix, title, desc) in kinds {
                if let (Some(name), Some(desc)) = (fill(title), fill(desc)) {
                    out.push((format!("{ship}_{suffix}"), (name, desc)));
                }
            }
        }
        out
    }

    fn add_sectors(&mut self, file: xml::SectorDataFile) {
        for t in file.types {
            let ids = t.sectors.iter().map(|s| s.trim().to_owned()).collect();
            self.sectors.types.insert(t.name, ids);
        }
        for d in file.descriptions {
            let names = d
                .name_list
                .into_iter()
                .next()
                .unwrap_or_default()
                .names
                .into_iter()
                .map(|n| match n.id {
                    Some(id) => SectorNameInfo {
                        short: n.short.unwrap_or_else(|| id.clone()),
                        text: id,
                        is_id: true,
                    },
                    None => SectorNameInfo {
                        text: n.text.trim().to_owned(),
                        short: n.short.unwrap_or_else(|| n.text.trim().to_owned()),
                        is_id: false,
                    },
                })
                .collect();
            self.sectors.descriptions.insert(
                d.name,
                SectorInfo {
                    min_sector: d.min_sector,
                    unique: d.unique,
                    names,
                },
            );
        }
    }

    /// Every text id a sector name or short name uses.
    fn sector_text_ids(&self) -> impl Iterator<Item = &String> {
        self.sectors
            .descriptions
            .values()
            .flat_map(|d| &d.names)
            .filter(|n| n.is_id)
            .flat_map(|n| [&n.text, &n.short])
    }

    /// Adds the ship quests and victories that `achievements.xml` lacks.
    fn add_ship_achievements(&mut self, lookup: impl Fn(&str) -> Option<String>) {
        let texts: BTreeMap<String, (String, String)> =
            self.ship_achievement_texts(lookup).into_iter().collect();
        for ship in PLAYER_SHIP_BASE_IDS {
            if !self.ships.contains_key(ship) {
                continue;
            }
            let quests = QUEST_SHIP_IDS.contains(&ship).then_some((
                "QUEST",
                AchievementKind::Quest,
                QUEST_IMAGE,
            ));
            for (suffix, kind, image) in
                quests
                    .into_iter()
                    .chain([("VICTORY", AchievementKind::Victory, VICTORY_IMAGE)])
            {
                let id = format!("{ship}_{suffix}");
                if self.achievements.iter().any(|a| a.id == id) {
                    continue;
                }
                let (name, description) = texts
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| (id.clone(), String::new()));
                self.achievements.push(Achievement {
                    id,
                    name,
                    description,
                    image: Some(image.to_owned()),
                    ship: Some(ship.to_owned()),
                    kind,
                });
            }
        }
    }

    fn note_title(&mut self, id: &str, texts: &[xml::Text]) {
        match xml::Text::id_of(texts) {
            Some(text) => self.title_ids.insert(id.to_owned(), text.to_owned()),
            None => self.title_ids.remove(id),
        };
    }

    fn add_blueprints(&mut self, file: xml::BlueprintsFile, text: &xml::TextLookup) {
        for s in file.ships {
            self.note_title(&s.id, &s.name);
            if let Some(class) = xml::Text::id_of(&s.class) {
                self.class_ids.insert(s.id.clone(), class.to_owned());
            }
            let ship = ShipBlueprint {
                class_name: text.resolve(&s.class),
                name: text.resolve(&s.name),
                description: text.resolve(&s.desc),
                systems: s
                    .system_list
                    .entries
                    .into_iter()
                    .map(|e| {
                        let (system, r) = e.split();
                        ShipSystem {
                            system: system.to_owned(),
                            power: r.power,
                            max: r.max,
                            room: r.room,
                            start: r.start.unwrap_or(true),
                            img: r.img,
                            slot: r.slot.and_then(|s| Some((s.number?, s.direction?))),
                        }
                    })
                    .collect(),
                weapon_slots: s.weapon_slots,
                drone_slots: s.drone_slots,
                missiles: s.weapon_list.first().map_or(0, |w| w.missiles),
                weapons: s
                    .weapon_list
                    .into_iter()
                    .next()
                    .map(|w| w.weapons.into_iter().map(|n| n.name).collect())
                    .unwrap_or_default(),
                // First wins for a repeated element. PLAYER_SHIP_FED has two droneList
                // (drones="2", then drones="0"): this answers 2, the Java editor 0.
                // What the game itself takes is unchecked.
                drone_parts: s.drone_list.first().map_or(0, |d| d.drones),
                drones: s
                    .drone_list
                    .into_iter()
                    .next()
                    .map(|d| d.drones_list.into_iter().map(|n| n.name).collect())
                    .unwrap_or_default(),
                augments: s.augs.into_iter().map(|a| a.name).collect(),
                hull: s.health.first().map_or(0, |h| h.amount),
                max_power: s.max_power.first().map_or(0, |p| p.amount),
                crew: s
                    .crew_count
                    .into_iter()
                    .map(|c| CrewCount {
                        race: c.class.unwrap_or_else(|| "human".into()),
                        amount: c.amount,
                    })
                    .collect(),
                layout: s.layout,
                img: s.img,
                id: s.id,
            };
            self.ships.insert(ship.id.clone(), ship);
        }
        for c in file.crew {
            self.note_title(&c.id, &c.title);
            let crew = CrewBlueprint {
                title: text.resolve(&c.title),
                short: text.resolve(&c.short),
                description: text.resolve(&c.desc),
                cost: c.cost.unwrap_or(0),
                color_layers: c
                    .color_list
                    .layers
                    .iter()
                    .map(|l| {
                        l.colors
                            .iter()
                            .map(|c| Rgba {
                                r: channel(c.r),
                                g: channel(c.g),
                                b: channel(c.b),
                                a: channel(c.a * 255.0),
                            })
                            .collect()
                    })
                    .collect(),
                id: c.id,
            };
            self.crew.insert(crew.id.clone(), crew);
        }
        for s in file.systems {
            self.note_title(&s.id, &s.title);
            let system = SystemBlueprint {
                title: text.resolve(&s.title),
                description: text.resolve(&s.desc),
                start_power: s.start_power,
                max_power: s.max_power,
                id: s.id,
            };
            self.systems.insert(system.id.clone(), system);
        }
        for w in file.weapons {
            self.note_title(&w.id, &w.title);
            let weapon = WeaponBlueprint {
                title: text.resolve(&w.title),
                // First wins for a repeated element. DRONE_MISSILE has two <short>
                // ("M. 0", then "STUPID MISSILES"): this answers "M. 0", the Java
                // editor the second. What the game itself shows is unchecked.
                short: text.resolve(&w.short),
                description: text.resolve(&w.desc),
                kind: w.kind.trim().to_owned(),
                weapon_art: w.weapon_art.map(|a| a.trim().to_owned()),
                power: w.power.first().copied().unwrap_or(0),
                cooldown_ms: w.cooldown.first().map(|s| (s * 1000.0) as i32),
                id: w.id,
            };
            self.weapons.insert(weapon.id.clone(), weapon);
        }
        for d in file.drones {
            self.note_title(&d.id, &d.title);
            let drone = DroneBlueprint {
                title: text.resolve(&d.title),
                short: text.resolve(&d.short),
                description: text.resolve(&d.desc),
                kind: d.kind.trim().to_owned(),
                drone_image: d.drone_image.map(|i| i.trim().to_owned()),
                power: d.power.first().copied().unwrap_or(0),
                cooldown: d.cooldown.first().copied(),
                id: d.id,
            };
            self.drones.insert(drone.id.clone(), drone);
        }
        for a in file.augments {
            self.note_title(&a.id, &a.title);
            let augment = AugmentBlueprint {
                title: text.resolve(&a.title),
                description: text.resolve(&a.desc),
                id: a.id,
            };
            self.augments.insert(augment.id.clone(), augment);
        }
    }
}
