//! Race names of the crew roster from the local game's `ftl.dat` (`FTL_DAT`
//! or the macOS Steam install), else a skip line.

use std::path::PathBuf;

use ftl_app::{SaveFile, ShipId, assets, crew};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
);

#[test]
fn the_fixture_crew_gets_the_race_titles_of_the_game() {
    let path = match std::env::var_os("FTL_DAT") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(
            "Library/Application Support/Steam/steamapps/common/FTL Faster Than Light/FTL.app/Contents/Resources/ftl.dat",
        ),
    };
    if !path.is_file() {
        println!(
            "skip: no ftl.dat at {} (set FTL_DAT to test against the game)",
            path.display()
        );
        return;
    }
    let data = assets::load(&path).unwrap();
    let mut file = SaveFile::open(FIXTURE).unwrap();
    let titles = |file: &SaveFile, on: ShipId| -> Vec<Option<String>> {
        crew::roster(file.game(), on, Some(&data))
            .into_iter()
            .map(|c| c.race_name)
            .collect()
    };
    let human = || Some("Human".to_owned());
    assert_eq!(titles(&file, ShipId::Player), [human(), human(), human()]);
    assert_eq!(
        titles(&file, ShipId::Nearby),
        [Some("Rockman".to_owned()), Some("Mantis".to_owned())]
    );
    // A title that is not the race id: the Lanius are `anaerobic`.
    file.apply(&ftl_app::Patch::new(vec![
        "crew.2.race=anaerobic".parse().unwrap(),
    ]))
    .unwrap();
    assert_eq!(
        titles(&file, ShipId::Player),
        [human(), human(), Some("Lanius".to_owned())]
    );
}

fn real_dat() -> Option<PathBuf> {
    let path = match std::env::var_os("FTL_DAT") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(
            "Library/Application Support/Steam/steamapps/common/FTL Faster Than Light/FTL.app/Contents/Resources/ftl.dat",
        ),
    };
    if path.is_file() {
        Some(path)
    } else {
        println!("skip: no ftl.dat at {}", path.display());
        None
    }
}

/// The tint layers of `RACES` are those of the crew blueprints, the names
/// those of names.xml, and the look images follow sex and tint.
#[test]
fn the_crew_look_and_names_come_from_the_game() {
    let Some(path) = real_dat() else { return };
    let data = assets::load(&path).unwrap();
    for race in ftl_app::RACES {
        let sizes: Vec<usize> = data
            .crew(race.id)
            .unwrap()
            .color_layers
            .iter()
            .map(Vec::len)
            .collect();
        assert_eq!(sizes, race.color_layers, "{}", race.id);
    }
    assert_eq!(&data.crew_names(true)[..2], ["Kirby", "Ma Fan"]);
    assert_eq!(&data.crew_names(false)[..2], ["T'ten", "Layla"]);

    let male = data.crew_look_frame("human", true, &[0, 1]).unwrap();
    let female = data.crew_look_frame("human", false, &[0, 1]).unwrap();
    let other_hair = data.crew_look_frame("human", true, &[0, 0]).unwrap();
    assert_ne!(male, female, "a female human wears the female sheet");
    assert_ne!(male, other_hair, "the tint changes the picture");
    // Every look of the character creator has a picture.
    for race in ftl_app::RACES {
        for look in crew::look_choices(race.id) {
            data.crew_look(race.id, look.male, &look.tints)
                .unwrap_or_else(|e| panic!("{}: {e}", look.look_id));
        }
    }
    assert_eq!(crew::look_choices("human").len(), 8);
    assert_eq!(crew::look_choices("engi").len(), 1);
}
