//! Rooms to augments: wire to domain and back.

use ftl_core::{
    Breach, Door, Drone, FileFormat, LockdownCrystal, Room, RoomShape, ShipBody, ShipShape, Square,
    Station, StationDirection, Weapon,
};

use crate::wire::{
    Bool32, FtlString, WireBreach, WireCrystals, WireDoor, WireDrone, WireLockdownCrystal,
    WireRoom, WireShipBody, WireSquare, WireStation, WireWeapon,
};

pub fn to_domain(w: WireShipBody, shape: &ShipShape) -> ShipBody {
    ShipBody {
        layout_id: shape.layout_id.clone(),
        rooms: w
            .rooms
            .into_iter()
            .zip(&shape.rooms)
            .map(|(r, s)| room_to_domain(r, s))
            .collect(),
        breaches: w
            .breaches
            .into_iter()
            .map(|b| Breach {
                x: b.x,
                y: b.y,
                health: b.health,
            })
            .collect(),
        doors: w.doors.into_iter().map(door_to_domain).collect(),
        cloak_anim_ticks: w.cloak_anim_ticks.unwrap_or(0),
        lockdown_crystals: w
            .lockdown_crystals
            .map(|c| c.crystals.into_iter().map(crystal_to_domain).collect())
            .unwrap_or_default(),
        weapons: w
            .weapons
            .into_iter()
            .map(|x| Weapon {
                id: x.id.0,
                armed: x.armed.0,
                cooldown_ticks: x.cooldown_ticks.unwrap_or(0),
            })
            .collect(),
        drones: w
            .drones
            .into_iter()
            .map(|d| Drone {
                id: d.id.0,
                armed: d.armed.0,
                deployed: d.deployed.0,
                body_x: d.body_x,
                body_y: d.body_y,
                body_room_id: d.body_room_id,
                body_room_square: d.body_room_square,
                health: d.health,
            })
            .collect(),
        augments: w.augments.into_iter().map(|a| a.0).collect(),
    }
}

pub fn to_wire(b: &ShipBody, f: FileFormat) -> WireShipBody {
    let s = |v: &str| FtlString(v.to_owned());
    WireShipBody {
        rooms: b.rooms.iter().map(|r| room_to_wire(r, f)).collect(),
        breaches: b
            .breaches
            .iter()
            .map(|x| WireBreach {
                x: x.x,
                y: x.y,
                health: x.health,
            })
            .collect(),
        doors: b.doors.iter().map(|d| door_to_wire(d, f)).collect(),
        cloak_anim_ticks: (f >= 7).then_some(b.cloak_anim_ticks),
        lockdown_crystals: (f >= 8).then(|| WireCrystals {
            crystals: b.lockdown_crystals.iter().map(crystal_to_wire).collect(),
        }),
        weapons: b
            .weapons
            .iter()
            .map(|x| WireWeapon {
                id: s(&x.id),
                armed: Bool32(x.armed),
                cooldown_ticks: (f == 2).then_some(x.cooldown_ticks),
            })
            .collect(),
        drones: b
            .drones
            .iter()
            .map(|d| WireDrone {
                id: s(&d.id),
                armed: Bool32(d.armed),
                deployed: Bool32(d.deployed),
                body_x: d.body_x,
                body_y: d.body_y,
                body_room_id: d.body_room_id,
                body_room_square: d.body_room_square,
                health: d.health,
            })
            .collect(),
        augments: b.augments.iter().map(|a| s(a)).collect(),
    }
}

/// The file stores squares column by column (index `h * v_count + v`);
/// the domain holds them by square id (index `v * h_count + h`).
fn room_to_domain(r: WireRoom, shape: &RoomShape) -> Room {
    let (h_count, v_count) = (shape.squares_h, shape.squares_v);
    let (hn, vn) = (h_count as usize, v_count as usize);
    let squares = (0..vn)
        .flat_map(|v| (0..hn).map(move |h| (h, v)))
        .map(|(h, v)| {
            let q = r.squares[h * vn + v];
            Square {
                fire_health: q.fire_health,
                ignition_progress: q.ignition_progress,
                extinguishment_progress: q.extinguishment_progress,
            }
        })
        .collect();
    Room {
        x: shape.x,
        y: shape.y,
        squares_h: h_count,
        squares_v: v_count,
        oxygen: r.oxygen,
        squares,
        station: r.station.map(|s| Station {
            square: s.square,
            // The wire asserts 0..=4.
            direction: StationDirection::ALL[s.direction as usize],
        }),
    }
}

fn room_to_wire(r: &Room, f: FileFormat) -> WireRoom {
    let (hn, vn) = (r.squares_h as usize, r.squares_v as usize);
    let squares = (0..hn)
        .flat_map(|h| (0..vn).map(move |v| (h, v)))
        .map(|(h, v)| {
            let q = r.squares[v * hn + h];
            WireSquare {
                fire_health: q.fire_health,
                ignition_progress: q.ignition_progress,
                extinguishment_progress: q.extinguishment_progress,
            }
        })
        .collect();
    WireRoom {
        oxygen: r.oxygen,
        squares,
        station: if f >= 7 {
            r.station.map(|s| WireStation {
                square: s.square,
                direction: StationDirection::ALL
                    .iter()
                    .position(|d| *d == s.direction)
                    .unwrap_or(4) as i32,
            })
        } else {
            None
        },
    }
}

fn door_to_domain(d: WireDoor) -> Door {
    let [current_max_health, health, nominal_health] = d.health.unwrap_or_default();
    let [door_level, hacked] = d.level_hacked.unwrap_or_default();
    Door {
        current_max_health,
        health,
        nominal_health,
        open: d.open.0,
        walking_through: d.walking_through.0,
        door_level,
        hacked,
    }
}

fn door_to_wire(d: &Door, f: FileFormat) -> WireDoor {
    WireDoor {
        health: (f >= 7).then_some([d.current_max_health, d.health, d.nominal_health]),
        open: Bool32(d.open),
        walking_through: Bool32(d.walking_through),
        level_hacked: (f >= 7).then_some([d.door_level, d.hacked]),
    }
}

fn crystal_to_domain(c: WireLockdownCrystal) -> LockdownCrystal {
    LockdownCrystal {
        current_position_x: c.current_position_x,
        current_position_y: c.current_position_y,
        speed: c.speed,
        goal_position_x: c.goal_position_x,
        goal_position_y: c.goal_position_y,
        arrived: c.arrived.0,
        done: c.done.0,
        lifetime: c.lifetime,
        super_freeze: c.super_freeze.0,
        locking_room: c.locking_room,
        anim_direction: c.anim_direction,
        shard_progress: c.shard_progress,
    }
}

fn crystal_to_wire(c: &LockdownCrystal) -> WireLockdownCrystal {
    WireLockdownCrystal {
        current_position_x: c.current_position_x,
        current_position_y: c.current_position_y,
        speed: c.speed,
        goal_position_x: c.goal_position_x,
        goal_position_y: c.goal_position_y,
        arrived: Bool32(c.arrived),
        done: Bool32(c.done),
        lifetime: c.lifetime,
        super_freeze: Bool32(c.super_freeze),
        locking_room: c.locking_room,
        anim_direction: c.anim_direction,
        shard_progress: c.shard_progress,
    }
}
