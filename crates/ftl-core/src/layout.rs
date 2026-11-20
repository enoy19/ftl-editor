//! The shape of a ship as its blueprint and layout file describe it.
//!
//! A saved game stores the rooms and doors of a ship without their count or
//! size: FTL reads those from `ftl.dat`. The codec therefore asks the
//! [`ShipLayouts`] port for the shape of each ship it decodes.

use std::collections::BTreeMap;
use std::fmt;

use crate::model::{DroneKind, Station, SystemKind};

/// Answers the facts of `ftl.dat` the codec needs: the shape of a ship by
/// its blueprint id, and the kind of a drone by its blueprint id.
///
/// The key is the blueprint id because that is what the saved game names;
/// the blueprint in turn names the layout (`ShipShape::layout_id`) and places
/// the systems in rooms. `None` means the ship is unknown, and the codec then
/// keeps everything from its rooms on undecoded.
pub trait ShipLayouts: fmt::Debug + Send + Sync {
    fn ship(&self, blueprint_id: &str) -> Option<ShipShape>;

    /// The kind of a drone blueprint. The extended ship info stores a drone
    /// pod whose fields depend on it; `None` keeps the encounter and
    /// everything after it undecoded.
    fn drone_kind(&self, drone_id: &str) -> Option<DroneKind>;

    /// The weapon and drone slots of a ship blueprint (`weaponSlots`,
    /// `droneSlots`); `None` for an unknown ship or one without the tags.
    /// The edits of weapons and drones need them, the codec does not.
    fn slots(&self, _blueprint_id: &str) -> Option<ItemSlots> {
        None
    }

    /// The facts of a weapon blueprint a new weapon needs; `None` for an
    /// unknown id.
    fn weapon(&self, _weapon_id: &str) -> Option<WeaponFacts> {
        None
    }

    /// The facts of a drone blueprint a new drone needs; `None` for an
    /// unknown id.
    fn drone(&self, _drone_id: &str) -> Option<DroneFacts> {
        None
    }

    /// Every `systemList` entry of a ship blueprint: the rooms the ship has
    /// for systems, also those it does not start with. `None` for an
    /// unknown ship. Installing and removing a system need them
    /// ([`crate::installs`]), the codec does not.
    fn system_rooms(&self, _blueprint_id: &str) -> Option<Vec<SystemRoom>> {
        None
    }

    /// The hull image base name of a ship blueprint (`img`, the name the
    /// save stores as `gfx_base_name`); `None` for an unknown ship. A type
    /// change needs it ([`crate::identity`]).
    fn ship_gfx(&self, _blueprint_id: &str) -> Option<String> {
        None
    }

    /// Whether `ftl.dat` has an augment blueprint of this id; false
    /// without game data. A store slot takes only a known augment
    /// ([`crate::beacons`]).
    fn is_augment(&self, _augment_id: &str) -> bool {
        false
    }

    /// The human crew names of `data/names.xml` (the lists without a
    /// language), male or female, in file order; empty without game data.
    /// A new crew member takes the first free one ([`crate::crew_list`]).
    fn crew_names(&self, _male: bool) -> Vec<String> {
        Vec::new()
    }

    /// The sector lists of `data/sector_data.xml`; `None` without game
    /// data. A sector jump needs them ([`crate::sector_tree`]).
    fn sector_catalog(&self) -> Option<&crate::sector_tree::SectorCatalog> {
        None
    }
}

/// One `systemList` entry of a ship blueprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemRoom {
    pub kind: SystemKind,
    /// The room id of the layout.
    pub room: usize,
    /// `power`: the level FTL installs the system at.
    pub power: i32,
    /// `max`: the ship's own cap of the level, if the entry names one.
    pub max: Option<i32>,
    /// `start` (FTL's default is true): the ship has the system when the
    /// run begins.
    pub start: bool,
    /// `<slot>`: the station of a system crew mans (square and direction),
    /// if the entry names one.
    pub slot: Option<Station>,
}

/// The slots a ship has for weapons and drones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemSlots {
    pub weapons: u32,
    pub drones: u32,
}

/// What a weapon blueprint says about a weapon in a slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponFacts {
    /// Reactor bars the weapon draws when armed.
    pub power: i32,
    /// `<cooldown>` in milliseconds (seconds times 1000 in `f32`); `None`
    /// for the weapons of drones.
    pub cooldown_ms: Option<i32>,
    /// `type` of the blueprint: `LASER`, `MISSILES`, `BEAM`, `BOMB`, `BURST`.
    pub kind: String,
}

/// What a drone blueprint says about a drone in a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DroneFacts {
    /// Reactor bars the drone draws when armed.
    pub power: i32,
    pub kind: DroneKind,
    /// `<cooldown>` in milliseconds; `None` without the tag.
    pub cooldown_ms: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipShape {
    /// Layout file name in `ftl.dat`, for example `kestral`.
    pub layout_id: String,
    /// Every room in room id order.
    pub rooms: Vec<RoomShape>,
    /// Number of doors in the layout, including the doors to vacuum.
    pub door_count: usize,
    /// Systems the blueprint places in more than one room, with their room
    /// count (the flagship artillery). Every other system has one slot.
    pub multi_room_systems: Vec<(SystemKind, usize)>,
}

impl ShipShape {
    /// Number of system slots the saved game stores for this system.
    pub fn system_slots(&self, kind: SystemKind) -> usize {
        self.multi_room_systems
            .iter()
            .find(|(k, _)| *k == kind)
            .map_or(1, |&(_, n)| n.max(1))
    }
}

/// Place and size of a room in squares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomShape {
    /// Column of the top left square on the ship grid, the layout offset
    /// (`X_OFFSET`) included: the coordinates a hull breach stores.
    pub x: i32,
    /// Row of the top left square, the layout offset (`Y_OFFSET`) included.
    pub y: i32,
    /// Squares from left to right.
    pub squares_h: u32,
    /// Squares from top to bottom.
    pub squares_v: u32,
}

impl RoomShape {
    pub fn square_count(self) -> usize {
        self.squares_h as usize * self.squares_v as usize
    }
}

/// A [`ShipLayouts`] that answers from a fixed table. Empty, it knows no
/// ship; tests fill it with the shapes of their fixtures.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnownLayouts {
    ships: BTreeMap<String, ShipShape>,
    drones: BTreeMap<String, DroneKind>,
    slots: BTreeMap<String, ItemSlots>,
    weapon_facts: BTreeMap<String, WeaponFacts>,
    drone_facts: BTreeMap<String, DroneFacts>,
    system_rooms: BTreeMap<String, Vec<SystemRoom>>,
    crew_names: Vec<String>,
    gfx: BTreeMap<String, String>,
}

impl KnownLayouts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, blueprint_id: &str, shape: ShipShape) -> Self {
        self.ships.insert(blueprint_id.to_owned(), shape);
        self
    }

    pub fn with_drone(mut self, drone_id: &str, kind: DroneKind) -> Self {
        self.drones.insert(drone_id.to_owned(), kind);
        self
    }

    pub fn with_slots(mut self, blueprint_id: &str, slots: ItemSlots) -> Self {
        self.slots.insert(blueprint_id.to_owned(), slots);
        self
    }

    pub fn with_weapon(mut self, weapon_id: &str, facts: WeaponFacts) -> Self {
        self.weapon_facts.insert(weapon_id.to_owned(), facts);
        self
    }

    /// Knows the drone with its facts; its kind answers [`ShipLayouts::drone_kind`] too.
    pub fn with_drone_facts(mut self, drone_id: &str, facts: DroneFacts) -> Self {
        self.drones.insert(drone_id.to_owned(), facts.kind);
        self.drone_facts.insert(drone_id.to_owned(), facts);
        self
    }

    pub fn with_system_rooms(mut self, blueprint_id: &str, rooms: Vec<SystemRoom>) -> Self {
        self.system_rooms.insert(blueprint_id.to_owned(), rooms);
        self
    }

    /// The hull image base name [`ShipLayouts::ship_gfx`] answers.
    pub fn with_gfx(mut self, blueprint_id: &str, gfx: &str) -> Self {
        self.gfx.insert(blueprint_id.to_owned(), gfx.to_owned());
        self
    }

    /// The male crew names [`ShipLayouts::crew_names`] answers.
    pub fn with_crew_names(mut self, names: &[&str]) -> Self {
        self.crew_names = names.iter().map(|n| (*n).to_owned()).collect();
        self
    }
}

impl ShipLayouts for KnownLayouts {
    fn ship(&self, blueprint_id: &str) -> Option<ShipShape> {
        self.ships.get(blueprint_id).cloned()
    }

    fn drone_kind(&self, drone_id: &str) -> Option<DroneKind> {
        self.drones.get(drone_id).copied()
    }

    fn slots(&self, blueprint_id: &str) -> Option<ItemSlots> {
        self.slots.get(blueprint_id).copied()
    }

    fn weapon(&self, weapon_id: &str) -> Option<WeaponFacts> {
        self.weapon_facts.get(weapon_id).cloned()
    }

    fn drone(&self, drone_id: &str) -> Option<DroneFacts> {
        self.drone_facts.get(drone_id).copied()
    }

    fn system_rooms(&self, blueprint_id: &str) -> Option<Vec<SystemRoom>> {
        self.system_rooms.get(blueprint_id).cloned()
    }

    fn ship_gfx(&self, blueprint_id: &str) -> Option<String> {
        self.gfx.get(blueprint_id).cloned()
    }

    fn crew_names(&self, male: bool) -> Vec<String> {
        if male {
            self.crew_names.clone()
        } else {
            Vec::new()
        }
    }
}
