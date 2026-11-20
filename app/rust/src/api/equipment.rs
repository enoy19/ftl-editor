//! The equipment of the open save for the Equipment page. A thin adapter
//! over `ftl_app::equipment`; edits go through [`SaveSession::apply`] with the
//! paths `reactor`, `systems.<id>.level`, `systems.<id>.installed`,
//! `systems.<id>.damage`, `systems.<id>.ion`, `augments.<ID>`, `weapons.<ID>`,
//! `weapons.<ID>.armed`, `drones.<ID>`, `drones.<ID>.armed` and `cargo.<ID>`.
//! [`equipment_preview`] shows the equipment as the pending edits leave it.

use flutter_rust_bridge::frb;
use ftl_app::assets;
use ftl_app::equipment;

use super::editor::{EditorError, FieldEdit, FieldValue, SaveSession, patch_of};
use super::field::{field_value, ship};

/// One system of the player ship.
pub struct EquipmentSystem {
    /// The system id (`shields`), also in `img/icons/s_<id>_*.png`.
    pub id: String,
    /// The title from the open `ftl.dat`; `None` in reduced mode.
    pub name: Option<String>,
    /// `systems.<id>.level` with its range (from 1 to the cap of the ship
    /// blueprint; below the power it lowers the power); `None` when the
    /// system is not installed.
    pub level: Option<FieldValue>,
    /// `systems.<id>.installed` (0 or 1): 0 to 1 when the system can be
    /// installed or removed now, 1 to 1 for one that stays, 0 to 0 for a
    /// missing one that cannot be installed (`block` says why).
    pub installed: FieldValue,
    /// Why the system cannot be installed (missing) or removed (installed)
    /// now, a stable code of `ftl_core::installs::InstallBlock`
    /// (`system_limit`, `starting_system`, `swap_back`, ...); `None` when
    /// it can.
    pub block: Option<String>,
    /// Bars the system draws now.
    pub power: i32,
    /// Broken bars.
    pub damaged: i32,
    /// Bars that ion damage locks; -1 is a lock without a count.
    pub ionized: i32,
    /// `systems.<id>.damage`: broken bars, 0 to the level; `None` when the
    /// system is not installed.
    pub damage: Option<FieldValue>,
    /// `systems.<id>.ion`: the ion lock count, 0 to 5; `None` when the
    /// system is not installed.
    pub ion: Option<FieldValue>,
    /// `systems.shields.energy`: the Zoltan shield points, 0 to 5 (the
    /// enemy up to its own max); `None` on every other system and on
    /// shields that are not installed.
    pub energy: Option<FieldValue>,
    /// Pilot, sensors, doors and battery: no reactor power.
    pub subsystem: bool,
}

/// A weapon or drone of the player ship, in a slot or in the cargo.
pub struct EquipmentItem {
    /// Blueprint id, also the id of its image (`GameImageKind`).
    pub id: String,
    /// A drone, else a weapon.
    pub drone: bool,
    pub name: Option<String>,
    /// Reactor bars from the blueprint; `None` in reduced mode.
    pub power: Option<u32>,
    /// Powered; `false` in the cargo.
    pub armed: bool,
}

/// The edit paths of one weapon or drone id; `None` where it cannot be
/// edited (reduced mode, an undecoded part, no weapons or drone system).
pub struct EquipmentItemFields {
    pub id: String,
    pub drone: bool,
    /// `weapons.<ID>` or `drones.<ID>`: copies in the slots, up to the free slots.
    pub slots: Option<FieldValue>,
    /// `weapons.<ID>.armed` or `drones.<ID>.armed`: armed copies, up to the free bars.
    pub armed: Option<FieldValue>,
    /// `cargo.<ID>`: copies in the cargo, 4 in all.
    pub cargo: Option<FieldValue>,
}

/// A weapon or drone of the game for the picker.
pub struct ItemChoice {
    pub id: String,
    pub drone: bool,
    pub name: String,
    pub power: u32,
}

/// An augment the ship carries.
pub struct EquipmentAugment {
    pub id: String,
    pub name: Option<String>,
    /// `augments.<ID>`: how many of it, 0 to 3 (3 in all).
    pub count: FieldValue,
}

pub struct Equipment {
    /// `reactor`, 0 to 25; below the bars in use it takes power from the
    /// systems, the last of the file first.
    pub reactor: FieldValue,
    /// Main systems first, then the subsystems.
    pub systems: Vec<EquipmentSystem>,
    /// `false` without a ship layout: weapons, drones and augments unknown.
    pub body_decoded: bool,
    pub weapons: Vec<EquipmentItem>,
    pub drones: Vec<EquipmentItem>,
    pub augments: Vec<EquipmentAugment>,
    /// From the ship blueprint; `None` in reduced mode.
    pub weapon_slots: Option<u32>,
    pub drone_slots: Option<u32>,
    pub augment_slots: u32,
    /// The cargo hold; `None` when it is not decoded.
    pub cargo: Option<Vec<EquipmentItem>>,
    pub cargo_slots: u32,
    /// The edit paths of every weapon and drone id of slots and cargo.
    pub item_fields: Vec<EquipmentItemFields>,
}

/// An augment of the game for the picker.
pub struct AugmentChoice {
    pub id: String,
    pub name: String,
}

fn item(i: equipment::ItemEntry) -> EquipmentItem {
    EquipmentItem {
        id: i.id,
        drone: i.kind == equipment::ItemKind::Drone,
        name: i.name,
        power: i.power,
        armed: i.armed,
    }
}

fn item_fields_dto(f: equipment::ItemFields) -> EquipmentItemFields {
    EquipmentItemFields {
        id: f.id,
        drone: f.kind == equipment::ItemKind::Drone,
        slots: f.slots.map(field_value),
        armed: f.armed.map(field_value),
        cargo: f.cargo.map(field_value),
    }
}

/// The equipment of the player ship (`nearby` false) or the nearby ship;
/// names in `language` (`en`, `de`). Fails (`edit`) without a nearby ship.
#[frb(sync)]
pub fn equipment_of(
    session: &SaveSession,
    nearby: bool,
    language: String,
) -> Result<Equipment, EditorError> {
    equipment_in(session, session.file.game(), nearby, &language)
}

/// The equipment as `edits` would leave it, with the ranges of that state:
/// what the Equipment page shows while edits are pending. Fails like
/// [`SaveSession::apply`] when an edit is invalid.
#[frb(sync)]
pub fn equipment_preview(
    session: &SaveSession,
    nearby: bool,
    edits: Vec<FieldEdit>,
    language: String,
) -> Result<Equipment, EditorError> {
    let game = session.file.preview(&patch_of(edits, vec![])?)?;
    equipment_in(session, &game, nearby, &language)
}

/// The edit paths of the weapon or drone `id` on the player ship (`nearby`
/// false) or the nearby ship, also for one the ship lacks; `None` for an id
/// that is no weapon or drone FTL equips.
#[frb(sync)]
pub fn item_fields(session: &SaveSession, nearby: bool, id: String) -> Option<EquipmentItemFields> {
    equipment::item_fields(
        session.file.game(),
        ship(nearby),
        session.file.layouts(),
        &id,
    )
    .map(item_fields_dto)
}

/// Every weapon and drone of the open `ftl.dat` FTL puts in a slot, by name;
/// empty in reduced mode.
#[frb(sync)]
pub fn item_choices(language: String) -> Vec<ItemChoice> {
    let data = assets::shared();
    equipment::item_choices(data.as_ref().map(|a| a.data().as_ref()), &language)
        .into_iter()
        .map(|c| ItemChoice {
            id: c.id,
            drone: c.kind == equipment::ItemKind::Drone,
            name: c.name,
            power: c.power,
        })
        .collect()
}

fn equipment_in(
    session: &SaveSession,
    game: &ftl_app::SavedGame,
    nearby: bool,
    language: &str,
) -> Result<Equipment, EditorError> {
    let data = assets::shared();
    let l = equipment::loadout(
        game,
        ship(nearby),
        data.as_ref().map(|a| a.data().as_ref()),
        session.file.layouts(),
        language,
    )
    .ok_or_else(|| EditorError {
        code: "edit".to_owned(),
        path: None,
        message: "no ship is at the current beacon, or the situation is not decoded".to_owned(),
    })?;
    Ok(Equipment {
        reactor: field_value(l.reactor),
        systems: l
            .systems
            .into_iter()
            .map(|s| EquipmentSystem {
                id: s.kind.id().to_owned(),
                name: s.name,
                level: s.level.map(field_value),
                installed: field_value(s.installed),
                block: s.block.map(|b| b.code().to_owned()),
                power: s.power,
                damaged: s.damaged,
                ionized: s.ionized,
                damage: s.damage.map(field_value),
                ion: s.ion.map(field_value),
                energy: s.energy.map(field_value),
                subsystem: s.kind.is_subsystem(),
            })
            .collect(),
        body_decoded: l.body_decoded,
        weapons: l.weapons.into_iter().map(item).collect(),
        drones: l.drones.into_iter().map(item).collect(),
        augments: l
            .augments
            .into_iter()
            .map(|a| EquipmentAugment {
                id: a.id,
                name: a.name,
                count: field_value(a.count),
            })
            .collect(),
        weapon_slots: l.weapon_slots,
        drone_slots: l.drone_slots,
        augment_slots: l.augment_slots,
        cargo: l.cargo.map(|c| c.into_iter().map(item).collect()),
        cargo_slots: l.cargo_slots,
        item_fields: l.item_fields.into_iter().map(item_fields_dto).collect(),
    })
}

/// `augments.<id>` of the player ship (`nearby` false) or the nearby ship
/// with its count and range, also for an augment the ship does not carry
/// yet (count 0).
#[frb(sync)]
pub fn augment_field(
    session: &SaveSession,
    nearby: bool,
    id: String,
) -> Result<FieldValue, EditorError> {
    Ok(field_value(equipment::augment_count(
        session.file.game(),
        ship(nearby),
        &id,
    )?))
}

/// Every augment of the open `ftl.dat` by name; empty in reduced mode.
#[frb(sync)]
pub fn augment_choices(language: String) -> Vec<AugmentChoice> {
    let data = assets::shared();
    equipment::augment_choices(data.as_ref().map(|a| a.data().as_ref()), &language)
        .into_iter()
        .map(|c| AugmentChoice {
            id: c.id,
            name: c.name,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::editor::{FieldEdit, open_fixture, open_save};

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
    );

    #[test]
    fn the_equipment_of_the_fixture_reaches_the_app_and_its_edits_apply() {
        let mut session = open_fixture(FIXTURE);
        let e = equipment_of(&session, false, "en".into()).unwrap();
        assert_eq!(
            (
                e.reactor.path.as_str(),
                e.reactor.value,
                e.reactor.min,
                e.reactor.max
            ),
            ("reactor", 8, 0, 25)
        );
        let shields = &e.systems[0];
        let level = shields.level.as_ref().unwrap();
        assert_eq!(
            (
                shields.id.as_str(),
                level.path.as_str(),
                level.value,
                shields.power,
                shields.subsystem
            ),
            ("shields", "systems.shields.level", 2, 2, false)
        );
        assert!(e.systems.last().unwrap().subsystem);
        let unknown = augment_field(&session, false, "SCRAP_COLLECTOR".into()).unwrap();
        assert_eq!(
            (unknown.path.as_str(), unknown.value, unknown.max),
            ("augments.SCRAP_COLLECTOR", 0, 3)
        );
        assert_eq!(
            augment_field(&session, false, "bad id".into())
                .err()
                .unwrap()
                .code,
            "edit"
        );

        let edits = [
            ("reactor", 10),
            ("systems.shields.level", 7),
            ("augments.SCRAP_COLLECTOR", 1),
        ];
        session
            .apply(
                edits
                    .map(|(path, value)| FieldEdit {
                        path: path.into(),
                        value,
                    })
                    .into(),
                vec![],
            )
            .unwrap();
        let after = equipment_of(&session, false, "en".into()).unwrap();
        assert_eq!(after.reactor.value, 10);
        assert_eq!(after.systems[0].level.as_ref().unwrap().value, 7);
        if after.body_decoded {
            let ids: Vec<_> = after.augments.iter().map(|a| a.id.as_str()).collect();
            assert_eq!(ids, ["SCRAP_COLLECTOR"]);
        }
    }

    /// With the ftl.dat of the machine: removing mind control frees the
    /// place the teleporter needs; the preview shows both, and the clone
    /// bay replaces the medbay.
    #[test]
    fn the_preview_installs_and_removes_systems() {
        let mut session = open_save(FIXTURE.to_owned()).unwrap();
        let e = equipment_of(&session, false, "en".into()).unwrap();
        if !e.body_decoded || e.weapon_slots.is_none() {
            println!("skip: no ftl.dat with the fixture ship");
            return;
        }
        // The fixture ship holds 5 of 8 systems: fill it with drones,
        // cloaking and mind control.
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
        let e = equipment_of(&session, false, "en".into()).unwrap();
        let sys = |e: &Equipment, id: &str| {
            let s = e.systems.iter().find(|s| s.id == id).unwrap();
            (
                (s.installed.value, s.installed.min, s.installed.max),
                s.block.clone(),
                s.level.as_ref().map(|f| f.value),
            )
        };
        assert_eq!(
            sys(&e, "teleporter"),
            ((0, 0, 0), Some("system_limit".into()), None)
        );
        let edits = [
            ("systems.mind.installed", 0),
            ("systems.teleporter.installed", 1),
            ("systems.clonebay.installed", 1),
        ]
        .map(|(path, value)| FieldEdit {
            path: path.into(),
            value,
        });
        let p = equipment_preview(&session, false, edits.into(), "en".into()).unwrap();
        assert_eq!(sys(&p, "teleporter"), ((1, 0, 1), None, Some(1)));
        // The ship is full again: mind control waits for a free place.
        assert_eq!(
            sys(&p, "mind"),
            ((0, 0, 0), Some("system_limit".into()), None)
        );
        assert_eq!(sys(&p, "clonebay").2, Some(1));
        assert_eq!(sys(&p, "medbay").2, None);
        // The editor stages edits in click order: remove mind, install the
        // teleporter, remove cloaking, install mind back (the file value,
        // dropped), remove mind again. The install now comes before the
        // removals that free its place, and the preview still holds.
        let clicks = [
            ("systems.teleporter.installed", 1),
            ("systems.cloaking.installed", 0),
            ("systems.mind.installed", 0),
        ]
        .map(|(path, value)| FieldEdit {
            path: path.into(),
            value,
        });
        let p = equipment_preview(&session, false, clicks.into(), "en".into()).unwrap();
        assert_eq!(sys(&p, "teleporter"), ((1, 0, 1), None, Some(1)));
        // Seven systems: the freed place takes hacking or cloaking.
        assert_eq!(sys(&p, "hacking"), ((0, 0, 1), None, None));
        assert_eq!(sys(&p, "cloaking"), ((0, 0, 1), None, None));
        let bad = [FieldEdit {
            path: "systems.shields.installed".into(),
            value: 0,
        }];
        let err = equipment_preview(&session, false, bad.into(), "en".into())
            .err()
            .unwrap();
        assert_eq!(err.code, "edit");
    }

    /// With the ftl.dat of the machine: the preview moves the burst laser
    /// into the cargo and an ion blast into the free slot, and the session
    /// keeps its state until apply.
    #[test]
    fn the_preview_shows_weapons_and_cargo_as_the_edits_leave_them() {
        let mut session = open_save(FIXTURE.to_owned()).unwrap();
        let e = equipment_of(&session, false, "en".into()).unwrap();
        if !e.body_decoded || e.weapon_slots.is_none() {
            println!("skip: no ftl.dat with the fixture ship");
            return;
        }
        let ids = |e: &Equipment| e.weapons.iter().map(|w| w.id.clone()).collect::<Vec<_>>();
        assert_eq!(ids(&e), ["MISSILES_2_PLAYER", "LASER_BURST_3"]);
        // The fixture has 2 of 4 weapon slots and no cargo: fill both.
        let full = [
            ("weapons.SHOTGUN_2", 1),
            ("weapons.BEAM_HULL", 1),
            ("cargo.REPAIR", 1),
        ]
        .map(|(path, value)| FieldEdit {
            path: path.into(),
            value,
        });
        session.apply(full.into(), vec![]).unwrap();
        let e = equipment_of(&session, false, "en".into()).unwrap();
        let full_ids = [
            "MISSILES_2_PLAYER",
            "LASER_BURST_3",
            "SHOTGUN_2",
            "BEAM_HULL",
        ];
        assert_eq!(ids(&e), full_ids);
        let edits = [
            ("weapons.LASER_BURST_3", 0),
            ("cargo.LASER_BURST_3", 1),
            ("weapons.ION_4", 1),
        ]
        .map(|(path, value)| FieldEdit {
            path: path.into(),
            value,
        });
        let p = equipment_preview(&session, false, edits.into(), "en".into()).unwrap();
        // The ion blast takes the slot the laser freed.
        assert_eq!(
            ids(&p),
            ["MISSILES_2_PLAYER", "ION_4", "SHOTGUN_2", "BEAM_HULL"]
        );
        let cargo: Vec<_> = p
            .cargo
            .unwrap()
            .into_iter()
            .map(|c| (c.id, c.drone))
            .collect();
        assert_eq!(
            cargo,
            [
                ("REPAIR".to_owned(), true),
                ("LASER_BURST_3".to_owned(), false)
            ]
        );
        let laser = item_fields(&session, false, "LASER_BURST_3".into()).unwrap();
        assert_eq!(laser.slots.unwrap().path, "weapons.LASER_BURST_3");
        assert!(item_fields(&session, false, "NO_SUCH_GUN".into()).is_none());
        assert!(
            item_choices("en".into())
                .iter()
                .any(|c| c.id == "ION_4" && !c.drone)
        );
        // The session itself still holds its state.
        assert_eq!(
            ids(&equipment_of(&session, false, "en".into()).unwrap()),
            full_ids
        );
        let bad = [FieldEdit {
            path: "weapons.LASER_BURST_3".into(),
            value: 2,
        }];
        assert_eq!(
            equipment_preview(&session, false, bad.into(), "en".into())
                .err()
                .unwrap()
                .code,
            "edit"
        );
    }

    /// The nearby ship reaches the app with its own `nearby.` paths: crew,
    /// equipment and summary; an edit of it applies to it alone.
    #[test]
    fn the_nearby_ship_of_the_fixture_reaches_the_app_under_nearby_paths() {
        let mut session = open_save(FIXTURE.to_owned()).unwrap();
        let summary = session.summary();
        if summary.nearby_blueprint_id.is_none() {
            println!("skip: no ftl.dat found, the situation stays undecoded");
            return;
        }
        assert_eq!(
            summary.nearby_blueprint_id.as_deref(),
            Some("JELLY_BUTTON_P_DLC")
        );
        let hull = &summary.nearby_fields[0];
        assert_eq!((hull.path.as_str(), hull.value), ("nearby.ship.hull", 8));
        let e = equipment_of(&session, true, "en".into()).unwrap();
        assert_eq!(
            (e.reactor.path.as_str(), e.reactor.value),
            ("nearby.reactor", 7)
        );
        assert!(e.cargo.is_none());
        assert_eq!(e.weapon_slots, Some(4));
        let crew = crate::api::crew::crew_of(&session, true);
        assert_eq!(crew.len(), 2);
        assert_eq!(crew[0].health.path, "nearby.crew.0.health");
        session
            .apply(
                vec![FieldEdit {
                    path: "nearby.reactor".into(),
                    value: 5,
                }],
                vec![],
            )
            .unwrap();
        let after = equipment_of(&session, true, "en".into()).unwrap();
        assert_eq!(after.reactor.value, 5);
        let player = equipment_of(&session, false, "en".into()).unwrap();
        assert_eq!(player.reactor.path, "reactor");
    }
}
