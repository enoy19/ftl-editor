//! Reactor and systems: wire to domain and back.

use ftl_core::{
    BatteryInfo, CloakingInfo, ClonebayInfo, DecodeError, FileFormat, ShieldsInfo, System,
    SystemKind, SystemState,
};

use crate::wire::{
    Bool32, WireBatteryInfo, WireCloakingInfo, WireClonebayInfo, WireShieldsInfo, WireSystem,
    WireSystemAdvanced, WireSystemState,
};

pub fn systems_to_domain(w: Vec<WireSystem>, kinds: &[usize]) -> Result<Vec<System>, DecodeError> {
    w.into_iter()
        .enumerate()
        .map(|(i, s)| {
            let kind = kinds
                .get(i)
                .and_then(|&k| SystemKind::ALL.get(k))
                .copied()
                .ok_or_else(|| DecodeError {
                    offset: 0,
                    field: "systems",
                    problem: format!("system entry {i} has no known system kind"),
                })?;
            Ok(System {
                kind,
                installed: s.state.map(|st| state_to_domain(s.capacity, st)),
            })
        })
        .collect()
}

fn state_to_domain(capacity: i32, s: WireSystemState) -> SystemState {
    let a = s.advanced.unwrap_or(WireSystemAdvanced {
        battery_power: 0,
        hack_level: 0,
        hacked: Bool32(false),
        temporary_capacity_cap: 0,
        temporary_capacity_loss: 0,
        temporary_capacity_divisor: 0,
    });
    SystemState {
        capacity,
        power: s.power,
        damaged_bars: s.damaged_bars,
        ionized_bars: s.ionized_bars,
        deionization_ticks: s.deionization_ticks,
        repair_progress: s.repair_progress,
        damage_progress: s.damage_progress,
        battery_power: a.battery_power,
        hack_level: a.hack_level,
        hacked: a.hacked.0,
        temporary_capacity_cap: a.temporary_capacity_cap,
        temporary_capacity_loss: a.temporary_capacity_loss,
        temporary_capacity_divisor: a.temporary_capacity_divisor,
    }
}

pub fn systems_to_wire(systems: &[System], f: FileFormat) -> Vec<WireSystem> {
    systems
        .iter()
        .map(|s| match &s.installed {
            None => WireSystem {
                capacity: 0,
                state: None,
            },
            Some(st) => WireSystem {
                capacity: st.capacity,
                state: Some(WireSystemState {
                    power: st.power,
                    damaged_bars: st.damaged_bars,
                    ionized_bars: st.ionized_bars,
                    deionization_ticks: st.deionization_ticks,
                    repair_progress: st.repair_progress,
                    damage_progress: st.damage_progress,
                    advanced: (f >= 7).then_some(WireSystemAdvanced {
                        battery_power: st.battery_power,
                        hack_level: st.hack_level,
                        hacked: Bool32(st.hacked),
                        temporary_capacity_cap: st.temporary_capacity_cap,
                        temporary_capacity_loss: st.temporary_capacity_loss,
                        temporary_capacity_divisor: st.temporary_capacity_divisor,
                    }),
                }),
            },
        })
        .collect()
}

pub fn clonebay_to_domain(w: WireClonebayInfo) -> ClonebayInfo {
    ClonebayInfo {
        build_ticks: w.build_ticks,
        build_ticks_goal: w.build_ticks_goal,
        doom_ticks: w.doom_ticks,
    }
}

pub fn clonebay_to_wire(c: &ClonebayInfo) -> WireClonebayInfo {
    WireClonebayInfo {
        build_ticks: c.build_ticks,
        build_ticks_goal: c.build_ticks_goal,
        doom_ticks: c.doom_ticks,
    }
}

pub fn battery_to_domain(w: WireBatteryInfo) -> BatteryInfo {
    BatteryInfo {
        active: w.active.0,
        used_battery: w.used_battery,
        discharge_ticks: w.discharge_ticks,
    }
}

pub fn battery_to_wire(b: &BatteryInfo) -> WireBatteryInfo {
    WireBatteryInfo {
        active: Bool32(b.active),
        used_battery: b.used_battery,
        discharge_ticks: b.discharge_ticks,
    }
}

pub fn shields_to_domain(w: WireShieldsInfo) -> ShieldsInfo {
    ShieldsInfo {
        shield_layers: w.shield_layers,
        energy_shield_layers: w.energy_shield_layers,
        energy_shield_max: w.energy_shield_max,
        shield_recharge_ticks: w.shield_recharge_ticks,
        drop_anim_on: w.drop_anim_on.0,
        drop_anim_ticks: w.drop_anim_ticks,
        raise_anim_on: w.raise_anim_on.0,
        raise_anim_ticks: w.raise_anim_ticks,
        energy_shield_anim_on: w.energy_shield_anim_on.0,
        energy_shield_anim_ticks: w.energy_shield_anim_ticks,
        shields_down_point_x: w.shields_down_point_x,
        shields_down_point_y: w.shields_down_point_y,
    }
}

pub fn shields_to_wire(s: &ShieldsInfo) -> WireShieldsInfo {
    WireShieldsInfo {
        shield_layers: s.shield_layers,
        energy_shield_layers: s.energy_shield_layers,
        energy_shield_max: s.energy_shield_max,
        shield_recharge_ticks: s.shield_recharge_ticks,
        drop_anim_on: Bool32(s.drop_anim_on),
        drop_anim_ticks: s.drop_anim_ticks,
        raise_anim_on: Bool32(s.raise_anim_on),
        raise_anim_ticks: s.raise_anim_ticks,
        energy_shield_anim_on: Bool32(s.energy_shield_anim_on),
        energy_shield_anim_ticks: s.energy_shield_anim_ticks,
        shields_down_point_x: s.shields_down_point_x,
        shields_down_point_y: s.shields_down_point_y,
    }
}

pub fn cloaking_to_domain(w: WireCloakingInfo) -> CloakingInfo {
    CloakingInfo {
        turned_on: w.turned_on,
        timer_running: w.timer_running,
        cloak_ticks_goal: w.cloak_ticks_goal,
        cloak_ticks: w.cloak_ticks,
    }
}

pub fn cloaking_to_wire(c: &CloakingInfo) -> WireCloakingInfo {
    WireCloakingInfo {
        turned_on: c.turned_on,
        timer_running: c.timer_running,
        cloak_ticks_goal: c.cloak_ticks_goal,
        cloak_ticks: c.cloak_ticks,
    }
}
