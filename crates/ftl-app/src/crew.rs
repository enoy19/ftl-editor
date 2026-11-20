//! The crew of the player ship as the CLI and the app show it: one entry per
//! crew member with its editable fields, their ranges and the mastery levels.
//!
//! Every value comes from `ftl-core` ([`Field::get`], [`Field::range`],
//! [`SkillKind::mastery_level`]); this module only collects them, so
//! `ftled crew list` and the Crew page show the same numbers. Race names come
//! from `ftl.dat` when it is open.

use ftl_assets::GameData;
use ftl_core::crew_list;
use ftl_core::{Field, RACES, SavedGame, ShipId, SkillKind, race};

use crate::assets::crew_look_id;

/// One editable field with its value and its accepted range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bounded {
    pub field: Field,
    pub value: i32,
    pub min: i32,
    pub max: i32,
}

impl Bounded {
    /// Call only with a field that exists in `game`.
    pub(crate) fn of(field: Field, game: &SavedGame) -> Bounded {
        let value = field
            .get(game)
            .expect("the roster names existing fields only");
        let (min, max) = field
            .range(game)
            .expect("the roster names existing fields only");
        Bounded {
            field,
            value,
            min,
            max,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillEntry {
    pub kind: SkillKind,
    /// The points, `crew.<n>.skills.<skill>`.
    pub points: Bounded,
    /// 0, 1 or 2.
    pub mastery: u8,
    /// The points of mastery 0, 1 and 2 for this race; a mastery step sets
    /// one of them ([`SkillKind::next_mastery_points`]).
    pub mastery_points: [i32; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrewEntry {
    /// Index in the crew list of the file, as in `crew.<n>`.
    pub index: usize,
    pub name: String,
    /// Race id as FTL writes it (`engi`, `anaerobic`, ...).
    pub race: String,
    /// The race title from `ftl.dat`; `None` without game data.
    pub race_name: Option<String>,
    /// FTL draws a female human with the `female` sprite; other races look
    /// the same either way.
    pub male: bool,
    /// `crew.<n>.health`, 1 to the full health of the race.
    pub health: Bounded,
    /// The six skills in [`SkillKind::ALL`] order.
    pub skills: Vec<SkillEntry>,
    /// `crew.<n>.male`, 0 or 1.
    pub sex: Bounded,
    /// `crew.<n>.tints.<layer>`, one per tint layer of the race.
    pub tints: Vec<Bounded>,
    /// `crew.<n>.remove`: 0 to 1, 0 to 0 for the last crew member of a side.
    pub remove: Bounded,
    /// A removal is staged (a preview of `crew.<n>.remove=1`).
    pub removed: bool,
    /// The id of its portrait with sex and tints ([`crate::assets::ImageId::CrewLook`]).
    pub look_id: String,
}

/// The crew of the ship `on` in file order; empty when the game has no such
/// ship. Enemy boarding drones share the crew list of the file; they are no
/// crew and are left out, so an index can be missing. The fields of the
/// nearby ship carry `nearby.`.
pub fn roster(game: &SavedGame, on: ShipId, data: Option<&GameData>) -> Vec<CrewEntry> {
    let Some(ship) = game.ship(on) else {
        return Vec::new();
    };
    ship.crew
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.enemy_boarding_drone)
        .map(|(index, c)| CrewEntry {
            index,
            name: c.name.clone(),
            race: c.race.clone(),
            race_name: data
                .and_then(|d| d.crew(&c.race))
                .map(|b| b.title.clone())
                .filter(|t| !t.is_empty()),
            male: c.male,
            health: Bounded::of(Field::CrewHealth { crew: index }.on(on), game),
            skills: SkillKind::ALL
                .into_iter()
                .map(|kind| {
                    let points = Bounded::of(
                        Field::CrewSkill {
                            crew: index,
                            skill: kind,
                        }
                        .on(on),
                        game,
                    );
                    SkillEntry {
                        kind,
                        mastery: kind.mastery_level(&c.race, points.value),
                        mastery_points: [0, 1, 2].map(|l| kind.mastery_points(&c.race, l)),
                        points,
                    }
                })
                .collect(),
            sex: Bounded::of(Field::CrewMale { crew: index }.on(on), game),
            tints: (0..crew_list::tint_layers(game, c).len())
                .map(|layer| Bounded::of(Field::CrewTint { crew: index, layer }.on(on), game))
                .collect(),
            remove: Bounded::of(Field::CrewRemove { crew: index }.on(on), game),
            removed: c.removed,
            look_id: crew_look_id(&c.race, c.male, &c.sprite_tints),
        })
        .collect()
}

/// `crew.add` of the ship `on` with its range: how many default humans the
/// ship can take; `None` without the ship or its rooms.
pub fn add_field(game: &SavedGame, on: ShipId) -> Option<Bounded> {
    let field = Field::CrewAdd.on(on);
    field.range(game).ok()?;
    Some(Bounded::of(field, game))
}

/// One look of FTL's character creator for a race: sex and a tint index
/// per layer, with the id of its portrait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookChoice {
    pub male: bool,
    pub tints: Vec<i32>,
    pub look_id: String,
}

/// Every look the character creator reaches for `race` (its arrows step
/// every color layer at once and switch the sex of a human): each index of
/// the layers with more than one color (every playable race has at most
/// one), other layers at 0, male first and then female for a human. Empty
/// for a race the editor does not know.
pub fn look_choices(race_id: &str) -> Vec<LookChoice> {
    let Some(r) = race(race_id) else {
        return Vec::new();
    };
    let sexes: &[bool] = if r.id == "human" {
        &[true, false]
    } else {
        &[true]
    };
    let steps = r.color_layers.iter().copied().max().unwrap_or(1).max(1);
    sexes
        .iter()
        .flat_map(|&male| {
            (0..steps).map(move |step| {
                let tints: Vec<i32> = r
                    .color_layers
                    .iter()
                    .map(|&size| {
                        if size > 1 {
                            step.min(size - 1) as i32
                        } else {
                            0
                        }
                    })
                    .collect();
                LookChoice {
                    male,
                    look_id: crew_look_id(r.id, male, &tints),
                    tints,
                }
            })
        })
        .collect()
}

/// A race the editor offers for `crew.<n>.race`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaceChoice {
    /// Race id, also the id of its sprite.
    pub id: &'static str,
    /// The race title from `ftl.dat` in `language`; `None` without game data.
    pub name: Option<String>,
}

/// Every race of [`RACES`] in its order, named from `ftl.dat` in `language`
/// (`de` reads `text-de.xml`, else English).
pub fn race_choices(data: Option<&GameData>, language: &str) -> Vec<RaceChoice> {
    RACES
        .iter()
        .map(|r| RaceChoice {
            id: r.id,
            name: data
                .and_then(|d| d.display_name_in(r.id, language))
                .filter(|t| !t.is_empty())
                .map(str::to_owned),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ftl_core::{KnownLayouts, SaveCodec};
    use ftl_sav::SavCodec;

    const FIXTURE: &[u8] =
        include_bytes!("../../ftl-sav/tests/fixtures/continue-1.6.13-format11.sav");

    fn patched(game: &mut SavedGame, paths: &[&str]) {
        let assignments = paths.iter().map(|p| p.parse().unwrap()).collect();
        ftl_core::Patch::new(assignments).apply(game).unwrap();
    }

    #[test]
    fn the_fixture_crew_lists_three_members_with_ranges_and_masteries() {
        let mut game = SavCodec.decode(FIXTURE, &KnownLayouts::new()).unwrap();
        let crew = roster(&game, ShipId::Player, None);
        let summary: Vec<_> = crew
            .iter()
            .map(|c| (c.index, c.name.as_str(), c.race.as_str(), c.male))
            .collect();
        assert_eq!(
            summary,
            [
                (0, "Berko", "human", true),
                (1, "Chris Malott", "human", true),
                (2, "Elizabeth", "human", false)
            ]
        );

        let first = &crew[0];
        assert_eq!(first.race_name.as_deref(), None);
        let races: Vec<_> = race_choices(None, "en").iter().map(|r| r.id).collect();
        assert_eq!(
            races,
            [
                "human",
                "engi",
                "mantis",
                "slug",
                "rock",
                "energy",
                "crystal",
                "anaerobic"
            ]
        );
        assert_eq!(
            first.health,
            Bounded {
                field: Field::CrewHealth { crew: 0 },
                value: 100,
                min: 1,
                max: 100
            }
        );
        let weapon = &crew[2].skills[3];
        assert_eq!(
            (weapon.kind, weapon.points.value, weapon.points.max),
            (SkillKind::Weapon, 4, 116)
        );
        assert_eq!((weapon.mastery, weapon.mastery_points), (0, [0, 58, 116]));
        assert_eq!(weapon.points.field.path(), "crew.2.skills.weapon");
        let points: Vec<Vec<i32>> = crew
            .iter()
            .map(|c| c.skills.iter().map(|s| s.points.value).collect())
            .collect();
        assert_eq!(
            points,
            [[2, 0, 0, 0, 0, 0], [0, 2, 0, 0, 0, 0], [0, 0, 0, 4, 0, 0]]
        );

        // The fixture crew has no mastery and no other race: build both.
        patched(
            &mut game,
            &[
                "crew.0.skills.weapon=116",
                "crew.0.skills.repair=16",
                "crew.1.skills.pilot=26",
                "crew.2.race=engi",
                "crew.2.skills.weapon=65",
                "crew.2.skills.combat=16",
            ],
        );
        let crew = roster(&game, ShipId::Player, None);
        let weapon = &crew[2].skills[3];
        assert_eq!(crew[2].race, "engi");
        assert_eq!(
            (weapon.points.value, weapon.points.max, weapon.mastery),
            (65, 130, 1)
        );
        assert_eq!(weapon.mastery_points, [0, 65, 130]);
        let masteries: Vec<Vec<u8>> = crew
            .iter()
            .map(|c| c.skills.iter().map(|s| s.mastery).collect())
            .collect();
        assert_eq!(
            masteries,
            [[0, 0, 0, 2, 1, 0], [2, 0, 0, 0, 0, 0], [0, 0, 0, 1, 0, 2]]
        );
    }

    #[test]
    fn the_roster_answers_look_and_removal_and_the_creator_its_looks() {
        let mut game = SavCodec.decode(FIXTURE, &KnownLayouts::new()).unwrap();
        let crew = roster(&game, ShipId::Player, None);
        let elizabeth = &crew[2];
        let tints: Vec<_> = elizabeth
            .tints
            .iter()
            .map(|t| (t.field.path(), t.value, t.max))
            .collect();
        assert_eq!(
            tints,
            [
                ("crew.2.tints.0".to_owned(), 0, 0),
                ("crew.2.tints.1".to_owned(), 3, 3)
            ]
        );
        assert_eq!(
            (elizabeth.sex.value, elizabeth.remove.max, elizabeth.removed),
            (0, 1, false)
        );
        assert_eq!(elizabeth.look_id, "human.f.0.3");
        assert_eq!(crew[1].look_id, "human.m.0.0");
        // The fixture crew is all human; an Engi has no tint layer.
        patched(&mut game, &["crew.1.race=engi"]);
        assert!(
            roster(&game, ShipId::Player, None)[1].tints.is_empty(),
            "an Engi has no tint layer"
        );
        // Without the rooms (no layouts) there is no place for a new member.
        assert_eq!(add_field(&game, ShipId::Player), None);

        let human: Vec<_> = look_choices("human")
            .into_iter()
            .map(|l| l.look_id)
            .collect();
        assert_eq!(
            human,
            [
                "human.m.0.0",
                "human.m.0.1",
                "human.m.0.2",
                "human.m.0.3",
                "human.f.0.0",
                "human.f.0.1",
                "human.f.0.2",
                "human.f.0.3"
            ]
        );
        assert_eq!(look_choices("slug").len(), 6);
        assert!(look_choices("ghost").is_empty());
    }
}
