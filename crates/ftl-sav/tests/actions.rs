//! The one-click actions of `ftl_core::actions` on a file FTL wrote: each
//! action builds a patch, the patch applies, the game state is what the
//! action promises, and the edited game encodes to bytes that decode to it.

use ftl_core::{
    Action, ActionNote, Breach, MAX_REACTOR, Patch, Resource, SaveCodec, SavedGame, ShipLimits,
    SkillKind, SystemKind, max_health,
};
use ftl_sav::SavCodec;

#[path = "common/layouts.rs"]
mod layouts;

const FORMAT11: &[u8] = include_bytes!("fixtures/continue-1.6.13-format11.sav");

fn fixture() -> SavedGame {
    SavCodec
        .decode(FORMAT11, &layouts::fixture_layouts())
        .unwrap()
}

/// Limits as ftl.dat 1.6.13 answers them for `PLAYER_SHIP_HARD`, except
/// engines at 6 (ftl.dat: 8), so the blueprint cap is seen to win over the
/// editor cap; a test lowers shields as well.
fn limits() -> ShipLimits {
    ShipLimits {
        max_hull: Some(30),
        system_levels: vec![
            (SystemKind::Shields, 8),
            (SystemKind::Engines, 6),
            (SystemKind::Oxygen, 3),
            (SystemKind::Weapons, 8),
            (SystemKind::Drones, 8),
            (SystemKind::Medbay, 3),
            (SystemKind::Pilot, 3),
            (SystemKind::Sensors, 3),
            (SystemKind::Doors, 3),
        ],
    }
}

/// Applies the action and checks the round trip; answers the edited game
/// and the notes.
fn run(action: Action, limits: &ShipLimits) -> (SavedGame, Vec<ActionNote>) {
    run_on(fixture(), action, limits)
}

fn run_on(
    mut game: SavedGame,
    action: Action,
    limits: &ShipLimits,
) -> (SavedGame, Vec<ActionNote>) {
    let plan = action.plan(&game, limits);
    plan.patch.apply(&mut game).unwrap();
    let again = SavCodec
        .decode(&SavCodec.encode(&game), &layouts::fixture_layouts())
        .unwrap();
    assert!(
        again == game,
        "{action:?}: the edited game does not round-trip"
    );
    (game, plan.notes)
}

fn level(game: &SavedGame, kind: SystemKind) -> Option<i32> {
    game.player_ship
        .systems
        .iter()
        .find(|s| s.kind == kind)
        .and_then(|s| s.installed.as_ref())
        .map(|s| s.capacity)
}

#[test]
fn max_everything_fills_resources_systems_and_reactor_within_the_limits() {
    // The fixture's Kestrel has no system the limits do not name: install
    // Cloaking and Battery first.
    let l = layouts::install_layouts();
    let mut start = SavCodec.decode(FORMAT11, &l).unwrap();
    Patch::new(vec![
        "systems.cloaking.installed=1".parse().unwrap(),
        "systems.battery.installed=1".parse().unwrap(),
    ])
    .apply_in(&mut start, &l)
    .unwrap();
    let (game, notes) = run_on(start.clone(), Action::MaxEverything, &limits());
    let r = &game.player_ship.resources;
    assert_eq!(r.hull, 30);
    for res in [
        Resource::Fuel,
        Resource::DroneParts,
        Resource::Missiles,
        Resource::Scrap,
    ] {
        assert_eq!(res.get(&game), res.range().1, "{res}");
    }
    assert_eq!(game.player_ship.reactor_power, MAX_REACTOR);
    // The blueprint caps engines at 6; systems it does not name keep the
    // cap of unmodded FTL.
    assert_eq!(level(&game, SystemKind::Engines), Some(6));
    assert_eq!(level(&game, SystemKind::Shields), Some(8));
    assert_eq!(level(&game, SystemKind::Cloaking), Some(3));
    assert_eq!(level(&game, SystemKind::Battery), Some(2));
    // Every installed system reached a cap; none was installed.
    let before = start;
    for (a, b) in game
        .player_ship
        .systems
        .iter()
        .zip(&before.player_ship.systems)
    {
        assert_eq!(a.installed.is_some(), b.installed.is_some(), "{:?}", a.kind);
    }
    assert!(notes.contains(&ActionNote::SystemsAtEditorCap));
    assert!(
        !notes
            .iter()
            .any(|n| matches!(n, ActionNote::HullAtEditorCap { .. }))
    );
}

#[test]
fn max_everything_never_lowers_a_value() {
    let mut low = limits();
    // A blueprint cap below the level the ship already has (a mod changed it).
    low.system_levels.retain(|(k, _)| *k != SystemKind::Shields);
    low.system_levels.push((SystemKind::Shields, 2));
    let (game, _) = run(Action::MaxEverything, &low);
    assert_eq!(
        level(&game, SystemKind::Shields),
        level(&fixture(), SystemKind::Shields)
    );
}

#[test]
fn crew_fully_trained_sets_every_skill_to_the_second_mastery() {
    let (game, notes) = run(Action::CrewFullyTrained, &ShipLimits::default());
    assert!(notes.is_empty());
    let crew: Vec<_> = game
        .player_ship
        .crew
        .iter()
        .filter(|c| !c.enemy_boarding_drone)
        .collect();
    assert!(!crew.is_empty());
    for c in crew {
        for kind in SkillKind::ALL {
            assert_eq!(
                c.skills.get(kind).points,
                kind.mastery_points(&c.race, 2),
                "{} {kind:?}",
                c.name
            );
        }
    }
}

#[test]
fn repair_ship_fills_the_hull_puts_out_fires_and_seals_breaches() {
    // The fixture neither burns nor leaks: light two rooms and open two breaches.
    let mut damaged = fixture();
    let body = damaged.player_ship.body.as_mut().unwrap();
    for room in [0, 2] {
        body.rooms[room].squares[0].fire_health = 100;
        body.rooms[room].squares[0].ignition_progress = 40;
    }
    body.breaches = vec![
        Breach {
            x: 14,
            y: 5,
            health: 100,
        },
        Breach {
            x: 11,
            y: 4,
            health: 60,
        },
    ];
    let before = damaged.clone();
    let (game, notes) = run_on(damaged, Action::RepairShip, &limits());

    assert_eq!(game.player_ship.resources.hull, 30);
    let body = game.player_ship.body.as_ref().unwrap();
    assert!(
        body.rooms
            .iter()
            .flat_map(|r| &r.squares)
            .all(|s| s.fire_health == 0)
    );
    assert!(body.breaches.is_empty());
    assert!(notes.is_empty(), "{notes:?}");
    // Only hull, fires and breaches change; scrap and the systems stay.
    assert_eq!(
        game.player_ship.resources.scrap,
        before.player_ship.resources.scrap
    );
    assert_eq!(game.player_ship.systems, before.player_ship.systems);
}

/// Low oxygen through the edit path; Repair ship refills every room to
/// 100, as `OxygenSystem::Restart` fills them, and lists only the rooms
/// that changed.
#[test]
fn repair_ship_refills_the_oxygen_of_every_room() {
    let mut low = fixture();
    Patch::new(vec![
        "rooms.0.oxygen=0".parse().unwrap(),
        "rooms.4.oxygen=37".parse().unwrap(),
    ])
    .apply(&mut low)
    .unwrap();
    let plan = Action::RepairShip.plan(&low, &limits());
    let oxygen: Vec<String> = plan
        .patch
        .assignments
        .iter()
        .map(|a| a.field.path())
        .filter(|p| p.starts_with("rooms."))
        .collect();
    assert_eq!(oxygen, ["rooms.0.oxygen", "rooms.4.oxygen"]);
    let (game, _) = run_on(low, Action::RepairShip, &limits());
    let body = game.player_ship.body.as_ref().unwrap();
    assert!(body.rooms.iter().all(|r| r.oxygen == 100));
}

#[test]
fn repair_ship_mends_broken_bars_and_clears_ion_through_the_same_paths() {
    // The fixture has no damage: break and ionize systems through the edit paths.
    let mut damaged = fixture();
    let staged = Patch::new(
        [
            "systems.shields.damage=2",
            "systems.engines.damage=1",
            "systems.weapons.ion=3",
        ]
        .map(|a| a.parse().unwrap())
        .to_vec(),
    );
    staged.apply(&mut damaged).unwrap();
    let plan = Action::RepairShip.plan(&damaged, &limits());
    let paths: Vec<String> = plan
        .patch
        .assignments
        .iter()
        .map(|a| format!("{}={:?}", a.field.path(), a.value))
        .collect();
    for p in [
        "systems.shields.damage=Int(0)",
        "systems.engines.damage=Int(0)",
        "systems.weapons.ion=Int(0)",
    ] {
        assert!(paths.contains(&p.to_owned()), "{p} in {paths:?}");
    }
    let (game, notes) = run_on(damaged, Action::RepairShip, &limits());
    assert!(notes.is_empty(), "{notes:?}");
    for s in game
        .player_ship
        .systems
        .iter()
        .filter_map(|s| s.installed.as_ref())
    {
        assert_eq!((s.damaged_bars, s.ionized_bars), (0, 0));
    }
}

#[test]
fn repair_ship_without_a_blueprint_uses_the_editor_cap_and_says_so() {
    let (game, notes) = run(Action::RepairShip, &ShipLimits::default());
    let cap = Resource::Hull.range().1;
    assert_eq!(game.player_ship.resources.hull, cap);
    assert!(notes.contains(&ActionNote::HullAtEditorCap { cap }));
}

#[test]
fn heal_crew_sets_every_crew_member_to_the_full_health_of_its_race() {
    let (game, _) = run(Action::HealCrew, &ShipLimits::default());
    for c in game
        .player_ship
        .crew
        .iter()
        .filter(|c| !c.enemy_boarding_drone)
    {
        assert_eq!(c.health, max_health(&c.race), "{}", c.name);
    }
}

#[test]
fn an_action_lists_only_changed_fields_and_twice_is_nothing() {
    let mut game = fixture();
    let plan = Action::MaxEverything.plan(&game, &limits());
    for c in plan.patch.apply(&mut game).unwrap() {
        assert_ne!(c.old, c.new, "{}", c.field);
    }
    assert!(
        Action::MaxEverything
            .plan(&game, &limits())
            .patch
            .assignments
            .is_empty()
    );
}

#[test]
fn action_ids_parse_back() {
    for a in Action::ALL {
        assert_eq!(a.id().parse::<Action>(), Ok(a));
    }
    assert!("everything".parse::<Action>().is_err());
}
