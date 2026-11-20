//! New and removed crew members and the crew look on the real fixture
//! (`ftl_core::crew_list`): every edit decodes back to the edited game, and
//! the files land in the test tmp dir for the Java oracle
//! (`fe-crewadd-*.sav`, see AGENTS.md).

use ftl_core::{EditError, Field, KnownLayouts, Patch, SaveCodec, SavedGame, ShipId};
use ftl_sav::SavCodec;

#[path = "common/layouts.rs"]
mod layouts;

const FORMAT11: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11.sav");

/// The first male names of `data/names.xml` in FTL 1.6.13.
const NAMES: [&str; 3] = ["Kirby", "Ma Fan", "Jon"];

fn data() -> KnownLayouts {
    layouts::install_layouts().with_crew_names(&NAMES)
}

fn game() -> SavedGame {
    SavCodec.decode(FORMAT11, &data()).unwrap()
}

fn apply(g: &mut SavedGame, edits: &[&str]) -> Result<(), EditError> {
    let patch = Patch::new(edits.iter().map(|e| e.parse().unwrap()).collect());
    patch.apply_in(g, &data()).map(|_| ())
}

fn round_trip(g: &SavedGame, name: &str) {
    let out = SavCodec.encode(g);
    let again = SavCodec.decode(&out, &data()).unwrap();
    assert_eq!(&again, g, "{name} does not decode to the edited game");
    assert!(again.undecoded.is_empty(), "{name}");
    std::fs::write(
        format!("{}/fe-crewadd-{name}.sav", env!("CARGO_TARGET_TMPDIR")),
        &out,
    )
    .unwrap();
}

#[test]
fn a_new_crew_member_is_a_default_human_on_a_free_square_with_a_free_name() {
    let mut g = game();
    assert_eq!(Field::CrewAdd.range(&g), Ok((0, 5)), "3 of 8 aboard");
    apply(&mut g, &["crew.add=2"]).unwrap();
    let crew = &g.player_ship.crew;
    assert_eq!(crew.len(), 5);
    let (kirby, ma_fan) = (&crew[3], &crew[4]);
    assert_eq!(
        (kirby.name.as_str(), ma_fan.name.as_str()),
        ("Kirby", "Ma Fan")
    );
    assert_eq!(
        (kirby.race.as_str(), kirby.health, kirby.male),
        ("human", 100, true)
    );
    assert!(kirby.player_controlled);
    assert_eq!(kirby.universal_death_count, crew[0].universal_death_count);
    // The pilot room (0) holds Berko on square 0; both new members take
    // the next free squares of rooms that hold no one else.
    let squares: Vec<_> = crew.iter().map(|c| (c.room_id, c.room_square)).collect();
    for (i, s) in squares.iter().enumerate() {
        assert_eq!(
            squares.iter().filter(|t| *t == s).count(),
            1,
            "crew {i} shares {s:?}"
        );
    }
    assert_eq!(kirby.room_id, 0, "the pilot room first");
    let room = &g.player_ship.body.as_ref().unwrap().rooms[kirby.room_id as usize];
    let (gx, gy) = room.grid_square(kirby.room_square as usize);
    assert_eq!(
        (kirby.sprite_x, kirby.sprite_y),
        (gx * 35 + 17, gy * 35 + 17)
    );
    round_trip(&g, "add");
}

#[test]
fn the_crew_limit_is_8_for_the_player_and_10_for_the_enemy() {
    let mut g = game();
    assert_eq!(
        apply(&mut g, &["crew.add=6"]),
        Err(EditError::OutOfRange {
            field: Field::CrewAdd,
            value: 6,
            min: 0,
            max: 5
        })
    );
    let enemies = g
        .ship(ShipId::Nearby)
        .unwrap()
        .crew
        .iter()
        .filter(|c| !c.player_controlled)
        .count();
    let nearby = Field::CrewAdd.on(ShipId::Nearby);
    assert_eq!(nearby.range(&g), Ok((0, 10 - enemies as i32)));
    apply(&mut g, &["nearby.crew.add=1"]).unwrap();
    let new = g.ship(ShipId::Nearby).unwrap().crew.last().unwrap().clone();
    assert!(!new.player_controlled);
    assert_eq!(new.name, "Kirby");
    round_trip(&g, "nearby-add");
}

#[test]
fn a_removal_happens_at_the_end_and_keeps_the_last_crew_member() {
    let mut g = game();
    // Indices stay those of the file within the patch.
    apply(&mut g, &["crew.0.remove=1", "crew.2.name=Ripley"]).unwrap();
    let names: Vec<_> = g.player_ship.crew.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Chris Malott", "Ripley"]);
    round_trip(&g, "remove");

    let mut g = game();
    let all = ["crew.0.remove=1", "crew.1.remove=1", "crew.2.remove=1"];
    assert!(matches!(
        apply(&mut g, &all),
        Err(EditError::LastCrewMember { .. })
    ));
    // At the limit a removal makes room for an add, in either order.
    let mut g = game();
    apply(&mut g, &["crew.add=5"]).unwrap();
    let mut a = g.clone();
    apply(&mut a, &["crew.add=1", "crew.0.remove=1"]).unwrap();
    let mut b = g.clone();
    apply(&mut b, &["crew.0.remove=1", "crew.add=1"]).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.player_ship.crew.len(), 8);
}

#[test]
fn a_staged_removal_stays_in_place_with_its_mark() {
    let mut g = game();
    let patch = Patch::new(vec!["crew.1.remove=1".parse().unwrap()]);
    patch.stage_in(&mut g, &data()).unwrap();
    let marks: Vec<_> = g.player_ship.crew.iter().map(|c| c.removed).collect();
    assert_eq!(marks, [false, true, false]);
    assert_eq!(Field::CrewRemove { crew: 1 }.get(&g), Ok(1));
}

#[test]
fn the_look_is_a_tint_per_layer_and_the_sex() {
    let mut g = game();
    // The fixture's crew is human: make crew 2 a Lanius (one layer of four
    // colors) and crew 0 an Engi (none).
    apply(&mut g, &["crew.2.race=anaerobic", "crew.0.race=engi"]).unwrap();
    assert_eq!(Field::CrewTint { crew: 2, layer: 0 }.range(&g), Ok((0, 3)));
    assert!(matches!(
        Field::CrewTint { crew: 0, layer: 0 }.range(&g),
        Err(EditError::NoTintLayer { layers: 0, .. })
    ));
    apply(
        &mut g,
        &[
            "crew.2.tints.0=3",
            "crew.add=1",
            "crew.3.male=0",
            "crew.3.tints.1=2",
            "nearby.crew.0.tints.0=3",
        ],
    )
    .unwrap();
    assert_eq!(g.player_ship.crew[2].sprite_tints, [3]);
    let new = &g.player_ship.crew[3];
    assert_eq!(
        (new.male, new.sprite_tints.as_slice()),
        (false, &[0, 2][..])
    );
    assert_eq!(
        apply(&mut g, &["crew.3.tints.1=4"]),
        Err(EditError::OutOfRange {
            field: Field::CrewTint { crew: 3, layer: 1 },
            value: 4,
            min: 0,
            max: 3
        })
    );
    round_trip(&g, "look");
}

#[test]
fn a_new_crew_member_needs_the_names_of_ftl_dat() {
    let mut g = game();
    let patch = Patch::new(vec!["crew.add=1".parse().unwrap()]);
    assert!(matches!(
        patch.apply_in(&mut g, &layouts::install_layouts()),
        Err(EditError::NoCrewNames { .. })
    ));
}
