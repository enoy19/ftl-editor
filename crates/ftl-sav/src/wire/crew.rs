//! The crew list of a ship (`readCrewMember`, `readAnim`).

use binrw::binrw;

use super::{Bool32, FtlString};

/// Race id after which a crew member carries [`WireCrystalLockdown`].
pub const CRYSTAL_RACE: &[u8] = b"crystal";

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireCrewMember {
    #[brw(args(format >= 11))]
    pub name: FtlString,
    #[brw(args(format >= 11))]
    pub race: FtlString,
    pub enemy_boarding_drone: Bool32,
    pub health: i32,
    pub sprite_x: i32,
    pub sprite_y: i32,
    pub room_id: i32,
    pub room_square: i32,
    pub player_controlled: Bool32,
    #[brw(if(format >= 7))]
    pub placement: Option<WireCrewPlacement>,
    /// Pilot, engine, shield, weapon, repair, combat.
    pub skills: [i32; 6],
    pub male: Bool32,
    pub repairs: i32,
    pub combat_kills: i32,
    pub piloted_evasions: i32,
    pub jumps_survived: i32,
    pub skill_masteries_earned: i32,
    #[br(if(format >= 7), args(format, race.0.as_bytes() == CRYSTAL_RACE))]
    #[bw(args(format, race.0.as_bytes() == CRYSTAL_RACE))]
    pub status: Option<WireCrewStatus>,
}

/// Format 7 and later, between the position and the skills.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireCrewPlacement {
    pub clone_ready: i32,
    pub death_order: i32,
    #[br(temp)]
    #[bw(calc = sprite_tints.len() as u32)]
    tint_count: u32,
    #[br(count = tint_count)]
    pub sprite_tints: Vec<i32>,
    pub mind_controlled: Bool32,
    pub saved_room_square: i32,
    pub saved_room_id: i32,
}

/// Format 7 and later, after the crew stats.
#[binrw]
#[brw(little, import(format: i32, crystal: bool))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireCrewStatus {
    pub stun_ticks: i32,
    pub health_boost: i32,
    pub clonebay_priority: i32,
    pub damage_boost: i32,
    pub clone_dying: i32,
    pub universal_death_count: i32,
    /// Format 8 and later: mastery one and two for each skill, in skill order.
    #[brw(if(format >= 8))]
    pub masteries: Option<[Bool32; 12]>,
    pub teleporting: Bool32,
    pub teleport_anim: WireAnim,
    pub teleport_anim_reverse: Bool32,
    #[brw(if(crystal))]
    pub lockdown: Option<WireCrystalLockdown>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireCrystalLockdown {
    pub recharge_ticks: i32,
    pub recharge_ticks_goal: i32,
    pub cloned: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireAnim {
    pub playing: Bool32,
    pub looping: Bool32,
    pub current_frame: i32,
    pub progress_ticks: i32,
    pub scale: i32,
    pub x: i32,
    pub y: i32,
}
