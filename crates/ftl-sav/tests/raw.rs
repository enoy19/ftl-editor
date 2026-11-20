//! The raw view of the real fixture (`ftl_core::raw`): every decoded field
//! with its path, type and range, edits of numbers, flags, unknown fields
//! and state vars that encode and decode back to the edited game.

use ftl_core::raw::{self, RawKind};
use ftl_core::{Assignment, EditError, Field, KnownLayouts, Patch, SaveCodec, SavedGame};
use ftl_sav::SavCodec;

#[path = "common/layouts.rs"]
mod layouts;

const FORMAT11: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11.sav");

fn game() -> SavedGame {
    SavCodec
        .decode(FORMAT11, &layouts::fixture_layouts())
        .expect("fixture decodes")
}

fn apply(game: &mut SavedGame, edit: &str) -> Result<(), EditError> {
    Patch::new(vec![edit.parse::<Assignment>()?])
        .apply(game)
        .map(|_| ())
}

/// The edited game encodes and decodes back to itself: what the verify
/// step of a save checks.
fn assert_round_trips(game: &SavedGame) {
    let again = SavCodec
        .decode(&SavCodec.encode(game), &layouts::fixture_layouts())
        .expect("the edited game decodes");
    assert_eq!(&again, game);
}

#[test]
fn the_tree_lists_every_area_of_the_decoded_save() {
    let g = game();
    let entries = raw::entries(&g);
    let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
    for path in [
        "file_format",
        "state_vars",
        "player_ship.resources.scrap",
        "player_ship.crew.0.health",
        "player_ship.systems.0",
        "world.beacons.0.seen",
        "situation.nearby_ship.ship.resources.hull",
        "undecoded",
    ] {
        assert!(paths.contains(&path), "{path} missing");
    }
    let scrap = entries
        .iter()
        .find(|e| e.path == "player_ship.resources.scrap")
        .unwrap();
    assert_eq!(scrap.kind, RawKind::I32);
    assert_eq!(scrap.depth, 2);
    assert_eq!(scrap.value, g.player_ship.resources.scrap.to_string());
    assert_eq!(
        scrap.field,
        Some(Field::Raw("player_ship.resources.scrap".into()))
    );
    assert_eq!(scrap.range, Some((i32::MIN, i32::MAX)));
    let list = entries
        .iter()
        .find(|e| e.path == "player_ship.crew")
        .unwrap();
    assert_eq!(list.kind, RawKind::List);
    assert_eq!(list.value, g.player_ship.crew.len().to_string());
    let name = entries
        .iter()
        .find(|e| e.path == "player_ship.name")
        .unwrap();
    assert_eq!((name.kind, name.field.clone()), (RawKind::Text, None));
    let difficulty = entries.iter().find(|e| e.path == "difficulty").unwrap();
    assert_eq!(difficulty.kind, RawKind::Enum);
    // Every field the fixture holds has a name from the FTL binary
    // (docs/unknown-fields.md); only a beam in flight still has one unknown.
    let unknown: Vec<_> = entries
        .iter()
        .filter(|e| e.unknown)
        .map(|e| &e.path)
        .collect();
    assert!(unknown.is_empty(), "{unknown:?}");
    // Every editable entry names a field that parses back to itself.
    for e in entries.iter().filter(|e| e.field.is_some()) {
        let field = e.field.clone().unwrap();
        assert_eq!(field.path().parse::<Field>().unwrap(), field);
        assert_eq!(field.range(&g).ok(), e.range, "{}", e.path);
    }
}

#[test]
fn a_raw_edit_to_the_same_value_leaves_the_game_equal() {
    // One write deserializes the whole tree: this proves the tree holds
    // every field of the model.
    let g = game();
    let mut edited = g.clone();
    let hull = g.player_ship.resources.hull;
    apply(
        &mut edited,
        &format!("raw.player_ship.resources.hull={hull}"),
    )
    .unwrap();
    assert_eq!(edited, g);
}

#[test]
fn raw_edits_of_numbers_flags_and_formerly_unknown_fields_round_trip() {
    let mut g = game();
    apply(&mut g, "raw.player_ship.resources.scrap=123").unwrap();
    assert_eq!(g.player_ship.resources.scrap, 123);
    let seen = i32::from(!g.world.as_ref().unwrap().beacons[0].seen);
    apply(&mut g, &format!("raw.world.beacons.0.seen={seen}")).unwrap();
    assert_eq!(i32::from(g.world.as_ref().unwrap().beacons[0].seen), seen);
    // A field the Java editor leaves unnamed (Door::iHacked, its epsilon).
    let path = "player_ship.body.doors.0.hacked";
    let value = raw::get(&g, path).unwrap();
    apply(&mut g, &format!("raw.{path}={}", value + 1)).unwrap();
    assert_eq!(raw::get(&g, path), Some(value + 1));
    assert_round_trips(&g);
}

#[test]
fn a_state_var_is_edited_by_its_name() {
    let mut g = game();
    let id = g.state_vars[0].id.clone();
    let entry = raw::entries(&g)
        .into_iter()
        .find(|e| e.path == "state_vars.0.value")
        .unwrap();
    assert_eq!(entry.field, Some(Field::StateVar(id.clone())));
    apply(&mut g, &format!("state.{id}=7")).unwrap();
    assert_eq!(g.state_vars[0].value, 7);
    assert_round_trips(&g);
    assert!(matches!(
        apply(&mut g, "state.no_such_var=1"),
        Err(EditError::NoStateVar { .. })
    ));
}

#[test]
fn raw_paths_that_name_no_number_are_refused() {
    let mut g = game();
    for edit in [
        "raw.player_ship.name=1",
        "raw.player_ship=1",
        "raw.no_such_field=1",
        "raw.player_ship.crew.99.health=1",
    ] {
        assert!(
            matches!(apply(&mut g, edit), Err(EditError::NoRawField { .. })),
            "{edit}"
        );
    }
    assert!(matches!(
        apply(&mut g, "raw.world.beacons.0.seen=2"),
        Err(EditError::OutOfRange { min: 0, max: 1, .. })
    ));
    assert!("nearby.raw.file_format".parse::<Field>().is_err());
}

#[test]
fn undecoded_bytes_stay_through_a_raw_edit() {
    // Without the layouts the rooms and everything after them stay undecoded.
    let mut g = SavCodec.decode(FORMAT11, &KnownLayouts::new()).unwrap();
    assert!(!g.undecoded.is_empty());
    let before = g.undecoded.clone();
    let entry = raw::entries(&g)
        .into_iter()
        .find(|e| e.path == "undecoded")
        .unwrap();
    assert_eq!(entry.value, format!("{} bytes", before.len()));
    apply(&mut g, "raw.player_ship.resources.fuel=3").unwrap();
    assert_eq!(g.undecoded, before);
    assert_eq!(raw::hex_lines(&before).len(), before.len().div_ceil(16));
}
