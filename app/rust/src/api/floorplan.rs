//! The floorplan of the open save for the Ship page. A thin adapter over
//! `ftl_app::floorplan`; positions are pixels of the game with the origin at
//! the top left of the drawing. Every square carries its fire and breach
//! paths (`fires.<x>.<y>`, `breaches.<x>.<y>`), which go through
//! [`SaveSession::apply`].

use flutter_rust_bridge::frb;
use ftl_app::assets;
use ftl_app::floorplan::{self, Placed};

use super::editor::{EditorError, FieldEdit, FieldValue, SaveSession, patch_of};
use super::field::field_value;

/// An image of `ftl.dat` (an inner path for `game_image`) and its rectangle.
pub struct FloorImage {
    pub inner_path: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub struct FloorRoom {
    /// Room id, the index in the room list of the file.
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// The installed system of the room (`shields`), as in `systems.<id>.level`.
    pub system: Option<String>,
    /// The interior images of the system, drawn over the floor.
    pub interior: Vec<FloorImage>,
    /// The oxygen the save stores for the room, 0 to 100 percent; FTL tints
    /// the floor by it. In a preview, the value the edits leave.
    pub oxygen: i32,
    /// `rooms.<n>.oxygen`, 0 to 100; `None` when the ship has no oxygen
    /// system (FTL ignores the stored value then).
    pub oxygen_field: Option<FieldValue>,
    /// Every square by square id: left to right, then top to bottom.
    pub squares: Vec<FloorSquare>,
}

/// One square of a room.
pub struct FloorSquare {
    /// Square id in its room.
    pub id: u32,
    /// Top left corner.
    pub x: i32,
    pub y: i32,
    /// `fires.<x>.<y>`, 0 to 100: 0 puts the fire out, more lights it.
    pub fire: FieldValue,
    /// `breaches.<x>.<y>`, 0 to 100: 0 seals the breach, more adds one.
    pub breach: FieldValue,
    /// The crew standing on it, as in `CrewMember::index`.
    pub crew: Vec<u32>,
}

/// A door on the wall segment of one square: a vertical door on the left
/// wall of the square at x, y, a horizontal one on its top wall.
pub struct FloorDoor {
    pub x: i32,
    pub y: i32,
    pub vertical: bool,
}

pub struct FloorCrew {
    /// Index in the crew list, as in `crew.<n>` and `CrewMember::index`.
    pub index: u32,
    /// Race id.
    pub race: String,
    /// The look id of its sprite (`GameImageKind::CrewLookFrame`): race,
    /// sex and tints.
    pub look_id: String,
    /// Center of the sprite.
    pub x: i32,
    pub y: i32,
}

/// A weapon mount of the hull with the weapon of its slot
/// (`ftl_app::floorplan::PlanMount`): the first frame of the weapon
/// (`GameImageKind::WeaponFrame`) fills the box, mirrored first when
/// `mirror`, then turned a quarter clockwise when `rotate`. FTL draws it
/// under the hull.
pub struct FloorMount {
    /// The weapon slot, as the n-th weapon of the Equipment page.
    pub slot: u32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub rotate: bool,
    pub mirror: bool,
    /// The weapon blueprint in the slot; `None` for a free slot.
    pub weapon: Option<String>,
    /// Powered: slid out of the hull.
    pub armed: bool,
}

pub struct Floorplan {
    pub width: i32,
    pub height: i32,
    pub hull: Option<FloorImage>,
    pub floor: Option<FloorImage>,
    pub rooms: Vec<FloorRoom>,
    pub doors: Vec<FloorDoor>,
    pub crew: Vec<FloorCrew>,
    /// One per weapon slot, in slot order.
    pub mounts: Vec<FloorMount>,
}

fn image(p: Placed) -> FloorImage {
    FloorImage {
        inner_path: p.inner_path,
        x: p.x,
        y: p.y,
        w: p.w,
        h: p.h,
    }
}

/// The floorplan of the player ship (`nearby` false) or the nearby ship
/// from the open `ftl.dat`; `None` in reduced mode, without that ship and
/// when its body is not decoded.
#[frb(sync)]
pub fn floorplan_of(session: &SaveSession, nearby: bool) -> Option<Floorplan> {
    plan_dto(session.file.game(), nearby)
}

/// The floorplan as `edits` would leave the ship: the systems of its rooms
/// and their interiors follow a staged install or removal. The fire and
/// breach values of its squares are those of the preview, so the page reads
/// the fields and their file values from [`floorplan_of`]. Fails like
/// [`SaveSession::apply`] when an edit is invalid.
#[frb(sync)]
pub fn floorplan_preview(
    session: &SaveSession,
    nearby: bool,
    edits: Vec<FieldEdit>,
) -> Result<Option<Floorplan>, EditorError> {
    let game = session.file.preview(&patch_of(edits, vec![])?)?;
    Ok(plan_dto(&game, nearby))
}

fn plan_dto(game: &ftl_app::SavedGame, nearby: bool) -> Option<Floorplan> {
    let shared = assets::shared()?;
    let p = floorplan::floorplan(game, super::field::ship(nearby), &shared)?;
    Some(Floorplan {
        width: p.width,
        height: p.height,
        hull: p.hull.map(image),
        floor: p.floor.map(image),
        rooms: p
            .rooms
            .into_iter()
            .map(|r| FloorRoom {
                id: r.id,
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
                system: r.system.map(|k| k.id().to_owned()),
                interior: r.interior.into_iter().map(image).collect(),
                oxygen: r.oxygen,
                oxygen_field: r.oxygen_field.map(field_value),
                squares: r
                    .squares
                    .into_iter()
                    .map(|q| FloorSquare {
                        id: q.id,
                        x: q.x,
                        y: q.y,
                        fire: field_value(q.fire),
                        breach: field_value(q.breach),
                        crew: q.crew.into_iter().map(|i| i as u32).collect(),
                    })
                    .collect(),
            })
            .collect(),
        doors: p
            .doors
            .into_iter()
            .map(|d| FloorDoor {
                x: d.x,
                y: d.y,
                vertical: d.vertical,
            })
            .collect(),
        crew: p
            .crew
            .into_iter()
            .map(|c| FloorCrew {
                index: c.index as u32,
                look_id: ftl_app::assets::crew_look_id(&c.race, c.male, &c.tints),
                race: c.race,
                x: c.x,
                y: c.y,
            })
            .collect(),
        mounts: p
            .mounts
            .into_iter()
            .map(|m| FloorMount {
                slot: m.slot as u32,
                x: m.x,
                y: m.y,
                w: m.w,
                h: m.h,
                rotate: m.rotate,
                mirror: m.mirror,
                weapon: m.weapon,
                armed: m.armed,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::editor::open_save;

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
    );

    /// Opening a save opens the located `ftl.dat` for the whole process.
    #[test]
    fn the_fixture_floorplan_reaches_the_app() {
        let session = open_save(FIXTURE.to_owned()).unwrap();
        let Some(p) = floorplan_of(&session, false) else {
            println!("skip: no ftl.dat located");
            return;
        };
        assert_eq!((p.rooms.len(), p.doors.len(), p.crew.len()), (17, 26, 3));
        let weapons: Vec<_> = p.mounts.iter().map(|m| m.weapon.as_deref()).collect();
        assert_eq!(
            weapons,
            [Some("MISSILES_2_PLAYER"), Some("LASER_BURST_3"), None, None]
        );
        let shields = &p.rooms[5];
        assert_eq!(
            (
                shields.system.as_deref(),
                shields.squares[1].fire.path.as_str()
            ),
            (Some("shields"), "fires.9.5")
        );
        assert_eq!(p.hull.unwrap().inner_path, "img/ship/kestral_base.png");
        let o2 = p.rooms[4].oxygen_field.as_ref().unwrap();
        assert_eq!(
            (p.rooms[4].oxygen, o2.path.as_str(), o2.max),
            (100, "rooms.4.oxygen", 100)
        );
        let edits = vec![FieldEdit {
            path: "rooms.4.oxygen".into(),
            value: 5,
        }];
        let preview = floorplan_preview(&session, false, edits).unwrap().unwrap();
        assert_eq!((preview.rooms[4].oxygen, preview.rooms[3].oxygen), (5, 100));
    }

    /// Cloaking out, the teleporter in: the preview moves the system from
    /// room 8 to room 15 (PLAYER_SHIP_HARD), with a teleporter pad per square.
    #[test]
    fn the_preview_floorplan_follows_a_staged_swap() {
        let mut session = open_save(FIXTURE.to_owned()).unwrap();
        if floorplan_of(&session, false).is_none() {
            println!("skip: no ftl.dat located");
            return;
        }
        // The fixture ship holds 5 of 8 systems: fill it with drones,
        // cloaking and mind control, so a teleporter needs a swap.
        let full = [
            ("systems.drones.installed", 1),
            ("systems.cloaking.installed", 1),
            ("systems.mind.installed", 1),
        ]
        .map(|(path, value)| FieldEdit {
            path: path.into(),
            value,
        });
        session.apply(full.into(), vec![]).unwrap();
        let alone = [FieldEdit {
            path: "systems.teleporter.installed".into(),
            value: 1,
        }];
        assert_eq!(
            floorplan_preview(&session, false, alone.into())
                .err()
                .unwrap()
                .code,
            "edit",
            "a full ship takes no teleporter"
        );
        let edits = [
            ("systems.cloaking.installed", 0),
            ("systems.teleporter.installed", 1),
        ]
        .map(|(path, value)| FieldEdit {
            path: path.into(),
            value,
        });
        let p = floorplan_preview(&session, false, edits.into())
            .unwrap()
            .unwrap();
        assert_eq!(p.rooms[8].system, None);
        assert!(p.rooms[8].interior.is_empty());
        assert_eq!(p.rooms[15].system.as_deref(), Some("teleporter"));
        assert_eq!(p.rooms[15].interior.len(), p.rooms[15].squares.len());
        let file = floorplan_of(&session, false).unwrap();
        assert_eq!(file.rooms[8].system.as_deref(), Some("cloaking"));
        assert_eq!(file.rooms[15].system, None);
        let bad = [FieldEdit {
            path: "systems.shields.installed".into(),
            value: 0,
        }];
        assert_eq!(
            floorplan_preview(&session, false, bad.into())
                .err()
                .unwrap()
                .code,
            "edit"
        );
    }
}
