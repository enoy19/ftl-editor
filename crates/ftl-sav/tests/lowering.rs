//! Lowering on a file FTL wrote: the reactor below the bars in use and a
//! system level below its power. The edited game keeps FTL's invariants
//! (power within the level, reactor bars within the reactor) and encodes to
//! bytes that decode to it.

use ftl_core::{Patch, SaveCodec, SavedGame, SystemKind, reactor_power_in_use};
use ftl_sav::SavCodec;

#[path = "common/layouts.rs"]
mod layouts;

const FORMAT11: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11.sav");

fn fixture() -> SavedGame {
    SavCodec
        .decode(FORMAT11, &layouts::fixture_layouts())
        .unwrap()
}

fn power(game: &SavedGame, kind: SystemKind) -> (i32, i32) {
    let s = game
        .player_ship
        .systems
        .iter()
        .find(|s| s.kind == kind)
        .and_then(|s| s.installed.as_ref())
        .unwrap();
    (s.capacity, s.power)
}

#[test]
fn a_lowered_reactor_and_system_round_trip() {
    let mut game = fixture();
    assert_eq!(
        (
            game.player_ship.reactor_power,
            reactor_power_in_use(&game.player_ship)
        ),
        (8, 8)
    );
    let shields_before = power(&game, SystemKind::Shields);
    Patch::new(vec![
        "systems.weapons.level=1".parse().unwrap(),
        "reactor=5".parse().unwrap(),
    ])
    .apply(&mut game)
    .unwrap();
    let (level, weapons) = power(&game, SystemKind::Weapons);
    assert_eq!(level, 1);
    assert!(weapons <= 1, "weapons draw {weapons} bars at level 1");
    assert_eq!(game.player_ship.reactor_power, 5);
    assert_eq!(reactor_power_in_use(&game.player_ship), 5);
    // Shields come first in the file and keep their power.
    assert_eq!(power(&game, SystemKind::Shields), shields_before);
    for s in game
        .player_ship
        .systems
        .iter()
        .filter_map(|s| s.installed.as_ref())
    {
        assert!(s.power <= s.capacity - s.damaged_bars);
    }

    let bytes = SavCodec.encode(&game);
    let again = SavCodec
        .decode(&bytes, &layouts::fixture_layouts())
        .unwrap();
    assert!(again == game, "the lowered game does not round-trip");
    assert_eq!(SavCodec.encode(&again), bytes);
}
