//! The sector map of the open save for the Map page. A thin adapter over
//! `ftl_app::map`; edits go through [`SaveSession::apply`] with the paths
//! `fleet.offset`, `beacons.<n>.visited`, `beacons.<n>.seen`,
//! `beacons.<n>.store.<shelf>.<slot>`, `beacons.<n>.store.<supply>`,
//! `beacons.current`, `beacons.<n>.store` and `sector.jump`, and the text edit
//! `beacons.<n>.store.<shelf>.<slot>.item`.
//! [`sector_map_preview`] shows the map as the pending edits leave it (a
//! moved fleet changes the fleet flag of every beacon).

use flutter_rust_bridge::frb;
use ftl_app::assets;
use ftl_app::map::{self, FleetPresence, StoreItemKind};

use super::editor::{EditorError, FieldEdit, FieldValue, SaveSession, TextEdit, patch_of};
use super::field::field_value;

/// A store item: a weapon, drone, augment, crew race or system.
pub struct StoreItemView {
    /// Blueprint id; the image id of a weapon (`WeaponFrame`), drone
    /// (`Drone`) or system (`SystemIcon`).
    pub id: String,
    /// The look id of a crew race (`GameImageKind::CrewLook`): its default
    /// look; `None` for other kinds.
    pub look_id: Option<String>,
    /// `weapon`, `drone`, `augment`, `crew` or `system`.
    pub kind: String,
    pub name: Option<String>,
    /// `beacons.<n>.store.<shelf>.<slot>`: 0 sold out, 1 for sale.
    pub stock: FieldValue,
    /// `beacons.<n>.store.<shelf>.<slot>.item`: a [`TextEdit`] with a
    /// blueprint id of the same kind swaps the item.
    pub item_path: String,
}

pub struct StoreView {
    /// The items in shelf order; an empty slot is left out.
    pub items: Vec<StoreItemView>,
    /// `beacons.<n>.store.fuel`, `.missiles`, `.drone_parts`, in that order.
    pub supplies: Vec<FieldValue>,
}

pub struct BeaconView {
    pub id: u32,
    /// Map pixels; `None` when the layout is unknown.
    pub x: Option<i32>,
    pub y: Option<i32>,
    /// `beacons.<n>.seen`.
    pub seen: FieldValue,
    /// `beacons.<n>.visited`; `None` at the current beacon.
    pub visited: Option<FieldValue>,
    /// `none`, `rebel`, `federation` or `both`.
    pub fleet: String,
    pub under_attack: bool,
    pub enemy: Option<String>,
    pub current: bool,
    pub quest: Option<String>,
    pub store: Option<StoreView>,
    /// `beacons.<n>.store`: 1 adds a store, 0 removes it.
    pub has_store: FieldValue,
    /// Why the ship cannot move here (`move_fleet_beacon`,
    /// `move_quest_beacon`, ...); `None` at the current beacon.
    pub move_block: Option<String>,
}

pub struct SectorMap {
    /// One-based.
    pub sector: i32,
    pub beacons: Vec<BeaconView>,
    /// Joins as flat index pairs: `[a0, b0, a1, b1, ...]`.
    pub joins: Vec<u32>,
    pub fleet_offset: i32,
    pub fleet_fudge: i32,
    pub fleet_radius: i32,
    /// `fleet.offset` with its range; `None` when the fleet stays.
    pub fleet_field: Option<FieldValue>,
    /// Why the fleet stays: `last_stand`, `map_native_random`, ...
    pub fleet_block: Option<String>,
    /// `beacons.current`: the beacon of the ship; `None` when it stays.
    pub ship_field: Option<FieldValue>,
    /// Why the ship stays: `last_stand`, `move_crew_aboard`,
    /// `move_not_decoded`.
    pub ship_block: Option<String>,
    /// Why no beacon has a position: a stable code (`map_native_random`,
    /// `map_count_mismatch`, `map_not_connected`).
    pub layout_block: Option<String>,
    pub width: i32,
    pub height: i32,
}

fn kind_id(kind: StoreItemKind) -> &'static str {
    match kind {
        StoreItemKind::Weapon => "weapon",
        StoreItemKind::Drone => "drone",
        StoreItemKind::Augment => "augment",
        StoreItemKind::Crew => "crew",
        StoreItemKind::System => "system",
    }
}

fn fleet_id(fleet: FleetPresence) -> &'static str {
    match fleet {
        FleetPresence::None => "none",
        FleetPresence::Rebel => "rebel",
        FleetPresence::Federation => "federation",
        FleetPresence::Both => "both",
    }
}

fn view(game: &ftl_app::SavedGame, language: &str) -> Result<SectorMap, EditorError> {
    let data = assets::shared();
    let v = map::sector_view(game, data.as_ref().map(|a| a.data().as_ref()), language).ok_or_else(
        || EditorError {
            code: "edit".to_owned(),
            path: None,
            message: "the sector map is not decoded".to_owned(),
        },
    )?;
    Ok(SectorMap {
        sector: v.sector,
        beacons: v
            .beacons
            .into_iter()
            .map(|b| BeaconView {
                id: b.id as u32,
                x: b.point.map(|p| p.x),
                y: b.point.map(|p| p.y),
                seen: field_value(b.seen),
                visited: b.visited.map(field_value),
                fleet: fleet_id(b.fleet).to_owned(),
                under_attack: b.under_attack,
                enemy: b.enemy,
                current: b.current,
                quest: b.quest,
                store: b.store.map(|s| StoreView {
                    items: s
                        .shelves
                        .into_iter()
                        .flat_map(|shelf| {
                            let kind = kind_id(shelf.kind);
                            shelf
                                .items
                                .into_iter()
                                .flatten()
                                .map(move |i| StoreItemView {
                                    look_id: (kind == "crew")
                                        .then(|| assets::crew_look_id(&i.id, true, &[])),
                                    id: i.id,
                                    kind: kind.to_owned(),
                                    name: i.name,
                                    stock: field_value(i.stock),
                                    item_path: i.item_path,
                                })
                        })
                        .collect(),
                    supplies: s
                        .supplies
                        .into_iter()
                        .map(|(_, f)| field_value(f))
                        .collect(),
                }),
                has_store: field_value(b.has_store),
                move_block: b.move_block.map(str::to_owned),
            })
            .collect(),
        joins: v
            .joins
            .into_iter()
            .flat_map(|(a, b)| [a as u32, b as u32])
            .collect(),
        fleet_offset: v.fleet.offset,
        fleet_fudge: v.fleet.fudge,
        fleet_radius: v.fleet.radius,
        fleet_field: v.fleet.field.map(field_value),
        fleet_block: v.fleet.block.map(str::to_owned),
        ship_field: v.ship.map(field_value),
        ship_block: v.ship_block.map(str::to_owned),
        layout_block: v.layout_block.map(|r| r.code().to_owned()),
        width: v.size.0,
        height: v.size.1,
    })
}

/// The sector map of the open save; names in `language` (`en`, `de`).
/// Fails (`edit`) when the world is not decoded.
#[frb(sync)]
pub fn sector_map_of(session: &SaveSession, language: String) -> Result<SectorMap, EditorError> {
    view(session.file.game(), &language)
}

/// The sector map as `edits` would leave it: what the Map page draws while
/// edits are pending. Fails like [`SaveSession::apply`] when an edit is
/// invalid.
#[frb(sync)]
pub fn sector_map_preview(
    session: &SaveSession,
    edits: Vec<FieldEdit>,
    texts: Vec<TextEdit>,
    language: String,
) -> Result<SectorMap, EditorError> {
    let game = session.file.preview(&patch_of(edits, texts)?)?;
    view(&game, &language)
}

/// One sector of the tree (`ftl_app::map::TreeSector`).
pub struct TreeSectorView {
    /// The value of `sector.jump` that moves the ship here.
    pub index: u32,
    /// `civilian`, `hostile` or `nebula`.
    pub kind: String,
    /// Zero-based.
    pub level: i32,
    /// FTL's point on its sector map.
    pub x: i32,
    pub y: i32,
    pub description: String,
    pub name: String,
    pub short_name: String,
    pub next: Vec<u32>,
    pub visited: bool,
    pub current: bool,
    pub reachable: bool,
    /// Why the ship cannot jump here (`jump_last_stand`, ...); `None` at
    /// the current sector.
    pub jump_block: Option<String>,
}

pub struct SectorTreeView {
    pub sectors: Vec<TreeSectorView>,
    /// `sector.jump` with its range; `None` when the ship stays.
    pub jump_field: Option<FieldValue>,
    /// Why the ship stays in its sector: `jump_last_stand`,
    /// `jump_crew_aboard`, `jump_distant_quests`, `jump_hidden_sector`,
    /// `jump_not_decoded`, or a tree code.
    pub jump_block: Option<String>,
    pub hidden_sector: bool,
}

fn tree_view(
    game: &ftl_app::SavedGame,
    session: &SaveSession,
    language: &str,
) -> Result<SectorTreeView, EditorError> {
    let data = assets::shared();
    let v = map::sector_tree_view(
        game,
        session.file.layouts(),
        data.as_ref().map(|a| a.data().as_ref()),
        language,
    )
    .map_err(|b| EditorError {
        code: b.code.to_owned(),
        path: None,
        message: b.message,
    })?;
    Ok(SectorTreeView {
        sectors: v
            .sectors
            .into_iter()
            .map(|s| TreeSectorView {
                index: s.index as u32,
                kind: s.kind.id().to_owned(),
                level: s.level,
                x: s.x,
                y: s.y,
                description: s.description,
                name: s.name,
                short_name: s.short_name,
                next: s.next.into_iter().map(|n| n as u32).collect(),
                visited: s.visited,
                current: s.current,
                reachable: s.reachable,
                jump_block: s.jump_block.map(str::to_owned),
            })
            .collect(),
        jump_field: v.jump.map(field_value),
        jump_block: v.jump_block.map(str::to_owned),
        hidden_sector: v.hidden_sector,
    })
}

/// The sector tree of the open save, as FTL rebuilds it from the tree
/// seed and `sector_data.xml`; names in `language`. Fails with a tree code
/// (`tree_no_game_data`, `tree_native_random`, ...) when there is none.
#[frb(sync)]
pub fn sector_tree_of(
    session: &SaveSession,
    language: String,
) -> Result<SectorTreeView, EditorError> {
    tree_view(session.file.game(), session, &language)
}

/// The sector tree as `edits` would leave it (a staged `sector.jump`).
#[frb(sync)]
pub fn sector_tree_preview(
    session: &SaveSession,
    edits: Vec<FieldEdit>,
    texts: Vec<TextEdit>,
    language: String,
) -> Result<SectorTreeView, EditorError> {
    let game = session.file.preview(&patch_of(edits, texts)?)?;
    tree_view(&game, session, &language)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::editor::open_fixture;

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
    );

    #[test]
    fn the_fixture_map_reaches_the_app_and_the_preview_moves_the_fleet() {
        let session = open_fixture(FIXTURE);
        let m = sector_map_of(&session, "en".into()).unwrap();
        assert_eq!((m.sector, m.beacons.len()), (1, 21));
        assert_eq!((m.beacons[0].x, m.beacons[0].y), (Some(69), Some(78)));
        // Early in sector 1 the fleet reaches no beacon.
        assert!(m.beacons.iter().all(|b| b.fleet == "none"));
        assert!(m.beacons[8].current && m.beacons[8].visited.is_none());
        assert_eq!(m.fleet_field.as_ref().unwrap().path, "fleet.offset");
        assert_eq!(m.joins.len() % 2, 0);

        // At its maximum the circle reaches beacon 0 (69, 78).
        let edits = vec![FieldEdit {
            path: "fleet.offset".into(),
            value: 60,
        }];
        let p = sector_map_preview(&session, edits, vec![], "en".into()).unwrap();
        assert_eq!(p.fleet_offset, 60);
        assert_eq!(p.beacons[0].fleet, "rebel");

        let edits = vec![FieldEdit {
            path: "fleet.offset".into(),
            value: -959,
        }];
        let p = sector_map_preview(&session, edits, vec![], "en".into()).unwrap();
        assert_eq!(p.fleet_offset, -959);
        assert!(p.beacons.iter().all(|b| b.fleet == "none"));
    }
}

#[cfg(test)]
mod moves {
    use super::*;
    use crate::api::editor::open_save;

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
    );

    /// A moved ship and a new store with a swapped item reach the preview.
    #[test]
    fn the_preview_moves_the_ship_and_builds_a_store() {
        let session = open_save(FIXTURE.to_owned()).unwrap();
        // The store swap needs the item blueprints of a real ftl.dat.
        if !matches!(
            session.file.layout_source(),
            ftl_app::LayoutSource::FtlDat(_)
        ) {
            println!("skip: no ftl.dat found");
            return;
        }
        let m = sector_map_of(&session, "en".into()).unwrap();
        // ftl.dat decodes the situation on this machine only.
        let Some(ship) = m.ship_field else {
            assert_eq!(m.ship_block.as_deref(), Some("move_not_decoded"));
            return;
        };
        assert_eq!((ship.path.as_str(), ship.value), ("beacons.current", 8));
        let edits = vec![
            FieldEdit {
                path: "beacons.current".into(),
                value: 2,
            },
            FieldEdit {
                path: "beacons.3.store".into(),
                value: 1,
            },
        ];
        let texts = vec![TextEdit {
            path: "beacons.3.store.2.0.item".into(),
            text: "NANO_MEDBAY".into(),
        }];
        let p = sector_map_preview(&session, edits, texts, "en".into()).unwrap();
        assert!(p.beacons[2].current && !p.beacons[8].current);
        let store = p.beacons[3].store.as_ref().unwrap();
        assert_eq!(store.items[6].id, "NANO_MEDBAY");
        assert_eq!(store.items[6].item_path, "beacons.3.store.2.0.item");
    }
}

#[cfg(test)]
mod tree {
    use super::*;
    use crate::api::editor::open_fixture;

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
    );

    /// A staged jump reaches both previews: the tree marks the new sector
    /// current, the beacon map is the new sector's.
    #[test]
    fn a_staged_jump_moves_the_ship_marker_and_the_beacon_map() {
        let session = open_fixture(FIXTURE);
        // The tree needs sector_data.xml; without ftl.dat on this machine
        // the bridge names why.
        let t = match sector_tree_of(&session, "en".into()) {
            Ok(t) => t,
            Err(e) => {
                assert_eq!(e.code, "tree_no_game_data");
                return;
            }
        };
        assert_eq!(t.sectors.len(), 19);
        assert!(t.sectors[0].current);
        assert_eq!(t.jump_field.as_ref().unwrap().path, "sector.jump");
        let edits = vec![FieldEdit {
            path: "sector.jump".into(),
            value: 2,
        }];
        let again = vec![FieldEdit {
            path: "sector.jump".into(),
            value: 2,
        }];
        let p = sector_tree_preview(&session, again, vec![], "en".into()).unwrap();
        assert!(p.sectors[2].current && p.sectors[0].visited && !p.sectors[0].current);
        let m = sector_map_preview(&session, edits, vec![], "en".into()).unwrap();
        assert_eq!(m.sector, 2);
        assert_eq!(m.fleet_offset, -959);
        assert_eq!(m.layout_block, None);
    }
}
