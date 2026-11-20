//! The crew of the open save for the Crew page. A thin adapter over
//! `ftl_app::crew::roster`; edits go through [`SaveSession::apply`] with the
//! paths `crew.<n>.skills.<skill>` and `crew.<n>.health`, and as text
//! ([`super::editor::TextEdit`]) `crew.<n>.name` and `crew.<n>.race`;
//! `nearby.` before each of them for the nearby ship.

use flutter_rust_bridge::frb;
use ftl_app::Field;
use ftl_app::assets;
use ftl_app::crew;

use super::editor::{EditorError, FieldEdit, FieldValue, SaveSession, TextEdit, patch_of};
use super::field::{field_value, ship};

/// One skill of a crew member.
pub struct CrewSkill {
    /// `pilot`, `engine`, `shield`, `weapon`, `repair` or `combat`.
    pub skill: String,
    /// The points as `crew.<n>.skills.<skill>` with their range.
    pub points: FieldValue,
    /// 0, 1 or 2, from the points in the file.
    pub mastery: u8,
    /// The points of mastery 0, 1 and 2 for the race of this crew member. A
    /// click on the skill ring sets the next of them (2 goes back to 0).
    pub mastery_points: Vec<i32>,
}

pub struct CrewMember {
    /// Index in the crew list of the file, as in `crew.<n>`.
    pub index: u32,
    pub name: String,
    /// `crew.<n>.name`, a text field: 1 to 32 characters.
    pub name_path: String,
    /// Race id as FTL writes it; also the id of the crew sprite.
    pub race: String,
    /// `crew.<n>.race`, a text field: an id of [`race_choices`].
    pub race_path: String,
    /// The race title from the open `ftl.dat`; `None` in reduced mode.
    pub race_name: Option<String>,
    /// A female human wears the `female` sprite.
    pub male: bool,
    /// `crew.<n>.health`, 1 to the full health of the race.
    pub health: FieldValue,
    /// The six skills in the order pilot, engine, shield, weapon, repair, combat.
    pub skills: Vec<CrewSkill>,
    /// `crew.<n>.male`: 1 male, 0 female.
    pub sex: FieldValue,
    /// `crew.<n>.tints.<layer>`, one per tint layer of the race.
    pub tints: Vec<FieldValue>,
    /// `crew.<n>.remove`: max 0 for the last crew member of a side.
    pub remove: FieldValue,
    /// A preview with `crew.<n>.remove=1` staged.
    pub removed: bool,
    /// The look of its sprite (`GameImageKind::CrewLook`, `CrewLookFrame`).
    pub look_id: String,
}

/// One look of FTL's character creator for a race.
pub struct LookChoice {
    pub male: bool,
    /// One tint index per layer, as `crew.<n>.tints.<layer>`.
    pub tints: Vec<i32>,
    pub look_id: String,
}

/// Every look the character creator offers for `race`: each color of its
/// tint layer, for a human male and female; empty for an unknown race.
#[frb(sync)]
pub fn look_choices(race: String) -> Vec<LookChoice> {
    crew::look_choices(&race)
        .into_iter()
        .map(|l| LookChoice {
            male: l.male,
            tints: l.tints,
            look_id: l.look_id,
        })
        .collect()
}

/// `crew.add` of the player ship (`nearby` false) or the nearby ship with
/// how many default humans it can take; `None` without the ship's rooms.
#[frb(sync)]
pub fn crew_add_field(session: &SaveSession, nearby: bool) -> Option<FieldValue> {
    crew::add_field(session.file.game(), ship(nearby)).map(field_value)
}

/// The crew of the player ship (`nearby` false) or the nearby ship in file
/// order, without enemy boarding drones; empty without a nearby ship.
#[frb(sync)]
pub fn crew_of(session: &SaveSession, nearby: bool) -> Vec<CrewMember> {
    crew_in(session.file.game(), nearby)
}

/// The crew of the player ship (`nearby` false) or the nearby ship as the
/// pending edits would leave it (a new race refits health and skills), with
/// the ranges of that state; fails like [`SaveSession::apply`] when an edit
/// is invalid.
#[frb(sync)]
pub fn crew_preview(
    session: &SaveSession,
    nearby: bool,
    edits: Vec<FieldEdit>,
    texts: Vec<TextEdit>,
) -> Result<Vec<CrewMember>, EditorError> {
    Ok(crew_in(
        &session.file.preview(&patch_of(edits, texts)?)?,
        nearby,
    ))
}

fn crew_in(game: &ftl_app::SavedGame, nearby: bool) -> Vec<CrewMember> {
    let data = assets::shared();
    let on = ship(nearby);
    crew::roster(game, on, data.as_ref().map(|a| a.data().as_ref()))
        .into_iter()
        .map(|c| CrewMember {
            index: c.index as u32,
            name: c.name,
            name_path: Field::CrewName { crew: c.index }.on(on).path(),
            race: c.race,
            race_path: Field::CrewRace { crew: c.index }.on(on).path(),
            race_name: c.race_name,
            male: c.male,
            health: field_value(c.health),
            skills: c
                .skills
                .into_iter()
                .map(|s| CrewSkill {
                    skill: s.kind.id().to_owned(),
                    points: field_value(s.points),
                    mastery: s.mastery,
                    mastery_points: s.mastery_points.to_vec(),
                })
                .collect(),
            sex: field_value(c.sex),
            tints: c.tints.into_iter().map(field_value).collect(),
            remove: field_value(c.remove),
            removed: c.removed,
            look_id: c.look_id,
        })
        .collect()
}

/// A race the race picker offers.
pub struct RaceChoice {
    /// Race id.
    pub id: String,
    /// Its default look (`GameImageKind::CrewLook`).
    pub look_id: String,
    /// The title from the open `ftl.dat` in `language`; `None` in reduced mode.
    pub name: Option<String>,
}

/// The races `crew.<n>.race` accepts, in the order of the game.
#[frb(sync)]
pub fn race_choices(language: String) -> Vec<RaceChoice> {
    let data = assets::shared();
    crew::race_choices(data.as_ref().map(|a| a.data().as_ref()), &language)
        .into_iter()
        .map(|r| RaceChoice {
            look_id: ftl_app::assets::crew_look_id(r.id, true, &[]),
            id: r.id.to_owned(),
            name: r.name,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::editor::open_save;

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
    );

    #[test]
    fn the_crew_of_the_fixture_reaches_the_app_and_a_step_edit_applies() {
        let mut session = open_save(FIXTURE.to_owned()).unwrap();
        let crew = crew_of(&session, false);
        let names: Vec<_> = crew
            .iter()
            .map(|c| (c.index, c.name.as_str(), c.race.as_str()))
            .collect();
        assert_eq!(
            names,
            [
                (0, "Berko", "human"),
                (1, "Chris Malott", "human"),
                (2, "Elizabeth", "human")
            ]
        );
        let elizabeth = &crew[2];
        assert_eq!(
            (
                elizabeth.health.path.as_str(),
                elizabeth.health.value,
                elizabeth.health.max
            ),
            ("crew.2.health", 100, 100)
        );
        let weapon = &elizabeth.skills[3];
        assert_eq!(
            (weapon.skill.as_str(), weapon.points.value, weapon.mastery),
            ("weapon", 4, 0)
        );
        assert_eq!(weapon.mastery_points, [0, 58, 116]);

        let preview = crew_preview(
            &session,
            false,
            vec![],
            vec![TextEdit {
                path: elizabeth.race_path.clone(),
                text: "crystal".into(),
            }],
        )
        .unwrap();
        assert_eq!(
            (preview[2].race.as_str(), preview[2].health.max),
            ("crystal", 125)
        );
        assert_eq!(
            crew_of(&session, false)[2].race,
            "human",
            "a preview changes nothing"
        );
        let changes = session
            .apply(
                vec![FieldEdit {
                    path: weapon.points.path.clone(),
                    value: 116,
                }],
                vec![
                    TextEdit {
                        path: elizabeth.name_path.clone(),
                        text: "Ripley".into(),
                    },
                    TextEdit {
                        path: elizabeth.race_path.clone(),
                        text: "crystal".into(),
                    },
                ],
            )
            .unwrap();
        let shown: Vec<_> = changes
            .iter()
            .map(|c| (c.path.as_str(), c.old.as_str(), c.new.as_str()))
            .collect();
        assert_eq!(
            shown,
            [
                ("crew.2.skills.weapon", "4", "116"),
                ("crew.2.name", "Elizabeth", "Ripley"),
                ("crew.2.race", "human", "crystal")
            ]
        );
        let after = &crew_of(&session, false)[2];
        assert_eq!(
            (after.name.as_str(), after.race.as_str()),
            ("Ripley", "crystal")
        );
        // The race change keeps mastery 2 of the weapons: 116 points of a
        // human become 130 on the Crystal interval.
        assert_eq!(
            (after.skills[3].points.value, after.skills[3].mastery),
            (130, 2)
        );
        assert_eq!(after.health.max, 125);
    }

    #[test]
    fn a_new_crew_member_and_a_look_reach_the_preview() {
        let session = open_save(FIXTURE.to_owned()).unwrap();
        let Some(add) = crew_add_field(&session, false) else {
            println!("skip: no ftl.dat located, the rooms are not decoded");
            return;
        };
        assert_eq!((add.path.as_str(), add.value, add.max), ("crew.add", 0, 5));
        let preview = crew_preview(
            &session,
            false,
            vec![
                FieldEdit {
                    path: "crew.add".into(),
                    value: 1,
                },
                FieldEdit {
                    path: "crew.3.male".into(),
                    value: 0,
                },
                FieldEdit {
                    path: "crew.3.tints.1".into(),
                    value: 2,
                },
                FieldEdit {
                    path: "crew.0.remove".into(),
                    value: 1,
                },
            ],
            vec![],
        )
        .unwrap();
        assert_eq!(preview.len(), 4, "a staged removal stays in the preview");
        assert!(preview[0].removed);
        let new = &preview[3];
        assert_eq!(
            (new.race.as_str(), new.look_id.as_str()),
            ("human", "human.f.0.2")
        );
        assert_eq!(new.tints.len(), 2);
        assert_eq!(look_choices("mantis".into()).len(), 5);
        // A look picked for a pending race applies after the race change.
        let preview = crew_preview(
            &session,
            false,
            vec![FieldEdit {
                path: "crew.2.tints.0".into(),
                value: 4,
            }],
            vec![TextEdit {
                path: "crew.2.race".into(),
                text: "mantis".into(),
            }],
        )
        .unwrap();
        // Elizabeth keeps her sex in the look id; a Mantis looks the same.
        assert_eq!(preview[2].look_id, "mantis.f.4");
    }
}
