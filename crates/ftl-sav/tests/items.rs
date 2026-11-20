//! Weapons, drones and cargo edited on the real fixture: every edit
//! decodes back to the edited game, and the files land in the test tmp dir
//! for the Java oracle (`fe-weapons-*.sav`, see AGENTS.md).

use ftl_core::{
    DroneKind, DronePodInfo, EditError, Field, KnownLayouts, Patch, SaveCodec, SavedGame,
    SystemKind,
};
use ftl_sav::SavCodec;

#[path = "common/layouts.rs"]
mod layouts;

use layouts::{install_layouts, item_layouts};

const FORMAT11: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11.sav");

fn game() -> SavedGame {
    SavCodec.decode(FORMAT11, &item_layouts()).unwrap()
}

fn apply(g: &mut SavedGame, edits: &[&str]) -> Result<(), EditError> {
    let patch = Patch::new(edits.iter().map(|e| e.parse().unwrap()).collect());
    patch.apply_in(g, &item_layouts()).map(|_| ())
}

/// The fixture after `edits`, applied with the system rooms as well. The
/// fixture's Kestrel has two weapons in four slots, no Drone Control, no
/// drone, no cargo and no augment: the tests build what they need.
fn game_with(edits: &[&str]) -> SavedGame {
    let mut g = game();
    let patch = Patch::new(edits.iter().map(|e| e.parse().unwrap()).collect());
    patch.apply_in(&mut g, &install_layouts()).unwrap();
    g
}

/// Drone Control installed, with these drones in its two slots.
fn with_drones(drones: &[&str]) -> SavedGame {
    let mut edits = vec!["systems.drones.installed=1".to_owned()];
    edits.extend(drones.iter().map(|d| format!("drones.{d}")));
    game_with(&edits.iter().map(String::as_str).collect::<Vec<_>>())
}

fn modules(g: &SavedGame) -> &[ftl_core::WeaponModule] {
    g.player_ship
        .extended
        .as_ref()
        .unwrap()
        .weapon_modules
        .as_ref()
        .unwrap()
}

/// Encodes, checks that the bytes decode to the same game, and writes them
/// for the Java oracle.
fn round_trip(g: &SavedGame, name: &str) -> Vec<u8> {
    let out = SavCodec.encode(g);
    let again = SavCodec.decode(&out, &item_layouts()).unwrap();
    assert_eq!(&again, g, "{name} does not decode to the edited game");
    assert!(again.undecoded.is_empty(), "{name}");
    std::fs::write(
        format!("{}/fe-weapons-{name}.sav", env!("CARGO_TARGET_TMPDIR")),
        &out,
    )
    .unwrap();
    out
}

fn weapons(g: &SavedGame) -> Vec<(&str, bool)> {
    g.player_ship
        .body
        .as_ref()
        .unwrap()
        .weapons
        .iter()
        .map(|w| (w.id.as_str(), w.armed))
        .collect()
}

fn drones(g: &SavedGame) -> Vec<(&str, bool)> {
    g.player_ship
        .body
        .as_ref()
        .unwrap()
        .drones
        .iter()
        .map(|d| (d.id.as_str(), d.armed))
        .collect()
}

fn power(g: &SavedGame, kind: SystemKind) -> i32 {
    g.player_ship
        .systems
        .iter()
        .find(|s| s.kind == kind)
        .and_then(|s| s.installed.as_ref())
        .unwrap()
        .power
}

#[test]
fn the_slots_fill_up_and_the_system_power_is_the_armed_weapons() {
    let g = game();
    assert_eq!(
        weapons(&g),
        [("MISSILES_2_PLAYER", true), ("LASER_BURST_3", true)]
    );
    // Artemis (1) and Burst Laser II (2) armed: the weapons system draws 3.
    assert_eq!(power(&g, SystemKind::Weapons), 3);
    let data = item_layouts();
    // Two of four slots used: room for two more.
    let f: Field = "weapons.ION_4".parse().unwrap();
    assert_eq!(f.range_in(&g, &data).unwrap(), (0, 2));
    let f: Field = "weapons.LASER_BURST_3".parse().unwrap();
    assert_eq!(
        (f.get(&g).unwrap(), f.range_in(&g, &data).unwrap()),
        (1, (0, 3))
    );
    // Four of four slots: no room for a fifth weapon.
    let mut g = g;
    apply(&mut g, &["weapons.ION_4=1", "weapons.SHOTGUN_2=1"]).unwrap();
    let f: Field = "weapons.BEAM_HULL".parse().unwrap();
    assert_eq!(f.range_in(&g, &data).unwrap(), (0, 0));
    // The system has no free bar for the new Ion Blast II.
    let f: Field = "weapons.ION_4.armed".parse().unwrap();
    assert_eq!(
        (f.get(&g).unwrap(), f.range_in(&g, &data).unwrap()),
        (0, (0, 0))
    );
    round_trip(&g, "full-slots");
}

#[test]
fn a_weapon_is_replaced_in_its_slot_with_a_new_module() {
    let mut g = game();
    apply(&mut g, &["weapons.MISSILES_2_PLAYER=0", "weapons.ION_4=1"]).unwrap();
    // The new weapon takes the slot the missile launcher freed (slot_order).
    assert_eq!(weapons(&g), [("ION_4", false), ("LASER_BURST_3", true)]);
    let modules = modules(&g);
    assert_eq!(modules.len(), 2);
    // The Burst Laser keeps its own module (cooldown goal 10799 in the file).
    assert_eq!(modules[1].cooldown_ticks_goal, 10799);
    let new = &modules[0];
    assert_eq!(
        (
            new.cooldown_ticks,
            new.cooldown_ticks_goal,
            new.target_id,
            new.autofire,
            new.anim_charge,
            new.last_projectile_id,
            new.weapon_anim.playing,
            (new.charge_anim.x, new.charge_anim.y),
        ),
        (0, 4000, -1, false, -1, -1, true, (-1000, -1000))
    );
    // The armed launcher left: its bar is free.
    assert_eq!(power(&g, SystemKind::Weapons), 2);
    round_trip(&g, "weapon");
}

#[test]
fn removing_an_armed_weapon_frees_its_bars_and_arming_takes_them() {
    let mut g = game();
    apply(&mut g, &["weapons.LASER_BURST_3=0"]).unwrap();
    assert_eq!(power(&g, SystemKind::Weapons), 1);
    apply(&mut g, &["weapons.BEAM_HULL=1"]).unwrap();
    apply(&mut g, &["weapons.BEAM_HULL.armed=1"]).unwrap();
    assert_eq!(power(&g, SystemKind::Weapons), 3);
    assert_eq!(
        weapons(&g),
        [("MISSILES_2_PLAYER", true), ("BEAM_HULL", true)]
    );
    apply(&mut g, &["weapons.MISSILES_2_PLAYER.armed=0"]).unwrap();
    assert_eq!(power(&g, SystemKind::Weapons), 2);
    // A missile launcher does not start its animation.
    apply(&mut g, &["weapons.MISSILES_2_PLAYER=2"]).unwrap();
    assert!(!modules(&g)[2].weapon_anim.playing);
    round_trip(&g, "armed");
}

#[test]
fn a_drone_of_every_kind_ftl_equips_round_trips() {
    for (id, kind, _, cooldown) in layouts::FIXTURE_ITEM_DRONES {
        // One Defense drone in the first slot; the new drone takes the second.
        let mut g = with_drones(&["ANTI_DRONE=1"]);
        let copies = 1 + i32::from(id == "ANTI_DRONE");
        apply(&mut g, &[&format!("drones.{id}={copies}")]).unwrap();
        let body = g.player_ship.body.as_ref().unwrap();
        let ext = g.player_ship.extended.as_ref().unwrap();
        assert_eq!(body.drones.len(), 2, "{id}");
        assert_eq!(ext.drones.len(), 2, "{id}");
        assert_eq!(body.drones[0].id, "ANTI_DRONE", "{id}");
        let (d, e) = (&body.drones[1], &ext.drones[1]);
        assert_eq!(
            (d.id.as_str(), d.armed, d.deployed, d.body_room_id, d.health),
            (id, false, false, -1, 1)
        );
        assert!(!e.deployed && !e.armed, "{id}");
        match &e.pod {
            None => assert!(!kind.has_pod(), "{id}"),
            Some(pod) => {
                assert_eq!(pod.kind, kind);
                assert_eq!(
                    (
                        pod.current_space,
                        pod.destination_space,
                        pod.current_position_x
                    ),
                    (0, -1, i32::MIN)
                );
                assert_eq!(pod.cooldown_ticks, cooldown.unwrap_or(0), "{id}");
                assert_eq!(pod.hops_to_live, i32::MAX);
                if kind == DroneKind::Combat || kind == DroneKind::ShipRepair {
                    assert!(matches!(
                        pod.info,
                        DronePodInfo::ZigZag {
                            transit_ticks: 1,
                            ..
                        }
                    ));
                }
            }
        }
        round_trip(&g, &format!("drone-{}", id.to_lowercase()));
    }
}

#[test]
fn a_drone_is_armed_within_the_free_bars_and_a_crew_drone_not_at_all() {
    // Drone Control at 2 bars with a Defense and a Combat drone; the
    // reactor raised to 10, of which the fixture's systems draw 8.
    let mut g = with_drones(&["ANTI_DRONE=1", "COMBAT_1=1"]);
    apply(&mut g, &["reactor=10"]).unwrap();
    let data = item_layouts();
    let f: Field = "drones.ANTI_DRONE.armed".parse().unwrap();
    assert_eq!(f.range_in(&g, &data).unwrap(), (0, 1));
    apply(&mut g, &["drones.ANTI_DRONE.armed=1"]).unwrap();
    assert_eq!(power(&g, SystemKind::Drones), 1);
    let ext = &g.player_ship.extended.as_ref().unwrap().drones[0];
    assert!(ext.armed && !ext.deployed);
    assert_eq!(drones(&g)[0], ("ANTI_DRONE", true));
    // Combat drones take 2 bars each; one free bar is left.
    let f: Field = "drones.COMBAT_1.armed".parse().unwrap();
    assert_eq!(f.range_in(&g, &data).unwrap(), (0, 0));
    round_trip(&g, "drone-armed");

    // A Repair drone needs one bar, which is free, but FTL never arms it.
    apply(&mut g, &["drones.COMBAT_1=0", "drones.REPAIR=1"]).unwrap();
    let f: Field = "drones.REPAIR.armed".parse().unwrap();
    assert_eq!(f.range_in(&g, &data).unwrap(), (0, 0));
}

#[test]
fn the_cargo_holds_four_weapons_or_drones_and_moves_with_the_slots() {
    let g = game();
    let data = item_layouts();
    let f: Field = "cargo.BEAM_HULL".parse().unwrap();
    assert_eq!(
        (f.get(&g).unwrap(), f.range_in(&g, &data).unwrap()),
        (0, (0, 4))
    );
    // A Repair drone in the cargo and both drone slots full.
    let mut g = with_drones(&["COMBAT_1=1", "ANTI_DRONE=1"]);
    apply(&mut g, &["cargo.REPAIR=1"]).unwrap();
    assert_eq!(f.range_in(&g, &data).unwrap(), (0, 3));
    // The launcher to the cargo, then two combat drones from the editor.
    apply(
        &mut g,
        &[
            "weapons.MISSILES_2_PLAYER=0",
            "cargo.MISSILES_2_PLAYER=1",
            "cargo.COMBAT_1=2",
        ],
    )
    .unwrap();
    let cargo = &g.world.as_ref().unwrap().cargo;
    assert_eq!(
        cargo,
        &["REPAIR", "MISSILES_2_PLAYER", "COMBAT_1", "COMBAT_1"]
    );
    assert_eq!(f.range_in(&g, &data).unwrap(), (0, 0));
    let err = apply(&mut g.clone(), &["cargo.BEAM_HULL=1"]).unwrap_err();
    assert_eq!(err.to_string(), "cargo.BEAM_HULL: 1 is outside 0..=0");
    // The repair drone goes from the cargo into a drone slot? Both slots
    // are full; first the combat drone leaves.
    apply(
        &mut g,
        &["cargo.REPAIR=0", "drones.COMBAT_1=0", "drones.REPAIR=1"],
    )
    .unwrap();
    assert_eq!(drones(&g), [("REPAIR", false), ("ANTI_DRONE", false)]);
    round_trip(&g, "cargo");
}

#[test]
fn weapon_and_drone_edits_need_the_blueprints_and_refuse_what_ftl_does_not_equip() {
    let mut g = game();
    let err = Patch::new(vec!["weapons.LASER_BURST_3=0".parse().unwrap()])
        .apply(&mut g)
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "weapons.LASER_BURST_3: the slots of the ship and the blueprints need ftl.dat"
    );
    let err = apply(&mut g, &["weapons.NO_SUCH_GUN=0"]).unwrap_err();
    assert!(matches!(err, EditError::UnknownItem { .. }), "{err}");
    let mut g = with_drones(&[]);
    let data = item_layouts().with_drone_facts(
        "DRONE_HACKING",
        ftl_core::DroneFacts {
            power: 1,
            kind: DroneKind::Hacking,
            cooldown_ms: None,
        },
    );
    let err = Patch::new(vec![
        "drones.COMBAT_1=1".parse().unwrap(),
        "drones.DRONE_HACKING=1".parse().unwrap(),
    ])
    .apply_in(&mut g, &data)
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        "drones.DRONE_HACKING: FTL puts no HACKING drone in a drone slot"
    );
    // Without the ship body nothing of it is known.
    let mut bare = SavCodec.decode(FORMAT11, &KnownLayouts::new()).unwrap();
    let err = Patch::new(vec!["cargo.ION_4=1".parse().unwrap()])
        .apply_in(&mut bare, &item_layouts())
        .unwrap_err();
    assert!(matches!(err, EditError::NotDecoded { .. }), "{err}");
}

#[test]
fn a_deployed_drone_stays() {
    let mut g = with_drones(&["ANTI_DRONE=1"]);
    g.player_ship.extended.as_mut().unwrap().drones[0].deployed = true;
    let err = apply(&mut g, &["drones.ANTI_DRONE=0"]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "drones.ANTI_DRONE: the drone is deployed; recall it in FTL first"
    );
}

#[test]
fn a_replace_keeps_the_slot_order_of_weapons_drones_cargo_and_augments() {
    // Four weapons, two Combat drones, a Repair drone in the cargo and one
    // augment: the fixture carries none of the last three.
    let mut g = with_drones(&["COMBAT_1=2"]);
    apply(
        &mut g,
        &[
            "weapons.SHOTGUN_2=1",
            "weapons.BEAM_HULL=1",
            "cargo.REPAIR=1",
            "augments.NANO_MEDBAY=1",
        ],
    )
    .unwrap();
    // The first and the last weapon leave; the two new ones take their
    // slots in order, whatever the order of the edits.
    apply(
        &mut g,
        &[
            "weapons.BEAM_HULL=0",
            "weapons.MISSILES_2_PLAYER=0",
            "weapons.ION_4=1",
            "weapons.BEAM_1=1",
        ],
    )
    .unwrap();
    assert_eq!(
        weapons(&g),
        [
            ("ION_4", false),
            ("LASER_BURST_3", true),
            ("SHOTGUN_2", false),
            ("BEAM_1", false)
        ]
    );
    // The armed launcher left: its bar is free.
    assert_eq!(power(&g, SystemKind::Weapons), 2);
    assert_eq!(
        modules(&g)[0].cooldown_ticks,
        0,
        "the new ion gun carries a new module"
    );
    // The first combat drone of two: the last copy leaves.
    apply(&mut g, &["drones.COMBAT_1=1", "drones.ANTI_DRONE=1"]).unwrap();
    assert_eq!(drones(&g), [("COMBAT_1", false), ("ANTI_DRONE", false)]);
    // A cargo box and an augment are replaced in place too.
    apply(&mut g, &["cargo.COMBAT_1=1", "cargo.BEAM_HULL=1"]).unwrap();
    apply(&mut g, &["cargo.REPAIR=0", "cargo.SHOTGUN_2=1"]).unwrap();
    assert_eq!(
        g.world.as_ref().unwrap().cargo,
        ["SHOTGUN_2", "COMBAT_1", "BEAM_HULL"]
    );
    let augments = |g: &SavedGame| g.player_ship.body.as_ref().unwrap().augments.clone();
    let before = augments(&g);
    assert_eq!(before, ["NANO_MEDBAY"]);
    apply(
        &mut g,
        &["augments.SCRAP_COLLECTOR=1", "augments.NANO_MEDBAY=0"],
    )
    .unwrap();
    // Removal staged after the addition: the addition went to the end first.
    assert_eq!(augments(&g), ["SCRAP_COLLECTOR"]);
    apply(&mut g, &["augments.O2_MASKS=1"]).unwrap();
    let mut g2 = g.clone();
    apply(
        &mut g2,
        &["augments.SCRAP_COLLECTOR=0", "augments.NANO_MEDBAY=1"],
    )
    .unwrap();
    assert_eq!(augments(&g2), ["NANO_MEDBAY", "O2_MASKS"]);
    round_trip(&g2, "replace");
}
