//! The floorplan of a ship (the player ship or the nearby ship): where hull,
//! floor, rooms, interiors, doors, fires, breaches and crew sit, in pixels of
//! the game. The fire and breach fields of the nearby ship carry `nearby.`.
//!
//! The geometry follows `SavedGameFloorplanPanel.addBundle` of the Java
//! editor, with the origin at the top left of the drawing:
//!
//! - A square is [`SQUARE`] pixels. The layout origin `L` is the layout
//!   offset (`X_OFFSET`, `Y_OFFSET`) times [`SQUARE`].
//! - The hull image sits at `L + (img.x, img.y)` with the size `img.w` x
//!   `img.h` of `data/<layout>.xml`; the floor image at the hull position
//!   plus `offsets/floor`.
//! - A room sits at `L + (x, y) * SQUARE`; its interior fills the room.
//! - A door sits on the wall at `L + (x, y) * SQUARE`: a vertical door
//!   covers the left wall of its square, a horizontal one the top wall.
//! - A square of a room sits at the room plus `(id % w, id / w) * SQUARE`.
//!   Its fire and breach are addressed by the grid square, offset included
//!   (`fires.<x>.<y>`, `breaches.<x>.<y>`), which the file stores for a
//!   breach and `Room::grid_square` answers for a fire.
//! - A crew member stores the center of its sprite (`sprite_x`, `sprite_y`)
//!   relative to the origin, also with the offset included.
//! - Weapon slot `n` sits on mount `n` of the layout (`<weaponMounts>`), at
//!   the hull position plus the mount's `x`, `y`; [`place_weapon`] names
//!   how FTL turns, mirrors and slides the art there.
//!
//! Then every position moves by the top left corner of the hull and the
//! rooms, so the drawing starts at 0, 0.

use ftl_assets::{GameData, ShipBlueprint, ShipLayout, Slide, WeaponMount};
use ftl_core::{CrewMember, Field, SavedGame, Ship, ShipId, SystemKind};

use crate::assets::GameAssets;
use crate::crew::Bounded;
use crate::layouts::DEFAULT_ITEM_SLOTS;

/// Pixels per square.
pub const SQUARE: i32 = 35;

/// An image of `ftl.dat` placed on the floorplan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub inner_path: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRoom {
    /// Room id, the index in the room list of the file.
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    /// The installed system of the room.
    pub system: Option<SystemKind>,
    /// The interior images of the system; empty without one.
    pub interior: Vec<Placed>,
    /// The oxygen the file stores for the room, in percent: what FTL tints
    /// the floor by (the app's `oxygenTint`). A ship without the oxygen system
    /// stores 0 and FTL draws it so.
    pub oxygen: i32,
    /// `rooms.<n>.oxygen`, 0 to 100; `None` when the ship lacks the oxygen
    /// system, which makes FTL ignore the value at load.
    pub oxygen_field: Option<Bounded>,
    /// Every square by square id: left to right, then top to bottom.
    pub squares: Vec<PlanSquare>,
}

/// One square of a room: what burns, leaks and stands on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanSquare {
    /// Square id in its room.
    pub id: u32,
    /// Top left corner.
    pub x: i32,
    pub y: i32,
    /// `fires.<x>.<y>`, 0 to 100: 0 without fire.
    pub fire: Bounded,
    /// `breaches.<x>.<y>`, 0 to 100: 0 without breach.
    pub breach: Bounded,
    /// The crew members standing on it (`crew.<n>`), by the room and square
    /// the file stores for them.
    pub crew: Vec<usize>,
}

/// A door: the wall segment it covers, one square long.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanDoor {
    pub x: i32,
    pub y: i32,
    pub vertical: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanCrew {
    /// Index in the crew list of the file, as in `crew.<n>`.
    pub index: usize,
    pub race: String,
    /// FTL draws a female human with the `female` sheet.
    pub male: bool,
    /// The tint index per color layer.
    pub tints: Vec<i32>,
    /// Center of the sprite.
    pub x: i32,
    pub y: i32,
}

/// A weapon mount of the hull with the weapon of its slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanMount {
    /// The weapon slot, the index in the weapon list of the ship.
    pub slot: usize,
    /// Where the art lies after FTL turned and slid it ([`place_weapon`]):
    /// the first frame of the weapon's strip fills this box, turned a
    /// quarter clockwise when [`Self::rotate`], mirrored first when
    /// [`Self::mirror`]. An empty mount, or a weapon without art, has the
    /// box of the Java editor (64 x 25 on the mount, along the barrel).
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

/// What [`place_weapon`] needs of a weapon's art (`WeaponArt` of ftl-assets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MountArt {
    pub frame_width: i32,
    pub frame_height: i32,
    /// `<mountPoint>` of the weapon's `weaponAnim`.
    pub mount_point: (i32, i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Floorplan {
    pub width: i32,
    pub height: i32,
    pub hull: Option<Placed>,
    pub floor: Option<Placed>,
    /// In room id order.
    pub rooms: Vec<PlanRoom>,
    pub doors: Vec<PlanDoor>,
    /// Crew aboard, without enemy boarding drones.
    pub crew: Vec<PlanCrew>,
    /// One per weapon slot of the ship blueprint, in slot order; none for
    /// a ship whose blueprint has no weapons system (the flagship).
    pub mounts: Vec<PlanMount>,
}

/// The floorplan of the ship `on` with the images of the open `ftl.dat`;
/// `None` when the game has no such ship, its body is not decoded or the
/// game data lack its blueprint or layout.
pub fn floorplan(game: &SavedGame, on: ShipId, assets: &GameAssets) -> Option<Floorplan> {
    let data = assets.data();
    let size = |inner_path: &str| assets.image(inner_path).ok().and_then(|b| png_size(&b));
    plan(game, on, data, &size)
}

/// [`floorplan`] over game data and a lookup of image sizes: `image_size`
/// answers the pixel size of an inner path, `None` when `ftl.dat` lacks it.
pub fn plan(
    game: &SavedGame,
    on: ShipId,
    data: &GameData,
    image_size: &dyn Fn(&str) -> Option<(i32, i32)>,
) -> Option<Floorplan> {
    let ship = game.ship(on)?;
    let blueprint = data.ship(&ship.blueprint_id)?;
    let layout = data.layout(&blueprint.layout)?;
    let art = |id: &str| {
        data.weapon_art(id).map(|a| MountArt {
            frame_width: a.frame_width as i32,
            frame_height: a.frame_height as i32,
            mount_point: a.mount_point,
        })
    };
    plan_with_art(game, on, blueprint, layout, image_size, &art)
}

/// [`plan`] with blueprint and layout at hand; weapons without art.
pub fn plan_with(
    game: &SavedGame,
    on: ShipId,
    blueprint: &ShipBlueprint,
    layout: &ShipLayout,
    image_size: &dyn Fn(&str) -> Option<(i32, i32)>,
) -> Option<Floorplan> {
    plan_with_art(game, on, blueprint, layout, image_size, &|_| None)
}

/// [`plan_with`] and the art of each weapon blueprint by id.
pub fn plan_with_art(
    game: &SavedGame,
    on: ShipId,
    blueprint: &ShipBlueprint,
    layout: &ShipLayout,
    image_size: &dyn Fn(&str) -> Option<(i32, i32)>,
    weapon_art: &dyn Fn(&str) -> Option<MountArt>,
) -> Option<Floorplan> {
    let ship = game.ship(on)?;
    let body = ship.body.as_ref()?;
    let (lx, ly) = (layout.offset_x * SQUARE, layout.offset_y * SQUARE);

    let hull = layout.image.and_then(|[x, y, w, h]| {
        let gfx = &ship.gfx_base_name;
        [
            format!("img/ship/{gfx}_base.png"),
            format!("img/ships_glow/{gfx}_base.png"),
        ]
        .into_iter()
        .find(|p| image_size(p).is_some())
        .map(|inner_path| Placed {
            inner_path,
            x: lx + x,
            y: ly + y,
            w,
            h,
        })
    });
    let floor = layout.image.and_then(|[x, y, _, _]| {
        let inner_path = format!("img/ship/{}_floor.png", ship.gfx_base_name);
        let (w, h) = image_size(&inner_path)?;
        let [fx, fy] = layout.floor_offset.unwrap_or([0, 0]);
        Some(Placed {
            inner_path,
            x: lx + x + fx,
            y: ly + y + fy,
            w,
            h,
        })
    });

    let mut rooms = Vec::with_capacity(body.rooms.len());
    for (id, state) in body.rooms.iter().enumerate() {
        let r = layout.rooms.iter().find(|r| r.id as usize == id)?;
        let (x, y) = (lx + r.x * SQUARE, ly + r.y * SQUARE);
        let (w, h) = (r.w as i32 * SQUARE, r.h as i32 * SQUARE);
        let system = installed_system(ship, blueprint, id as u32);
        let interior = system
            .map(|(kind, img)| interior(kind, img, (x, y, w, h), r.w, image_size))
            .unwrap_or_default();
        let squares = (0..state.squares.len())
            .map(|s| {
                let (gx, gy) = state.grid_square(s);
                let i = s as i32;
                PlanSquare {
                    id: s as u32,
                    x: x + (i % r.w as i32) * SQUARE,
                    y: y + (i / r.w as i32) * SQUARE,
                    fire: Bounded::of(Field::Fire { x: gx, y: gy }.on(on), game),
                    breach: Bounded::of(Field::Breach { x: gx, y: gy }.on(on), game),
                    crew: crew_on(ship, id, s),
                }
            })
            .collect();
        let oxygen = Field::RoomOxygen { room: id }.on(on);
        rooms.push(PlanRoom {
            id: id as u32,
            x,
            y,
            w,
            h,
            system: system.map(|(k, _)| k),
            interior,
            oxygen: state.oxygen,
            oxygen_field: oxygen.get(game).is_ok().then(|| Bounded::of(oxygen, game)),
            squares,
        });
    }

    let mut doors: Vec<PlanDoor> = Vec::new();
    for d in &layout.doors {
        let door = PlanDoor {
            x: lx + d.x * SQUARE,
            y: ly + d.y * SQUARE,
            vertical: d.vertical,
        };
        // Java keys doors by (x, y, vertical); a layout may list one twice.
        if !doors.contains(&door) {
            doors.push(door);
        }
    }

    let crew = ship
        .crew
        .iter()
        .enumerate()
        .filter(|(_, c)| aboard(c))
        .map(|(index, c)| PlanCrew {
            index,
            race: c.race.clone(),
            male: c.male,
            tints: c.sprite_tints.clone(),
            x: c.sprite_x,
            y: c.sprite_y,
        })
        .collect();

    // The hull image's top left, where FTL's mount coordinates start.
    let hull_origin = layout
        .image
        .map_or((lx, ly), |[x, y, _, _]| (lx + x, ly + y));
    let has_weapons = blueprint.systems.iter().any(|s| s.system == "weapons");
    let slots = if has_weapons {
        blueprint.weapon_slots.unwrap_or(DEFAULT_ITEM_SLOTS) as usize
    } else {
        0
    };
    let mounts = layout
        .weapon_mounts
        .iter()
        .take(slots)
        .enumerate()
        .map(|(slot, mount)| {
            let weapon = body.weapons.get(slot);
            let armed = weapon.is_some_and(|w| w.armed);
            let art = weapon.and_then(|w| weapon_art(&w.id));
            let (x, y, w, h) = place_weapon(mount, hull_origin, art, armed);
            PlanMount {
                slot,
                x,
                y,
                w,
                h,
                rotate: mount.rotate,
                mirror: mount.mirror,
                weapon: weapon.map(|w| w.id.clone()),
                armed,
            }
        })
        .collect();

    let mut plan = Floorplan {
        width: 0,
        height: 0,
        hull,
        floor,
        rooms,
        doors,
        crew,
        mounts,
    };
    plan.move_to_origin();
    Some(plan)
}

impl Floorplan {
    /// Moves every position so the hull and the rooms start at 0, 0, and
    /// sets the size to cover them.
    fn move_to_origin(&mut self) {
        // The art of a weapon may reach past the hull; an empty mount's box
        // shows only under the pointer and does not count.
        let boxes = self
            .hull
            .iter()
            .map(|p| (p.x, p.y, p.w, p.h))
            .chain(self.rooms.iter().map(|r| (r.x, r.y, r.w, r.h)))
            .chain(
                self.mounts
                    .iter()
                    .filter(|m| m.weapon.is_some())
                    .map(|m| (m.x, m.y, m.w, m.h)),
            );
        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for (x, y, w, h) in boxes {
            (x0, y0) = (x0.min(x), y0.min(y));
            (x1, y1) = (x1.max(x + w), y1.max(y + h));
        }
        if x0 > x1 {
            return;
        }
        let place = |p: &mut Placed| {
            p.x -= x0;
            p.y -= y0;
        };
        self.hull.iter_mut().for_each(place);
        self.floor.iter_mut().for_each(place);
        for r in &mut self.rooms {
            (r.x, r.y) = (r.x - x0, r.y - y0);
            r.interior.iter_mut().for_each(place);
            for q in &mut r.squares {
                (q.x, q.y) = (q.x - x0, q.y - y0);
            }
        }
        for d in &mut self.doors {
            (d.x, d.y) = (d.x - x0, d.y - y0);
        }
        for c in &mut self.crew {
            (c.x, c.y) = (c.x - x0, c.y - y0);
        }
        for m in &mut self.mounts {
            (m.x, m.y) = (m.x - x0, m.y - y0);
        }
        (self.width, self.height) = (x1 - x0, y1 - y0);
    }
}

/// The box a weapon's art fills on its mount, as FTL 1.6.13 draws it
/// (macOS binary, Ghidra, read 2026-11-19):
///
/// - `ShipManager::RenderWeapons` (0x100095c90) translates to
///   `Ship::GetShipCorner` and draws slot `n` on `Ship::GetWeaponMount(n)`.
///   The corner and the mounts both carry the `glowOffset` of the layout
///   (`Ship::OnInit` adds it to the one and takes it from the other), so a
///   mount lies at the hull image's top left plus its `x`, `y`, as in the
///   Java editor. The weapons draw before `Ship::OnRenderBase`: under the hull.
/// - `WeaponAnimation::OnRender` (0x10004d550) moves the frame so its
///   `mountPoint` lies on the mount (a mirrored frame takes `fw - x`), adds
///   the slide of `GetSlide` and turns it 90 degrees about that point when
///   the mount says `rotate` (`GL_Rotate` 90.0 at 0x1002b4a74: a quarter
///   clockwise on the screen, so the barrel that points up in the strip
///   points right). `Animation::OnRender` flips a mirrored frame first.
/// - `WeaponAnimation::GetSlide` (0x10004cdd0) moves the art by
///   `(1 - progress)` of half the extent of its box along the slide (half
///   the box height up or down, half its width left or right; a turned
///   box is frame height wide and frame width high): a powered weapon
///   (progress 1) sits on the mount, an unpowered one (`SetPowered(false)`
///   runs the tracker back to 0) that far back into the hull, against its
///   slide direction.
///
/// Without art the box is the Java editor's (`addWeaponSprite`): 64 x 25
/// to the right of a turned mount, else 25 x 64 above it.
pub fn place_weapon(
    mount: &WeaponMount,
    (ox, oy): (i32, i32),
    art: Option<MountArt>,
    armed: bool,
) -> (i32, i32, i32, i32) {
    let (rx, ry) = (ox + mount.x, oy + mount.y);
    let Some(art) = art else {
        const LONG: i32 = 64;
        const SHORT: i32 = 25;
        return if mount.rotate {
            (rx, ry - SHORT / 2, LONG, SHORT)
        } else {
            (rx - SHORT / 2, ry - LONG, SHORT, LONG)
        };
    };
    let (fw, fh) = (art.frame_width, art.frame_height);
    let (mut mx, my) = art.mount_point;
    if mount.mirror {
        mx = fw - mx;
    }
    let (sx, sy) = if armed {
        (0, 0)
    } else {
        let (along_x, along_y) = if mount.rotate { (fh, fw) } else { (fw, fh) };
        let (half_x, half_y) = (along_x / 2, along_y / 2);
        match mount.slide {
            Slide::Down => (0, -half_y),
            Slide::Up => (0, half_y),
            Slide::Right => (-half_x, 0),
            Slide::Left => (half_x, 0),
            Slide::No => (0, 0),
        }
    };
    if mount.rotate {
        // A point (u, v) of the frame lands at the mount plus the slide
        // plus (my - v, u - mx).
        (rx + sx + my - fh, ry + sy - mx, fh, fw)
    } else {
        (rx + sx - mx, ry + sy - my, fw, fh)
    }
}

/// A crew member drawn on the floorplan: aboard, no enemy boarding drone,
/// and not staged for removal (a preview).
fn aboard(c: &CrewMember) -> bool {
    !c.enemy_boarding_drone && c.room_id >= 0 && !c.removed
}

/// The crew standing on square `square` of room `room`.
fn crew_on(ship: &Ship, room: usize, square: usize) -> Vec<usize> {
    ship.crew
        .iter()
        .enumerate()
        .filter(|(_, c)| aboard(c) && c.room_id == room as i32 && c.room_square == square as i32)
        .map(|(i, _)| i)
        .collect()
}

/// The system of a room that the ship has installed, with the interior
/// image the blueprint names for it.
fn installed_system<'b>(
    ship: &Ship,
    blueprint: &'b ShipBlueprint,
    room: u32,
) -> Option<(SystemKind, Option<&'b str>)> {
    blueprint
        .systems
        .iter()
        .filter(|s| s.room == room)
        .find_map(|s| {
            let kind = SystemKind::from_id(&s.system)?;
            let installed = ship
                .systems
                .iter()
                .any(|slot| slot.kind == kind && slot.installed.is_some());
            installed.then_some((kind, s.img.as_deref()))
        })
}

/// The interior of a system room. A blueprint `img` names the image; without
/// one FTL draws `room_<system>` when the game has it. The image fills the
/// room. The teleporter draws one pad (`teleporter_off`) centered on every
/// square, like the Java editor.
fn interior(
    kind: SystemKind,
    img: Option<&str>,
    (x, y, w, h): (i32, i32, i32, i32),
    squares_h: u32,
    image_size: &dyn Fn(&str) -> Option<(i32, i32)>,
) -> Vec<Placed> {
    let path = |name: &str| format!("img/ship/interior/{name}.png");
    if img.is_none() && kind == SystemKind::Teleporter {
        let inner_path = path("teleporter_off");
        let Some((pw, ph)) = image_size(&inner_path) else {
            return vec![];
        };
        let squares = (w / SQUARE) * (h / SQUARE);
        let cols = squares_h as i32;
        return (0..squares)
            .map(|s| Placed {
                inner_path: inner_path.clone(),
                x: x + (s % cols) * SQUARE + (SQUARE - pw) / 2,
                y: y + (s / cols) * SQUARE + (SQUARE - ph) / 2,
                w: pw,
                h: ph,
            })
            .collect();
    }
    let inner_path = img.map_or_else(|| path(&format!("room_{}", kind.id())), path);
    if image_size(&inner_path).is_none() {
        return vec![];
    }
    vec![Placed {
        inner_path,
        x,
        y,
        w,
        h,
    }]
}

/// Width and height from the IHDR chunk of a PNG.
fn png_size(bytes: &[u8]) -> Option<(i32, i32)> {
    if bytes.get(..8)? != b"\x89PNG\r\n\x1a\n" || bytes.get(12..16)? != b"IHDR" {
        return None;
    }
    let int = |at: usize| -> Option<i32> {
        i32::try_from(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?)).ok()
    };
    Some((int(16)?, int(20)?))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::SaveFile;
    use crate::assets::{ShipLayoutDoor, ShipLayoutRoom, ShipSystem};
    use crate::fixture_layouts as layouts;
    use ftl_core::Patch;

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
    );

    /// The rooms of `data/kestral.txt` (FTL 1.6.13) as `(x, y, w, h)`.
    const KESTRAL_ROOMS: [(i32, i32, i32, i32); 17] = [
        (14, 2, 1, 2),
        (12, 2, 2, 2),
        (10, 2, 2, 1),
        (10, 3, 2, 1),
        (8, 1, 2, 2),
        (8, 3, 2, 2),
        (6, 0, 2, 1),
        (6, 1, 2, 2),
        (6, 3, 2, 2),
        (6, 5, 2, 1),
        (4, 2, 2, 2),
        (3, 1, 2, 1),
        (3, 4, 2, 1),
        (1, 1, 2, 1),
        (1, 2, 2, 2),
        (1, 4, 2, 1),
        (0, 2, 1, 2),
    ];

    /// The numbers of `data/kestral.txt` and `.xml` (FTL 1.6.13):
    /// offset 0, 2; hull image at -59, -103, 652 x 418; floor at 45, 93.
    fn kestral() -> ShipLayout {
        #[rustfmt::skip]
        let doors = [
            (14, 3, 0, 1, 1), (12, 3, 1, 3, 1), (12, 2, 1, 2, 1), (10, 2, 2, 4, 1),
            (10, 3, 3, 5, 1), (8, 3, 4, 5, 0), (8, 1, 4, 7, 1), (8, 4, 5, 8, 1),
            (7, 5, 8, 9, 0), (7, 1, 6, 7, 0), (6, 2, 7, 10, 1), (6, 3, 8, 10, 1),
            (4, 4, 10, 12, 0), (4, 2, 10, 11, 0), (3, 1, 11, 13, 1), (2, 2, 13, 14, 0),
            (2, 4, 14, 15, 0), (3, 4, 12, 15, 1), (1, 3, 14, 16, 1), (1, 2, 14, 16, 1),
            (0, 2, 16, -1, 1), (0, 3, 16, -1, 1), (6, 6, 9, -1, 0), (7, 6, 9, -1, 0),
            (6, 0, 6, -1, 0), (7, 0, 6, -1, 0),
        ];
        ShipLayout {
            offset_x: 0,
            offset_y: 2,
            rooms: KESTRAL_ROOMS
                .iter()
                .enumerate()
                .map(|(id, &(x, y, w, h))| ShipLayoutRoom {
                    id: id as u32,
                    x,
                    y,
                    w: w as u32,
                    h: h as u32,
                })
                .collect(),
            doors: doors
                .iter()
                .map(|&(x, y, a, b, v)| ShipLayoutDoor {
                    x,
                    y,
                    room_a: a,
                    room_b: b,
                    vertical: v == 1,
                })
                .collect(),
            image: Some([-59, -103, 652, 418]),
            floor_offset: Some([45, 93]),
            ..ShipLayout::default()
        }
    }

    fn system(system: &str, room: u32, img: Option<&str>) -> ShipSystem {
        ShipSystem {
            slot: None,
            system: system.into(),
            power: 1,
            max: None,
            room,
            start: true,
            img: img.map(Into::into),
        }
    }

    fn blueprint() -> ShipBlueprint {
        ShipBlueprint {
            id: "PLAYER_SHIP_HARD".into(),
            layout: "kestral".into(),
            img: "kestral".into(),
            class_name: String::new(),
            name: String::new(),
            description: String::new(),
            systems: vec![
                system("pilot", 0, None),
                system("shields", 5, Some("room_shields_4")),
                system("weapons", 10, None),
                system("teleporter", 15, None),
            ],
            weapon_slots: None,
            drone_slots: None,
            weapons: vec![],
            drones: vec![],
            augments: vec![],
            missiles: 0,
            drone_parts: 0,
            hull: 0,
            max_power: 0,
            crew: vec![],
        }
    }

    /// Every image exists except any `room_weapons`; the floor is 330 x 260.
    fn sizes(inner_path: &str) -> Option<(i32, i32)> {
        match inner_path {
            p if p.contains("room_weapons") => None,
            p if p.ends_with("_floor.png") => Some((330, 260)),
            _ => Some((35, 35)),
        }
    }

    fn fixture_plan() -> Floorplan {
        let file =
            SaveFile::open_with_layouts(FIXTURE, Arc::new(layouts::fixture_layouts())).unwrap();
        plan_with(
            file.game(),
            ShipId::Player,
            &blueprint(),
            &kestral(),
            &sizes,
        )
        .unwrap()
    }

    #[test]
    fn the_layout_of_the_fixture_ship_maps_to_pixels() {
        let p = fixture_plan();
        // The hull is the top left: 0 * 35 - 59 = -59, 2 * 35 - 103 = -33.
        let (x0, y0) = (-59, 2 * 35 - 103);
        assert_eq!((p.width, p.height), (652, 418));
        let hull = p.hull.as_ref().unwrap();
        assert_eq!(
            (hull.inner_path.as_str(), hull.x, hull.y, hull.w, hull.h),
            ("img/ship/kestral_base.png", 0, 0, 652, 418)
        );
        let floor = p.floor.as_ref().unwrap();
        assert_eq!((floor.x, floor.y, floor.w, floor.h), (45, 93, 330, 260));
        assert_eq!(p.rooms.len(), 17);
        for (room, &(x, y, w, h)) in p.rooms.iter().zip(&KESTRAL_ROOMS) {
            assert_eq!(
                (room.x, room.y, room.w, room.h),
                (x * 35 - x0, (2 + y) * 35 - y0, w * 35, h * 35),
                "room {}",
                room.id
            );
        }
        assert_eq!(p.doors.len(), 26);
        let d = &p.doors[1];
        assert_eq!((d.x, d.y, d.vertical), (12 * 35 - x0, 5 * 35 - y0, true));
    }

    #[test]
    fn crew_stand_at_their_sprite_centers() {
        let p = fixture_plan();
        // Elizabeth: sprite 192, 157 = center of square 1 of room 10.
        let room = &p.rooms[10];
        let c = &p.crew[2];
        assert_eq!((c.index, c.race.as_str()), (2, "human"));
        assert_eq!((c.x, c.y), (room.x + 35 + 17, room.y + 17));
        assert_eq!((c.x, c.y), (192 + 59, 157 + 33));
        assert_eq!(p.crew.len(), 3);
        // The square the file stores for the crew member holds them.
        assert_eq!(room.squares[1].crew, [2]);
        // Berko: sprite 507, 157 = square 0 of room 0, the one-wide pilot room.
        let c = &p.crew[0];
        assert_eq!((c.index, c.x, c.y), (0, 507 + 59, 157 + 33));
        assert_eq!(p.rooms[0].squares[0].crew, [0]);
        let standing: usize = p
            .rooms
            .iter()
            .flat_map(|r| &r.squares)
            .map(|q| q.crew.len())
            .sum();
        assert_eq!(standing, 3);
    }

    #[test]
    fn installed_systems_get_their_interior() {
        let p = fixture_plan();
        let shields = &p.rooms[5];
        assert_eq!(shields.system, Some(SystemKind::Shields));
        let i = &shields.interior[0];
        assert_eq!(
            (i.inner_path.as_str(), i.x, i.y, i.w, i.h),
            (
                "img/ship/interior/room_shields_4.png",
                shields.x,
                shields.y,
                70,
                70
            )
        );
        // Without img FTL's default name, when the game has the image.
        assert_eq!(
            p.rooms[0].interior[0].inner_path,
            "img/ship/interior/room_pilot.png"
        );
        assert_eq!(p.rooms[10].system, Some(SystemKind::Weapons));
        assert!(p.rooms[10].interior.is_empty());
        // The fixture has no teleporter.
        assert_eq!(p.rooms[15].system, None);
        assert_eq!(p.rooms[1].system, None, "drones are not in this blueprint");
    }

    /// A plan follows the game it is given: the editor passes the preview
    /// of the staged edits, so an install shows its system in its room, the
    /// teleporter with a pad per square.
    #[test]
    fn an_installed_system_draws_in_its_blueprint_room() {
        let file =
            SaveFile::open_with_layouts(FIXTURE, Arc::new(layouts::fixture_layouts())).unwrap();
        let mut game = file.game().clone();
        let ship = &mut game.player_ship;
        let installed = ship.systems.iter().find_map(|s| s.installed.clone());
        let slot = ship
            .systems
            .iter_mut()
            .find(|s| s.kind == SystemKind::Teleporter);
        slot.unwrap().installed = installed;
        let p = plan_with(&game, ShipId::Player, &blueprint(), &kestral(), &sizes).unwrap();
        assert_eq!(p.rooms[15].system, Some(SystemKind::Teleporter));
        assert_eq!(p.rooms[15].interior.len(), 2, "a pad per square");
        assert_eq!(fixture_plan().rooms[15].system, None);
    }

    #[test]
    fn every_square_carries_its_fire_and_breach_path() {
        let file =
            SaveFile::open_with_layouts(FIXTURE, Arc::new(layouts::fixture_layouts())).unwrap();
        let mut game = file.game().clone();
        let body = game.player_ship.body.as_mut().unwrap();
        body.rooms[5].squares[3].fire_health = 100;
        body.breaches.push(ftl_core::Breach {
            x: 9,
            y: 6,
            health: 100,
        });
        let p = plan_with(&game, ShipId::Player, &blueprint(), &kestral(), &sizes).unwrap();
        // Shields: room 5, 2x2 at 8, 3 of kestral (offset 0, 2).
        let shields = &p.rooms[5];
        let q = &shields.squares[3];
        assert_eq!((q.id, q.x, q.y), (3, shields.x + 35, shields.y + 35));
        assert_eq!(
            (q.fire.field.path(), q.fire.value, q.fire.max),
            ("fires.9.6".to_owned(), 100, 100)
        );
        assert_eq!(
            (q.breach.field.path(), q.breach.value),
            ("breaches.9.6".to_owned(), 100)
        );
        let quiet = &shields.squares[0];
        assert_eq!(
            (
                quiet.fire.field.path(),
                quiet.fire.value,
                quiet.breach.value
            ),
            ("fires.8.5".to_owned(), 0, 0)
        );
        // The quiet fixture: nothing burns, nothing is breached.
        let quiet = plan_with(
            file.game(),
            ShipId::Player,
            &blueprint(),
            &kestral(),
            &sizes,
        )
        .unwrap();
        assert!(
            quiet
                .rooms
                .iter()
                .flat_map(|r| &r.squares)
                .all(|q| q.fire.value == 0 && q.breach.value == 0)
        );
    }

    /// Each room carries its oxygen and `rooms.<n>.oxygen`; a preview of
    /// staged edits carries the staged value.
    #[test]
    fn every_room_carries_its_oxygen_and_its_path() {
        let file =
            SaveFile::open_with_layouts(FIXTURE, Arc::new(layouts::fixture_layouts())).unwrap();
        let p = fixture_plan();
        let r = &p.rooms[4];
        let f = r.oxygen_field.as_ref().unwrap();
        assert_eq!(
            (r.oxygen, f.field.path(), f.value, f.min, f.max),
            (100, "rooms.4.oxygen".to_owned(), 100, 0, 100)
        );
        let patch = Patch::new(vec!["rooms.4.oxygen=20".parse().unwrap()]);
        let game = file.preview(&patch).unwrap();
        let p = plan_with(&game, ShipId::Player, &blueprint(), &kestral(), &sizes).unwrap();
        assert_eq!(p.rooms[4].oxygen, 20);
        assert_eq!(p.rooms[3].oxygen, 100);
        // A ship without the oxygen system: FTL ignores the stored value,
        // so the room has no field. The Kestrel cannot lose its start
        // system through an edit, so the test takes it out of the model.
        let mut game = file.game().clone();
        let slot = game
            .player_ship
            .systems
            .iter_mut()
            .find(|s| s.kind == SystemKind::Oxygen)
            .unwrap();
        slot.installed = None;
        let p = plan_with(&game, ShipId::Player, &blueprint(), &kestral(), &sizes).unwrap();
        assert!(p.rooms.iter().all(|r| r.oxygen_field.is_none()));
    }

    #[test]
    fn an_undecoded_body_has_no_floorplan() {
        let file =
            SaveFile::open_with_layouts(FIXTURE, Arc::new(ftl_core::KnownLayouts::new())).unwrap();
        assert!(
            plan_with(
                file.game(),
                ShipId::Player,
                &blueprint(),
                &kestral(),
                &sizes
            )
            .is_none()
        );
    }

    /// `place_weapon` against `WeaponAnimation::OnRender` and `GetSlide`:
    /// a 16 x 60 frame with its mount point at 2, 44 on a mount at 100, 200.
    #[test]
    fn a_weapon_turns_mirrors_and_slides_about_its_mount_point() {
        let art = Some(MountArt {
            frame_width: 16,
            frame_height: 60,
            mount_point: (2, 44),
        });
        let mount = |rotate, mirror, slide| WeaponMount {
            x: 100,
            y: 200,
            rotate,
            mirror,
            slide,
        };
        let at = |m, armed| place_weapon(&m, (10, 20), art, armed);
        // Upright: the mount point on the mount (110, 220).
        assert_eq!(at(mount(false, false, Slide::Up), true), (108, 176, 16, 60));
        // Mirrored: the mount point is 16 - 2 = 14 from the left.
        assert_eq!(at(mount(false, true, Slide::Up), true), (96, 176, 16, 60));
        // Unpowered: half the box back against the slide.
        assert_eq!(
            at(mount(false, false, Slide::Up), false),
            (108, 176 + 30, 16, 60)
        );
        assert_eq!(
            at(mount(false, false, Slide::Down), false),
            (108, 176 - 30, 16, 60)
        );
        assert_eq!(
            at(mount(false, false, Slide::Left), false),
            (108 + 8, 176, 16, 60)
        );
        assert_eq!(
            at(mount(false, false, Slide::Right), false),
            (108 - 8, 176, 16, 60)
        );
        assert_eq!(
            at(mount(false, false, Slide::No), false),
            (108, 176, 16, 60)
        );
        // Turned a quarter clockwise about the mount point: (u, v) lands at
        // (110 + 44 - v, 220 + u - 2), a box 60 wide and 16 high.
        assert_eq!(at(mount(true, false, Slide::Up), true), (94, 218, 60, 16));
        // Turned and unpowered: the box is 16 high, so up slides 8.
        assert_eq!(
            at(mount(true, false, Slide::Up), false),
            (94, 218 + 8, 60, 16)
        );
        assert_eq!(
            at(mount(true, false, Slide::Left), false),
            (94 + 30, 218, 60, 16)
        );
        // Without art: the Java editor's box.
        assert_eq!(
            place_weapon(&mount(true, true, Slide::Up), (10, 20), None, true),
            (110, 208, 64, 25)
        );
        assert_eq!(
            place_weapon(&mount(false, true, Slide::Up), (10, 20), None, true),
            (98, 156, 25, 64)
        );
    }

    #[test]
    fn png_size_reads_the_header() {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        png.extend(330u32.to_be_bytes());
        png.extend(260u32.to_be_bytes());
        assert_eq!(png_size(&png), Some((330, 260)));
        assert_eq!(png_size(b"GIF89a"), None);
    }
}
