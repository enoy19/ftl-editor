//! Installing and removing systems of a ship (`systems.<id>.installed` and
//! `nearby.systems.<id>.installed`, 0 or 1).
//!
//! Evidence (FTL 1.6.13, macOS binary, symbols read with Ghidra 12.1.4 on
//! 2026-11-19; field offsets from Hyperspace `FTLGameStripped.h`; the ship
//! blueprints of `ftl.dat`):
//!
//! - Rooms. `BlueprintManager::ProcessShipBlueprint` puts every child of
//!   `<systemList>` into `ShipBlueprint::systemInfo` (system id to template:
//!   power, room, slot, ...) and appends the id to `ShipBlueprint::systems`
//!   only when `start` parses true (no attribute is true). Every player ship
//!   of 1.6.13 lists the systems it can buy with `start="false"`; the
//!   Kestrel A lists no clonebay, battery, mind or hacking, so it can never
//!   have them.
//! - Load. `ShipManager::ImportShip` calls `OnInit(blueprint, 0)`, whose
//!   `CreateSystems` resets `systemKey` (16 ids, -1 each) and calls
//!   `AddSystem` for each id of `systems`: the start systems. Then it reads
//!   one capacity per system id in file order; for a capacity above 0 whose
//!   `systemKey` is -1 it calls `AddSystem(id)`, then upgrades the system up
//!   to the capacity (never down), powers it bar by bar and reads its state.
//!   A capacity of 0 reads nothing and leaves the system as `OnInit` built
//!   it.
//! - `ShipManager::AddSystem(id)` returns 0 at once when `systemInfo` has no
//!   entry for the id. `ImportShip` then takes the system of `systemKey[id]`
//!   (-1, so a null pointer) and calls `DecreasePower` on it: a save that
//!   marks a system installed whose blueprint has no room for it crashes
//!   FTL at load. The editor refuses it ([`InstallBlock::NoRoom`]).
//! - The level of an added system is `max(1, template power) + shipLevel`
//!   (0 for the player ship: `ImportShip` passes 0 to `OnInit`), capped at
//!   the system blueprint's `maxPower`. The load only upgrades, so the
//!   editor installs at exactly that level ([`install_level`]).
//! - Removing. A start system that the file marks as not installed comes
//!   back at load (`OnInit` builds it, a capacity of 0 does not remove it),
//!   and the extended info the file then lacks shifts every later read:
//!   `ImportShip` reads `CloneSystem::Load`, the battery fields and
//!   `CloakingSystem::Load` whenever the system exists, and
//!   `ImportBattleState` the hacking and mind state. Only a system the
//!   blueprint lists with `start="false"` can be removed
//!   ([`InstallBlock::StartingSystem`]). `ImportShip` adds weapons and
//!   drones only while their system exists (`ShipManager::AddWeapon`,
//!   `AddDrone`), so a weapons or drone system with items in its slots
//!   stays ([`InstallBlock::ItemsInSlots`]).
//! - Medbay and Clone Bay share one room in every player blueprint (the
//!   same `room` on both entries). `AddSystem(5)` with a clonebay present
//!   upgrades the new medbay to the clonebay's level, takes the clonebay's
//!   power, puts the medbay into its place in the system list, sets
//!   `systemKey[13]` to -1, deletes the clonebay and removes its equipment;
//!   `AddSystem(13)` does the same the other way round. The store sells the
//!   other one with `confirm_buy_clonebay` / `confirm_buy_medbay`
//!   (`SystemStoreBox::Activate`). Installing one here removes the other
//!   and carries its level the same way. Removing one whose counterpart is
//!   a start system would give the ship that counterpart back at load;
//!   the editor refuses it ([`InstallBlock::SwapBack`]).
//! - Limits. `SystemStoreBox::CanHold` asks `ShipManager::CanFitSubsystem`
//!   for a subsystem (`ShipSystem::IsSubsystem`: pilot, sensors, doors,
//!   battery) and `CanFitSystem` for any other: fewer than 4 subsystems,
//!   fewer than 8 other systems; `CanFitSystem(medbay)` is true while a
//!   clonebay exists and the other way round. The 8th system asks
//!   `confirm_buy_last_system`. `ImportShip` checks no limit; whether FTL's
//!   panels show a 9th system is not known, so the editor keeps the store's
//!   limits ([`InstallBlock::SystemLimit`], [`InstallBlock::SubsystemLimit`]).
//!   The reactor limits nothing: a new system starts without power.
//! - Stations. For a system crew can man (`ShipSystem::bBoostable`:
//!   shields, engines, weapons, pilot, sensors, doors) `AddSystem` sets the
//!   room's primary slot from the template `<slot>`, else for the player
//!   ship shields 0 left, engines 2 down, weapons 1 up, pilot 0 right,
//!   sensors and doors 0 up. The file stores the station per room and
//!   `ImportShip` sets it back (`Ship::SetPrimarySlot`), so the editor
//!   writes it with the system; a room without a manned system stores -1,
//!   none (the fixture agrees for every room).
//! - Extended info of a fresh system, as its constructor leaves it and its
//!   save function writes it (`FileHelper::writeFloat` stores
//!   `(int)(f * 1000)`):
//!   - `CloneSystem::Save`: -1.0, 12.0, -1.0, so build ticks -1000, goal
//!     12000, doom ticks -1000.
//!   - the battery fields in `ShipManager::ExportShip`: off, 0 battery
//!     bars, timer progress 0.
//!   - `CloakingSystem::Save`: not on, timer not running, goal 0.5 s, 0 s.
//!   - `MindSystem::Save`: 14.0 and 14.0 (the idle mind system of the
//!     first fixture, replaced on 2026-11-19, stored 14000, 14000).
//!   - `HackingSystem::SaveState`: no target (-1), drone not deployed, not
//!     launched, disruption 0 of 4 s, not disrupting, and the hacking drone
//!     (`DRONE_HACKING`) as `HackingDrone::HackingDrone` builds it: its
//!     pod is [`crate::items::new_pod`] of a hacking drone. The constructor
//!     moves the drone to the room centre (`SetCurrentLocation`, also its
//!     unsaved start point); the editor keeps the unset position of
//!     `SpaceDrone`, which `HackingDrone::LoadState` reads back as it is.
//!   - a weapons system gets one fresh weapon module per weapon
//!     ([`crate::items::new_weapon_module`]).
//! - Artillery appears in player blueprints only as a start system (the
//!   Federation cruisers), so it is never installed here
//!   ([`InstallBlock::Artillery`]).
//! - The nearby ship. `WorldManager::LoadGame` builds it as
//!   `ShipManager(1)` and loads it with the same `ImportShip` (also
//!   `OnInit(blueprint, 0)`), so every rule above holds for it. Its one
//!   difference is the station: `AddSystem` on a ship whose `iShipId` is not
//!   0 and whose template has no `<slot>` gives the pilot square 0 facing up
//!   and every other manned system `Ship::ChoosePrimarySlot`, a random wall
//!   square of the room. No enemy blueprint of 1.6.13 names a `<slot>`, and
//!   `ImportShip` sets the stored station back (`Room::SetPrimarySlot`
//!   ignores -1, `SetPrimaryDirection` takes any value), so a manned system
//!   other than the pilot has no station the editor could write as FTL
//!   would ([`InstallBlock::NoStation`]). The store limits stay for both
//!   ships, though only the player ship meets a store.

use crate::edit::{EditError, Field};
use crate::items::{new_pod, new_weapon_module};
use crate::layout::{ShipLayouts, SystemRoom};
use crate::model::{
    BatteryInfo, CloakingInfo, ClonebayInfo, DroneKind, HackingInfo, MindInfo, SavedGame, Ship,
    ShipId, Station, StationDirection, SystemKind, SystemState,
};

/// FTL's store sells a system while the ship has fewer than this many
/// systems that are not subsystems (`ShipManager::CanFitSystem`).
pub const MAX_SYSTEMS: usize = 8;

/// ... and a subsystem while it has fewer than this many
/// (`ShipManager::CanFitSubsystem`).
pub const MAX_SUBSYSTEMS: usize = 4;

/// The drone blueprint of the hacking system's drone
/// (`HackingDrone::HackingDrone`).
pub const HACKING_DRONE: &str = "DRONE_HACKING";

/// The first format that stores the extended system info.
const EXTENDED_FORMAT: i32 = 7;

/// Why a system cannot be installed or removed; the module doc names the
/// FTL function behind each. `code()` is stable for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallBlock {
    /// The ship blueprint (ftl.dat) is not known.
    NoGameData,
    /// The blueprint has no room for the system.
    NoRoom,
    /// The file format has no slot for the system (format 2 stores 12).
    NotInFormat,
    /// Artillery is a start system only.
    Artillery,
    /// The ship has [`MAX_SYSTEMS`] systems.
    SystemLimit,
    /// The ship has [`MAX_SUBSYSTEMS`] subsystems.
    SubsystemLimit,
    /// The blueprint starts with the system: FTL builds it again at load.
    StartingSystem,
    /// Medbay or Clone Bay of a ship that starts with one of them: only a
    /// swap keeps one installed.
    SwapBack,
    /// Weapons or drones are in the slots of the system.
    ItemsInSlots,
    /// A part of the file the change writes is not decoded.
    NotDecoded,
    /// A manned system on the nearby ship, whose station FTL picks at
    /// random.
    NoStation,
}

impl InstallBlock {
    pub fn code(self) -> &'static str {
        match self {
            InstallBlock::NoGameData => "no_game_data",
            InstallBlock::NoRoom => "no_room",
            InstallBlock::NotInFormat => "not_in_format",
            InstallBlock::Artillery => "artillery",
            InstallBlock::SystemLimit => "system_limit",
            InstallBlock::SubsystemLimit => "subsystem_limit",
            InstallBlock::StartingSystem => "starting_system",
            InstallBlock::SwapBack => "swap_back",
            InstallBlock::ItemsInSlots => "items_in_slots",
            InstallBlock::NotDecoded => "not_decoded",
            InstallBlock::NoStation => "no_station",
        }
    }

    /// The English reason, for the CLI and error details.
    pub fn reason(self) -> &'static str {
        match self {
            InstallBlock::NoGameData => "the ship blueprint needs ftl.dat",
            InstallBlock::NoRoom => {
                "the ship blueprint has no room for it; FTL crashes when it loads such a save"
            }
            InstallBlock::NotInFormat => "this file format has no place for it",
            InstallBlock::Artillery => "FTL gives artillery only to the ships that start with it",
            InstallBlock::SystemLimit => "the ship has 8 systems, the most FTL's store sells",
            InstallBlock::SubsystemLimit => "the ship has 4 subsystems, the most FTL's store sells",
            InstallBlock::StartingSystem => {
                "the ship starts with it; FTL builds it again when it loads the save"
            }
            InstallBlock::SwapBack => {
                "the ship starts with a Medbay or Clone Bay, which FTL builds again at load; install the other one to swap them"
            }
            InstallBlock::ItemsInSlots => {
                "its slots hold weapons or drones, which FTL drops without the system; remove them first"
            }
            InstallBlock::NotDecoded => "a part of the save it changes is not decoded (ftl.dat)",
            InstallBlock::NoStation => {
                "on an enemy ship FTL puts the station of this system on a random square"
            }
        }
    }
}

fn slot_index(ship: &Ship, kind: SystemKind) -> Option<usize> {
    ship.systems.iter().position(|s| s.kind == kind)
}

fn installed(ship: &Ship, kind: SystemKind) -> Option<&SystemState> {
    ship.systems
        .iter()
        .find(|s| s.kind == kind)
        .and_then(|s| s.installed.as_ref())
}

/// Whether the ship has the system.
pub fn is_installed(ship: &Ship, kind: SystemKind) -> bool {
    installed(ship, kind).is_some()
}

/// The other one of Medbay and Clone Bay.
pub fn counterpart(kind: SystemKind) -> Option<SystemKind> {
    match kind {
        SystemKind::Medbay => Some(SystemKind::Clonebay),
        SystemKind::Clonebay => Some(SystemKind::Medbay),
        _ => None,
    }
}

/// Whether crew mans the system (`ShipSystem::bBoostable`) and FTL gives
/// its room a station.
pub fn manned(kind: SystemKind) -> bool {
    matches!(
        kind,
        SystemKind::Shields
            | SystemKind::Engines
            | SystemKind::Weapons
            | SystemKind::Pilot
            | SystemKind::Sensors
            | SystemKind::Doors
    )
}

/// The station `AddSystem` gives the room of a manned system without a
/// `<slot>` in the blueprint; `None` where FTL picks one at random (the
/// nearby ship, see the module doc).
fn default_station(kind: SystemKind, on: ShipId) -> Option<Station> {
    let (square, direction) = match (on, kind) {
        (ShipId::Nearby, SystemKind::Pilot) => (0, StationDirection::Up),
        (ShipId::Nearby, _) => return None,
        (ShipId::Player, SystemKind::Shields) => (0, StationDirection::Left),
        (ShipId::Player, SystemKind::Engines) => (2, StationDirection::Down),
        (ShipId::Player, SystemKind::Weapons) => (1, StationDirection::Up),
        (ShipId::Player, SystemKind::Pilot) => (0, StationDirection::Right),
        (ShipId::Player, _) => (0, StationDirection::Up),
    };
    Some(Station { square, direction })
}

const NO_STATION: Station = Station {
    square: -1,
    direction: StationDirection::None,
};

/// The blueprint entry of the system, or why there is none.
fn entry(
    ship: &Ship,
    kind: SystemKind,
    layouts: &dyn ShipLayouts,
) -> Result<SystemRoom, InstallBlock> {
    let rooms = layouts
        .system_rooms(&ship.blueprint_id)
        .ok_or(InstallBlock::NoGameData)?;
    rooms
        .into_iter()
        .find(|r| r.kind == kind)
        .ok_or(InstallBlock::NoRoom)
}

/// The level FTL gives the system when it adds it (see the module doc):
/// the template power, at least 1, at most the system's highest level; at
/// least the level of the Medbay or Clone Bay it replaces.
pub fn install_level(ship: &Ship, room: &SystemRoom) -> i32 {
    let mut level = room.power.max(1);
    if let Some(other) = counterpart(room.kind).and_then(|c| installed(ship, c)) {
        level = level.max(other.capacity);
    }
    level.min(room.kind.max_level())
}

/// The ship `on`, or [`InstallBlock::NotDecoded`] for a nearby ship the
/// codec did not decode.
fn ship_on(game: &SavedGame, on: ShipId) -> Result<&Ship, InstallBlock> {
    game.ship(on).ok_or(InstallBlock::NotDecoded)
}

/// Checks that the system can be installed now on the ship `on`.
pub fn can_install(
    game: &SavedGame,
    on: ShipId,
    kind: SystemKind,
    layouts: &dyn ShipLayouts,
) -> Result<(), InstallBlock> {
    let ship = ship_on(game, on)?;
    slot_index(ship, kind).ok_or(InstallBlock::NotInFormat)?;
    if kind == SystemKind::Artillery {
        return Err(InstallBlock::Artillery);
    }
    let room = entry(ship, kind, layouts)?;
    if manned(kind) && room.slot.is_none() && default_station(kind, on).is_none() {
        return Err(InstallBlock::NoStation);
    }
    let swaps = counterpart(kind).is_some_and(|c| installed(ship, c).is_some());
    if !swaps {
        let count = ship
            .systems
            .iter()
            .filter(|s| s.installed.is_some() && s.kind.is_subsystem() == kind.is_subsystem())
            .count();
        if kind.is_subsystem() && count >= MAX_SUBSYSTEMS {
            return Err(InstallBlock::SubsystemLimit);
        }
        if !kind.is_subsystem() && count >= MAX_SYSTEMS {
            return Err(InstallBlock::SystemLimit);
        }
    }
    if manned(kind) && ship.body.is_none() {
        return Err(InstallBlock::NotDecoded);
    }
    let needs_extended = matches!(
        kind,
        SystemKind::Hacking | SystemKind::Mind | SystemKind::Weapons
    );
    if needs_extended && game.file_format >= EXTENDED_FORMAT && ship.extended.is_none() {
        return Err(InstallBlock::NotDecoded);
    }
    if kind == SystemKind::Hacking && layouts.drone(HACKING_DRONE).is_none() {
        return Err(InstallBlock::NoGameData);
    }
    if kind == SystemKind::Weapons {
        let weapons = ship.body.as_ref().map_or(&[][..], |b| &b.weapons[..]);
        if weapons.iter().any(|w| layouts.weapon(&w.id).is_none()) {
            return Err(InstallBlock::NoGameData);
        }
    }
    Ok(())
}

/// Checks that the system can be removed now from the ship `on`.
pub fn can_remove(
    game: &SavedGame,
    on: ShipId,
    kind: SystemKind,
    layouts: &dyn ShipLayouts,
) -> Result<(), InstallBlock> {
    let ship = ship_on(game, on)?;
    let rooms = layouts
        .system_rooms(&ship.blueprint_id)
        .ok_or(InstallBlock::NoGameData)?;
    let room = rooms
        .iter()
        .find(|r| r.kind == kind)
        .ok_or(InstallBlock::NoRoom)?;
    let other = counterpart(kind).and_then(|c| rooms.iter().find(|r| r.kind == c));
    if room.start && other.is_some() || other.is_some_and(|o| o.start) {
        return Err(InstallBlock::SwapBack);
    }
    if room.start {
        return Err(InstallBlock::StartingSystem);
    }
    let body = ship.body.as_ref();
    let holds = match kind {
        SystemKind::Weapons => body.is_none_or(|b| !b.weapons.is_empty()),
        SystemKind::Drones => body.is_none_or(|b| !b.drones.is_empty()),
        _ => false,
    };
    if holds {
        return Err(InstallBlock::ItemsInSlots);
    }
    if manned(kind) && body.is_none() {
        return Err(InstallBlock::NotDecoded);
    }
    let needs_extended = matches!(
        kind,
        SystemKind::Hacking | SystemKind::Mind | SystemKind::Weapons
    );
    if needs_extended && game.file_format >= EXTENDED_FORMAT && ship.extended.is_none() {
        return Err(InstallBlock::NotDecoded);
    }
    Ok(())
}

/// The range of `systems.<id>.installed`: 0 to 1 when the system can be
/// installed or removed, 1 to 1 for an installed one that stays; an error
/// for a missing one that cannot be installed.
pub(crate) fn range(
    game: &SavedGame,
    field: &Field,
    kind: SystemKind,
    layouts: &dyn ShipLayouts,
) -> Result<(i32, i32), EditError> {
    let on = field.ship();
    if is_installed(crate::edit::ship(game, field)?, kind) {
        Ok((i32::from(can_remove(game, on, kind, layouts).is_err()), 1))
    } else {
        can_install(game, on, kind, layouts).map_err(|block| blocked(field, kind, block))?;
        Ok((0, 1))
    }
}

fn blocked(field: &Field, kind: SystemKind, block: InstallBlock) -> EditError {
    EditError::SystemBlocked {
        field: field.clone(),
        system: kind,
        block,
    }
}

/// Installs (`value` 1) or removes (0) the system; returns the old value.
pub(crate) fn set(
    game: &mut SavedGame,
    field: &Field,
    kind: SystemKind,
    value: i32,
    layouts: &dyn ShipLayouts,
) -> Result<i32, EditError> {
    let on = field.ship();
    let old = i32::from(is_installed(crate::edit::ship(game, field)?, kind));
    if !(0..=1).contains(&value) {
        return Err(EditError::OutOfRange {
            field: field.clone(),
            value,
            min: 0,
            max: 1,
        });
    }
    if value == old {
        return Ok(old);
    }
    if value == 1 {
        can_install(game, on, kind, layouts).map_err(|b| blocked(field, kind, b))?;
        install(game, on, kind, layouts);
    } else {
        can_remove(game, on, kind, layouts).map_err(|b| blocked(field, kind, b))?;
        remove(game, on, kind, layouts);
    }
    Ok(old)
}

fn fresh_system(level: i32) -> SystemState {
    SystemState {
        capacity: level,
        power: 0,
        damaged_bars: 0,
        ionized_bars: 0,
        deionization_ticks: 0,
        repair_progress: 0,
        damage_progress: 0,
        battery_power: 0,
        hack_level: 0,
        hacked: false,
        temporary_capacity_cap: 1000,
        temporary_capacity_loss: 0,
        temporary_capacity_divisor: 1,
    }
}

/// Call only after [`can_install`] returned `Ok`.
fn install(game: &mut SavedGame, on: ShipId, kind: SystemKind, layouts: &dyn ShipLayouts) {
    let ship = game.ship(on).expect("can_install checked the ship");
    let room = entry(ship, kind, layouts).expect("can_install checked the room");
    let level = install_level(ship, &room);
    if let Some(other) = counterpart(kind).filter(|&c| is_installed(ship, c)) {
        remove(game, on, other, layouts);
    }
    let extended_format = game.file_format >= EXTENDED_FORMAT;
    let ship = game.ship_mut(on).expect("can_install checked the ship");
    let index = slot_index(ship, kind).expect("can_install checked the slot");
    ship.systems[index].installed = Some(fresh_system(level));
    if manned(kind)
        && let Some(r) = ship.body.as_mut().and_then(|b| b.rooms.get_mut(room.room))
    {
        let station = room
            .slot
            .or_else(|| default_station(kind, on))
            .expect("can_install checked the station");
        r.station = r.station.map(|_| station);
    }
    if !extended_format {
        return;
    }
    match kind {
        SystemKind::Clonebay => {
            ship.clonebay = Some(ClonebayInfo {
                build_ticks: -1000,
                build_ticks_goal: 12000,
                doom_ticks: -1000,
            });
        }
        SystemKind::Battery => {
            ship.battery = Some(BatteryInfo {
                active: false,
                used_battery: 0,
                discharge_ticks: 0,
            });
        }
        SystemKind::Cloaking => {
            ship.cloaking = Some(CloakingInfo {
                turned_on: 0,
                timer_running: 0,
                cloak_ticks_goal: 500,
                cloak_ticks: 0,
            });
        }
        SystemKind::Mind => {
            if let Some(e) = ship.extended.as_mut() {
                e.mind = Some(MindInfo {
                    mind_control_ticks: 14000,
                    mind_control_ticks_goal: 14000,
                });
            }
        }
        SystemKind::Hacking => {
            let cooldown = layouts.drone(HACKING_DRONE).and_then(|d| d.cooldown_ms);
            if let Some(e) = ship.extended.as_mut() {
                e.hacking = Some(HackingInfo {
                    target_system: None,
                    drone_powered: 0,
                    drone_pod_visible: false,
                    hacking: 0,
                    disruption_ticks: 0,
                    disruption_ticks_goal: 4000,
                    disrupting: false,
                    drone_pod: new_pod(DroneKind::Hacking, cooldown, on)
                        .expect("a hacking drone has a pod"),
                });
            }
        }
        SystemKind::Weapons => {
            let modules = ship.body.as_ref().map_or_else(Vec::new, |b| {
                b.weapons
                    .iter()
                    .map(|w| {
                        new_weapon_module(
                            &layouts
                                .weapon(&w.id)
                                .expect("can_install checked the weapons"),
                        )
                    })
                    .collect()
            });
            if let Some(e) = ship.extended.as_mut() {
                e.weapon_modules = Some(modules);
            }
        }
        _ => {}
    }
}

/// Removes the system with its extended info and station.
fn remove(game: &mut SavedGame, on: ShipId, kind: SystemKind, layouts: &dyn ShipLayouts) {
    let room = entry(
        game.ship(on).expect("can_remove checked the ship"),
        kind,
        layouts,
    );
    let ship = game.ship_mut(on).expect("can_remove checked the ship");
    if let Some(i) = slot_index(ship, kind) {
        ship.systems[i].installed = None;
    }
    match kind {
        SystemKind::Clonebay => ship.clonebay = None,
        SystemKind::Battery => {
            ship.battery = None;
            for s in ship.systems.iter_mut().filter_map(|s| s.installed.as_mut()) {
                s.power -= s.battery_power;
                s.battery_power = 0;
            }
        }
        SystemKind::Cloaking => ship.cloaking = None,
        SystemKind::Mind => {
            if let Some(e) = ship.extended.as_mut() {
                e.mind = None;
            }
        }
        SystemKind::Hacking => {
            if let Some(e) = ship.extended.as_mut() {
                e.hacking = None;
            }
        }
        SystemKind::Weapons => {
            if let Some(e) = ship.extended.as_mut() {
                e.weapon_modules = None;
            }
        }
        _ => {}
    }
    // The station of a manned system's room goes with it.
    if manned(kind)
        && let Ok(room) = room
        && let Some(r) = ship.body.as_mut().and_then(|b| b.rooms.get_mut(room.room))
    {
        r.station = r.station.map(|_| NO_STATION);
    }
}
