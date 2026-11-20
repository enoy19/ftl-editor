//! Edits of the sector map: the rebel fleet (`fleet.offset`), a beacon's
//! visit (`beacons.<n>.visited`), the beacon of the ship
//! (`beacons.current`), a store at a beacon (`beacons.<n>.store`), the
//! stock of a store (`beacons.<n>.store.<shelf>.<slot>`,
//! `beacons.<n>.store.<supply>`) and the item in a store slot
//! (`beacons.<n>.store.<shelf>.<slot>.item`, a text field).
//!
//! Evidence (FTL 1.6.13, macOS binary, read with Ghidra 12.1.4 on
//! 2026-11-19); the map itself is in [`crate::sector_map`]:
//!
//! - `StarMap::LoadGame` takes the stored fleet offset as it is and turns
//!   every beacon with the fleet flag into a fleet location. `SaveGame`
//!   writes the flag from the circle ([`fleet_covers`]) outside the Last
//!   Stand. A new offset therefore sets the flag of every beacon as FTL
//!   would write it for that offset: a beacon the circle leaves gets back
//!   the event `GenerateMap` gave it. The Last Stand keeps the flag of the
//!   location (its fleet comes from the flagship), so the editor refuses
//!   the offset there.
//! - A beacon's visit count: `LoadGame` reads the background after a count
//!   above 0 and clears the event of a visited beacon, except the current
//!   beacon after its first visit. A count of 0 keeps the event of
//!   `GenerateMap`, so an unvisited beacon offers its event again. The
//!   editor only takes a visit away (the file holds no background for a
//!   new one) and never from the current beacon, where the ship and its
//!   encounter are.
//! - `Store::LoadStore` reads per slot an int and, unless it is negative, the
//!   blueprint id and (format 8 and later) an int it drops; it builds the
//!   box from the blueprint and stores the first int as the box's count,
//!   which `SaveStore` writes back. A box with count 0 is sold out. The
//!   three ints after the shelves are the counts of the fuel, missile and
//!   drone part boxes. Prices are not stored: the box takes the cost of its
//!   blueprint. An item's count is 0 or 1 (the editor keeps a larger count
//!   of the file); a supply count up to [`MAX_STORE_SUPPLY`].
//!
//! What the save holds and what an edit of the beacons must write
//! (Ghidra on the same binary, 2026-11-19):
//!
//! - Only the current sector has beacons in the file. The sectors before
//!   and after are the sector tree: `StarMap::LoadGame` seeds the RNG with
//!   the stored tree seed and calls `StarMap::GenerateSectorMap`, which
//!   builds the columns with `AddSectorColumn` and gives each sector a
//!   description through `EventGenerator::GetSectorDescription` (both draw
//!   from the same RNG and from `sector_data.xml`). The file adds one
//!   visited flag per sector of that tree (`sector_visitation`), the sector
//!   number and the hidden-Crystal flag. [`crate::sector_tree`] builds
//!   the tree and the jump to another sector (`sector.jump`): a new layout
//!   seed and the beacon list `GenerateMap` gives it; the events of the
//!   new beacons come back from the seed when FTL loads the map.
//! - Moving the ship inside the sector (`beacons.current`):
//!   `StarMap::LoadGame` takes the stored current beacon id as the
//!   location of the ship (clamped to the last beacon). It clears the
//!   event of every visited beacon except the current one after its first
//!   visit, so a beacon the ship reaches with a visit count of 1 would play
//!   its `GenerateMap` event again: `WorldManager::LoadGame` uses the
//!   location's event when the stored encounter names no event
//!   (`last_event_id` empty) and opens it with `CreateChoiceBox`, while
//!   `CreateLocation` builds no ship during a load. The edit therefore
//!   gives the new beacon a visit count of at least 2, as a return to a
//!   beacon does, and FTL clears its event. A beacon with a quest is
//!   refused: `LoadGame` builds the quest event again at the current beacon
//!   (`GetBaseEvent`). A beacon of the rebel fleet is refused:
//!   `LocationEvent::ClearEvent` keeps a fleet event. The old beacon keeps
//!   its record (a ship event stays there, as when FTL jumps away). The
//!   situation is the one of an arrival without a ship: no nearby ship,
//!   no projectiles, no hazard (`SpaceManager::LoadSpace` reads the stored
//!   environment), no event in the encounter, no waiting. A beacon never
//!   visited takes the background of the beacon the ship leaves (`LoadGame`
//!   reads a background for every visit count above 0). A crew member
//!   aboard the other ship refuses the move: the jump would leave them
//!   there.
//! - A store at a beacon (`beacons.<n>.store`): `LoadGame` gives the
//!   beacon's event a `Store` and reads its stock (`Store::LoadStore`)
//!   when the beacon's store flag is 1, whatever event `GenerateMap` gave
//!   it. `StarMap::LocationHasStore` labels every event with a store
//!   STORE on the map, and `WorldManager::CreateLocation` hands that store
//!   to the store button (`CommandGui::SetStore`) when the ship arrives.
//!   Only an event with the `store` flag and no stored store rolls a stock
//!   at arrival (`CommandGui::CreateNewStore`, `Store::OnInit`: 2 to 4
//!   shelves of different kinds, from the live RNG), so a stored stock is
//!   never generated from the seed. A new store gets [`new_store`]. 0
//!   removes the stored store.
//! - The other kinds are not stored: a beacon's event (distress, nebula,
//!   a fight) comes from `GenerateMap` and `GenerateEvents` with the event
//!   lists of `ftl.dat`; only the ship event of an enemy, the fleet flag
//!   and the quests are written. A quest is `GetBaseEvent(<event id>)` of
//!   an event the editor cannot name without the event lists; the
//!   distress flag (`under_attack`) changes the map icon only.

use std::fmt;

use crate::edit::{EditError, Field};
use crate::layout::ShipLayouts;
use crate::model::{
    Beacon, BeaconBackground, Environment, FleetPresence, SavedGame, Store, StoreItem,
    StoreItemKind, StoreShelf, SystemKind, race,
};
use crate::sector_map::{self, FLEET_MAX, FLEET_START, LAST_STAND_SECTOR, MapPoint, fleet_covers};

/// The largest count of a store's fuel, missiles or drone parts the editor
/// writes. An editor cap: `LoadStore` takes any int.
pub const MAX_STORE_SUPPLY: i32 = 99;

/// The supplies a store sells by the piece, in file order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoreSupply {
    Fuel,
    Missiles,
    DroneParts,
}

impl StoreSupply {
    pub const ALL: [StoreSupply; 3] = [
        StoreSupply::Fuel,
        StoreSupply::Missiles,
        StoreSupply::DroneParts,
    ];

    pub fn id(self) -> &'static str {
        match self {
            StoreSupply::Fuel => "fuel",
            StoreSupply::Missiles => "missiles",
            StoreSupply::DroneParts => "drone_parts",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.id() == id)
    }

    pub fn of(self, store: &Store) -> i32 {
        match self {
            StoreSupply::Fuel => store.fuel,
            StoreSupply::Missiles => store.missiles,
            StoreSupply::DroneParts => store.drone_parts,
        }
    }

    fn slot(self, store: &mut Store) -> &mut i32 {
        match self {
            StoreSupply::Fuel => &mut store.fuel,
            StoreSupply::Missiles => &mut store.missiles,
            StoreSupply::DroneParts => &mut store.drone_parts,
        }
    }
}

impl fmt::Display for StoreSupply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

pub(crate) fn not_decoded(field: &Field) -> EditError {
    EditError::NotDecoded {
        field: field.clone(),
        part: "the sector map",
    }
}

/// The beacon positions for a fleet edit, or why the edit is refused.
fn positions(game: &SavedGame, field: &Field) -> Result<Vec<MapPoint>, EditError> {
    let world = game.world.as_ref().ok_or_else(|| not_decoded(field))?;
    if world.sector_number >= LAST_STAND_SECTOR {
        return Err(EditError::LastStand {
            field: field.clone(),
        });
    }
    sector_map::layout_of(game)
        .map(|l| l.beacons)
        .map_err(|reason| EditError::NoMapLayout {
            field: field.clone(),
            reason,
        })
}

pub(crate) fn fleet_offset(game: &SavedGame, field: &Field) -> Result<i32, EditError> {
    let world = game.world.as_ref().ok_or_else(|| not_decoded(field))?;
    Ok(world.rebel_fleet.offset)
}

/// From a new sector's offset to where the fleet stops, widened to the
/// file's own value.
pub(crate) fn fleet_range(game: &SavedGame, field: &Field) -> Result<(i32, i32), EditError> {
    positions(game, field)?;
    let now = fleet_offset(game, field)?;
    Ok((now.min(FLEET_START), now.max(FLEET_MAX)))
}

/// Sets the offset and the fleet flag of every beacon as `SaveGame` would
/// write them. Call after the range check.
pub(crate) fn set_fleet_offset(
    game: &mut SavedGame,
    field: &Field,
    value: i32,
) -> Result<i32, EditError> {
    let points = positions(game, field)?;
    let world = game.world.as_mut().ok_or_else(|| not_decoded(field))?;
    let fudge = world.rebel_fleet.fudge;
    for (beacon, &point) in world.beacons.iter_mut().zip(&points) {
        if matches!(
            beacon.fleet_presence,
            FleetPresence::None | FleetPresence::Rebel
        ) {
            beacon.fleet_presence = if fleet_covers(point, value, fudge) {
                FleetPresence::Rebel
            } else {
                FleetPresence::None
            };
        }
    }
    Ok(std::mem::replace(&mut world.rebel_fleet.offset, value))
}

fn beacon<'g>(game: &'g SavedGame, field: &Field, index: usize) -> Result<&'g Beacon, EditError> {
    let world = game.world.as_ref().ok_or_else(|| not_decoded(field))?;
    world.beacons.get(index).ok_or(EditError::NoSuchBeacon {
        index,
        count: world.beacons.len(),
    })
}

fn beacon_mut(game: &mut SavedGame, index: usize) -> &mut Beacon {
    game.world
        .as_mut()
        .and_then(|w| w.beacons.get_mut(index))
        .expect("the range checked this beacon")
}

pub(crate) fn visited(game: &SavedGame, field: &Field, index: usize) -> Result<i32, EditError> {
    Ok(i32::from(beacon(game, field, index)?.visit_count > 0))
}

/// 0 to the file's value: a visit can be taken away, not given.
pub(crate) fn visited_range(
    game: &SavedGame,
    field: &Field,
    index: usize,
) -> Result<(i32, i32), EditError> {
    let now = visited(game, field, index)?;
    let current = game.world.as_ref().map(|w| w.current_beacon_id);
    if now == 1 && current == Some(index as i32) {
        return Err(EditError::CurrentBeacon {
            field: field.clone(),
        });
    }
    Ok((0, now))
}

pub(crate) fn set_visited(game: &mut SavedGame, index: usize, value: i32) -> i32 {
    let b = beacon_mut(game, index);
    let old = i32::from(b.visit_count > 0);
    if value == 0 {
        b.visit_count = 0;
        b.background = None;
    }
    old
}

fn store<'g>(game: &'g SavedGame, field: &Field, index: usize) -> Result<&'g Store, EditError> {
    beacon(game, field, index)?
        .store
        .as_ref()
        .ok_or_else(|| EditError::NoStore {
            field: field.clone(),
        })
}

fn store_mut(game: &mut SavedGame, index: usize) -> &mut Store {
    beacon_mut(game, index)
        .store
        .as_mut()
        .expect("the range checked the store")
}

pub(crate) fn stock(
    game: &SavedGame,
    field: &Field,
    index: usize,
    shelf: usize,
    slot: usize,
) -> Result<i32, EditError> {
    let item = store(game, field, index)?
        .shelves
        .get(shelf)
        .and_then(|s| s.slots.get(slot))
        .and_then(Option::as_ref)
        .ok_or_else(|| EditError::NoStoreItem {
            field: field.clone(),
        })?;
    Ok(item.available)
}

pub(crate) fn stock_range(
    game: &SavedGame,
    field: &Field,
    index: usize,
    shelf: usize,
    slot: usize,
) -> Result<(i32, i32), EditError> {
    Ok((0, stock(game, field, index, shelf, slot)?.max(1)))
}

pub(crate) fn set_stock(
    game: &mut SavedGame,
    index: usize,
    shelf: usize,
    slot: usize,
    value: i32,
) -> i32 {
    let item = store_mut(game, index).shelves[shelf].slots[slot]
        .as_mut()
        .expect("the range checked the item");
    std::mem::replace(&mut item.available, value)
}

pub(crate) fn supply(
    game: &SavedGame,
    field: &Field,
    index: usize,
    supply: StoreSupply,
) -> Result<i32, EditError> {
    Ok(supply.of(store(game, field, index)?))
}

pub(crate) fn supply_range(
    game: &SavedGame,
    field: &Field,
    index: usize,
    kind: StoreSupply,
) -> Result<(i32, i32), EditError> {
    Ok((0, supply(game, field, index, kind)?.max(MAX_STORE_SUPPLY)))
}

pub(crate) fn set_supply(game: &mut SavedGame, index: usize, kind: StoreSupply, value: i32) -> i32 {
    std::mem::replace(kind.slot(store_mut(game, index)), value)
}

/// Why the ship cannot move to another beacon. Each has a stable code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveBlock {
    /// The situation after the sector map is not decoded.
    NotDecoded,
    /// The Last Stand: the flagship and its fleet follow the ship.
    LastStand,
    /// A crew member stands aboard the other ship.
    CrewAboard,
    /// The beacon holds the rebel fleet; FTL keeps a fleet event.
    FleetBeacon,
    /// A quest waits at the beacon; FTL builds its event again.
    QuestBeacon,
}

impl MoveBlock {
    pub fn code(self) -> &'static str {
        match self {
            MoveBlock::NotDecoded => "move_not_decoded",
            MoveBlock::LastStand => "last_stand",
            MoveBlock::CrewAboard => "move_crew_aboard",
            MoveBlock::FleetBeacon => "move_fleet_beacon",
            MoveBlock::QuestBeacon => "move_quest_beacon",
        }
    }
}

impl fmt::Display for MoveBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MoveBlock::NotDecoded => {
                "the situation after the sector map is not decoded; it needs the ship layouts from ftl.dat"
            }
            MoveBlock::LastStand => "the Last Stand keeps the ship on the flagship's path",
            MoveBlock::CrewAboard => {
                "a crew member is aboard the other ship; the jump would leave them there"
            }
            MoveBlock::FleetBeacon => {
                "the rebel fleet holds that beacon; FTL keeps its fleet event, which needs a ship"
            }
            MoveBlock::QuestBeacon => {
                "a quest waits at that beacon; FTL plays its event again without its ship"
            }
        })
    }
}

/// Why the ship cannot leave its beacon at all, or `None`.
pub fn move_block(game: &SavedGame) -> Option<MoveBlock> {
    let world = game.world.as_ref()?;
    let Some(situation) = game.situation.as_ref() else {
        return Some(MoveBlock::NotDecoded);
    };
    if world.sector_number >= LAST_STAND_SECTOR {
        return Some(MoveBlock::LastStand);
    }
    let intruder = game.player_ship.crew.iter().any(|c| !c.player_controlled);
    let boarder = situation
        .nearby_ship
        .as_ref()
        .is_some_and(|n| n.ship.crew.iter().any(|c| c.player_controlled));
    (intruder || boarder).then_some(MoveBlock::CrewAboard)
}

/// Why the ship cannot move to beacon `index`, or `None`.
pub fn move_block_to(game: &SavedGame, index: usize) -> Option<MoveBlock> {
    if let Some(block) = move_block(game) {
        return Some(block);
    }
    let world = game.world.as_ref()?;
    let beacon = world.beacons.get(index)?;
    if matches!(
        beacon.fleet_presence,
        FleetPresence::Rebel | FleetPresence::Both
    ) {
        return Some(MoveBlock::FleetBeacon);
    }
    world
        .quest_events
        .iter()
        .any(|q| q.beacon_id == index as i32)
        .then_some(MoveBlock::QuestBeacon)
}

pub(crate) fn current(game: &SavedGame, field: &Field) -> Result<i32, EditError> {
    let world = game.world.as_ref().ok_or_else(|| not_decoded(field))?;
    Ok(world.current_beacon_id)
}

/// Every beacon of the sector, unless the ship cannot leave its beacon.
pub(crate) fn current_range(game: &SavedGame, field: &Field) -> Result<(i32, i32), EditError> {
    let world = game.world.as_ref().ok_or_else(|| not_decoded(field))?;
    if let Some(reason) = move_block(game) {
        return Err(EditError::ShipStays {
            field: field.clone(),
            reason,
        });
    }
    Ok((0, world.beacons.len() as i32 - 1))
}

/// Moves the ship to beacon `value` as an arrival without a ship (see the
/// module doc). Call after the range check.
pub(crate) fn set_current(
    game: &mut SavedGame,
    field: &Field,
    value: i32,
) -> Result<i32, EditError> {
    let old = current(game, field)?;
    if value == old {
        return Ok(old);
    }
    let index = value as usize;
    if let Some(reason) = move_block_to(game, index) {
        return Err(EditError::ShipStays {
            field: field.clone(),
            reason,
        });
    }
    let world = game.world.as_mut().ok_or_else(|| not_decoded(field))?;
    let left = usize::try_from(old)
        .ok()
        .and_then(|n| world.beacons.get(n))
        .and_then(|b| b.background.clone());
    world.current_beacon_id = value;
    world.waiting = false;
    world.wait_event_seed = -1;
    let to = &mut world.beacons[index];
    to.visit_count = (to.visit_count + 1).max(2);
    to.seen = true;
    if to.background.is_none() {
        to.background = Some(left.unwrap_or_else(no_background));
    }
    arrive_without_ship(game);
    Ok(old)
}

/// The background FTL reads as none (`"NONE"` images).
pub(crate) fn no_background() -> BeaconBackground {
    BeaconBackground {
        starscape: "NONE".to_owned(),
        sprite: "NONE".to_owned(),
        sprite_x: 0,
        sprite_y: 0,
        sprite_rotation: 0,
    }
}

/// The situation of an arrival without a ship: no nearby ship, no
/// projectiles, no event in the encounter, no hazard (see the module doc).
pub(crate) fn arrive_without_ship(game: &mut SavedGame) {
    let situation = game
        .situation
        .as_mut()
        .expect("the caller checked the situation");
    situation.nearby_ship = None;
    situation.projectiles.clear();
    situation.ship_event_present = 0;
    if let Some(e) = situation.encounter.as_mut() {
        e.last_event_id.clear();
        e.text.clear();
        e.choices.clear();
        e.affected_crew_seed = -1;
    }
    if let Some(env) = situation.environment.as_mut() {
        *env = Environment {
            red_giant_present: false,
            pulsar_present: false,
            pds_present: false,
            vulnerable_ships: env.vulnerable_ships,
            asteroid_field: None,
            solar_flare_fade_ticks: 0,
            havoc_ticks: 0,
            pds_ticks: 0,
        };
    }
}

/// The stock a new store gets: a weapon, a drone and an augment shelf,
/// every item for sale, and a few supplies. `Store::OnInit` builds 2 to 4
/// shelves of different kinds; the editor picks three well-known items per
/// shelf, and the slots can be swapped (`.item`).
pub fn new_store() -> Store {
    let shelf = |kind, ids: [&str; 3]| StoreShelf {
        kind,
        slots: ids.map(|id| {
            Some(StoreItem {
                id: id.to_owned(),
                available: 1,
                extra_data: 0,
            })
        }),
    };
    Store {
        shelves: vec![
            shelf(
                StoreItemKind::Weapon,
                ["LASER_BURST_2", "MISSILES_1", "ION_2"],
            ),
            shelf(StoreItemKind::Drone, ["COMBAT_1", "DEFENSE_1", "REPAIR"]),
            shelf(
                StoreItemKind::Augment,
                ["SCRAP_COLLECTOR", "AUTO_COOLDOWN", "SHIELD_RECHARGE"],
            ),
        ],
        fuel: 6,
        missiles: 4,
        drone_parts: 3,
    }
}

pub(crate) fn has_store(game: &SavedGame, field: &Field, index: usize) -> Result<i32, EditError> {
    Ok(i32::from(beacon(game, field, index)?.store.is_some()))
}

pub(crate) fn set_has_store(game: &mut SavedGame, index: usize, value: i32) -> i32 {
    let b = beacon_mut(game, index);
    let old = i32::from(b.store.is_some());
    match (old, value) {
        (0, 1) => b.store = Some(new_store()),
        (1, 0) => b.store = None,
        _ => {}
    }
    old
}

/// The id in a store slot; an empty slot answers an empty text.
pub(crate) fn item_id(
    game: &SavedGame,
    field: &Field,
    index: usize,
    shelf: usize,
    slot: usize,
) -> Result<String, EditError> {
    let shelf = store(game, field, index)?
        .shelves
        .get(shelf)
        .ok_or_else(|| EditError::NoStoreItem {
            field: field.clone(),
        })?;
    Ok(shelf.slots[slot]
        .as_ref()
        .map(|i| i.id.clone())
        .unwrap_or_default())
}

/// Whether `id` is a blueprint of `ftl.dat` FTL can put on a shelf of
/// `kind`: `Store::LoadStore` builds the box from the blueprint of that
/// kind. Without `ftl.dat` only crew races and systems are known.
fn fits_shelf(kind: StoreItemKind, id: &str, data: &dyn ShipLayouts) -> bool {
    match kind {
        StoreItemKind::Weapon => data.weapon(id).is_some(),
        StoreItemKind::Drone => data.drone(id).is_some(),
        StoreItemKind::Augment => data.is_augment(id),
        StoreItemKind::Crew => race(id).is_some(),
        StoreItemKind::System => SystemKind::from_id(id).is_some(),
    }
}

/// Puts `id` into a store slot (for sale when the slot was empty); returns
/// the old id, empty for an empty slot.
pub(crate) fn set_item(
    game: &mut SavedGame,
    field: &Field,
    index: usize,
    (shelf, slot): (usize, usize),
    id: &str,
    data: &dyn ShipLayouts,
) -> Result<String, EditError> {
    let old = item_id(game, field, index, shelf, slot)?;
    let kind = store(game, field, index)?.shelves[shelf].kind;
    if !fits_shelf(kind, id, data) {
        return Err(EditError::NotForShelf {
            field: field.clone(),
            id: id.to_owned(),
            kind,
        });
    }
    let item = &mut store_mut(game, index).shelves[shelf].slots[slot];
    match item {
        Some(i) => i.id = id.to_owned(),
        None => {
            *item = Some(StoreItem {
                id: id.to_owned(),
                available: 1,
                extra_data: 0,
            })
        }
    }
    Ok(old)
}
