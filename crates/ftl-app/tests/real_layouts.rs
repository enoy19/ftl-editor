//! The layout adapter against the local game's `ftl.dat` (`FTL_DAT` or the
//! macOS Steam install), else a skip line.

use std::path::PathBuf;
use std::sync::Arc;

use ftl_app::layouts::GameLayouts;
use ftl_app::{SaveFile, ShipLayouts, assets};

#[path = "../../ftl-sav/tests/common/layouts.rs"]
mod fake;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
);

const DRONE_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ftl-sav/tests/fixtures/continue-1.6.13-format11-sector3-enemy-drone.sav"
);

fn real_data() -> Option<Arc<assets::GameData>> {
    let path = match std::env::var_os("FTL_DAT") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(std::env::var_os("HOME")?).join(
            "Library/Application Support/Steam/steamapps/common/FTL Faster Than Light/FTL.app/Contents/Resources/ftl.dat",
        ),
    };
    if !path.is_file() {
        println!(
            "skip: no ftl.dat at {} (set FTL_DAT to test against the game)",
            path.display()
        );
        return None;
    }
    Some(Arc::new(assets::load(&path).unwrap()))
}

fn real_layouts() -> Option<GameLayouts> {
    real_data().map(GameLayouts::new)
}

#[test]
fn every_ship_of_the_game_has_a_shape() {
    let Some(data) = real_data() else { return };
    let layouts = GameLayouts::new(data.clone());
    let without: Vec<&str> = data
        .ships()
        .map(|s| s.id.as_str())
        .filter(|id| layouts.ship(id).is_none())
        .collect();
    assert!(without.is_empty(), "ships without a shape: {without:?}");
}

#[test]
fn the_real_shape_of_the_fixture_ship_equals_the_fake() {
    let Some(real) = real_layouts() else { return };
    let fake = fake::fixture_layouts();
    for id in [fake::FIXTURE_SHIP, fake::FIXTURE_NEARBY_SHIP] {
        assert_eq!(real.ship(id), fake.ship(id), "{id}");
    }
    for (id, kind) in fake::FIXTURE_DRONES {
        assert_eq!(real.drone_kind(id), Some(kind), "{id}");
    }
    assert_eq!(
        real.system_rooms(fake::FIXTURE_SHIP),
        Some(fake::kestrel_system_rooms())
    );
    assert_eq!(real.drone("DRONE_HACKING"), Some(fake::HACKING_DRONE));
    let second = fake::drone_fixture_layouts();
    let id = fake::DRONE_FIXTURE_NEARBY_SHIP;
    assert_eq!(real.ship(id), second.ship(id), "{id}");
    let (drone, kind) = fake::DRONE_FIXTURE_DRONE;
    assert_eq!(real.drone_kind(drone), Some(kind), "{drone}");
}

/// Every player ship lists Medbay and Clone Bay in the same room: they
/// share one place, so installing one replaces the other.
#[test]
fn medbay_and_clonebay_share_their_room_on_every_player_ship() {
    let Some(data) = real_data() else { return };
    let layouts = GameLayouts::new(data.clone());
    let mut checked = 0;
    for ship in data.ships().filter(|s| s.id.starts_with("PLAYER_SHIP_")) {
        let rooms = layouts.system_rooms(&ship.id).unwrap();
        let room = |k: ftl_core::SystemKind| rooms.iter().find(|r| r.kind == k).map(|r| r.room);
        if let (Some(m), Some(c)) = (
            room(ftl_core::SystemKind::Medbay),
            room(ftl_core::SystemKind::Clonebay),
        ) {
            assert_eq!(m, c, "{}", ship.id);
            checked += 1;
        }
    }
    assert!(checked > 20, "{checked}");
}

#[test]
fn every_drone_of_the_game_has_a_kind() {
    let Some(data) = real_data() else { return };
    let layouts = GameLayouts::new(data.clone());
    let without: Vec<&str> = data
        .drones()
        .map(|d| d.id.as_str())
        .filter(|id| layouts.drone_kind(id).is_none())
        .collect();
    assert!(without.is_empty(), "drones without a kind: {without:?}");
}

#[test]
fn the_fixture_body_decodes_completely_with_the_real_layouts() {
    let Some(real) = real_layouts() else { return };
    let f = SaveFile::open_with_layouts(FIXTURE, Arc::new(real)).unwrap();
    let g = f.game();
    let body = g
        .player_ship
        .body
        .as_ref()
        .expect("the real layouts know the fixture ship");
    assert_eq!(
        (body.layout_id.as_str(), body.rooms.len(), body.doors.len()),
        ("kestral", 17, 26)
    );
    let weapons: Vec<&str> = body.weapons.iter().map(|w| w.id.as_str()).collect();
    assert_eq!(weapons, ["MISSILES_2_PLAYER", "LASER_BURST_3"]);
    assert!(body.augments.is_empty());
    // With the real ftl.dat the codec decodes the whole file.
    assert!(g.world.is_some() && g.situation.is_some());
    assert!(g.undecoded.is_empty());
    assert!(!f.is_dirty());
    f.verify().unwrap();
}

/// The second fixture, with a deployed enemy drone, decodes to its end
/// with the real `ftl.dat` too.
#[test]
fn the_drone_fixture_decodes_completely_with_the_real_layouts() {
    let Some(real) = real_layouts() else { return };
    let f = SaveFile::open_with_layouts(DRONE_FIXTURE, Arc::new(real)).unwrap();
    let g = f.game();
    let s = g
        .situation
        .as_ref()
        .expect("the real layouts know AUTO_ASSAULT");
    let n = s.nearby_ship.as_ref().unwrap();
    let drones: Vec<&str> = n
        .ship
        .body
        .as_ref()
        .unwrap()
        .drones
        .iter()
        .map(|d| d.id.as_str())
        .collect();
    assert_eq!(drones, ["COMBAT_BEAM"]);
    assert!(g.undecoded.is_empty());
    f.verify().unwrap();
}

#[test]
fn the_real_weapon_and_drone_facts_equal_the_fake() {
    let Some(real) = real_layouts() else { return };
    let fake = fake::item_layouts();
    assert_eq!(
        real.slots(fake::FIXTURE_SHIP),
        fake.slots(fake::FIXTURE_SHIP)
    );
    for (id, ..) in fake::FIXTURE_WEAPONS {
        assert_eq!(real.weapon(id), fake.weapon(id), "{id}");
    }
    for (id, ..) in fake::FIXTURE_ITEM_DRONES {
        assert_eq!(real.drone(id), fake.drone(id), "{id}");
    }
}

#[test]
fn the_fixture_edited_with_the_real_blueprints_verifies() {
    let Some(real) = real_layouts() else { return };
    let mut f = SaveFile::open_with_layouts(FIXTURE, Arc::new(real)).unwrap();
    let patch = ftl_app::Patch::new(
        [
            // The Kestrel of the fixture has no drone system and no drones.
            "systems.drones.installed=1",
            "reactor=10",
            "weapons.LASER_BURST_3=0",
            "cargo.LASER_BURST_3=1",
            "weapons.ION_4=1",
            "drones.COMBAT_1=1",
            "drones.DRONE_SHIELD_PLAYER=1",
            "drones.COMBAT_1.armed=1",
        ]
        .map(|a| a.parse().unwrap())
        .to_vec(),
    );
    f.apply(&patch).unwrap();
    f.verify().unwrap();
}

/// Enemy blueprints mostly name no `weaponSlots` and `droneSlots`; FTL then
/// gives 4 of each (`BlueprintManager::ProcessShipBlueprint`).
#[test]
fn a_blueprint_without_slot_tags_has_ftls_four_and_four() {
    let Some(data) = real_data() else { return };
    let layouts = GameLayouts::new(data.clone());
    let bp = data.ship(fake::FIXTURE_NEARBY_SHIP).unwrap();
    assert_eq!((bp.weapon_slots, bp.drone_slots), (None, None));
    let slots = layouts.slots(fake::FIXTURE_NEARBY_SHIP).unwrap();
    assert_eq!((slots.weapons, slots.drones), (4, 4));
    let rooms = layouts.system_rooms(fake::FIXTURE_NEARBY_SHIP).unwrap();
    assert!(rooms.iter().all(|r| r.slot.is_none()), "{rooms:?}");
}
