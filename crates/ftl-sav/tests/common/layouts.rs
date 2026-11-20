//! A fake of the layout port for the fixtures. The shapes were read from
//! ftl.dat 1.6.13 through the Java editor (`DataManager.getShipLayout(id)`:
//! room count, `squaresH` x `squaresV` per room, `getDoorMap().size()`;
//! `DataManager.getDrone(id).getType()`); the room positions from
//! `data/<layout>.txt` (`ROOM id x y w h`, plus `X_OFFSET`, `Y_OFFSET`).
//! `crates/ftl-app/tests/real_layouts.rs` checks them against the real
//! `ftl.dat`.

use ftl_core::{
    DroneFacts, DroneKind, ItemSlots, KnownLayouts, RoomShape, ShipShape, Station,
    StationDirection, SystemKind, SystemRoom, WeaponFacts,
};

/// Rooms as `(x, y, w, h)` of the layout file, moved by the layout offset.
fn rooms(offset: (i32, i32), rooms: &[(i32, i32, u32, u32)]) -> Vec<RoomShape> {
    rooms
        .iter()
        .map(|&(x, y, squares_h, squares_v)| RoomShape {
            x: offset.0 + x,
            y: offset.1 + y,
            squares_h,
            squares_v,
        })
        .collect()
}

/// Drones the tests add: neither ship of the fixture carries one, but a
/// drone added to a ship needs its kind for the extended ship info.
pub const FIXTURE_DRONES: [(&str, DroneKind); 2] = [
    ("COMBAT_1", DroneKind::Combat),
    ("ANTI_DRONE", DroneKind::Defense),
];

/// The player ship of the fixture.
pub const FIXTURE_SHIP: &str = "PLAYER_SHIP_HARD";

/// The nearby ship of the fixture.
pub const FIXTURE_NEARBY_SHIP: &str = "JELLY_BUTTON_P_DLC";

/// `PLAYER_SHIP_HARD` (layout `kestral`), the ship of
/// continue-1.6.13-format11.sav, its nearby ship `JELLY_BUTTON_P_DLC`
/// (layout `jelly_button_pirate`) and the drones the tests add.
pub fn fixture_layouts() -> KnownLayouts {
    ship_layouts()
        .with(
            FIXTURE_NEARBY_SHIP,
            ShipShape {
                layout_id: "jelly_button_pirate".into(),
                // jelly_button_pirate.txt has no offset lines.
                rooms: rooms(
                    (0, 0),
                    &[
                        (2, 0, 1, 2),
                        (1, 1, 1, 2),
                        (3, 1, 1, 2),
                        (0, 3, 2, 1),
                        (2, 3, 1, 2),
                        (3, 3, 2, 1),
                    ],
                ),
                door_count: 6,
                multi_room_systems: vec![],
            },
        )
        .with_drone(FIXTURE_DRONES[0].0, FIXTURE_DRONES[0].1)
        .with_drone(FIXTURE_DRONES[1].0, FIXTURE_DRONES[1].1)
}

/// The nearby ship of the second fixture,
/// continue-1.6.13-format11-sector3-enemy-drone.sav.
#[allow(dead_code)]
pub const DRONE_FIXTURE_NEARBY_SHIP: &str = "AUTO_ASSAULT";

/// The drone that ship has deployed: `COMBAT_BEAM`, type `COMBAT`, power 2,
/// cooldown 1000 ms (`data/blueprints.xml`).
#[allow(dead_code)]
pub const DRONE_FIXTURE_DRONE: (&str, DroneKind) = ("COMBAT_BEAM", DroneKind::Combat);

/// The player ship, the nearby `AUTO_ASSAULT` (layout `auto_assault`,
/// no offset lines) and its drone kind: what the second fixture needs to
/// decode to its end.
#[allow(dead_code)]
pub fn drone_fixture_layouts() -> KnownLayouts {
    ship_layouts()
        .with(
            DRONE_FIXTURE_NEARBY_SHIP,
            ShipShape {
                layout_id: "auto_assault".into(),
                rooms: rooms(
                    (0, 0),
                    &[
                        (0, 1, 1, 2),
                        (1, 0, 1, 2),
                        (3, 0, 1, 2),
                        (5, 0, 1, 2),
                        (6, 1, 1, 2),
                    ],
                ),
                door_count: 2,
                multi_room_systems: vec![],
            },
        )
        .with_drone(DRONE_FIXTURE_DRONE.0, DRONE_FIXTURE_DRONE.1)
}

/// Weapons of ftl.dat 1.6.13 as `(id, type, power, cooldown in ms)`: the
/// two of the player ship, the two of the nearby ship and three more
/// (`data/blueprints.xml`).
#[allow(dead_code)]
pub const FIXTURE_WEAPONS: [(&str, &str, i32, i32); 7] = [
    ("MISSILES_2_PLAYER", "MISSILES", 1, 11000),
    ("LASER_BURST_3", "LASER", 2, 12000),
    ("LASER_BURST_1", "LASER", 1, 10000),
    ("BEAM_1", "BEAM", 1, 12000),
    ("ION_4", "LASER", 3, 4000),
    ("SHOTGUN_2", "BURST", 3, 21000),
    ("BEAM_HULL", "BEAM", 2, 14000),
];

/// Drones of ftl.dat 1.6.13 as `(id, kind, power, cooldown in ms)`: the
/// two of [`FIXTURE_DRONES`] and one per other kind FTL equips
/// (`data/blueprints.xml`, `data/dlcBlueprints.xml`).
#[allow(dead_code)]
pub const FIXTURE_ITEM_DRONES: [(&str, DroneKind, i32, Option<i32>); 6] = [
    ("COMBAT_1", DroneKind::Combat, 2, Some(1000)),
    ("ANTI_DRONE", DroneKind::Defense, 1, Some(7000)),
    ("BOARDER", DroneKind::Boarder, 3, None),
    ("REPAIR", DroneKind::Repair, 1, None),
    ("DRONE_SHIELD_PLAYER", DroneKind::Shield, 2, Some(15000)),
    ("SHIP_REPAIR", DroneKind::ShipRepair, 2, Some(1000)),
];

/// [`fixture_layouts`] with the slots of the fixture ship and the weapon
/// and drone blueprints above: what the edits of weapons, drones and cargo
/// need.
#[allow(dead_code)]
pub fn item_layouts() -> KnownLayouts {
    let mut l = fixture_layouts().with_slots(
        FIXTURE_SHIP,
        ItemSlots {
            weapons: 4,
            drones: 2,
        },
    );
    for (id, kind, power, cooldown) in FIXTURE_WEAPONS {
        l = l.with_weapon(
            id,
            WeaponFacts {
                power,
                cooldown_ms: Some(cooldown),
                kind: kind.into(),
            },
        );
    }
    for (id, kind, power, cooldown_ms) in FIXTURE_ITEM_DRONES {
        l = l.with_drone_facts(
            id,
            DroneFacts {
                power,
                kind,
                cooldown_ms,
            },
        );
    }
    l
}

/// The player ship alone: no nearby ship, no drones.
pub fn ship_layouts() -> KnownLayouts {
    // kestral.txt: X_OFFSET 0, Y_OFFSET 2.
    let kestrel = [
        (14, 2, 1, 2),
        (12, 2, 2, 2),
        (10, 2, 2, 1),
        (10, 3, 2, 1),
        (8, 1, 2, 2),
        (8, 3, 2, 2),
        (6, 0, 2, 1),
        (6, 1, 2, 2),
        (6, 3, 2, 2),
        (6, 5, 2, 1),
        (4, 2, 2, 2),
        (3, 1, 2, 1),
        (3, 4, 2, 1),
        (1, 1, 2, 1),
        (1, 2, 2, 2),
        (1, 4, 2, 1),
        (0, 2, 1, 2),
    ];
    KnownLayouts::new().with(
        FIXTURE_SHIP,
        ShipShape {
            layout_id: "kestral".into(),
            rooms: rooms((0, 2), &kestrel),
            door_count: 26,
            multi_room_systems: vec![],
        },
    )
}

/// `(system, power, room, start, slot)` of a `systemList` entry.
#[allow(dead_code)]
type KestrelSystem = (
    SystemKind,
    i32,
    usize,
    bool,
    Option<(i32, StationDirection)>,
);

/// The `systemList` of `PLAYER_SHIP_HARD` in `data/blueprints.xml` of
/// 1.6.13 as `(system, power, room, start, slot)`. The Medbay and Clone Bay
/// slots name a square but no direction, so the port answers no slot.
#[allow(dead_code)]
pub const KESTREL_SYSTEMS: [KestrelSystem; 15] = [
    (
        SystemKind::Pilot,
        1,
        0,
        true,
        Some((0, StationDirection::Right)),
    ),
    (
        SystemKind::Doors,
        1,
        2,
        true,
        Some((1, StationDirection::Up)),
    ),
    (
        SystemKind::Sensors,
        1,
        3,
        true,
        Some((1, StationDirection::Down)),
    ),
    (SystemKind::Medbay, 1, 4, true, None),
    (SystemKind::Clonebay, 1, 4, false, None),
    (SystemKind::Oxygen, 1, 13, true, None),
    (SystemKind::Shields, 2, 5, true, None),
    (SystemKind::Engines, 2, 14, true, None),
    (SystemKind::Weapons, 3, 10, true, None),
    (SystemKind::Drones, 2, 1, false, None),
    (SystemKind::Teleporter, 1, 15, false, None),
    (SystemKind::Cloaking, 1, 8, false, None),
    (SystemKind::Battery, 1, 11, false, None),
    (SystemKind::Mind, 1, 7, false, None),
    (
        SystemKind::Hacking,
        1,
        12,
        false,
        Some((0, StationDirection::Down)),
    ),
];

/// [`KESTREL_SYSTEMS`] as the port answers them.
#[allow(dead_code)]
pub fn kestrel_system_rooms() -> Vec<SystemRoom> {
    KESTREL_SYSTEMS
        .iter()
        .map(|&(kind, power, room, start, slot)| SystemRoom {
            kind,
            room,
            power,
            max: None,
            start,
            slot: slot.map(|(square, direction)| Station { square, direction }),
        })
        .collect()
}

/// The hacking system's drone (`data/dlcBlueprints.xml`): power 1, no cooldown.
#[allow(dead_code)]
pub const HACKING_DRONE: DroneFacts = DroneFacts {
    power: 1,
    kind: DroneKind::Hacking,
    cooldown_ms: None,
};

/// [`item_layouts`] with the system rooms of the fixture ship and the
/// hacking drone: what installing and removing systems need.
#[allow(dead_code)]
pub fn install_layouts() -> KnownLayouts {
    item_layouts()
        .with_system_rooms(FIXTURE_SHIP, kestrel_system_rooms())
        .with_drone_facts("DRONE_HACKING", HACKING_DRONE)
}
