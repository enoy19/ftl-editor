//! Reads the local game's `ftl.dat` when it exists (`FTL_DAT` or the macOS
//! Steam install), else prints a skip line. Nothing extracted is kept.

use std::path::PathBuf;

use ftl_assets::GameData;

const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";

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
fn reads_every_blueprint_kind_of_the_real_game() {
    let Some(path) = real_dat() else { return };
    let g = GameData::open(&path).unwrap();
    println!(
        "{}: ships {}, crew {}, systems {}, weapons {}, drones {}, augments {}",
        path.display(),
        g.ships().count(),
        g.crew_races().count(),
        g.systems().count(),
        g.weapons().count(),
        g.drones().count(),
        g.augments().count(),
    );
    let no_layout: Vec<&str> = g
        .ships()
        .filter(|s| g.ship_layout(&s.id).is_none())
        .map(|s| s.id.as_str())
        .collect();
    assert!(
        no_layout.is_empty(),
        "ships without a layout file: {no_layout:?}"
    );
    assert!(g.ships().count() > 100);
    assert!(g.crew_races().count() >= 9);
    assert!(g.systems().count() >= 16);
    assert!(g.weapons().count() > 50);
    assert!(g.drones().count() > 10);
    assert!(g.augments().count() > 20);
    // Every player ship has a name and a layout with rooms.
    for ship in g.player_ships() {
        assert!(!ship.name.is_empty(), "{} has no name", ship.id);
        let layout = g
            .ship_layout(&ship.id)
            .unwrap_or_else(|| panic!("{} has no layout", ship.id));
        assert!(
            !layout.rooms.is_empty() && !layout.doors.is_empty(),
            "{}",
            ship.id
        );
        assert!(layout.image.is_some(), "{}", ship.id);
    }
}

#[test]
fn knows_the_kestrel_and_the_dlc_ships() {
    let Some(path) = real_dat() else { return };
    let g = GameData::open(&path).unwrap();
    let kestrel = g.ship("PLAYER_SHIP_HARD").unwrap();
    assert_eq!(kestrel.name, "The Kestrel");
    assert_eq!(kestrel.layout, "kestral");
    assert_eq!(g.max_system_level("PLAYER_SHIP_HARD", "shields"), Some(8));
    assert_eq!(g.display_name("LASER_BURST_3"), Some("Burst Laser Mark II"));
    // 10 families with Type A and B, 8 with Type C (not Crystal, not Lanius).
    let ids: Vec<&str> = g.player_ships().iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids.first(), Some(&"PLAYER_SHIP_HARD"));
    assert_eq!(ids.len(), 28, "{ids:?}");
    assert!(!ids.contains(&"PLAYER_SHIP_ANAEROBIC_3"), "{ids:?}");
    // The Java probe failed on this enemy ship without the DLC files.
    assert!(g.ship("REBEL_SKINNY_ELITE_DLC").is_some());
}

#[test]
fn answers_png_bytes_for_crew_systems_and_weapons() {
    let Some(path) = real_dat() else { return };
    let g = GameData::open(&path).unwrap();
    for race in [
        "human",
        "engi",
        "mantis",
        "slug",
        "rock",
        "energy",
        "crystal",
        "anaerobic",
    ] {
        assert!(
            g.crew_sprite(race).unwrap().starts_with(PNG_MAGIC),
            "{race}"
        );
    }
    for system in [
        "shields", "engines", "oxygen", "weapons", "pilot", "cloaking", "hacking",
    ] {
        assert!(
            g.system_icon(system).unwrap().starts_with(PNG_MAGIC),
            "{system}"
        );
    }
    let art = g.weapon_art("LASER_BURST_1").unwrap();
    assert!(art.png.starts_with(PNG_MAGIC));
    assert_eq!(
        (art.frame_width, art.frame_height, art.frames),
        (16, 60, 12)
    );
    // `<mountPoint x="2" y="44"/>` of `laser_burst_1`.
    assert_eq!(art.mount_point, (2, 44));
}

#[test]
fn reads_the_weapon_mounts_of_the_real_layouts() {
    use ftl_assets::{Slide, WeaponMount};
    let Some(path) = real_dat() else { return };
    let g = GameData::open(&path).unwrap();
    // data/kestral.xml: four mounts, then four for artillery tests.
    let kestrel = g.layout("kestral").unwrap();
    assert_eq!(kestrel.weapon_mounts.len(), 8);
    assert_eq!(
        kestrel.weapon_mounts[..2],
        [
            WeaponMount {
                x: 540,
                y: 155,
                rotate: true,
                mirror: true,
                slide: Slide::Up
            },
            WeaponMount {
                x: 540,
                y: 262,
                rotate: true,
                mirror: false,
                slide: Slide::Down
            },
        ]
    );
    // The pirate of the fixture: upright mounts that slide sideways.
    let jelly = g.ship_layout("JELLY_BUTTON_P_DLC").unwrap();
    assert_eq!(
        jelly.weapon_mounts[0],
        WeaponMount {
            x: 52,
            y: 136,
            rotate: false,
            mirror: true,
            slide: Slide::Left
        }
    );
    // Every ship with a weapons system has a mount per weapon slot (4 when
    // the blueprint names none, as FTL's default).
    let short: Vec<String> = g
        .ships()
        .filter(|s| s.systems.iter().any(|x| x.system == "weapons"))
        .filter_map(|s| {
            let mounts = g.ship_layout(&s.id)?.weapon_mounts.len();
            let slots = s.weapon_slots.unwrap_or(4) as usize;
            (mounts < slots).then(|| format!("{} {mounts}<{slots}", s.id))
        })
        .collect();
    assert!(short.is_empty(), "{short:?}");
    // Every weapon of the game draws with a sheet, except `CRYSTAL_1`, the
    // lockdown shard of the Crystal crew, whose `weaponArt` names a crew
    // `anim`, no `weaponAnim`: no slot ever holds it.
    let no_art: Vec<&str> = g
        .weapons()
        .filter(|w| w.id != "CRYSTAL_1")
        .filter(|w| w.weapon_art.is_some() && g.weapon_art(&w.id).is_none())
        .map(|w| w.id.as_str())
        .collect();
    assert!(no_art.is_empty(), "{no_art:?}");
}

#[test]
fn cuts_the_standing_frame_of_every_crew_race() {
    let Some(path) = real_dat() else { return };
    let g = GameData::open(&path).unwrap();
    let races: Vec<String> = g
        .crew_races()
        .map(|c| c.id.clone())
        .chain(["female".into(), "ghost".into()])
        .collect();
    for race in &races {
        let png = g.crew_frame(race).unwrap_or_else(|e| panic!("{race}: {e}"));
        let frame = image::load_from_memory(&png).unwrap();
        assert_eq!((frame.width(), frame.height()), (35, 35), "{race}");
        assert!(
            frame.to_rgba8().pixels().any(|p| p.0[3] > 0),
            "{race}: the frame is empty"
        );
    }
    for race in &races {
        // The portrait's visible box is centered: its margins differ by at
        // most one pixel on each axis.
        let portrait = image::load_from_memory(&g.crew_portrait(race).unwrap())
            .unwrap()
            .to_rgba8();
        let opaque = || portrait.enumerate_pixels().filter(|(_, _, p)| p.0[3] > 0);
        let (l, r) = (
            opaque().map(|p| p.0).min().unwrap(),
            opaque().map(|p| p.0).max().unwrap(),
        );
        let (t, b) = (
            opaque().map(|p| p.1).min().unwrap(),
            opaque().map(|p| p.1).max().unwrap(),
        );
        assert!(
            (l as i32 - (34 - r) as i32).abs() <= 1,
            "{race}: x {l}..{r}"
        );
        assert!(
            (t as i32 - (34 - b) as i32).abs() <= 1,
            "{race}: y {t}..{b}"
        );
    }
    println!("cut {} crew frames: {races:?}", races.len());
    let art = g.weapon_frame("LASER_BURST_1").unwrap();
    let frame = image::load_from_memory(&art).unwrap();
    assert_eq!((frame.width(), frame.height()), (16, 60));
}

#[test]
fn every_player_ship_drone_and_achievement_image_path_exists() {
    let Some(path) = real_dat() else { return };
    let g = GameData::open(&path).unwrap();
    let read = |inner: &str| ftl_assets::read_entry(&path, inner);
    for ship in g.player_ships() {
        let [plain, _glow] = g.ship_image_paths(&ship.id).unwrap();
        assert!(
            read(&plain)
                .unwrap_or_else(|e| panic!("{}: {e}", ship.id))
                .starts_with(PNG_MAGIC)
        );
    }
    // Enemy hulls live in img/ships_glow since FTL 1.5.4.
    let [plain, glow] = g.ship_image_paths("REBEL_SKINNY_ELITE_DLC").unwrap();
    assert!(
        read(&plain).is_err() && read(&glow).unwrap().starts_with(PNG_MAGIC),
        "{plain} / {glow}"
    );
    let first_png = |candidates: &[String]| {
        candidates
            .iter()
            .any(|c| read(c).is_ok_and(|png| png.starts_with(PNG_MAGIC)))
    };
    let drones: Vec<[String; 2]> = g
        .drones()
        .filter_map(|d| g.drone_image_paths(&d.id))
        .collect();
    assert!(drones.len() > 10, "{drones:?}");
    let missing: Vec<&[String; 2]> = drones.iter().filter(|c| !first_png(*c)).collect();
    assert!(missing.is_empty(), "{missing:?}");
    // Every achievement has an image, the ship quests and victories too.
    // 51 in achievements.xml (ACH_LANIUS_SUFFOCATION is commented out).
    assert_eq!(g.achievements().len(), 51 + 7 + 10);
    let no_image: Vec<&str> = g
        .achievements()
        .iter()
        .filter(|a| !first_png(&[a.image.clone().unwrap_or_default()]))
        .map(|a| a.id.as_str())
        .collect();
    assert!(no_image.is_empty(), "{no_image:?}");
}

#[test]
fn every_achievement_has_a_name_in_english_and_german() {
    let Some(path) = real_dat() else { return };
    let g = GameData::open(&path).unwrap();
    for a in g.achievements() {
        assert!(!a.name.is_empty() && !a.name.starts_with("ACH_"), "{a:?}");
        assert!(!a.description.is_empty(), "{a:?}");
        let (de, de_desc) = g.achievement_text_in(&a.id, "de").unwrap();
        assert!(!de.is_empty() && !de_desc.is_empty(), "{}", a.id);
    }
    let v = g.achievement("PLAYER_SHIP_HARD_VICTORY").unwrap();
    assert_eq!(v.name, "Kestrel Cruiser Victory");
    let (quest, _) = g
        .achievement_text_in("PLAYER_SHIP_STEALTH_QUEST", "de")
        .unwrap();
    assert!(quest.ends_with("-Auftrag"), "{quest}");
    assert_eq!(
        g.achievement_text_in("ACH_SECTOR_5", "de").unwrap().0,
        "Es geht los!"
    );
}

#[test]
fn decodes_every_latin_font_of_the_real_game() {
    let Some(path) = real_dat() else { return };
    for name in [
        "JustinFont8",
        "JustinFont10",
        "JustinFont11Bold",
        "JustinFont12Bold",
        "HL1",
        "HL2",
        "num_font",
    ] {
        let bytes = ftl_assets::read_entry(&path, &format!("fonts/{name}.font")).unwrap();
        let f = ftl_assets::BitmapFont::parse(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        // Not every font has every ASCII sign (JustinFont11Bold lacks `{`).
        for c in ('0'..='9')
            .chain('A'..='Z')
            .chain('a'..='z')
            .chain(" .,:!?-+/%()ÄÖÜäöüß".chars())
        {
            let g = f.glyph(c).unwrap_or_else(|| panic!("{name} lacks {c:?}"));
            assert!(g.advance() > 0, "{name} {c:?}");
        }
        assert!(
            f.coverage.iter().all(|&a| a == 0 || a == 255),
            "{name}: coverage is 0 or 255"
        );
        assert!(f.atlas_png().starts_with(PNG_MAGIC));
        println!(
            "{name}: {} glyphs, atlas {}x{}, ascent {}, descent {}, line height {}, baseline {}",
            f.glyphs.len(),
            f.atlas_width,
            f.atlas_height,
            f.ascent(),
            f.descent(),
            f.line_height,
            f.baseline
        );
    }
}

#[test]
fn measures_the_insets_of_the_real_frames() {
    let Some(path) = real_dat() else { return };
    let insets = |p: &str| {
        ftl_assets::measure_insets(p, &ftl_assets::read_entry(&path, p).unwrap()).unwrap()
    };
    let ltrb = |i: ftl_assets::Insets| (i.left, i.top, i.right, i.bottom);
    for p in [
        "img/window_outline.png",
        "img/window_base.png",
        "img/optionsUI/tab_page_on.png",
        "img/optionsUI/tab_page_off.png",
        "img/customizeUI/button_accept_on.png",
        "img/customizeUI/box_crew_details.png",
    ] {
        println!("{p}: {:?}", ltrb(insets(p)));
    }
    // The straight edge starts at 19; the glow of the cut corners reaches to 30 and 35.
    assert_eq!(ltrb(insets("img/window_outline.png")), (31, 36, 32, 35));
    // A tab has slanted sides and no uniform row: one middle row is left.
    assert_eq!(
        ltrb(insets("img/optionsUI/tab_page_on.png")),
        (33, 14, 33, 14)
    );
    assert_eq!(
        ltrb(insets("img/customizeUI/button_accept_on.png")),
        (2, 2, 2, 2)
    );
}

#[test]
fn reads_power_and_german_names_of_the_real_game() {
    let Some(path) = real_dat() else { return };
    let g = GameData::open(&path).unwrap();
    assert_eq!(g.weapon("ION_4").unwrap().power, 3);
    assert_eq!(g.drone("ANTI_DRONE").unwrap().power, 1);
    assert_eq!(g.weapon("SHOTGUN_2").unwrap().cooldown_ms, Some(21000));
    assert_eq!(g.drone("ANTI_DRONE").unwrap().cooldown, Some(7000));
    assert_eq!(g.drone("BOARDER").unwrap().cooldown, None);
    assert_eq!(g.display_name_in("shields", "de"), Some("Schilde"));
    assert_eq!(g.display_name_in("ION_4", "de"), Some("Ionenschlag II"));
    assert_eq!(
        g.display_name_in("NANO_MEDBAY", "de"),
        Some("Engi-Medbot-Verteiler")
    );
}
