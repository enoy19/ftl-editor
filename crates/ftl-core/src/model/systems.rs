//! Reactor and systems of a ship (`readSystem` and the extended system info
//! in `readShip` of SavedGameParser.java).

use serde::{Deserialize, Serialize};

/// The systems in file order. Format 2 stores the first twelve only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemKind {
    Shields,
    Engines,
    Oxygen,
    Weapons,
    Drones,
    Medbay,
    Pilot,
    Sensors,
    Doors,
    Teleporter,
    Cloaking,
    Artillery,
    Battery,
    Clonebay,
    Mind,
    Hacking,
}

impl SystemKind {
    pub const ALL: [SystemKind; 16] = [
        SystemKind::Shields,
        SystemKind::Engines,
        SystemKind::Oxygen,
        SystemKind::Weapons,
        SystemKind::Drones,
        SystemKind::Medbay,
        SystemKind::Pilot,
        SystemKind::Sensors,
        SystemKind::Doors,
        SystemKind::Teleporter,
        SystemKind::Cloaking,
        SystemKind::Artillery,
        SystemKind::Battery,
        SystemKind::Clonebay,
        SystemKind::Mind,
        SystemKind::Hacking,
    ];

    /// The system id FTL uses in blueprints and store items.
    pub fn id(self) -> &'static str {
        match self {
            SystemKind::Shields => "shields",
            SystemKind::Engines => "engines",
            SystemKind::Oxygen => "oxygen",
            SystemKind::Weapons => "weapons",
            SystemKind::Drones => "drones",
            SystemKind::Medbay => "medbay",
            SystemKind::Pilot => "pilot",
            SystemKind::Sensors => "sensors",
            SystemKind::Doors => "doors",
            SystemKind::Teleporter => "teleporter",
            SystemKind::Cloaking => "cloaking",
            SystemKind::Artillery => "artillery",
            SystemKind::Battery => "battery",
            SystemKind::Clonebay => "clonebay",
            SystemKind::Mind => "mind",
            SystemKind::Hacking => "hacking",
        }
    }

    pub fn from_id(id: &str) -> Option<SystemKind> {
        SystemKind::ALL.into_iter().find(|k| k.id() == id)
    }

    /// Pilot, sensors, doors and battery: they draw no reactor power and FTL's
    /// upgrade screen draws them with the short bar.
    pub fn is_subsystem(self) -> bool {
        matches!(
            self,
            SystemKind::Pilot | SystemKind::Sensors | SystemKind::Doors | SystemKind::Battery
        )
    }

    /// The highest level of the system in unmodded FTL 1.6.13.
    ///
    /// Source: `maxPower` of the system blueprints in ftl.dat, read through
    /// the Java editor (`DataManager.getSystem(id).getMaxPower()`). A mod can
    /// change these; the limits move to the blueprints once the editor reads
    /// ftl.dat.
    pub fn max_level(self) -> i32 {
        match self {
            SystemKind::Shields
            | SystemKind::Engines
            | SystemKind::Weapons
            | SystemKind::Drones => 8,
            SystemKind::Artillery => 4,
            SystemKind::Battery => 2,
            _ => 3,
        }
    }
}

/// One system slot of a ship, in file order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct System {
    pub kind: SystemKind,
    /// `None` when the ship does not have the system (capacity 0 on disk).
    pub installed: Option<SystemState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemState {
    /// The level the player has bought, at least 1.
    pub capacity: i32,
    /// Reactor bars assigned to the system.
    pub power: i32,
    pub damaged_bars: i32,
    /// Java notes that active mind control may store -1.
    pub ionized_bars: i32,
    /// May be i32::MIN.
    pub deionization_ticks: i32,
    pub repair_progress: i32,
    pub damage_progress: i32,
    /// Format 7 and later.
    pub battery_power: i32,
    /// Format 7 and later.
    pub hack_level: i32,
    /// Format 7 and later.
    pub hacked: bool,
    /// Format 7 and later. 1000 when no event limits the system.
    pub temporary_capacity_cap: i32,
    /// Format 7 and later.
    pub temporary_capacity_loss: i32,
    /// Format 7 and later. 1 when no event limits the system.
    pub temporary_capacity_divisor: i32,
}

/// Format 7 and later, stored when the clonebay is installed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClonebayInfo {
    pub build_ticks: i32,
    pub build_ticks_goal: i32,
    pub doom_ticks: i32,
}

/// Format 7 and later, stored when the backup battery is installed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub active: bool,
    pub used_battery: i32,
    pub discharge_ticks: i32,
}

/// Format 7 and later, stored for every ship, also without shields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShieldsInfo {
    /// Currently filled bubbles.
    pub shield_layers: i32,
    pub energy_shield_layers: i32,
    pub energy_shield_max: i32,
    pub shield_recharge_ticks: i32,
    pub drop_anim_on: bool,
    pub drop_anim_ticks: i32,
    pub raise_anim_on: bool,
    pub raise_anim_ticks: i32,
    pub energy_shield_anim_on: bool,
    pub energy_shield_anim_ticks: i32,
    /// `Shields::shieldsDownPoint` (+0x2e4, `Shields::Save`, FTL 1.6.13
    /// macOS 0x1001f0b20), times 1000: where the last hit broke a layer,
    /// the center of the shield-down animation. Java: lambda and mu.
    pub shields_down_point_x: i32,
    pub shields_down_point_y: i32,
}

/// Format 7 and later, stored when the cloaking is installed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloakingInfo {
    /// `CloakingSystem::bTurnedOn` (+0x264, `CloakingSystem::Save`,
    /// 0x1001e98d0): the ship is cloaked now. Java: alpha.
    pub turned_on: i32,
    /// Whether the cloak timer runs (`TimerHelper::Running` of
    /// `CloakingSystem::timer`, +0x268). Java: beta.
    pub timer_running: i32,
    pub cloak_ticks_goal: i32,
    /// May be i32::MIN.
    pub cloak_ticks: i32,
}
