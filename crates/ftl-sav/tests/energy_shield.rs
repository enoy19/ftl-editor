//! The Zoltan shield (`systems.shields.energy`, [`ftl_core::energy_shield`])
//! on both fixtures: the sector-3 save is FTL's own Zoltan shield (the
//! `ENERGY_SHIELD` augment, 5 points of 5), the sector-1 save has none.
//! Built through the edit path, the main fixture holds the same pair FTL
//! wrote, round trips byte for byte and lands in the test tmp dir for the
//! Java oracle (`fe-zoltan-*.sav`).

use ftl_core::{EditError, Field, Patch, SaveCodec, SavedGame, ShieldsInfo, SystemKind};
use ftl_sav::SavCodec;

#[path = "common/layouts.rs"]
mod layouts;

use layouts::{drone_fixture_layouts, fixture_layouts};

const FORMAT11: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11.sav");
const SECTOR3: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11-sector3-enemy-drone.sav");

fn main_fixture() -> SavedGame {
    SavCodec.decode(FORMAT11, &fixture_layouts()).unwrap()
}

fn pair(info: &Option<ShieldsInfo>) -> (i32, i32) {
    let info = info.as_ref().unwrap();
    (info.energy_shield_layers, info.energy_shield_max)
}

fn nearby(game: &SavedGame) -> &ftl_core::Ship {
    &game
        .situation
        .as_ref()
        .unwrap()
        .nearby_ship
        .as_ref()
        .unwrap()
        .ship
}

fn field(path: &str) -> Field {
    path.parse().unwrap()
}

#[test]
fn ftl_writes_five_of_five_for_the_zoltan_shield_augment() {
    let game = SavCodec.decode(SECTOR3, &drone_fixture_layouts()).unwrap();
    let augments = &game.player_ship.body.as_ref().unwrap().augments;
    assert!(augments.iter().any(|a| a == "ENERGY_SHIELD"));
    assert_eq!(pair(&game.player_ship.shields), (5, 5));
    let f = field("systems.shields.energy");
    assert_eq!(
        (f.get(&game).unwrap(), f.range(&game).unwrap()),
        (5, (0, 5))
    );
    // The enemy auto ship has no augment and stores none.
    assert_eq!(pair(&nearby(&game).shields), (0, 0));
}

#[test]
fn the_edit_path_builds_the_pair_ftl_writes_and_round_trips() {
    let mut game = main_fixture();
    assert_eq!(pair(&game.player_ship.shields), (0, 0));
    Patch::new(vec![
        "systems.shields.energy=5".parse().unwrap(),
        "nearby.systems.shields.energy=3".parse().unwrap(),
    ])
    .apply(&mut game)
    .unwrap();
    // AddSuperShield's max of 5, as FTL wrote it into the sector-3 save.
    assert_eq!(pair(&game.player_ship.shields), (5, 5));
    assert_eq!(pair(&nearby(&game).shields), (3, 5));

    let bytes = SavCodec.encode(&game);
    let back = SavCodec.decode(&bytes, &fixture_layouts()).unwrap();
    assert_eq!(back, game);
    assert_eq!(SavCodec.encode(&back), bytes);
    std::fs::write(
        format!("{}/fe-zoltan-energy.sav", env!("CARGO_TARGET_TMPDIR")),
        &bytes,
    )
    .unwrap();

    // Taking the points away keeps the max, as FTL's damage does.
    Patch::new(vec!["systems.shields.energy=0".parse().unwrap()])
        .apply(&mut game)
        .unwrap();
    assert_eq!(pair(&game.player_ship.shields), (0, 5));
}

#[test]
fn the_player_keeps_at_most_five_and_the_enemy_its_own_max() {
    let mut game = main_fixture();
    let err = Patch::new(vec!["systems.shields.energy=6".parse().unwrap()])
        .apply(&mut game)
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "systems.shields.energy: 6 is outside 0..=5"
    );
    // The flagship's PowerSuperShields writes 12 of 12.
    let shields = game
        .situation
        .as_mut()
        .unwrap()
        .nearby_ship
        .as_mut()
        .unwrap()
        .ship
        .shields
        .as_mut()
        .unwrap();
    shields.energy_shield_max = 12;
    let f = field("nearby.systems.shields.energy");
    assert_eq!(f.range(&game).unwrap(), (0, 12));
}

#[test]
fn without_the_shields_system_there_is_no_zoltan_shield() {
    let mut game = main_fixture();
    game.player_ship.systems[SystemKind::Shields as usize].installed = None;
    let err = field("systems.shields.energy").range(&game).unwrap_err();
    assert!(matches!(err, EditError::NotInstalled { .. }), "{err}");
}
