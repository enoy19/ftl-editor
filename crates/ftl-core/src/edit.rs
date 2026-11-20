//! Edits of a saved game as data: a [`Patch`] is a list of assignments.
//!
//! The CLI parses `ship.scrap=9999` or `crew.0.skills.pilot=30` into an
//! [`Assignment`]; the Flutter app builds the same values. Both run
//! [`Patch::apply`], the one write path. Path syntax, ranges and every other
//! rule of an edit live here, not in an adapter.

use std::fmt;
use std::str::FromStr;

use serde::{Serialize, Serializer};

use crate::beacons::{self, StoreSupply};
use crate::crew_list;
use crate::damage;
use crate::energy_shield;
use crate::identity;
use crate::installs::{self, InstallBlock};
use crate::items::{self, Slots};
use crate::layout::{KnownLayouts, ShipLayouts};
use crate::model::{
    Beacon, Breach, CrewMember, DroneKind, RACES, Room, SavedGame, Ship, ShipBody, ShipId,
    SkillKind, StateVar, SystemKind, SystemState, max_health, race,
};
use crate::raw;
use crate::sector_map::NoLayout;
use crate::sector_tree;
use crate::slot_order::{Holes, Snapshot};

/// A fixed resource of a ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Resource {
    Hull,
    Fuel,
    DroneParts,
    Missiles,
    Scrap,
}

impl Resource {
    pub const ALL: [Resource; 5] = [
        Resource::Hull,
        Resource::Fuel,
        Resource::DroneParts,
        Resource::Missiles,
        Resource::Scrap,
    ];

    pub fn path(self) -> &'static str {
        match self {
            Resource::Hull => "ship.hull",
            Resource::Fuel => "ship.fuel",
            Resource::DroneParts => "ship.drone_parts",
            Resource::Missiles => "ship.missiles",
            Resource::Scrap => "ship.scrap",
        }
    }

    /// Inclusive value range the editor accepts for this resource.
    ///
    /// The upper bounds are generous caps that keep the HUD readable; the
    /// format itself stores any i32. Hull above the blueprint maximum needs
    /// the game assets to check and is not limited here yet.
    pub fn range(self) -> (i32, i32) {
        match self {
            Resource::Hull => (1, 999),
            Resource::Fuel | Resource::DroneParts | Resource::Missiles => (0, 999),
            Resource::Scrap => (0, 99_999),
        }
    }

    /// The value on the player ship.
    pub fn get(self, game: &SavedGame) -> i32 {
        self.of(&game.player_ship)
    }

    /// The value on `ship`.
    pub fn of(self, ship: &Ship) -> i32 {
        let r = &ship.resources;
        match self {
            Resource::Hull => r.hull,
            Resource::Fuel => r.fuel,
            Resource::DroneParts => r.drone_parts,
            Resource::Missiles => r.missiles,
            Resource::Scrap => r.scrap,
        }
    }

    fn slot(self, ship: &mut Ship) -> &mut i32 {
        let r = &mut ship.resources;
        match self {
            Resource::Hull => &mut r.hull,
            Resource::Fuel => &mut r.fuel,
            Resource::DroneParts => &mut r.drone_parts,
            Resource::Missiles => &mut r.missiles,
            Resource::Scrap => &mut r.scrap,
        }
    }
}

/// An editable field, addressed by a dotted path.
///
/// | Path | Field |
/// |---|---|
/// | `ship.hull`, `ship.fuel`, `ship.drone_parts`, `ship.missiles`, `ship.scrap` | [`Resource`] |
/// | `crew.<n>.skills.<pilot\|engine\|shield\|weapon\|repair\|combat>` | skill points of crew member `n` (0-based, file order) |
/// | `crew.<n>.health` | health of crew member `n`, 1 to the full health of its race ([`max_health`]) |
/// | `crew.<n>.name` | name of crew member `n` (text, 1 to [`MAX_CREW_NAME`] characters, no control characters) |
/// | `crew.<n>.race` | race of crew member `n` (text, an id of [`RACES`]); fits health, skills and tints ([`CrewMember::change_race`]) |
/// | `crew.<n>.tints.<layer>` | the color of tint layer `layer` of crew member `n`, 0 to the last color of the layer of its race ([`crate::crew_list`]) |
/// | `crew.<n>.male` | 1 male, 0 female; FTL draws a female human with its own sprite |
/// | `crew.<n>.remove` | 1 removes crew member `n` when the patch ends; never the last of its side ([`crate::crew_list`]) |
/// | `crew.add` | how many default humans the patch appends, on free squares with free names, within FTL's crew limit ([`crate::crew_list`]); always 0 in a file |
/// | `reactor` | reactor power of the player ship, 0 to [`MAX_REACTOR`]; below the bars in use it takes power away ([`fit_power_to_reactor`]) |
/// | `systems.<id>.level` | level (capacity) of an installed system, ids as in [`SystemKind::id`]; below its power it lowers the power ([`fit_power_to_capacity`]) |
/// | `systems.<id>.installed` | 1 when the system is installed, 0 when not; installs it at the level FTL gives it or removes it, within the rooms and limits of [`crate::installs`] |
/// | `augments.<ID>` | how many augments with this blueprint id the ship carries |
/// | `beacons.<n>.seen` | 1 when beacon `n` (0-based beacon id) is revealed on the sector map, else 0 |
/// | `beacons.<n>.visited` | 1 when the player has been at beacon `n`; only 1 to 0, never at the current beacon ([`crate::beacons`]) |
/// | `beacons.<n>.store.<shelf>.<slot>` | the count of the store item in that slot, 0 sold out, 1 for sale |
/// | `beacons.<n>.store.<fuel\|missiles\|drone_parts>` | the store's count of that supply, 0 to [`crate::beacons::MAX_STORE_SUPPLY`] |
/// | `fleet.offset` | x of the rebel fleet's circle on the sector map, -959 (a new sector) to 60; sets the fleet flag of every beacon ([`crate::beacons`]) |
/// | `nearby.<path>` | the field `<path>` of the ship at the current beacon, for every path above but `beacons` and `cargo` ([`Field::Nearby`]) |
/// | `ship.name` | the ship name (text, 1 to [`crate::identity::MAX_SHIP_NAME`] characters); the player's header copy too ([`crate::identity`]) |
/// | `ship.type` | the ship blueprint id (text); only a blueprint of the same layout whose rooms fit ([`crate::identity::TypeBlock`]) |
/// | `difficulty` | 0 Easy, 1 Normal, 2 Hard (format 7 and later); refused while the rebel flagship is on the map ([`crate::identity`]) |
/// | `sector` | the one-based sector number; always refused, it follows the sector tree ([`EditError::SectorFixed`]) |
/// | `sector.jump` | the sector of the tree the ship is in (0-based, FTL's order); a new one is an arrival there ([`crate::sector_tree`]) |
/// | `fires.<x>.<y>` | health of the fire at square `x`,`y` of the ship grid (0 to 100); 0 puts it out, more lights it ([`set_fire`]) |
/// | `breaches.<x>.<y>` | health of the hull breach at square `x`,`y` of the ship grid (0 to 100); 0 seals it, more adds one ([`set_breach`]) |
/// | `rooms.<n>.oxygen` | oxygen of room `n` (room id) in percent, 0 to 100; needs the oxygen system ([`Field::RoomOxygen`]) |
/// | `weapons.<ID>`, `drones.<ID>` | how many weapons or drones with this blueprint id the slots hold, within the slots of the ship blueprint ([`crate::items`]) |
/// | `weapons.<ID>.armed`, `drones.<ID>.armed` | how many of them are armed, within the free bars of system and reactor |
/// | `cargo.<ID>` | how many weapons or drones with this blueprint id the cargo holds, 4 in all |
/// | `raw.<path>` | any number or flag of the decoded save at its raw path ([`crate::raw`], `ftled dump`), within its type |
/// | `state.<id>` | the value of the state var `id` |
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Field {
    Resource(Resource),
    CrewSkill {
        crew: usize,
        skill: SkillKind,
    },
    CrewHealth {
        crew: usize,
    },
    /// The name of a crew member; a text field.
    CrewName {
        crew: usize,
    },
    /// The race id of a crew member; a text field.
    CrewRace {
        crew: usize,
    },
    /// The index of one tint layer of a crew member ([`crate::crew_list`]).
    CrewTint {
        crew: usize,
        layer: usize,
    },
    /// 1 when the crew member is male.
    CrewMale {
        crew: usize,
    },
    /// 1 stages the removal of the crew member ([`crate::crew_list`]).
    CrewRemove {
        crew: usize,
    },
    /// How many new crew members the patch appends ([`crate::crew_list`]).
    CrewAdd,
    /// The reactor power of the player ship.
    Reactor,
    SystemLevel(SystemKind),
    /// The damaged bars of an installed system, 0 to its level
    /// ([`crate::damage`]).
    SystemDamage(SystemKind),
    /// The ion lock count of an installed system, 0 to
    /// [`crate::damage::MAX_ION`] ([`crate::damage`]).
    SystemIon(SystemKind),
    /// The Zoltan shield points of the shields system
    /// ([`crate::energy_shield`]).
    ShieldEnergy,
    /// 1 when the player ship has the system, 0 when not ([`crate::installs`]).
    SystemInstalled(SystemKind),
    /// An augment blueprint id: uppercase ASCII letters, digits and `_`.
    Augment(String),
    /// The seen flag of a beacon, by beacon id.
    BeaconSeen(usize),
    /// Whether the player has been at a beacon ([`crate::beacons`]).
    BeaconVisited(usize),
    /// The count of a store item, by beacon, shelf and slot.
    StoreStock {
        beacon: usize,
        shelf: usize,
        slot: usize,
    },
    /// A store's count of fuel, missiles or drone parts.
    StoreSupply {
        beacon: usize,
        supply: StoreSupply,
    },
    /// The x of the rebel fleet's circle on the sector map.
    FleetOffset,
    /// The beacon id of the ship ([`crate::beacons`]): a move inside the
    /// sector.
    ShipBeacon,
    /// 1 when the beacon has a store; 1 adds [`beacons::new_store`].
    BeaconStore(usize),
    /// The blueprint id in a store slot; a text field.
    StoreItem {
        beacon: usize,
        shelf: usize,
        slot: usize,
    },
    /// A field of the nearby ship: `nearby.` before a path of a ship field
    /// ([`Field::is_ship_field`]). Every rule applies alike, because FTL
    /// loads both ships with the same code ([`ShipId`]).
    Nearby(Box<Field>),
    /// The health of the fire on a square of the player ship, by its square
    /// on the ship grid (the coordinates of [`Field::Breach`]).
    Fire {
        x: i32,
        y: i32,
    },
    /// The health of a hull breach, by its square on the ship grid (the
    /// coordinates the file stores, layout offset included).
    Breach {
        x: i32,
        y: i32,
    },
    /// The oxygen of a room in percent, by room id.
    ///
    /// FTL 1.6.13 (macOS binary, Ghidra, read 2026-11-19) keeps it as a
    /// float per room in `OxygenSystem` (+0x268). `ShipManager::ExportShip`
    /// writes `(int)` of `OxygenSystem::GetOxygenLevel(room)`: a whole
    /// percent, truncated, not the `(int)(f * 1000)` of `writeFloat`; a ship
    /// without the oxygen system writes 0 for every room.
    /// `ShipManager::ImportShip` reads the int and calls
    /// `OxygenSystem::SetRoomOxygen(room, (float)n)` only when the ship has
    /// the system, so the field exists only then. `OxygenSystem::Restart`
    /// fills every room with 100.0; that is a refill.
    RoomOxygen {
        room: usize,
    },
    /// The copies of a weapon blueprint in the weapon slots.
    Weapon(String),
    /// The armed copies of a weapon blueprint.
    WeaponArmed(String),
    /// The copies of a drone blueprint in the drone slots.
    Drone(String),
    /// The armed copies of a drone blueprint.
    DroneArmed(String),
    /// The copies of a weapon or drone blueprint in the cargo hold.
    Cargo(String),
    /// A number or flag of the decoded save by its raw path
    /// ([`crate::raw`]): the Expert area. Only its type limits the value;
    /// the verify step of a save still guards the file.
    Raw(String),
    /// The value of a state var, by its id.
    StateVar(String),
    /// The ship name; a text field ([`crate::identity`]).
    ShipName,
    /// The ship blueprint id; a text field ([`crate::identity`]).
    ShipType,
    /// The difficulty: 0 Easy, 1 Normal, 2 Hard ([`crate::identity`]).
    Difficulty,
    /// The one-based sector number; read only ([`EditError::SectorFixed`]).
    Sector,
    /// The sector of the tree the ship is in; a new one jumps there
    /// ([`crate::sector_tree`]). Needs the sector lists of `ftl.dat`.
    SectorJump,
}

/// The longest crew name the editor writes, in characters. An editor cap:
/// the file stores any length, but the crew boxes of FTL show about a dozen
/// characters.
pub const MAX_CREW_NAME: usize = 32;

/// Augments a ship holds in FTL.
pub const MAX_AUGMENTS: usize = 3;

/// The reactor power FTL's upgrade screen sells up to.
pub const MAX_REACTOR: i32 = 25;

/// The bars of a system that come from the reactor: its power without the
/// bars the backup battery feeds.
fn reactor_bars(s: &SystemState) -> i32 {
    (s.power - s.battery_power).max(0)
}

/// The reactor bars the systems of `ship` draw: the power of every installed
/// system that is not a subsystem, without the bars the backup battery feeds.
pub fn reactor_power_in_use(ship: &Ship) -> i32 {
    ship.systems
        .iter()
        .filter(|s| !s.kind.is_subsystem())
        .filter_map(|s| s.installed.as_ref())
        .map(reactor_bars)
        .sum()
}

/// Takes reactor bars away from the systems until they draw no more than
/// the reactor holds, the last system of the file first.
///
/// This mirrors how FTL 1.6.13 loads a ship (`ShipManager::ImportShip`,
/// macOS binary, read 2026-11-19): it sets the reactor first, then gives
/// each system its saved power one bar at a time with
/// `ShipSystem::IncreasePower`, in file order, and a bar the reactor no
/// longer has fails. The systems early in the file keep their power, the
/// later ones lose it. Subsystems draw no reactor bars and battery bars stay.
/// A weapon or drone that stays armed without the power is left as it is:
/// `WeaponSystem::PowerWeapon` at load refuses a weapon the system power
/// does not cover.
pub fn fit_power_to_reactor(ship: &mut Ship) {
    let mut over = reactor_power_in_use(ship) - ship.reactor_power;
    for system in ship.systems.iter_mut().rev() {
        if over <= 0 {
            break;
        }
        if system.kind.is_subsystem() {
            continue;
        }
        if let Some(s) = system.installed.as_mut() {
            let taken = reactor_bars(s).min(over);
            s.power -= taken;
            over -= taken;
        }
    }
}

/// Lowers the power of a system to what its level allows: FTL powers at
/// most `capacity - damaged` bars (`ShipSystem::GetMaxPower`, without the
/// temporary limits of events). Damaged bars stay within the level, the
/// reactor bars go first and the battery bars last.
pub fn fit_power_to_capacity(s: &mut SystemState) {
    s.damaged_bars = s.damaged_bars.min(s.capacity);
    let allowed = (s.capacity - s.damaged_bars).max(0);
    s.power = s.power.min(allowed);
    s.battery_power = s.battery_power.min(s.power);
}

/// The paths whose range depends on the game, for help and error texts.
pub const INDEXED_PATHS: [&str; 36] = [
    "ship.name",
    "ship.type",
    "difficulty",
    "sector",
    "sector.jump",
    "reactor",
    "crew.<n>.skills.<pilot|engine|shield|weapon|repair|combat>",
    "crew.<n>.health",
    "crew.<n>.name",
    "crew.<n>.race",
    "crew.<n>.tints.<layer>",
    "crew.<n>.male",
    "crew.<n>.remove",
    "crew.add",
    "systems.<shields|engines|oxygen|weapons|drones|medbay|pilot|sensors|doors|teleporter|cloaking|artillery|battery|clonebay|mind|hacking>.level",
    "systems.<id>.installed",
    "systems.<id>.damage",
    "systems.<id>.ion",
    "systems.shields.energy",
    "augments.<AUGMENT_ID>",
    "beacons.<n>.seen",
    "beacons.<n>.visited",
    "beacons.<n>.store.<shelf>.<slot>",
    "beacons.<n>.store.<fuel|missiles|drone_parts>",
    "fleet.offset",
    "nearby.<ship field>",
    "fires.<x>.<y>",
    "breaches.<x>.<y>",
    "rooms.<n>.oxygen",
    "weapons.<WEAPON_ID>",
    "weapons.<WEAPON_ID>.armed",
    "drones.<DRONE_ID>",
    "drones.<DRONE_ID>.armed",
    "cargo.<WEAPON_OR_DRONE_ID>",
    "raw.<path of ftled dump>",
    "state.<STATE_VAR_ID>",
];

impl Field {
    /// The fixed resource fields. Indexed fields (crew, systems) depend on
    /// the game and are listed in [`INDEXED_PATHS`].
    pub const ALL: [Resource; 5] = Resource::ALL;

    pub const HULL: Field = Field::Resource(Resource::Hull);
    pub const FUEL: Field = Field::Resource(Resource::Fuel);
    pub const DRONE_PARTS: Field = Field::Resource(Resource::DroneParts);
    pub const MISSILES: Field = Field::Resource(Resource::Missiles);
    pub const SCRAP: Field = Field::Resource(Resource::Scrap);

    /// The same field on the ship `ship`: `nearby.<path>` for the nearby
    /// ship. A field of the nearby ship stays one.
    pub fn on(self, ship: ShipId) -> Field {
        match (ship, self) {
            (ShipId::Nearby, f @ Field::Nearby(_)) => f,
            (ShipId::Nearby, f) => Field::Nearby(Box::new(f)),
            (ShipId::Player, f) => f,
        }
    }

    /// The ship the field edits.
    pub fn ship(&self) -> ShipId {
        match self {
            Field::Nearby(_) => ShipId::Nearby,
            _ => ShipId::Player,
        }
    }

    /// Whether the field belongs to a ship, so `nearby.` can stand before
    /// it: not the sector map, not the cargo hold (both belong to the
    /// player), not a field that names its ship already.
    pub fn is_ship_field(&self) -> bool {
        !matches!(
            self,
            Field::BeaconSeen(_)
                | Field::BeaconVisited(_)
                | Field::StoreStock { .. }
                | Field::StoreSupply { .. }
                | Field::FleetOffset
                | Field::ShipBeacon
                | Field::BeaconStore(_)
                | Field::StoreItem { .. }
                | Field::Cargo(_)
                | Field::Nearby(_)
                | Field::Raw(_)
                | Field::StateVar(_)
                | Field::Difficulty
                | Field::Sector
                | Field::SectorJump
        )
    }

    /// The field without its ship: the inner field of `nearby.<path>`.
    fn target(&self) -> Result<&Field, EditError> {
        match self {
            Field::Nearby(f) if f.is_ship_field() => Ok(f),
            Field::Nearby(_) => Err(EditError::UnknownField(self.path())),
            f => Ok(f),
        }
    }

    /// The canonical dotted path; parsing it gives the same field.
    pub fn path(&self) -> String {
        match self {
            Field::Resource(r) => r.path().to_owned(),
            Field::CrewSkill { crew, skill } => format!("crew.{crew}.skills.{}", skill.id()),
            Field::CrewHealth { crew } => format!("crew.{crew}.health"),
            Field::CrewName { crew } => format!("crew.{crew}.name"),
            Field::CrewRace { crew } => format!("crew.{crew}.race"),
            Field::CrewTint { crew, layer } => format!("crew.{crew}.tints.{layer}"),
            Field::CrewMale { crew } => format!("crew.{crew}.male"),
            Field::CrewRemove { crew } => format!("crew.{crew}.remove"),
            Field::CrewAdd => "crew.add".to_owned(),
            Field::Reactor => "reactor".to_owned(),
            Field::SystemLevel(kind) => format!("systems.{}.level", kind.id()),
            Field::SystemInstalled(kind) => format!("systems.{}.installed", kind.id()),
            Field::SystemDamage(kind) => format!("systems.{}.damage", kind.id()),
            Field::SystemIon(kind) => format!("systems.{}.ion", kind.id()),
            Field::ShieldEnergy => "systems.shields.energy".to_owned(),
            Field::Augment(id) => format!("augments.{id}"),
            Field::BeaconSeen(n) => format!("beacons.{n}.seen"),
            Field::BeaconVisited(n) => format!("beacons.{n}.visited"),
            Field::StoreStock {
                beacon,
                shelf,
                slot,
            } => format!("beacons.{beacon}.store.{shelf}.{slot}"),
            Field::StoreSupply { beacon, supply } => format!("beacons.{beacon}.store.{supply}"),
            Field::FleetOffset => "fleet.offset".to_owned(),
            Field::ShipBeacon => "beacons.current".to_owned(),
            Field::BeaconStore(n) => format!("beacons.{n}.store"),
            Field::StoreItem {
                beacon,
                shelf,
                slot,
            } => format!("beacons.{beacon}.store.{shelf}.{slot}.item"),
            Field::Nearby(f) => format!("nearby.{}", f.path()),
            Field::Fire { x, y } => format!("fires.{x}.{y}"),
            Field::Breach { x, y } => format!("breaches.{x}.{y}"),
            Field::RoomOxygen { room } => format!("rooms.{room}.oxygen"),
            Field::Weapon(id) => format!("weapons.{id}"),
            Field::WeaponArmed(id) => format!("weapons.{id}.armed"),
            Field::Drone(id) => format!("drones.{id}"),
            Field::DroneArmed(id) => format!("drones.{id}.armed"),
            Field::Cargo(id) => format!("cargo.{id}"),
            Field::Raw(path) => format!("raw.{path}"),
            Field::StateVar(id) => format!("state.{id}"),
            Field::ShipName => "ship.name".to_owned(),
            Field::ShipType => "ship.type".to_owned(),
            Field::Difficulty => "difficulty".to_owned(),
            Field::Sector => "sector".to_owned(),
            Field::SectorJump => "sector.jump".to_owned(),
        }
    }

    /// Whether the field sets the look of a crew member (`crew.<n>.tints.<layer>`,
    /// `crew.<n>.male`). A race change gives new tints
    /// ([`CrewMember::change_race`]), so a patch that changes both puts the
    /// look after the race.
    pub fn is_crew_look(&self) -> bool {
        matches!(
            self.target(),
            Ok(Field::CrewTint { .. } | Field::CrewMale { .. })
        )
    }

    /// Whether the field holds text ([`Value::Text`]) instead of a number.
    pub fn is_text(&self) -> bool {
        matches!(
            self.target(),
            Ok(Field::CrewName { .. }
                | Field::CrewRace { .. }
                | Field::StoreItem { .. }
                | Field::ShipName
                | Field::ShipType)
        )
    }

    /// The current value of any field, number or text.
    pub fn value(&self, game: &SavedGame) -> Result<Value, EditError> {
        match *self.target()? {
            Field::CrewName { crew } => {
                Ok(Value::Text(crew_member(game, self, crew)?.name.clone()))
            }
            Field::CrewRace { crew } => {
                Ok(Value::Text(crew_member(game, self, crew)?.race.clone()))
            }
            Field::StoreItem {
                beacon,
                shelf,
                slot,
            } => beacons::item_id(game, self, beacon, shelf, slot).map(Value::Text),
            Field::ShipName => identity::name(game, self).map(Value::Text),
            Field::ShipType => Ok(Value::Text(ship(game, self)?.blueprint_id.clone())),
            _ => self.get(game).map(Value::Int),
        }
    }

    /// The current value of a number field, or why the field does not exist
    /// in this game. A text field answers [`EditError::TextField`].
    pub fn get(&self, game: &SavedGame) -> Result<i32, EditError> {
        match self.target()? {
            Field::CrewName { .. }
            | Field::CrewRace { .. }
            | Field::StoreItem { .. }
            | Field::ShipName
            | Field::ShipType => Err(EditError::TextField {
                field: self.clone(),
            }),
            Field::Difficulty => Ok(identity::difficulty_number(game.difficulty)),
            Field::Sector => Ok(game.sector_number),
            Field::SectorJump => {
                sector_tree::current_index(game).ok_or_else(|| beacons::not_decoded(self))
            }
            &Field::Resource(r) => Ok(r.of(ship(game, self)?)),
            &Field::CrewSkill { crew, skill } => {
                Ok(crew_member(game, self, crew)?.skills.get(skill).points)
            }
            &Field::CrewHealth { crew } => Ok(crew_member(game, self, crew)?.health),
            &Field::CrewTint { crew, layer } => {
                let member = crew_member(game, self, crew)?;
                crew_list::tint_range(game, self, member, layer)?;
                Ok(crew_list::tint(member, layer))
            }
            &Field::CrewMale { crew } => Ok(i32::from(crew_member(game, self, crew)?.male)),
            &Field::CrewRemove { crew } => Ok(i32::from(crew_member(game, self, crew)?.removed)),
            Field::CrewAdd => {
                crew_list::add_range(game, self)?;
                Ok(0)
            }
            Field::Reactor => Ok(ship(game, self)?.reactor_power),
            &Field::SystemLevel(kind) => Ok(installed_system(game, self, kind)?.capacity),
            &Field::SystemDamage(kind) => Ok(installed_system(game, self, kind)?.damaged_bars),
            &Field::SystemIon(kind) => Ok(installed_system(game, self, kind)?.ionized_bars),
            Field::ShieldEnergy => Ok(energy_shield::info(game, self)?.energy_shield_layers),
            &Field::SystemInstalled(kind) => {
                Ok(i32::from(installs::is_installed(ship(game, self)?, kind)))
            }
            Field::Augment(id) => Ok(count(augments(game, self)?, id)),
            &Field::BeaconSeen(n) => Ok(i32::from(beacon(game, self, n)?.seen)),
            &Field::BeaconVisited(n) => beacons::visited(game, self, n),
            &Field::StoreStock {
                beacon,
                shelf,
                slot,
            } => beacons::stock(game, self, beacon, shelf, slot),
            &Field::StoreSupply { beacon, supply } => beacons::supply(game, self, beacon, supply),
            Field::FleetOffset => beacons::fleet_offset(game, self),
            Field::ShipBeacon => beacons::current(game, self),
            &Field::BeaconStore(n) => beacons::has_store(game, self, n),
            Field::Nearby(_) => unreachable!("target() unwraps the ship"),
            &Field::Fire { x, y } => {
                let (r, s) = grid_square(game, self, x, y)?;
                Ok(body(game, self)?.rooms[r].squares[s].fire_health)
            }
            &Field::Breach { x, y } => {
                grid_square(game, self, x, y)?;
                Ok(breach(game, self, x, y)?.map_or(0, |(_, b)| b.health))
            }
            &Field::RoomOxygen { room } => Ok(oxygen_room(game, self, room)?.oxygen),
            Field::Weapon(id) | Field::Drone(id) => {
                body(game, self)?;
                Ok(items::count(ship(game, self)?, self.slots(), id))
            }
            Field::WeaponArmed(id) | Field::DroneArmed(id) => {
                body(game, self)?;
                Ok(items::armed_count(ship(game, self)?, self.slots(), id))
            }
            Field::Cargo(id) => items::cargo_count(game, self, id),
            Field::Raw(path) => raw::get(game, path).ok_or_else(|| EditError::NoRawField {
                field: self.clone(),
            }),
            Field::StateVar(id) => Ok(state_var(game, self, id)?.value),
        }
    }

    /// The slots of a weapon or drone field.
    fn slots(&self) -> Slots {
        match self.target() {
            Ok(Field::Drone(_) | Field::DroneArmed(_)) => Slots::Drones,
            _ => Slots::Weapons,
        }
    }

    /// The sector tree of the game for [`Field::SectorJump`].
    fn tree(
        &self,
        game: &SavedGame,
        data: &dyn ShipLayouts,
    ) -> Result<sector_tree::GameTree, EditError> {
        sector_tree::tree_of(game, data.sector_catalog()).map_err(|reason| {
            EditError::NoSectorTree {
                field: self.clone(),
                reason,
            }
        })
    }

    /// Inclusive value range for this number field in this game, without
    /// the blueprints of `ftl.dat`: a weapon, drone or cargo field then
    /// answers [`EditError::NoGameData`] or [`EditError::UnknownItem`]. A
    /// text field answers [`EditError::TextField`].
    pub fn range(&self, game: &SavedGame) -> Result<(i32, i32), EditError> {
        self.range_in(game, &KnownLayouts::new())
    }

    /// Inclusive value range for this field in this game; `data` answers
    /// the slots of the ship and the weapon and drone blueprints.
    pub fn range_in(
        &self,
        game: &SavedGame,
        data: &dyn ShipLayouts,
    ) -> Result<(i32, i32), EditError> {
        match self.target()? {
            Field::CrewName { .. }
            | Field::CrewRace { .. }
            | Field::StoreItem { .. }
            | Field::ShipName
            | Field::ShipType => Err(EditError::TextField {
                field: self.clone(),
            }),
            Field::Difficulty => identity::difficulty_range(game, self),
            Field::Sector => Err(EditError::SectorFixed {
                field: self.clone(),
            }),
            Field::SectorJump => {
                let tree = self.tree(game, data)?;
                if let Some(reason) = sector_tree::jump_block(game, &tree) {
                    return Err(EditError::SectorStays {
                        field: self.clone(),
                        reason,
                    });
                }
                Ok((0, tree.tree.sectors.len() as i32 - 1))
            }
            &Field::Resource(r) => {
                ship(game, self)?;
                Ok(r.range())
            }
            &Field::CrewSkill { crew, skill } => {
                // Full skill is two mastery levels. The mastery flags stay as
                // they are; FTL announces a mastery when it sees the points.
                let race = &crew_member(game, self, crew)?.race;
                Ok((0, 2 * skill.interval(race)))
            }
            // 0 is a dead crew member, which FTL removes; the editor keeps them alive.
            &Field::CrewHealth { crew } => {
                Ok((1, max_health(&crew_member(game, self, crew)?.race)))
            }
            &Field::CrewTint { crew, layer } => {
                let member = crew_member(game, self, crew)?;
                crew_list::tint_range(game, self, member, layer)
            }
            &Field::CrewMale { crew } => {
                not_a_drone(game, self, crew)?;
                Ok((0, 1))
            }
            &Field::CrewRemove { crew } => {
                not_a_drone(game, self, crew)?;
                Ok(crew_list::remove_range(game, self, crew))
            }
            Field::CrewAdd => crew_list::add_range(game, self),
            Field::Reactor => {
                ship(game, self)?;
                Ok((0, MAX_REACTOR))
            }
            &Field::SystemLevel(kind) => {
                installed_system(game, self, kind)?;
                Ok((1, kind.max_level()))
            }
            &Field::SystemDamage(kind) => Ok((0, installed_system(game, self, kind)?.capacity)),
            &Field::SystemIon(kind) => {
                installed_system(game, self, kind)?;
                Ok((0, damage::MAX_ION))
            }
            Field::ShieldEnergy => Ok((
                0,
                energy_shield::cap(self.ship(), energy_shield::info(game, self)?),
            )),
            &Field::SystemInstalled(kind) => installs::range(game, self, kind, data),
            Field::Augment(_) => {
                augments(game, self)?;
                Ok((0, MAX_AUGMENTS as i32))
            }
            &Field::BeaconSeen(n) => {
                beacon(game, self, n)?;
                Ok((0, 1))
            }
            &Field::BeaconVisited(n) => beacons::visited_range(game, self, n),
            &Field::StoreStock {
                beacon,
                shelf,
                slot,
            } => beacons::stock_range(game, self, beacon, shelf, slot),
            &Field::StoreSupply { beacon, supply } => {
                beacons::supply_range(game, self, beacon, supply)
            }
            Field::FleetOffset => beacons::fleet_range(game, self),
            Field::ShipBeacon => beacons::current_range(game, self),
            &Field::BeaconStore(n) => {
                beacons::has_store(game, self, n)?;
                Ok((0, 1))
            }
            Field::Nearby(_) => unreachable!("target() unwraps the ship"),
            &Field::Fire { x, y } => {
                grid_square(game, self, x, y)?;
                Ok((0, MAX_FIRE_HEALTH))
            }
            &Field::Breach { x, y } => {
                grid_square(game, self, x, y)?;
                Ok((0, MAX_BREACH_HEALTH))
            }
            &Field::RoomOxygen { room } => {
                oxygen_room(game, self, room)?;
                Ok((0, MAX_ROOM_OXYGEN))
            }
            Field::Weapon(id) => items::count_range(game, self, Slots::Weapons, id, data),
            Field::Drone(id) => items::count_range(game, self, Slots::Drones, id, data),
            Field::WeaponArmed(id) => items::armed_range(game, self, Slots::Weapons, id, data),
            Field::DroneArmed(id) => items::armed_range(game, self, Slots::Drones, id, data),
            Field::Cargo(id) => items::cargo_range(game, self, id, data),
            Field::Raw(path) => raw::range(game, path).ok_or_else(|| EditError::NoRawField {
                field: self.clone(),
            }),
            Field::StateVar(id) => {
                state_var(game, self, id)?;
                Ok((i32::MIN, i32::MAX))
            }
        }
    }

    /// Checks the value and writes it; returns the old value.
    fn set(
        &self,
        game: &mut SavedGame,
        value: &Value,
        data: &dyn ShipLayouts,
    ) -> Result<Value, EditError> {
        match (self.target()?, value) {
            (
                &Field::StoreItem {
                    beacon,
                    shelf,
                    slot,
                },
                Value::Text(text),
            ) => beacons::set_item(game, self, beacon, (shelf, slot), text, data).map(Value::Text),
            (Field::CrewName { .. } | Field::CrewRace { .. }, Value::Text(text)) => {
                self.set_text(game, text).map(Value::Text)
            }
            (Field::ShipName, Value::Text(text)) => {
                identity::set_name(game, self, text).map(Value::Text)
            }
            (Field::ShipType, Value::Text(text)) => {
                identity::set_type(game, self, text, data).map(Value::Text)
            }
            (
                Field::CrewName { .. }
                | Field::CrewRace { .. }
                | Field::StoreItem { .. }
                | Field::ShipName
                | Field::ShipType,
                Value::Int(n),
            ) => Err(EditError::NotText {
                field: self.clone(),
                value: *n,
            }),
            (_, Value::Int(n)) => self.set_number(game, *n, data).map(Value::Int),
            (_, Value::Text(text)) => Err(EditError::NotAnInteger {
                field: self.clone(),
                text: text.clone(),
            }),
        }
    }

    /// Checks and writes a text field; returns the old text.
    fn set_text(&self, game: &mut SavedGame, text: &str) -> Result<String, EditError> {
        match *self.target()? {
            Field::CrewName { crew } => {
                crew_member(game, self, crew)?;
                let name = text.trim();
                let length = name.chars().count();
                if length == 0 || length > MAX_CREW_NAME || name.chars().any(char::is_control) {
                    return Err(EditError::BadName {
                        field: self.clone(),
                        text: text.to_owned(),
                    });
                }
                let member = &mut ship_mut(game, self).crew[crew];
                Ok(std::mem::replace(&mut member.name, name.to_owned()))
            }
            Field::CrewRace { crew } => {
                let member = crew_member(game, self, crew)?;
                if member.enemy_boarding_drone {
                    return Err(EditError::BoardingDrone {
                        field: self.clone(),
                    });
                }
                let to = race(text).ok_or_else(|| EditError::UnknownRace {
                    field: self.clone(),
                    text: text.to_owned(),
                })?;
                let member = &mut ship_mut(game, self).crew[crew];
                let old = member.race.clone();
                member.change_race(to);
                Ok(old)
            }
            _ => unreachable!("set() routes number fields to set_number"),
        }
    }

    /// Checks and writes a number field; returns the old value.
    fn set_number(
        &self,
        game: &mut SavedGame,
        value: i32,
        data: &dyn ShipLayouts,
    ) -> Result<i32, EditError> {
        let target = self.target()?;
        // Install and remove name their own reason instead of a range.
        if let &Field::SystemInstalled(kind) = target {
            return installs::set(game, self, kind, value, data);
        }
        // The last crew member of a side names its reason too.
        if let &Field::CrewRemove { crew } = target {
            not_a_drone(game, self, crew)?;
            if value == 0 || value == 1 {
                return crew_list::set_removed(game, self, crew, value);
            }
        }
        let (min, max) = self.range_in(game, data)?;
        if value < min || value > max {
            return Err(EditError::OutOfRange {
                field: self.clone(),
                value,
                min,
                max,
            });
        }
        let slot = match target {
            &Field::Resource(r) => r.slot(ship_mut(game, self)),
            &Field::CrewSkill { crew, skill } => {
                &mut ship_mut(game, self).crew[crew].skills.get_mut(skill).points
            }
            &Field::CrewHealth { crew } => &mut ship_mut(game, self).crew[crew].health,
            &Field::CrewTint { crew, layer } => {
                let layers = crew_list::tint_layers(game, crew_member(game, self, crew)?).len();
                let member = &mut ship_mut(game, self).crew[crew];
                return Ok(crew_list::set_tint(member, layers, layer, value));
            }
            &Field::CrewMale { crew } => {
                let member = &mut ship_mut(game, self).crew[crew];
                return Ok(i32::from(std::mem::replace(&mut member.male, value == 1)));
            }
            Field::CrewRemove { .. } => unreachable!("handled above"),
            Field::CrewAdd => return crew_list::add(game, self, value, data),
            Field::CrewName { .. }
            | Field::CrewRace { .. }
            | Field::ShipName
            | Field::ShipType
            | Field::Sector => {
                unreachable!("range() refuses text fields and the sector")
            }
            Field::Difficulty => return Ok(identity::set_difficulty(game, value)),
            Field::Reactor => {
                let ship = ship_mut(game, self);
                let old = std::mem::replace(&mut ship.reactor_power, value);
                fit_power_to_reactor(ship);
                return Ok(old);
            }
            Field::SystemInstalled(_) | Field::Nearby(_) => unreachable!("handled above"),
            &Field::SystemLevel(kind) => {
                let s = installed_system_mut(game, self, kind);
                let old = std::mem::replace(&mut s.capacity, value);
                fit_power_to_capacity(s);
                return Ok(old);
            }
            &Field::SystemDamage(kind) => {
                return Ok(damage::set_damage(
                    installed_system_mut(game, self, kind),
                    value,
                ));
            }
            &Field::SystemIon(kind) => {
                return Ok(damage::set_ion(
                    installed_system_mut(game, self, kind),
                    value,
                ));
            }
            Field::ShieldEnergy => return Ok(energy_shield::set(game, self, value)),
            Field::Augment(id) => return set_augment_count(game, self, id, value),
            &Field::BeaconSeen(n) => {
                let b = beacon_mut(game, n);
                let old = i32::from(b.seen);
                b.seen = value == 1;
                return Ok(old);
            }
            &Field::BeaconVisited(n) => return Ok(beacons::set_visited(game, n, value)),
            &Field::StoreStock {
                beacon,
                shelf,
                slot,
            } => return Ok(beacons::set_stock(game, beacon, shelf, slot, value)),
            &Field::StoreSupply { beacon, supply } => {
                return Ok(beacons::set_supply(game, beacon, supply, value));
            }
            Field::FleetOffset => return beacons::set_fleet_offset(game, self, value),
            Field::SectorJump => {
                let tree = self.tree(game, data)?;
                if let Some(reason) = sector_tree::jump_block_to(game, &tree, value as usize) {
                    return Err(EditError::SectorStays {
                        field: self.clone(),
                        reason,
                    });
                }
                return Ok(sector_tree::jump(game, &tree, value as usize));
            }
            Field::ShipBeacon => return beacons::set_current(game, self, value),
            &Field::BeaconStore(n) => return Ok(beacons::set_has_store(game, n, value)),
            Field::StoreItem { .. } => unreachable!("range() refuses text fields"),
            &Field::Fire { x, y } => return set_fire(game, self, x, y, value),
            &Field::Breach { x, y } => return set_breach(game, self, x, y, value),
            &Field::RoomOxygen { room } => {
                &mut ship_mut(game, self)
                    .body
                    .as_mut()
                    .expect("oxygen_room() checked the body")
                    .rooms[room]
                    .oxygen
            }
            Field::Weapon(id) => {
                return items::set_count(game, self, Slots::Weapons, id, value, data);
            }
            Field::Drone(id) => {
                return items::set_count(game, self, Slots::Drones, id, value, data);
            }
            Field::WeaponArmed(id) => {
                return items::set_armed_count(game, self, Slots::Weapons, id, value, data);
            }
            Field::DroneArmed(id) => {
                return items::set_armed_count(game, self, Slots::Drones, id, value, data);
            }
            Field::Cargo(id) => return items::set_cargo_count(game, self, id, value),
            Field::Raw(path) => {
                return raw::set(game, path, value).ok_or_else(|| EditError::NoRawField {
                    field: self.clone(),
                });
            }
            Field::StateVar(id) => {
                let i = game
                    .state_vars
                    .iter()
                    .position(|v| &v.id == id)
                    .expect("range() found the state var");
                &mut game.state_vars[i].value
            }
        };
        Ok(std::mem::replace(slot, value))
    }
}

/// The health FTL gives a fresh hull breach (Java: `breachHealth` 0-100;
/// its `addBreach` adds one with 100).
pub const MAX_BREACH_HEALTH: i32 = 100;

/// The health of a fresh fire (Java: `setFireHealth` 0-100; its `addFire`
/// lights a square with 100). FTL keeps both as a `Repairable` whose damage
/// runs up to `fMaxDamage` (Hyperspace `FTLGameStripped.h`, macOS 1.6.13:
/// `Fire` and `OuterHull` extend `Repairable`).
pub const MAX_FIRE_HEALTH: i32 = 100;

/// A full room: the oxygen `OxygenSystem::Restart` gives every room (100.0)
/// and the most the codec accepts.
pub const MAX_ROOM_OXYGEN: i32 = 100;

/// The ship the field edits, or why it is not there: the nearby ship needs
/// the decoded situation and a ship at the beacon.
pub(crate) fn ship<'g>(game: &'g SavedGame, field: &Field) -> Result<&'g Ship, EditError> {
    if let Some(ship) = game.ship(field.ship()) {
        return Ok(ship);
    }
    if game.situation.is_none() {
        return Err(EditError::NotDecoded {
            field: field.clone(),
            part: "the nearby ship",
        });
    }
    Err(EditError::NoNearbyShip {
        field: field.clone(),
    })
}

/// Call only after [`ship`] returned `Ok` for the same field.
pub(crate) fn ship_mut<'g>(game: &'g mut SavedGame, field: &Field) -> &'g mut Ship {
    game.ship_mut(field.ship())
        .expect("the range checked the ship")
}

fn body<'g>(game: &'g SavedGame, field: &Field) -> Result<&'g ShipBody, EditError> {
    ship(game, field)?
        .body
        .as_ref()
        .ok_or_else(|| EditError::NotDecoded {
            field: field.clone(),
            part: "the room list of the ship",
        })
}

/// The room index and square id of the grid square `x`, `y`; an error when
/// no room of the ship covers it (FTL keeps fires and breaches on
/// room squares only).
fn grid_square(
    game: &SavedGame,
    field: &Field,
    x: i32,
    y: i32,
) -> Result<(usize, usize), EditError> {
    body(game, field)?
        .rooms
        .iter()
        .enumerate()
        .find_map(|(r, room)| room.square_at(x, y).map(|s| (r, s)))
        .ok_or_else(|| EditError::NoSuchSquare {
            field: field.clone(),
            x,
            y,
        })
}

/// The room `room` of a ship with the oxygen system; FTL ignores the
/// oxygen of a ship without it at load ([`Field::RoomOxygen`]).
fn oxygen_room<'g>(game: &'g SavedGame, field: &Field, room: usize) -> Result<&'g Room, EditError> {
    let rooms = &body(game, field)?.rooms;
    let found = rooms.get(room).ok_or(EditError::NoSuchRoom {
        field: field.clone(),
        room,
        count: rooms.len(),
    })?;
    installed_system(game, field, SystemKind::Oxygen)?;
    Ok(found)
}

fn state_var<'g>(game: &'g SavedGame, field: &Field, id: &str) -> Result<&'g StateVar, EditError> {
    game.state_vars
        .iter()
        .find(|v| v.id == id)
        .ok_or_else(|| EditError::NoStateVar {
            field: field.clone(),
        })
}

fn burns(room: &Room) -> bool {
    room.squares.iter().any(|s| s.fire_health > 0)
}

/// Sets the fire health of a square; returns the old one. A value above 0
/// lights a quiet square as Java's `addFire` does (fire health only) and
/// marks its extinguishment as not running (-1, the value of a square
/// without a dying fire). 0 puts the square out with no progress towards a
/// new fire; once no square of the room burns, the ignition progress of
/// the whole room goes too, so the fire does not come back by itself.
fn set_fire(
    game: &mut SavedGame,
    field: &Field,
    x: i32,
    y: i32,
    value: i32,
) -> Result<i32, EditError> {
    let (r, s) = grid_square(game, field, x, y)?;
    let room = &mut ship_mut(game, field)
        .body
        .as_mut()
        .expect("grid_square() checked the body")
        .rooms[r];
    let square = &mut room.squares[s];
    let old = square.fire_health;
    if value == 0 {
        square.fire_health = 0;
        square.ignition_progress = 0;
        if !burns(room) {
            room.squares
                .iter_mut()
                .for_each(|q| q.ignition_progress = 0);
        }
    } else {
        if old <= 0 {
            square.extinguishment_progress = -1;
        }
        square.fire_health = value;
    }
    Ok(old)
}

fn breach<'g>(
    game: &'g SavedGame,
    field: &Field,
    x: i32,
    y: i32,
) -> Result<Option<(usize, &'g Breach)>, EditError> {
    Ok(body(game, field)?
        .breaches
        .iter()
        .enumerate()
        .find(|(_, b)| b.x == x && b.y == y))
}

/// Sets the health of the breach at a room square; returns the old health
/// (0 without a breach). 0 removes it from the list, which is how a sealed
/// breach looks in a file FTL writes; a new breach goes to the end of the
/// list, as Java's `addBreach` puts it into its breach map.
fn set_breach(
    game: &mut SavedGame,
    field: &Field,
    x: i32,
    y: i32,
    value: i32,
) -> Result<i32, EditError> {
    grid_square(game, field, x, y)?;
    let found = breach(game, field, x, y)?.map(|(i, b)| (i, b.health));
    let list = &mut ship_mut(game, field)
        .body
        .as_mut()
        .expect("grid_square() checked the body")
        .breaches;
    match (found, value) {
        (Some((i, old)), 0) => {
            list.remove(i);
            Ok(old)
        }
        (Some((i, old)), _) => {
            list[i].health = value;
            Ok(old)
        }
        (None, 0) => Ok(0),
        (None, _) => {
            list.push(Breach {
                x,
                y,
                health: value,
            });
            Ok(0)
        }
    }
}

fn augments<'g>(game: &'g SavedGame, field: &Field) -> Result<&'g Vec<String>, EditError> {
    ship(game, field)?
        .body
        .as_ref()
        .map(|b| &b.augments)
        .ok_or_else(|| EditError::NotDecoded {
            field: field.clone(),
            part: "the augment list of the ship",
        })
}

fn beacon<'g>(game: &'g SavedGame, field: &Field, index: usize) -> Result<&'g Beacon, EditError> {
    let world = game.world.as_ref().ok_or_else(|| EditError::NotDecoded {
        field: field.clone(),
        part: "the sector map",
    })?;
    world.beacons.get(index).ok_or(EditError::NoSuchBeacon {
        index,
        count: world.beacons.len(),
    })
}

/// Call only after [`beacon`] returned `Ok` for the same index.
fn beacon_mut(game: &mut SavedGame, index: usize) -> &mut Beacon {
    game.world
        .as_mut()
        .and_then(|w| w.beacons.get_mut(index))
        .expect("beacon() checked this index")
}

fn count(list: &[String], id: &str) -> i32 {
    list.iter().filter(|a| *a == id).count() as i32
}

/// Adds copies of the augment at the end, or removes its last copies, until
/// the ship carries `value` of them. Returns the old count.
fn set_augment_count(
    game: &mut SavedGame,
    field: &Field,
    id: &str,
    value: i32,
) -> Result<i32, EditError> {
    let old = count(augments(game, field)?, id);
    let total = augments(game, field)?.len() as i32 - old + value;
    if total > MAX_AUGMENTS as i32 {
        return Err(EditError::TooManyAugments {
            field: field.clone(),
            total,
        });
    }
    let list = &mut ship_mut(game, field)
        .body
        .as_mut()
        .expect("augments() checked the body")
        .augments;
    for _ in old..value {
        list.push(id.to_owned());
    }
    for _ in value..old {
        let last = list.iter().rposition(|a| a == id).expect("counted above");
        list.remove(last);
    }
    Ok(old)
}

/// The first slot of the system kind. Ships with a system in several rooms
/// (the flagship) store one slot per room; the player ship has one.
fn installed_system<'g>(
    game: &'g SavedGame,
    field: &Field,
    kind: SystemKind,
) -> Result<&'g SystemState, EditError> {
    ship(game, field)?
        .systems
        .iter()
        .find(|s| s.kind == kind)
        .and_then(|s| s.installed.as_ref())
        .ok_or_else(|| EditError::NotInstalled {
            field: field.clone(),
            system: kind,
        })
}

/// Call only after [`installed_system`] returned `Ok` for the same kind.
fn installed_system_mut<'g>(
    game: &'g mut SavedGame,
    field: &Field,
    kind: SystemKind,
) -> &'g mut SystemState {
    ship_mut(game, field)
        .systems
        .iter_mut()
        .find(|s| s.kind == kind)
        .and_then(|s| s.installed.as_mut())
        .expect("installed_system checked this slot")
}

/// Refuses a field of an enemy boarding drone: it is no crew member.
fn not_a_drone(game: &SavedGame, field: &Field, crew: usize) -> Result<(), EditError> {
    if crew_member(game, field, crew)?.enemy_boarding_drone {
        return Err(EditError::BoardingDrone {
            field: field.clone(),
        });
    }
    Ok(())
}

fn crew_member<'g>(
    game: &'g SavedGame,
    field: &Field,
    index: usize,
) -> Result<&'g CrewMember, EditError> {
    let crew = &ship(game, field)?.crew;
    crew.get(index).ok_or(EditError::NoSuchCrew {
        ship: field.ship(),
        index,
        count: crew.len(),
    })
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.path())
    }
}

impl From<Resource> for Field {
    fn from(r: Resource) -> Self {
        Field::Resource(r)
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.path())
    }
}

impl Serialize for Field {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.path())
    }
}

impl FromStr for Field {
    type Err = EditError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let unknown = || EditError::UnknownField(s.to_owned());
        if let Some(r) = Resource::ALL.into_iter().find(|r| r.path() == s) {
            return Ok(Field::Resource(r));
        }
        if let Some(path) = s.strip_prefix("raw.").filter(|p| !p.is_empty()) {
            return Ok(Field::Raw(path.to_owned()));
        }
        if let Some(id) = s.strip_prefix("state.").filter(|id| !id.is_empty()) {
            return Ok(Field::StateVar(id.to_owned()));
        }
        let parts: Vec<&str> = s.split('.').collect();
        match parts.as_slice() {
            ["crew", n, "skills", skill] => Ok(Field::CrewSkill {
                crew: index(n).ok_or_else(unknown)?,
                skill: SkillKind::from_id(skill).ok_or_else(unknown)?,
            }),
            ["crew", n, "health"] => Ok(Field::CrewHealth {
                crew: index(n).ok_or_else(unknown)?,
            }),
            ["crew", n, "name"] => Ok(Field::CrewName {
                crew: index(n).ok_or_else(unknown)?,
            }),
            ["crew", n, "race"] => Ok(Field::CrewRace {
                crew: index(n).ok_or_else(unknown)?,
            }),
            ["crew", n, "tints", layer] => Ok(Field::CrewTint {
                crew: index(n).ok_or_else(unknown)?,
                layer: index(layer).ok_or_else(unknown)?,
            }),
            ["crew", n, "male"] => Ok(Field::CrewMale {
                crew: index(n).ok_or_else(unknown)?,
            }),
            ["crew", n, "remove"] => Ok(Field::CrewRemove {
                crew: index(n).ok_or_else(unknown)?,
            }),
            ["crew", "add"] => Ok(Field::CrewAdd),
            ["systems", id, "level"] => Ok(Field::SystemLevel(
                SystemKind::from_id(id).ok_or_else(unknown)?,
            )),
            ["systems", id, "installed"] => Ok(Field::SystemInstalled(
                SystemKind::from_id(id).ok_or_else(unknown)?,
            )),
            ["systems", id, "damage"] => Ok(Field::SystemDamage(
                SystemKind::from_id(id).ok_or_else(unknown)?,
            )),
            ["systems", "shields", "energy"] => Ok(Field::ShieldEnergy),
            ["systems", id, "ion"] => Ok(Field::SystemIon(
                SystemKind::from_id(id).ok_or_else(unknown)?,
            )),
            ["augments", id] if is_blueprint_id(id) => Ok(Field::Augment((*id).to_owned())),
            ["beacons", n, "seen"] => Ok(Field::BeaconSeen(index(n).ok_or_else(unknown)?)),
            ["beacons", n, "visited"] => Ok(Field::BeaconVisited(index(n).ok_or_else(unknown)?)),
            ["beacons", "current"] => Ok(Field::ShipBeacon),
            ["beacons", n, "store"] => Ok(Field::BeaconStore(index(n).ok_or_else(unknown)?)),
            ["beacons", n, "store", shelf, slot, "item"] => Ok(Field::StoreItem {
                beacon: index(n).ok_or_else(unknown)?,
                shelf: index(shelf).ok_or_else(unknown)?,
                slot: index(slot).filter(|&s| s < 3).ok_or_else(unknown)?,
            }),
            ["beacons", n, "store", shelf, slot] => Ok(Field::StoreStock {
                beacon: index(n).ok_or_else(unknown)?,
                shelf: index(shelf).ok_or_else(unknown)?,
                slot: index(slot).filter(|&s| s < 3).ok_or_else(unknown)?,
            }),
            ["beacons", n, "store", supply] => Ok(Field::StoreSupply {
                beacon: index(n).ok_or_else(unknown)?,
                supply: StoreSupply::from_id(supply).ok_or_else(unknown)?,
            }),
            ["fleet", "offset"] => Ok(Field::FleetOffset),
            ["ship", "name"] => Ok(Field::ShipName),
            ["ship", "type"] => Ok(Field::ShipType),
            ["difficulty"] => Ok(Field::Difficulty),
            ["sector"] => Ok(Field::Sector),
            ["sector", "jump"] => Ok(Field::SectorJump),
            ["nearby", _, ..] => {
                let rest = s.strip_prefix("nearby.").ok_or_else(unknown)?;
                let inner: Field = rest.parse().map_err(|_| unknown())?;
                if inner.is_ship_field() {
                    Ok(Field::Nearby(Box::new(inner)))
                } else {
                    Err(unknown())
                }
            }
            ["fires", x, y] => Ok(Field::Fire {
                x: coordinate(x).ok_or_else(unknown)?,
                y: coordinate(y).ok_or_else(unknown)?,
            }),
            ["breaches", x, y] => Ok(Field::Breach {
                x: coordinate(x).ok_or_else(unknown)?,
                y: coordinate(y).ok_or_else(unknown)?,
            }),
            ["rooms", n, "oxygen"] => Ok(Field::RoomOxygen {
                room: index(n).ok_or_else(unknown)?,
            }),
            ["reactor"] => Ok(Field::Reactor),
            ["weapons", id] if is_blueprint_id(id) => Ok(Field::Weapon((*id).to_owned())),
            ["weapons", id, "armed"] if is_blueprint_id(id) => {
                Ok(Field::WeaponArmed((*id).to_owned()))
            }
            ["drones", id] if is_blueprint_id(id) => Ok(Field::Drone((*id).to_owned())),
            ["drones", id, "armed"] if is_blueprint_id(id) => {
                Ok(Field::DroneArmed((*id).to_owned()))
            }
            ["cargo", id] if is_blueprint_id(id) => Ok(Field::Cargo((*id).to_owned())),
            _ => Err(unknown()),
        }
    }
}

/// A blueprint id as FTL writes it: uppercase ASCII letters, digits, `_`.
fn is_blueprint_id(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
}

/// A list index in a path: decimal digits only, no sign, no leading `+`.
fn index(s: &str) -> Option<usize> {
    (!s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .then(|| s.parse().ok())
        .flatten()
}

/// A square coordinate in a path: like [`index`], as an `i32`.
fn coordinate(s: &str) -> Option<i32> {
    index(s).and_then(|n| i32::try_from(n).ok())
}

/// The value of a field: a number, or text for [`Field::is_text`] fields.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Int(i32),
    Text(String),
}

impl Value {
    /// The number, or `None` for text.
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Text(_) => None,
        }
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Int(n)
    }
}

impl From<&str> for Value {
    fn from(text: &str) -> Self {
        Value::Text(text.to_owned())
    }
}

impl PartialEq<i32> for Value {
    fn eq(&self, other: &i32) -> bool {
        self.as_int() == Some(*other)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Text(text) => f.write_str(text),
        }
    }
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Int(n) => s.serialize_i32(*n),
            Value::Text(text) => s.serialize_str(text),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Assignment {
    pub field: Field,
    pub value: Value,
}

impl Assignment {
    pub fn new(field: Field, value: impl Into<Value>) -> Self {
        Assignment {
            field,
            value: value.into(),
        }
    }
}

impl FromStr for Assignment {
    type Err = EditError;

    /// Parses `ship.scrap=9999` or `crew.0.name=Ann Lee` (the text of a text
    /// field without the spaces around it).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (path, value) = s
            .split_once('=')
            .ok_or_else(|| EditError::Syntax(s.to_owned()))?;
        let field: Field = path.trim().parse()?;
        if field.is_text() {
            return Ok(Assignment::new(field, value.trim()));
        }
        let value = value
            .trim()
            .parse::<i32>()
            .map_err(|_| EditError::NotAnInteger {
                field: field.clone(),
                text: value.trim().to_owned(),
            })?;
        Ok(Assignment::new(field, value))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Patch {
    pub assignments: Vec<Assignment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Change {
    pub field: Field,
    pub old: Value,
    pub new: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EditError {
    #[error("unknown field '{0}'; known fields: {known}", known = known_fields())]
    UnknownField(String),
    #[error("expected <field>=<value>, got '{0}'")]
    Syntax(String),
    #[error("{field}: '{text}' is not an integer")]
    NotAnInteger { field: Field, text: String },
    #[error("{field} holds text, not a number")]
    TextField { field: Field },
    #[error("{field} takes text, got the number {value}")]
    NotText { field: Field, value: i32 },
    #[error(
        "{field}: '{text}' is no crew name (1 to {MAX_CREW_NAME} characters, no control characters)"
    )]
    BadName { field: Field, text: String },
    #[error("{field}: unknown race '{text}'; known races: {known}", known = RACES.map(|r| r.id).join(", "))]
    UnknownRace { field: Field, text: String },
    #[error("{field}: the crew member is an enemy boarding drone; FTL makes its race battle")]
    BoardingDrone { field: Field },
    #[error("{field}: {value} is outside {min}..={max}")]
    OutOfRange {
        field: Field,
        value: i32,
        min: i32,
        max: i32,
    },
    #[error("no crew member {index}; {name} has {count} ({prefix}crew.0 to {prefix}crew.{last})", name = ship.name(), prefix = ship.prefix(), last = count.saturating_sub(1))]
    NoSuchCrew {
        ship: ShipId,
        index: usize,
        count: usize,
    },
    #[error("{field}: {name} has no {id} system; install it first ({install}=1)", name = field.ship().name(), id = system.id(), install = Field::SystemInstalled(*system).on(field.ship()))]
    NotInstalled { field: Field, system: SystemKind },
    #[error("{field}: {part} is not decoded; it needs the ship layouts from ftl.dat")]
    NotDecoded { field: Field, part: &'static str },
    #[error("no beacon {index}; the sector map has {count} (beacons.0 to beacons.{last})", last = count.saturating_sub(1))]
    NoSuchBeacon { index: usize, count: usize },
    #[error("{field}: no ship is at the current beacon")]
    NoNearbyShip { field: Field },
    #[error("{field}: no room of {name} covers square {x},{y}", name = field.ship().name())]
    NoSuchSquare { field: Field, x: i32, y: i32 },
    #[error("{field}: {name} has {count} rooms (0 to {last})", name = field.ship().name(), last = count.saturating_sub(1))]
    NoSuchRoom {
        field: Field,
        room: usize,
        count: usize,
    },
    #[error("{field}: the ship would carry {total} augments; FTL holds 3")]
    TooManyAugments { field: Field, total: i32 },
    #[error("{field}: the slots of the ship and the blueprints need ftl.dat")]
    NoGameData { field: Field },
    #[error("{field}: ftl.dat has no weapon or drone blueprint {id} that FTL equips")]
    UnknownItem { field: Field, id: String },
    #[error("{field}: FTL puts no {kind} drone in a drone slot", kind = kind.id())]
    NotEquippable { field: Field, kind: DroneKind },
    #[error("{field}: {name} has no {id} system", name = field.ship().name(), id = system.id())]
    NoSystem { field: Field, system: SystemKind },
    #[error("{field}: the drone is deployed; recall it in FTL first")]
    DroneDeployed { field: Field },
    #[error("{field}: {reason}")]
    NoMapLayout { field: Field, reason: NoLayout },
    #[error(
        "{field}: the Last Stand keeps the fleet of the flagship; FTL ignores the circle there"
    )]
    LastStand { field: Field },
    #[error("{field}: the ship is at this beacon; FTL clears its event while it is there")]
    CurrentBeacon { field: Field },
    #[error("{field}: the beacon has no store")]
    NoStore { field: Field },
    #[error("{field}: the store has no item in that slot")]
    NoStoreItem { field: Field },
    #[error("{field}: the ship stays at its beacon: {reason}")]
    ShipStays {
        field: Field,
        reason: beacons::MoveBlock,
    },
    #[error("{field}: {id} is no {kind} blueprint of ftl.dat for this shelf", kind = format!("{kind:?}").to_lowercase())]
    NotForShelf {
        field: Field,
        id: String,
        kind: crate::model::StoreItemKind,
    },
    #[error("{field}: the save has no number or flag at this path (ftled dump lists them)")]
    NoRawField { field: Field },
    #[error("{field}: the save has no state var of this name")]
    NoStateVar { field: Field },
    #[error("{field}: a new crew member needs the crew names of ftl.dat")]
    NoCrewNames { field: Field },
    #[error("{field}: every square of {name} holds a crew member", name = field.ship().name())]
    NoFreeSquare { field: Field },
    #[error(
        "{field}: the last crew member of a side stays; without crew FTL ends the game or counts the crew as killed"
    )]
    LastCrewMember { field: Field },
    #[error("{field}: the race {race} has {layers} tint layers in this file")]
    NoTintLayer {
        field: Field,
        race: String,
        layers: usize,
    },
    #[error("{field}: {reason}", reason = block.reason())]
    SystemBlocked {
        field: Field,
        system: SystemKind,
        block: InstallBlock,
    },
    #[error(
        "{field}: '{text}' is no ship name (1 to {max} characters, no control characters)",
        max = identity::MAX_SHIP_NAME
    )]
    BadShipName { field: Field, text: String },
    #[error("{field}: the ship cannot become {to}: {reason}", reason = block.reason())]
    TypeBlocked {
        field: Field,
        to: String,
        block: identity::TypeBlock,
    },
    #[error("{field}: the difficulty stays: {reason}", reason = block.reason())]
    DifficultyBlocked {
        field: Field,
        block: identity::DifficultyBlock,
    },
    #[error(
        "{field}: the sector follows the sector tree (its seed and the visited sectors); a changed number alone would give FTL the map of one sector and the level of another. A jump is sector.jump"
    )]
    SectorFixed { field: Field },
    #[error("{field}: {reason}")]
    NoSectorTree {
        field: Field,
        reason: sector_tree::NoTree,
    },
    #[error("{field}: the ship stays in its sector: {reason}")]
    SectorStays {
        field: Field,
        reason: sector_tree::JumpBlock,
    },
}

fn known_fields() -> String {
    let fixed = Field::ALL.map(Resource::path);
    fixed
        .iter()
        .chain(INDEXED_PATHS.iter())
        .copied()
        .collect::<Vec<_>>()
        .join(", ")
}

impl Patch {
    pub fn new(assignments: Vec<Assignment>) -> Self {
        Patch { assignments }
    }

    /// Validates and applies the assignments in order; one that fails waits
    /// for a later one that makes it possible ([`Patch::apply_in`]).
    ///
    /// Either every assignment applies or none does: the patch runs on a
    /// copy, which replaces the game only when every assignment passed. The
    /// result lists one change per assignment, in order, also when old and
    /// new are equal.
    pub fn apply(&self, game: &mut SavedGame) -> Result<Vec<Change>, EditError> {
        self.apply_in(game, &KnownLayouts::new())
    }

    /// [`Patch::apply`] with the slots of the ship and the weapon and drone
    /// blueprints that `data` answers.
    ///
    /// An assignment that fails in order waits until a later one makes it
    /// possible: an install staged before the removal that frees its place,
    /// the level of a system staged before its install, a removal staged
    /// before its items leave the slots. The editor stages edits in the
    /// order of the clicks, so the set, not its order, decides. Only when a
    /// pass over the waiting assignments applies none of them does the
    /// patch fail, with the first error of that pass.
    ///
    /// Crew members staged for removal (`crew.<n>.remove`) leave the ship
    /// at the end, so every index of the patch is an index of the file.
    /// A weapon, drone, augment or cargo item the patch adds takes the
    /// first slot a removal of the same patch freed, so a replace keeps
    /// the slot order ([`crate::slot_order`]).
    pub fn apply_in(
        &self,
        game: &mut SavedGame,
        data: &dyn ShipLayouts,
    ) -> Result<Vec<Change>, EditError> {
        let changes = self.stage_in(game, data)?;
        crew_list::drop_removed(game);
        Ok(changes)
    }

    /// [`Patch::apply_in`] without the removal of the crew members staged
    /// for removal: they stay with [`CrewMember::removed`] set, so a preview
    /// shows them in their place.
    pub fn stage_in(
        &self,
        game: &mut SavedGame,
        data: &dyn ShipLayouts,
    ) -> Result<Vec<Change>, EditError> {
        let mut next = game.clone();
        let mut holes = Holes::default();
        let in_order = self
            .assignments
            .iter()
            .map(|a| Self::apply_one(a, &mut next, &mut holes, data))
            .collect::<Result<Vec<_>, EditError>>();
        let changes = match in_order {
            Ok(changes) => changes,
            Err(_) => {
                next = game.clone();
                self.apply_waiting(&mut next, data)?
            }
        };
        *game = next;
        Ok(changes)
    }

    /// Applies one assignment; a weapon, drone, augment or cargo item it
    /// adds takes the first slot an earlier assignment freed
    /// ([`crate::slot_order`]).
    fn apply_one(
        a: &Assignment,
        game: &mut SavedGame,
        holes: &mut Holes,
        data: &dyn ShipLayouts,
    ) -> Result<Change, EditError> {
        let before = Snapshot::of(game);
        let old = a.field.set(game, &a.value, data)?;
        holes.settle(&before, game);
        // A text field answers what it stores (a name without the spaces
        // around it); a number field can be gone after the write (a sealed
        // breach), so it answers the assigned value.
        let new = if a.field.is_text() {
            a.field.value(game)?
        } else {
            a.value.clone()
        };
        Ok(Change {
            field: a.field.clone(),
            old,
            new,
        })
    }

    /// Applies the assignments in passes; a failed one leaves `game` as it
    /// was (it runs on a copy) and waits for the next pass. Returns one
    /// change per assignment, in the order of the patch.
    fn apply_waiting(
        &self,
        game: &mut SavedGame,
        data: &dyn ShipLayouts,
    ) -> Result<Vec<Change>, EditError> {
        let mut changes: Vec<Option<Change>> = vec![None; self.assignments.len()];
        let mut waiting: Vec<usize> = (0..self.assignments.len()).collect();
        let mut holes = Holes::default();
        while !waiting.is_empty() {
            let mut first_error = None;
            let before = waiting.len();
            waiting.retain(|&i| {
                let mut trial = game.clone();
                let mut trial_holes = holes.clone();
                match Self::apply_one(&self.assignments[i], &mut trial, &mut trial_holes, data) {
                    Ok(change) => {
                        *game = trial;
                        holes = trial_holes;
                        changes[i] = Some(change);
                        false
                    }
                    Err(e) => {
                        first_error.get_or_insert(e);
                        true
                    }
                }
            });
            if waiting.len() == before
                && let Some(e) = first_error
            {
                return Err(e);
            }
        }
        Ok(changes.into_iter().flatten().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn int(v: &Value) -> i32 {
        v.as_int().expect("a number field")
    }

    fn crew(name: &str, race: &str) -> CrewMember {
        CrewMember {
            name: name.into(),
            race: race.into(),
            enemy_boarding_drone: false,
            health: 100,
            sprite_x: 0,
            sprite_y: 0,
            room_id: 0,
            room_square: 0,
            player_controlled: true,
            clone_ready: 0,
            death_order: -1,
            sprite_tints: vec![],
            mind_controlled: false,
            saved_room_square: 0,
            saved_room_id: 0,
            skills: Skills::default(),
            male: true,
            repairs: 0,
            combat_kills: 0,
            piloted_evasions: 0,
            jumps_survived: 0,
            skill_masteries_earned: 0,
            stun_ticks: 0,
            health_boost: 0,
            clonebay_priority: -1,
            damage_boost: 1000,
            clone_dying: 0,
            universal_death_count: 0,
            teleporting: false,
            teleport_anim: Anim::default(),
            teleport_anim_reverse: false,
            lockdown_recharge_ticks: 0,
            lockdown_recharge_ticks_goal: 0,
            cloned: 0,
            removed: false,
        }
    }

    fn system(capacity: i32, power: i32) -> SystemState {
        SystemState {
            capacity,
            power,
            damaged_bars: 0,
            ionized_bars: 0,
            deionization_ticks: 0,
            repair_progress: 0,
            damage_progress: 0,
            battery_power: 0,
            hack_level: 0,
            hacked: false,
            temporary_capacity_cap: 1000,
            temporary_capacity_loss: 0,
            temporary_capacity_divisor: 1,
        }
    }

    fn game() -> SavedGame {
        SavedGame {
            file_format: 11,
            random_native: false,
            dlc_enabled: true,
            difficulty: Difficulty::Normal,
            stats: SessionStats {
                ships_defeated: 0,
                beacons_explored: 0,
                scrap_collected: 0,
                crew_hired: 0,
            },
            player_ship_name: "Test".into(),
            player_ship_blueprint_id: "PLAYER_SHIP_HARD".into(),
            sector_number: 1,
            victory: 0,
            state_vars: vec![],
            player_ship: Ship {
                blueprint_id: "PLAYER_SHIP_HARD".into(),
                name: "Test".into(),
                gfx_base_name: "kestral".into(),
                starting_crew: vec![],
                hostile: true,
                jump_charge_ticks: 0,
                jumping: false,
                jump_anim_ticks: 0,
                resources: Resources {
                    hull: 30,
                    fuel: 16,
                    drone_parts: 0,
                    missiles: 8,
                    scrap: 10,
                },
                crew: vec![crew("Ann", "human"), crew("Bob", "engi")],
                reactor_power: 8,
                systems: SystemKind::ALL
                    .map(|kind| System {
                        kind,
                        installed: (kind == SystemKind::Shields).then(|| system(2, 2)),
                    })
                    .to_vec(),
                clonebay: None,
                battery: None,
                shields: None,
                cloaking: None,
                body: None,
                extended: None,
            },
            world: None,
            situation: None,
            undecoded: vec![],
        }
    }

    #[test]
    fn parses_an_assignment() {
        let a: Assignment = "ship.scrap = 9999".parse().unwrap();
        assert_eq!(a, Assignment::new(Field::SCRAP, 9999));
    }

    #[test]
    fn names_the_known_fields_for_an_unknown_one() {
        let err = "ship.gold=1".parse::<Assignment>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "unknown field 'ship.gold'; known fields: ship.hull, ship.fuel, ship.drone_parts, ship.missiles, ship.scrap, ship.name, ship.type, difficulty, sector, sector.jump, reactor, crew.<n>.skills.<pilot|engine|shield|weapon|repair|combat>, crew.<n>.health, crew.<n>.name, crew.<n>.race, crew.<n>.tints.<layer>, crew.<n>.male, crew.<n>.remove, crew.add, systems.<shields|engines|oxygen|weapons|drones|medbay|pilot|sensors|doors|teleporter|cloaking|artillery|battery|clonebay|mind|hacking>.level, systems.<id>.installed, systems.<id>.damage, systems.<id>.ion, systems.shields.energy, augments.<AUGMENT_ID>, beacons.<n>.seen, beacons.<n>.visited, beacons.<n>.store.<shelf>.<slot>, beacons.<n>.store.<fuel|missiles|drone_parts>, fleet.offset, nearby.<ship field>, fires.<x>.<y>, breaches.<x>.<y>, rooms.<n>.oxygen, weapons.<WEAPON_ID>, weapons.<WEAPON_ID>.armed, drones.<DRONE_ID>, drones.<DRONE_ID>.armed, cargo.<WEAPON_OR_DRONE_ID>, raw.<path of ftled dump>, state.<STATE_VAR_ID>"
        );
    }

    #[test]
    fn a_crew_skill_path_parses_and_prints_back() {
        let f: Field = "crew.1.skills.pilot".parse().unwrap();
        assert_eq!(
            f,
            Field::CrewSkill {
                crew: 1,
                skill: SkillKind::Pilot
            }
        );
        assert_eq!(f.path(), "crew.1.skills.pilot");
        for bad in [
            "crew.x.skills.pilot",
            "crew.-1.skills.pilot",
            "crew.+1.skills.pilot",
            "crew.0.skills.cooking",
            "crew.0.skill.pilot",
        ] {
            assert!(
                matches!(bad.parse::<Field>(), Err(EditError::UnknownField(_))),
                "{bad}"
            );
        }
    }

    #[test]
    fn a_skill_is_capped_at_two_masteries_of_the_race() {
        let mut g = game();
        let human = Patch::new(vec!["crew.0.skills.pilot=26".parse().unwrap()]);
        assert_eq!(human.apply(&mut g).unwrap()[0].new, 26);
        let err = Patch::new(vec!["crew.0.skills.pilot=27".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(err.to_string(), "crew.0.skills.pilot: 27 is outside 0..=26");
        Patch::new(vec!["crew.1.skills.pilot=30".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(g.player_ship.crew[1].skills.pilot.points, 30);
    }

    #[test]
    fn crew_health_is_set_between_1_and_the_full_health_of_the_race() {
        let mut g = game();
        g.player_ship.crew[1].race = "rock".into();
        let f: Field = "crew.1.health".parse().unwrap();
        assert_eq!(
            (f.path(), f.range(&g).unwrap()),
            ("crew.1.health".to_owned(), (1, 150))
        );
        let c = Patch::new(vec!["crew.1.health=150".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(
            (int(&c[0].old), int(&c[0].new), g.player_ship.crew[1].health),
            (100, 150, 150)
        );
        let err = Patch::new(vec!["crew.0.health=0".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(err.to_string(), "crew.0.health: 0 is outside 1..=100");
        assert!(matches!(
            "crew.0.hp".parse::<Field>(),
            Err(EditError::UnknownField(_))
        ));
    }

    #[test]
    fn a_crew_name_is_text_and_is_stored_trimmed() {
        let mut g = game();
        let a: Assignment = "crew.1.name =  Bob Lee ".parse().unwrap();
        assert_eq!(a, Assignment::new(Field::CrewName { crew: 1 }, "Bob Lee"));
        let c = Patch::new(vec![a]).apply(&mut g).unwrap();
        assert_eq!(
            (c[0].old.clone(), c[0].new.clone()),
            ("Bob".into(), "Bob Lee".into())
        );
        assert_eq!(g.player_ship.crew[1].name, "Bob Lee");
        let f = Field::CrewName { crew: 1 };
        assert_eq!(f.value(&g).unwrap(), Value::from("Bob Lee"));
        assert_eq!(
            f.get(&g).unwrap_err().to_string(),
            "crew.1.name holds text, not a number"
        );
        for bad in [
            "crew.0.name=",
            "crew.0.name=a\tb",
            &format!("crew.0.name={}", "x".repeat(33)),
        ] {
            let err = Patch::new(vec![bad.parse().unwrap()])
                .apply(&mut g)
                .unwrap_err();
            assert!(matches!(err, EditError::BadName { .. }), "{bad}: {err}");
        }
        let err = Patch::new(vec![Assignment::new(f, 3)])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(err.to_string(), "crew.1.name takes text, got the number 3");
        let err = Patch::new(vec![Assignment::new(Field::SCRAP, "lots")])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(err.to_string(), "ship.scrap: 'lots' is not an integer");
    }

    #[test]
    fn a_race_change_fits_health_skills_tints_and_crystal_fields() {
        let mut g = game();
        // Ann: human, wounded, pilot full (26), engine mid level 1, tints for
        // a human (shirt 0, hair 3).
        let ann = &mut g.player_ship.crew[0];
        ann.health = 50;
        ann.skills.pilot.points = 26;
        ann.skills.engine.points = 20;
        ann.skills.combat.points = 3;
        ann.sprite_tints = vec![0, 3];
        let c = Patch::new(vec!["crew.0.race=rock".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(
            (c[0].old.clone(), c[0].new.clone()),
            ("human".into(), "rock".into())
        );
        let ann = &g.player_ship.crew[0];
        assert_eq!(ann.race, "rock");
        assert_eq!(ann.health, 75, "half of 150");
        assert_eq!(ann.skills.pilot.points, 30, "full stays full: 2 * 15");
        assert_eq!(ann.skills.engine.points, 15 + 7, "level 1 plus progress 7");
        assert_eq!(ann.skills.combat.points, 3);
        assert_eq!(
            ann.sprite_tints,
            [0],
            "one layer of 4 colors; the human layer had 1"
        );

        // Rock to Crystal: same layer size keeps the index; Crystal gets the
        // lockdown fields of a fresh player Crystal.
        g.player_ship.crew[0].sprite_tints = vec![2];
        Patch::new(vec!["crew.0.race=crystal".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        let ann = &g.player_ship.crew[0];
        assert_eq!((ann.health, ann.sprite_tints.clone()), (63, vec![2]));
        assert_eq!(
            ann.lockdown_recharge_ticks_goal, 50_000,
            "50.0 s as writeFloat stores it"
        );
        Patch::new(vec!["crew.0.race=engi".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        let ann = &g.player_ship.crew[0];
        assert!(ann.sprite_tints.is_empty());
        assert_eq!((ann.lockdown_recharge_ticks_goal, ann.health), (0, 50));
        assert_eq!(ann.skills.pilot.points, 30);

        let err = Patch::new(vec!["crew.0.race=ghost".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "crew.0.race: unknown race 'ghost'; known races: human, engi, mantis, slug, rock, energy, crystal, anaerobic"
        );
        g.player_ship.crew[1].enemy_boarding_drone = true;
        let err = Patch::new(vec!["crew.1.race=human".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert!(matches!(err, EditError::BoardingDrone { .. }), "{err}");
    }

    #[test]
    fn the_same_race_changes_nothing() {
        let mut g = game();
        g.player_ship.crew[0].health = 37;
        g.player_ship.crew[0].sprite_tints = vec![0, 3];
        let before = g.player_ship.crew[0].clone();
        Patch::new(vec!["crew.0.race=human".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(g.player_ship.crew[0], before);
    }

    #[test]
    fn a_missing_crew_member_is_named_with_the_crew_size() {
        let mut g = game();
        let err = Patch::new(vec!["crew.5.skills.pilot=1".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "no crew member 5; the player ship has 2 (crew.0 to crew.1)"
        );
    }

    #[test]
    fn a_system_level_is_set_within_the_system_maximum() {
        let mut g = game();
        let c = Patch::new(vec!["systems.shields.level=8".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(
            (int(&c[0].old), int(&c[0].new), c[0].field.path()),
            (2, 8, "systems.shields.level".to_owned())
        );
        let err = Patch::new(vec!["systems.shields.level=9".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(err.to_string(), "systems.shields.level: 9 is outside 1..=8");
    }

    #[test]
    fn a_missing_system_is_refused() {
        let mut g = game();
        let err = Patch::new(vec!["systems.oxygen.level=2".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "systems.oxygen.level: the player ship has no oxygen system; install it first (systems.oxygen.installed=1)"
        );
        assert!(matches!(
            "systems.warp.level".parse::<Field>(),
            Err(EditError::UnknownField(_))
        ));
    }

    fn shields(g: &SavedGame) -> &SystemState {
        installed_system(g, &Field::Reactor, SystemKind::Shields).unwrap()
    }

    #[test]
    fn a_system_level_below_its_power_lowers_the_power() {
        let mut g = game();
        g.player_ship.systems[SystemKind::Shields as usize].installed = Some(system(6, 4));
        let c = Patch::new(vec!["systems.shields.level=3".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!((int(&c[0].old), int(&c[0].new)), (6, 3));
        assert_eq!((shields(&g).capacity, shields(&g).power), (3, 3));
        // A raise afterwards keeps the lowered power.
        Patch::new(vec!["systems.shields.level=8".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!((shields(&g).capacity, shields(&g).power), (8, 3));
        let err = Patch::new(vec!["systems.shields.level=0".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(err.to_string(), "systems.shields.level: 0 is outside 1..=8");
    }

    #[test]
    fn a_lower_level_keeps_damaged_bars_inside_and_battery_bars_last() {
        let mut g = game();
        let mut s = system(4, 2);
        s.damaged_bars = 2;
        s.battery_power = 1;
        g.player_ship.systems[SystemKind::Shields as usize].installed = Some(s);
        Patch::new(vec!["systems.shields.level=3".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        let s = shields(&g);
        assert_eq!(
            (s.capacity, s.damaged_bars, s.power, s.battery_power),
            (3, 2, 1, 1)
        );
        Patch::new(vec!["systems.shields.level=1".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        let s = shields(&g);
        assert_eq!(
            (s.capacity, s.damaged_bars, s.power, s.battery_power),
            (1, 1, 0, 0)
        );
    }

    #[test]
    fn damage_breaks_bars_within_the_level_and_takes_their_power() {
        let mut g = game();
        let mut s = system(4, 4);
        s.repair_progress = 40;
        g.player_ship.systems[SystemKind::Shields as usize].installed = Some(s);
        let f: Field = "systems.shields.damage".parse().unwrap();
        assert_eq!(
            (f.path(), f.get(&g).unwrap(), f.range(&g).unwrap()),
            ("systems.shields.damage".to_owned(), 0, (0, 4))
        );
        Patch::new(vec!["systems.shields.damage=3".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        let s = shields(&g);
        assert_eq!(
            (s.damaged_bars, s.power, s.repair_progress),
            (3, 1, 0),
            "a damaged bar holds no power (CheckMaxPower)"
        );
        let err = Patch::new(vec!["systems.shields.damage=5".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "systems.shields.damage: 5 is outside 0..=4"
        );
        Patch::new(vec!["systems.shields.damage=0".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!((shields(&g).damaged_bars, shields(&g).power), (0, 1));
    }

    #[test]
    fn ion_locks_up_to_five_with_a_fresh_timer_and_keeps_the_power() {
        let mut g = game();
        let mut s = system(2, 2);
        s.deionization_ticks = 12_000;
        s.ionized_bars = 1;
        g.player_ship.systems[SystemKind::Shields as usize].installed = Some(s);
        let f: Field = "nearby.systems.shields.ion".parse().unwrap();
        assert_eq!(f.path(), "nearby.systems.shields.ion");
        let f: Field = "systems.shields.ion".parse().unwrap();
        assert_eq!((f.get(&g).unwrap(), f.range(&g).unwrap()), (1, (0, 5)));
        Patch::new(vec!["systems.shields.ion=5".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        let s = shields(&g);
        assert_eq!((s.ionized_bars, s.deionization_ticks, s.power), (5, 0, 2));
        assert!(
            Patch::new(vec!["systems.shields.ion=6".parse().unwrap()])
                .apply(&mut g)
                .is_err()
        );
        assert!(
            Patch::new(vec!["systems.cloaking.ion=1".parse().unwrap()])
                .apply(&mut g)
                .is_err(),
            "a system the ship lacks has no ion lock"
        );
    }

    #[test]
    fn the_reactor_is_set_between_0_and_25() {
        let mut g = game();
        let f: Field = "reactor".parse().unwrap();
        assert_eq!(
            (f.path(), f.get(&g).unwrap(), f.range(&g).unwrap()),
            ("reactor".to_owned(), 8, (0, 25))
        );
        let c = Patch::new(vec!["reactor=25".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(
            (int(&c[0].old), int(&c[0].new), g.player_ship.reactor_power),
            (8, 25, 25)
        );
        for bad in [26, -1] {
            let err = Patch::new(vec![format!("reactor={bad}").parse().unwrap()])
                .apply(&mut g)
                .unwrap_err();
            assert_eq!(err.to_string(), format!("reactor: {bad} is outside 0..=25"));
        }
    }

    #[test]
    fn a_reactor_below_the_power_in_use_takes_power_from_the_last_systems_first() {
        let mut g = game();
        // Shields 2 bars (first in the file), engines 2, weapons 3 of which
        // the battery feeds 1, the pilot (a subsystem) 1.
        g.player_ship.systems[SystemKind::Engines as usize].installed = Some(system(3, 2));
        g.player_ship.systems[SystemKind::Pilot as usize].installed = Some(system(1, 1));
        let mut weapons = system(4, 3);
        weapons.battery_power = 1;
        g.player_ship.systems[SystemKind::Weapons as usize].installed = Some(weapons);
        assert_eq!(reactor_power_in_use(&g.player_ship), 6);
        let all = |g: &SavedGame| {
            [
                SystemKind::Shields,
                SystemKind::Engines,
                SystemKind::Weapons,
                SystemKind::Pilot,
            ]
            .map(|k| installed_system(g, &Field::Reactor, k).unwrap().power)
        };

        Patch::new(vec!["reactor=6".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(all(&g), [2, 2, 3, 1]);
        // Weapons (last in the file) lose both reactor bars and keep the
        // battery bar, engines lose one, shields and the pilot keep theirs.
        let c = Patch::new(vec!["reactor=3".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!((int(&c[0].old), int(&c[0].new)), (6, 3));
        assert_eq!(all(&g), [2, 1, 1, 1]);
        assert_eq!(reactor_power_in_use(&g.player_ship), 3);
        // A raise afterwards gives nothing back.
        Patch::new(vec![
            "reactor=0".parse().unwrap(),
            "reactor=8".parse().unwrap(),
        ])
        .apply(&mut g)
        .unwrap();
        assert_eq!(all(&g), [0, 0, 1, 1]);
    }

    fn with_body(mut g: SavedGame, augments: &[&str]) -> SavedGame {
        g.player_ship.body = Some(ShipBody {
            layout_id: "kestral".into(),
            rooms: vec![],
            breaches: vec![],
            doors: vec![],
            cloak_anim_ticks: 0,
            lockdown_crystals: vec![],
            weapons: vec![],
            drones: vec![],
            augments: augments.iter().map(|a| a.to_string()).collect(),
        });
        g
    }

    fn augments(g: &SavedGame) -> Vec<String> {
        g.player_ship.body.as_ref().unwrap().augments.clone()
    }

    #[test]
    fn an_augment_count_adds_and_removes_that_augment() {
        let mut g = with_body(game(), &["NANO_MEDBAY"]);
        let c = Patch::new(vec!["augments.SCRAP_COLLECTOR=2".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(
            (c[0].field.path(), int(&c[0].old), int(&c[0].new)),
            ("augments.SCRAP_COLLECTOR".to_owned(), 0, 2)
        );
        assert_eq!(
            augments(&g),
            ["NANO_MEDBAY", "SCRAP_COLLECTOR", "SCRAP_COLLECTOR"]
        );
        Patch::new(vec!["augments.NANO_MEDBAY=0".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(augments(&g), ["SCRAP_COLLECTOR", "SCRAP_COLLECTOR"]);
    }

    #[test]
    fn more_than_three_augments_or_an_unknown_body_are_refused() {
        let mut g = with_body(game(), &["NANO_MEDBAY", "O2_MASKS"]);
        let err = Patch::new(vec!["augments.SCRAP_COLLECTOR=2".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "augments.SCRAP_COLLECTOR: the ship would carry 4 augments; FTL holds 3"
        );
        assert_eq!(augments(&g), ["NANO_MEDBAY", "O2_MASKS"]);
        let mut g = game();
        let err = Patch::new(vec!["augments.SCRAP_COLLECTOR=1".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "augments.SCRAP_COLLECTOR: the augment list of the ship is not decoded; it needs the ship layouts from ftl.dat"
        );
        for bad in ["augments.scrap", "augments.", "augments.A-B"] {
            assert!(
                matches!(bad.parse::<Field>(), Err(EditError::UnknownField(_))),
                "{bad}"
            );
        }
    }

    fn square(fire_health: i32) -> Square {
        Square {
            fire_health,
            ignition_progress: if fire_health > 0 { 40 } else { 0 },
            extinguishment_progress: -1,
        }
    }

    /// A body with two rooms on the grid: room 0 (2x1 at 3, 2) burns in its
    /// second square (4, 2), room 1 (1x1 at 5, 2) does not; one breach at
    /// (4, 2) with 70 health.
    fn with_fire_and_breach(mut g: SavedGame) -> SavedGame {
        let mut g2 = with_body(game(), &[]);
        let body = g2.player_ship.body.as_mut().unwrap();
        let room = |squares: Vec<Square>, x, h| Room {
            x,
            y: 2,
            squares_h: h,
            squares_v: 1,
            oxygen: 100,
            squares,
            station: None,
        };
        body.rooms = vec![
            room(vec![square(0), square(80)], 3, 2),
            room(vec![square(0)], 5, 1),
        ];
        body.breaches = vec![Breach {
            x: 4,
            y: 2,
            health: 70,
        }];
        g.player_ship.body = g2.player_ship.body;
        g
    }

    fn rooms(g: &SavedGame) -> &[Room] {
        &g.player_ship.body.as_ref().unwrap().rooms
    }

    /// Room oxygen is a percent per room id, and only a ship with the
    /// oxygen system has it: FTL ignores the value of any other ship.
    #[test]
    fn room_oxygen_needs_the_oxygen_system_and_stays_within_0_to_100() {
        let mut g = with_fire_and_breach(game());
        let f: Field = "rooms.1.oxygen".parse().unwrap();
        assert_eq!(f, Field::RoomOxygen { room: 1 });
        assert_eq!(f.path(), "rooms.1.oxygen");
        assert_eq!(
            f.get(&g).unwrap_err().to_string(),
            "rooms.1.oxygen: the player ship has no oxygen system; install it first (systems.oxygen.installed=1)"
        );
        let oxygen = g
            .player_ship
            .systems
            .iter_mut()
            .find(|s| s.kind == SystemKind::Oxygen)
            .unwrap();
        oxygen.installed = Some(system(1, 1));
        assert_eq!((f.get(&g).unwrap(), f.range(&g).unwrap()), (100, (0, 100)));
        let c = Patch::new(vec!["rooms.1.oxygen=35".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!((int(&c[0].old), int(&c[0].new)), (100, 35));
        assert_eq!((rooms(&g)[0].oxygen, rooms(&g)[1].oxygen), (100, 35));
        let err = Patch::new(vec!["rooms.0.oxygen=101".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(err.to_string(), "rooms.0.oxygen: 101 is outside 0..=100");
        let err = Patch::new(vec!["rooms.2.oxygen=50".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "rooms.2.oxygen: the player ship has 2 rooms (0 to 1)"
        );
        for bad in ["rooms.x.oxygen", "rooms.1", "rooms.-1.oxygen"] {
            assert!(
                matches!(bad.parse::<Field>(), Err(EditError::UnknownField(_))),
                "{bad}"
            );
        }
        assert_eq!(
            "nearby.rooms.3.oxygen".parse::<Field>().unwrap(),
            Field::RoomOxygen { room: 3 }.on(ShipId::Nearby)
        );
    }

    #[test]
    fn a_fire_is_the_health_of_one_square_and_0_puts_it_out() {
        let mut g = with_fire_and_breach(game());
        let f: Field = "fires.4.2".parse().unwrap();
        assert_eq!(f.path(), "fires.4.2");
        assert_eq!((f.get(&g).unwrap(), f.range(&g).unwrap()), (80, (0, 100)));
        let quiet: Field = "fires.3.2".parse().unwrap();
        assert_eq!(
            (quiet.get(&g).unwrap(), quiet.range(&g).unwrap()),
            (0, (0, 100))
        );
        // The neighbour grows towards a fire of its own.
        g.player_ship.body.as_mut().unwrap().rooms[0].squares[0].ignition_progress = 30;
        let c = Patch::new(vec!["fires.4.2=0".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!((int(&c[0].old), int(&c[0].new)), (80, 0));
        // The last fire of the room is out: no ignition progress is left.
        assert_eq!(rooms(&g)[0].squares, [square(0), square(0)]);
    }

    #[test]
    fn putting_out_one_of_two_fires_keeps_the_progress_of_the_other() {
        let mut g = with_fire_and_breach(game());
        g.player_ship.body.as_mut().unwrap().rooms[0].squares[0] = square(50);
        Patch::new(vec!["fires.4.2=0".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(rooms(&g)[0].squares, [square(50), square(0)]);
    }

    #[test]
    fn a_fire_lights_a_quiet_square_like_the_java_editor() {
        let mut g = with_fire_and_breach(game());
        g.player_ship.body.as_mut().unwrap().rooms[1].squares[0].extinguishment_progress = 3;
        let c = Patch::new(vec![
            format!("fires.5.2={MAX_FIRE_HEALTH}").parse().unwrap(),
        ])
        .apply(&mut g)
        .unwrap();
        assert_eq!((int(&c[0].old), int(&c[0].new)), (0, 100));
        assert_eq!(
            rooms(&g)[1].squares[0],
            Square {
                fire_health: 100,
                ignition_progress: 0,
                extinguishment_progress: -1,
            }
        );
        // A burning square only changes its health.
        Patch::new(vec!["fires.4.2=100".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(rooms(&g)[0].squares[1].ignition_progress, 40);
        let err = Patch::new(vec!["fires.5.2=101".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(err.to_string(), "fires.5.2: 101 is outside 0..=100");
    }

    #[test]
    fn a_square_without_a_room_or_an_undecoded_body_is_refused() {
        let mut g = with_fire_and_breach(game());
        for path in ["fires.6.2", "fires.4.3", "breaches.2.2"] {
            let err = Patch::new(vec![format!("{path}=1").parse().unwrap()])
                .apply(&mut g)
                .unwrap_err();
            let (x, y) = path.split_once('.').unwrap().1.split_once('.').unwrap();
            assert_eq!(
                err.to_string(),
                format!("{path}: no room of the player ship covers square {x},{y}")
            );
        }
        let err = "fires.4.2".parse::<Field>().unwrap().get(&game());
        assert_eq!(
            err.unwrap_err().to_string(),
            "fires.4.2: the room list of the ship is not decoded; it needs the ship layouts from ftl.dat"
        );
    }

    #[test]
    fn a_breach_health_of_0_seals_the_breach() {
        let mut g = with_fire_and_breach(game());
        let f: Field = "breaches.4.2".parse().unwrap();
        assert_eq!(f.path(), "breaches.4.2");
        assert_eq!((f.get(&g).unwrap(), f.range(&g).unwrap()), (70, (0, 100)));
        Patch::new(vec!["breaches.4.2=30".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(g.player_ship.body.as_ref().unwrap().breaches[0].health, 30);
        let c = Patch::new(vec!["breaches.4.2=0".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!((int(&c[0].old), int(&c[0].new)), (30, 0));
        assert!(g.player_ship.body.as_ref().unwrap().breaches.is_empty());
        assert_eq!(f.get(&g).unwrap(), 0);
        for bad in [
            "breaches.4",
            "breaches.-1.2",
            "breaches.a.b",
            "fires.4",
            "fires.-1.2",
            "rooms.0.fire",
        ] {
            assert!(
                matches!(bad.parse::<Field>(), Err(EditError::UnknownField(_))),
                "{bad}"
            );
        }
    }

    #[test]
    fn a_breach_on_a_quiet_square_is_added_at_the_end() {
        let mut g = with_fire_and_breach(game());
        let c = Patch::new(vec![
            "breaches.3.2=100".parse().unwrap(),
            "breaches.5.2=0".parse().unwrap(),
        ])
        .apply(&mut g)
        .unwrap();
        assert_eq!(
            (
                int(&c[0].old),
                int(&c[0].new),
                int(&c[1].old),
                int(&c[1].new)
            ),
            (0, 100, 0, 0)
        );
        assert_eq!(
            g.player_ship.body.as_ref().unwrap().breaches,
            [
                Breach {
                    x: 4,
                    y: 2,
                    health: 70
                },
                Breach {
                    x: 3,
                    y: 2,
                    health: 100
                },
            ]
        );
    }

    fn with_beacons(mut g: SavedGame, seen: &[bool]) -> SavedGame {
        g.world = Some(World {
            cargo: vec![],
            sector_tree_seed: 0,
            sector_layout_seed: 0,
            rebel_fleet: RebelFleet {
                offset: 0,
                fudge: 0,
                pursuit_mod: 0,
                flagship_visible: false,
                flagship_hop: 0,
                flagship_moving: false,
                flagship_retreating: false,
                flagship_base_turns: 0,
            },
            current_beacon_id: 0,
            waiting: false,
            wait_event_seed: -1,
            last_escape_event: String::new(),
            sector_hazards_visible: false,
            sector_visitation: vec![],
            sector_number: 0,
            sector_is_hidden_crystal_worlds: false,
            beacons: seen
                .iter()
                .map(|&seen| Beacon {
                    visit_count: 0,
                    background: None,
                    seen,
                    enemy: None,
                    fleet_presence: FleetPresence::None,
                    under_attack: false,
                    store: None,
                })
                .collect(),
            quest_events: vec![],
            distant_quest_events: vec![],
        });
        g
    }

    #[test]
    fn a_beacon_seen_flag_is_0_or_1() {
        let f: Field = "beacons.1.seen".parse().unwrap();
        assert_eq!(
            (f.clone(), f.path()),
            (Field::BeaconSeen(1), "beacons.1.seen".to_owned())
        );
        let mut g = with_beacons(game(), &[true, true]);
        let c = Patch::new(vec!["beacons.1.seen=0".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!((int(&c[0].old), int(&c[0].new)), (1, 0));
        assert!(!g.world.as_ref().unwrap().beacons[1].seen);
        let err = Patch::new(vec!["beacons.1.seen=2".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(err.to_string(), "beacons.1.seen: 2 is outside 0..=1");
    }

    #[test]
    fn a_missing_beacon_or_an_undecoded_sector_map_is_refused() {
        let mut g = with_beacons(game(), &[true]);
        let err = Patch::new(vec!["beacons.3.seen=1".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "no beacon 3; the sector map has 1 (beacons.0 to beacons.0)"
        );
        let err = Patch::new(vec!["beacons.0.seen=1".parse().unwrap()])
            .apply(&mut game())
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "beacons.0.seen: the sector map is not decoded; it needs the ship layouts from ftl.dat"
        );
        assert!(matches!(
            "beacons.x.seen".parse::<Field>(),
            Err(EditError::UnknownField(_))
        ));
    }

    #[test]
    fn the_nearby_hull_needs_a_decoded_situation_with_a_ship() {
        let f: Field = "nearby.ship.hull".parse().unwrap();
        assert_eq!(f.path(), "nearby.ship.hull");
        let err = Patch::new(vec!["nearby.ship.hull=5".parse().unwrap()])
            .apply(&mut game())
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "nearby.ship.hull: the nearby ship is not decoded; it needs the ship layouts from ftl.dat"
        );
        let mut g = game();
        g.situation = Some(Situation {
            ship_event_present: 0,
            encounter: None,
            nearby_ship: None,
            environment: None,
            projectiles: vec![],
            player_teleport_target_room: 0,
            autofire: false,
            rebel_flagship: RebelFlagship {
                current_stage: 0,
                pending_stage: 1,
                power_ticks_goal: 0,
                power_ticks: 0,
                previous_occupancy: vec![],
            },
        });
        let err = Patch::new(vec!["nearby.ship.hull=5".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "nearby.ship.hull: no ship is at the current beacon"
        );
        let ship = g.player_ship.clone();
        g.situation.as_mut().unwrap().nearby_ship = Some(NearbyShip {
            rebel_flagship_nearby: false,
            ship,
            ai: None,
            teleport_target_room: 0,
        });
        let c = Patch::new(vec!["nearby.ship.hull=5".parse().unwrap()])
            .apply(&mut g)
            .unwrap();
        assert_eq!(int(&c[0].new), 5);
        assert_eq!(
            g.situation
                .unwrap()
                .nearby_ship
                .unwrap()
                .ship
                .resources
                .hull,
            5
        );
        assert_ne!(g.player_ship.resources.hull, 5);
    }

    fn with_nearby(mut g: SavedGame) -> SavedGame {
        let mut ship = g.player_ship.clone();
        ship.resources.hull = 12;
        g.situation = Some(Situation {
            ship_event_present: 0,
            encounter: None,
            nearby_ship: Some(NearbyShip {
                rebel_flagship_nearby: false,
                ship,
                ai: None,
                teleport_target_room: 0,
            }),
            environment: None,
            projectiles: vec![],
            player_teleport_target_room: 0,
            autofire: false,
            rebel_flagship: RebelFlagship {
                current_stage: 0,
                pending_stage: 1,
                power_ticks_goal: 0,
                power_ticks: 0,
                previous_occupancy: vec![],
            },
        });
        g
    }

    #[test]
    fn a_nearby_path_edits_the_same_field_on_the_nearby_ship() {
        for path in [
            "nearby.ship.hull",
            "nearby.reactor",
            "nearby.crew.1.health",
            "nearby.crew.0.skills.pilot",
            "nearby.systems.shields.level",
            "nearby.systems.cloaking.installed",
            "nearby.augments.SCRAP_COLLECTOR",
            "nearby.fires.3.4",
            "nearby.breaches.3.4",
            "nearby.weapons.LASER_BURST_1.armed",
        ] {
            let f: Field = path.parse().unwrap();
            assert_eq!((f.path(), f.ship()), (path.to_owned(), ShipId::Nearby));
        }
        assert_eq!(
            Field::Reactor.on(ShipId::Nearby).on(ShipId::Nearby).path(),
            "nearby.reactor"
        );
        for path in [
            "nearby.beacons.0.seen",
            "nearby.cargo.LASER_BURST_1",
            "nearby.nearby.reactor",
            "nearby.hull",
            "nearby",
        ] {
            assert!(
                matches!(path.parse::<Field>(), Err(EditError::UnknownField(_))),
                "{path}"
            );
        }

        let mut g = with_nearby(game());
        g.ship_mut(ShipId::Nearby).unwrap().systems[SystemKind::Shields as usize].installed =
            Some(system(6, 4));
        let c = Patch::new(vec![
            "nearby.ship.hull=3".parse().unwrap(),
            "nearby.reactor=2".parse().unwrap(),
            "nearby.crew.1.health=20".parse().unwrap(),
            "nearby.systems.shields.level=2".parse().unwrap(),
        ])
        .apply(&mut g)
        .unwrap();
        assert_eq!(
            c.iter()
                .map(|c| (int(&c.old), int(&c.new)))
                .collect::<Vec<_>>(),
            [(12, 3), (8, 2), (100, 20), (6, 2)]
        );
        let near = g.ship(ShipId::Nearby).unwrap();
        assert_eq!(
            (near.resources.hull, near.reactor_power, near.crew[1].health),
            (3, 2, 20)
        );
        let shields = near.systems[SystemKind::Shields as usize]
            .installed
            .as_ref()
            .unwrap();
        assert_eq!((shields.capacity, shields.power), (2, 2));
        assert_eq!(g.player_ship, game().player_ship);

        // The text fields of the crew too: a name and a race of the nearby ship.
        let f: Field = "nearby.crew.1.race".parse().unwrap();
        assert!(f.is_text());
        Patch::new(vec![
            "nearby.crew.0.name=Zed".parse().unwrap(),
            "nearby.crew.1.race=rock".parse().unwrap(),
        ])
        .apply(&mut g)
        .unwrap();
        let near = g.ship(ShipId::Nearby).unwrap();
        assert_eq!(
            (near.crew[0].name.as_str(), near.crew[1].race.as_str()),
            ("Zed", "rock")
        );
        assert_eq!(f.value(&g).unwrap(), Value::Text("rock".into()));
        assert_eq!(g.player_ship, game().player_ship);

        let err = Patch::new(vec!["nearby.crew.7.health=5".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "no crew member 7; the nearby ship has 2 (nearby.crew.0 to nearby.crew.1)"
        );
        let err = Patch::new(vec!["nearby.systems.engines.level=2".parse().unwrap()])
            .apply(&mut g)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "nearby.systems.engines.level: the nearby ship has no engines system; install it first (nearby.systems.engines.installed=1)"
        );
    }

    #[test]
    fn applies_all_and_reports_old_values() {
        let mut g = game();
        let changes = Patch::new(vec![
            "ship.scrap=9999".parse().unwrap(),
            "ship.fuel=99".parse().unwrap(),
        ])
        .apply(&mut g)
        .unwrap();
        assert_eq!(g.player_ship.resources.scrap, 9999);
        assert_eq!(g.player_ship.resources.fuel, 99);
        assert_eq!(
            changes[0],
            Change {
                field: Field::SCRAP,
                old: 10.into(),
                new: 9999.into()
            }
        );
    }

    #[test]
    fn applies_nothing_when_one_assignment_is_out_of_range() {
        let mut g = game();
        let err = Patch::new(vec![
            "ship.scrap=9999".parse().unwrap(),
            "ship.hull=0".parse().unwrap(),
        ])
        .apply(&mut g)
        .unwrap_err();
        assert_eq!(err.to_string(), "ship.hull: 0 is outside 1..=999");
        assert_eq!(g, game());
    }
}
