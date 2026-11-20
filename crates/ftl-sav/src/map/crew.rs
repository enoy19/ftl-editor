//! Crew members: wire to domain and back.

use ftl_core::{Anim, CRYSTAL_RACE, CrewMember, FileFormat, Skill, SkillKind, Skills};

use crate::wire::{
    Bool32, FtlString, WireAnim, WireCrewMember, WireCrewPlacement, WireCrewStatus,
    WireCrystalLockdown,
};

pub fn to_domain(w: WireCrewMember) -> CrewMember {
    let p = w.placement.unwrap_or(WireCrewPlacement {
        clone_ready: 0,
        death_order: 0,
        sprite_tints: vec![],
        mind_controlled: Bool32(false),
        saved_room_square: 0,
        saved_room_id: 0,
    });
    let st = w.status;
    let masteries = st
        .as_ref()
        .and_then(|s| s.masteries)
        .map(|m| m.map(|b| b.0))
        .unwrap_or_default();
    let skill = |i: usize| Skill {
        points: w.skills[i],
        mastery_one: masteries[2 * i],
        mastery_two: masteries[2 * i + 1],
    };
    let skills = Skills {
        pilot: skill(0),
        engine: skill(1),
        shield: skill(2),
        weapon: skill(3),
        repair: skill(4),
        combat: skill(5),
    };
    let lockdown = st.as_ref().and_then(|s| s.lockdown.clone());
    CrewMember {
        name: w.name.0,
        race: w.race.0,
        enemy_boarding_drone: w.enemy_boarding_drone.0,
        health: w.health,
        sprite_x: w.sprite_x,
        sprite_y: w.sprite_y,
        room_id: w.room_id,
        room_square: w.room_square,
        player_controlled: w.player_controlled.0,
        clone_ready: p.clone_ready,
        death_order: p.death_order,
        sprite_tints: p.sprite_tints,
        mind_controlled: p.mind_controlled.0,
        saved_room_square: p.saved_room_square,
        saved_room_id: p.saved_room_id,
        skills,
        male: w.male.0,
        repairs: w.repairs,
        combat_kills: w.combat_kills,
        piloted_evasions: w.piloted_evasions,
        jumps_survived: w.jumps_survived,
        skill_masteries_earned: w.skill_masteries_earned,
        stun_ticks: st.as_ref().map_or(0, |s| s.stun_ticks),
        health_boost: st.as_ref().map_or(0, |s| s.health_boost),
        clonebay_priority: st.as_ref().map_or(0, |s| s.clonebay_priority),
        damage_boost: st.as_ref().map_or(0, |s| s.damage_boost),
        clone_dying: st.as_ref().map_or(0, |s| s.clone_dying),
        universal_death_count: st.as_ref().map_or(0, |s| s.universal_death_count),
        teleporting: st.as_ref().is_some_and(|s| s.teleporting.0),
        teleport_anim: st
            .as_ref()
            .map(|s| anim_to_domain(s.teleport_anim))
            .unwrap_or_default(),
        teleport_anim_reverse: st.as_ref().is_some_and(|s| s.teleport_anim_reverse.0),
        lockdown_recharge_ticks: lockdown.as_ref().map_or(0, |l| l.recharge_ticks),
        lockdown_recharge_ticks_goal: lockdown.as_ref().map_or(0, |l| l.recharge_ticks_goal),
        cloned: lockdown.as_ref().map_or(0, |l| l.cloned),
        removed: false,
    }
}

pub fn to_wire(c: &CrewMember, f: FileFormat) -> WireCrewMember {
    let s = |v: &str| FtlString(v.to_owned());
    let skills = SkillKind::ALL.map(|k| *c.skills.get(k));
    let mut masteries = [Bool32(false); 12];
    for (i, sk) in skills.iter().enumerate() {
        masteries[2 * i] = Bool32(sk.mastery_one);
        masteries[2 * i + 1] = Bool32(sk.mastery_two);
    }
    WireCrewMember {
        name: s(&c.name),
        race: s(&c.race),
        enemy_boarding_drone: Bool32(c.enemy_boarding_drone),
        health: c.health,
        sprite_x: c.sprite_x,
        sprite_y: c.sprite_y,
        room_id: c.room_id,
        room_square: c.room_square,
        player_controlled: Bool32(c.player_controlled),
        placement: (f >= 7).then(|| WireCrewPlacement {
            clone_ready: c.clone_ready,
            death_order: c.death_order,
            sprite_tints: c.sprite_tints.clone(),
            mind_controlled: Bool32(c.mind_controlled),
            saved_room_square: c.saved_room_square,
            saved_room_id: c.saved_room_id,
        }),
        skills: skills.map(|sk| sk.points),
        male: Bool32(c.male),
        repairs: c.repairs,
        combat_kills: c.combat_kills,
        piloted_evasions: c.piloted_evasions,
        jumps_survived: c.jumps_survived,
        skill_masteries_earned: c.skill_masteries_earned,
        status: (f >= 7).then(|| WireCrewStatus {
            stun_ticks: c.stun_ticks,
            health_boost: c.health_boost,
            clonebay_priority: c.clonebay_priority,
            damage_boost: c.damage_boost,
            clone_dying: c.clone_dying,
            universal_death_count: c.universal_death_count,
            masteries: (f >= 8).then_some(masteries),
            teleporting: Bool32(c.teleporting),
            teleport_anim: anim_to_wire(c.teleport_anim),
            teleport_anim_reverse: Bool32(c.teleport_anim_reverse),
            lockdown: (c.race == CRYSTAL_RACE).then_some(WireCrystalLockdown {
                recharge_ticks: c.lockdown_recharge_ticks,
                recharge_ticks_goal: c.lockdown_recharge_ticks_goal,
                cloned: c.cloned,
            }),
        }),
    }
}

pub fn anim_to_domain(a: WireAnim) -> Anim {
    Anim {
        playing: a.playing.0,
        looping: a.looping.0,
        current_frame: a.current_frame,
        progress_ticks: a.progress_ticks,
        scale: a.scale,
        x: a.x,
        y: a.y,
    }
}

pub fn anim_to_wire(a: Anim) -> WireAnim {
    WireAnim {
        playing: Bool32(a.playing),
        looping: Bool32(a.looping),
        current_frame: a.current_frame,
        progress_ticks: a.progress_ticks,
        scale: a.scale,
        x: a.x,
        y: a.y,
    }
}
