//! The reader path without the game: each test packs a tiny `ftl.dat` from
//! hand-written XML (no game content) and reads it back through `ftl_assets`.

use std::path::{Path, PathBuf};

use ftl_assets::{AchievementKind, GameData, ShipLayout};
use ftldat::{Package, PackageEntry};

/// The 67-byte PNG of one transparent pixel.
const PIXEL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

// Quirks on purpose: an XML declaration, an <FTL> root in one file and none in
// another (several roots), stray text between elements, a text id, an inline
// title, a repeated droneList (the first counts), a literal "\n" in the lookup text, and a later file overriding an id.
const BLUEPRINTS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<!-- tiny test fixture -->
<FTL>
<blueprintList name="IGNORED"><name>X</name></blueprintList>
<shipBlueprint name="PLAYER_SHIP_HARD" layout="tiny" img="tiny">
	<class id="ship_tiny_class"/>
	<name id="ship_tiny_name"/>
	<desc>Inline description.</desc>
	<systemList>
		<pilot power="1" room="0" start="true" img="room_pilot">
			<slot><direction>right</direction><number>0</number></slot>
		</pilot>
		<shields power="2" max="4" room="1" start="true"/>
		<artillery power="1" room="0"/>
		<artillery power="1" room="1"/>
	</systemList>
	<weaponSlots>3</weaponSlots>
	<droneSlots>2</droneSlots>
	<weaponList count="1" missiles="8">
		<weapon name="LASER_TINY"/>
	</weaponList>
	<aug name="AUG_TINY"/>
	<droneList count="1" drones="2">
		<drone name="DRONE_TINY"/>
	</droneList>
	<health amount="30"/>
	<maxPower amount ="8"/>
	<crewCount amount = "2" class="human"/>
	<crewCount amount="1" class="engi"/>
	<droneList count="0" drones="0"/>
	<short>first</short>
</shipBlueprint>
stray text
<crewBlueprint name="human">
	<desc id="crew_human_desc"/>
	<cost>45</cost>
	<title id="crew_human_title"/>
	<short>Human</short>
	<colorList>
		<layer><!-- Shirt -->
			<color r="140" g="140" b="255" a="0.7"/> <!--
			<color r="1" g="2" b="3" a="1"/>-->
		</layer>
		<layer>
			<color r="255" g="0" b="0" a="1"/>
			<color r="0" g="255" b="0" a="0"/>
		</layer>
	</colorList>
</crewBlueprint>
<systemBlueprint name="pilot">
	<type>pilot</type>
	<title>Piloting</title>
	<startPower>1</startPower>
	<maxPower>3</maxPower>
</systemBlueprint>
<systemBlueprint name="shields">
	<type>shields</type>
	<title id="system_shields_title"/>
	<startPower>2</startPower>
	<maxPower>8</maxPower>
</systemBlueprint>
<systemBlueprint name="artillery">
	<type>artillery</type>
	<title>Artillery</title>
	<startPower>1</startPower>
	<maxPower>4</maxPower>
</systemBlueprint>
<weaponBlueprint name="LASER_TINY">
	<type>LASER</type>
	<title>Old Title</title>
	<weaponArt>laser_tiny</weaponArt>
</weaponBlueprint>
<droneBlueprint name="DRONE_TINY">
	<type>COMBAT</type>
	<title>Tiny Drone</title>
	<power>1</power>
	<cooldown>880</cooldown> <!-- in ms -->
	<droneImage>drone_tiny</droneImage>
</droneBlueprint>
<augBlueprint name="AUG_TINY">
	<title>Tiny Augment</title>
</augBlueprint>
</FTL>
"#;

const DLC_BLUEPRINTS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<weaponBlueprint name="LASER_TINY">
	<type>LASER</type>
	<title id="weapon_LASER_TINY_title"/>
	<power>2</power>
	<cooldown>8.5</cooldown>
	<weaponArt>laser_tiny</weaponArt>
</weaponBlueprint>
<shipBlueprint name="PLAYER_SHIP_ANAEROBIC" layout="tiny" img="tiny">
	<class>Lanius Cruiser</class>
	<name>DLC Ship</name>
	<systemList><pilot power="1" room="0" start="true"/></systemList>
	<health amount="20"/>
	<maxPower amount="8"/>
</shipBlueprint>
"#;

const TEXT: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<FTL>
<text name="ship_tiny_class">Tiny Cruiser</text>
<text name="ACH_TINY_name">Small Start</text>
<text name="ACH_TINY_desc">Reach sector 2.</text>
<text name="victory_ach_title">\1 Victory</text>
<text name="victory_ach">Win a made-up run with any layout of the \1</text>
<text name="ship_tiny_name">The Tiny</text>
<text name="crew_human_title">Human</text>
<text name="crew_human_desc">Line one\nline two</text>
<text name="system_shields_title">Shields</text>
<text name="weapon_LASER_TINY_title">Tiny Laser</text>
</FTL>
"#;

const TEXT_DE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<FTL>
<text name="system_shields_title" language="de">Schilde</text>
<text name="ACH_TINY_name" language="de">Kleiner Anfang</text>
<text name="ship_tiny_class" language="de">Winzkreuzer</text>
<text name="victory_ach_title" language="de">\1-Sieg</text>
<text name="victory_ach" language="de">Besiege das Flaggschiff mit einer Variante von \1.</text>
<text name="weapon_LASER_TINY_title" language="de">Winziger Laser</text>
</FTL>
"#;

const LAYOUT_TXT: &str = "X_OFFSET\n1\nY_OFFSET\n2\nHORIZONTAL\n3\nVERTICAL\n4\nELLIPSE\n100\n50\n-5\n6\n\
ROOM\n0\n0\n0\n2\n1\nROOM\n1\n2\n0\n1\n2\nDOOR\n2\n0\n0\n1\n1\n";

const LAYOUT_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<FTL>
<img x="-10" y="-20" w="300" h="200"/>
<offsets>
	<floor x="7" y="8"/>
	<cloak x="1" y="1"/>
</offsets>
<weaponMounts><mount x="1" y="2" rotate="true" mirror="false" gib="1" slide="up"/></weaponMounts>
</FTL>
"#;

const ANIMATIONS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<FTL>
<animSheet name="laser_tiny" w="32" h="10" fw="16" fh="10">weapons/laser_tiny_strip2.png</animSheet>
<anim name="something"><sheet>laser_tiny</sheet><desc length="2" x="0" y="0"/><time>1</time></anim>
<weaponAnim name="laser_tiny">
	<sheet>laser_tiny</sheet>
	<desc length="2" x="0" y="0"/>
	<chargedFrame>1</chargedFrame>
</weaponAnim>
</FTL>
"#;

const ACHIEVEMENTS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<FTL>
<achievement id="ACH_TINY">
	<name id="ACH_TINY_name"/>
	<desc id="ACH_TINY_desc"/>
	<img>achievements/tiny.png</img>
</achievement>
<achievement id="ACH_TINY_SHIP">
	<name>Own name</name>
	<img>achievements/tiny.png</img>
	<ship>PLAYER_SHIP_HARD</ship>
</achievement>
</FTL>
"#;

const NAMES: &str = r#"<FTL>
<nameList race="human" sex="male"><name>Kirby</name><name> Jon </name></nameList>
<nameList race="human" sex="female"><name>Layla</name></nameList>
<nameList race="human" sex="male" language="de"><name>Klaus</name></nameList>
</FTL>
"#;

fn package() -> Package {
    let mut p = Package::new();
    let text = |path: &str, s: &str| PackageEntry::from_string(path, s);
    let png = |path: &str| PackageEntry::from_byte_array(path, PIXEL_PNG.to_vec());
    for e in [
        text("data/blueprints.xml", BLUEPRINTS),
        text("data/dlcBlueprints.xml", DLC_BLUEPRINTS),
        text("data/text_blueprints.xml", TEXT),
        text("data/text-de.xml", TEXT_DE),
        text("data/tiny.txt", LAYOUT_TXT),
        text("data/tiny.xml", LAYOUT_XML),
        text("data/animations.xml", ANIMATIONS),
        text("data/achievements.xml", ACHIEVEMENTS),
        text("data/names.xml", NAMES),
        png("img/ship/tiny_base.png"),
        png("img/people/human_base.png"),
        png("img/icons/s_shields_overlay.png"),
        png("img/weapons/laser_tiny_strip2.png"),
    ] {
        p.add_entry(e).unwrap();
    }
    p
}

fn write_pkg(dir: &Path) -> PathBuf {
    let path = dir.join("ftl.dat");
    package().to_path_pkg(&path).unwrap();
    path
}

fn load_pkg() -> GameData {
    let dir = tempfile::tempdir().unwrap();
    let path = write_pkg(dir.path());
    let bytes = std::fs::metadata(&path).unwrap().len();
    assert!(
        bytes < 16 * 1024,
        "the fixture stays a few KB, got {bytes} bytes"
    );
    GameData::open(&path).unwrap()
}

#[test]
fn opens_a_pkg_and_a_dat_package() {
    let dir = tempfile::tempdir().unwrap();
    let pkg = write_pkg(dir.path());
    let dat = dir.path().join("old.dat");
    package().to_path_dat(&dat).unwrap();
    for path in [pkg, dat] {
        let g = GameData::open(&path).unwrap();
        assert_eq!(
            g.ship("PLAYER_SHIP_HARD").unwrap().layout,
            "tiny",
            "{}",
            path.display()
        );
    }
}

#[test]
fn names_come_from_the_text_lookup_or_the_element() {
    let g = load_pkg();
    let ship = g.ship("PLAYER_SHIP_HARD").unwrap();
    assert_eq!(ship.name, "The Tiny");
    assert_eq!(ship.class_name, "Tiny Cruiser");
    assert_eq!(ship.description, "Inline description.");
    assert_eq!(g.crew("human").unwrap().title, "Human");
    assert_eq!(g.crew("human").unwrap().description, "Line one\nline two");
    assert_eq!(g.system("shields").unwrap().title, "Shields");
    assert_eq!(g.system("pilot").unwrap().title, "Piloting");
    assert_eq!(g.drone("DRONE_TINY").unwrap().title, "Tiny Drone");
    assert_eq!(g.augment("AUG_TINY").unwrap().title, "Tiny Augment");
    // dlcBlueprints.xml overrides blueprints.xml, like the Java DataManager.
    assert_eq!(g.weapon("LASER_TINY").unwrap().title, "Tiny Laser");
    assert_eq!(g.display_name("PLAYER_SHIP_HARD"), Some("The Tiny"));
    assert_eq!(g.display_name("LASER_TINY"), Some("Tiny Laser"));
    assert_eq!(g.display_name("NOPE"), None);
}

#[test]
fn german_names_come_from_text_de_else_the_english_name() {
    let g = load_pkg();
    assert_eq!(g.display_name_in("shields", "de"), Some("Schilde"));
    assert_eq!(
        g.display_name_in("LASER_TINY", "de"),
        Some("Winziger Laser")
    );
    // An inline title and a language without a file keep the English name.
    assert_eq!(g.display_name_in("pilot", "de"), Some("Piloting"));
    assert_eq!(g.display_name_in("LASER_TINY", "fr"), Some("Tiny Laser"));
    assert_eq!(g.display_name_in("NOPE", "de"), None);
}

#[test]
fn weapons_and_drones_carry_their_power() {
    let g = load_pkg();
    assert_eq!(g.weapon("LASER_TINY").unwrap().power, 2);
    assert_eq!(g.drone("DRONE_TINY").unwrap().power, 1);
    assert_eq!(g.weapon("LASER_TINY").unwrap().cooldown_ms, Some(8500));
    assert_eq!(g.drone("DRONE_TINY").unwrap().cooldown, Some(880));
}

#[test]
fn ship_blueprint_carries_systems_and_equipment() {
    let g = load_pkg();
    let ship = g.ship("PLAYER_SHIP_HARD").unwrap();
    let ids: Vec<&str> = ship.systems.iter().map(|s| s.system.as_str()).collect();
    assert_eq!(ids, ["pilot", "shields", "artillery", "artillery"]);
    // The interior image of a room: `img` of the system tag, if any.
    assert_eq!(ship.systems[0].img.as_deref(), Some("room_pilot"));
    assert_eq!(ship.systems[1].img, None);
    assert_eq!(ship.weapons, ["LASER_TINY"]);
    assert_eq!(ship.drones, ["DRONE_TINY"]);
    assert_eq!(ship.augments, ["AUG_TINY"]);
    assert_eq!((ship.weapon_slots, ship.drone_slots), (Some(3), Some(2)));
    assert_eq!((ship.missiles, ship.drone_parts), (8, 2));
    assert_eq!((ship.hull, ship.max_power), (30, 8));
    let crew: Vec<(&str, u32)> = ship
        .crew
        .iter()
        .map(|c| (c.race.as_str(), c.amount))
        .collect();
    assert_eq!(crew, [("human", 2), ("engi", 1)]);
}

#[test]
fn max_system_level_takes_the_ship_override_else_the_system_blueprint() {
    let g = load_pkg();
    assert_eq!(g.max_system_level("PLAYER_SHIP_HARD", "shields"), Some(4));
    assert_eq!(g.max_system_level("PLAYER_SHIP_HARD", "pilot"), Some(3));
    assert_eq!(g.max_system_level("PLAYER_SHIP_HARD", "engines"), None);
    assert_eq!(g.max_system_level("NOPE", "pilot"), None);
}

#[test]
fn player_ships_follow_the_game_order_and_skip_missing_variants() {
    let g = load_pkg();
    let ids: Vec<&str> = g.player_ships().iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, ["PLAYER_SHIP_HARD", "PLAYER_SHIP_ANAEROBIC"]);
}

#[test]
fn ship_layout_has_rooms_doors_and_offsets() {
    let g = load_pkg();
    let l: &ShipLayout = g.layout("tiny").unwrap();
    assert_eq!(
        (l.offset_x, l.offset_y, l.horizontal, l.vertical),
        (1, 2, 3, 4)
    );
    assert_eq!(l.shield_ellipse, Some([100, 50, -5, 6]));
    assert_eq!(l.rooms.len(), 2);
    assert_eq!(
        (l.rooms[1].id, l.rooms[1].x, l.rooms[1].w, l.rooms[1].h),
        (1, 2, 1, 2)
    );
    assert_eq!(l.doors.len(), 1);
    let d = &l.doors[0];
    assert_eq!(
        (d.x, d.y, d.room_a, d.room_b, d.vertical),
        (2, 0, 0, 1, true)
    );
    assert_eq!(l.image, Some([-10, -20, 300, 200]));
    assert_eq!(l.floor_offset, Some([7, 8]));
    assert!(std::ptr::eq(g.ship_layout("PLAYER_SHIP_HARD").unwrap(), l));
}

#[test]
fn sprites_are_png_bytes_by_id() {
    let g = load_pkg();
    assert_eq!(g.crew_sprite("human"), Some(PIXEL_PNG));
    assert_eq!(g.system_icon("shields"), Some(PIXEL_PNG));
    assert_eq!(g.system_icon("pilot"), None);
    let art = g.weapon_art("LASER_TINY").unwrap();
    assert_eq!(art.png, PIXEL_PNG);
    assert_eq!((art.frame_width, art.frame_height, art.frames), (16, 10, 2));
    assert!(g.weapon_art("NOPE").is_none());
}

#[test]
fn a_missing_blueprints_file_names_the_inner_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ftl.dat");
    let mut p = Package::new();
    p.add_entry(PackageEntry::from_string("data/text_blueprints.xml", TEXT))
        .unwrap();
    p.to_path_pkg(&path).unwrap();
    let err = GameData::open(&path).unwrap_err().to_string();
    assert!(err.contains("data/blueprints.xml"), "{err}");
}

#[test]
fn broken_xml_names_the_inner_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ftl.dat");
    let mut p = package();
    p.put_entry(PackageEntry::from_string(
        "data/blueprints.xml",
        "<shipBlueprint name=\"A\"><health amount=\"x\"/>",
    ));
    p.to_path_pkg(&path).unwrap();
    let err = GameData::open(&path).unwrap_err().to_string();
    assert!(err.contains("data/blueprints.xml"), "{err}");
}

#[test]
fn a_file_that_is_no_package_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ftl.dat");
    std::fs::write(&path, b"not a package").unwrap();
    let err = GameData::open(&path).unwrap_err().to_string();
    assert!(err.contains(&path.display().to_string()), "{err}");
}

#[test]
fn a_corrupt_entry_offset_is_an_error_not_a_panic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ftl.dat");
    // A DAT index with one entry whose offset points far past the end.
    let mut bytes = 1u32.to_le_bytes().to_vec();
    bytes.extend(0xFFFF_0000u32.to_le_bytes());
    std::fs::write(&path, bytes).unwrap();
    let err = GameData::open(&path).unwrap_err().to_string();
    assert!(err.contains("corrupt package"), "{err}");
}

/// A sheet of `frames` 35x35 frames in a row: frame i is filled with
/// `[i * 40, 0, 255 - i * 40, 255]`, and its bottom-right pixel is white.
fn crew_sheet(frames: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_fn(35 * frames, 35, |x, y| {
        let i = (x / 35) as u8;
        if x % 35 == 34 && y == 34 {
            image::Rgba([255, 255, 255, 255])
        } else {
            image::Rgba([i * 40, 0, 255 - i * 40, 255])
        }
    });
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

fn load_with(images: &[(&str, Vec<u8>)]) -> GameData {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ftl.dat");
    let mut p = package();
    for (inner, png) in images {
        p.put_entry(PackageEntry::from_byte_array(*inner, png.clone()));
    }
    p.to_path_pkg(&path).unwrap();
    GameData::open(&path).unwrap()
}

fn decode(png: &[u8]) -> image::RgbaImage {
    image::load_from_memory(png).unwrap().to_rgba8()
}

#[test]
fn the_crew_frame_is_the_top_left_35_by_35_of_the_base_sheet() {
    let g = load_with(&[("img/people/human_base.png", crew_sheet(3))]);
    let frame = decode(&g.crew_frame("human").unwrap());
    assert_eq!(frame.dimensions(), (35, 35));
    assert_eq!(frame.get_pixel(0, 0).0, [0, 0, 255, 255]);
    assert_eq!(frame.get_pixel(34, 34).0, [255, 255, 255, 255]);
    // Ghosts are drawn from the human sheet, like in the Java editor.
    assert_eq!(
        g.crew_frame("ghost").unwrap(),
        g.crew_frame("human").unwrap()
    );
}

#[test]
fn the_crew_portrait_centers_the_visible_pixels_of_the_frame() {
    // A 35x35 frame whose only opaque pixels are a 6x3 block at 2..8, 4..7,
    // like the Lanius art that sits left of the cell's middle.
    let frame = image::RgbaImage::from_fn(35, 35, |x, y| {
        let inside = (2..8).contains(&x) && (4..7).contains(&y);
        image::Rgba([200, 10, 10, if inside { 255 } else { 0 }])
    });
    let mut png = std::io::Cursor::new(Vec::new());
    frame.write_to(&mut png, image::ImageFormat::Png).unwrap();
    let g = load_with(&[("img/people/human_base.png", png.into_inner())]);
    let portrait = decode(&g.crew_portrait("human").unwrap());
    assert_eq!(portrait.dimensions(), (35, 35));
    let opaque: Vec<(u32, u32)> = portrait
        .enumerate_pixels()
        .filter(|(_, _, p)| p.0[3] > 0)
        .map(|(x, y, _)| (x, y))
        .collect();
    assert_eq!(opaque.len(), 18);
    // 35 - 6 = 29 columns around it: 15 left, 14 right; 35 - 3 = 32 rows: 16 above.
    assert_eq!(opaque.first(), Some(&(15, 16)));
    assert_eq!(opaque.last(), Some(&(20, 18)));
}

#[test]
fn the_weapon_frame_is_the_first_frame_of_its_strip() {
    let strip = image::RgbaImage::from_fn(32, 10, |x, _| {
        image::Rgba([if x < 16 { 10 } else { 200 }, 0, 0, 255])
    });
    let mut png = std::io::Cursor::new(Vec::new());
    strip.write_to(&mut png, image::ImageFormat::Png).unwrap();
    let g = load_with(&[("img/weapons/laser_tiny_strip2.png", png.into_inner())]);
    let frame = decode(&g.weapon_frame("LASER_TINY").unwrap());
    assert_eq!(frame.dimensions(), (16, 10));
    assert!(frame.pixels().all(|p| p.0 == [10, 0, 0, 255]));
}

#[test]
fn a_sheet_smaller_than_the_frame_and_an_unknown_id_are_errors() {
    // The plain fixture packs 1x1 PNGs.
    let g = load_pkg();
    let err = g.crew_frame("human").unwrap_err().to_string();
    assert!(
        err.contains("img/people/human_base.png") && err.contains("35x35"),
        "{err}"
    );
    let err = g.crew_frame("zoltan_unknown").unwrap_err().to_string();
    assert!(err.contains("zoltan_unknown"), "{err}");
    assert!(
        g.weapon_frame("NOPE")
            .unwrap_err()
            .to_string()
            .contains("NOPE")
    );
}

#[test]
fn image_paths_by_id_follow_the_java_editor() {
    let g = load_pkg();
    // Player ships live in img/ship, enemy ships in img/ships_glow (ShipBundle).
    assert_eq!(
        g.ship_image_paths("PLAYER_SHIP_HARD").unwrap(),
        ["img/ship/tiny_base.png", "img/ships_glow/tiny_base.png"]
    );
    assert_eq!(g.ship_image_paths("NOPE"), None);
    assert_eq!(
        g.drone_image_paths("DRONE_TINY").unwrap(),
        [
            "img/ship/drones/drone_tiny_base.png",
            "img/ship/drones/drone_tiny_off.png"
        ]
    );
    assert_eq!(g.drone_image_paths("NOPE"), None);
    assert_eq!(
        g.achievement_image_path("ACH_TINY"),
        Some("img/achievements/tiny.png")
    );
    assert_eq!(g.achievement_image_path("NOPE"), None);
}

#[test]
fn achievements_carry_names_kinds_and_the_ship_victories() {
    let g = load_pkg();
    let ids: Vec<_> = g.achievements().iter().map(|a| a.id.as_str()).collect();
    // achievements.xml first, then the victories of the blueprints present.
    assert_eq!(
        ids,
        [
            "ACH_TINY",
            "ACH_TINY_SHIP",
            "PLAYER_SHIP_HARD_VICTORY",
            "PLAYER_SHIP_ANAEROBIC_VICTORY"
        ]
    );
    let tiny = g.achievement("ACH_TINY").unwrap();
    assert_eq!(
        (tiny.name.as_str(), tiny.description.as_str(), tiny.kind),
        ("Small Start", "Reach sector 2.", AchievementKind::General)
    );
    let ship = g.achievement("ACH_TINY_SHIP").unwrap();
    assert_eq!(ship.kind, AchievementKind::Ship);
    assert_eq!(ship.ship.as_deref(), Some("PLAYER_SHIP_HARD"));
    assert_eq!(ship.name, "Own name");
    let v = g.achievement("PLAYER_SHIP_HARD_VICTORY").unwrap();
    assert_eq!(v.kind, AchievementKind::Victory);
    assert_eq!(v.name, "Tiny Cruiser Victory");
    assert_eq!(
        v.description,
        "Win a made-up run with any layout of the Tiny Cruiser"
    );
    assert_eq!(v.image.as_deref(), Some("img/achievements/S_V_on.png"));
    // German: the translation, else the English text.
    assert_eq!(
        g.achievement_text_in("ACH_TINY", "de"),
        Some(("Kleiner Anfang", ""))
    );
    assert_eq!(
        g.achievement_text_in("PLAYER_SHIP_HARD_VICTORY", "de")
            .unwrap()
            .0,
        "Winzkreuzer-Sieg"
    );
    assert_eq!(
        g.achievement_text_in("ACH_TINY_SHIP", "de").unwrap().0,
        "Own name"
    );
    // The Lanius class has no text id: its English class fills the German template.
    assert_eq!(
        g.achievement_text_in("PLAYER_SHIP_ANAEROBIC_VICTORY", "de")
            .unwrap()
            .0,
        "Lanius Cruiser-Sieg"
    );
    assert_eq!(g.achievement_text_in("NOPE", "de"), None);
}

#[test]
fn read_entry_reads_one_entry_and_names_a_missing_one() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_pkg(dir.path());
    assert_eq!(
        ftl_assets::read_entry(&path, "img/ship/tiny_base.png").unwrap(),
        PIXEL_PNG
    );
    let err = ftl_assets::read_entry(&path, "img/ship/none_base.png").unwrap_err();
    assert!(
        matches!(&err, ftl_assets::Error::Missing { inner_path } if inner_path == "img/ship/none_base.png"),
        "{err}"
    );
    let err = ftl_assets::read_entry(dir.path().join("nothing.dat"), "x")
        .unwrap_err()
        .to_string();
    assert!(err.contains("nothing.dat"), "{err}");
}

fn solid(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let img = image::RgbaImage::from_pixel(w, h, image::Rgba(rgba));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

#[test]
fn crew_colors_and_names_come_from_the_blueprints_and_names_xml() {
    let g = load_pkg();
    let layers = &g.crew("human").unwrap().color_layers;
    assert_eq!(layers.iter().map(Vec::len).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(
        layers[0][0],
        ftl_assets::Rgba {
            r: 140,
            g: 140,
            b: 255,
            a: 179
        }
    );
    assert_eq!(g.crew_names(true), ["Kirby", "Jon"]);
    assert_eq!(g.crew_names(false), ["Layla"]);
}

#[test]
fn the_crew_look_lays_the_tinted_layers_over_the_sheet_of_the_sex() {
    let g = load_with(&[
        (
            "img/people/human_base.png",
            solid(35, 35, [10, 10, 10, 255]),
        ),
        (
            "img/people/female_base.png",
            solid(35, 35, [20, 20, 20, 255]),
        ),
        (
            "img/people/human_layer2.png",
            solid(35, 35, [255, 255, 255, 255]),
        ),
    ]);
    let pixel = |png: Vec<u8>| decode(&png).get_pixel(17, 17).0;
    // Layer 2, color 0: opaque red over the base.
    assert_eq!(
        pixel(g.crew_look_frame("human", true, &[0, 0]).unwrap()),
        [255, 0, 0, 255]
    );
    // Color 1 has alpha 0: the base shows.
    assert_eq!(
        pixel(g.crew_look_frame("human", true, &[0, 1]).unwrap()),
        [10, 10, 10, 255]
    );
    // A female human wears the female sheet (no layer sheet there).
    assert_eq!(
        pixel(g.crew_look_frame("human", false, &[0, 1]).unwrap()),
        [20, 20, 20, 255]
    );
}
