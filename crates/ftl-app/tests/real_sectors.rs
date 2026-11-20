//! The sector tree of the fixture and the sector jump against the local
//! game's `ftl.dat` (`FTL_DAT` or the macOS Steam install), else a skip
//! line. The jumped files land in the test tmp dir for the Java oracle
//! (`fe-sectors-*.sav`, see AGENTS.md).

use std::path::PathBuf;
use std::sync::Arc;

use ftl_app::layouts::GameLayouts;
use ftl_app::map::sector_tree_view;
use ftl_app::{Assignment, Field, Patch, SaveFile, assets};
use ftl_core::sector_map;
use ftl_core::sector_tree::SectorKind;
use ftl_core::{EditError, FleetPresence};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
);

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
        println!(
            "skip: no ftl.dat at {} (set FTL_DAT to test against the game)",
            path.display()
        );
        None
    }
}

fn open() -> Option<(SaveFile, Arc<assets::GameData>)> {
    let path = real_dat()?;
    let data = Arc::new(assets::load(&path).unwrap());
    let layouts = Arc::new(GameLayouts::new(data.clone()));
    Some((SaveFile::open_with_layouts(FIXTURE, layouts).unwrap(), data))
}

fn jump(to: i32) -> Patch {
    Patch::new(vec![Assignment::new(Field::SectorJump, to)])
}

/// The fixture's 19 visited flags map onto the tree of its seed: FTL
/// marked sector 0, the ship is there, and both sectors of column 1 lie
/// ahead. Names come from `text_sectorname.xml` and `text-de.xml`.
#[test]
fn the_fixture_tree_has_nineteen_sectors_and_the_ship_in_the_first() {
    let Some((file, data)) = open() else { return };
    let v = sector_tree_view(file.game(), file.layouts(), Some(&data), "en").unwrap();
    assert_eq!(v.sectors.len(), 19);
    let first = &v.sectors[0];
    assert!(first.current && first.visited && first.reachable);
    assert_eq!(
        (first.name.as_str(), first.kind),
        ("Civilian Sector", SectorKind::Civilian)
    );
    assert_eq!(first.next, [1, 2]);
    assert_eq!(v.sectors[18].description, "FINAL");
    assert_eq!(v.sectors[18].name, "The Last Stand");
    assert_eq!(v.sectors[18].jump_block, Some("jump_last_stand"));
    assert!(v.sectors[1..].iter().all(|s| !s.visited && s.reachable));
    let jump = v.jump.unwrap();
    assert_eq!((jump.value, jump.min, jump.max), (0, 0, 18));
    let de = sector_tree_view(file.game(), file.layouts(), Some(&data), "de").unwrap();
    assert_ne!(de.sectors[18].name, "The Last Stand");
}

/// A jump writes an arrival: the new visited route, the world and header
/// sector, a new layout seed whose beacons the file holds, the ship on
/// FTL's start beacon, the fleet at -959. Both files pass the codec round
/// trip and go to the tmp dir for the Java oracle.
#[test]
fn a_jump_writes_an_arrival_in_the_new_sector() {
    let Some((mut file, _data)) = open() else {
        return;
    };
    let old_seed = file.game().world.as_ref().unwrap().sector_layout_seed;
    let g = file.preview(&jump(2)).unwrap();
    let world = g.world.as_ref().unwrap();
    assert_eq!(g.sector_number, 2);
    assert_eq!(world.sector_number, 1);
    assert_eq!(
        world.sector_visitation.iter().filter(|&&v| v).count(),
        2,
        "{:?}",
        world.sector_visitation
    );
    assert!(world.sector_visitation[0] && world.sector_visitation[2]);
    assert_ne!(world.sector_layout_seed, old_seed);
    let layout = sector_map::layout_of(&g).unwrap();
    assert_eq!(world.current_beacon_id, layout.start as i32);
    assert_eq!(world.rebel_fleet.offset, sector_map::FLEET_START);
    assert_eq!(world.rebel_fleet.fudge, layout.fleet_fudge);
    let start = &world.beacons[layout.start];
    assert_eq!(start.visit_count, 2);
    assert!(start.background.is_some() && start.seen);
    assert!(
        world
            .beacons
            .iter()
            .all(|b| b.store.is_none() && b.enemy.is_none())
    );
    assert!(
        world
            .beacons
            .iter()
            .all(|b| b.fleet_presence == FleetPresence::None)
    );
    assert!(world.quest_events.is_empty());
    let situation = g.situation.as_ref().unwrap();
    assert!(situation.nearby_ship.is_none());

    file.apply(&jump(2)).unwrap();
    let tmp = env!("CARGO_TARGET_TMPDIR");
    std::fs::write(format!("{tmp}/fe-sectors-jump-2.sav"), file.encoded()).unwrap();

    // Further down the tree and back to the first sector.
    file.apply(&jump(9)).unwrap();
    let w = file.game().world.as_ref().unwrap();
    assert_eq!(w.sector_number, 4);
    let route: Vec<usize> = (0..19).filter(|&n| w.sector_visitation[n]).collect();
    assert_eq!(route.len(), 5, "{route:?}");
    let v = sector_tree_view(file.game(), file.layouts(), None, "en").unwrap();
    for pair in route.windows(2) {
        assert!(v.sectors[pair[0]].next.contains(&pair[1]), "{route:?}");
    }
    assert!(v.sectors[9].current);
    std::fs::write(format!("{tmp}/fe-sectors-jump-9.sav"), file.encoded()).unwrap();
    file.apply(&jump(0)).unwrap();
    let w = file.game().world.as_ref().unwrap();
    assert_eq!((w.sector_number, file.game().sector_number), (0, 1));
    assert_eq!(w.sector_visitation.iter().filter(|&&v| v).count(), 1);
}

#[test]
fn the_last_stand_and_waiting_quests_refuse_the_jump() {
    let Some((file, _data)) = open() else { return };
    assert!(matches!(
        file.preview(&jump(18)),
        Err(ftl_app::AppError::Edit(EditError::SectorStays { .. }))
    ));
    let mut g = file.game().clone();
    g.world
        .as_mut()
        .unwrap()
        .distant_quest_events
        .push("QUEST_X".into());
    let err = jump(2).apply_in(&mut g, file.layouts()).unwrap_err();
    assert!(
        err.to_string().contains("quests wait for the next sector"),
        "{err}"
    );
}
