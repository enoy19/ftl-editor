//! The ship types of the fixture's Kestrel against the local game's
//! `ftl.dat` (`FTL_DAT` or the macOS Steam install), else a skip line.

use std::path::PathBuf;

use ftl_app::identity::ship_type_choices;
use ftl_app::{Assignment, Field, Patch, SaveFile, assets};
use ftl_core::identity::TypeBlock;

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

/// No two hangar ships share a layout file, so every other hangar type is
/// refused for its rooms and doors; the unused `PLAYER_SHIP_EASY` shares
/// `kestral` and its systems with Kestrel A and is taken.
#[test]
fn every_other_hangar_type_has_other_rooms_and_the_easy_kestrel_fits() {
    let Some(path) = real_dat() else { return };
    let data = assets::load(&path).unwrap();
    let file = SaveFile::open(FIXTURE).unwrap();
    let choices = ship_type_choices(file.game(), Some(&data), file.layouts(), "en");
    assert_eq!(choices.len(), 28);
    let own = choices.iter().find(|c| c.id == "PLAYER_SHIP_HARD").unwrap();
    assert_eq!(
        (own.name.as_str(), own.layout, own.block),
        ("The Kestrel", 'A', None)
    );
    let b = choices
        .iter()
        .find(|c| c.id == "PLAYER_SHIP_HARD_2")
        .unwrap();
    assert_eq!((b.name.as_str(), b.layout), ("Red-Tail", 'B'));
    for c in choices.iter().filter(|c| c.id != "PLAYER_SHIP_HARD") {
        assert_eq!(c.block, Some(TypeBlock::OtherLayout), "{}", c.id);
    }
    let easy = Patch::new(vec![Assignment::new(Field::ShipType, "PLAYER_SHIP_EASY")]);
    let g = file.preview(&easy).unwrap();
    assert_eq!(g.player_ship.blueprint_id, "PLAYER_SHIP_EASY");
    assert_eq!(g.player_ship.gfx_base_name, "kestral");
}
