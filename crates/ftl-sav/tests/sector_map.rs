//! The sector map of the real fixture: the beacon positions FTL builds from
//! the layout seed, the fleet flags they explain, and the map edits
//! (`fleet.offset`, `beacons.<n>.visited`, the store stock). Every edited
//! file decodes back to the edited game and lands in the test tmp dir for
//! the Java oracle (`fe-map-*.sav`, see AGENTS.md).

use ftl_core::beacons::{MoveBlock, StoreSupply};
use ftl_core::sector_map::{self, FLEET_START, NoLayout, fleet_covers};
use ftl_core::{
    Assignment, EditError, Field, FleetPresence, Patch, QuestEvent, SaveCodec, SavedGame, Store,
    StoreItem, StoreItemKind, StoreShelf,
};
use ftl_sav::SavCodec;

#[path = "common/layouts.rs"]
mod layouts;

const FORMAT11: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11.sav");

/// The second fixture: sector 3, written by FTL with a deployed enemy drone.
const DRONE_FIXTURE: &[u8] =
    include_bytes!("fixtures/continue-1.6.13-format11-sector3-enemy-drone.sav");

fn fixture() -> SavedGame {
    SavCodec.decode(FORMAT11, &layouts::ship_layouts()).unwrap()
}

/// Encodes, checks that the bytes decode to the same game, and writes them
/// for the Java oracle.
fn round_trip(g: &SavedGame, name: &str) {
    let out = SavCodec.encode(g);
    let again = SavCodec.decode(&out, &layouts::ship_layouts()).unwrap();
    assert_eq!(&again, g, "{name} does not decode to the edited game");
    // The nearby ship needs layouts this test does not pass; the world
    // before it decodes either way.
    assert!(again.world.is_some(), "{name}");
    std::fs::write(
        format!("{}/fe-map-{name}.sav", env!("CARGO_TARGET_TMPDIR")),
        &out,
    )
    .unwrap();
}

fn set(g: &mut SavedGame, path: &str, value: i32) -> Result<(), EditError> {
    let field: Field = path.parse()?;
    Patch::new(vec![Assignment::new(field, value)])
        .apply(g)
        .map(|_| ())
}

fn presence(g: &SavedGame) -> Vec<FleetPresence> {
    let world = g.world.as_ref().unwrap();
    world.beacons.iter().map(|b| b.fleet_presence).collect()
}

/// The layout seed builds the 21 beacons of the file and the fudge it
/// stores, and the circle around (offset, fudge) explains every fleet flag
/// FTL wrote: the positions are FTL's. In both fixtures (sector 1, offset
/// -831; sector 3, offset -895, fudge 103) the circle covers no beacon and
/// FTL flagged none, so a flag FTL set is proven only by the first fixture
/// (sector 7, replaced on 2026-11-19).
#[test]
fn the_fixture_layout_explains_every_fleet_flag() {
    for bytes in [FORMAT11, DRONE_FIXTURE] {
        let g = SavCodec.decode(bytes, &layouts::ship_layouts()).unwrap();
        let world = g.world.as_ref().unwrap();
        let layout = sector_map::layout_of(&g).unwrap();
        assert_eq!(layout.beacons.len(), 21);
        assert_eq!(layout.fleet_fudge, world.rebel_fleet.fudge);
        let fleet = &world.rebel_fleet;
        for (n, (b, &p)) in world.beacons.iter().zip(&layout.beacons).enumerate() {
            let covered = fleet_covers(p, fleet.offset, fleet.fudge);
            assert_eq!(
                b.fleet_presence == FleetPresence::Rebel,
                covered,
                "sector {}, beacon {n} at {p:?}",
                g.sector_number
            );
        }
    }
}

/// The second fixture's layout seed builds FTL's fudge and the beacons FTL
/// visited: the current beacon 6 and beacon 2 (`visit_count` 2).
#[test]
fn the_sector_3_layout_matches_the_second_fixture() {
    let g = SavCodec
        .decode(DRONE_FIXTURE, &layouts::ship_layouts())
        .unwrap();
    let world = g.world.as_ref().unwrap();
    assert_eq!(
        (
            world.sector_layout_seed,
            world.rebel_fleet.offset,
            world.rebel_fleet.fudge
        ),
        (1169293550, -895, 103)
    );
    let layout = sector_map::layout_of(&g).unwrap();
    assert_eq!(layout.fleet_fudge, 103);
    assert_eq!(world.current_beacon_id, 6);
    let visited: Vec<usize> = (0..21)
        .filter(|&n| world.beacons[n].visit_count > 0)
        .collect();
    assert_eq!(visited, [2, 6]);
}

/// FTL puts an arriving ship on the beacon of cell (0, `r % 4`): the start
/// beacon of each fixture's sector stands in the first column and FTL
/// recorded a visit there (sector 3: beacon 2, visited twice).
#[test]
fn the_start_beacon_is_a_visited_beacon_of_the_first_column() {
    for (bytes, expected) in [(FORMAT11, None), (DRONE_FIXTURE, Some(2))] {
        let g = SavCodec.decode(bytes, &layouts::ship_layouts()).unwrap();
        let world = g.world.as_ref().unwrap();
        let layout = sector_map::layout_of(&g).unwrap();
        let start = layout.start;
        if let Some(e) = expected {
            assert_eq!(start, e);
        }
        assert!(layout.beacons[start].x < 110, "{:?}", layout.beacons[start]);
        assert!(world.beacons[start].visit_count > 0, "start {start}");
        let first_column = (0..world.beacons.len()).filter(|&n| layout.beacons[n].x < 110);
        let visited: Vec<usize> = first_column
            .filter(|&n| world.beacons[n].visit_count > 0)
            .collect();
        assert_eq!(visited, [start], "the only visited beacon of column 0");
    }
}

#[test]
fn pushing_the_fleet_back_clears_the_flags_the_circle_leaves() {
    let mut g = fixture();
    // The fixture's fleet covers no beacon yet: move it forward first.
    set(&mut g, "fleet.offset", -100).unwrap();
    let rebel = |g: &SavedGame| {
        presence(g)
            .iter()
            .filter(|&&p| p == FleetPresence::Rebel)
            .count()
    };
    let ahead = rebel(&g);
    assert!(ahead > 0);
    set(&mut g, "fleet.offset", -500).unwrap();
    let layout = sector_map::layout_of(&g).unwrap();
    let world = g.world.as_ref().unwrap();
    assert_eq!(world.rebel_fleet.offset, -500);
    for (b, &p) in world.beacons.iter().zip(&layout.beacons) {
        assert_eq!(
            b.fleet_presence == FleetPresence::Rebel,
            fleet_covers(p, -500, world.rebel_fleet.fudge)
        );
    }
    assert!(rebel(&g) < ahead, "{} of {ahead} flags left", rebel(&g));
    round_trip(&g, "fleet-pushed-back");

    set(&mut g, "fleet.offset", FLEET_START).unwrap();
    assert!(presence(&g).iter().all(|&p| p == FleetPresence::None));
    round_trip(&g, "fleet-at-start");
}

#[test]
fn the_fleet_offset_stays_within_a_new_sector_and_the_stop() {
    let g = fixture();
    let field: Field = "fleet.offset".parse().unwrap();
    assert_eq!(field.range(&g).unwrap(), (-959, 60));
    let mut g = g;
    assert!(matches!(
        set(&mut g, "fleet.offset", 61),
        Err(EditError::OutOfRange { .. })
    ));
}

#[test]
fn the_last_stand_and_the_native_rng_refuse_the_fleet() {
    let mut g = fixture();
    g.world.as_mut().unwrap().sector_number = 7;
    assert!(matches!(
        set(&mut g, "fleet.offset", -500),
        Err(EditError::LastStand { .. })
    ));
    let mut g = fixture();
    g.random_native = true;
    assert!(matches!(
        set(&mut g, "fleet.offset", -500),
        Err(EditError::NoMapLayout {
            reason: NoLayout::NativeRandom,
            ..
        })
    ));
}

#[test]
fn a_visit_can_be_taken_away_but_not_from_the_current_beacon() {
    let mut g = fixture();
    let field: Field = "beacons.2.visited".parse().unwrap();
    assert_eq!(field.range(&g).unwrap(), (0, 1));
    let unvisited: Field = "beacons.0.visited".parse().unwrap();
    assert_eq!(unvisited.range(&g).unwrap(), (0, 0));
    assert!(matches!(
        set(&mut g, "beacons.8.visited", 0),
        Err(EditError::CurrentBeacon { .. })
    ));
    set(&mut g, "beacons.2.visited", 0).unwrap();
    let b = &g.world.as_ref().unwrap().beacons[2];
    assert_eq!((b.visit_count, b.background.is_none()), (0, true));
    round_trip(&g, "beacon-unvisited");
}

/// The fixture has no store; this one follows `Store::LoadStore`: a
/// weapon shelf with a sold item and an empty last slot, a crew shelf. The
/// empty slot is the last one, because Java's `readStoreShelf` drops an
/// empty slot and moves the next item into it, so its round trip of an
/// empty middle slot differs.
fn with_store(mut g: SavedGame) -> SavedGame {
    let item = |id: &str, available| {
        Some(StoreItem {
            id: id.to_owned(),
            available,
            extra_data: 0,
        })
    };
    let b = &mut g.world.as_mut().unwrap().beacons[18];
    b.store = Some(Store {
        shelves: vec![
            StoreShelf {
                kind: StoreItemKind::Weapon,
                slots: [item("LASER_BURST_3", 0), item("ION_1", 1), None],
            },
            StoreShelf {
                kind: StoreItemKind::Crew,
                slots: [item("engi", 1), item("human", 1), item("rock", 1)],
            },
        ],
        fuel: 3,
        missiles: 2,
        drone_parts: 0,
    });
    g
}

#[test]
fn a_store_restocks_and_sells_more_supplies() {
    let mut g = with_store(fixture());
    round_trip(&g, "store");
    let sold: Field = "beacons.18.store.0.0".parse().unwrap();
    assert_eq!(sold.range(&g).unwrap(), (0, 1));
    set(&mut g, "beacons.18.store.0.0", 1).unwrap();
    set(&mut g, "beacons.18.store.fuel", 20).unwrap();
    set(&mut g, "beacons.18.store.drone_parts", 99).unwrap();
    let store = g.world.as_ref().unwrap().beacons[18].store.clone().unwrap();
    assert_eq!(store.shelves[0].slots[0].as_ref().unwrap().available, 1);
    assert_eq!(StoreSupply::Fuel.of(&store), 20);
    assert_eq!(store.drone_parts, 99);
    round_trip(&g, "store-restocked");

    assert!(matches!(
        set(&mut g, "beacons.18.store.0.2", 1),
        Err(EditError::NoStoreItem { .. })
    ));
    assert!(matches!(
        set(&mut g, "beacons.17.store.fuel", 1),
        Err(EditError::NoStore { .. })
    ));
    assert!(matches!(
        set(&mut g, "beacons.18.store.missiles", 100),
        Err(EditError::OutOfRange { .. })
    ));
}

#[test]
fn map_paths_parse_back_to_themselves() {
    for path in [
        "fleet.offset",
        "beacons.3.visited",
        "beacons.3.store.1.2",
        "beacons.3.store.drone_parts",
    ] {
        let field: Field = path.parse().unwrap();
        assert_eq!(field.path(), path);
        assert!(!field.is_ship_field());
    }
    assert!("beacons.3.store.0.3".parse::<Field>().is_err());
    assert!("nearby.fleet.offset".parse::<Field>().is_err());
}

/// The fixture with both ships decoded (the situation) and the weapon and
/// drone blueprints a store slot needs.
fn full() -> SavedGame {
    SavCodec.decode(FORMAT11, &layouts::item_layouts()).unwrap()
}

fn edit(g: &mut SavedGame, edits: &[&str]) -> Result<(), EditError> {
    let patch = Patch::new(edits.iter().map(|e| e.parse().unwrap()).collect());
    patch.apply_in(g, &layouts::item_layouts()).map(|_| ())
}

/// Encodes with the full layouts, so the situation decodes back too.
fn round_trip_full(g: &SavedGame, name: &str) {
    let out = SavCodec.encode(g);
    let again = SavCodec.decode(&out, &layouts::item_layouts()).unwrap();
    assert_eq!(&again, g, "{name} does not decode to the edited game");
    assert!(
        again.situation.is_some() && again.undecoded.is_empty(),
        "{name}"
    );
    std::fs::write(
        format!("{}/fe-maprework-{name}.sav", env!("CARGO_TARGET_TMPDIR")),
        &out,
    )
    .unwrap();
}

/// The ship leaves the pirate at beacon 8 for beacon 2, which it visited
/// before: FTL's arrival without a ship (`beacons` module doc).
#[test]
fn the_ship_moves_to_another_beacon_and_leaves_the_fight() {
    let mut g = full();
    assert!(g.situation.as_ref().unwrap().nearby_ship.is_some());
    edit(&mut g, &["beacons.current=2"]).unwrap();
    let w = g.world.as_ref().unwrap();
    assert_eq!(w.current_beacon_id, 2);
    assert_eq!(w.beacons[2].visit_count, 2, "a return clears the event");
    // The old beacon keeps its record: the pirate waits there.
    assert_eq!(w.beacons[8].visit_count, 1);
    assert!(w.beacons[8].enemy.is_some());
    let s = g.situation.as_ref().unwrap();
    assert!(s.nearby_ship.is_none() && s.projectiles.is_empty());
    assert_eq!(s.ship_event_present, 0);
    let e = s.encounter.as_ref().unwrap();
    assert!(e.last_event_id.is_empty() && e.text.is_empty() && e.choices.is_empty());
    round_trip_full(&g, "moved");
}

#[test]
fn a_beacon_never_visited_takes_the_background_of_the_one_left() {
    let mut g = full();
    let left = g.world.as_ref().unwrap().beacons[8].background.clone();
    edit(&mut g, &["beacons.current=0"]).unwrap();
    let b = &g.world.as_ref().unwrap().beacons[0];
    assert_eq!((b.visit_count, b.seen), (2, true));
    assert_eq!(b.background, left);
    round_trip_full(&g, "moved-unvisited");
}

#[test]
fn the_ship_stays_with_a_boarder_at_a_fleet_or_a_quest_beacon() {
    let mut g = full();
    g.player_ship.crew[0].player_controlled = false;
    assert!(matches!(
        edit(&mut g, &["beacons.current=2"]),
        Err(EditError::ShipStays {
            reason: MoveBlock::CrewAboard,
            ..
        })
    ));
    let mut g = full();
    edit(&mut g, &["fleet.offset=60"]).unwrap();
    assert_eq!(
        g.world.as_ref().unwrap().beacons[0].fleet_presence,
        FleetPresence::Rebel
    );
    assert!(matches!(
        edit(&mut g, &["beacons.current=0"]),
        Err(EditError::ShipStays {
            reason: MoveBlock::FleetBeacon,
            ..
        })
    ));
    let mut g = full();
    g.world.as_mut().unwrap().quest_events.push(QuestEvent {
        event_id: "DONOR_MANTIS_CHASE2".into(),
        beacon_id: 3,
    });
    assert!(matches!(
        edit(&mut g, &["beacons.current=3"]),
        Err(EditError::ShipStays {
            reason: MoveBlock::QuestBeacon,
            ..
        })
    ));
    // Without the nearby ship's layout the situation stays undecoded.
    let mut g = fixture();
    assert!(matches!(
        set(&mut g, "beacons.current", 2),
        Err(EditError::ShipStays {
            reason: MoveBlock::NotDecoded,
            ..
        })
    ));
}

/// A beacon becomes a store through its edit path; its stock and items
/// are edited like a store FTL wrote. No empty slot, so the Java oracle
/// reads it back (pitfall of the empty middle slot).
#[test]
fn a_beacon_becomes_a_store_with_editable_stock_and_items() {
    let mut g = full();
    edit(
        &mut g,
        &[
            "beacons.3.store=1",
            "beacons.3.store.0.1=0",
            "beacons.3.store.fuel=12",
            "beacons.3.store.0.0.item=LASER_BURST_3",
            "beacons.3.store.1.2.item=COMBAT_1",
        ],
    )
    .unwrap();
    let store = g.world.as_ref().unwrap().beacons[3].store.clone().unwrap();
    assert_eq!(store.shelves.len(), 3);
    let ids: Vec<_> = store.shelves[0]
        .slots
        .iter()
        .map(|i| i.as_ref().unwrap().id.as_str())
        .collect();
    assert_eq!(ids, ["LASER_BURST_3", "MISSILES_1", "ION_2"]);
    assert_eq!(store.shelves[0].slots[1].as_ref().unwrap().available, 0);
    assert_eq!(store.shelves[1].slots[2].as_ref().unwrap().id, "COMBAT_1");
    assert_eq!(store.fuel, 12);
    round_trip_full(&g, "new-store");

    // A weapon does not go on the drone shelf; an unknown id nowhere.
    for bad in [
        "beacons.3.store.1.0.item=LASER_BURST_3",
        "beacons.3.store.0.0.item=NOPE",
    ] {
        assert!(
            matches!(edit(&mut g, &[bad]), Err(EditError::NotForShelf { .. })),
            "{bad}"
        );
    }
    edit(&mut g, &["beacons.3.store=0"]).unwrap();
    assert!(g.world.as_ref().unwrap().beacons[3].store.is_none());
}

#[test]
fn the_new_map_paths_parse_back_to_themselves() {
    for path in [
        "beacons.current",
        "beacons.4.store",
        "beacons.4.store.2.1.item",
    ] {
        let field: Field = path.parse().unwrap();
        assert_eq!(field.path(), path);
        assert!(!field.is_ship_field());
    }
    assert!(
        "beacons.4.store.0.0.item"
            .parse::<Field>()
            .unwrap()
            .is_text()
    );
}
