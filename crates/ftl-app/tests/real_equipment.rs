//! The equipment of the fixture with the names and limits of the local
//! game's `ftl.dat` (`FTL_DAT` or the macOS Steam install), else a skip line.

use std::path::PathBuf;

use ftl_app::{SaveFile, ShipId, assets, equipment};

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
fn the_fixture_loadout_has_names_blueprint_limits_and_items() {
    let Some(path) = real_dat() else { return };
    let data = assets::load(&path).unwrap();
    let mut file = SaveFile::open(FIXTURE).unwrap();
    let l = equipment::loadout(
        file.game(),
        ShipId::Player,
        Some(&data),
        file.layouts(),
        "en",
    )
    .unwrap();

    // The Kestrel lists drones, clone bay, battery and more without having them.
    let systems: Vec<_> = l
        .systems
        .iter()
        .map(|s| {
            (
                s.kind.id(),
                s.name.as_deref().unwrap(),
                s.level.as_ref().map(|b| (b.value, b.max)),
            )
        })
        .collect();
    assert_eq!(&systems[0], &("shields", "Shields", Some((2, 8))));
    assert!(systems.contains(&("clonebay", "Clone Bay", None)));
    assert!(systems.contains(&("battery", "Backup Battery", None)));
    assert!(!systems.iter().any(|s| s.0 == "artillery"));
    assert!(l.body_decoded);
    let weapons: Vec<_> = l
        .weapons
        .iter()
        .map(|w| (w.id.as_str(), w.name.as_deref().unwrap(), w.power, w.armed))
        .collect();
    assert_eq!(
        weapons,
        [
            ("MISSILES_2_PLAYER", "Artemis Missiles", Some(1), true),
            ("LASER_BURST_3", "Burst Laser Mark II", Some(2), true)
        ]
    );
    assert!(l.drones.is_empty() && l.augments.is_empty());
    assert!(l.cargo.as_ref().unwrap().is_empty());
    assert_eq!((l.weapon_slots, l.drone_slots), (Some(4), Some(2)));
    let fresh =
        equipment::item_fields(file.game(), ShipId::Player, file.layouts(), "LASER_BURST_3")
            .unwrap();
    let range = |b: &Option<ftl_app::crew::Bounded>| b.as_ref().map(|b| (b.value, b.min, b.max));
    assert_eq!(range(&fresh.slots), Some((1, 0, 3)));

    // The fixture has no drone, augment, cargo, battery or third weapon:
    // build them through the edit paths.
    file.apply(&ftl_app::Patch::new(
        [
            "systems.drones.installed=1",
            "systems.battery.installed=1",
            "drones.COMBAT_1=1",
            "drones.ANTI_DRONE=1",
            "weapons.ION_4=1",
            "augments.NANO_MEDBAY=1",
            "cargo.REPAIR=1",
        ]
        .iter()
        .map(|a| a.parse().unwrap())
        .collect(),
    ))
    .unwrap();
    let l = equipment::loadout(
        file.game(),
        ShipId::Player,
        Some(&data),
        file.layouts(),
        "en",
    )
    .unwrap();
    let battery = l.systems.iter().find(|s| s.kind.id() == "battery").unwrap();
    assert_eq!(
        battery.level.as_ref().map(|b| (b.value, b.max)),
        Some((1, 2))
    );
    let weapons: Vec<_> = l
        .weapons
        .iter()
        .map(|w| (w.id.as_str(), w.name.as_deref().unwrap(), w.power, w.armed))
        .collect();
    assert_eq!(
        weapons[2],
        ("ION_4", "Ion Blast Mark II", Some(3), false),
        "a new weapon starts unarmed"
    );
    assert_eq!(
        l.drones.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(),
        ["COMBAT_1", "ANTI_DRONE"]
    );
    assert_eq!(
        (
            l.augments[0].id.as_str(),
            l.augments[0].name.as_deref(),
            l.augments[0].count.value
        ),
        ("NANO_MEDBAY", Some("Engi Med-bot Dispersal"), 1)
    );
    // The repair drone in the cargo; 3 of the 4 boxes are free.
    let cargo = l.cargo.as_ref().unwrap();
    assert_eq!(
        (
            cargo[0].id.as_str(),
            cargo[0].kind,
            cargo[0].name.as_deref()
        ),
        (
            "REPAIR",
            equipment::ItemKind::Drone,
            Some("System Repair Drone")
        )
    );
    let ion = l.item_fields.iter().find(|f| f.id == "ION_4").unwrap();
    assert_eq!(
        (range(&ion.slots), range(&ion.armed), range(&ion.cargo)),
        (Some((1, 0, 2)), Some((0, 0, 0)), Some((0, 0, 3)))
    );
    let repair = l.item_fields.iter().find(|f| f.id == "REPAIR").unwrap();
    // Two of two drone slots are full; a repair drone is armed in FTL.
    assert_eq!(
        (
            range(&repair.slots),
            range(&repair.armed),
            range(&repair.cargo)
        ),
        (Some((0, 0, 0)), Some((0, 0, 0)), Some((1, 0, 4)))
    );
    assert!(
        equipment::item_fields(file.game(), ShipId::Player, file.layouts(), "DRONE_HACKING")
            .is_none()
    );
    let items = equipment::item_choices(Some(&data), "de");
    assert!(
        items
            .iter()
            .any(|c| c.id == "ION_4" && c.name == "Ionenschlag II")
    );
    assert!(items.iter().any(|c| c.id == "COMBAT_1"));
    assert!(
        !items
            .iter()
            .any(|c| c.id == "DRONE_HACKING" || c.id == "DRONE_LASER")
    );

    let de = equipment::loadout(
        file.game(),
        ShipId::Player,
        Some(&data),
        file.layouts(),
        "de",
    )
    .unwrap();
    assert_eq!(de.systems[0].name.as_deref(), Some("Schilde"));
    let choices = equipment::augment_choices(Some(&data), "en");
    assert!(choices.len() > 20, "{} augments", choices.len());
    assert!(
        choices
            .iter()
            .any(|c| c.id == "NANO_MEDBAY" && c.name == "Engi Med-bot Dispersal")
    );
}

/// The fixture ship holds 5 of 8 systems; with drones, teleporter and
/// cloaking bought it holds 8: then hacking and mind control wait for a
/// free place, the clone bay can replace the medbay, the bought systems
/// can go and the start systems stay.
#[test]
fn the_fixture_loadout_says_which_systems_can_be_installed_or_removed() {
    use ftl_core::installs::InstallBlock;
    let Some(path) = real_dat() else { return };
    let data = assets::load(&path).unwrap();
    let mut file = SaveFile::open(FIXTURE).unwrap();
    let rows = |file: &SaveFile, ids: &[&str]| {
        let l = equipment::loadout(
            file.game(),
            ShipId::Player,
            Some(&data),
            file.layouts(),
            "en",
        )
        .unwrap();
        ids.iter()
            .map(|id| {
                let s = l.systems.iter().find(|s| s.kind.id() == *id).unwrap();
                (
                    (
                        s.installed.field.path(),
                        s.installed.value,
                        s.installed.min,
                        s.installed.max,
                    ),
                    s.block,
                )
            })
            .collect::<Vec<_>>()
    };
    let row = |id: &str, value, min, max, block| {
        ((format!("systems.{id}.installed"), value, min, max), block)
    };
    assert_eq!(
        rows(&file, &["teleporter", "mind", "hacking"]),
        [
            row("teleporter", 0, 0, 1, None),
            row("mind", 0, 0, 1, None),
            row("hacking", 0, 0, 1, None)
        ]
    );
    file.apply(&ftl_app::Patch::new(
        [
            "systems.drones.installed=1",
            "systems.teleporter.installed=1",
            "systems.cloaking.installed=1",
        ]
        .iter()
        .map(|a| a.parse().unwrap())
        .collect(),
    ))
    .unwrap();
    assert_eq!(
        rows(
            &file,
            &[
                "mind",
                "hacking",
                "clonebay",
                "teleporter",
                "shields",
                "medbay"
            ]
        ),
        [
            row("mind", 0, 0, 0, Some(InstallBlock::SystemLimit)),
            row("hacking", 0, 0, 0, Some(InstallBlock::SystemLimit)),
            row("clonebay", 0, 0, 1, None),
            row("teleporter", 1, 0, 1, None),
            row("shields", 1, 1, 1, Some(InstallBlock::StartingSystem)),
            row("medbay", 1, 1, 1, Some(InstallBlock::SwapBack)),
        ]
    );
}
