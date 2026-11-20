//! The sector map after the player ship (`readSavedGame` from the cargo to
//! the distant quests, `readBeacon`, `readStoreShelf`).

use binrw::binrw;

use super::{Bool32, FtlString};

/// Cargo to distant quests. Format 2 stores the current beacon id after
/// the quests; later formats store it among the sector fields. Both land in
/// this struct, so the pass after it starts at the encounter (format 7+) or
/// at the nearby ship flag (format 2).
#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireWorld {
    #[br(temp)]
    #[bw(calc = cargo.len() as u32)]
    cargo_count: u32,
    #[br(count = cargo_count, args { inner: (format >= 11,) })]
    #[bw(args(format >= 11))]
    pub cargo: Vec<FtlString>,
    pub sector_tree_seed: i32,
    pub sector_layout_seed: i32,
    pub rebel_fleet_offset: i32,
    pub rebel_fleet_fudge: i32,
    pub rebel_pursuit_mod: i32,
    #[brw(if(format >= 7), args(format))]
    pub sector_ae: Option<WireSectorAe>,
    #[brw(if(format == 2))]
    pub sector_classic: Option<WireSectorClassic>,
    #[br(temp)]
    #[bw(calc = sector_visitation.len() as u32)]
    visitation_count: u32,
    #[br(count = visitation_count)]
    pub sector_visitation: Vec<Bool32>,
    pub sector_number: i32,
    pub sector_is_hidden_crystal_worlds: Bool32,
    #[br(temp)]
    #[bw(calc = beacons.len() as u32)]
    beacon_count: u32,
    #[br(count = beacon_count, args { inner: (format,) })]
    #[bw(args(format))]
    pub beacons: Vec<WireBeacon>,
    #[br(temp)]
    #[bw(calc = quest_events.len() as u32)]
    quest_count: u32,
    #[br(count = quest_count, args { inner: (format >= 11,) })]
    #[bw(args(format >= 11))]
    pub quest_events: Vec<WireQuestEvent>,
    #[br(temp)]
    #[bw(calc = distant_quest_events.len() as u32)]
    distant_quest_count: u32,
    #[br(count = distant_quest_count, args { inner: (format >= 11,) })]
    #[bw(args(format >= 11))]
    pub distant_quest_events: Vec<FtlString>,
    /// Format 2 only.
    #[brw(if(format == 2))]
    pub current_beacon_id_classic: Option<i32>,
}

/// Format 7 and later: sector fields after the rebel pursuit mod.
#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireSectorAe {
    pub current_beacon_id: i32,
    pub waiting: Bool32,
    pub wait_event_seed: i32,
    #[brw(args(format >= 11))]
    pub last_escape_event: FtlString,
    pub sector_hazards_visible: Bool32,
    pub rebel_flagship_visible: Bool32,
    pub rebel_flagship_hop: i32,
    pub rebel_flagship_moving: Bool32,
    pub rebel_flagship_retreating: Bool32,
    pub rebel_flagship_base_turns: i32,
}

/// Format 2: sector fields after the rebel pursuit mod.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireSectorClassic {
    pub sector_hazards_visible: Bool32,
    pub rebel_flagship_visible: Bool32,
    pub rebel_flagship_hop: i32,
    pub rebel_flagship_moving: Bool32,
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireBeacon {
    pub visit_count: i32,
    #[br(if(visit_count > 0), args(format >= 11))]
    #[bw(args(format >= 11))]
    pub background: Option<WireBeaconBackground>,
    pub seen: Bool32,
    pub enemy_present: Bool32,
    #[br(if(enemy_present.0), args(format >= 11))]
    #[bw(args(format >= 11))]
    pub enemy: Option<WireBeaconEnemy>,
    /// 0 none, 1 rebel, 2 federation, 3 both.
    #[br(assert((0..=3).contains(&fleet_presence), "fleet presence flag {} is outside 0..=3", fleet_presence))]
    pub fleet_presence: i32,
    pub under_attack: Bool32,
    pub store_present: Bool32,
    #[br(if(store_present.0), args(format))]
    #[bw(args(format))]
    pub store: Option<WireStore>,
}

#[binrw]
#[brw(little, import(utf8: bool))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireBeaconBackground {
    #[brw(args(utf8))]
    pub starscape: FtlString,
    #[brw(args(utf8))]
    pub sprite: FtlString,
    pub sprite_x: i32,
    pub sprite_y: i32,
    pub sprite_rotation: i32,
}

#[binrw]
#[brw(little, import(utf8: bool))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireBeaconEnemy {
    #[brw(args(utf8))]
    pub ship_event_id: FtlString,
    #[brw(args(utf8))]
    pub auto_blueprint_id: FtlString,
    pub ship_event_seed: i32,
}

/// A store. Format 2 has two shelves and no shelf count.
#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireStore {
    #[br(temp, if(format >= 7))]
    #[bw(calc = (format >= 7).then_some(shelves.len() as u32))]
    shelf_count: Option<u32>,
    #[br(count = shelf_count.unwrap_or(2), args { inner: (format,) })]
    #[bw(args(format))]
    pub shelves: Vec<WireStoreShelf>,
    pub fuel: i32,
    pub missiles: i32,
    pub drone_parts: i32,
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireStoreShelf {
    /// 0 weapon, 1 drone, 2 augment, 3 crew, 4 system.
    #[br(assert((0..=4).contains(&kind), "store item type {} is outside 0..=4", kind))]
    pub kind: i32,
    #[brw(args(format))]
    pub slots: [WireStoreSlot; 3],
}

/// A shelf slot: the availability flag, then an item unless it is negative
/// (-1 is an empty slot).
#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireStoreSlot {
    pub available: i32,
    #[br(if(available >= 0), args(format))]
    #[bw(args(format))]
    pub item: Option<WireStoreItem>,
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireStoreItem {
    #[brw(args(format >= 11))]
    pub id: FtlString,
    /// Format 8 and later.
    #[brw(if(format >= 8))]
    pub extra_data: Option<i32>,
}

#[binrw]
#[brw(little, import(utf8: bool))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireQuestEvent {
    #[brw(args(utf8))]
    pub event_id: FtlString,
    pub beacon_id: i32,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite};

    use super::*;

    fn int(v: i32) -> Vec<u8> {
        v.to_le_bytes().to_vec()
    }

    fn string(s: &str) -> Vec<u8> {
        [int(s.len() as i32), s.as_bytes().to_vec()].concat()
    }

    fn round_trip<T>(bytes: &[u8], format: i32) -> T
    where
        T: for<'a> BinRead<Args<'a> = (i32,)> + for<'a> BinWrite<Args<'a> = (i32,)>,
    {
        let mut r = Cursor::new(bytes);
        let v = T::read_le_args(&mut r, (format,)).unwrap();
        assert_eq!(r.position() as usize, bytes.len(), "read every byte");
        let mut out = Cursor::new(Vec::new());
        v.write_le_args(&mut out, (format,)).unwrap();
        assert_eq!(out.into_inner(), bytes, "writes the same bytes");
        v
    }

    /// Not seen in a real file: no beacon of the fixture has a store.
    #[test]
    fn a_format_11_store_keeps_empty_slots_in_place() {
        let shelf = [
            int(0), // weapon shelf
            int(1),
            string("LASER_BURST_3"),
            int(0),
            int(-1), // empty middle slot
            int(0),
            string("ION_1"),
            int(7),
        ]
        .concat();
        let bytes = [
            int(1), // visit count
            string("stars/bg.png"),
            string("stars/planet.png"),
            int(10),
            int(20),
            int(180),
            int(1), // seen
            int(0), // no enemy
            int(2), // federation
            int(0), // not under attack
            int(1), // store
            int(1), // one shelf
            shelf,
            int(5),
            int(6),
            int(7),
        ]
        .concat();
        let b: WireBeacon = round_trip(&bytes, 11);
        let store = b.store.unwrap();
        assert_eq!((store.fuel, store.missiles, store.drone_parts), (5, 6, 7));
        let slots = &store.shelves[0].slots;
        assert_eq!(slots[0].item.as_ref().unwrap().id.0, "LASER_BURST_3");
        assert_eq!(slots[1].available, -1);
        assert!(slots[1].item.is_none());
        assert_eq!(slots[2].item.as_ref().unwrap().extra_data, Some(7));
    }

    /// Not seen in a real file: format 2 has two shelves without a count
    /// and items without extra data.
    #[test]
    fn a_format_2_store_has_two_shelves_without_extra_data() {
        let shelf =
            |kind: i32, id: &str| [int(kind), int(1), string(id), int(-1), int(-1)].concat();
        let bytes = [
            int(0), // never visited: no background
            int(0),
            int(1), // enemy
            string("PIRATE"),
            string("PIRATE_SHIP"),
            int(42),
            int(1),
            int(1),
            int(1), // store
            shelf(1, "COMBAT_1"),
            shelf(3, "human"),
            int(1),
            int(2),
            int(3),
        ]
        .concat();
        let b: WireBeacon = round_trip(&bytes, 2);
        assert!(b.background.is_none());
        assert_eq!(b.enemy.unwrap().ship_event_seed, 42);
        let store = b.store.unwrap();
        assert_eq!(store.shelves.len(), 2);
        assert_eq!(store.shelves[1].kind, 3);
        assert_eq!(
            store.shelves[1].slots[0].item.as_ref().unwrap().extra_data,
            None
        );
    }

    #[test]
    fn an_unknown_fleet_presence_is_refused() {
        let bytes = [int(0), int(0), int(0), int(4), int(0), int(0)].concat();
        let err = WireBeacon::read_le_args(&mut Cursor::new(&bytes), (11,)).unwrap_err();
        assert!(err.to_string().contains("fleet presence flag 4"), "{err}");
    }
}
