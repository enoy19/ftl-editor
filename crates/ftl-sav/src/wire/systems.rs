//! Reactor and systems (`readSystem` and the extended system info in
//! `readShip`).
//!
//! Java reads one more system entry for each extra room the ship blueprint
//! gives a system (the flagship artillery). The caller passes the system
//! kind of every slot (`kinds`). Without a known ship it passes one slot per
//! system, which holds for every unmodded player ship; a modded player ship
//! with a multi-room system then decodes wrong bytes into the later fields
//! (still round-trips).

use binrw::binrw;

use super::Bool32;

/// Index of a system in the file order (see `SystemKind::ALL` in ftl-core).
pub const WEAPONS: usize = 3;
pub const CLOAKING: usize = 10;
pub const ARTILLERY: usize = 11;
pub const BATTERY: usize = 12;
pub const CLONEBAY: usize = 13;
pub const MIND: usize = 14;
pub const HACKING: usize = 15;

/// Number of system entries for a file format.
pub fn system_count(format: i32) -> usize {
    if format >= 7 { 16 } else { 12 }
}

/// Whether the first slot of the system kind `kind` is installed (capacity
/// above 0). `kinds` names the kind index of each slot.
pub fn installed(systems: &[WireSystem], kinds: &[usize], kind: usize) -> bool {
    kinds
        .iter()
        .position(|&k| k == kind)
        .and_then(|i| systems.get(i))
        .is_some_and(|s| s.capacity > 0)
}

/// Reactor, systems and extended system info of a ship.
#[binrw]
#[br(little, import(format: i32, kinds: Vec<usize>))]
#[bw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireShipSystems {
    /// Java: reservePowerCapacity.
    pub reactor_power: i32,
    #[br(count = kinds.len(), args { inner: (format,) })]
    #[bw(args(format))]
    pub systems: Vec<WireSystem>,
    #[br(if(format >= 7 && installed(&systems, &kinds, CLONEBAY)))]
    pub clonebay: Option<WireClonebayInfo>,
    #[br(if(format >= 7 && installed(&systems, &kinds, BATTERY)))]
    pub battery: Option<WireBatteryInfo>,
    #[br(if(format >= 7))]
    pub shields: Option<WireShieldsInfo>,
    #[br(if(format >= 7 && installed(&systems, &kinds, CLOAKING)))]
    pub cloaking: Option<WireCloakingInfo>,
}

/// A system slot. An absent system is the capacity 0 alone.
#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireSystem {
    pub capacity: i32,
    #[br(if(capacity > 0), args(format))]
    #[bw(args(format))]
    pub state: Option<WireSystemState>,
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireSystemState {
    pub power: i32,
    pub damaged_bars: i32,
    pub ionized_bars: i32,
    pub deionization_ticks: i32,
    pub repair_progress: i32,
    pub damage_progress: i32,
    #[brw(if(format >= 7))]
    pub advanced: Option<WireSystemAdvanced>,
}

/// Format 7 and later.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireSystemAdvanced {
    pub battery_power: i32,
    pub hack_level: i32,
    pub hacked: Bool32,
    pub temporary_capacity_cap: i32,
    pub temporary_capacity_loss: i32,
    pub temporary_capacity_divisor: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireClonebayInfo {
    pub build_ticks: i32,
    pub build_ticks_goal: i32,
    pub doom_ticks: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireBatteryInfo {
    pub active: Bool32,
    pub used_battery: i32,
    pub discharge_ticks: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireShieldsInfo {
    pub shield_layers: i32,
    pub energy_shield_layers: i32,
    pub energy_shield_max: i32,
    pub shield_recharge_ticks: i32,
    pub drop_anim_on: Bool32,
    pub drop_anim_ticks: i32,
    pub raise_anim_on: Bool32,
    pub raise_anim_ticks: i32,
    pub energy_shield_anim_on: Bool32,
    pub energy_shield_anim_ticks: i32,
    pub shields_down_point_x: i32,
    pub shields_down_point_y: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireCloakingInfo {
    pub turned_on: i32,
    pub timer_running: i32,
    pub cloak_ticks_goal: i32,
    pub cloak_ticks: i32,
}
