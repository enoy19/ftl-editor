//! Ship name, ship type, difficulty and sector on the real fixture
//! (`ftl_core::identity`): every edit decodes back to the edited game, and
//! the files land in the test tmp dir for the Java oracle
//! (`fe-identity-*.sav`, see AGENTS.md).

use ftl_core::identity::{DifficultyBlock, TypeBlock};
use ftl_core::{
    Difficulty, EditError, Field, ItemSlots, KnownLayouts, Patch, SaveCodec, SavedGame, ShipShape,
    SystemKind,
};
use ftl_sav::SavCodec;

#[path = "common/layouts.rs"]
mod layouts;

use layouts::{FIXTURE_SHIP, install_layouts, kestrel_system_rooms};

const FORMAT11: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11.sav");

/// `PLAYER_SHIP_EASY` of `data/blueprints.xml` (1.6.13): the unused first
/// Kestrel, same layout file `kestral`, same system rooms, slots 4 and 2,
/// image `kestral`. The real data agrees (`crates/ftl-app/tests/real_identity.rs`).
const EASY: &str = "PLAYER_SHIP_EASY";

fn layouts() -> KnownLayouts {
    let l = install_layouts();
    let shape: ShipShape = ftl_core::ShipLayouts::ship(&l, FIXTURE_SHIP).unwrap();
    l.with(EASY, shape)
        .with_system_rooms(EASY, kestrel_system_rooms())
        .with_slots(
            EASY,
            ItemSlots {
                weapons: 4,
                drones: 2,
            },
        )
        .with_gfx(FIXTURE_SHIP, "kestral")
        .with_gfx(EASY, "kestral")
}

fn game() -> SavedGame {
    SavCodec.decode(FORMAT11, &layouts()).unwrap()
}

fn apply(g: &mut SavedGame, edits: &[&str]) -> Result<(), EditError> {
    let patch = Patch::new(edits.iter().map(|e| e.parse().unwrap()).collect());
    patch.apply_in(g, &layouts()).map(|_| ())
}

/// Encodes, checks that the bytes decode to the same game, and writes them
/// for the Java oracle.
fn round_trip(g: &SavedGame, name: &str) {
    let out = SavCodec.encode(g);
    let again = SavCodec.decode(&out, &layouts()).unwrap();
    assert_eq!(&again, g, "{name} does not decode to the edited game");
    assert!(again.undecoded.is_empty(), "{name}");
    std::fs::write(
        format!("{}/fe-identity-{name}.sav", env!("CARGO_TARGET_TMPDIR")),
        &out,
    )
    .unwrap();
}

#[test]
fn the_ship_name_changes_the_ship_and_the_header() {
    let mut g = game();
    assert_eq!(g.player_ship.name, "The Kestrel");
    apply(&mut g, &["ship.name=  Red Dwarf  "]).unwrap();
    assert_eq!(g.player_ship.name, "Red Dwarf");
    assert_eq!(g.player_ship_name, "Red Dwarf");
    round_trip(&g, "name");
}

#[test]
fn the_enemy_ship_name_changes_only_that_ship() {
    let mut g = game();
    apply(&mut g, &["nearby.ship.name=Jelly Bean"]).unwrap();
    assert_eq!(g.ship(ftl_core::ShipId::Nearby).unwrap().name, "Jelly Bean");
    assert_eq!(g.player_ship_name, "The Kestrel");
    round_trip(&g, "nearby-name");
}

#[test]
fn a_ship_name_follows_the_name_rule() {
    let mut g = game();
    for bad in [
        "ship.name=   ",
        "ship.name=a\u{7}b",
        &format!("ship.name={}", "x".repeat(33)),
    ] {
        assert!(
            matches!(apply(&mut g, &[bad]), Err(EditError::BadShipName { .. })),
            "{bad}"
        );
    }
    apply(&mut g, &[&format!("ship.name={}", "x".repeat(32))]).unwrap();
}

#[test]
fn the_difficulty_goes_to_hard_and_back() {
    let mut g = game();
    assert_eq!(g.difficulty, Difficulty::Easy);
    assert_eq!(Field::Difficulty.range(&g).unwrap(), (0, 2));
    apply(&mut g, &["difficulty=2"]).unwrap();
    assert_eq!(g.difficulty, Difficulty::Hard);
    round_trip(&g, "hard");
    apply(&mut g, &["difficulty=1"]).unwrap();
    assert_eq!(g.difficulty, Difficulty::Normal);
    round_trip(&g, "normal");
    assert!(matches!(
        apply(&mut g, &["difficulty=3"]),
        Err(EditError::OutOfRange { max: 2, .. })
    ));
}

#[test]
fn the_difficulty_stays_while_the_flagship_is_on_the_map() {
    let mut g = game();
    g.world.as_mut().unwrap().rebel_fleet.flagship_visible = true;
    assert!(matches!(
        apply(&mut g, &["difficulty=1"]),
        Err(EditError::DifficultyBlocked {
            block: DifficultyBlock::Flagship,
            ..
        })
    ));
}

#[test]
fn the_sector_is_read_only_with_its_reason() {
    let mut g = game();
    assert_eq!(Field::Sector.get(&g).unwrap(), 1);
    assert!(matches!(
        apply(&mut g, &["sector=2"]),
        Err(EditError::SectorFixed { .. })
    ));
}

#[test]
fn a_type_of_the_same_layout_writes_id_header_and_image() {
    let mut g = game();
    apply(&mut g, &[&format!("ship.type={EASY}")]).unwrap();
    assert_eq!(g.player_ship.blueprint_id, EASY);
    assert_eq!(g.player_ship_blueprint_id, EASY);
    assert_eq!(g.player_ship.gfx_base_name, "kestral");
    round_trip(&g, "type");
}

#[test]
fn a_type_change_takes_the_hull_image_of_the_new_blueprint() {
    let mut g = game();
    let l = layouts().with_gfx(EASY, "kestral_easy_test");
    let patch = Patch::new(vec![format!("ship.type={EASY}").parse().unwrap()]);
    patch.apply_in(&mut g, &l).unwrap();
    assert_eq!(g.player_ship.gfx_base_name, "kestral_easy_test");
}

#[test]
fn a_type_change_names_why_it_is_refused() {
    let block = |l: KnownLayouts, edits: &[&str]| -> TypeBlock {
        let mut g = game();
        let patch = Patch::new(edits.iter().map(|e| e.parse().unwrap()).collect());
        match patch.apply_in(&mut g, &l) {
            Err(EditError::TypeBlocked { block, .. }) => block,
            other => panic!("expected a refused type, got {other:?}"),
        }
    };
    let to = format!("ship.type={EASY}");
    // No such blueprint.
    assert_eq!(
        block(layouts(), &["ship.type=PLAYER_SHIP_NOPE"]),
        TypeBlock::UnknownShip
    );
    // Another layout file.
    let mut other: ShipShape = ftl_core::ShipLayouts::ship(&layouts(), EASY).unwrap();
    other.layout_id = "kestral_2".into();
    assert_eq!(
        block(layouts().with(EASY, other), &[&to]),
        TypeBlock::OtherLayout
    );
    // An installed system in another room.
    let mut moved = kestrel_system_rooms();
    let shields = moved
        .iter_mut()
        .find(|r| r.kind == SystemKind::Shields)
        .unwrap();
    shields.room = 1;
    assert_eq!(
        block(layouts().with_system_rooms(EASY, moved), &[&to]),
        TypeBlock::SystemRoom(SystemKind::Shields)
    );
    // A start system the ship lacks.
    let mut start = kestrel_system_rooms();
    start
        .iter_mut()
        .find(|r| r.kind == SystemKind::Drones)
        .unwrap()
        .start = true;
    assert_eq!(
        block(layouts().with_system_rooms(EASY, start), &[&to]),
        TypeBlock::StartSystem(SystemKind::Drones)
    );
    // Two weapons, one slot.
    let narrow = layouts().with_slots(
        EASY,
        ItemSlots {
            weapons: 1,
            drones: 2,
        },
    );
    assert_eq!(block(narrow, &[&to]), TypeBlock::WeaponSlots);
    // The enemy ship.
    assert_eq!(
        block(layouts(), &["nearby.ship.type=JELLY_BUTTON_P_DLC"]),
        TypeBlock::EnemyShip
    );
}
