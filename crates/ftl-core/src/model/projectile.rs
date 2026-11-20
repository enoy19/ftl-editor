//! A projectile in flight or queued in a weapon (`readProjectile`,
//! `readDamage` and the projectile info variants).

use serde::{Deserialize, Serialize};

use crate::model::Anim;

/// A projectile entry. FTL writes an invalid projectile (type flag 0) as the
/// flag alone; it is [`Projectile::Invalid`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Projectile {
    Invalid,
    Valid(Box<ProjectileState>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectileState {
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
    pub damage: Damage,
    pub lifespan: i32,
    pub destination_space: i32,
    pub current_space: i32,
    pub target_id: i32,
    pub dead: bool,
    pub death_anim_id: String,
    pub flight_anim_id: String,
    pub death_anim: Anim,
    pub flight_anim: Anim,
    pub velocity_x: i32,
    pub velocity_y: i32,
    pub missed: bool,
    pub hit_target: bool,
    pub hit_solid_sound: String,
    pub hit_shield_sound: String,
    pub miss_sound: String,
    pub entry_angle: i32,
    pub started_dying: bool,
    pub passed_target: bool,
    /// Java: type. Usually 4 for a laser and 2 for a burst.
    pub subtype: i32,
    pub broadcast_target: bool,
    /// The info that the type flag selects.
    pub info: ProjectileInfo,
}

/// The variant part of a projectile, selected by its type flag (1 to 6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectileInfo {
    /// Flag 1 (`LaserBlast::SaveProjectile`, FTL 1.6.13 macOS 0x100021460).
    LaserOrBurst {
        /// `LaserBlast::spinAngle` (+0x300), times 1000. Java: alpha.
        spin_angle: i32,
        /// `LaserBlast::spinSpeed` (+0x304), times 1000.
        spin: i32,
    },
    /// Flag 2: an asteroid or an explosion. No more fields.
    RockOrExplosion,
    /// Flag 3. No more fields.
    Missile,
    /// Flag 4 (`BombProjectile::SaveProjectile`, 0x100018d40).
    Bomb {
        /// `BombProjectile::bMissed` (+0x2f8). Java: alpha.
        missed: i32,
        /// `BombProjectile::explosiveDelay` (+0x308), times 1000.
        fuse_ticks: i32,
        /// `BombProjectile::bSuperShield` (+0x30c). Java: gamma.
        super_shield: i32,
        /// `BombProjectile::superShieldBypass` (+0x30d). Java: delta.
        super_shield_bypass: i32,
        arrived: bool,
    },
    /// Flag 5.
    Beam(BeamInfo),
    /// Flag 6, format 11 only: a shot of the planetary defense system
    /// (`PDSFire::SaveProjectile`, 0x1000234b0; Java: alpha to zeta).
    Pds {
        /// `PDSFire::startPoint` (+0x308), times 1000.
        start_point_x: i32,
        start_point_y: i32,
        /// `PDSFire::passedTarget` (+0x310).
        passed_target: i32,
        /// `PDSFire::currentScale` (+0x314), times 1000.
        current_scale: i32,
        /// `PDSFire::missed` (+0x318).
        missed: i32,
        /// `PDSFire::explosionAnimation` (+0x320).
        explosion_anim: Anim,
    },
}

impl ProjectileInfo {
    /// The type flag on disk.
    pub fn flag(&self) -> i32 {
        match self {
            ProjectileInfo::LaserOrBurst { .. } => 1,
            ProjectileInfo::RockOrExplosion => 2,
            ProjectileInfo::Missile => 3,
            ProjectileInfo::Bomb { .. } => 4,
            ProjectileInfo::Beam(_) => 5,
            ProjectileInfo::Pds { .. } => 6,
        }
    }
}

/// Fields of a beam (`readBeamProjectileInfo`), in the order
/// `BeamWeapon::SaveProjectile` (FTL 1.6.13 macOS 0x100015a80) writes them.
/// The names after the Java ones come from the `BeamWeapon` struct of the
/// FTL: Hyperspace headers (`FTLGameStripped.h`, GPL-3.0-or-later); the
/// offsets are the binary's. Floats are stored times 1000.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeamInfo {
    /// `sub_end` (+0x2f8).
    pub emission_end_x: i32,
    pub emission_end_y: i32,
    /// `sub_start` (+0x300).
    pub strafe_source_x: i32,
    pub strafe_source_y: i32,
    /// `shield_end` (+0x308).
    pub strafe_end_x: i32,
    pub strafe_end_y: i32,
    /// `final_end` (+0x310). Java: beta.
    pub final_end_x: i32,
    pub final_end_y: i32,
    /// `target2` (+0x318).
    pub swath_end_x: i32,
    pub swath_end_y: i32,
    /// `target1` (+0x320).
    pub swath_start_x: i32,
    pub swath_start_y: i32,
    /// `lifespan` (+0x328). Java: gamma.
    pub lifespan: i32,
    /// `length` (+0x32c).
    pub swath_length: i32,
    /// `dh` (+0x330); the header gives no more than this name. Java: delta.
    pub unknown_delta: i32,
    /// The point of `last_collision` (+0x338). Java: epsilon.
    pub last_collision_x: i32,
    pub last_collision_y: i32,
    /// `animationTimer` (+0x368). Java: zeta.
    pub animation_timer: i32,
    /// `lastDamage` (+0x36c), an int. Java: eta.
    pub last_damage: i32,
    /// `start_heading` (+0x378).
    pub emission_angle: i32,
    /// `timer` (+0x37c), a float like the others. Java reads a bool here
    /// (iota), which fails for every beam whose timer is not 0 or 0.001.
    pub timer: i32,
    /// `piercedShield` (+0x388). Java: kappa.
    pub pierced_shield: bool,
    /// `oneSpace` (+0x389).
    pub from_drone_pod: bool,
    /// `bDamageSuperShield` (+0x38a). Java: mu.
    pub damage_super_shield: bool,
    /// `movingTargetId` (+0x38c), an int: the ship id the beam follows.
    /// Java reads a bool here (nu).
    pub moving_target_id: i32,
}

/// The damage a projectile carries (`readDamage`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Damage {
    pub hull_damage: i32,
    pub shield_piercing: i32,
    pub fire_chance: i32,
    pub breach_chance: i32,
    pub ion_damage: i32,
    pub system_damage: i32,
    pub personnel_damage: i32,
    pub hull_buster: bool,
    pub owner_id: i32,
    pub self_id: i32,
    pub lockdown: bool,
    pub crystal_shard: bool,
    pub stun_chance: i32,
    pub stun_amount: i32,
}
