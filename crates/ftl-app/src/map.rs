//! The sector map as the Map page and `ftled` show it: every beacon with its
//! position, visit, fleet, enemy and store, the rebel fleet's circle, and
//! each editable value with its range.
//!
//! The positions come from the layout seed ([`ftl_core::sector_map`]); the
//! rules of the edits from [`ftl_core::beacons`]; item names from `ftl.dat`
//! when it is open.

use ftl_assets::GameData;
use ftl_core::beacons::{self, MoveBlock, StoreSupply};
use ftl_core::sector_map::{self, FLEET_RADIUS, MAP_SIZE, MapPoint, NoLayout};
use ftl_core::sector_tree::{self, SectorKind};
use ftl_core::{EditError, Field, SavedGame, ShipLayouts};
pub use ftl_core::{FleetPresence, StoreItemKind};

use crate::crew::Bounded;

/// One beacon of the sector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconEntry {
    /// The beacon id: its index in the file.
    pub id: usize,
    /// Where FTL draws it; `None` when the layout is unknown
    /// ([`SectorView::layout_block`]).
    pub point: Option<MapPoint>,
    /// `beacons.<n>.seen`.
    pub seen: Bounded,
    /// `beacons.<n>.visited`: 0 to 1 for a visited beacon, 0 to 0 for one
    /// never visited; `None` at the current beacon, which keeps its visit.
    pub visited: Option<Bounded>,
    pub fleet: FleetPresence,
    /// The beacon flashes as a distress call.
    pub under_attack: bool,
    /// The ship event waiting here (`PIRATE`, `LONG_FLEET`).
    pub enemy: Option<String>,
    /// The ship is here.
    pub current: bool,
    /// A quest event placed at this beacon.
    pub quest: Option<String>,
    pub store: Option<StoreEntry>,
    /// `beacons.<n>.store`: 0 to 1, 1 adds a store
    /// ([`ftl_core::beacons::new_store`]), 0 removes it.
    pub has_store: Bounded,
    /// Why the ship cannot move here: a stable code of
    /// [`ftl_core::beacons::MoveBlock`]; `None` at the current beacon too,
    /// where nothing moves.
    pub move_block: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreEntry {
    pub shelves: Vec<ShelfEntry>,
    /// `beacons.<n>.store.<fuel|missiles|drone_parts>`, in that order.
    pub supplies: Vec<(StoreSupply, Bounded)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShelfEntry {
    pub kind: StoreItemKind,
    /// Three slots; `None` is an empty slot.
    pub items: Vec<Option<StoreItemEntry>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreItemEntry {
    /// The blueprint id: a weapon, drone, augment, crew race or system.
    pub id: String,
    /// The title from `ftl.dat` in the app language; `None` without it.
    pub name: Option<String>,
    /// `beacons.<n>.store.<shelf>.<slot>`: 0 sold out, 1 for sale.
    pub stock: Bounded,
    /// `beacons.<n>.store.<shelf>.<slot>.item`: the text edit that swaps
    /// the item for another blueprint of the shelf's kind.
    pub item_path: String,
}

/// The rebel fleet's circle: all within [`FLEET_RADIUS`] of (offset, fudge).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetEntry {
    pub offset: i32,
    pub fudge: i32,
    pub radius: i32,
    /// `fleet.offset` with its range; `None` when it cannot be edited
    /// ([`FleetEntry::block`]).
    pub field: Option<Bounded>,
    /// Why the fleet cannot be moved: a stable code (`last_stand`,
    /// `map_native_random`, ...).
    pub block: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorView {
    /// One-based, as FTL names the sector.
    pub sector: i32,
    pub beacons: Vec<BeaconEntry>,
    /// Joins between beacons (index pairs), as FTL builds them.
    pub joins: Vec<(usize, usize)>,
    pub fleet: FleetEntry,
    /// `beacons.current` with its range; `None` when the ship stays
    /// ([`SectorView::ship_block`]).
    pub ship: Option<Bounded>,
    /// Why the ship cannot leave its beacon: a stable code of
    /// [`ftl_core::beacons::MoveBlock`].
    pub ship_block: Option<&'static str>,
    /// Why no beacon has a position; `None` when every beacon has one.
    pub layout_block: Option<NoLayout>,
    /// The area the beacons stand in, in map pixels.
    pub size: (i32, i32),
}

fn bounded(field: Field, game: &SavedGame) -> Result<Bounded, EditError> {
    let value = field.get(game)?;
    let (min, max) = field.range(game)?;
    Ok(Bounded {
        field,
        value,
        min,
        max,
    })
}

fn name(data: Option<&GameData>, id: &str, language: &str) -> Option<String> {
    data.and_then(|d| d.display_name_in(id, language))
        .map(str::to_owned)
}

/// The sector map of `game`, or `None` when the world is not decoded.
pub fn sector_view(
    game: &SavedGame,
    data: Option<&GameData>,
    language: &str,
) -> Option<SectorView> {
    let world = game.world.as_ref()?;
    let (points, joins, layout_block) = match sector_map::layout_of(game) {
        Ok(l) => (l.beacons.into_iter().map(Some).collect(), l.joins, None),
        Err(e) => (vec![None; world.beacons.len()], vec![], Some(e)),
    };
    let beacons = world
        .beacons
        .iter()
        .zip(points)
        .enumerate()
        .map(|(n, (b, point))| BeaconEntry {
            id: n,
            point,
            seen: bounded(Field::BeaconSeen(n), game).expect("the beacon exists"),
            visited: bounded(Field::BeaconVisited(n), game).ok(),
            fleet: b.fleet_presence,
            under_attack: b.under_attack,
            enemy: b.enemy.as_ref().map(|e| e.ship_event_id.clone()),
            current: world.current_beacon_id == n as i32,
            quest: world
                .quest_events
                .iter()
                .find(|q| q.beacon_id == n as i32)
                .map(|q| q.event_id.clone()),
            store: b.store.as_ref().map(|s| StoreEntry {
                shelves: s
                    .shelves
                    .iter()
                    .enumerate()
                    .map(|(shelf, sh)| ShelfEntry {
                        kind: sh.kind,
                        items: sh
                            .slots
                            .iter()
                            .enumerate()
                            .map(|(slot, item)| {
                                item.as_ref().map(|i| StoreItemEntry {
                                    id: i.id.clone(),
                                    name: name(data, &i.id, language),
                                    stock: bounded(
                                        Field::StoreStock {
                                            beacon: n,
                                            shelf,
                                            slot,
                                        },
                                        game,
                                    )
                                    .expect("the slot holds an item"),
                                    item_path: Field::StoreItem {
                                        beacon: n,
                                        shelf,
                                        slot,
                                    }
                                    .path(),
                                })
                            })
                            .collect(),
                    })
                    .collect(),
                supplies: StoreSupply::ALL
                    .into_iter()
                    .map(|supply| {
                        let field = Field::StoreSupply { beacon: n, supply };
                        (supply, bounded(field, game).expect("the store exists"))
                    })
                    .collect(),
            }),
            has_store: bounded(Field::BeaconStore(n), game).expect("the beacon exists"),
            move_block: (world.current_beacon_id != n as i32)
                .then(|| beacons::move_block_to(game, n).map(MoveBlock::code))
                .flatten(),
        })
        .collect();
    let fleet = &world.rebel_fleet;
    let (field, block) = match bounded(Field::FleetOffset, game) {
        Ok(b) => (Some(b), None),
        Err(EditError::LastStand { .. }) => (None, Some("last_stand")),
        Err(EditError::NoMapLayout { reason, .. }) => (None, Some(reason.code())),
        Err(_) => (None, Some("map_not_decoded")),
    };
    let (ship, ship_block) = match beacons::move_block(game) {
        None => (bounded(Field::ShipBeacon, game).ok(), None),
        Some(b) => (None, Some(b.code())),
    };
    Some(SectorView {
        sector: world.sector_number + 1,
        ship,
        ship_block,
        beacons,
        joins,
        fleet: FleetEntry {
            offset: fleet.offset,
            fudge: fleet.fudge,
            radius: FLEET_RADIUS,
            field,
            block,
        },
        layout_block,
        size: MAP_SIZE,
    })
}

/// One sector of the tree as the Map page draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeSector {
    /// The index in FTL's order (the order of the visited flags).
    pub index: usize,
    pub kind: SectorKind,
    /// Zero-based; the Map page shows it one-based.
    pub level: i32,
    /// FTL's point on its sector map.
    pub x: i32,
    pub y: i32,
    /// The `sectorDescription` id (`ZOLTAN_HOME`).
    pub description: String,
    /// The name in the app language (from `ftl.dat`), else the text id.
    pub name: String,
    pub short_name: String,
    /// The sectors of the next column this one leads to.
    pub next: Vec<usize>,
    pub visited: bool,
    pub current: bool,
    /// The joins reach it from the current sector.
    pub reachable: bool,
    /// Why the ship cannot jump here: a stable code of
    /// [`ftl_core::sector_tree::JumpBlock`]; `None` at the current sector.
    pub jump_block: Option<&'static str>,
}

/// The sector tree of a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeView {
    pub sectors: Vec<TreeSector>,
    /// `sector.jump` with its range; `None` when the ship stays
    /// ([`TreeView::jump_block`]).
    pub jump: Option<Bounded>,
    /// Why the ship cannot leave its sector: a stable code.
    pub jump_block: Option<&'static str>,
    /// The ship is in the hidden Crystal sector, outside the tree.
    pub hidden_sector: bool,
}

/// Why a game shows no tree: the stable code and the English message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeBlock {
    pub code: &'static str,
    pub message: String,
}

/// The sector tree of `game` as FTL rebuilds it from the tree seed and the
/// sector lists of `layouts` (`ftl.dat`); names from `data` in `language`.
pub fn sector_tree_view(
    game: &SavedGame,
    layouts: &dyn ShipLayouts,
    data: Option<&GameData>,
    language: &str,
) -> Result<TreeView, TreeBlock> {
    let tree = sector_tree::tree_of(game, layouts.sector_catalog()).map_err(|e| TreeBlock {
        code: e.code(),
        message: e.to_string(),
    })?;
    let text = |id: &str, is_id: bool| -> String {
        if !is_id {
            return id.to_owned();
        }
        data.and_then(|d| d.sector_text_in(id, language))
            .unwrap_or(id)
            .to_owned()
    };
    let sectors = tree
        .tree
        .sectors
        .iter()
        .enumerate()
        .map(|(n, s)| TreeSector {
            index: n,
            kind: s.kind,
            level: s.level,
            x: s.x,
            y: s.y,
            description: s.description.clone(),
            name: text(&s.name.text, s.name.is_id),
            short_name: text(&s.name.short, s.name.is_id),
            next: s.next.clone(),
            visited: tree.visited[n],
            current: tree.current == Some(n),
            reachable: tree.reachable[n],
            jump_block: (tree.current != Some(n))
                .then(|| sector_tree::jump_block_to(game, &tree, n).map(|b| b.code()))
                .flatten(),
        })
        .collect();
    let jump_block = sector_tree::jump_block(game, &tree);
    let jump = match &jump_block {
        None => {
            let field = Field::SectorJump;
            let value = field.get(game).ok();
            let range = field.range_in(game, layouts).ok();
            value.zip(range).map(|(value, (min, max))| Bounded {
                field,
                value,
                min,
                max,
            })
        }
        Some(_) => None,
    };
    Ok(TreeView {
        sectors,
        jump,
        jump_block: jump_block.map(|b| b.code()),
        hidden_sector: tree.current.is_none(),
    })
}
