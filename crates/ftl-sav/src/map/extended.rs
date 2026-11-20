//! The extended ship info: wire to domain and back.

use ftl_core::{
    DecodeError, DronePod, DronePodInfo, ExtendedDroneInfo, ExtendedShipInfo, HackingInfo,
    MindInfo, Reticle, StandaloneDrone, SystemKind, WeaponModule,
};

use super::crew::{anim_to_domain, anim_to_wire};
use super::projectile;
use crate::wire::{
    Bool32, FtlString, WireDronePod, WireDronePodInfo, WireExtendedDrone, WireExtendedShipInfo,
    WireHackingInfo, WireMindInfo, WireStandaloneDrone, WireWeaponModule, WireWeaponModules,
};

pub fn to_domain(w: WireExtendedShipInfo) -> Result<ExtendedShipInfo, DecodeError> {
    Ok(ExtendedShipInfo {
        drones: w
            .drones
            .into_iter()
            .map(|d| ExtendedDroneInfo {
                deployed: d.deployed.0,
                armed: d.armed.0,
                pod: d.pod.map(pod_to_domain),
            })
            .collect(),
        hacking: w.hacking.map(|h| HackingInfo {
            // The reader asserts -1..=15.
            target_system: usize::try_from(h.target_system)
                .ok()
                .map(|i| SystemKind::ALL[i]),
            drone_powered: h.drone_powered,
            drone_pod_visible: h.drone_pod_visible.0,
            hacking: h.hacking,
            disruption_ticks: h.disruption_ticks,
            disruption_ticks_goal: h.disruption_ticks_goal,
            disrupting: h.disrupting.0,
            drone_pod: pod_to_domain(h.drone_pod),
        }),
        mind: w.mind.map(|m| MindInfo {
            mind_control_ticks: m.mind_control_ticks,
            mind_control_ticks_goal: m.mind_control_ticks_goal,
        }),
        weapon_modules: w
            .weapon_modules
            .map(|m| m.modules.into_iter().map(module_to_domain).collect()),
        artillery: w.artillery.into_iter().map(module_to_domain).collect(),
        standalone_drones: w
            .standalone_drones
            .into_iter()
            .map(|d| StandaloneDrone {
                drone_id: d.drone_id.0,
                drone_pod: pod_to_domain(d.drone_pod),
                deployed: d.deployed,
                destroyed: d.destroyed,
                armed: d.armed,
            })
            .collect(),
    })
}

pub fn to_wire(e: &ExtendedShipInfo) -> WireExtendedShipInfo {
    WireExtendedShipInfo {
        drones: e
            .drones
            .iter()
            .map(|d| WireExtendedDrone {
                deployed: Bool32(d.deployed),
                armed: Bool32(d.armed),
                pod: d.pod.as_ref().map(pod_to_wire),
            })
            .collect(),
        hacking: e.hacking.as_ref().map(|h| WireHackingInfo {
            target_system: h
                .target_system
                .and_then(|k| SystemKind::ALL.iter().position(|&x| x == k))
                .map_or(-1, |i| i as i32),
            drone_powered: h.drone_powered,
            drone_pod_visible: Bool32(h.drone_pod_visible),
            hacking: h.hacking,
            disruption_ticks: h.disruption_ticks,
            disruption_ticks_goal: h.disruption_ticks_goal,
            disrupting: Bool32(h.disrupting),
            drone_pod: pod_to_wire(&h.drone_pod),
        }),
        mind: e.mind.as_ref().map(|m| WireMindInfo {
            mind_control_ticks: m.mind_control_ticks,
            mind_control_ticks_goal: m.mind_control_ticks_goal,
        }),
        weapon_modules: e.weapon_modules.as_ref().map(|m| WireWeaponModules {
            modules: m.iter().map(module_to_wire).collect(),
        }),
        artillery: e.artillery.iter().map(module_to_wire).collect(),
        standalone_drones: e
            .standalone_drones
            .iter()
            .map(|d| WireStandaloneDrone {
                drone_id: FtlString(d.drone_id.clone()),
                drone_pod: pod_to_wire(&d.drone_pod),
                deployed: d.deployed,
                destroyed: d.destroyed,
                armed: d.armed,
            })
            .collect(),
    }
}

fn module_to_domain(m: WireWeaponModule) -> WeaponModule {
    let reticle = |[x, y]: [i32; 2]| Reticle { x, y };
    let (anim_charge, charge_anim) = m
        .charge_anim
        .map_or((0, Default::default()), |(c, a)| (c, anim_to_domain(a)));
    WeaponModule {
        cooldown_ticks: m.cooldown_ticks,
        cooldown_ticks_goal: m.cooldown_ticks_goal,
        subcooldown_ticks: m.subcooldown_ticks,
        subcooldown_ticks_goal: m.subcooldown_ticks_goal,
        boost: m.boost,
        charge: m.charge,
        current_targets: m.current_targets.into_iter().map(reticle).collect(),
        previous_targets: m.previous_targets.into_iter().map(reticle).collect(),
        autofire: m.autofire.0,
        fire_when_ready: m.fire_when_ready.0,
        target_id: m.target_id,
        weapon_anim: anim_to_domain(m.weapon_anim),
        protract_anim_ticks: m.protract_anim_ticks,
        firing: m.firing.0,
        fire_shot: m.fire_shot.0,
        anim_charge,
        charge_anim,
        last_projectile_id: m.last_projectile_id,
        pending_projectiles: m
            .pending_projectiles
            .into_iter()
            .map(projectile::to_domain)
            .collect(),
    }
}

/// The format decides whether the charge animation is written; the caller
/// passes the format to the writer, which skips it before format 9.
fn module_to_wire(m: &WeaponModule) -> WireWeaponModule {
    let reticle = |r: &Reticle| [r.x, r.y];
    WireWeaponModule {
        cooldown_ticks: m.cooldown_ticks,
        cooldown_ticks_goal: m.cooldown_ticks_goal,
        subcooldown_ticks: m.subcooldown_ticks,
        subcooldown_ticks_goal: m.subcooldown_ticks_goal,
        boost: m.boost,
        charge: m.charge,
        current_targets: m.current_targets.iter().map(reticle).collect(),
        previous_targets: m.previous_targets.iter().map(reticle).collect(),
        autofire: Bool32(m.autofire),
        fire_when_ready: Bool32(m.fire_when_ready),
        target_id: m.target_id,
        weapon_anim: anim_to_wire(m.weapon_anim),
        protract_anim_ticks: m.protract_anim_ticks,
        firing: Bool32(m.firing),
        fire_shot: Bool32(m.fire_shot),
        charge_anim: Some((m.anim_charge, anim_to_wire(m.charge_anim))),
        last_projectile_id: m.last_projectile_id,
        pending_projectiles: m
            .pending_projectiles
            .iter()
            .map(projectile::to_wire)
            .collect(),
    }
}

fn pod_to_domain(p: WireDronePod) -> DronePod {
    let info = match p.info {
        WireDronePodInfo::Boarder {
            died_in_space,
            delivered_drone,
            starting_position_x,
            starting_position_y,
            body_health,
            body_x,
            body_y,
            body_room_id,
            body_room_square,
        } => DronePodInfo::Boarder {
            died_in_space,
            delivered_drone,
            starting_position_x,
            starting_position_y,
            body_health,
            body_x,
            body_y,
            body_room_id,
            body_room_square,
        },
        WireDronePodInfo::Hacking {
            attach_position_x,
            attach_position_y,
            arrived,
            finished_setup,
            landing_anim,
            extension_anim,
        } => DronePodInfo::Hacking {
            attach_position_x,
            attach_position_y,
            arrived,
            finished_setup,
            landing_anim: anim_to_domain(landing_anim),
            extension_anim: anim_to_domain(extension_anim),
        },
        WireDronePodInfo::ZigZag {
            last_waypoint_x,
            last_waypoint_y,
            transit_ticks,
            exhaust_angle,
            previous_exhaust_angle,
        } => DronePodInfo::ZigZag {
            last_waypoint_x,
            last_waypoint_y,
            transit_ticks,
            exhaust_angle,
            previous_exhaust_angle,
        },
        WireDronePodInfo::Empty => DronePodInfo::Empty,
        WireDronePodInfo::Shield { glow_animation } => DronePodInfo::Shield { glow_animation },
    };
    DronePod {
        kind: p.kind,
        mourning_ticks: p.mourning_ticks,
        current_space: p.current_space,
        destination_space: p.destination_space,
        current_position_x: p.current_position_x,
        current_position_y: p.current_position_y,
        previous_position_x: p.previous_position_x,
        previous_position_y: p.previous_position_y,
        goal_position_x: p.goal_position_x,
        goal_position_y: p.goal_position_y,
        point_target_x: p.point_target_x,
        point_target_y: p.point_target_y,
        next_target_x: p.next_target_x,
        next_target_y: p.next_target_y,
        target_speed_x: p.target_speed_x,
        target_speed_y: p.target_speed_y,
        buildup_ticks: p.buildup_ticks,
        stationary_ticks: p.stationary_ticks,
        cooldown_ticks: p.cooldown_ticks,
        orbit_angle: p.orbit_angle,
        turret_angle: p.turret_angle,
        last_aiming_angle: p.last_aiming_angle,
        hops_to_live: p.hops_to_live,
        disrupted: p.disrupted,
        powered_last_frame: p.powered_last_frame,
        overload_ticks: p.overload_ticks,
        desired_aiming_angle: p.desired_aiming_angle,
        deployed_last_frame: p.deployed_last_frame,
        delta_position_x: p.delta_position_x,
        delta_position_y: p.delta_position_y,
        death_anim: anim_to_domain(p.death_anim),
        info,
    }
}

fn pod_to_wire(p: &DronePod) -> WireDronePod {
    let info = match &p.info {
        &DronePodInfo::Boarder {
            died_in_space,
            delivered_drone,
            starting_position_x,
            starting_position_y,
            body_health,
            body_x,
            body_y,
            body_room_id,
            body_room_square,
        } => WireDronePodInfo::Boarder {
            died_in_space,
            delivered_drone,
            starting_position_x,
            starting_position_y,
            body_health,
            body_x,
            body_y,
            body_room_id,
            body_room_square,
        },
        &DronePodInfo::Hacking {
            attach_position_x,
            attach_position_y,
            arrived,
            finished_setup,
            landing_anim,
            extension_anim,
        } => WireDronePodInfo::Hacking {
            attach_position_x,
            attach_position_y,
            arrived,
            finished_setup,
            landing_anim: anim_to_wire(landing_anim),
            extension_anim: anim_to_wire(extension_anim),
        },
        &DronePodInfo::ZigZag {
            last_waypoint_x,
            last_waypoint_y,
            transit_ticks,
            exhaust_angle,
            previous_exhaust_angle,
        } => WireDronePodInfo::ZigZag {
            last_waypoint_x,
            last_waypoint_y,
            transit_ticks,
            exhaust_angle,
            previous_exhaust_angle,
        },
        DronePodInfo::Empty => WireDronePodInfo::Empty,
        &DronePodInfo::Shield { glow_animation } => WireDronePodInfo::Shield { glow_animation },
    };
    WireDronePod {
        kind: p.kind,
        mourning_ticks: p.mourning_ticks,
        current_space: p.current_space,
        destination_space: p.destination_space,
        current_position_x: p.current_position_x,
        current_position_y: p.current_position_y,
        previous_position_x: p.previous_position_x,
        previous_position_y: p.previous_position_y,
        goal_position_x: p.goal_position_x,
        goal_position_y: p.goal_position_y,
        point_target_x: p.point_target_x,
        point_target_y: p.point_target_y,
        next_target_x: p.next_target_x,
        next_target_y: p.next_target_y,
        target_speed_x: p.target_speed_x,
        target_speed_y: p.target_speed_y,
        buildup_ticks: p.buildup_ticks,
        stationary_ticks: p.stationary_ticks,
        cooldown_ticks: p.cooldown_ticks,
        orbit_angle: p.orbit_angle,
        turret_angle: p.turret_angle,
        last_aiming_angle: p.last_aiming_angle,
        hops_to_live: p.hops_to_live,
        disrupted: p.disrupted,
        powered_last_frame: p.powered_last_frame,
        overload_ticks: p.overload_ticks,
        desired_aiming_angle: p.desired_aiming_angle,
        deployed_last_frame: p.deployed_last_frame,
        delta_position_x: p.delta_position_x,
        delta_position_y: p.delta_position_y,
        death_anim: anim_to_wire(p.death_anim),
        info,
    }
}

/// Structures the fixture does not contain, built from the layout of
/// SavedGameParser.java (`readProjectile`, `readDronePod`,
/// `readExtendedShipInfo`, `readWeaponModule`). Each test reads the bytes,
/// maps them to the domain and back, and writes the same bytes.
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite};
    use ftl_core::{DroneKind, Projectile, ProjectileInfo};

    use super::*;
    use crate::wire::{ExtendedShape, WireProjectile, read_extended};

    fn ints(v: &[i32]) -> Vec<u8> {
        v.iter().flat_map(|i| i.to_le_bytes()).collect()
    }

    fn string(s: &str) -> Vec<u8> {
        [ints(&[s.len() as i32]), s.as_bytes().to_vec()].concat()
    }

    /// playing, looping, frame, ticks, scale, x, y.
    fn anim() -> Vec<u8> {
        ints(&[1, 0, 3, 4, 1000, 5, 6])
    }

    fn projectile(kind: i32, info: Vec<u8>) -> Vec<u8> {
        [
            ints(&[kind]),
            ints(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            // damage: 7 ints, hull buster, owner, self, lockdown, shard, 2 ints
            ints(&[1, 0, 0, 0, 0, 1, 0, 0, -1, 77, 0, 0, 0, 0]),
            ints(&[100, 1, 0, -1, 0]),
            string("explosion_big"),
            string("laser_heavy_flight"),
            anim(),
            anim(),
            ints(&[11, 12, 0, 1]),
            string("hitHull"),
            string("hitShield"),
            string("miss"),
            ints(&[i32::MIN, 0, 0, 4, 1]),
            info,
        ]
        .concat()
    }

    fn projectile_round_trip(bytes: &[u8], format: i32) -> Projectile {
        let w = WireProjectile::read_le_args(&mut Cursor::new(bytes), (format,)).unwrap();
        let p = projectile::to_domain(w);
        let mut out = Cursor::new(Vec::new());
        projectile::to_wire(&p)
            .write_le_args(&mut out, (format,))
            .unwrap();
        assert_eq!(out.into_inner(), bytes);
        p
    }

    /// Not seen in a real file: the fixture has no projectile in flight.
    #[test]
    fn every_projectile_kind_round_trips() {
        let cases: [(i32, Vec<u8>); 6] = [
            (1, ints(&[0, 9])),
            (2, vec![]),
            (3, vec![]),
            (4, ints(&[1, 2, 3, 4, 1])),
            (5, [ints(&[7; 20]), ints(&[1, 0, 1, 0, 1])].concat()),
            (6, [ints(&[1, 2, 3, 4, 5]), anim()].concat()),
        ];
        for (kind, info) in cases {
            let p = projectile_round_trip(&projectile(kind, info), 11);
            let Projectile::Valid(p) = p else {
                panic!("kind {kind} is valid")
            };
            assert_eq!(p.info.flag(), kind);
            assert_eq!((p.damage.self_id, p.entry_angle), (77, i32::MIN));
            if kind == 1 {
                assert_eq!(
                    p.info,
                    ProjectileInfo::LaserOrBurst {
                        spin_angle: 0,
                        spin: 9
                    }
                );
            }
        }
        assert_eq!(projectile_round_trip(&ints(&[0]), 11), Projectile::Invalid);
    }

    #[test]
    fn a_pds_projectile_is_refused_before_format_11() {
        let bytes = projectile(6, [ints(&[1, 2, 3, 4, 5]), anim()].concat());
        let err = WireProjectile::read_le_args(&mut Cursor::new(&bytes), (9,)).unwrap_err();
        assert!(err.to_string().contains("projectile type flag 6"), "{err}");
    }

    /// The 29 ints and the death anim every pod starts with.
    fn pod_head() -> Vec<u8> {
        [ints(&(1..=29).collect::<Vec<_>>()), anim()].concat()
    }

    /// A format 8 weapon module (no charge anim) with one pending laser.
    fn weapon_module_format_8() -> Vec<u8> {
        [
            ints(&[1, 2, 3, 4, 5, 6]),
            ints(&[1, 10, 20]), // one current target
            ints(&[0]),         // no previous target
            ints(&[1, 0, -1]),
            anim(),
            ints(&[0, 0, 0]),
            ints(&[42]), // last projectile id
            ints(&[1]),
            projectile(1, ints(&[0, 3])),
        ]
        .concat()
    }

    /// Not seen in a real file: boarder, hacking, shield and beam pods, the
    /// hacking info, an artillery module, a standalone drone and a format 8
    /// weapon module.
    #[test]
    fn extended_info_of_a_flagship_like_ship_round_trips() {
        let format = 8;
        let bytes = [
            // drones: boarder, shield, repair (no pod)
            ints(&[1, 1]),
            pod_head(),
            ints(&[1, 2, 3, 4, 150, 5, 6, 7, 8]),
            ints(&[0, 1]),
            pod_head(),
            ints(&[99]),
            ints(&[0, 0]),
            // hacking: target engines
            ints(&[1, 0, 1, 0, 500, 1000, 1]),
            pod_head(),
            ints(&[10, 20, 30, 40]),
            anim(),
            anim(),
            // weapons: one module
            ints(&[1]),
            weapon_module_format_8(),
            // artillery: one module
            weapon_module_format_8(),
            // one standalone beam drone
            ints(&[1]),
            ints(&[11]),
            b"BEAM_DRONE1".to_vec(),
            pod_head(),
            ints(&[1, 2, 3, 4, 5]),
            ints(&[7, 8, 9]),
        ]
        .concat();
        let shape = ExtendedShape {
            drone_kinds: vec![DroneKind::Boarder, DroneKind::Shield, DroneKind::Repair],
            hacking: true,
            mind: false,
            weapons: Some(1),
            artillery: 1,
        };
        let kind = |id: &str| (id == "BEAM_DRONE1").then_some(DroneKind::Beam);
        let mut r = Cursor::new(&bytes[..]);
        let w = read_extended(&mut r, format, &shape, &kind)
            .unwrap()
            .unwrap();
        assert_eq!(r.position() as usize, bytes.len(), "read every byte");
        let e = to_domain(w).unwrap();
        assert!(matches!(
            e.drones[0].pod.as_ref().unwrap().info,
            DronePodInfo::Boarder {
                body_health: 150,
                ..
            }
        ));
        assert!(e.drones[2].pod.is_none());
        let h = e.hacking.as_ref().unwrap();
        assert_eq!(h.target_system, Some(SystemKind::Engines));
        assert_eq!(h.drone_pod.kind, DroneKind::Hacking);
        let m = &e.weapon_modules.as_ref().unwrap()[0];
        assert_eq!(
            (m.current_targets[0], m.last_projectile_id),
            (Reticle { x: 10, y: 20 }, 42)
        );
        assert_eq!(m.pending_projectiles.len(), 1);
        assert_eq!(e.artillery.len(), 1);
        let s = &e.standalone_drones[0];
        assert_eq!(
            (s.drone_id.as_str(), s.drone_pod.kind, s.armed),
            ("BEAM_DRONE1", DroneKind::Beam, 9)
        );
        let mut out = Cursor::new(Vec::new());
        to_wire(&e).write_le_args(&mut out, (format,)).unwrap();
        assert_eq!(out.into_inner(), bytes);

        // A standalone drone of unknown kind leaves the info undecoded.
        let none = |_: &str| None;
        let mut r = Cursor::new(&bytes[..]);
        assert!(
            read_extended(&mut r, format, &shape, &none)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_weapon_module_count_that_differs_from_the_weapons_is_refused() {
        let bytes = [
            ints(&[2]),
            weapon_module_format_8(),
            weapon_module_format_8(),
            ints(&[0]),
        ]
        .concat();
        let shape = ExtendedShape {
            drone_kinds: vec![],
            hacking: false,
            mind: false,
            weapons: Some(1),
            artillery: 0,
        };
        let err = read_extended(&mut Cursor::new(&bytes), 8, &shape, &|_| None).unwrap_err();
        assert!(
            err.to_string()
                .contains("found 2 weapon modules, but the ship has 1 weapons"),
            "{err}"
        );
    }
}
