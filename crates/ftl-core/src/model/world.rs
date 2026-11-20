//! The sector map after the player ship (`readSavedGame` from the cargo to
//! the distant quests): cargo, seeds, the rebel fleet, the beacons with
//! their stores and the quests.
//!
//! The codec decodes it only together with the player ship body, because
//! it follows the body on disk.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    /// Item ids in the cargo hold (weapons and drones beyond the slots).
    pub cargo: Vec<String>,
    pub sector_tree_seed: i32,
    pub sector_layout_seed: i32,
    pub rebel_fleet: RebelFleet,
    /// Beacon id of the player ship. Format 2 stores it after the quests.
    pub current_beacon_id: i32,
    /// Format 7 and later: the player waits at the beacon.
    pub waiting: bool,
    /// Format 7 and later; -1 when not waiting.
    pub wait_event_seed: i32,
    /// Format 7 and later. `StarMap::lastEscapeEvent` (+0xc30, written by
    /// `StarMap::SaveGame`, FTL 1.6.13 macOS 0x1001ca4d0, between
    /// `fuelEventSeed` and `waitedLast`); the name is the FTL: Hyperspace
    /// header's. Empty in every observed file. Java: epsilon.
    pub last_escape_event: String,
    /// The hazards of the sector map are visible (long-range scanners).
    pub sector_hazards_visible: bool,
    /// One flag per column of the sector tree: whether the route passed it.
    pub sector_visitation: Vec<bool>,
    /// Zero-based sector number (the header stores it one-based).
    pub sector_number: i32,
    /// The sector is the hidden Crystal sector.
    pub sector_is_hidden_crystal_worlds: bool,
    /// Every beacon of the sector map, by beacon id.
    pub beacons: Vec<Beacon>,
    /// Quest events placed in this sector.
    pub quest_events: Vec<QuestEvent>,
    /// Quest event ids waiting for the next sector.
    pub distant_quest_events: Vec<String>,
}

/// The rebel fleet and the flagship position on the sector map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebelFleet {
    /// Pixel offset of the fleet front; it grows towards the exit.
    pub offset: i32,
    /// Random spread of the fleet front.
    pub fudge: i32,
    /// Extra advance per jump (events that delay or speed up the fleet).
    pub pursuit_mod: i32,
    /// The flagship is visible on the sector map (last sector).
    pub flagship_visible: bool,
    /// Beacon hops the flagship made towards the base.
    pub flagship_hop: i32,
    pub flagship_moving: bool,
    /// Format 7 and later.
    pub flagship_retreating: bool,
    /// Format 7 and later: turns the flagship has spent at the base.
    pub flagship_base_turns: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Beacon {
    /// How often the player arrived here; 0 for a beacon never visited.
    pub visit_count: i32,
    /// Background art, stored when `visit_count > 0`.
    pub background: Option<BeaconBackground>,
    /// The beacon is revealed on the map.
    pub seen: bool,
    /// The ship event waiting here, when an enemy is present.
    pub enemy: Option<BeaconEnemy>,
    pub fleet_presence: FleetPresence,
    pub under_attack: bool,
    pub store: Option<Store>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBackground {
    /// Inner path in `ftl.dat`, for example `stars/bg_darknebula.png`.
    pub starscape: String,
    /// Inner path in `ftl.dat`, for example `stars/planet_peach.png`.
    pub sprite: String,
    pub sprite_x: i32,
    pub sprite_y: i32,
    pub sprite_rotation: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconEnemy {
    /// Event id, for example `LONG_FLEET`.
    pub ship_event_id: String,
    /// Ship blueprint list or id, for example `SHIPS_REBEL_ELITE`.
    pub auto_blueprint_id: String,
    /// Equals the encounter seed while the player is at this beacon.
    pub ship_event_seed: i32,
}

/// Fleets at a beacon, stored as 0 to 3 in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FleetPresence {
    None,
    Rebel,
    Federation,
    Both,
}

impl FleetPresence {
    pub const ALL: [FleetPresence; 4] = [
        FleetPresence::None,
        FleetPresence::Rebel,
        FleetPresence::Federation,
        FleetPresence::Both,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Store {
    /// Format 2 has exactly two shelves; later formats any number.
    pub shelves: Vec<StoreShelf>,
    pub fuel: i32,
    pub missiles: i32,
    pub drone_parts: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreShelf {
    pub kind: StoreItemKind,
    /// Always three slots; `None` is an empty slot. The file keeps the slot
    /// position, so an empty slot may sit between two items.
    pub slots: [Option<StoreItem>; 3],
}

/// The kind of a store shelf, stored as 0 to 4 in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoreItemKind {
    Weapon,
    Drone,
    Augment,
    Crew,
    System,
}

impl StoreItemKind {
    pub const ALL: [StoreItemKind; 5] = [
        StoreItemKind::Weapon,
        StoreItemKind::Drone,
        StoreItemKind::Augment,
        StoreItemKind::Crew,
        StoreItemKind::System,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreItem {
    /// Blueprint id: a weapon, drone, augment, crew race or system.
    pub id: String,
    /// The availability flag as stored: 1 for sale, 0 sold. FTL writes -1
    /// for an empty slot (then the slot is `None`); other values are kept.
    pub available: i32,
    /// Format 8 and later. Java calls it extra data; its meaning depends on
    /// the item.
    pub extra_data: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestEvent {
    pub event_id: String,
    pub beacon_id: i32,
}
