//! The [`ShipLayouts`] port over the game's own data (`ftl-assets`).
//!
//! The codec needs, per ship blueprint id, the rooms in squares, the door
//! count and the systems placed in several rooms. The blueprint names the
//! layout file and lists its systems; the layout file lists rooms and doors.

use std::collections::BTreeSet;
use std::sync::Arc;

use std::path::PathBuf;

use ftl_core::{
    DroneFacts, DroneKind, ItemSlots, KnownLayouts, RoomShape, ShipLayouts, ShipShape, Station,
    StationDirection, SystemKind, SystemRoom, WeaponFacts,
};

use crate::assets::{self, GameData, ShipBlueprint, ShipLayout};
use ftl_core::sector_tree::{SectorCatalog, SectorDescription, SectorName};

/// Where the ship layouts of an opened file came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutSource {
    /// Loaded from this `ftl.dat`.
    FtlDat(PathBuf),
    /// Passed by the caller ([`crate::SaveFile::open_with_layouts`]).
    Given,
    /// No layouts: the ship bodies stay undecoded. The reason names the
    /// missing or unreadable `ftl.dat`.
    None { reason: String },
}

/// The layouts [`crate::SaveFile::open`] uses: those of the located
/// `ftl.dat` ([`assets::locate`]), else none with the reason.
pub fn default_layouts() -> (Arc<dyn ShipLayouts>, LayoutSource) {
    let none = |reason: String| -> (Arc<dyn ShipLayouts>, LayoutSource) {
        (Arc::new(KnownLayouts::new()), LayoutSource::None { reason })
    };
    let located = match assets::locate() {
        Ok(l) => l,
        Err(e) => return none(e.to_string()),
    };
    // The process-wide cache: the app opens ftl.dat once for images and
    // layouts alike, and a second SaveFile::open reads no archive again.
    match assets::open_shared(&located.path) {
        Ok(shared) => (
            Arc::new(GameLayouts::new(shared.data().clone())),
            LayoutSource::FtlDat(located.path),
        ),
        Err(e) => none(e.to_string()),
    }
}

/// Ship shapes from a loaded `ftl.dat`.
#[derive(Debug, Clone)]
pub struct GameLayouts {
    data: Arc<GameData>,
    /// The sector lists, converted once (`None` without `sector_data.xml`).
    sectors: Option<SectorCatalog>,
}

impl GameLayouts {
    pub fn new(data: Arc<GameData>) -> Self {
        let sectors = sector_catalog(&data);
        GameLayouts { data, sectors }
    }
}

/// The sector lists of `ftl.dat` as the core's [`SectorCatalog`]; `None`
/// when the package has no `sector_data.xml`.
pub fn sector_catalog(data: &GameData) -> Option<SectorCatalog> {
    let s = data.sectors();
    if s.types.is_empty() {
        return None;
    }
    let descriptions = s
        .descriptions
        .iter()
        .map(|(id, d)| {
            let names = d
                .names
                .iter()
                .map(|n| SectorName {
                    text: n.text.clone(),
                    short: n.short.clone(),
                    is_id: n.is_id,
                })
                .collect();
            let d = SectorDescription {
                min_sector: d.min_sector,
                unique: d.unique,
                names,
            };
            (id.clone(), d)
        })
        .collect();
    Some(SectorCatalog {
        types: s.types.clone(),
        descriptions,
    })
}

/// The weapon and drone slots of a ship blueprint without `weaponSlots` or
/// `droneSlots`: `BlueprintManager::ProcessShipBlueprint` (FTL 1.6.13, macOS
/// binary, Ghidra) sets both to 4 before it reads the tags.
pub const DEFAULT_ITEM_SLOTS: u32 = 4;

impl ShipLayouts for GameLayouts {
    fn ship(&self, blueprint_id: &str) -> Option<ShipShape> {
        let blueprint = self.data.ship(blueprint_id)?;
        let layout = self.data.layout(&blueprint.layout)?;
        shape_of(blueprint, layout)
    }

    /// The `type` of the drone blueprint (`DroneBlueprint::kind`).
    fn drone_kind(&self, drone_id: &str) -> Option<DroneKind> {
        DroneKind::from_id(&self.data.drone(drone_id)?.kind)
    }

    /// `weaponSlots` and `droneSlots` of the blueprint, else FTL's
    /// [`DEFAULT_ITEM_SLOTS`]: every player ship of 1.6.13 names both, most
    /// enemy blueprints (`autoBlueprints.xml`) neither.
    fn ship_gfx(&self, blueprint_id: &str) -> Option<String> {
        Some(self.data.ship(blueprint_id)?.img.clone())
    }

    fn slots(&self, blueprint_id: &str) -> Option<ItemSlots> {
        let b = self.data.ship(blueprint_id)?;
        Some(ItemSlots {
            weapons: b.weapon_slots.unwrap_or(DEFAULT_ITEM_SLOTS),
            drones: b.drone_slots.unwrap_or(DEFAULT_ITEM_SLOTS),
        })
    }

    fn weapon(&self, weapon_id: &str) -> Option<WeaponFacts> {
        let w = self.data.weapon(weapon_id)?;
        Some(WeaponFacts {
            power: w.power as i32,
            cooldown_ms: w.cooldown_ms,
            kind: w.kind.clone(),
        })
    }

    fn is_augment(&self, augment_id: &str) -> bool {
        self.data.augment(augment_id).is_some()
    }

    fn drone(&self, drone_id: &str) -> Option<DroneFacts> {
        let d = self.data.drone(drone_id)?;
        Some(DroneFacts {
            power: d.power as i32,
            kind: DroneKind::from_id(&d.kind)?,
            cooldown_ms: d.cooldown,
        })
    }

    /// The `systemList` entries the editor knows by id; a later entry of
    /// the same system (the flagship artillery) does not replace the first.
    fn system_rooms(&self, blueprint_id: &str) -> Option<Vec<SystemRoom>> {
        let blueprint = self.data.ship(blueprint_id)?;
        Some(system_rooms_of(blueprint))
    }

    /// The English human names of `data/names.xml`.
    fn crew_names(&self, male: bool) -> Vec<String> {
        self.data.crew_names(male).to_vec()
    }

    /// `data/sector_data.xml`.
    fn sector_catalog(&self) -> Option<&SectorCatalog> {
        self.sectors.as_ref()
    }
}

/// The `systemList` of a blueprint as [`SystemRoom`]s, first entry per system.
pub fn system_rooms_of(blueprint: &ShipBlueprint) -> Vec<SystemRoom> {
    let mut rooms: Vec<SystemRoom> = Vec::new();
    for s in &blueprint.systems {
        let Some(kind) = SystemKind::from_id(&s.system) else {
            continue;
        };
        if rooms.iter().any(|r| r.kind == kind) {
            continue;
        }
        let slot = s.slot.as_ref().and_then(|(square, direction)| {
            let direction = match direction.as_str() {
                "down" => StationDirection::Down,
                "right" => StationDirection::Right,
                "up" => StationDirection::Up,
                "left" => StationDirection::Left,
                _ => return None,
            };
            Some(Station {
                square: *square,
                direction,
            })
        });
        rooms.push(SystemRoom {
            kind,
            room: s.room as usize,
            power: s.power as i32,
            max: s.max.map(|m| m as i32),
            start: s.start,
            slot,
        });
    }
    rooms
}

/// The shape of a ship as `SavedGameParser.readShip` reads it:
/// - rooms by room id (`ShipLayout.getRoom(id)`); `None` when the ids are not
///   exactly 0 to n-1, because the file then has no defined room order,
/// - doors counted by distinct (x, y, vertical) like Java's door map, which
///   keys doors by that coordinate,
/// - a system with several `systemList` entries in the blueprint has one slot
///   per entry (`getSystemRoom(type).length`).
pub fn shape_of(blueprint: &ShipBlueprint, layout: &ShipLayout) -> Option<ShipShape> {
    let mut rooms = vec![None; layout.rooms.len()];
    for r in &layout.rooms {
        let slot = rooms.get_mut(r.id as usize)?;
        if slot.is_some() {
            return None;
        }
        *slot = Some(RoomShape {
            x: layout.offset_x + r.x,
            y: layout.offset_y + r.y,
            squares_h: r.w,
            squares_v: r.h,
        });
    }
    let rooms = rooms.into_iter().collect::<Option<Vec<_>>>()?;
    let door_count = layout
        .doors
        .iter()
        .map(|d| (d.x, d.y, d.vertical))
        .collect::<BTreeSet<_>>()
        .len();
    let multi_room_systems = SystemKind::ALL
        .into_iter()
        .filter_map(|kind| {
            let n = blueprint
                .systems
                .iter()
                .filter(|s| s.system == kind.id())
                .count();
            (n > 1).then_some((kind, n))
        })
        .collect();
    Some(ShipShape {
        layout_id: blueprint.layout.clone(),
        rooms,
        door_count,
        multi_room_systems,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{ShipLayoutDoor, ShipLayoutRoom, ShipSystem};

    fn room(id: u32, w: u32, h: u32) -> ShipLayoutRoom {
        ShipLayoutRoom {
            id,
            x: id as i32 * 2,
            y: 1,
            w,
            h,
        }
    }

    fn door(x: i32, y: i32, vertical: bool) -> ShipLayoutDoor {
        ShipLayoutDoor {
            x,
            y,
            room_a: 0,
            room_b: -1,
            vertical,
        }
    }

    fn system(name: &str, room: u32) -> ShipSystem {
        ShipSystem {
            slot: None,
            system: name.into(),
            power: 1,
            max: None,
            room,
            start: true,
            img: None,
        }
    }

    fn blueprint(systems: Vec<ShipSystem>) -> ShipBlueprint {
        ShipBlueprint {
            id: "TEST_SHIP".into(),
            layout: "test_layout".into(),
            img: String::new(),
            class_name: String::new(),
            name: String::new(),
            description: String::new(),
            systems,
            weapon_slots: None,
            drone_slots: None,
            weapons: vec![],
            drones: vec![],
            augments: vec![],
            missiles: 0,
            drone_parts: 0,
            hull: 0,
            max_power: 0,
            crew: vec![],
        }
    }

    #[test]
    fn rooms_go_by_id_doors_by_coordinate_systems_by_entry() {
        let layout = ShipLayout {
            offset_x: 3,
            offset_y: 2,
            rooms: vec![room(1, 1, 2), room(0, 2, 2)],
            doors: vec![door(1, 1, true), door(1, 1, true), door(1, 1, false)],
            ..ShipLayout::default()
        };
        let bp = blueprint(vec![
            system("artillery", 0),
            system("artillery", 1),
            system("pilot", 1),
        ]);
        let s = shape_of(&bp, &layout).unwrap();
        assert_eq!(s.layout_id, "test_layout");
        assert_eq!(
            s.rooms,
            [
                RoomShape {
                    x: 3,
                    y: 3,
                    squares_h: 2,
                    squares_v: 2
                },
                RoomShape {
                    x: 5,
                    y: 3,
                    squares_h: 1,
                    squares_v: 2
                }
            ]
        );
        assert_eq!(s.door_count, 2);
        assert_eq!(s.multi_room_systems, [(SystemKind::Artillery, 2)]);
    }

    #[test]
    fn a_gap_or_a_twice_used_room_id_gives_no_shape() {
        let bp = blueprint(vec![]);
        let gap = ShipLayout {
            rooms: vec![room(0, 1, 1), room(2, 1, 1)],
            ..ShipLayout::default()
        };
        assert_eq!(shape_of(&bp, &gap), None);
        let twice = ShipLayout {
            rooms: vec![room(0, 1, 1), room(0, 1, 1)],
            ..ShipLayout::default()
        };
        assert_eq!(shape_of(&bp, &twice), None);
    }
}
