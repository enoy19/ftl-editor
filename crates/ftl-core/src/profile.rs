//! The player profile (`ae_prof.sav`, legacy `prof.sav`): ship unlocks,
//! achievements, high scores and lifetime stats.
//!
//! The codec in `ftl-sav` reads the file as `ProfileParser.java` of the Java
//! editor does. This module holds the domain and the pure edit
//! [`unlock_all_ships`].

use serde::{Serialize, Serializer};

use crate::model::Difficulty;

/// File format number, the first int of the file.
///
/// | value | file | FTL version |
/// |---|---|---|
/// | 4 | `prof.sav` | 1.01 to 1.03.3 |
/// | 9 | `ae_prof.sav` | 1.5.4 and later (UTF-8 strings since 1.6.1) |
pub type ProfileFormat = i32;

/// Which hint the ship list shows. Format 9 only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NewbieTipLevel {
    ShipsUnlocked,
    ShipListIntro,
    Veteran,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Profile {
    pub file_format: ProfileFormat,
    /// `Some` in format 9, `None` in format 4.
    pub newbie_tip_level: Option<NewbieTipLevel>,
    pub achievements: Vec<AchievementRecord>,
    /// The twelve ship slots of the file, in file order. Format 9 has ten
    /// ships and two unused slots (`UNKNOWN_BETA`, `UNKNOWN_GAMMA`).
    pub ships: Vec<ShipAvailability>,
    pub top_scores: Vec<Score>,
    /// The best score of each ship.
    pub ship_best: Vec<Score>,
    pub stats: ProfileStats,
    pub crew_records: CrewRecords,
    /// Bytes after the last decoded field, kept verbatim. Empty in every
    /// observed file.
    #[serde(serialize_with = "len_only")]
    pub undecoded: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AchievementRecord {
    pub id: String,
    pub difficulty: Difficulty,
    /// Victory achievements in format 9 only: the highest difficulty each
    /// layout (A, B, C) won with, `None` for a layout that has not won.
    pub layouts: Option<[Option<Difficulty>; 3]>,
}

/// Type-A and Type-C unlock of one ship slot. Type-B has no flag: FTL
/// unlocks it when two of the ship's three achievements are present.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShipAvailability {
    pub ship_id: String,
    pub unlocked_a: bool,
    /// Format 9 only; always false in format 4.
    pub unlocked_c: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Score {
    pub ship_name: String,
    pub ship_id: String,
    pub value: i32,
    /// One-based sector the run ended in.
    pub sector: i32,
    pub victory: bool,
    pub difficulty: Difficulty,
    /// Format 9 only; false in format 4.
    pub dlc_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileStats {
    pub most_ships_defeated: i32,
    pub total_ships_defeated: i32,
    pub most_beacons_explored: i32,
    pub total_beacons_explored: i32,
    pub most_scrap_collected: i32,
    pub total_scrap_collected: i32,
    pub most_crew_hired: i32,
    pub total_crew_hired: i32,
    pub total_games_played: i32,
    pub total_victories: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CrewRecord {
    pub value: i32,
    pub name: String,
    pub race: String,
    pub male: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CrewRecords {
    pub most_repairs: CrewRecord,
    pub most_combat_kills: CrewRecord,
    pub most_piloted_evasions: CrewRecord,
    pub most_jumps_survived: CrewRecord,
    pub most_skill_masteries: CrewRecord,
}

/// A player ship of vanilla FTL as the profile knows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerShip {
    pub id: &'static str,
    /// The three ship achievements; two of them unlock Type-B.
    pub achievements: [&'static str; 3],
    pub has_layout_c: bool,
    /// Advanced Edition ship: absent from format 4.
    pub advanced_edition: bool,
    /// Whether the ship has an unlock quest (`<id>_QUEST`): every ship but
    /// the Kestrel, the Engi cruiser and the Lanius (`DefaultDataManager`).
    pub has_quest: bool,
}

const fn ship(
    id: &'static str,
    achievements: [&'static str; 3],
    has_layout_c: bool,
    advanced_edition: bool,
) -> PlayerShip {
    PlayerShip {
        id,
        achievements,
        has_layout_c,
        advanced_edition,
        has_quest: !matches!(
            id.as_bytes(),
            b"PLAYER_SHIP_HARD" | b"PLAYER_SHIP_CIRCLE" | b"PLAYER_SHIP_ANAEROBIC"
        ),
    }
}

/// The vanilla player ships with their ship achievements.
///
/// Source: the achievements and blueprints of FTL 1.6.13 `ftl.dat`, read
/// through the Java editor's DataManager (`getShipAchievements`,
/// `getPlayerShipVariant`). Mods with other ships need `ftl.dat`; that
/// reader is the next slice.
pub const PLAYER_SHIPS: [PlayerShip; 10] = [
    ship(
        "PLAYER_SHIP_HARD",
        [
            "ACH_UNITED_FEDERATION",
            "ACH_FULL_ARSENAL",
            "ACH_TOUGH_SHIP",
        ],
        true,
        false,
    ),
    ship(
        "PLAYER_SHIP_STEALTH",
        [
            "ACH_STEALTH_DESTROY",
            "ACH_STEALTH_AVOID",
            "ACH_STEALTH_TACTICAL",
        ],
        true,
        false,
    ),
    ship(
        "PLAYER_SHIP_MANTIS",
        [
            "ACH_MANTIS_CREW_DEAD",
            "ACH_MANTIS_SLAUGHTER",
            "ACH_MANTIS_SURVIVOR",
        ],
        true,
        false,
    ),
    ship(
        "PLAYER_SHIP_CIRCLE",
        ["ACH_ROBOTIC", "ACH_ONLY_DRONES", "ACH_IONED"],
        true,
        false,
    ),
    ship(
        "PLAYER_SHIP_FED",
        ["ACH_FED_PATIENCE", "ACH_FED_DIPLOMACY", "ACH_FED_UPGRADE"],
        true,
        false,
    ),
    ship(
        "PLAYER_SHIP_JELLY",
        ["ACH_SLUG_VISION", "ACH_SLUG_NEBULA", "ACH_SLUG_BIO"],
        true,
        false,
    ),
    ship(
        "PLAYER_SHIP_ROCK",
        ["ACH_ROCK_FIRE", "ACH_ROCK_MISSILES", "ACH_ROCK_CRYSTAL"],
        true,
        false,
    ),
    ship(
        "PLAYER_SHIP_ENERGY",
        [
            "ACH_ENERGY_SHIELDS",
            "ACH_ENERGY_POWER",
            "ACH_ENERGY_MANPOWER",
        ],
        true,
        false,
    ),
    ship(
        "PLAYER_SHIP_CRYSTAL",
        [
            "ACH_CRYSTAL_SHARD",
            "ACH_CRYSTAL_LOCKDOWN",
            "ACH_CRYSTAL_CLASH",
        ],
        false,
        false,
    ),
    ship(
        "PLAYER_SHIP_ANAEROBIC",
        [
            "ACH_LANIUS_ADVANCED",
            "ACH_LANIUS_SCRAP",
            "ACH_LANIUS_OXYGEN",
        ],
        false,
        true,
    ),
];

pub fn player_ship(id: &str) -> Option<&'static PlayerShip> {
    PLAYER_SHIPS.iter().find(|s| s.id == id)
}

/// Whether an achievement id is a ship victory (`PLAYER_SHIP_*_VICTORY`).
/// Format 9 stores three layout flags after each of these.
pub fn is_victory_achievement(id: &str) -> bool {
    PLAYER_SHIPS
        .iter()
        .any(|s| id.strip_prefix(s.id).is_some_and(|rest| rest == "_VICTORY"))
}

/// Whether an achievement id is a ship quest (`PLAYER_SHIP_*_QUEST`).
pub fn is_quest_achievement(id: &str) -> bool {
    PLAYER_SHIPS
        .iter()
        .any(|s| s.has_quest && id.strip_prefix(s.id).is_some_and(|rest| rest == "_QUEST"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Layout {
    A,
    B,
    C,
}

/// What a profile holds for one ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ShipUnlockState {
    pub layout_a: bool,
    pub layout_b: bool,
    pub layout_c: bool,
}

impl Profile {
    pub fn has_achievement(&self, id: &str) -> bool {
        self.achievements.iter().any(|a| a.id == id)
    }

    /// The unlock state of a vanilla ship, `None` for an unknown id or a
    /// ship the file has no slot for.
    pub fn ship_unlocks(&self, ship_id: &str) -> Option<ShipUnlockState> {
        let ship = player_ship(ship_id)?;
        let slot = self.ships.iter().find(|s| s.ship_id == ship_id)?;
        let earned = ship
            .achievements
            .iter()
            .filter(|a| self.has_achievement(a))
            .count();
        Some(ShipUnlockState {
            layout_a: slot.unlocked_a,
            layout_b: earned >= 2,
            layout_c: slot.unlocked_c,
        })
    }
}

/// One layout that [`unlock_all_ships`] unlocked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnlockChange {
    pub ship_id: String,
    pub layout: Layout,
    /// For Type-B: the ship achievements added (on Easy) to reach two.
    pub added_achievements: Vec<String>,
}

/// Unlocks Type-A, Type-B and Type-C of every vanilla ship the profile has
/// a slot for: [`unlock_all_patch`] applied. Returns one change per layout
/// that was locked, in slot order. A second call returns no changes.
pub fn unlock_all_ships(profile: &mut Profile) -> Vec<UnlockChange> {
    let changes = unlock_all_patch(profile)
        .apply(profile)
        .expect("unlock_all_patch only names layouts the profile stores");
    changes
        .into_iter()
        .filter_map(|c| match c.field {
            ProfileField::Layout { ship, layout } => Some(UnlockChange {
                ship_id: ship,
                layout,
                added_achievements: c.added_achievements,
            }),
            ProfileField::Achievement(_) => None,
        })
        .collect()
}

/// The edits that unlock every locked layout: Type-A and Type-B of every
/// vanilla ship the profile has a slot for, Type-C where the ship has one
/// and the format stores it (format 9). Empty when all is unlocked. The
/// app stages these edits like any other; [`unlock_all_ships`] applies them.
pub fn unlock_all_patch(profile: &Profile) -> ProfilePatch {
    let mut assignments = Vec::new();
    for slot in &profile.ships {
        let Some(ship) = player_ship(&slot.ship_id) else {
            continue;
        };
        let Some(state) = profile.ship_unlocks(ship.id) else {
            continue;
        };
        for (layout, unlocked) in [
            (Layout::A, state.layout_a),
            (Layout::B, state.layout_b),
            (Layout::C, state.layout_c),
        ] {
            let field = ProfileField::Layout {
                ship: ship.id.to_owned(),
                layout,
            };
            if !unlocked && field.check(profile).is_ok() {
                assignments.push(ProfileAssignment { field, value: 1 });
            }
        }
    }
    ProfilePatch { assignments }
}

/// An editable field of the profile, addressed by a dotted path.
///
/// | Path | Value |
/// |---|---|
/// | `ships.<SHIP_ID>.a`, `.b`, `.c` | 1 when that layout of a vanilla ship is unlocked, else 0 |
/// | `achievements.<ID>` | 0 when the profile lacks the achievement, else its difficulty: 1 Easy, 2 Normal, 3 Hard |
///
/// Type-B has no flag: FTL unlocks it from two of the ship's three
/// achievements. `ships.<ID>.b=1` adds missing ship achievements on Easy
/// until two are present; `ships.<ID>.b=0` removes them until one is left
/// (the first present one in [`PlayerShip::achievements`] stays). A ship
/// victory (`PLAYER_SHIP_*_VICTORY`) that is added counts as won with
/// Type-A at its difficulty, so format 9 can store its layout flags.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum ProfileField {
    Layout {
        ship: String,
        layout: Layout,
    },
    /// An achievement id: uppercase ASCII letters, digits and `_`.
    Achievement(String),
}

/// The paths `ftled profile fields` lists.
pub const PROFILE_PATHS: [&str; 2] = ["ships.<SHIP_ID>.<a|b|c>", "achievements.<ID>"];

impl Layout {
    pub fn letter(self) -> char {
        match self {
            Layout::A => 'a',
            Layout::B => 'b',
            Layout::C => 'c',
        }
    }
}

fn difficulty_value(d: Difficulty) -> i32 {
    match d {
        Difficulty::Easy => 1,
        Difficulty::Normal => 2,
        Difficulty::Hard => 3,
    }
}

fn value_difficulty(v: i32) -> Option<Difficulty> {
    match v {
        1 => Some(Difficulty::Easy),
        2 => Some(Difficulty::Normal),
        3 => Some(Difficulty::Hard),
        _ => None,
    }
}

impl ProfileField {
    pub fn path(&self) -> String {
        match self {
            ProfileField::Layout { ship, layout } => format!("ships.{ship}.{}", layout.letter()),
            ProfileField::Achievement(id) => format!("achievements.{id}"),
        }
    }

    /// The inclusive range of values.
    pub fn range(&self) -> (i32, i32) {
        match self {
            ProfileField::Layout { .. } => (0, 1),
            ProfileField::Achievement(_) => (0, 3),
        }
    }

    /// The value of this field in `profile`; the field must pass `check`.
    pub fn get(&self, profile: &Profile) -> i32 {
        match self {
            ProfileField::Layout { ship, layout } => {
                let s = profile.ship_unlocks(ship).unwrap_or(ShipUnlockState {
                    layout_a: false,
                    layout_b: false,
                    layout_c: false,
                });
                i32::from(match layout {
                    Layout::A => s.layout_a,
                    Layout::B => s.layout_b,
                    Layout::C => s.layout_c,
                })
            }
            ProfileField::Achievement(id) => profile
                .achievements
                .iter()
                .find(|a| &a.id == id)
                .map_or(0, |a| difficulty_value(a.difficulty)),
        }
    }

    /// Whether `profile` can hold this field at all.
    fn check(&self, profile: &Profile) -> Result<(), ProfileEditError> {
        match self {
            ProfileField::Layout { ship, layout } => {
                let known =
                    player_ship(ship).filter(|_| profile.ships.iter().any(|s| &s.ship_id == ship));
                let Some(known) = known else {
                    return Err(ProfileEditError::NoSuchShip { ship: ship.clone() });
                };
                if *layout == Layout::C && !(known.has_layout_c && profile.file_format >= 9) {
                    return Err(ProfileEditError::NoLayoutC {
                        ship: ship.clone(),
                        file_format: profile.file_format,
                    });
                }
                Ok(())
            }
            ProfileField::Achievement(id) => {
                if profile.file_format < 9
                    && (is_victory_achievement(id) || is_quest_achievement(id))
                {
                    return Err(ProfileEditError::NotInFormat {
                        id: id.clone(),
                        file_format: profile.file_format,
                    });
                }
                Ok(())
            }
        }
    }
}

impl std::fmt::Display for ProfileField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.path())
    }
}

fn is_id(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
}

impl std::str::FromStr for ProfileField {
    type Err = ProfileEditError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let unknown = || ProfileEditError::UnknownField(s.to_owned());
        let parts: Vec<&str> = s.split('.').collect();
        match parts.as_slice() {
            ["ships", ship, layout] if is_id(ship) => {
                let layout = match *layout {
                    "a" => Layout::A,
                    "b" => Layout::B,
                    "c" => Layout::C,
                    _ => return Err(unknown()),
                };
                Ok(ProfileField::Layout {
                    ship: (*ship).to_owned(),
                    layout,
                })
            }
            ["achievements", id] if is_id(id) => Ok(ProfileField::Achievement((*id).to_owned())),
            _ => Err(unknown()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProfileAssignment {
    pub field: ProfileField,
    pub value: i32,
}

impl std::str::FromStr for ProfileAssignment {
    type Err = ProfileEditError;

    /// Parses `ships.PLAYER_SHIP_HARD.b=1` or `achievements.ACH_SECTOR_5=3`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (path, value) = s
            .split_once('=')
            .ok_or_else(|| ProfileEditError::Syntax(s.to_owned()))?;
        let field: ProfileField = path.trim().parse()?;
        let text = value.trim();
        let value = text.parse().map_err(|_| ProfileEditError::NotAnInteger {
            field: field.clone(),
            text: text.to_owned(),
        })?;
        Ok(ProfileAssignment { field, value })
    }
}

/// Profile edits as data: a list of assignments, applied in order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ProfilePatch {
    pub assignments: Vec<ProfileAssignment>,
}

/// What one assignment changed. `old` and `new` are field values; a Type-B
/// edit names the achievements it added or removed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProfileChange {
    pub field: ProfileField,
    pub old: i32,
    pub new: i32,
    pub added_achievements: Vec<String>,
    pub removed_achievements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProfileEditError {
    #[error("unknown profile field '{0}'; known fields: {known}", known = PROFILE_PATHS.join(", "))]
    UnknownField(String),
    #[error("expected <field>=<value>, got '{0}'")]
    Syntax(String),
    #[error("{field}: '{text}' is not an integer")]
    NotAnInteger { field: ProfileField, text: String },
    #[error("{field}: {value} is outside {min}..={max}")]
    OutOfRange {
        field: ProfileField,
        value: i32,
        min: i32,
        max: i32,
    },
    #[error("{ship} is not a vanilla player ship of this profile")]
    NoSuchShip { ship: String },
    #[error(
        "{ship} has no Type-C here: Crystal and Lanius have none, and format {file_format} stores none before format 9"
    )]
    NoLayoutC { ship: String, file_format: i32 },
    #[error("{id}: format {file_format} has no quest or victory achievements (format 9 has)")]
    NotInFormat { id: String, file_format: i32 },
}

impl ProfilePatch {
    pub fn new(assignments: Vec<ProfileAssignment>) -> Self {
        ProfilePatch { assignments }
    }

    /// Validates and applies the assignments in order. Either every
    /// assignment applies or none does: the patch runs on a copy. The result
    /// lists one change per assignment, in order.
    pub fn apply(&self, profile: &mut Profile) -> Result<Vec<ProfileChange>, ProfileEditError> {
        let mut edited = profile.clone();
        let mut changes = Vec::with_capacity(self.assignments.len());
        for a in &self.assignments {
            changes.push(assign(&mut edited, &a.field, a.value)?);
        }
        *profile = edited;
        Ok(changes)
    }
}

fn assign(
    p: &mut Profile,
    field: &ProfileField,
    value: i32,
) -> Result<ProfileChange, ProfileEditError> {
    field.check(p)?;
    let (min, max) = field.range();
    if !(min..=max).contains(&value) {
        return Err(ProfileEditError::OutOfRange {
            field: field.clone(),
            value,
            min,
            max,
        });
    }
    let old = field.get(p);
    let mut added = Vec::new();
    let mut removed = Vec::new();
    match field {
        ProfileField::Layout { ship, layout } => {
            let slot = p
                .ships
                .iter_mut()
                .find(|s| &s.ship_id == ship)
                .expect("check found the slot");
            match layout {
                Layout::A => slot.unlocked_a = value == 1,
                Layout::C => slot.unlocked_c = value == 1,
                Layout::B => {
                    let ship = player_ship(ship).expect("check found the ship");
                    let present: Vec<&str> = ship
                        .achievements
                        .iter()
                        .copied()
                        .filter(|a| p.has_achievement(a))
                        .collect();
                    if value == 1 && present.len() < 2 {
                        for id in ship.achievements.iter().filter(|a| !present.contains(a)) {
                            if present.len() + added.len() >= 2 {
                                break;
                            }
                            added.push((*id).to_owned());
                        }
                        for id in &added {
                            p.achievements.push(AchievementRecord {
                                id: id.clone(),
                                difficulty: Difficulty::Easy,
                                layouts: None,
                            });
                        }
                    } else if value == 0 && present.len() >= 2 {
                        removed = present[1..].iter().map(|a| (*a).to_owned()).collect();
                        p.achievements.retain(|a| !removed.contains(&a.id));
                    }
                }
            }
        }
        ProfileField::Achievement(id) => match value_difficulty(value) {
            None => p.achievements.retain(|a| &a.id != id),
            Some(d) => {
                let stores_layouts = p.file_format >= 9 && is_victory_achievement(id);
                match p.achievements.iter_mut().find(|a| &a.id == id) {
                    Some(a) => {
                        a.difficulty = d;
                        if let Some(layouts) = &mut a.layouts
                            && layouts.iter().all(Option::is_none)
                        {
                            layouts[0] = Some(d);
                        }
                    }
                    None => p.achievements.push(AchievementRecord {
                        id: id.clone(),
                        difficulty: d,
                        layouts: stores_layouts.then_some([Some(d), None, None]),
                    }),
                }
            }
        },
    }
    Ok(ProfileChange {
        field: field.clone(),
        old,
        new: field.get(p),
        added_achievements: added,
        removed_achievements: removed,
    })
}

fn len_only<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&format!("{} bytes", bytes.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crew() -> CrewRecord {
        CrewRecord {
            value: 0,
            name: String::new(),
            race: String::new(),
            male: true,
        }
    }

    fn profile(format: ProfileFormat) -> Profile {
        let mut ids: Vec<&str> = PLAYER_SHIPS.iter().take(9).map(|s| s.id).collect();
        if format >= 9 {
            ids.extend(["PLAYER_SHIP_ANAEROBIC", "UNKNOWN_BETA", "UNKNOWN_GAMMA"]);
        } else {
            ids.extend(["UNKNOWN_ALPHA", "UNKNOWN_BETA", "UNKNOWN_GAMMA"]);
        }
        Profile {
            file_format: format,
            newbie_tip_level: (format >= 9).then_some(NewbieTipLevel::Veteran),
            achievements: vec![AchievementRecord {
                id: "ACH_FULL_ARSENAL".into(),
                difficulty: Difficulty::Normal,
                layouts: None,
            }],
            ships: ids
                .into_iter()
                .map(|id| ShipAvailability {
                    ship_id: id.into(),
                    unlocked_a: id == "PLAYER_SHIP_HARD",
                    unlocked_c: false,
                })
                .collect(),
            top_scores: vec![],
            ship_best: vec![],
            stats: ProfileStats {
                most_ships_defeated: 0,
                total_ships_defeated: 0,
                most_beacons_explored: 0,
                total_beacons_explored: 0,
                most_scrap_collected: 0,
                total_scrap_collected: 0,
                most_crew_hired: 0,
                total_crew_hired: 0,
                total_games_played: 0,
                total_victories: 0,
            },
            crew_records: CrewRecords {
                most_repairs: crew(),
                most_combat_kills: crew(),
                most_piloted_evasions: crew(),
                most_jumps_survived: crew(),
                most_skill_masteries: crew(),
            },
            undecoded: vec![],
        }
    }

    #[test]
    fn unlock_all_unlocks_every_layout_of_every_ship() {
        let mut p = profile(9);
        let changes = unlock_all_ships(&mut p);
        for ship in PLAYER_SHIPS {
            let s = p.ship_unlocks(ship.id).unwrap();
            assert!(s.layout_a && s.layout_b, "{}: {s:?}", ship.id);
            assert_eq!(s.layout_c, ship.has_layout_c, "{}", ship.id);
        }
        // The unused slots stay as they were.
        assert!(
            p.ships
                .iter()
                .filter(|s| s.ship_id.starts_with("UNKNOWN_"))
                .all(|s| !s.unlocked_a)
        );
        // Kestrel: A was unlocked and one ship achievement was present.
        let kestrel: Vec<_> = changes
            .iter()
            .filter(|c| c.ship_id == "PLAYER_SHIP_HARD")
            .collect();
        assert_eq!(kestrel.len(), 2);
        assert_eq!(kestrel[0].layout, Layout::B);
        assert_eq!(kestrel[0].added_achievements, ["ACH_UNITED_FEDERATION"]);
        // The present achievement keeps its difficulty.
        assert_eq!(p.achievements[0].difficulty, Difficulty::Normal);
        assert_eq!(p.achievements.len(), 1 + 1 + 9 * 2);
        assert!(unlock_all_ships(&mut p).is_empty());
    }

    #[test]
    fn unlock_all_leaves_layout_c_alone_in_format_4() {
        let mut p = profile(4);
        unlock_all_ships(&mut p);
        assert!(p.ships.iter().all(|s| !s.unlocked_c));
        assert!(p.ship_unlocks("PLAYER_SHIP_ANAEROBIC").is_none());
        assert!(p.ship_unlocks("PLAYER_SHIP_MANTIS").unwrap().layout_b);
    }

    fn patch(edits: &[&str]) -> ProfilePatch {
        ProfilePatch::new(edits.iter().map(|e| e.parse().unwrap()).collect())
    }

    #[test]
    fn each_layout_unlocks_and_locks_on_its_own() {
        let mut p = profile(9);
        let changes = patch(&[
            "ships.PLAYER_SHIP_MANTIS.a=1",
            "ships.PLAYER_SHIP_MANTIS.c=1",
        ])
        .apply(&mut p)
        .unwrap();
        let s = p.ship_unlocks("PLAYER_SHIP_MANTIS").unwrap();
        assert!(s.layout_a && !s.layout_b && s.layout_c, "{s:?}");
        assert_eq!((changes[0].old, changes[0].new), (0, 1));
        patch(&["ships.PLAYER_SHIP_HARD.a=0"])
            .apply(&mut p)
            .unwrap();
        assert!(!p.ship_unlocks("PLAYER_SHIP_HARD").unwrap().layout_a);
    }

    #[test]
    fn type_b_adds_and_removes_ship_achievements() {
        let mut p = profile(9);
        // The Kestrel has ACH_FULL_ARSENAL on Normal: unlocking B adds one more.
        let c = patch(&["ships.PLAYER_SHIP_HARD.b=1"])
            .apply(&mut p)
            .unwrap();
        assert_eq!(c[0].added_achievements, ["ACH_UNITED_FEDERATION"]);
        assert!(p.ship_unlocks("PLAYER_SHIP_HARD").unwrap().layout_b);
        // Locking keeps the first present one, ACH_UNITED_FEDERATION.
        let c = patch(&["ships.PLAYER_SHIP_HARD.b=0"])
            .apply(&mut p)
            .unwrap();
        assert_eq!(c[0].removed_achievements, ["ACH_FULL_ARSENAL"]);
        assert!(!p.ship_unlocks("PLAYER_SHIP_HARD").unwrap().layout_b);
        assert!(p.has_achievement("ACH_UNITED_FEDERATION"));
        // A ship without any adds two on Easy.
        let c = patch(&["ships.PLAYER_SHIP_ROCK.b=1"])
            .apply(&mut p)
            .unwrap();
        assert_eq!(
            c[0].added_achievements,
            ["ACH_ROCK_FIRE", "ACH_ROCK_MISSILES"]
        );
        // An unchanged value changes nothing.
        let c = patch(&["ships.PLAYER_SHIP_ROCK.b=1"])
            .apply(&mut p)
            .unwrap();
        assert!(c[0].added_achievements.is_empty() && c[0].old == 1);
    }

    #[test]
    fn achievements_are_added_set_and_removed() {
        let mut p = profile(9);
        let c = patch(&[
            "achievements.ACH_SECTOR_5=3",
            "achievements.ACH_FULL_ARSENAL=1",
        ])
        .apply(&mut p)
        .unwrap();
        assert_eq!((c[0].old, c[0].new), (0, 3));
        assert_eq!((c[1].old, c[1].new), (2, 1));
        let a = p
            .achievements
            .iter()
            .find(|a| a.id == "ACH_SECTOR_5")
            .unwrap();
        assert_eq!((a.difficulty, a.layouts), (Difficulty::Hard, None));
        patch(&["achievements.ACH_FULL_ARSENAL=0"])
            .apply(&mut p)
            .unwrap();
        assert!(!p.has_achievement("ACH_FULL_ARSENAL"));
        // A victory stores its layout flags in format 9: won with Type-A.
        patch(&["achievements.PLAYER_SHIP_HARD_VICTORY=2"])
            .apply(&mut p)
            .unwrap();
        let v = p.achievements.last().unwrap();
        assert_eq!(v.layouts, Some([Some(Difficulty::Normal), None, None]));
    }

    #[test]
    fn a_bad_edit_changes_nothing() {
        let mut p = profile(9);
        let before = p.clone();
        let err = patch(&["ships.PLAYER_SHIP_HARD.a=1", "achievements.ACH_SECTOR_5=4"])
            .apply(&mut p)
            .unwrap_err();
        assert!(matches!(err, ProfileEditError::OutOfRange { .. }), "{err}");
        assert_eq!(p, before);
        let err = patch(&["ships.PLAYER_SHIP_CRYSTAL.c=1"])
            .apply(&mut p)
            .unwrap_err();
        assert!(matches!(err, ProfileEditError::NoLayoutC { .. }), "{err}");
        let err = patch(&["ships.UNKNOWN_BETA.a=1"])
            .apply(&mut p)
            .unwrap_err();
        assert!(matches!(err, ProfileEditError::NoSuchShip { .. }), "{err}");
        let mut p4 = profile(4);
        for e in [
            "ships.PLAYER_SHIP_HARD.c=1",
            "achievements.PLAYER_SHIP_HARD_VICTORY=1",
        ] {
            assert!(patch(&[e]).apply(&mut p4).is_err(), "{e}");
        }
        for bad in [
            "ships.PLAYER_SHIP_HARD.d=1",
            "achievements.ach_x=1",
            "crew.0=1",
        ] {
            assert!(bad.parse::<ProfileAssignment>().is_err(), "{bad}");
        }
    }

    #[test]
    fn paths_round_trip() {
        for path in ["ships.PLAYER_SHIP_HARD.b", "achievements.ACH_SECTOR_5"] {
            assert_eq!(path.parse::<ProfileField>().unwrap().path(), path);
        }
    }

    #[test]
    fn unlock_all_patch_names_only_locked_layouts() {
        let p = profile(9);
        let patch = unlock_all_patch(&p);
        let paths: Vec<_> = patch.assignments.iter().map(|a| a.field.path()).collect();
        assert!(!paths.contains(&"ships.PLAYER_SHIP_HARD.a".to_owned()));
        assert!(paths.contains(&"ships.PLAYER_SHIP_HARD.c".to_owned()));
        assert!(!paths.iter().any(|p| p == "ships.PLAYER_SHIP_CRYSTAL.c"));
        assert!(paths.iter().all(|p| !p.contains("UNKNOWN")));
        assert!(is_quest_achievement("PLAYER_SHIP_STEALTH_QUEST"));
        assert!(!is_quest_achievement("PLAYER_SHIP_HARD_QUEST"));
    }

    #[test]
    fn victory_achievements_are_the_ten_ship_victories() {
        assert!(is_victory_achievement("PLAYER_SHIP_HARD_VICTORY"));
        assert!(is_victory_achievement("PLAYER_SHIP_ANAEROBIC_VICTORY"));
        assert!(!is_victory_achievement("PLAYER_SHIP_STEALTH_QUEST"));
        assert!(!is_victory_achievement("ACH_WIN_EASY"));
    }
}
