//! Installing and removing systems on the real fixture (`ftl_core::installs`):
//! every edit decodes back to the edited game, and the files land in the
//! test tmp dir for the Java oracle (`fe-install-*.sav`, see AGENTS.md).

use ftl_core::installs::{InstallBlock, MAX_SYSTEMS};
use ftl_core::{
    EditError, Field, KnownLayouts, Patch, SaveCodec, SavedGame, Station, StationDirection,
    SystemKind,
};
use ftl_sav::SavCodec;

#[path = "common/layouts.rs"]
mod layouts;

use layouts::{install_layouts, item_layouts, kestrel_system_rooms};

const FORMAT11: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11.sav");

fn game() -> SavedGame {
    SavCodec.decode(FORMAT11, &install_layouts()).unwrap()
}

/// The fixture's Kestrel has five main systems. With Drone Control,
/// Cloaking and Mind Control (and the Battery subsystem) installed through
/// the edit paths it is full: eight main systems, [`MAX_SYSTEMS`].
fn full() -> SavedGame {
    let mut g = game();
    apply(
        &mut g,
        &[
            "systems.drones.installed=1",
            "systems.cloaking.installed=1",
            "systems.mind.installed=1",
            "systems.battery.installed=1",
        ],
    )
    .unwrap();
    g
}

fn apply_in(g: &mut SavedGame, edits: &[&str], l: &KnownLayouts) -> Result<(), EditError> {
    let patch = Patch::new(edits.iter().map(|e| e.parse().unwrap()).collect());
    patch.apply_in(g, l).map(|_| ())
}

fn apply(g: &mut SavedGame, edits: &[&str]) -> Result<(), EditError> {
    apply_in(g, edits, &install_layouts())
}

fn block(r: Result<(), EditError>) -> InstallBlock {
    match r {
        Err(EditError::SystemBlocked { block, .. }) => block,
        other => panic!("expected a blocked install, got {other:?}"),
    }
}

/// Encodes, checks that the bytes decode to the same game, and writes them
/// for the Java oracle.
fn round_trip(g: &SavedGame, name: &str) {
    let out = SavCodec.encode(g);
    let again = SavCodec.decode(&out, &install_layouts()).unwrap();
    assert_eq!(&again, g, "{name} does not decode to the edited game");
    assert!(again.undecoded.is_empty(), "{name}");
    std::fs::write(
        format!("{}/fe-install-{name}.sav", env!("CARGO_TARGET_TMPDIR")),
        &out,
    )
    .unwrap();
}

fn level(g: &SavedGame, kind: SystemKind) -> Option<i32> {
    g.player_ship
        .systems
        .iter()
        .find(|s| s.kind == kind)
        .and_then(|s| s.installed.as_ref())
        .map(|s| s.capacity)
}

fn station(g: &SavedGame, room: usize) -> Station {
    g.player_ship.body.as_ref().unwrap().rooms[room]
        .station
        .unwrap()
}

#[test]
fn a_full_ship_refuses_a_ninth_system() {
    let main = |g: &SavedGame| {
        g.player_ship
            .systems
            .iter()
            .filter(|s| s.installed.is_some() && !s.kind.is_subsystem())
            .count()
    };
    assert_eq!(main(&game()), 5);
    let mut g = full();
    assert_eq!(main(&g), MAX_SYSTEMS);
    round_trip(&g, "full");
    assert_eq!(
        block(apply(&mut g, &["systems.teleporter.installed=1"])),
        InstallBlock::SystemLimit
    );
    assert_eq!(
        Field::SystemInstalled(SystemKind::Teleporter)
            .range_in(&g, &install_layouts())
            .map_err(|_| ()),
        Err(())
    );
}

#[test]
fn a_clone_bay_replaces_the_medbay_also_on_a_full_ship() {
    let mut g = full();
    apply(&mut g, &["systems.clonebay.installed=1"]).unwrap();
    assert_eq!(level(&g, SystemKind::Clonebay), Some(1));
    assert_eq!(level(&g, SystemKind::Medbay), None);
    let c = g.player_ship.clonebay.as_ref().unwrap();
    assert_eq!(
        (c.build_ticks, c.build_ticks_goal, c.doom_ticks),
        (-1000, 12000, -1000)
    );
    round_trip(&g, "clonebay");
    // Back: the medbay replaces the clone bay and its info goes.
    apply(&mut g, &["systems.medbay.installed=1"]).unwrap();
    assert_eq!(level(&g, SystemKind::Medbay), Some(1));
    assert!(g.player_ship.clonebay.is_none());
    // A swap installs the system unpowered; the fixture's Medbay had its bar.
    let mut expected = full();
    expected
        .player_ship
        .systems
        .iter_mut()
        .find(|s| s.kind == SystemKind::Medbay)
        .and_then(|s| s.installed.as_mut())
        .unwrap()
        .power = 0;
    assert_eq!(g, expected);
}

#[test]
fn the_swap_carries_the_level_of_the_replaced_system() {
    let mut g = game();
    apply(
        &mut g,
        &["systems.medbay.level=3", "systems.clonebay.installed=1"],
    )
    .unwrap();
    assert_eq!(level(&g, SystemKind::Clonebay), Some(3));
}

#[test]
fn start_systems_and_the_starting_medbay_stay() {
    let mut g = game();
    assert_eq!(
        block(apply(&mut g, &["systems.shields.installed=0"])),
        InstallBlock::StartingSystem
    );
    assert_eq!(
        block(apply(&mut g, &["systems.medbay.installed=0"])),
        InstallBlock::SwapBack
    );
    apply(&mut g, &["systems.clonebay.installed=1"]).unwrap();
    assert_eq!(
        block(apply(&mut g, &["systems.clonebay.installed=0"])),
        InstallBlock::SwapBack
    );
    let l = install_layouts();
    assert_eq!(
        Field::SystemInstalled(SystemKind::Shields).range_in(&g, &l),
        Ok((1, 1))
    );
    assert_eq!(
        Field::SystemInstalled(SystemKind::Mind).range_in(&g, &l),
        Ok((0, 1))
    );
}

#[test]
fn removing_a_bought_system_frees_a_place_for_another() {
    let mut g = full();
    apply(
        &mut g,
        &[
            "systems.mind.installed=0",
            "systems.cloaking.installed=0",
            "systems.hacking.installed=1",
            "systems.teleporter.installed=1",
        ],
    )
    .unwrap();
    let ship = &g.player_ship;
    assert_eq!(level(&g, SystemKind::Mind), None);
    assert!(ship.cloaking.is_none());
    let ext = ship.extended.as_ref().unwrap();
    assert!(ext.mind.is_none());
    let h = ext.hacking.as_ref().unwrap();
    assert_eq!(
        (h.target_system, h.disruption_ticks_goal, h.drone_pod.kind),
        (None, 4000, ftl_core::DroneKind::Hacking)
    );
    // Hacking is not manned: its room keeps no station.
    assert_eq!(
        station(&g, 12),
        Station {
            square: -1,
            direction: StationDirection::None
        }
    );
    round_trip(&g, "hacking-teleporter");
    // And back: the infos come back as fresh ones.
    apply(
        &mut g,
        &[
            "systems.hacking.installed=0",
            "systems.teleporter.installed=0",
            "systems.mind.installed=1",
            "systems.cloaking.installed=1",
        ],
    )
    .unwrap();
    let ship = &g.player_ship;
    let c = ship.cloaking.as_ref().unwrap();
    assert_eq!((c.cloak_ticks_goal, c.cloak_ticks), (500, 0));
    let m = ship.extended.as_ref().unwrap().mind.as_ref().unwrap();
    assert_eq!(
        (m.mind_control_ticks, m.mind_control_ticks_goal),
        (14000, 14000)
    );
    assert!(ship.extended.as_ref().unwrap().hacking.is_none());
    round_trip(&g, "back");
}

#[test]
fn removing_the_battery_takes_its_bars_from_the_systems() {
    let mut g = full();
    apply(&mut g, &["systems.shields.level=6"]).unwrap();
    let shields = g
        .player_ship
        .systems
        .iter_mut()
        .find(|s| s.kind == SystemKind::Shields)
        .and_then(|s| s.installed.as_mut())
        .unwrap();
    shields.battery_power = 2;
    shields.power = 6;
    apply(&mut g, &["systems.battery.installed=0"]).unwrap();
    assert!(g.player_ship.battery.is_none());
    let s = g.player_ship.systems[0].installed.as_ref().unwrap();
    assert_eq!((s.power, s.battery_power), (4, 0));
    apply(&mut g, &["systems.battery.installed=1"]).unwrap();
    let b = g.player_ship.battery.as_ref().unwrap();
    assert_eq!((b.active, b.used_battery, b.discharge_ticks), (false, 0, 0));
    round_trip(&g, "battery");
}

/// A ship whose blueprint lets it buy sensors (Kestrel C, Stealth A and
/// more list sensors with start="false"): the room gets its station.
#[test]
fn a_manned_system_gets_its_station_from_the_slot_or_ftls_default() {
    let mut rooms = kestrel_system_rooms();
    for r in rooms.iter_mut() {
        if r.kind == SystemKind::Sensors {
            r.start = false;
        }
        if r.kind == SystemKind::Doors {
            r.start = false;
            r.slot = None;
        }
    }
    let l = install_layouts().with_system_rooms(layouts::FIXTURE_SHIP, rooms);
    let mut g = game();
    apply_in(
        &mut g,
        &["systems.sensors.installed=0", "systems.doors.installed=0"],
        &l,
    )
    .unwrap();
    let none = Station {
        square: -1,
        direction: StationDirection::None,
    };
    assert_eq!((station(&g, 3), station(&g, 2)), (none, none));
    apply_in(
        &mut g,
        &["systems.sensors.installed=1", "systems.doors.installed=1"],
        &l,
    )
    .unwrap();
    assert_eq!(
        station(&g, 3),
        Station {
            square: 1,
            direction: StationDirection::Down
        }
    );
    assert_eq!(
        station(&g, 2),
        Station {
            square: 0,
            direction: StationDirection::Up
        }
    );
    assert_eq!(level(&g, SystemKind::Sensors), Some(1));
    round_trip(&g, "sensors");
}

#[test]
fn a_system_without_a_room_or_game_data_is_refused() {
    let mut g = full();
    let no_teleporter: Vec<_> = kestrel_system_rooms()
        .into_iter()
        .filter(|r| r.kind != SystemKind::Teleporter)
        .collect();
    let l = install_layouts().with_system_rooms(layouts::FIXTURE_SHIP, no_teleporter);
    apply(&mut g, &["systems.mind.installed=0"]).unwrap();
    assert_eq!(
        block(apply_in(&mut g, &["systems.teleporter.installed=1"], &l)),
        InstallBlock::NoRoom
    );
    assert_eq!(
        block(apply_in(
            &mut g,
            &["systems.teleporter.installed=1"],
            &item_layouts()
        )),
        InstallBlock::NoGameData
    );
    assert_eq!(
        block(apply(&mut g, &["systems.artillery.installed=1"])),
        InstallBlock::Artillery
    );
}

#[test]
fn the_drone_system_stays_while_drones_are_in_its_slots() {
    // Drone Control is no starting system of the Kestrel; the fixture has
    // neither the system nor drones, so the test installs both.
    let l = install_layouts();
    let mut g = game();
    apply(
        &mut g,
        &[
            "systems.drones.installed=1",
            "drones.COMBAT_1=1",
            "drones.ANTI_DRONE=1",
        ],
    )
    .unwrap();
    assert_eq!(
        block(apply_in(&mut g, &["systems.drones.installed=0"], &l)),
        InstallBlock::ItemsInSlots
    );
    apply_in(
        &mut g,
        &[
            "drones.COMBAT_1=0",
            "drones.ANTI_DRONE=0",
            "systems.drones.installed=0",
        ],
        &l,
    )
    .unwrap();
    assert_eq!(level(&g, SystemKind::Drones), None);
    round_trip(&g, "no-drones");
}

/// Every order of a set of pending installs and removals on a full ship
/// leaves the same game: the editor stages edits in the order of the
/// clicks, and an install staged before the removal that frees its place
/// must not fail. The level of a new system applies after its install.
#[test]
fn installs_and_removals_at_the_limit_apply_in_any_order() {
    let edits = [
        "systems.teleporter.installed=1",
        "systems.hacking.level=2",
        "systems.mind.installed=0",
        "systems.hacking.installed=1",
        "systems.cloaking.installed=0",
    ];
    let mut expected = full();
    apply(
        &mut expected,
        &[edits[2], edits[4], edits[0], edits[3], edits[1]],
    )
    .unwrap();
    assert_eq!(level(&expected, SystemKind::Hacking), Some(2));
    assert_eq!(level(&expected, SystemKind::Teleporter), Some(1));
    let mut orders = vec![vec![]];
    for _ in 0..edits.len() {
        orders = orders
            .into_iter()
            .flat_map(|o: Vec<usize>| {
                (0..edits.len())
                    .filter(|i| !o.contains(i))
                    .map(|i| [o.clone(), vec![i]].concat())
                    .collect::<Vec<_>>()
            })
            .collect();
    }
    assert_eq!(orders.len(), 120);
    for order in orders {
        let picked: Vec<&str> = order.iter().map(|&i| edits[i]).collect();
        let mut g = full();
        let patch = Patch::new(picked.iter().map(|e| e.parse().unwrap()).collect());
        let changes = patch
            .apply_in(&mut g, &install_layouts())
            .unwrap_or_else(|e| panic!("{picked:?}: {e}"));
        assert_eq!(g, expected, "{picked:?}");
        // One change per assignment, in the order of the patch.
        let fields: Vec<String> = changes.iter().map(|c| c.field.to_string()).collect();
        let paths: Vec<&str> = picked
            .iter()
            .map(|e| e.split('=').next().unwrap())
            .collect();
        assert_eq!(fields, paths);
    }
    // A set that is illegal in every order still fails, and the game stays.
    let mut g = full();
    assert_eq!(
        block(apply(
            &mut g,
            &[
                "systems.teleporter.installed=1",
                "systems.hacking.installed=1",
                "systems.mind.installed=0",
            ],
        )),
        InstallBlock::SystemLimit
    );
    assert_eq!(g, full());
}
