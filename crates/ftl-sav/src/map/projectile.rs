//! Projectiles: wire to domain and back.

use ftl_core::{BeamInfo, Damage, Projectile, ProjectileInfo, ProjectileState};

use super::crew::{anim_to_domain, anim_to_wire};
use crate::wire::{
    Bool32, FtlString, WireBeamInfo, WireDamage, WireProjectile, WireProjectileBody,
    WireProjectileInfo,
};

pub fn to_domain(w: WireProjectile) -> Projectile {
    let Some(b) = w.body else {
        return Projectile::Invalid;
    };
    let info = match b.info {
        WireProjectileInfo::Laser { spin_angle, spin } => {
            ProjectileInfo::LaserOrBurst { spin_angle, spin }
        }
        WireProjectileInfo::Empty if w.kind == 2 => ProjectileInfo::RockOrExplosion,
        WireProjectileInfo::Empty => ProjectileInfo::Missile,
        WireProjectileInfo::Bomb {
            missed,
            fuse_ticks,
            super_shield,
            super_shield_bypass,
            arrived,
        } => ProjectileInfo::Bomb {
            missed,
            fuse_ticks,
            super_shield,
            super_shield_bypass,
            arrived: arrived.0,
        },
        WireProjectileInfo::Beam(x) => ProjectileInfo::Beam(beam_to_domain(x)),
        WireProjectileInfo::Pds {
            start_point_x,
            start_point_y,
            passed_target,
            current_scale,
            missed,
            explosion_anim,
        } => ProjectileInfo::Pds {
            start_point_x,
            start_point_y,
            passed_target,
            current_scale,
            missed,
            explosion_anim: anim_to_domain(explosion_anim),
        },
    };
    let d = b.damage;
    Projectile::Valid(Box::new(ProjectileState {
        current_position_x: b.current_position_x,
        current_position_y: b.current_position_y,
        previous_position_x: b.previous_position_x,
        previous_position_y: b.previous_position_y,
        speed: b.speed,
        goal_position_x: b.goal_position_x,
        goal_position_y: b.goal_position_y,
        heading: b.heading,
        owner_id: b.owner_id,
        self_id: b.self_id,
        damage: Damage {
            hull_damage: d.hull_damage,
            shield_piercing: d.shield_piercing,
            fire_chance: d.fire_chance,
            breach_chance: d.breach_chance,
            ion_damage: d.ion_damage,
            system_damage: d.system_damage,
            personnel_damage: d.personnel_damage,
            hull_buster: d.hull_buster.0,
            owner_id: d.owner_id,
            self_id: d.self_id,
            lockdown: d.lockdown.0,
            crystal_shard: d.crystal_shard.0,
            stun_chance: d.stun_chance,
            stun_amount: d.stun_amount,
        },
        lifespan: b.lifespan,
        destination_space: b.destination_space,
        current_space: b.current_space,
        target_id: b.target_id,
        dead: b.dead.0,
        death_anim_id: b.death_anim_id.0,
        flight_anim_id: b.flight_anim_id.0,
        death_anim: anim_to_domain(b.death_anim),
        flight_anim: anim_to_domain(b.flight_anim),
        velocity_x: b.velocity_x,
        velocity_y: b.velocity_y,
        missed: b.missed.0,
        hit_target: b.hit_target.0,
        hit_solid_sound: b.hit_solid_sound.0,
        hit_shield_sound: b.hit_shield_sound.0,
        miss_sound: b.miss_sound.0,
        entry_angle: b.entry_angle,
        started_dying: b.started_dying.0,
        passed_target: b.passed_target.0,
        subtype: b.subtype,
        broadcast_target: b.broadcast_target.0,
        info,
    }))
}

pub fn to_wire(p: &Projectile) -> WireProjectile {
    let p = match p {
        Projectile::Invalid => {
            return WireProjectile {
                kind: 0,
                body: None,
            };
        }
        Projectile::Valid(p) => p,
    };
    let s = |v: &str| FtlString(v.to_owned());
    let info = match &p.info {
        &ProjectileInfo::LaserOrBurst { spin_angle, spin } => {
            WireProjectileInfo::Laser { spin_angle, spin }
        }
        ProjectileInfo::RockOrExplosion | ProjectileInfo::Missile => WireProjectileInfo::Empty,
        &ProjectileInfo::Bomb {
            missed,
            fuse_ticks,
            super_shield,
            super_shield_bypass,
            arrived,
        } => WireProjectileInfo::Bomb {
            missed,
            fuse_ticks,
            super_shield,
            super_shield_bypass,
            arrived: Bool32(arrived),
        },
        ProjectileInfo::Beam(x) => WireProjectileInfo::Beam(beam_to_wire(x)),
        &ProjectileInfo::Pds {
            start_point_x,
            start_point_y,
            passed_target,
            current_scale,
            missed,
            explosion_anim,
        } => WireProjectileInfo::Pds {
            start_point_x,
            start_point_y,
            passed_target,
            current_scale,
            missed,
            explosion_anim: anim_to_wire(explosion_anim),
        },
    };
    let d = &p.damage;
    WireProjectile {
        kind: p.info.flag(),
        body: Some(WireProjectileBody {
            current_position_x: p.current_position_x,
            current_position_y: p.current_position_y,
            previous_position_x: p.previous_position_x,
            previous_position_y: p.previous_position_y,
            speed: p.speed,
            goal_position_x: p.goal_position_x,
            goal_position_y: p.goal_position_y,
            heading: p.heading,
            owner_id: p.owner_id,
            self_id: p.self_id,
            damage: WireDamage {
                hull_damage: d.hull_damage,
                shield_piercing: d.shield_piercing,
                fire_chance: d.fire_chance,
                breach_chance: d.breach_chance,
                ion_damage: d.ion_damage,
                system_damage: d.system_damage,
                personnel_damage: d.personnel_damage,
                hull_buster: Bool32(d.hull_buster),
                owner_id: d.owner_id,
                self_id: d.self_id,
                lockdown: Bool32(d.lockdown),
                crystal_shard: Bool32(d.crystal_shard),
                stun_chance: d.stun_chance,
                stun_amount: d.stun_amount,
            },
            lifespan: p.lifespan,
            destination_space: p.destination_space,
            current_space: p.current_space,
            target_id: p.target_id,
            dead: Bool32(p.dead),
            death_anim_id: s(&p.death_anim_id),
            flight_anim_id: s(&p.flight_anim_id),
            death_anim: anim_to_wire(p.death_anim),
            flight_anim: anim_to_wire(p.flight_anim),
            velocity_x: p.velocity_x,
            velocity_y: p.velocity_y,
            missed: Bool32(p.missed),
            hit_target: Bool32(p.hit_target),
            hit_solid_sound: s(&p.hit_solid_sound),
            hit_shield_sound: s(&p.hit_shield_sound),
            miss_sound: s(&p.miss_sound),
            entry_angle: p.entry_angle,
            started_dying: Bool32(p.started_dying),
            passed_target: Bool32(p.passed_target),
            subtype: p.subtype,
            broadcast_target: Bool32(p.broadcast_target),
            info,
        }),
    }
}

fn beam_to_domain(b: WireBeamInfo) -> BeamInfo {
    BeamInfo {
        emission_end_x: b.emission_end_x,
        emission_end_y: b.emission_end_y,
        strafe_source_x: b.strafe_source_x,
        strafe_source_y: b.strafe_source_y,
        strafe_end_x: b.strafe_end_x,
        strafe_end_y: b.strafe_end_y,
        final_end_x: b.final_end_x,
        final_end_y: b.final_end_y,
        swath_end_x: b.swath_end_x,
        swath_end_y: b.swath_end_y,
        swath_start_x: b.swath_start_x,
        swath_start_y: b.swath_start_y,
        lifespan: b.lifespan,
        swath_length: b.swath_length,
        unknown_delta: b.unknown_delta,
        last_collision_x: b.last_collision_x,
        last_collision_y: b.last_collision_y,
        animation_timer: b.animation_timer,
        last_damage: b.last_damage,
        emission_angle: b.emission_angle,
        timer: b.timer,
        pierced_shield: b.pierced_shield.0,
        from_drone_pod: b.from_drone_pod.0,
        damage_super_shield: b.damage_super_shield.0,
        moving_target_id: b.moving_target_id,
    }
}

fn beam_to_wire(b: &BeamInfo) -> WireBeamInfo {
    WireBeamInfo {
        emission_end_x: b.emission_end_x,
        emission_end_y: b.emission_end_y,
        strafe_source_x: b.strafe_source_x,
        strafe_source_y: b.strafe_source_y,
        strafe_end_x: b.strafe_end_x,
        strafe_end_y: b.strafe_end_y,
        final_end_x: b.final_end_x,
        final_end_y: b.final_end_y,
        swath_end_x: b.swath_end_x,
        swath_end_y: b.swath_end_y,
        swath_start_x: b.swath_start_x,
        swath_start_y: b.swath_start_y,
        lifespan: b.lifespan,
        swath_length: b.swath_length,
        unknown_delta: b.unknown_delta,
        last_collision_x: b.last_collision_x,
        last_collision_y: b.last_collision_y,
        animation_timer: b.animation_timer,
        last_damage: b.last_damage,
        emission_angle: b.emission_angle,
        timer: b.timer,
        pierced_shield: Bool32(b.pierced_shield),
        from_drone_pod: Bool32(b.from_drone_pod),
        damage_super_shield: Bool32(b.damage_super_shield),
        moving_target_id: b.moving_target_id,
    }
}
