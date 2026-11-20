//! Rooms to augments (`readShip` from the rooms on, `readRoom`, `readDoor`,
//! `readLockdownCrystal`, `readDrone`).
//!
//! The file does not store the number of rooms, their squares or the number
//! of doors; the caller passes them from the ship layout.

use binrw::{binrw, helpers::args_iter};

use super::{Bool32, FtlString};

#[binrw]
#[br(little, import(format: i32, rooms: Vec<(u32, u32)>, door_count: usize))]
#[bw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireShipBody {
    /// One per layout room; the pair is (squares_h, squares_v).
    #[br(parse_with = args_iter(rooms.iter().map(|&(h, v)| (format, h, v))))]
    #[bw(args(format))]
    pub rooms: Vec<WireRoom>,
    #[br(temp)]
    #[bw(calc = breaches.len() as u32)]
    breach_count: u32,
    #[br(count = breach_count)]
    pub breaches: Vec<WireBreach>,
    #[br(count = door_count, args { inner: (format,) })]
    #[bw(args(format))]
    pub doors: Vec<WireDoor>,
    #[brw(if(format >= 7))]
    pub cloak_anim_ticks: Option<i32>,
    #[brw(if(format >= 8))]
    pub lockdown_crystals: Option<WireCrystals>,
    #[br(temp)]
    #[bw(calc = weapons.len() as u32)]
    weapon_count: u32,
    #[br(count = weapon_count, args { inner: (format,) })]
    #[bw(args(format))]
    pub weapons: Vec<WireWeapon>,
    #[br(temp)]
    #[bw(calc = drones.len() as u32)]
    drone_count: u32,
    #[br(count = drone_count, args { inner: (format >= 11,) })]
    #[bw(args(format >= 11))]
    pub drones: Vec<WireDrone>,
    #[br(temp)]
    #[bw(calc = augments.len() as u32)]
    augment_count: u32,
    #[br(count = augment_count, args { inner: (format >= 11,) })]
    #[bw(args(format >= 11))]
    pub augments: Vec<FtlString>,
}

#[binrw]
#[br(little, import(format: i32, squares_h: u32, squares_v: u32))]
#[bw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireRoom {
    #[br(assert((0..=100).contains(&oxygen), "room oxygen {} is outside 0..=100", oxygen))]
    pub oxygen: i32,
    /// Disk order: for each column left to right, its squares top to bottom.
    #[br(count = squares_h * squares_v)]
    pub squares: Vec<WireSquare>,
    #[brw(if(format >= 7))]
    pub station: Option<WireStation>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireSquare {
    pub fire_health: i32,
    pub ignition_progress: i32,
    pub extinguishment_progress: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireStation {
    pub square: i32,
    /// 0 down, 1 right, 2 up, 3 left, 4 none.
    #[br(assert((0..=4).contains(&direction), "station direction flag {} is outside 0..=4", direction))]
    pub direction: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireBreach {
    pub x: i32,
    pub y: i32,
    pub health: i32,
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireDoor {
    /// Format 7 and later: current max health, health, nominal health.
    #[brw(if(format >= 7))]
    pub health: Option<[i32; 3]>,
    pub open: Bool32,
    pub walking_through: Bool32,
    /// Format 7 and later: the door level and the hacked flag
    /// (`Door::SaveState`; Java: delta and epsilon).
    #[brw(if(format >= 7))]
    pub level_hacked: Option<[i32; 2]>,
}

/// Format 8 and later.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireCrystals {
    #[br(temp)]
    #[bw(calc = crystals.len() as u32)]
    count: u32,
    #[br(count = count)]
    pub crystals: Vec<WireLockdownCrystal>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireLockdownCrystal {
    pub current_position_x: i32,
    pub current_position_y: i32,
    pub speed: i32,
    pub goal_position_x: i32,
    pub goal_position_y: i32,
    pub arrived: Bool32,
    pub done: Bool32,
    pub lifetime: i32,
    pub super_freeze: Bool32,
    pub locking_room: i32,
    pub anim_direction: i32,
    pub shard_progress: i32,
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireWeapon {
    #[brw(args(format >= 11))]
    pub id: FtlString,
    pub armed: Bool32,
    /// Format 2 only.
    #[brw(if(format == 2))]
    pub cooldown_ticks: Option<i32>,
}

#[binrw]
#[brw(little, import(utf8: bool))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireDrone {
    #[brw(args(utf8))]
    pub id: FtlString,
    pub armed: Bool32,
    pub deployed: Bool32,
    pub body_x: i32,
    pub body_y: i32,
    pub body_room_id: i32,
    pub body_room_square: i32,
    pub health: i32,
}
