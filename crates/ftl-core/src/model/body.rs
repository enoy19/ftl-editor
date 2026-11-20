//! The part of a ship after its systems (`readShip` from the rooms on):
//! rooms, breaches, doors, lockdown crystals, weapons, drones, augments.
//!
//! The codec decodes it only when the [`crate::ShipLayouts`] port knows the
//! ship, because the file does not store how many rooms, squares and doors
//! there are.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShipBody {
    /// Layout file name the shape came from.
    pub layout_id: String,
    /// One room per room of the layout, in room id order.
    pub rooms: Vec<Room>,
    /// Hull breaches in file order.
    pub breaches: Vec<Breach>,
    /// One door per door of the layout. File order: the doors between two
    /// rooms in layout order, then the doors to vacuum.
    pub doors: Vec<Door>,
    /// Format 7 and later.
    pub cloak_anim_ticks: i32,
    /// Format 8 and later.
    pub lockdown_crystals: Vec<LockdownCrystal>,
    pub weapons: Vec<Weapon>,
    pub drones: Vec<Drone>,
    /// Augment blueprint ids, for example `SCRAP_COLLECTOR`. FTL allows three.
    pub augments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    /// Column of the top left square on the ship grid, layout offset
    /// included (the coordinates of [`Breach`]); from the layout.
    pub x: i32,
    /// Row of the top left square, layout offset included; from the layout.
    pub y: i32,
    /// Width in squares, from the layout.
    pub squares_h: u32,
    /// Height in squares, from the layout.
    pub squares_v: u32,
    /// Percent, 0 to 100.
    pub oxygen: i32,
    /// Squares by square id: left to right, then top to bottom. (The file
    /// stores them top to bottom, then left to right.)
    pub squares: Vec<Square>,
    /// Format 7 and later: where a crew member mans the room's system.
    pub station: Option<Station>,
}

impl Room {
    /// The square id of the grid square `x`, `y`, when the room covers it.
    pub fn square_at(&self, x: i32, y: i32) -> Option<usize> {
        let (h, v) = (self.squares_h as i32, self.squares_v as i32);
        let (dx, dy) = (x - self.x, y - self.y);
        ((0..h).contains(&dx) && (0..v).contains(&dy)).then(|| (dy * h + dx) as usize)
    }

    /// The grid square of square id `square`.
    pub fn grid_square(&self, square: usize) -> (i32, i32) {
        let h = self.squares_h as i32;
        let s = square as i32;
        (self.x + s % h, self.y + s / h)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Square {
    /// 0 without fire.
    pub fire_health: i32,
    pub ignition_progress: i32,
    pub extinguishment_progress: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Station {
    /// Square id of the station; -1 when the room has none.
    pub square: i32,
    pub direction: StationDirection,
}

/// Facing of a station, stored as 0 to 4 in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StationDirection {
    Down,
    Right,
    Up,
    Left,
    None,
}

impl StationDirection {
    pub const ALL: [StationDirection; 5] = [
        StationDirection::Down,
        StationDirection::Right,
        StationDirection::Up,
        StationDirection::Left,
        StationDirection::None,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Breach {
    /// Square coordinates on the ship grid, layout offset included.
    pub x: i32,
    pub y: i32,
    pub health: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Door {
    /// Format 7 and later.
    pub current_max_health: i32,
    /// Format 7 and later.
    pub health: i32,
    /// Format 7 and later.
    pub nominal_health: i32,
    pub open: bool,
    pub walking_through: bool,
    /// Format 7 and later. `Door::iBlast` (+0x28): the level of the door
    /// (0 to 3), which `Door::SetBlastDoor` derives from the doors system
    /// level and which picks the door health. Java: delta.
    pub door_level: i32,
    /// Format 7 and later. `Door::iHacked` (+0x2ac): the door is hacked;
    /// `SetBlastDoor` then forces level 2. Java: epsilon (a guess of drone
    /// lockdown).
    pub hacked: i32,
}

/// A Crystal lockdown shard (`readLockdownCrystal`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockdownCrystal {
    pub current_position_x: i32,
    pub current_position_y: i32,
    pub speed: i32,
    pub goal_position_x: i32,
    pub goal_position_y: i32,
    pub arrived: bool,
    pub done: bool,
    pub lifetime: i32,
    pub super_freeze: bool,
    pub locking_room: i32,
    pub anim_direction: i32,
    pub shard_progress: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    /// Weapon blueprint id, for example `BURST_LASER_2`.
    pub id: String,
    /// Powered.
    pub armed: bool,
    /// Format 2 only; later formats store the cooldown elsewhere.
    pub cooldown_ticks: i32,
}

/// A drone in the drone list (`readDrone`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Drone {
    /// Drone blueprint id, for example `COMBAT_1`.
    pub id: String,
    /// Powered (`Drone::GetPowered`).
    pub armed: bool,
    /// Out of its bay (`Drone::GetDeployed`). Java calls it
    /// `playerControlled`, but `ShipManager::ExportShip` (FTL 1.6.13,
    /// 0x10009ba90) writes the int of vtable slot 0x60 of `Drone`, which is
    /// `GetDeployed` (slot 0x38 is `GetPowered`, the `armed` int before it).
    /// The second fixture's enemy drone, armed and deployed, stores 1.
    pub deployed: bool,
    /// Position of a drone that walks the ship (boarding or repair drone).
    pub body_x: i32,
    pub body_y: i32,
    pub body_room_id: i32,
    pub body_room_square: i32,
    pub health: i32,
}
