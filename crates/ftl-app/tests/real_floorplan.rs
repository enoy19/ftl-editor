//! The floorplan of the fixture with the layout and images of the local
//! game's `ftl.dat` (`FTL_DAT` or the macOS Steam install), else a skip line.

use std::path::PathBuf;
use std::sync::Arc;

use ftl_app::assets::GameAssets;
use ftl_app::floorplan::{self, SQUARE};
use ftl_app::layouts::GameLayouts;
use ftl_app::{SaveFile, ShipId};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
);

fn real_dat() -> Option<PathBuf> {
    let path = match std::env::var_os("FTL_DAT") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(std::env::var_os("HOME")?).join(
            "Library/Application Support/Steam/steamapps/common/FTL Faster Than Light/FTL.app/Contents/Resources/ftl.dat",
        ),
    };
    if path.is_file() {
        Some(path)
    } else {
        println!(
            "skip: no ftl.dat at {} (set FTL_DAT to test against the game)",
            path.display()
        );
        None
    }
}

#[test]
fn the_fixture_floorplan_follows_the_layout_file() {
    let Some(path) = real_dat() else { return };
    let assets = GameAssets::open(&path).unwrap();
    let file =
        SaveFile::open_with_layouts(FIXTURE, Arc::new(GameLayouts::new(assets.data().clone())))
            .unwrap();
    let p = floorplan::floorplan(file.game(), ShipId::Player, &assets).unwrap();
    let layout = assets.data().layout("kestral").unwrap();
    let [ix, iy, iw, ih] = layout.image.unwrap();
    let (x0, y0) = (layout.offset_x * SQUARE + ix, layout.offset_y * SQUARE + iy);

    assert_eq!((p.width, p.height), (iw, ih));
    let hull = p.hull.as_ref().unwrap();
    assert_eq!((hull.x, hull.y, hull.w, hull.h), (0, 0, iw, ih));
    let floor = p.floor.as_ref().unwrap();
    let [fx, fy] = layout.floor_offset.unwrap();
    assert_eq!(
        (
            floor.inner_path.as_str(),
            floor.x,
            floor.y,
            floor.w,
            floor.h
        ),
        ("img/ship/kestral_floor.png", fx, fy, 552, 235)
    );
    assert_eq!((p.rooms.len(), p.doors.len()), (17, 26));
    for r in &layout.rooms {
        let room = &p.rooms[r.id as usize];
        assert_eq!(
            (room.x, room.y, room.w, room.h),
            (
                (layout.offset_x + r.x) * SQUARE - x0,
                (layout.offset_y + r.y) * SQUARE - y0,
                r.w as i32 * SQUARE,
                r.h as i32 * SQUARE
            )
        );
    }
    let interiors: Vec<_> = p
        .rooms
        .iter()
        .filter_map(|r| {
            Some((
                r.id,
                r.system?.id(),
                r.interior.first()?.inner_path.as_str(),
            ))
        })
        .collect();
    assert!(interiors.contains(&(5, "shields", "img/ship/interior/room_shields.png")));
    assert!(interiors.contains(&(0, "pilot", "img/ship/interior/room_pilot.png")));
    assert_eq!(p.rooms[10].system.map(|k| k.id()), Some("weapons"));
    let crew: Vec<_> = p.crew.iter().map(|c| (c.index, c.x, c.y)).collect();
    assert_eq!(
        crew,
        [
            (0, 507 - x0, 157 - y0),
            (1, 52 - x0, 192 - y0),
            (2, 192 - x0, 157 - y0)
        ]
    );
}

/// FTL draws every room floor itself; the floor image of the ship only lies
/// below. The image of the Kestrel leaves two squares of its rooms open, one
/// in the medbay and one in the shields room, so a drawing that relies on
/// it shows the hull through them.
#[test]
fn the_floor_image_of_the_fixture_ship_leaves_room_squares_open() {
    let Some(path) = real_dat() else { return };
    let assets = GameAssets::open(&path).unwrap();
    let file =
        SaveFile::open_with_layouts(FIXTURE, Arc::new(GameLayouts::new(assets.data().clone())))
            .unwrap();
    let p = floorplan::floorplan(file.game(), ShipId::Player, &assets).unwrap();
    let floor = p.floor.as_ref().unwrap();
    let bytes = assets.image(&floor.inner_path).unwrap();
    let image = image::load_from_memory(&bytes).unwrap().to_rgba8();
    let mut open = Vec::new();
    for r in &p.rooms {
        for sy in (0..r.h).step_by(SQUARE as usize) {
            for sx in (0..r.w).step_by(SQUARE as usize) {
                let (cx, cy) = (
                    r.x + sx + SQUARE / 2 - floor.x,
                    r.y + sy + SQUARE / 2 - floor.y,
                );
                if image.get_pixel(cx as u32, cy as u32)[3] == 0 {
                    open.push((r.id, sx / SQUARE, sy / SQUARE));
                }
            }
        }
    }
    assert_eq!(open, [(4, 0, 1), (5, 0, 0)]);
    assert_eq!(p.rooms[4].system.map(|k| k.id()), Some("medbay"));
    assert_eq!(p.rooms[5].system.map(|k| k.id()), Some("shields"));
}

/// The nearby ship of the fixture (`JELLY_BUTTON_P_DLC`, layout
/// `jelly_button_pirate`) has a floorplan of its own: hull image (enemy
/// hulls live in `img/ships_glow`), six rooms and its crew, every fire and
/// breach field under `nearby.`.
#[test]
fn the_nearby_ship_has_a_floorplan_with_nearby_fields() {
    let Some(path) = real_dat() else { return };
    let assets = GameAssets::open(&path).unwrap();
    let file =
        SaveFile::open_with_layouts(FIXTURE, Arc::new(GameLayouts::new(assets.data().clone())))
            .unwrap();
    let p = floorplan::floorplan(file.game(), ShipId::Nearby, &assets).unwrap();
    assert_eq!(
        p.hull.as_ref().map(|h| h.inner_path.as_str()),
        Some("img/ships_glow/jelly_button_pirate_base.png")
    );
    assert_eq!(p.rooms.len(), 6);
    assert_eq!(p.crew.len(), 2);
    let square = &p.rooms[0].squares[0];
    assert_eq!(square.fire.field.path(), "nearby.fires.2.0");
    assert_eq!(square.breach.field.path(), "nearby.breaches.2.0");
    let player = floorplan::floorplan(file.game(), ShipId::Player, &assets).unwrap();
    assert_ne!(player.rooms.len(), p.rooms.len());
}

/// The weapons of the fixture on their mounts, placed as FTL 1.6.13 places
/// them (`floorplan::place_weapon`), worked out by hand from
/// `data/kestral.xml`, `data/jelly_button_pirate.xml` and the
/// `<mountPoint>` of each `weaponAnim`.
#[test]
fn the_weapons_of_both_ships_sit_on_their_mounts() {
    let Some(path) = real_dat() else { return };
    let assets = GameAssets::open(&path).unwrap();
    let file =
        SaveFile::open_with_layouts(FIXTURE, Arc::new(GameLayouts::new(assets.data().clone())))
            .unwrap();
    let rect = |m: &floorplan::PlanMount| (m.weapon.clone(), m.x, m.y, m.w, m.h, m.armed);
    let p = floorplan::floorplan(file.game(), ShipId::Player, &assets).unwrap();
    // The art stays inside the hull: the plan keeps its size.
    assert_eq!((p.width, p.height), (652, 418));
    assert_eq!(p.mounts.len(), 4, "weaponSlots 4 of 8 mounts");
    // Mount 0 at 540, 155, turned and mirrored; Artemis frames 25 x 43,
    // mount point 7, 30, mirrored 18, 30: x = 540 + 30 - 43, y = 155 - 18.
    assert_eq!(
        rect(&p.mounts[0]),
        (Some("MISSILES_2_PLAYER".into()), 527, 137, 43, 25, true)
    );
    // Mount 1 at 540, 262, turned; Burst Laser II 23 x 64 at 3, 44.
    assert_eq!(
        rect(&p.mounts[1]),
        (Some("LASER_BURST_3".into()), 520, 259, 64, 23, true)
    );
    // Free slots have the box of the Java editor right of the mount.
    assert_eq!(rect(&p.mounts[2]), (None, 390, 122 - 12, 64, 25, false));
    // The pirate: upright mounts. Mount 0 at 52, 136, mirrored; Basic
    // Laser 16 x 60 at 2, 44, mirrored 14, 44.
    let p = floorplan::floorplan(file.game(), ShipId::Nearby, &assets).unwrap();
    assert_eq!(
        rect(&p.mounts[0]),
        (Some("LASER_BURST_1".into()), 38, 92, 16, 60, true)
    );
    assert!(p.mounts.iter().all(|m| !m.rotate));
    // An unpowered Burst Laser II slides back into the hull against its
    // slide "down", by half the height of its turned box (23 / 2): up.
    let patch = ftl_core::Patch::new(vec!["weapons.LASER_BURST_3.armed=0".parse().unwrap()]);
    let game = file.preview(&patch).unwrap();
    let p = floorplan::floorplan(&game, ShipId::Player, &assets).unwrap();
    assert_eq!(
        rect(&p.mounts[1]),
        (Some("LASER_BURST_3".into()), 520, 259 - 11, 64, 23, false)
    );
}
