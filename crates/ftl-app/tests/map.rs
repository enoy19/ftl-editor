//! The sector map of the fixture as the Map page reads it, and the names
//! of store items from the local game's `ftl.dat` (`FTL_DAT` or the macOS
//! Steam install), else a skip line for that part. The fixture opens with the
//! fake layout port, so the world decodes without a local `ftl.dat`.

use std::path::PathBuf;
use std::sync::Arc;

use ftl_app::map::sector_view;
use ftl_app::{SaveFile, assets};
use ftl_core::sector_map::MapPoint;
use ftl_core::{FleetPresence, SavedGame, Store, StoreItem, StoreItemKind, StoreShelf};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
);

#[path = "../../ftl-sav/tests/common/layouts.rs"]
mod fake;

fn open_fixture() -> SaveFile {
    SaveFile::open_with_layouts(FIXTURE, Arc::new(fake::fixture_layouts())).unwrap()
}

fn real_dat() -> Option<PathBuf> {
    let path = match std::env::var_os("FTL_DAT") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(std::env::var_os("HOME")?).join(
            "Library/Application Support/Steam/steamapps/common/FTL Faster Than Light/FTL.app/Contents/Resources/ftl.dat",
        ),
    };
    if path.is_file() {
        Some(path)
    } else {
        println!("skip: no ftl.dat at {}", path.display());
        None
    }
}

#[test]
fn the_fixture_map_has_positions_the_fleet_and_the_current_beacon() {
    let mut file = open_fixture();
    let v = sector_view(file.game(), None, "en").unwrap();
    assert_eq!(v.sector, 1);
    assert_eq!(v.beacons.len(), 21);
    assert_eq!(v.layout_block, None);
    assert_eq!(v.beacons[0].point, Some(MapPoint { x: 69, y: 78 }));
    let current: Vec<_> = v.beacons.iter().filter(|b| b.current).collect();
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].id, 8);
    // The ship keeps its visit at the current beacon.
    assert!(current[0].visited.is_none());
    assert_eq!(current[0].enemy.as_deref(), Some("PIRATE_BRIBER"));
    let b2 = &v.beacons[2];
    let visited = b2.visited.as_ref().unwrap();
    assert_eq!(
        (
            visited.field.path(),
            visited.value,
            visited.min,
            visited.max
        ),
        ("beacons.2.visited".to_owned(), 1, 0, 1)
    );
    // Early in sector 1 the fleet reaches no beacon, and no quest waits.
    assert!(v.beacons.iter().all(|b| b.fleet == FleetPresence::None));
    assert!(v.beacons.iter().all(|b| b.quest.is_none()));
    let fleet = &v.fleet;
    assert_eq!((fleet.offset, fleet.fudge, fleet.radius), (-831, 100, 767));
    let field = fleet.field.as_ref().unwrap();
    assert_eq!((field.min, field.max), (-959, 60));
    assert!(!v.joins.is_empty());

    // Move the fleet to its maximum: beacon 0 (69, 78) lies within 767 px
    // of (60, 100) and turns rebel. Place a quest at beacon 1.
    file.apply(&ftl_app::Patch::new(vec![
        "fleet.offset=60".parse().unwrap(),
    ]))
    .unwrap();
    let mut game: SavedGame = file.game().clone();
    game.world
        .as_mut()
        .unwrap()
        .quest_events
        .push(ftl_core::QuestEvent {
            event_id: "DONOR_MANTIS_CHASE2".into(),
            beacon_id: 1,
        });
    let v = sector_view(&game, None, "en").unwrap();
    assert_eq!(v.fleet.offset, 60);
    assert_eq!(v.beacons[0].fleet, FleetPresence::Rebel);
    assert_eq!(v.beacons[1].quest.as_deref(), Some("DONOR_MANTIS_CHASE2"));
}

#[test]
fn the_last_stand_names_why_the_fleet_stays() {
    let file = open_fixture();
    let mut game: SavedGame = file.game().clone();
    game.world.as_mut().unwrap().sector_number = 7;
    let v = sector_view(&game, None, "en").unwrap();
    assert_eq!(v.fleet.field, None);
    assert_eq!(v.fleet.block, Some("last_stand"));
}

#[test]
fn a_store_lists_its_items_with_names_and_stock() {
    let file = open_fixture();
    let mut game: SavedGame = file.game().clone();
    game.world.as_mut().unwrap().beacons[18].store = Some(Store {
        shelves: vec![StoreShelf {
            kind: StoreItemKind::Weapon,
            slots: [
                Some(StoreItem {
                    id: "LASER_BURST_3".into(),
                    available: 0,
                    extra_data: 0,
                }),
                None,
                None,
            ],
        }],
        fuel: 4,
        missiles: 0,
        drone_parts: 1,
    });
    let data = real_dat().map(|p| assets::load(&p).unwrap());
    let v = sector_view(&game, data.as_ref(), "de").unwrap();
    let store = v.beacons[18].store.as_ref().unwrap();
    let item = store.shelves[0].items[0].as_ref().unwrap();
    assert_eq!(item.stock.field.path(), "beacons.18.store.0.0");
    assert_eq!((item.stock.value, item.stock.max), (0, 1));
    assert!(store.shelves[0].items[1].is_none());
    let (supply, fuel) = &store.supplies[0];
    assert_eq!(supply.id(), "fuel");
    assert_eq!((fuel.value, fuel.max), (4, 99));
    if data.is_some() {
        assert_eq!(item.name.as_deref(), Some("Schnellfeuerlaser II"));
    }
}
