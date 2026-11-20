//! The equipment of a ship (the player ship or the nearby ship) as the
//! Equipment page and `ftled` show it: reactor, systems, weapons, drones and
//! augments, each editable value with the range it may take. The fields of
//! the nearby ship carry `nearby.` ([`Field::on`]); the cargo hold belongs to
//! the player ship only.
//!
//! The values and the rules come from `ftl-core` ([`Field::get`],
//! [`Field::range`], [`ftl_core::reactor_power_in_use`]); the limits of the ship
//! blueprint and the names come from `ftl.dat` when it is open. Every value
//! can be lowered: a level below the power of its system lowers the power,
//! a reactor below the bars in use takes power from the systems
//! ([`ftl_core::fit_power_to_capacity`], [`ftl_core::fit_power_to_reactor`]).

use ftl_assets::GameData;
use ftl_core::installs::{self, InstallBlock};
use ftl_core::items::{MAX_CARGO, equippable};
use ftl_core::{
    DroneKind, EditError, Field, MAX_AUGMENTS, SavedGame, ShipId, ShipLayouts, SystemKind,
    reactor_power_in_use,
};

use crate::crew::Bounded;
use crate::layouts::DEFAULT_ITEM_SLOTS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemEntry {
    pub kind: SystemKind,
    /// The system title from `ftl.dat`; `None` without game data.
    pub name: Option<String>,
    /// `systems.<id>.level` with its range; `None` when the system is not
    /// installed.
    pub level: Option<Bounded>,
    /// `systems.<id>.installed` (0 or 1) with its range: 0 to 1 when the
    /// system can be installed or removed now, 1 to 1 for one that stays,
    /// 0 to 0 for a missing one that cannot be installed ([`Self::block`]
    /// says why).
    pub installed: Bounded,
    /// Why the system cannot be installed (missing) or removed
    /// (installed) now ([`ftl_core::installs`]); `None` when it can.
    pub block: Option<InstallBlock>,
    /// Reactor or battery bars the system draws now.
    pub power: i32,
    /// Bars that are broken.
    pub damaged: i32,
    /// Bars that ion damage locks; -1 is a lock without a count.
    pub ionized: i32,
    /// `systems.<id>.damage`, 0 to the level ([`ftl_core::damage`]);
    /// `None` when the system is not installed.
    pub damage: Option<Bounded>,
    /// `systems.<id>.ion`, 0 to 5; `None` when the system is not installed.
    pub ion: Option<Bounded>,
    /// `systems.shields.energy`, the Zoltan shield points
    /// ([`ftl_core::energy_shield`]); `None` on every other system and on
    /// shields that are not installed.
    pub energy: Option<Bounded>,
}

/// A weapon or a drone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Weapon,
    Drone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemEntry {
    /// The blueprint id as FTL writes it (`ION_4`).
    pub id: String,
    pub kind: ItemKind,
    pub name: Option<String>,
    /// Reactor bars the weapon or drone draws, from its blueprint.
    pub power: Option<u32>,
    /// Powered; always `false` in the cargo.
    pub armed: bool,
}

/// The edit paths of one blueprint id with their values and ranges in this
/// game; `None` where the path does not apply or cannot be edited (no
/// ftl.dat, an undecoded part, no weapons system, a kind FTL does not
/// equip).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFields {
    pub id: String,
    pub kind: ItemKind,
    /// `weapons.<ID>` or `drones.<ID>`: the copies in the slots.
    pub slots: Option<Bounded>,
    /// `weapons.<ID>.armed` or `drones.<ID>.armed`: the armed copies.
    pub armed: Option<Bounded>,
    /// `cargo.<ID>`: the copies in the cargo.
    pub cargo: Option<Bounded>,
}

/// A weapon or drone of the game for the picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemChoice {
    pub id: String,
    pub kind: ItemKind,
    pub name: String,
    pub power: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AugmentEntry {
    pub id: String,
    pub name: Option<String>,
    /// `augments.<ID>`: how many of this augment the ship carries.
    pub count: Bounded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loadout {
    /// `reactor`, 0 to 25.
    pub reactor: Bounded,
    /// The reactor bars the systems draw now ([`reactor_power_in_use`]).
    pub reactor_in_use: i32,
    /// The systems the ship can have: every installed one, and with game
    /// data the ones its blueprint lists. Main systems first, then the
    /// subsystems, each in file order.
    pub systems: Vec<SystemEntry>,
    /// `false` when the ship body is not decoded (no ship layout): weapons,
    /// drones and augments are then unknown, not empty.
    pub body_decoded: bool,
    pub weapons: Vec<ItemEntry>,
    pub drones: Vec<ItemEntry>,
    /// One entry per distinct augment, in the order of the file.
    pub augments: Vec<AugmentEntry>,
    /// From the ship blueprint (FTL's 4 when it names none); `None`
    /// without game data.
    pub weapon_slots: Option<u32>,
    pub drone_slots: Option<u32>,
    /// FTL holds three augments on every ship.
    pub augment_slots: u32,
    /// The weapons and drones in the cargo hold, in file order; `None`
    /// when the sector map after the ship body is not decoded, and for the
    /// nearby ship, which has no cargo hold.
    pub cargo: Option<Vec<ItemEntry>>,
    /// FTL's equipment screen has four cargo boxes.
    pub cargo_slots: u32,
    /// The edit paths of every weapon and drone id in the slots and the
    /// cargo, first seen first.
    pub item_fields: Vec<ItemFields>,
}

/// An augment of the game for the picker: id and name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AugmentChoice {
    pub id: String,
    pub name: String,
}

fn name(data: Option<&GameData>, id: &str, language: &str) -> Option<String> {
    data.and_then(|d| d.display_name_in(id, language))
        .filter(|n| !n.is_empty())
        .map(str::to_owned)
}

fn bounded(field: Field, game: &SavedGame, layouts: &dyn ShipLayouts) -> Option<Bounded> {
    let value = field.get(game).ok()?;
    let (min, max) = field.range_in(game, layouts).ok()?;
    Some(Bounded {
        field,
        value,
        min,
        max,
    })
}

/// The kind of a blueprint id: a weapon, an equippable drone, or neither.
fn kind_of(layouts: &dyn ShipLayouts, id: &str) -> Option<ItemKind> {
    if layouts.weapon(id).is_some() {
        Some(ItemKind::Weapon)
    } else if layouts.drone(id).is_some_and(|d| equippable(d.kind)) {
        Some(ItemKind::Drone)
    } else {
        None
    }
}

/// The edit paths of `id` on the ship `on` (also for an id the ship lacks:
/// its counts are 0). `None` for an id that is no weapon or drone FTL
/// equips. The cargo path is the player's only.
pub fn item_fields(
    game: &SavedGame,
    on: ShipId,
    layouts: &dyn ShipLayouts,
    id: &str,
) -> Option<ItemFields> {
    let kind = kind_of(layouts, id)?;
    let (slots, armed) = match kind {
        ItemKind::Weapon => (Field::Weapon(id.into()), Field::WeaponArmed(id.into())),
        ItemKind::Drone => (Field::Drone(id.into()), Field::DroneArmed(id.into())),
    };
    let cargo = Field::Cargo(id.into());
    Some(ItemFields {
        id: id.to_owned(),
        kind,
        slots: bounded(slots.on(on), game, layouts),
        armed: bounded(armed.on(on), game, layouts),
        cargo: (on == ShipId::Player)
            .then(|| bounded(cargo, game, layouts))
            .flatten(),
    })
}

/// Every weapon and drone of the game FTL puts in a slot, by name; empty
/// without game data. Blueprints without a name and the weapons of drones
/// (no `<cooldown>`) are left out.
pub fn item_choices(data: Option<&GameData>, language: &str) -> Vec<ItemChoice> {
    let Some(data) = data else {
        return Vec::new();
    };
    let weapons = data
        .weapons()
        .filter(|w| w.cooldown_ms.is_some())
        .filter_map(|w| {
            Some(ItemChoice {
                name: name(Some(data), &w.id, language)?,
                id: w.id.clone(),
                kind: ItemKind::Weapon,
                power: w.power,
            })
        });
    let drones = data
        .drones()
        .filter(|d| DroneKind::from_id(&d.kind).is_some_and(equippable))
        .filter_map(|d| {
            Some(ItemChoice {
                name: name(Some(data), &d.id, language)?,
                id: d.id.clone(),
                kind: ItemKind::Drone,
                power: d.power,
            })
        });
    let mut choices: Vec<ItemChoice> = weapons.chain(drones).collect();
    choices.sort_by_key(|c| c.name.to_lowercase());
    choices
}

/// The equipment of the ship `on`; `None` when the game has no such ship
/// (no ship at the beacon, or the situation is not decoded). `language`
/// picks the names (`en`, `de`); a name the language lacks is the English
/// one. `layouts` answers the slots and blueprints the ranges of weapons,
/// drones and cargo need.
pub fn loadout(
    game: &SavedGame,
    on: ShipId,
    data: Option<&GameData>,
    layouts: &dyn ShipLayouts,
    language: &str,
) -> Option<Loadout> {
    let ship = game.ship(on)?;
    let blueprint = data.and_then(|d| d.ship(&ship.blueprint_id));
    let listed = |kind: SystemKind| {
        blueprint.is_some_and(|b| b.systems.iter().any(|s| s.system == kind.id()))
    };
    let mut systems: Vec<SystemEntry> = ship
        .systems
        .iter()
        .filter(|s| s.installed.is_some() || listed(s.kind))
        .map(|s| {
            let level = s.installed.as_ref().map(|_| {
                let mut b = Bounded::of(Field::SystemLevel(s.kind).on(on), game);
                let cap = data.and_then(|d| d.max_system_level(&ship.blueprint_id, s.kind.id()));
                if let Some(cap) = cap {
                    b.max = b.max.min(cap as i32).max(b.value);
                }
                b
            });
            let block = if s.installed.is_some() {
                installs::can_remove(game, on, s.kind, layouts).err()
            } else {
                installs::can_install(game, on, s.kind, layouts).err()
            };
            let installed = Field::SystemInstalled(s.kind).on(on);
            SystemEntry {
                kind: s.kind,
                name: name(data, s.kind.id(), language),
                level,
                installed: bounded(installed.clone(), game, layouts).unwrap_or(Bounded {
                    field: installed,
                    value: 0,
                    min: 0,
                    max: 0,
                }),
                block,
                power: s.installed.as_ref().map_or(0, |i| i.power),
                damaged: s.installed.as_ref().map_or(0, |i| i.damaged_bars),
                ionized: s.installed.as_ref().map_or(0, |i| i.ionized_bars),
                damage: bounded(Field::SystemDamage(s.kind).on(on), game, layouts),
                ion: bounded(Field::SystemIon(s.kind).on(on), game, layouts),
                energy: (s.kind == SystemKind::Shields)
                    .then(|| bounded(Field::ShieldEnergy.on(on), game, layouts))
                    .flatten(),
            }
        })
        .collect();
    systems.sort_by_key(|s| s.kind.is_subsystem());

    let body = ship.body.as_ref();
    let weapon = |id: &str, armed: bool| ItemEntry {
        id: id.to_owned(),
        kind: ItemKind::Weapon,
        name: name(data, id, language),
        power: data.and_then(|d| d.weapon(id)).map(|bp| bp.power),
        armed,
    };
    let drone = |id: &str, armed: bool| ItemEntry {
        id: id.to_owned(),
        kind: ItemKind::Drone,
        name: name(data, id, language),
        power: data.and_then(|g| g.drone(id)).map(|bp| bp.power),
        armed,
    };
    let weapons: Vec<ItemEntry> = body.map_or_else(Vec::new, |b| {
        b.weapons.iter().map(|w| weapon(&w.id, w.armed)).collect()
    });
    let drones: Vec<ItemEntry> = body.map_or_else(Vec::new, |b| {
        b.drones.iter().map(|d| drone(&d.id, d.armed)).collect()
    });
    // A cargo id is a weapon when ftl.dat knows it as one, else a drone.
    let world = game.world.as_ref().filter(|_| on == ShipId::Player);
    let cargo: Option<Vec<ItemEntry>> = world.map(|w| {
        w.cargo
            .iter()
            .map(|id| match data.and_then(|d| d.drone(id)) {
                Some(_) => drone(id, false),
                None => weapon(id, false),
            })
            .collect()
    });
    let mut item_ids: Vec<&str> = Vec::new();
    for e in weapons.iter().chain(&drones).chain(cargo.iter().flatten()) {
        if !item_ids.contains(&e.id.as_str()) {
            item_ids.push(&e.id);
        }
    }
    let item_fields = item_ids
        .iter()
        .filter_map(|id| item_fields(game, on, layouts, id))
        .collect();
    let mut augments: Vec<AugmentEntry> = Vec::new();
    for id in body.map_or(&[][..], |b| &b.augments[..]) {
        if augments.iter().all(|a| &a.id != id) {
            augments.push(AugmentEntry {
                id: id.clone(),
                name: name(data, id, language),
                count: Bounded::of(Field::Augment(id.clone()).on(on), game),
            });
        }
    }
    Some(Loadout {
        reactor: Bounded::of(Field::Reactor.on(on), game),
        reactor_in_use: reactor_power_in_use(ship),
        systems,
        body_decoded: body.is_some(),
        weapons,
        drones,
        augments,
        weapon_slots: blueprint.map(|b| b.weapon_slots.unwrap_or(DEFAULT_ITEM_SLOTS)),
        drone_slots: blueprint.map(|b| b.drone_slots.unwrap_or(DEFAULT_ITEM_SLOTS)),
        augment_slots: MAX_AUGMENTS as u32,
        cargo,
        cargo_slots: MAX_CARGO as u32,
        item_fields,
    })
}

/// `augments.<id>` of the ship `on` with its count and range, also for an
/// augment the ship does not carry (count 0). Fails for an id that is no
/// blueprint id or without a decoded ship body.
pub fn augment_count(game: &SavedGame, on: ShipId, id: &str) -> Result<Bounded, EditError> {
    let field = format!("augments.{id}").parse::<Field>()?.on(on);
    Ok(Bounded {
        value: field.get(game)?,
        min: field.range(game)?.0,
        max: field.range(game)?.1,
        field,
    })
}

/// Every augment of the game, by name; empty without game data. Augments
/// without a name (internal blueprints) are left out.
pub fn augment_choices(data: Option<&GameData>, language: &str) -> Vec<AugmentChoice> {
    let Some(data) = data else {
        return Vec::new();
    };
    let mut choices: Vec<AugmentChoice> = data
        .augments()
        .filter_map(|a| {
            let name = name(Some(data), &a.id, language)?;
            Some(AugmentChoice {
                id: a.id.clone(),
                name,
            })
        })
        .collect();
    choices.sort_by_key(|a| a.name.to_lowercase());
    choices
}

#[cfg(test)]
mod tests {
    use super::*;
    use ftl_core::{KnownLayouts, SaveCodec};
    use ftl_sav::SavCodec;

    const FIXTURE: &[u8] =
        include_bytes!("../../ftl-sav/tests/fixtures/continue-1.6.13-format11.sav");

    #[test]
    fn without_game_data_the_loadout_has_reactor_and_installed_systems() {
        let game = SavCodec.decode(FIXTURE, &KnownLayouts::new()).unwrap();
        let l = loadout(&game, ShipId::Player, None, &KnownLayouts::new(), "en").unwrap();
        assert_eq!(
            (
                l.reactor.field.path(),
                l.reactor.value,
                l.reactor.min,
                l.reactor.max
            ),
            ("reactor".to_owned(), 8, 0, 25)
        );
        assert_eq!(l.reactor_in_use, 8);
        let systems: Vec<_> = l
            .systems
            .iter()
            .map(|s| {
                let b = s.level.as_ref().unwrap();
                (s.kind.id(), b.value, b.min, b.max)
            })
            .collect();
        assert_eq!(
            systems,
            [
                ("shields", 2, 1, 8),
                ("engines", 2, 1, 8),
                ("oxygen", 1, 1, 3),
                ("weapons", 3, 1, 8),
                ("medbay", 1, 1, 3),
                ("pilot", 1, 1, 3),
                ("sensors", 1, 1, 3),
                ("doors", 1, 1, 3),
            ]
        );
        assert!(!l.body_decoded);
        assert!(l.weapons.is_empty() && l.augments.is_empty());
        assert_eq!((l.weapon_slots, l.augment_slots), (None, 3));
        assert!(augment_choices(None, "en").is_empty());

        // The fixture has no battery, whose cap of 2 differs from the other
        // subsystems: install one by copying the doors' state into its slot.
        let mut game = game;
        let doors = game
            .player_ship
            .systems
            .iter()
            .find(|s| s.kind == SystemKind::Doors)
            .and_then(|s| s.installed.clone())
            .unwrap();
        game.player_ship
            .systems
            .iter_mut()
            .find(|s| s.kind == SystemKind::Battery)
            .unwrap()
            .installed = Some(doors);
        let l = loadout(&game, ShipId::Player, None, &KnownLayouts::new(), "en").unwrap();
        let battery = l
            .systems
            .iter()
            .find(|s| s.kind == SystemKind::Battery)
            .and_then(|s| s.level.as_ref())
            .map(|b| (b.value, b.min, b.max));
        assert_eq!(battery, Some((1, 1, 2)));
    }

    #[test]
    fn a_system_reports_its_damage_and_ion_fields() {
        let mut game = SavCodec.decode(FIXTURE, &KnownLayouts::new()).unwrap();
        ftl_core::Patch::new(vec![
            "systems.shields.damage=2".parse().unwrap(),
            "systems.shields.ion=1".parse().unwrap(),
        ])
        .apply(&mut game)
        .unwrap();
        let l = loadout(&game, ShipId::Player, None, &KnownLayouts::new(), "en").unwrap();
        let system = |kind| l.systems.iter().find(|s| s.kind == kind).unwrap();
        let field = |b: &Option<Bounded>| {
            let b = b.as_ref().unwrap();
            (b.field.path(), b.value, b.min, b.max)
        };
        let s = system(SystemKind::Shields);
        assert_eq!((s.damaged, s.ionized), (2, 1));
        assert_eq!(field(&s.damage), ("systems.shields.damage".into(), 2, 0, 2));
        assert_eq!(field(&s.ion), ("systems.shields.ion".into(), 1, 0, 5));
        let engines = system(SystemKind::Engines);
        assert_eq!((engines.damaged, engines.ionized), (0, 0));
        assert_eq!(field(&engines.damage).1, 0);
        // Without game data only installed systems are listed; each has both fields.
        assert!(
            l.systems
                .iter()
                .all(|s| s.damage.is_some() && s.ion.is_some())
        );
    }

    #[test]
    fn only_the_shields_row_carries_the_zoltan_shield() {
        let mut game = SavCodec.decode(FIXTURE, &KnownLayouts::new()).unwrap();
        ftl_core::Patch::new(vec!["systems.shields.energy=2".parse().unwrap()])
            .apply(&mut game)
            .unwrap();
        let l = loadout(&game, ShipId::Player, None, &KnownLayouts::new(), "en").unwrap();
        let energy: Vec<_> = l
            .systems
            .iter()
            .filter_map(|s| {
                s.energy
                    .as_ref()
                    .map(|b| (s.kind, b.field.path(), b.value, b.max))
            })
            .collect();
        assert_eq!(
            energy,
            [(SystemKind::Shields, "systems.shields.energy".into(), 2, 5)]
        );
    }
}
