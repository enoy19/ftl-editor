//! The identity of a run: the ship name (`ship.name`, `nearby.ship.name`),
//! the ship type (`ship.type`), the difficulty (`difficulty`) and the
//! sector number (`sector`).
//!
//! Evidence (FTL 1.6.13, macOS binary, symbols read with Ghidra 12.1.4 on
//! 2026-11-19; field offsets from Hyperspace `FTLGameStripped.h`):
//!
//! - Where the file stores them. `WorldManager::LoadGame` (0x1001da810)
//!   reads the format (and for format 11 the RNG flag, from format 3 the
//!   DLC flag), then `ScoreKeeper::LoadGame` (0x100029940) the rest of the
//!   header: the difficulty into the global `Settings::difficulty`
//!   (0x100347944), the four session stats, the ship name and the ship
//!   blueprint id into the running score (`ScoreKeeper+0x2a8`, `+0x2c0`),
//!   the one-based sector (`+0x2e0`) and the victory flag (`+0x2e8`).
//!   `ScoreKeeper::SaveGame` (0x1000298a0) writes the same fields back from
//!   the same places. Then `ShipManager::ImportShip` (0x10009cf10) reads the
//!   player ship: blueprint id, name, hull image base name. The nearby ship
//!   carries the same three fields (`ImportShip` on `ShipManager(1)`). The
//!   zero-based sector number of the world follows the sector visitation
//!   (`StarMap::LoadGame`, 0x1001cab20).
//! - Difficulty. Below format 3 FTL reads the flag inverted: 0 is Normal,
//!   1 Easy (`DAT = (flag == 0)`); from format 3 on it takes the value, 0
//!   Easy, 1 Normal, 2 Hard. The global is the only place the value lands,
//!   and FTL reads it live: rewards (`GenerateReward`), the enemy ships it
//!   builds (`ShipGenerator::CreateShip`, `GenerateSystemMaxes`), door
//!   health (`Door::GetBaseHealth`), AI priorities, the scrap modifier
//!   (`ShipManager::ResetScrapLevel`: 30, 10 on Normal, 0 on Hard; `OnInit`
//!   sets it at load), the fleet beacon events (`TurnIntoFleetLocation`),
//!   the boss, the score and the achievements. None of that is stored, so
//!   a changed difficulty applies from the load on. One reader runs during
//!   the load and shapes stored data: `StarMap::GenerateMap` (0x1001cb870),
//!   which `LoadGame` calls to rebuild the beacons from the layout seed,
//!   draws the Last Stand's grid with Hard-only bounds while the rebel
//!   flagship is on the map (`StarMap+0xac8`, set from the stored
//!   `flagship_visible`). The beacons of the file were made with the old
//!   bounds ("location count doesn't match" is logged when they differ), so
//!   the editor refuses the change then ([`DifficultyBlock::Flagship`]).
//!   Hard exists from format 7 on (Advanced Edition); the codec writes no
//!   other value.
//! - Sector number. `StarMap::LoadGame` seeds the RNG with the sector tree
//!   seed, rebuilds the tree (`GenerateSectorMap`), and marks each stored
//!   visitation flag on its tree node; the deepest visited node becomes the
//!   current sector (`StarMap+0x900`) and its level the score's sector
//!   (`ScoreKeeper::SetSector(level + 1)`, which overwrites the header's).
//!   Then it reads the world's sector number into `StarMap+0x548`
//!   (`worldLevel`), and for the hidden Crystal sector writes that level
//!   onto the node. The map, the next jump (the neighbours of the current
//!   node) and the Last Stand (level 7: flagship, base) follow the node and
//!   the level. A sector number that disagrees with the visited node leaves
//!   FTL with a map of one sector and the level of another, so the editor
//!   refuses `sector` alone ([`EditError::SectorFixed`]); `sector.jump`
//!   ([`crate::sector_tree`]) sets the visited route, both numbers and the
//!   map of the new sector together.
//! - Ship name. `ImportShip` stores the read name in its copy of the
//!   blueprint (`ShipManager+0x700`, the name at `+0x8a0` with the literal
//!   flag `+0x8b8` set): FTL shows it as written, no text lookup. The
//!   hangar's text box takes 16 characters (`ShipBuilder::ShipBuilder`:
//!   `TextInput(16, ...)`), the crew box 15; the editor keeps its crew name
//!   rule (1 to [`MAX_SHIP_NAME`] characters, trimmed, no control
//!   characters). For the player ship it writes the header's copy too, as
//!   `ScoreKeeper::SaveGame` would after a rename in the hangar. The nearby
//!   ship's name is never shown: the fixture's pirate carries FTL's default
//!   "should not be seen".
//! - Ship type. `ImportShip` takes the blueprint by id
//!   (`BlueprintManager::GetShipBlueprint`), then reads name and image base
//!   name over its copy; `Ship::LoadImages` and `ResourceControl::
//!   LoadEnemyShips` draw the hull from that image name, so a new type needs
//!   the image of its blueprint ([`ShipLayouts::ship_gfx`]). `OnInit` builds
//!   the start systems of the new blueprint and its layout; every later read
//!   of the ship (rooms, squares, doors, stations, crew positions in pixels,
//!   systems by the rooms of `systemList`, weapons and drones by slot)
//!   follows that layout. The achievement tracker takes the blueprint id
//!   from the ship (`ShipManager+0x888`). So a change keeps the file sound
//!   only when the new blueprint has the same layout file, every installed
//!   system has the same rooms there, every start system of the new
//!   blueprint is installed (else it comes back at load without its stored
//!   extended info, as [`crate::installs`] explains), and the weapons and
//!   drones fit its slots ([`TypeBlock`]). The editor then writes the id
//!   (ship and header), and the image base name.
//!
//! Design note, the full type change (not built): no two of the 28 hangar
//! blueprints of 1.6.13 share a layout file, and no two layouts share
//! their rooms (checked on 2026-11-19 against `ftl.dat`), so A, B and C of
//! one ship differ in rooms and doors; only the unused `PLAYER_SHIP_EASY`
//! and the tutorial Kestrel share a layout (`kestral`) with a hangar ship,
//! Kestrel A. The editor therefore offers no type change of a hangar ship
//! today and names the reason. A change across layouts has to rebuild the
//! ship body for the new layout: rooms (oxygen, squares with fire,
//! stations), breaches, doors, crew squares and pixel positions (the
//! pilot station and free squares of [`crate::crew_list`]), the systems
//! (remove what the new blueprint has no room for, add its start systems
//! with their extended info, keep levels), weapons and drones beyond the
//! new slots (to the cargo hold, at most 4), lockdown crystals and drone
//! bodies on the old floor, and every room id another part of the file
//! names (the nearby ship's weapon targets, its boarders on our floor,
//! hacking and mind targets, projectiles). Outside a fight (no nearby ship,
//! no projectiles) only the body and the crew need it.

use std::collections::BTreeSet;

use crate::edit::{EditError, Field, ship, ship_mut};
use crate::installs::is_installed;
use crate::layout::{ShipLayouts, SystemRoom};
use crate::model::{Difficulty, SavedGame, ShipId, SystemKind};

/// The longest ship name the editor writes, in characters: the crew name
/// rule ([`crate::edit::MAX_CREW_NAME`]). The hangar takes 16.
pub const MAX_SHIP_NAME: usize = 32;

/// The first file format with the Hard difficulty (Advanced Edition).
const HARD_FORMAT: i32 = 7;

/// The difficulty as the editor writes it: 0 Easy, 1 Normal, 2 Hard.
pub fn difficulty_number(d: Difficulty) -> i32 {
    match d {
        Difficulty::Easy => 0,
        Difficulty::Normal => 1,
        Difficulty::Hard => 2,
    }
}

fn difficulty_of(n: i32) -> Difficulty {
    match n {
        0 => Difficulty::Easy,
        1 => Difficulty::Normal,
        _ => Difficulty::Hard,
    }
}

/// Why the difficulty stays; `code()` is stable for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyBlock {
    /// The rebel flagship is on the map: FTL rebuilds the Last Stand's
    /// beacons with bounds that depend on Hard.
    Flagship,
    /// The sector map is not decoded, so the editor cannot tell.
    NotDecoded,
}

impl DifficultyBlock {
    pub fn code(self) -> &'static str {
        match self {
            DifficultyBlock::Flagship => "flagship",
            DifficultyBlock::NotDecoded => "not_decoded",
        }
    }

    pub fn reason(self) -> &'static str {
        match self {
            DifficultyBlock::Flagship => {
                "the rebel flagship is on the map; FTL rebuilds its beacons with bounds that depend on Hard, so the stored beacons would not match"
            }
            DifficultyBlock::NotDecoded => "the sector map is not decoded (ftl.dat)",
        }
    }
}

/// Why the difficulty of this game stays, or `None`.
pub fn difficulty_block(game: &SavedGame) -> Option<DifficultyBlock> {
    match &game.world {
        None => Some(DifficultyBlock::NotDecoded),
        Some(w) if w.rebel_fleet.flagship_visible => Some(DifficultyBlock::Flagship),
        Some(_) => None,
    }
}

pub(crate) fn difficulty_range(game: &SavedGame, field: &Field) -> Result<(i32, i32), EditError> {
    if let Some(block) = difficulty_block(game) {
        return Err(EditError::DifficultyBlocked {
            field: field.clone(),
            block,
        });
    }
    Ok((
        0,
        if game.file_format >= HARD_FORMAT {
            2
        } else {
            1
        },
    ))
}

/// Writes a difficulty the range checked; returns the old number.
pub(crate) fn set_difficulty(game: &mut SavedGame, value: i32) -> i32 {
    difficulty_number(std::mem::replace(
        &mut game.difficulty,
        difficulty_of(value),
    ))
}

/// The ship name of the field's ship.
pub(crate) fn name(game: &SavedGame, field: &Field) -> Result<String, EditError> {
    Ok(ship(game, field)?.name.clone())
}

/// Checks and writes a ship name; the player ship's header copy too.
pub(crate) fn set_name(
    game: &mut SavedGame,
    field: &Field,
    text: &str,
) -> Result<String, EditError> {
    ship(game, field)?;
    let name = text.trim();
    let length = name.chars().count();
    if length == 0 || length > MAX_SHIP_NAME || name.chars().any(char::is_control) {
        return Err(EditError::BadShipName {
            field: field.clone(),
            text: text.to_owned(),
        });
    }
    if field.ship() == ShipId::Player {
        game.player_ship_name = name.to_owned();
    }
    Ok(std::mem::replace(
        &mut ship_mut(game, field).name,
        name.to_owned(),
    ))
}

/// Why a ship cannot take another blueprint; the module doc names the FTL
/// functions behind each. `code()` is stable for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeBlock {
    /// The nearby ship: its blueprint drives its AI and crew, the editor
    /// changes the player ship only.
    EnemyShip,
    /// The blueprints need ftl.dat.
    NoGameData,
    /// ftl.dat has no ship blueprint (with a layout) of that id.
    UnknownShip,
    /// The ship's rooms are not decoded.
    NotDecoded,
    /// The new blueprint uses another layout file: rooms and doors differ.
    OtherLayout,
    /// An installed system has other rooms (or none) in the new blueprint.
    SystemRoom(SystemKind),
    /// The new blueprint starts with a system the ship lacks.
    StartSystem(SystemKind),
    /// The weapons do not fit the new blueprint's slots.
    WeaponSlots,
    /// The drones do not fit the new blueprint's slots.
    DroneSlots,
}

impl TypeBlock {
    pub fn code(self) -> &'static str {
        match self {
            TypeBlock::EnemyShip => "enemy_ship",
            TypeBlock::NoGameData => "no_game_data",
            TypeBlock::UnknownShip => "unknown_ship",
            TypeBlock::NotDecoded => "not_decoded",
            TypeBlock::OtherLayout => "other_layout",
            TypeBlock::SystemRoom(_) => "system_room",
            TypeBlock::StartSystem(_) => "start_system",
            TypeBlock::WeaponSlots => "weapon_slots",
            TypeBlock::DroneSlots => "drone_slots",
        }
    }

    /// The system the block names, if any.
    pub fn system(self) -> Option<SystemKind> {
        match self {
            TypeBlock::SystemRoom(k) | TypeBlock::StartSystem(k) => Some(k),
            _ => None,
        }
    }

    /// The English reason, for the CLI and error details.
    pub fn reason(self) -> String {
        match self {
            TypeBlock::EnemyShip => "the editor changes the type of the player ship only".into(),
            TypeBlock::NoGameData => "the ship blueprints need ftl.dat".into(),
            TypeBlock::UnknownShip => "ftl.dat has no ship blueprint of that id".into(),
            TypeBlock::NotDecoded => "the rooms of the ship are not decoded (ftl.dat)".into(),
            TypeBlock::OtherLayout => {
                "the type has other rooms and doors; FTL would read the stored rooms, doors and crew positions on the wrong floor".into()
            }
            TypeBlock::SystemRoom(k) => format!(
                "the {} system sits in other rooms of that type",
                k.id()
            ),
            TypeBlock::StartSystem(k) => format!(
                "that type starts with the {} system, which FTL builds at load without its stored state",
                k.id()
            ),
            TypeBlock::WeaponSlots => "the weapons do not fit the weapon slots of that type".into(),
            TypeBlock::DroneSlots => "the drones do not fit the drone slots of that type".into(),
        }
    }
}

fn rooms_of(entries: &[SystemRoom], kind: SystemKind) -> BTreeSet<usize> {
    entries
        .iter()
        .filter(|e| e.kind == kind)
        .map(|e| e.room)
        .collect()
}

/// Why ship `on` cannot become blueprint `to`, or `None` when it can (also
/// for its own blueprint).
pub fn type_block(
    game: &SavedGame,
    on: ShipId,
    to: &str,
    data: &dyn ShipLayouts,
) -> Option<TypeBlock> {
    if on == ShipId::Nearby {
        return Some(TypeBlock::EnemyShip);
    }
    let ship = game.ship(on)?;
    if ship.blueprint_id == to {
        return None;
    }
    let Some(from_shape) = data.ship(&ship.blueprint_id) else {
        return Some(TypeBlock::NoGameData);
    };
    let (Some(to_shape), Some(_)) = (data.ship(to), data.ship_gfx(to)) else {
        return Some(TypeBlock::UnknownShip);
    };
    let Some(body) = &ship.body else {
        return Some(TypeBlock::NotDecoded);
    };
    if from_shape.layout_id != to_shape.layout_id {
        return Some(TypeBlock::OtherLayout);
    }
    let from_rooms = data.system_rooms(&ship.blueprint_id).unwrap_or_default();
    let to_rooms = data.system_rooms(to).unwrap_or_default();
    for kind in SystemKind::ALL {
        if is_installed(ship, kind) && rooms_of(&from_rooms, kind) != rooms_of(&to_rooms, kind) {
            return Some(TypeBlock::SystemRoom(kind));
        }
    }
    if let Some(e) = to_rooms
        .iter()
        .find(|e| e.start && !is_installed(ship, e.kind))
    {
        return Some(TypeBlock::StartSystem(e.kind));
    }
    let Some(slots) = data.slots(to) else {
        return Some(TypeBlock::UnknownShip);
    };
    if body.weapons.len() > slots.weapons as usize {
        return Some(TypeBlock::WeaponSlots);
    }
    if body.drones.len() > slots.drones as usize {
        return Some(TypeBlock::DroneSlots);
    }
    None
}

/// Checks and writes a ship type: the blueprint id (ship and header) and
/// the hull image of the new blueprint.
pub(crate) fn set_type(
    game: &mut SavedGame,
    field: &Field,
    to: &str,
    data: &dyn ShipLayouts,
) -> Result<String, EditError> {
    ship(game, field)?;
    if let Some(block) = type_block(game, field.ship(), to, data) {
        return Err(EditError::TypeBlocked {
            field: field.clone(),
            to: to.to_owned(),
            block,
        });
    }
    let ship = ship_mut(game, field);
    if ship.blueprint_id == to {
        return Ok(to.to_owned());
    }
    ship.gfx_base_name = data.ship_gfx(to).expect("type_block checked the image");
    let old = std::mem::replace(&mut ship.blueprint_id, to.to_owned());
    game.player_ship_blueprint_id = to.to_owned();
    Ok(old)
}
