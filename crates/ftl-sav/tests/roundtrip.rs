//! The codec contract on real files: decode, encode, same bytes.
//!
//! Every file under tests/fixtures/ is a saved game written by FTL itself.
//! A new FTL version or a new edge case gets a new fixture here.

use ftl_core::{
    Difficulty, DroneKind, DronePodInfo, FleetPresence, HazardVulnerability, KnownLayouts, Patch,
    SaveCodec, SavedGame, ShipId, ShipLayouts, StationDirection, SystemKind,
};
use ftl_sav::SavCodec;

#[path = "common/layouts.rs"]
mod layouts;

use layouts::fixture_layouts;

/// Decodes with the fake layout of the fixture ship.
fn decode(bytes: &[u8]) -> Result<SavedGame, ftl_core::DecodeError> {
    SavCodec.decode(bytes, &fixture_layouts())
}

const FORMAT11: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11.sav");

/// The second fixture: The Kestrel in sector 3 with the auto ship
/// `AUTO_ASSAULT` nearby, whose `COMBAT_BEAM` is deployed. It holds the
/// drone state the main fixture lacks.
const DRONE_FIXTURE: &[u8] =
    include_bytes!("fixtures/continue-1.6.13-format11-sector3-enemy-drone.sav");

#[test]
fn every_fixture_round_trips_byte_identical() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    let mut seen = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "sav") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        // With and without the layout: the codec contract holds for both.
        for layouts in [fixture_layouts(), KnownLayouts::new()] {
            let game = SavCodec
                .decode(&bytes, &layouts)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert!(
                SavCodec.encode(&game) == bytes,
                "{} does not round-trip",
                path.display()
            );
        }
        seen += 1;
    }
    assert!(seen > 0, "no fixtures in {dir}");
}

#[test]
fn decodes_the_header_and_resources_of_a_1_6_13_save() {
    let g = decode(FORMAT11).unwrap();
    assert_eq!(g.file_format, 11);
    assert!(g.dlc_enabled);
    assert_eq!(g.difficulty, Difficulty::Easy);
    assert_eq!(g.player_ship.name, "The Kestrel");
    assert_eq!(g.player_ship.blueprint_id, "PLAYER_SHIP_HARD");
    assert_eq!(g.player_ship_name, g.player_ship.name);
    assert_eq!(g.sector_number, 1);
    assert!(g.player_ship.hostile);
    let r = g.player_ship.resources;
    assert_eq!(
        (r.hull, r.fuel, r.missiles, r.drone_parts, r.scrap),
        (28, 15, 6, 3, 53)
    );
}

/// Values of the fixture as `ftled show --json` prints them; the Java
/// oracle (`SaveRoundTrip`) reads the file byte-identical.
#[test]
fn decodes_the_crew_of_a_1_6_13_save() {
    let g = decode(FORMAT11).unwrap();
    let crew = &g.player_ship.crew;
    let names: Vec<&str> = crew.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Berko", "Chris Malott", "Elizabeth"]);
    let c = &crew[0];
    assert_eq!(
        (c.race.as_str(), c.health, c.sprite_x, c.sprite_y),
        ("human", 100, 507, 157)
    );
    assert_eq!(
        (
            c.room_id,
            c.room_square,
            c.saved_room_id,
            c.saved_room_square
        ),
        (0, 0, 0, 0)
    );
    assert!(c.player_controlled && c.male && !c.mind_controlled);
    let s = &c.skills;
    assert_eq!(
        [
            s.pilot.points,
            s.engine.points,
            s.shield.points,
            s.weapon.points,
            s.repair.points,
            s.combat.points
        ],
        [2, 0, 0, 0, 0, 0]
    );
    // The fixture's crew has no mastery yet; the crew list tests set one.
    assert!(
        [s.pilot, s.engine, s.shield, s.weapon, s.repair, s.combat]
            .iter()
            .all(|k| !k.mastery_one && !k.mastery_two)
    );
    assert_eq!(
        (
            c.repairs,
            c.combat_kills,
            c.jumps_survived,
            c.skill_masteries_earned
        ),
        (0, 0, 2, 0)
    );
    assert_eq!(
        (c.clonebay_priority, c.damage_boost, c.universal_death_count),
        (-1, 1000, 0)
    );
    assert!(c.teleport_anim.playing && c.teleport_anim.scale == 1000);
    assert_eq!(c.piloted_evasions, 4);
    assert_eq!(c.sprite_tints, [0, 3]);
    assert!(!crew[2].male && crew[2].race == "human");
}

#[test]
fn an_edit_changes_only_the_four_bytes_of_the_field() {
    let mut g = decode(FORMAT11).unwrap();
    Patch::new(vec!["ship.scrap=9999".parse().unwrap()])
        .apply(&mut g)
        .unwrap();
    let out = SavCodec.encode(&g);
    assert_eq!(out.len(), FORMAT11.len());
    let diff: Vec<usize> = (0..out.len()).filter(|&i| out[i] != FORMAT11[i]).collect();
    assert!(
        !diff.is_empty() && diff.len() <= 4,
        "changed offsets: {diff:?}"
    );
    // Strings before the field have any length, so the field is not 4-aligned.
    // 53 -> 9999 changes the lowest byte, so the first changed byte starts the int.
    let at = diff[0];
    assert_eq!(
        i32::from_le_bytes(out[at..at + 4].try_into().unwrap()),
        9999
    );
}

#[test]
fn a_truncated_file_names_offset_and_field() {
    let err = decode(&FORMAT11[..60]).unwrap_err();
    assert!(err.offset > 0 && err.offset <= 60, "{err}");
    assert!(
        err.problem.contains("string length") || err.problem.contains("end"),
        "{err}"
    );
}

#[test]
fn an_unknown_format_is_refused_with_its_number() {
    let mut bytes = FORMAT11.to_vec();
    bytes[0] = 12;
    let err = decode(&bytes).unwrap_err();
    assert!(err.problem.contains("unsupported file format 12"), "{err}");
}

/// Offsets where `edited` differs from the fixture, which must keep its size.
fn changed_offsets(edited: &[u8]) -> Vec<usize> {
    assert_eq!(edited.len(), FORMAT11.len());
    (0..edited.len())
        .filter(|&i| edited[i] != FORMAT11[i])
        .collect()
}

#[test]
fn a_crew_skill_edit_changes_only_that_int() {
    let mut g = decode(FORMAT11).unwrap();
    Patch::new(vec!["crew.1.skills.engine=20".parse().unwrap()])
        .apply(&mut g)
        .unwrap();
    let out = SavCodec.encode(&g);
    let diff = changed_offsets(&out);
    assert_eq!(diff.len(), 1, "changed offsets: {diff:?}");
    assert_eq!(out[diff[0]], 20);
    assert_eq!(decode(&out).unwrap(), g);
}

/// Values of the fixture as `ftled show --json` prints them; the Java
/// oracle (`SaveRoundTrip`) reads the file byte-identical.
#[test]
fn decodes_reactor_and_systems_of_a_1_6_13_save() {
    let g = decode(FORMAT11).unwrap();
    let s = &g.player_ship;
    assert_eq!(s.reactor_power, 8);
    let levels: Vec<(&str, i32, i32)> = s
        .systems
        .iter()
        .map(|x| {
            let st = x.installed.as_ref();
            (
                x.kind.id(),
                st.map_or(0, |i| i.capacity),
                st.map_or(0, |i| i.power),
            )
        })
        .collect();
    assert_eq!(
        levels,
        [
            ("shields", 2, 2),
            ("engines", 2, 1),
            ("oxygen", 1, 1),
            ("weapons", 3, 3),
            ("drones", 0, 0),
            ("medbay", 1, 1),
            ("pilot", 1, 1),
            ("sensors", 1, 1),
            ("doors", 1, 1),
            ("teleporter", 0, 0),
            ("cloaking", 0, 0),
            ("artillery", 0, 0),
            ("battery", 0, 0),
            ("clonebay", 0, 0),
            ("mind", 0, 0),
            ("hacking", 0, 0),
        ]
    );
    let shields = s.systems[0].installed.as_ref().unwrap();
    assert_eq!(
        (
            shields.temporary_capacity_cap,
            shields.temporary_capacity_divisor
        ),
        (1000, 1)
    );
    assert!(s.clonebay.is_none() && s.battery.is_none() && s.cloaking.is_none());
    let sh = s.shields.as_ref().unwrap();
    assert_eq!(
        (sh.shield_layers, sh.raise_anim_on, sh.raise_anim_ticks),
        (1, true, 1000)
    );
    assert_eq!(
        (sh.shields_down_point_x, sh.shields_down_point_y),
        (569371, 157735)
    );
}

/// The fixture's Kestrel has neither Battery nor Cloaking: both installed
/// through the edit paths write their state blocks, which decode back.
#[test]
fn an_installed_battery_and_cloaking_write_their_state_blocks() {
    let l = layouts::install_layouts();
    let mut g = SavCodec.decode(FORMAT11, &l).unwrap();
    Patch::new(vec![
        "systems.battery.installed=1".parse().unwrap(),
        "systems.cloaking.installed=1".parse().unwrap(),
    ])
    .apply_in(&mut g, &l)
    .unwrap();
    let out = SavCodec.encode(&g);
    let back = SavCodec.decode(&out, &l).unwrap();
    assert_eq!(back, g);
    assert!(back.undecoded.is_empty());
    let battery = back.player_ship.battery.as_ref().unwrap();
    assert!(!battery.active && battery.used_battery == 0);
    assert!(back.player_ship.cloaking.is_some());
}

#[test]
fn a_system_level_edit_changes_only_that_int() {
    let mut g = decode(FORMAT11).unwrap();
    Patch::new(vec!["systems.shields.level=8".parse().unwrap()])
        .apply(&mut g)
        .unwrap();
    let out = SavCodec.encode(&g);
    let diff = changed_offsets(&out);
    assert_eq!(diff.len(), 1, "changed offsets: {diff:?}");
    assert_eq!((FORMAT11[diff[0]], out[diff[0]]), (2, 8));
    assert_eq!(decode(&out).unwrap(), g);
}

/// Values of the fixture as `ftled show --json` prints them; the Java
/// oracle (`SaveRoundTrip`) reads the file byte-identical.
#[test]
fn decodes_rooms_doors_and_loadout_with_the_layout() {
    let g = decode(FORMAT11).unwrap();
    let b = g
        .player_ship
        .body
        .as_ref()
        .expect("the fake layout knows the fixture ship");
    assert_eq!(b.layout_id, "kestral");
    assert_eq!(b.rooms.len(), 17);
    assert!(b.rooms.iter().all(|r| r.oxygen == 100));
    let st = |i: usize| b.rooms[i].station.map(|s| (s.square, s.direction));
    assert_eq!(st(0), Some((0, StationDirection::Right)));
    assert_eq!(st(1), Some((-1, StationDirection::None)));
    assert_eq!(st(2), Some((1, StationDirection::Up)));
    assert_eq!(st(5), Some((0, StationDirection::Left)));
    assert!(
        b.rooms[0]
            .squares
            .iter()
            .all(|q| q.fire_health == 0 && q.extinguishment_progress == -1)
    );
    assert!(b.breaches.is_empty());
    assert_eq!(b.doors.len(), 26);
    let d = b.doors[0];
    assert_eq!(
        (
            d.current_max_health,
            d.health,
            d.nominal_health,
            d.door_level,
            d.hacked
        ),
        (4, 4, 4, 0, 0)
    );
    assert!(!d.open && !d.walking_through);
    assert_eq!(b.cloak_anim_ticks, 0);
    assert!(b.lockdown_crystals.is_empty());
    let weapons: Vec<(&str, bool)> = b.weapons.iter().map(|w| (w.id.as_str(), w.armed)).collect();
    assert_eq!(
        weapons,
        [("MISSILES_2_PLAYER", true), ("LASER_BURST_3", true)]
    );
    // No drone and no augment in the fixture; tests/items.rs adds drones.
    assert!(b.drones.is_empty() && b.augments.is_empty());
}

#[test]
fn without_a_layout_the_body_stays_undecoded() {
    let with = decode(FORMAT11).unwrap();
    let without = SavCodec.decode(FORMAT11, &KnownLayouts::new()).unwrap();
    assert!(without.player_ship.body.is_none());
    // The body and the world of the fixture ship are 2488 bytes, the
    // situation 2233: with the layouts nothing is left.
    assert_eq!((with.undecoded.len(), without.undecoded.len()), (0, 4721));
    assert_eq!(without.player_ship.systems, with.player_ship.systems);
}

/// Adds an augment and writes the result to the test tmp dir, where the
/// Java oracle can read it (`fe-codec-augment.sav`).
#[test]
fn an_added_augment_grows_the_file_by_its_string() {
    let mut g = decode(FORMAT11).unwrap();
    Patch::new(vec!["augments.SCRAP_COLLECTOR=1".parse().unwrap()])
        .apply(&mut g)
        .unwrap();
    let out = SavCodec.encode(&g);
    // 4 bytes count stays, + 4 bytes length + 15 bytes id.
    assert_eq!(out.len(), FORMAT11.len() + 4 + "SCRAP_COLLECTOR".len());
    let again = decode(&out).unwrap();
    assert_eq!(again, g);
    assert_eq!(
        again.player_ship.body.unwrap().augments,
        ["SCRAP_COLLECTOR"]
    );
    std::fs::write(
        concat!(env!("CARGO_TARGET_TMPDIR"), "/fe-codec-augment.sav"),
        &out,
    )
    .unwrap();
}

/// Values cross-checked with the Java parser (SavedGameState.toString of
/// the fixture: "Sector Data", "Sector Beacons", "Quests").
#[test]
fn decodes_sector_beacons_and_rebel_fleet_with_the_layout() {
    let g = decode(FORMAT11).unwrap();
    let w = g
        .world
        .as_ref()
        .expect("the world follows the decoded body");
    assert!(w.cargo.is_empty());
    assert_eq!(
        (w.sector_tree_seed, w.sector_layout_seed),
        (571972150, 169148137)
    );
    let f = &w.rebel_fleet;
    assert_eq!((f.offset, f.fudge, f.pursuit_mod), (-831, 100, 0));
    assert!(!f.flagship_visible && !f.flagship_moving && !f.flagship_retreating);
    assert_eq!((f.flagship_hop, f.flagship_base_turns), (0, 0));
    assert_eq!((w.current_beacon_id, w.wait_event_seed), (8, -1));
    assert!(!w.waiting && w.last_escape_event.is_empty() && !w.sector_hazards_visible);
    let route: String = w
        .sector_visitation
        .iter()
        .map(|&v| if v { 'T' } else { 'F' })
        .collect();
    assert_eq!(route, "TFFFFFFFFFFFFFFFFFF");
    assert_eq!(w.sector_number, g.sector_number - 1);
    assert!(!w.sector_is_hidden_crystal_worlds);
    let b8 = &w.beacons[8];
    assert_eq!(b8.visit_count, 1);
    let bg = b8.background.as_ref().unwrap();
    assert_eq!(
        (
            bg.starscape.as_str(),
            bg.sprite.as_str(),
            bg.sprite_x,
            bg.sprite_y,
            bg.sprite_rotation
        ),
        (
            "stars/bg_lonelystar.png",
            "stars/planet_gas_blue.png",
            276,
            85,
            180
        )
    );
    let e = b8.enemy.as_ref().unwrap();
    assert_eq!(
        (
            e.ship_event_id.as_str(),
            e.auto_blueprint_id.as_str(),
            e.ship_event_seed
        ),
        ("PIRATE_BRIBER", "SHIPS_PIRATE", 1670091146)
    );
    assert!(b8.seen && !b8.under_attack && b8.store.is_none());
    assert!(w.beacons[0].background.is_none() && w.beacons[0].enemy.is_none());
    assert_eq!(
        w.beacons[5].background.as_ref().unwrap().sprite,
        "stars/planet_bigblue.png"
    );
    assert_eq!(w.beacons.len(), 21);
    // Sector 1: the rebel fleet has reached no beacon yet.
    assert!(
        w.beacons
            .iter()
            .all(|b| b.fleet_presence == FleetPresence::None)
    );
    assert!(w.beacons[12].seen && !w.beacons[20].seen);
    assert!(
        w.beacons.iter().all(|b| b.store.is_none()),
        "no store in the fixture"
    );
    assert!(w.quest_events.is_empty() && w.distant_quest_events.is_empty());
}

#[test]
fn without_a_layout_the_world_stays_undecoded() {
    let without = SavCodec.decode(FORMAT11, &KnownLayouts::new()).unwrap();
    assert!(without.world.is_none());
}

#[test]
fn a_beacon_seen_edit_changes_only_that_int() {
    let mut g = decode(FORMAT11).unwrap();
    Patch::new(vec!["beacons.5.seen=0".parse().unwrap()])
        .apply(&mut g)
        .unwrap();
    let out = SavCodec.encode(&g);
    let diff = changed_offsets(&out);
    assert_eq!(diff.len(), 1, "changed offsets: {diff:?}");
    assert_eq!((FORMAT11[diff[0]], out[diff[0]]), (1, 0));
    assert_eq!(decode(&out).unwrap(), g);
}

/// Values cross-checked with the Java parser (SavedGameState.toString of
/// the fixture: "Current Encounter", "Nearby Ship", "Nearby Ship AI",
/// "Environment Hazards", "Rebel Flagship", the extended drone info).
#[test]
fn decodes_encounter_nearby_ship_and_flagship_to_the_end_of_the_file() {
    let g = decode(FORMAT11).unwrap();
    assert!(g.undecoded.is_empty(), "{} bytes left", g.undecoded.len());
    let s = g.situation.as_ref().expect("the fake knows every fact");
    assert_eq!(s.ship_event_present, 1);
    let e = s.encounter.as_ref().unwrap();
    assert_eq!(e.ship_event_seed, 1670091146);
    assert_eq!(e.destroyed_event_id, "PIRATE_BRIBER destroyed  1230");
    assert_eq!(e.text, "event_PIRATE_BRIBER_c2_text");
    assert_eq!((e.affected_crew_seed, e.choices.clone()), (-1, vec![1, 0]));
    let n = s.nearby_ship.as_ref().unwrap();
    assert!(!n.rebel_flagship_nearby);
    assert_eq!(
        (n.ship.blueprint_id.as_str(), n.ship.name.as_str()),
        ("JELLY_BUTTON_P_DLC", "should not be seen")
    );
    let r = n.ship.resources;
    assert_eq!((r.hull, r.fuel, r.missiles, r.scrap), (8, 16, 6, 175));
    let crew: Vec<(&str, &str)> = n
        .ship
        .crew
        .iter()
        .map(|c| (c.name.as_str(), c.race.as_str()))
        .collect();
    assert_eq!(crew, [("Jack", "rock"), ("Ferry", "mantis")]);
    assert_eq!(n.ship.reactor_power, 7);
    let body = n.ship.body.as_ref().unwrap();
    assert_eq!(
        (body.layout_id.as_str(), body.rooms.len()),
        ("jelly_button_pirate", 6)
    );
    assert_eq!(body.augments, ["SLUG_GEL"]);
    let ai = n.ai.as_ref().unwrap();
    assert_eq!(
        (
            ai.surrender_threshold,
            ai.escape_threshold,
            ai.escape_ticks,
            ai.boarders_needed
        ),
        (0, 4, 15000, 1)
    );
    let x = n.ship.extended.as_ref().unwrap();
    assert_eq!(x.weapon_modules.as_ref().map(Vec::len), Some(2));
    assert!(x.drones.is_empty() && x.standalone_drones.is_empty());
    let env = s.environment.as_ref().unwrap();
    assert!(!env.pds_present && !env.red_giant_present && env.asteroid_field.is_none());
    assert_eq!(env.vulnerable_ships, HazardVulnerability::BothShips);
    assert_eq!((env.havoc_ticks, env.pds_ticks), (0, 0));
    assert!(s.projectiles.is_empty());
    assert_eq!(
        (
            n.teleport_target_room,
            s.player_teleport_target_room,
            s.autofire
        ),
        (0, 0, false)
    );
    let f = &s.rebel_flagship;
    assert_eq!(
        (
            f.current_stage,
            f.pending_stage,
            f.power_ticks_goal,
            f.power_ticks
        ),
        (0, 1, 30000, 0)
    );
    assert!(f.previous_occupancy.is_empty());

    let p = g.player_ship.extended.as_ref().unwrap();
    // No drone and no Mind Control on the fixture's Kestrel.
    assert!(p.drones.is_empty() && p.mind.is_none());
    let modules = p.weapon_modules.as_ref().unwrap();
    assert_eq!(modules.len(), 2);
    assert_eq!(
        (modules[0].cooldown_ticks, modules[0].cooldown_ticks_goal),
        (702, 9900)
    );
    assert_eq!(modules[1].last_projectile_id, -1);
    assert!(!modules[0].autofire);
    assert!(p.hacking.is_none() && p.artillery.is_empty());
}

/// A drone FTL deployed, read from the second fixture (values as `ftled
/// dump` prints them; the Java oracle `SaveRoundTrip` reads the file
/// byte-identical). The nearby ship's drone system draws exactly the bars
/// of its armed drone: power 2 = `COMBAT_BEAM` 2.
#[test]
fn decodes_the_deployed_drone_of_the_nearby_ship() {
    let g = SavCodec
        .decode(DRONE_FIXTURE, &layouts::drone_fixture_layouts())
        .unwrap();
    assert!(g.undecoded.is_empty());
    assert_eq!(g.sector_number, 3);
    let s = g.situation.as_ref().expect("the fake knows AUTO_ASSAULT");
    let n = &s.nearby_ship.as_ref().unwrap().ship;
    assert_eq!(n.blueprint_id, layouts::DRONE_FIXTURE_NEARBY_SHIP);
    assert!(n.crew.is_empty(), "an auto ship has no crew");
    let drones = n
        .systems
        .iter()
        .find(|x| x.kind == SystemKind::Drones)
        .and_then(|x| x.installed.as_ref())
        .unwrap();
    assert_eq!((drones.capacity, drones.power), (3, 2));

    let body = n.body.as_ref().unwrap();
    assert_eq!(body.drones.len(), 1);
    let d = &body.drones[0];
    assert_eq!(
        (
            d.id.as_str(),
            d.armed,
            d.deployed,
            d.body_x,
            d.body_y,
            d.body_room_id,
            d.body_room_square,
            d.health
        ),
        ("COMBAT_BEAM", true, true, -1, -1, -1, -1, 1)
    );

    let x = n.extended.as_ref().unwrap();
    assert_eq!(x.drones.len(), 1);
    let e = &x.drones[0];
    assert!(e.deployed && e.armed);
    let pod = e.pod.as_ref().unwrap();
    assert_eq!(pod.kind, DroneKind::Combat);
    // In the player's space (0), flying to its goal there.
    assert_eq!((pod.current_space, pod.destination_space), (0, 0));
    assert_eq!(
        (
            pod.current_position_x,
            pod.current_position_y,
            pod.previous_position_x,
            pod.previous_position_y,
            pod.delta_position_x,
            pod.delta_position_y
        ),
        (218838, 340003, 222770, 339029, -3931, 973)
    );
    assert_eq!((pod.goal_position_x, pod.goal_position_y), (18707, 389556));
    assert_eq!((pod.next_target_x, pod.next_target_y), (70000, 175000));
    assert_eq!(
        (
            pod.orbit_angle,
            pod.turret_angle,
            pod.last_aiming_angle,
            pod.desired_aiming_angle
        ),
        (122000, -51108, 0, -76555)
    );
    assert_eq!(
        pod.info,
        DronePodInfo::ZigZag {
            last_waypoint_x: 620785,
            last_waypoint_y: 240481,
            transit_ticks: 667,
            exhaust_angle: 166093,
            previous_exhaust_angle: 21224,
        }
    );
    assert!(x.standalone_drones.is_empty() && x.mind.is_none() && x.hacking.is_none());
    let env = s.environment.as_ref().unwrap();
    let rocks = env.asteroid_field.as_ref().unwrap();
    assert_eq!((rocks.init_shields, rocks.state), (2, 1));
    assert_eq!(s.projectiles.len(), 1);
}

/// The fields `SpaceDrone::SpaceDrone` sets and flight leaves alone hold the
/// values `items::new_pod` writes for a new drone: FTL wrote the same for
/// its deployed `COMBAT_BEAM` (cooldown 1000 ms in the blueprint). The
/// fields that move in flight (positions, goal, angles, waypoint, transit)
/// carry no evidence for a fresh drone.
#[test]
fn a_new_drone_pod_matches_what_ftl_keeps_for_a_deployed_one() {
    let g = SavCodec
        .decode(DRONE_FIXTURE, &layouts::drone_fixture_layouts())
        .unwrap();
    let n = &g
        .situation
        .as_ref()
        .unwrap()
        .nearby_ship
        .as_ref()
        .unwrap()
        .ship;
    let ftl = n.extended.as_ref().unwrap().drones[0].pod.as_ref().unwrap();
    let new = ftl_core::items::new_pod(DroneKind::Combat, Some(1000), ShipId::Nearby).unwrap();
    let stable = |p: &ftl_core::DronePod| {
        [
            p.mourning_ticks,
            p.point_target_x,
            p.point_target_y,
            p.target_speed_x,
            p.target_speed_y,
            p.buildup_ticks,
            p.stationary_ticks,
            p.cooldown_ticks,
            p.hops_to_live,
            p.disrupted,
            p.powered_last_frame,
            p.overload_ticks,
            p.deployed_last_frame,
        ]
    };
    assert_eq!(stable(ftl), stable(&new));
    assert_eq!(ftl.death_anim, new.death_anim);
    assert_eq!(
        (ftl.point_target_x, ftl.buildup_ticks, ftl.hops_to_live),
        (i32::MIN, -1000, i32::MAX)
    );
}

#[test]
fn without_a_nearby_layout_or_drone_kind_the_situation_stays_undecoded() {
    let world_only = decode(FORMAT11).unwrap().undecoded.len();
    assert_eq!(world_only, 0);
    // Without the nearby ship's layout.
    let g = SavCodec.decode(FORMAT11, &layouts::ship_layouts()).unwrap();
    assert!(g.world.is_some() && g.situation.is_none());
    assert!(g.player_ship.extended.is_none());
    assert_eq!(g.undecoded.len(), 2233);
    assert!(SavCodec.encode(&g) == FORMAT11);

    // Without the kind of a drone the player ship carries. The fixture's
    // Kestrel has no drone, so the test adds a COMBAT_1 first.
    let l = layouts::install_layouts();
    let mut g = SavCodec.decode(FORMAT11, &l).unwrap();
    Patch::new(vec![
        "systems.drones.installed=1".parse().unwrap(),
        "drones.COMBAT_1=1".parse().unwrap(),
    ])
    .apply_in(&mut g, &l)
    .unwrap();
    let with_drone = SavCodec.encode(&g);
    let no_drone_kind = layouts::ship_layouts().with(
        layouts::FIXTURE_NEARBY_SHIP,
        fixture_layouts()
            .ship(layouts::FIXTURE_NEARBY_SHIP)
            .unwrap(),
    );
    let g = SavCodec.decode(&with_drone, &no_drone_kind).unwrap();
    assert!(g.world.is_some() && g.situation.is_none());
    assert!(g.player_ship.extended.is_none());
    assert!(!g.undecoded.is_empty());
    assert!(SavCodec.encode(&g) == with_drone);
}

#[test]
fn a_nearby_hull_edit_changes_only_that_int() {
    let mut g = decode(FORMAT11).unwrap();
    Patch::new(vec!["nearby.ship.hull=3".parse().unwrap()])
        .apply(&mut g)
        .unwrap();
    let out = SavCodec.encode(&g);
    let diff = changed_offsets(&out);
    assert_eq!(diff.len(), 1, "changed offsets: {diff:?}");
    assert_eq!((FORMAT11[diff[0]], out[diff[0]]), (8, 3));
    assert_eq!(decode(&out).unwrap(), g);
}

/// A fire and a breach written into the fixture, then cleared through the
/// edit paths `fires.<x>.<y>` and `breaches.<x>.<y>`, give back the bytes
/// FTL wrote: both paths touch only what they clear. Square 1 of room 5
/// (2x2 at 8, 3 in kestral.txt, offset 0, 2) is grid square 9, 5.
#[test]
fn putting_out_a_fire_and_sealing_a_breach_give_back_the_fixture() {
    let mut g = decode(FORMAT11).unwrap();
    let body = g.player_ship.body.as_mut().unwrap();
    assert_eq!(body.rooms[5].grid_square(1), (9, 5));
    let square = &mut body.rooms[5].squares[1];
    assert_eq!((square.fire_health, square.ignition_progress), (0, 0));
    square.fire_health = 100;
    square.ignition_progress = 25;
    body.breaches.push(ftl_core::Breach {
        x: 9,
        y: 6,
        health: 100,
    });
    let burning = SavCodec.encode(&g);
    assert_ne!(burning, FORMAT11);

    let mut g = decode(&burning).unwrap();
    let changes = Patch::new(vec![
        "fires.9.5=0".parse().unwrap(),
        "breaches.9.6=0".parse().unwrap(),
    ])
    .apply(&mut g)
    .unwrap();
    assert_eq!(
        changes.iter().map(|c| c.old.clone()).collect::<Vec<_>>(),
        [100, 100]
    );
    assert!(SavCodec.encode(&g) == FORMAT11);
}

#[test]
fn a_renamed_crew_member_with_a_new_race_round_trips_and_changes_back() {
    let mut g = decode(FORMAT11).unwrap();
    Patch::new(vec![
        "crew.2.name=Ripley".parse().unwrap(),
        "crew.2.race=crystal".parse().unwrap(),
        "crew.0.race=engi".parse().unwrap(),
    ])
    .apply(&mut g)
    .unwrap();
    let bytes = SavCodec.encode(&g);
    let back = decode(&bytes).unwrap();
    assert_eq!(
        back, g,
        "the Crystal lockdown fields and the tints decode back"
    );
    let ripley = &back.player_ship.crew[2];
    assert_eq!(
        (ripley.name.as_str(), ripley.race.as_str()),
        ("Ripley", "crystal")
    );
    assert_eq!(back.player_ship.crew[0].race, "engi");

    // Back to the old names and races: the fixture's crew had full health,
    // so health comes back. Engi and Crystal have no second tint layer, so
    // the race change dropped the human's index 3 of layer 1; setting it
    // again gives back every byte.
    let mut g = back;
    Patch::new(vec![
        "crew.2.name=Elizabeth".parse().unwrap(),
        "crew.2.race=human".parse().unwrap(),
        "crew.0.race=human".parse().unwrap(),
        "crew.0.tints.1=3".parse().unwrap(),
        "crew.2.tints.1=3".parse().unwrap(),
    ])
    .apply(&mut g)
    .unwrap();
    assert!(SavCodec.encode(&g) == FORMAT11);
}

/// Lighting a fire and adding a breach through the edit paths writes a file
/// that decodes to the same game; clearing both gives back the fixture.
#[test]
fn a_new_fire_and_a_new_breach_round_trip() {
    let mut g = decode(FORMAT11).unwrap();
    Patch::new(vec![
        "fires.9.5=100".parse().unwrap(),
        "breaches.9.6=100".parse().unwrap(),
    ])
    .apply(&mut g)
    .unwrap();
    let body = g.player_ship.body.as_ref().unwrap();
    assert_eq!(body.rooms[5].squares[1].fire_health, 100);
    assert_eq!(body.rooms[5].squares[1].extinguishment_progress, -1);
    assert_eq!(
        body.breaches,
        [ftl_core::Breach {
            x: 9,
            y: 6,
            health: 100
        }]
    );
    let lit = SavCodec.encode(&g);
    assert_ne!(lit, FORMAT11);
    let mut back = decode(&lit).unwrap();
    assert_eq!(back, g);
    assert!(SavCodec.encode(&back) == lit);
    Patch::new(vec![
        "fires.9.5=0".parse().unwrap(),
        "breaches.9.6=0".parse().unwrap(),
    ])
    .apply(&mut back)
    .unwrap();
    assert!(SavCodec.encode(&back) == FORMAT11);
}
