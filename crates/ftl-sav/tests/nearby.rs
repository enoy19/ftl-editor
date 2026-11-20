//! The nearby ship of the real fixture (`JELLY_BUTTON_P_DLC`) edited
//! through `nearby.<path>`: the same rules as the player ship, every edit
//! decodes back to the edited game and leaves the player ship alone, and
//! the files land in the test tmp dir for the Java oracle
//! (`fe-enemy-*.sav`, see AGENTS.md).

use ftl_core::installs::InstallBlock;
use ftl_core::{
    DronePodInfo, EditError, ItemSlots, KnownLayouts, Patch, SaveCodec, SavedGame, Ship, ShipId,
    Station, StationDirection, SystemKind, SystemRoom,
};
use ftl_sav::SavCodec;

#[path = "common/layouts.rs"]
mod layouts;

use layouts::{FIXTURE_NEARBY_SHIP, install_layouts};

const FORMAT11: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11.sav");

/// The `systemList` of `JELLY_BUTTON_P_DLC` (`data/dlcPirateBlueprints.xml`)
/// as `(kind, power, room, start)`; no entry names a `<slot>`.
const NEARBY_SYSTEMS: [(SystemKind, i32, usize, bool); 6] = [
    (SystemKind::Oxygen, 1, 1, true),
    (SystemKind::Pilot, 1, 4, true),
    (SystemKind::Shields, 2, 5, true),
    (SystemKind::Engines, 1, 3, true),
    (SystemKind::Weapons, 2, 0, true),
    (SystemKind::Mind, 1, 2, false),
];

fn system_rooms(systems: &[(SystemKind, i32, usize, bool)]) -> Vec<SystemRoom> {
    systems
        .iter()
        .map(|&(kind, power, room, start)| SystemRoom {
            kind,
            room,
            power,
            max: None,
            start,
            slot: None,
        })
        .collect()
}

/// [`install_layouts`] with the nearby blueprint: its systems and FTL's
/// default of 4 weapon and 4 drone slots (the blueprint names none). Its
/// two weapons are in [`layouts::FIXTURE_WEAPONS`].
fn nearby_layouts(systems: &[(SystemKind, i32, usize, bool)]) -> KnownLayouts {
    install_layouts()
        .with_system_rooms(FIXTURE_NEARBY_SHIP, system_rooms(systems))
        .with_slots(
            FIXTURE_NEARBY_SHIP,
            ItemSlots {
                weapons: 4,
                drones: 4,
            },
        )
}

fn game() -> SavedGame {
    SavCodec
        .decode(FORMAT11, &nearby_layouts(&NEARBY_SYSTEMS))
        .unwrap()
}

fn apply_in(g: &mut SavedGame, edits: &[&str], l: &KnownLayouts) -> Result<(), EditError> {
    let patch = Patch::new(edits.iter().map(|e| e.parse().unwrap()).collect());
    patch.apply_in(g, l).map(|_| ())
}

fn apply(g: &mut SavedGame, edits: &[&str]) -> Result<(), EditError> {
    apply_in(g, edits, &nearby_layouts(&NEARBY_SYSTEMS))
}

fn near(g: &SavedGame) -> &Ship {
    g.ship(ShipId::Nearby).unwrap()
}

/// Encodes, checks that the bytes decode to the same game, and writes them
/// for the Java oracle.
fn round_trip(g: &SavedGame, name: &str) {
    let out = SavCodec.encode(g);
    let again = SavCodec
        .decode(&out, &nearby_layouts(&NEARBY_SYSTEMS))
        .unwrap();
    assert_eq!(&again, g, "{name} does not decode to the edited game");
    assert!(again.undecoded.is_empty(), "{name}");
    std::fs::write(
        format!("{}/fe-enemy-{name}.sav", env!("CARGO_TARGET_TMPDIR")),
        &out,
    )
    .unwrap();
}

fn level(ship: &Ship, kind: SystemKind) -> Option<(i32, i32)> {
    ship.systems
        .iter()
        .find(|s| s.kind == kind)
        .and_then(|s| s.installed.as_ref())
        .map(|s| (s.capacity, s.power))
}

fn block(r: Result<(), EditError>) -> InstallBlock {
    match r {
        Err(EditError::SystemBlocked { block, .. }) => block,
        other => panic!("expected a blocked install, got {other:?}"),
    }
}

#[test]
fn the_fixture_has_a_decoded_nearby_ship_with_crew_systems_and_weapons() {
    let g = game();
    assert!(g.undecoded.is_empty());
    let n = near(&g);
    assert_eq!(n.blueprint_id, FIXTURE_NEARBY_SHIP);
    assert_eq!((n.resources.hull, n.reactor_power, n.crew.len()), (8, 7, 2));
    let body = n.body.as_ref().unwrap();
    assert_eq!(body.rooms.len(), 6);
    assert_eq!(
        body.weapons
            .iter()
            .map(|w| w.id.as_str())
            .collect::<Vec<_>>(),
        ["LASER_BURST_1", "BEAM_1"]
    );
    assert_eq!(body.augments, ["SLUG_GEL"]);
}

#[test]
fn hull_crew_reactor_levels_fires_and_breaches_edit_the_nearby_ship_only() {
    let before = game();
    let mut g = game();
    apply(
        &mut g,
        &[
            "nearby.ship.hull=5",
            "nearby.ship.missiles=3",
            "nearby.crew.0.health=40",
            "nearby.crew.1.skills.pilot=26",
            "nearby.systems.shields.level=8",
            "nearby.reactor=4",
            "nearby.fires.2.0=100",
            "nearby.breaches.2.1=100",
            "nearby.augments.SCRAP_COLLECTOR=1",
        ],
    )
    .unwrap();
    let n = near(&g);
    assert_eq!(
        (
            n.resources.hull,
            n.resources.missiles,
            n.crew[0].health,
            n.crew[1].skills.pilot.points
        ),
        (5, 3, 40, 26)
    );
    // The reactor of 4 keeps the first systems of the file powered and
    // takes the bars of the later ones (shields 2, engines 1, oxygen 1).
    assert_eq!(n.reactor_power, 4);
    assert_eq!(level(n, SystemKind::Shields), Some((8, 2)));
    assert_eq!(level(n, SystemKind::Engines), Some((1, 1)));
    assert_eq!(level(n, SystemKind::Oxygen), Some((1, 1)));
    assert_eq!(level(n, SystemKind::Weapons), Some((2, 0)));
    let body = n.body.as_ref().unwrap();
    assert_eq!(body.rooms[0].squares[0].fire_health, 100);
    assert_eq!(
        body.breaches
            .iter()
            .map(|b| (b.x, b.y, b.health))
            .collect::<Vec<_>>(),
        [(2, 1, 100)]
    );
    assert_eq!(body.augments, ["SLUG_GEL", "SCRAP_COLLECTOR"]);
    assert_eq!(g.player_ship, before.player_ship);
    round_trip(&g, "ship");

    // The same refusals as on the player ship, naming the nearby ship.
    let err = apply(&mut g, &["nearby.fires.0.0=100"]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "nearby.fires.0.0: no room of the nearby ship covers square 0,0"
    );
}

#[test]
fn weapons_and_drones_of_the_nearby_ship_use_its_slots_and_its_space() {
    let mut g = game();
    apply(
        &mut g,
        &[
            "nearby.systems.weapons.level=4",
            "nearby.weapons.LASER_BURST_1=0",
            "nearby.weapons.ION_4=1",
            "nearby.weapons.BEAM_1.armed=0",
            "nearby.weapons.ION_4.armed=1",
        ],
    )
    .unwrap();
    let n = near(&g);
    let body = n.body.as_ref().unwrap();
    assert_eq!(
        body.weapons
            .iter()
            .map(|w| (w.id.as_str(), w.armed))
            .collect::<Vec<_>>(),
        // Ion Blast II takes the slot Burst I freed.
        [("ION_4", true), ("BEAM_1", false)]
    );
    // Burst I and the Mini Beam gave 1 + 1 bars back, Ion Blast II took 3.
    assert_eq!(level(n, SystemKind::Weapons), Some((4, 3)));
    let modules = n
        .extended
        .as_ref()
        .unwrap()
        .weapon_modules
        .as_ref()
        .unwrap();
    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].cooldown_ticks_goal, 4000);
    round_trip(&g, "weapons");

    // Four slots, FTL's default for a blueprint without `weaponSlots`.
    let err = apply(&mut g, &["nearby.weapons.ION_4=4"]).unwrap_err();
    assert!(matches!(err, EditError::OutOfRange { max: 3, .. }), "{err}");

    // A drone needs a drone system, which this ship lacks.
    let err = apply(&mut g, &["nearby.drones.COMBAT_1=1"]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "nearby.drones.COMBAT_1: the nearby ship has no drones system"
    );
    // The cargo hold is the player's.
    assert!("nearby.cargo.ION_4".parse::<ftl_core::Field>().is_err());
}

/// A drone the nearby ship builds starts in its own space:
/// `SpaceDrone::SpaceDrone` stores `iShipId` (1) as the current space.
#[test]
fn a_new_drone_on_the_nearby_ship_starts_in_its_space() {
    let mut g = game();
    let drones = near(&g).systems[SystemKind::Drones as usize].clone();
    assert!(drones.installed.is_none());
    // Give the ship the drone system of the player to hold a drone; the
    // fixture's Kestrel gets one first.
    apply(&mut g, &["systems.drones.installed=1"]).unwrap();
    let player_drones = g.player_ship.systems[SystemKind::Drones as usize].clone();
    g.ship_mut(ShipId::Nearby).unwrap().systems[SystemKind::Drones as usize] = player_drones;
    apply(&mut g, &["nearby.drones.COMBAT_1=1"]).unwrap();
    let pod = near(&g).extended.as_ref().unwrap().drones[0]
        .pod
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!((pod.current_space, pod.destination_space), (1, -1));
    assert!(matches!(pod.info, DronePodInfo::ZigZag { .. }));
    round_trip(&g, "drone");
}

#[test]
fn systems_install_and_remove_on_the_nearby_ship_by_its_blueprint() {
    let mut g = game();
    // Mind Control is the one system the blueprint lists with start="false".
    apply(&mut g, &["nearby.systems.mind.installed=1"]).unwrap();
    assert_eq!(level(near(&g), SystemKind::Mind), Some((1, 0)));
    round_trip(&g, "install");
    apply(&mut g, &["nearby.systems.mind.installed=0"]).unwrap();
    assert_eq!(level(near(&g), SystemKind::Mind), None);
    assert_eq!(
        block(apply(&mut g, &["nearby.systems.shields.installed=0"])),
        InstallBlock::StartingSystem
    );
    assert_eq!(
        block(apply(&mut g, &["nearby.systems.cloaking.installed=1"])),
        InstallBlock::NoRoom
    );
}

/// On a ship that is not the player's, `ShipManager::AddSystem` gives the
/// pilot square 0 facing up and every other manned system a random wall
/// square (`Ship::ChoosePrimarySlot`); no enemy blueprint names a `<slot>`.
#[test]
fn a_manned_system_on_the_nearby_ship_needs_a_station_ftl_does_not_pick_at_random() {
    let mut systems = NEARBY_SYSTEMS;
    systems[1].3 = false; // pilot can go
    let mut with_doors = systems.to_vec();
    with_doors.push((SystemKind::Doors, 1, 2, false));
    let l = nearby_layouts(&with_doors);
    let mut g = game();
    apply_in(&mut g, &["nearby.systems.pilot.installed=0"], &l).unwrap();
    let station = |g: &SavedGame| near(g).body.as_ref().unwrap().rooms[4].station;
    assert_eq!(
        station(&g),
        Some(Station {
            square: -1,
            direction: StationDirection::None
        })
    );
    apply_in(&mut g, &["nearby.systems.pilot.installed=1"], &l).unwrap();
    assert_eq!(
        station(&g),
        Some(Station {
            square: 0,
            direction: StationDirection::Up
        })
    );
    let r = apply_in(&mut g, &["nearby.systems.doors.installed=1"], &l);
    assert_eq!(block(r), InstallBlock::NoStation);
}

#[test]
fn without_a_ship_at_the_beacon_a_nearby_path_names_why() {
    let mut g = game();
    g.situation.as_mut().unwrap().nearby_ship = None;
    let err = apply(&mut g, &["nearby.ship.hull=5"]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "nearby.ship.hull: no ship is at the current beacon"
    );
    let mut g = SavCodec.decode(FORMAT11, &layouts::ship_layouts()).unwrap();
    assert!(g.situation.is_none());
    let err = apply(&mut g, &["nearby.reactor=5"]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "nearby.reactor: the nearby ship is not decoded; it needs the ship layouts from ftl.dat"
    );
}

#[test]
fn damage_and_ion_edit_both_ships_and_round_trip() {
    let mut g = game();
    apply(
        &mut g,
        &[
            "systems.shields.damage=2",
            "systems.engines.ion=2",
            "nearby.systems.shields.damage=1",
            "nearby.systems.weapons.ion=3",
        ],
    )
    .unwrap();
    let state = |ship: &Ship, kind: SystemKind| {
        let s = ship
            .systems
            .iter()
            .find(|s| s.kind == kind)
            .and_then(|s| s.installed.as_ref())
            .unwrap();
        (s.capacity, s.power, s.damaged_bars, s.ionized_bars)
    };
    // The player's shields (level 2) lose their power with both bars;
    // the enemy's shields keep one of two bars.
    assert_eq!(state(&g.player_ship, SystemKind::Shields), (2, 0, 2, 0));
    assert_eq!(state(near(&g), SystemKind::Shields).2, 1);
    assert_eq!(state(near(&g), SystemKind::Weapons).3, 3);
    assert_eq!(state(&g.player_ship, SystemKind::Engines).3, 2);
    round_trip(&g, "damage");
}

#[test]
fn room_oxygen_edits_the_room_of_the_nearby_ship_only() {
    let before = game();
    let mut g = game();
    apply(
        &mut g,
        &["nearby.rooms.3.oxygen=10", "nearby.rooms.0.oxygen=0"],
    )
    .unwrap();
    let oxygen = |s: &Ship| -> Vec<i32> {
        s.body
            .as_ref()
            .unwrap()
            .rooms
            .iter()
            .map(|r| r.oxygen)
            .collect()
    };
    assert_eq!(oxygen(near(&g)), [0, 100, 100, 10, 100, 100]);
    assert_eq!(g.player_ship, before.player_ship);
    round_trip(&g, "oxygen");
    let err = apply(&mut g, &["nearby.rooms.6.oxygen=50"]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "nearby.rooms.6.oxygen: the nearby ship has 6 rooms (0 to 5)"
    );
}
