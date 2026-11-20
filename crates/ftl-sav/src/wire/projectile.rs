//! A projectile (`readProjectile`, `readDamage`, `readBeamProjectileInfo`,
//! `readBombProjectileInfo`, `readLaserProjectileInfo`,
//! `readPDSProjectileInfo`).

use binrw::binrw;

use super::{Bool32, FtlString, WireAnim};

/// The type flag, then the body unless the flag is 0 (invalid).
#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireProjectile {
    /// 0 invalid, 1 laser or burst, 2 rock or explosion, 3 missile, 4 bomb,
    /// 5 beam, 6 PDS (format 11 only).
    #[br(assert(
        (0..=5).contains(&kind) || (kind == 6 && format == 11),
        "projectile type flag {} is unknown for file format {}", kind, format
    ))]
    pub kind: i32,
    #[br(if(kind != 0), args(format >= 11, kind))]
    #[bw(args(format >= 11, *kind))]
    pub body: Option<WireProjectileBody>,
}

#[binrw]
#[brw(little, import(utf8: bool, kind: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireProjectileBody {
    pub current_position_x: i32,
    pub current_position_y: i32,
    pub previous_position_x: i32,
    pub previous_position_y: i32,
    pub speed: i32,
    pub goal_position_x: i32,
    pub goal_position_y: i32,
    pub heading: i32,
    pub owner_id: i32,
    pub self_id: i32,
    pub damage: WireDamage,
    pub lifespan: i32,
    pub destination_space: i32,
    pub current_space: i32,
    pub target_id: i32,
    pub dead: Bool32,
    #[brw(args(utf8))]
    pub death_anim_id: FtlString,
    #[brw(args(utf8))]
    pub flight_anim_id: FtlString,
    pub death_anim: WireAnim,
    pub flight_anim: WireAnim,
    pub velocity_x: i32,
    pub velocity_y: i32,
    pub missed: Bool32,
    pub hit_target: Bool32,
    #[brw(args(utf8))]
    pub hit_solid_sound: FtlString,
    #[brw(args(utf8))]
    pub hit_shield_sound: FtlString,
    #[brw(args(utf8))]
    pub miss_sound: FtlString,
    pub entry_angle: i32,
    pub started_dying: Bool32,
    pub passed_target: Bool32,
    /// Java: type.
    pub subtype: i32,
    pub broadcast_target: Bool32,
    #[br(args(kind))]
    pub info: WireProjectileInfo,
}

/// The variant part, selected by the type flag.
#[binrw]
#[br(little, import(kind: i32))]
#[bw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireProjectileInfo {
    #[br(pre_assert(kind == 1))]
    Laser { spin_angle: i32, spin: i32 },
    /// Rock or explosion (2) and missile (3) store nothing more.
    #[br(pre_assert(kind == 2 || kind == 3))]
    Empty,
    #[br(pre_assert(kind == 4))]
    Bomb {
        missed: i32,
        fuse_ticks: i32,
        super_shield: i32,
        super_shield_bypass: i32,
        arrived: Bool32,
    },
    #[br(pre_assert(kind == 5))]
    Beam(WireBeamInfo),
    #[br(pre_assert(kind == 6))]
    Pds {
        start_point_x: i32,
        start_point_y: i32,
        passed_target: i32,
        current_scale: i32,
        missed: i32,
        explosion_anim: WireAnim,
    },
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireBeamInfo {
    pub emission_end_x: i32,
    pub emission_end_y: i32,
    pub strafe_source_x: i32,
    pub strafe_source_y: i32,
    pub strafe_end_x: i32,
    pub strafe_end_y: i32,
    pub final_end_x: i32,
    pub final_end_y: i32,
    pub swath_end_x: i32,
    pub swath_end_y: i32,
    pub swath_start_x: i32,
    pub swath_start_y: i32,
    pub lifespan: i32,
    pub swath_length: i32,
    pub unknown_delta: i32,
    pub last_collision_x: i32,
    pub last_collision_y: i32,
    pub animation_timer: i32,
    pub last_damage: i32,
    pub emission_angle: i32,
    /// A float, times 1000 (`BeamWeapon::timer`), not a bool.
    pub timer: i32,
    pub pierced_shield: Bool32,
    pub from_drone_pod: Bool32,
    pub damage_super_shield: Bool32,
    /// An int (`BeamWeapon::movingTargetId`), not a bool.
    pub moving_target_id: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireDamage {
    pub hull_damage: i32,
    pub shield_piercing: i32,
    pub fire_chance: i32,
    pub breach_chance: i32,
    pub ion_damage: i32,
    pub system_damage: i32,
    pub personnel_damage: i32,
    pub hull_buster: Bool32,
    pub owner_id: i32,
    pub self_id: i32,
    pub lockdown: Bool32,
    pub crystal_shard: Bool32,
    pub stun_chance: i32,
    pub stun_amount: i32,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite};

    use super::WireBeamInfo;

    /// `BeamWeapon::SaveProjectile` writes the timer as a float (times 1000)
    /// and the moving target as an int; a beam in flight has a timer.
    #[test]
    fn a_beam_with_a_running_timer_round_trips() {
        let values: Vec<i32> = (1..=20).chain([1500, 1, 0, 0, 1]).collect();
        let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        let beam = WireBeamInfo::read_le(&mut Cursor::new(&bytes)).unwrap();
        assert_eq!(beam.timer, 1500);
        assert_eq!(beam.moving_target_id, 1);
        let mut out = Cursor::new(Vec::new());
        beam.write_le(&mut out).unwrap();
        assert_eq!(out.into_inner(), bytes);
    }
}
