//! The one-click actions on the fixture with the limits of the local game's
//! `ftl.dat` (`FTL_DAT` or the macOS Steam install), else a skip line.

use std::path::PathBuf;

use ftl_app::actions::{self, Action, ActionNote};
use ftl_app::{SaveFile, assets};

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
fn max_everything_with_the_real_blueprint_needs_no_editor_cap() {
    let Some(path) = real_dat() else { return };
    let data = assets::load(&path).unwrap();
    let mut file = SaveFile::open(FIXTURE).unwrap();
    let limits = actions::limits(file.game(), Some(&data));
    let hull = data
        .ship(&file.game().player_ship.blueprint_id)
        .unwrap()
        .hull as i32;
    assert_eq!(limits.max_hull, Some(hull));

    let plan = actions::plan(Action::MaxEverything, file.game(), Some(&data));
    assert!(
        !plan.notes.iter().any(|n| matches!(
            n,
            ActionNote::SystemsAtEditorCap | ActionNote::HullAtEditorCap { .. }
        )),
        "{:?}",
        plan.notes
    );
    file.apply(&plan.patch).unwrap();
    file.verify().unwrap();
    assert_eq!(file.game().player_ship.resources.hull, hull);
    let after = actions::plan(Action::MaxEverything, file.game(), Some(&data));
    assert!(after.patch.assignments.is_empty());
}
