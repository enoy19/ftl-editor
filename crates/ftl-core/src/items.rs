//! Weapons and drones in the slots of a ship and the cargo hold:
//! what a new one looks like in the file, and the edits that add, remove,
//! arm and disarm them (`weapons.<ID>`, `weapons.<ID>.armed`,
//! `drones.<ID>`, `drones.<ID>.armed`, `cargo.<ID>`).
//!
//! Evidence (FTL 1.6.13, macOS binary, symbols read with Ghidra 12.1.4 on
//! 2026-11-19; field offsets from Hyperspace `FTLGameStripped.h`):
//!
//! - `ShipManager::ImportShip` reads each weapon and calls
//!   `WeaponSystem::AddWeapon(bp, -1)` only when the weapons system is
//!   installed, then `PowerWeapon` for an armed one; each drone goes through
//!   `CreateCrewDrone` / `CreateSpaceDrone` and `DroneSystem::AddDrone(d, -1)`
//!   and `PowerDrone` when armed. `ShipManager::AddWeapon` and `AddDrone`
//!   return nothing without the system. A slot is the next free one, so the
//!   file order is the slot order: a new item takes the first slot a removal
//!   of the same patch freed, else it goes to the end
//!   ([`crate::slot_order`]).
//! - `ShipManager::ImportBattleState` reads one `deployed` and one `armed` int
//!   per drone and its pod (`SpaceDrone::LoadState`), then a weapon module
//!   count and that many `ProjectileFactory::LoadState`, each a plain
//!   overwrite of the fields the constructor set. A new item therefore
//!   carries the state of a freshly built one:
//!   - `ProjectileFactory::ProjectileFactory`: cooldown 0, cooldown goal the
//!     blueprint cooldown, subcooldown 0 of 0, boost 0, charge 0, no targets,
//!     autofire and fire-when-ready off, target -1, last projectile -1, no
//!     queued projectiles; its `WeaponAnimation` starts playing for every
//!     weapon type but `MISSILES` (`StringToWeapon`: LASER 0, MISSILES 1, BEAM
//!     2, BOMB 3, BURST 4) and has charged frame -1, not firing.
//!   - `SpaceDrone::SpaceDrone(iShipId, ...)`: the owner's space (its
//!     `iShipId`: 0 for the player ship, 1 for the nearby ship) and no
//!     destination (-1), positions at `-FLT_MAX` (written as `i32::MIN`), goal
//!     at -1, build-up -1, stationary 0, the blueprint cooldown, a random
//!     orbit angle (0 here), turret 0, desired aiming angle -1, hops
//!     `i32::MAX`, every flag off, no delta. `CombatDrone` (also hull repair)
//!     adds waypoint 0,0 and a transit of 1 and leaves the exhaust angle and
//!     the previous exhaust angle unset (0 here); `SuperShieldDrone` adds a
//!     glow of 0;
//!     `BoarderPodDrone` adds two flags off, which make `LoadState` skip the
//!     body fields.
//!   - `FileHelper::writeFloat` stores `(int)(f * 1000)`.
//!   - The second fixture, `continue-1.6.13-format11-sector3-enemy-drone.sav`,
//!     holds a `COMBAT_BEAM` FTL deployed: every field the constructor sets
//!     and flight leaves alone (mourning 0, point target and target speed
//!     `i32::MIN`, build-up -1000, stationary 0, the blueprint cooldown
//!     1000, hops `i32::MAX`, every flag off, the idle death animation)
//!     equals [`new_pod`] (`tests/roundtrip.rs`). The other fields
//!     (positions, goal, angles, waypoint, transit) had moved in flight, so
//!     no FTL-written file shows them for a fresh drone yet; they stay
//!     inferred from the constructor.
//! - The animations a constructor takes from `AnimationControl` are not
//!   started; the file stores them as FTL wrote the idle weapon and the
//!   undeployed drones of the first `continue-1.6.13-format11.sav`
//!   (replaced on 2026-11-19 by a save without drones; playing off, frame
//!   0, scale 1000; the charge animation at -1000, -1000). The second
//!   fixture's deployed drone has the same idle death animation.
//! - A drone in the drone list stores its body as -1 and health 1 while it
//!   has none: every space drone of that first fixture and the deployed
//!   space drone of the second (Java's `DroneState` defaults). Its second
//!   flag is `Drone::GetDeployed` (Java: `playerControlled`): 0 for a new
//!   or newly armed drone, 1 for the deployed one of the second fixture.
//!   A crew drone (repair, anti-personnel) restarts its crew member when
//!   it is deployed (`CrewDrone::SetDeployed`).
//! - `Equipment::OnInit` builds four cargo boxes (and one over-capacity box);
//!   `Equipment::AddToCargo` puts a weapon or a drone of the file into them.
//! - A powered system draws exactly the bars of its armed weapons or drones
//!   (fixture: weapons power 3 = Artemis 1 + Burst Laser II 2 armed; the
//!   first fixture: drones power 0 with none armed; the second fixture's
//!   nearby ship: drones power 2 = `COMBAT_BEAM` 2 armed), so arming takes
//!   a bar of system capacity and reactor per power, and disarming gives
//!   them back.
//!
//! `CreateSpaceDrone` builds no hacking or beam drone (`StringToDrone` knows
//! neither), so those kinds never enter a drone slot. Arming a crew drone
//! needs a place on the ship for its body, which the editor does not pick:
//! FTL powers it there.

use crate::edit::{EditError, Field};
use crate::layout::{DroneFacts, ShipLayouts, WeaponFacts};
use crate::model::{
    Anim, Drone, DroneKind, DronePod, DronePodInfo, ExtendedDroneInfo, SavedGame, Ship, ShipId,
    SystemKind, SystemState, Weapon, WeaponModule,
};

/// The cargo boxes of FTL's equipment screen.
pub const MAX_CARGO: usize = 4;

/// The first format with the extended ship info (weapon modules and drone pods).
const EXTENDED_FORMAT: i32 = 7;

fn idle_anim() -> Anim {
    Anim {
        playing: false,
        looping: false,
        current_frame: 0,
        progress_ticks: 0,
        scale: 1000,
        x: 0,
        y: 0,
    }
}

/// The weapon module FTL builds for a new weapon (see the module doc).
pub fn new_weapon_module(facts: &WeaponFacts) -> WeaponModule {
    WeaponModule {
        cooldown_ticks: 0,
        cooldown_ticks_goal: facts.cooldown_ms.unwrap_or(0),
        subcooldown_ticks: 0,
        subcooldown_ticks_goal: 0,
        boost: 0,
        charge: 0,
        current_targets: vec![],
        previous_targets: vec![],
        autofire: false,
        fire_when_ready: false,
        target_id: -1,
        weapon_anim: Anim {
            playing: facts.kind != "MISSILES",
            ..idle_anim()
        },
        protract_anim_ticks: 0,
        firing: false,
        fire_shot: false,
        anim_charge: -1,
        charge_anim: Anim {
            x: -1000,
            y: -1000,
            ..idle_anim()
        },
        last_projectile_id: -1,
        pending_projectiles: vec![],
    }
}

/// Whether FTL puts a drone of this kind in a drone slot.
pub fn equippable(kind: DroneKind) -> bool {
    !matches!(kind, DroneKind::Hacking | DroneKind::Beam)
}

/// Whether the editor can arm a drone of this kind: a crew drone needs a
/// place for its body on the ship.
pub fn armable(kind: DroneKind) -> bool {
    kind.has_pod()
}

/// The drone pod a freshly built space drone of `kind` stores on the ship
/// `owner` (see the module doc). `None` for the crew drones, which have none.
pub fn new_pod(kind: DroneKind, cooldown_ms: Option<i32>, owner: ShipId) -> Option<DronePod> {
    let info = match kind {
        DroneKind::Repair | DroneKind::Battle => return None,
        DroneKind::Combat | DroneKind::Beam | DroneKind::ShipRepair => DronePodInfo::ZigZag {
            last_waypoint_x: 0,
            last_waypoint_y: 0,
            transit_ticks: 1,
            exhaust_angle: 0,
            previous_exhaust_angle: 0,
        },
        DroneKind::Defense => DronePodInfo::Empty,
        DroneKind::Shield => DronePodInfo::Shield { glow_animation: 0 },
        DroneKind::Boarder => DronePodInfo::Boarder {
            died_in_space: 0,
            delivered_drone: 0,
            starting_position_x: 0,
            starting_position_y: 0,
            body_health: 0,
            body_x: -1,
            body_y: -1,
            body_room_id: -1,
            body_room_square: -1,
        },
        DroneKind::Hacking => DronePodInfo::Hacking {
            attach_position_x: 0,
            attach_position_y: 0,
            arrived: 0,
            finished_setup: 0,
            landing_anim: idle_anim(),
            extension_anim: idle_anim(),
        },
    };
    let far = i32::MIN;
    Some(DronePod {
        kind,
        mourning_ticks: 0,
        current_space: owner.number(),
        destination_space: -1,
        current_position_x: far,
        current_position_y: far,
        previous_position_x: far,
        previous_position_y: far,
        goal_position_x: -1000,
        goal_position_y: -1000,
        point_target_x: far,
        point_target_y: far,
        next_target_x: far,
        next_target_y: far,
        target_speed_x: far,
        target_speed_y: far,
        buildup_ticks: -1000,
        stationary_ticks: 0,
        cooldown_ticks: cooldown_ms.unwrap_or(0),
        orbit_angle: 0,
        turret_angle: 0,
        last_aiming_angle: 0,
        hops_to_live: i32::MAX,
        disrupted: 0,
        powered_last_frame: 0,
        overload_ticks: 0,
        desired_aiming_angle: -1000,
        deployed_last_frame: 0,
        delta_position_x: 0,
        delta_position_y: 0,
        death_anim: idle_anim(),
        info,
    })
}

/// A new drone in a slot of the ship `owner`: unarmed, without a body, with
/// its extended info.
pub fn new_drone(id: &str, facts: &DroneFacts, owner: ShipId) -> (Drone, ExtendedDroneInfo) {
    (
        Drone {
            id: id.to_owned(),
            armed: false,
            deployed: false,
            body_x: -1,
            body_y: -1,
            body_room_id: -1,
            body_room_square: -1,
            health: 1,
        },
        ExtendedDroneInfo {
            deployed: false,
            armed: false,
            pod: new_pod(facts.kind, facts.cooldown_ms, owner),
        },
    )
}

/// Weapons or drones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Slots {
    Weapons,
    Drones,
}

impl Slots {
    fn system(self) -> SystemKind {
        match self {
            Slots::Weapons => SystemKind::Weapons,
            Slots::Drones => SystemKind::Drones,
        }
    }

    fn ids(self, ship: &Ship) -> Vec<&str> {
        let Some(body) = ship.body.as_ref() else {
            return vec![];
        };
        match self {
            Slots::Weapons => body.weapons.iter().map(|w| w.id.as_str()).collect(),
            Slots::Drones => body.drones.iter().map(|d| d.id.as_str()).collect(),
        }
    }

    fn armed(self, ship: &Ship, i: usize) -> bool {
        let body = ship.body.as_ref().expect("checked by the caller");
        match self {
            Slots::Weapons => body.weapons[i].armed,
            Slots::Drones => body.drones[i].armed,
        }
    }
}

/// What one item of the edit is: its power and, for a drone, its kind.
struct Facts {
    power: i32,
    weapon: Option<WeaponFacts>,
    drone: Option<DroneFacts>,
}

fn facts(
    field: &Field,
    slots: Slots,
    id: &str,
    data: &dyn ShipLayouts,
) -> Result<Facts, EditError> {
    let unknown = || EditError::UnknownItem {
        field: field.clone(),
        id: id.to_owned(),
    };
    match slots {
        Slots::Weapons => {
            let w = data.weapon(id).ok_or_else(unknown)?;
            Ok(Facts {
                power: w.power,
                weapon: Some(w),
                drone: None,
            })
        }
        Slots::Drones => {
            let d = data.drone(id).ok_or_else(unknown)?;
            if !equippable(d.kind) {
                return Err(EditError::NotEquippable {
                    field: field.clone(),
                    kind: d.kind,
                });
            }
            Ok(Facts {
                power: d.power,
                weapon: None,
                drone: Some(d),
            })
        }
    }
}

/// Checks that the slots of `slots` can be edited and answers how many the
/// ship has.
fn check(
    game: &SavedGame,
    field: &Field,
    slots: Slots,
    data: &dyn ShipLayouts,
) -> Result<usize, EditError> {
    let ship = crate::edit::ship(game, field)?;
    if ship.body.is_none() {
        return Err(EditError::NotDecoded {
            field: field.clone(),
            part: "the weapon and drone list of the ship",
        });
    }
    if game.file_format >= EXTENDED_FORMAT && ship.extended.is_none() {
        return Err(EditError::NotDecoded {
            field: field.clone(),
            part: "the weapon and drone states after the nearby ship",
        });
    }
    if system(ship, slots).is_none() {
        return Err(EditError::NoSystem {
            field: field.clone(),
            system: slots.system(),
        });
    }
    let n = data
        .slots(&ship.blueprint_id)
        .ok_or_else(|| EditError::NoGameData {
            field: field.clone(),
        })?;
    Ok(match slots {
        Slots::Weapons => n.weapons,
        Slots::Drones => n.drones,
    } as usize)
}

fn system(ship: &Ship, slots: Slots) -> Option<&SystemState> {
    ship.systems
        .iter()
        .find(|s| s.kind == slots.system())
        .and_then(|s| s.installed.as_ref())
}

fn system_mut(ship: &mut Ship, slots: Slots) -> &mut SystemState {
    ship.systems
        .iter_mut()
        .find(|s| s.kind == slots.system())
        .and_then(|s| s.installed.as_mut())
        .expect("check() found the system")
}

/// How many copies of `id` the slots hold.
pub(crate) fn count(ship: &Ship, slots: Slots, id: &str) -> i32 {
    slots.ids(ship).iter().filter(|i| **i == id).count() as i32
}

/// How many copies of `id` are armed.
pub(crate) fn armed_count(ship: &Ship, slots: Slots, id: &str) -> i32 {
    slots
        .ids(ship)
        .iter()
        .enumerate()
        .filter(|(i, x)| **x == id && slots.armed(ship, *i))
        .count() as i32
}

/// `weapons.<ID>` / `drones.<ID>`: 0 to the copies plus the free slots.
pub(crate) fn count_range(
    game: &SavedGame,
    field: &Field,
    slots: Slots,
    id: &str,
    data: &dyn ShipLayouts,
) -> Result<(i32, i32), EditError> {
    let n = check(game, field, slots, data)?;
    facts(field, slots, id, data)?;
    let ship = crate::edit::ship(game, field)?;
    let used = slots.ids(ship).len();
    Ok((0, count(ship, slots, id) + n.saturating_sub(used) as i32))
}

/// The bars a new armed item could take: the free capacity of the system
/// (level minus broken bars minus power) and the free reactor bars.
fn free_bars(ship: &Ship, slots: Slots) -> i32 {
    let s = system(ship, slots).expect("check() found the system");
    let capacity = (s.capacity - s.damaged_bars - s.power).max(0);
    let reactor = (ship.reactor_power - crate::edit::reactor_power_in_use(ship)).max(0);
    capacity.min(reactor)
}

/// Whether drone `i` is out of the ship (deployed or with a body aboard).
fn deployed(ship: &Ship, i: usize) -> bool {
    let body = ship.body.as_ref().expect("checked by the caller");
    body.drones[i].deployed
        || ship
            .extended
            .as_ref()
            .and_then(|e| e.drones.get(i))
            .is_some_and(|e| e.deployed)
}

/// `weapons.<ID>.armed` / `drones.<ID>.armed`: 0 to the armed copies plus
/// those the free bars can power (a crew drone stays as it is).
pub(crate) fn armed_range(
    game: &SavedGame,
    field: &Field,
    slots: Slots,
    id: &str,
    data: &dyn ShipLayouts,
) -> Result<(i32, i32), EditError> {
    check(game, field, slots, data)?;
    let f = facts(field, slots, id, data)?;
    let ship = crate::edit::ship(game, field)?;
    let armed = armed_count(ship, slots, id);
    let unarmed = count(ship, slots, id) - armed;
    if f.drone.is_some_and(|d| !armable(d.kind)) {
        return Ok((0, armed));
    }
    let more = if f.power <= 0 {
        unarmed
    } else {
        unarmed.min(free_bars(ship, slots) / f.power)
    };
    Ok((0, armed + more))
}

/// Arms or disarms copy `i` and moves the bars of its power.
fn set_armed(ship: &mut Ship, slots: Slots, i: usize, on: bool, power: i32) {
    let body = ship.body.as_mut().expect("checked by the caller");
    match slots {
        Slots::Weapons => {
            let w = &mut body.weapons[i];
            w.armed = on;
            // Java's WeaponState.setArmed(false) clears the format 2 cooldown.
            if !on {
                w.cooldown_ticks = 0;
            }
        }
        Slots::Drones => {
            let d = &mut body.drones[i];
            d.armed = on;
            d.deployed = false;
            if let Some(e) = ship.extended.as_mut().and_then(|e| e.drones.get_mut(i)) {
                e.armed = on;
                e.deployed = false;
            }
        }
    }
    let s = system_mut(ship, slots);
    if on {
        s.power += power;
    } else {
        s.power = (s.power - power).max(0);
        s.battery_power = s.battery_power.min(s.power);
    }
}

/// The copy of `id` an edit takes away: the last one that matches `pick`
/// and is not deployed.
fn last_copy(ship: &Ship, slots: Slots, id: &str, pick: impl Fn(usize) -> bool) -> Option<usize> {
    let ids = slots.ids(ship);
    (0..ids.len())
        .rev()
        .find(|&i| ids[i] == id && pick(i) && !(slots == Slots::Drones && deployed(ship, i)))
}

pub(crate) fn set_armed_count(
    game: &mut SavedGame,
    field: &Field,
    slots: Slots,
    id: &str,
    value: i32,
    data: &dyn ShipLayouts,
) -> Result<i32, EditError> {
    let power = facts(field, slots, id, data)?.power;
    let ship = crate::edit::ship_mut(game, field);
    let old = armed_count(ship, slots, id);
    for _ in old..value {
        let ids = slots.ids(ship);
        let i = (0..ids.len())
            .find(|&i| ids[i] == id && !slots.armed(ship, i))
            .expect("the range counted an unarmed copy");
        set_armed(ship, slots, i, true, power);
    }
    for _ in value..old {
        let i = last_copy(ship, slots, id, |i| slots.armed(ship, i)).ok_or_else(|| {
            EditError::DroneDeployed {
                field: field.clone(),
            }
        })?;
        set_armed(ship, slots, i, false, power);
    }
    Ok(old)
}

pub(crate) fn set_count(
    game: &mut SavedGame,
    field: &Field,
    slots: Slots,
    id: &str,
    value: i32,
    data: &dyn ShipLayouts,
) -> Result<i32, EditError> {
    let f = facts(field, slots, id, data)?;
    let owner = field.ship();
    let ship = crate::edit::ship_mut(game, field);
    let old = count(ship, slots, id);
    for _ in value..old {
        let i = last_copy(ship, slots, id, |_| true).ok_or_else(|| EditError::DroneDeployed {
            field: field.clone(),
        })?;
        if slots.armed(ship, i) {
            set_armed(ship, slots, i, false, f.power);
        }
        let body = ship.body.as_mut().expect("checked by the caller");
        match slots {
            Slots::Weapons => {
                body.weapons.remove(i);
                if let Some(m) = ship
                    .extended
                    .as_mut()
                    .and_then(|e| e.weapon_modules.as_mut())
                {
                    m.remove(i);
                }
            }
            Slots::Drones => {
                body.drones.remove(i);
                if let Some(e) = ship.extended.as_mut() {
                    e.drones.remove(i);
                }
            }
        }
    }
    for _ in old..value {
        let body = ship.body.as_mut().expect("checked by the caller");
        match (&f.weapon, &f.drone) {
            (Some(w), _) => {
                body.weapons.push(Weapon {
                    id: id.to_owned(),
                    armed: false,
                    cooldown_ticks: 0,
                });
                if let Some(m) = ship
                    .extended
                    .as_mut()
                    .and_then(|e| e.weapon_modules.as_mut())
                {
                    m.push(new_weapon_module(w));
                }
            }
            (None, Some(d)) => {
                let (drone, info) = new_drone(id, d, owner);
                body.drones.push(drone);
                if let Some(e) = ship.extended.as_mut() {
                    e.drones.push(info);
                }
            }
            (None, None) => unreachable!("facts() answers a weapon or a drone"),
        }
    }
    Ok(old)
}

fn cargo<'g>(game: &'g SavedGame, field: &Field) -> Result<&'g Vec<String>, EditError> {
    game.world
        .as_ref()
        .map(|w| &w.cargo)
        .ok_or_else(|| EditError::NotDecoded {
            field: field.clone(),
            part: "the cargo hold",
        })
}

/// How many copies of `id` the cargo holds.
pub(crate) fn cargo_count(game: &SavedGame, field: &Field, id: &str) -> Result<i32, EditError> {
    Ok(cargo(game, field)?.iter().filter(|c| *c == id).count() as i32)
}

/// `cargo.<ID>`: 0 to the copies plus the free cargo boxes; `id` must be a
/// weapon or a drone FTL equips.
pub(crate) fn cargo_range(
    game: &SavedGame,
    field: &Field,
    id: &str,
    data: &dyn ShipLayouts,
) -> Result<(i32, i32), EditError> {
    let list = cargo(game, field)?;
    let known = data.weapon(id).is_some() || data.drone(id).is_some_and(|d| equippable(d.kind));
    if !known {
        return Err(EditError::UnknownItem {
            field: field.clone(),
            id: id.to_owned(),
        });
    }
    let free = MAX_CARGO.saturating_sub(list.len()) as i32;
    Ok((0, cargo_count(game, field, id)? + free))
}

/// Adds copies at the end of the cargo or removes its last copies.
pub(crate) fn set_cargo_count(
    game: &mut SavedGame,
    field: &Field,
    id: &str,
    value: i32,
) -> Result<i32, EditError> {
    let old = cargo_count(game, field, id)?;
    let list = &mut game
        .world
        .as_mut()
        .expect("cargo() checked the world")
        .cargo;
    for _ in old..value {
        list.push(id.to_owned());
    }
    for _ in value..old {
        let last = list.iter().rposition(|c| c == id).expect("counted above");
        list.remove(last);
    }
    Ok(old)
}
