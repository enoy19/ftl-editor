//! Cargo to distant quests: wire to domain and back.

use ftl_core::{
    Beacon, BeaconBackground, BeaconEnemy, FileFormat, FleetPresence, QuestEvent, RebelFleet,
    Store, StoreItem, StoreItemKind, StoreShelf, World,
};

use crate::wire::{
    Bool32, FtlString, WireBeacon, WireBeaconBackground, WireBeaconEnemy, WireQuestEvent,
    WireSectorAe, WireSectorClassic, WireStore, WireStoreItem, WireStoreShelf, WireStoreSlot,
    WireWorld,
};

pub fn to_domain(w: WireWorld) -> World {
    let (current_beacon_id, waiting, wait_event_seed, last_escape_event, hazards, flagship) =
        match (w.sector_ae, w.sector_classic) {
            (Some(a), _) => (
                a.current_beacon_id,
                a.waiting.0,
                a.wait_event_seed,
                a.last_escape_event.0,
                a.sector_hazards_visible.0,
                (
                    a.rebel_flagship_visible.0,
                    a.rebel_flagship_hop,
                    a.rebel_flagship_moving.0,
                    a.rebel_flagship_retreating.0,
                    a.rebel_flagship_base_turns,
                ),
            ),
            (None, Some(c)) => (
                w.current_beacon_id_classic.unwrap_or(0),
                false,
                -1,
                String::new(),
                c.sector_hazards_visible.0,
                (
                    c.rebel_flagship_visible.0,
                    c.rebel_flagship_hop,
                    c.rebel_flagship_moving.0,
                    false,
                    0,
                ),
            ),
            (None, None) => unreachable!("the reader sets one of the two by format"),
        };
    World {
        cargo: w.cargo.into_iter().map(|c| c.0).collect(),
        sector_tree_seed: w.sector_tree_seed,
        sector_layout_seed: w.sector_layout_seed,
        rebel_fleet: RebelFleet {
            offset: w.rebel_fleet_offset,
            fudge: w.rebel_fleet_fudge,
            pursuit_mod: w.rebel_pursuit_mod,
            flagship_visible: flagship.0,
            flagship_hop: flagship.1,
            flagship_moving: flagship.2,
            flagship_retreating: flagship.3,
            flagship_base_turns: flagship.4,
        },
        current_beacon_id,
        waiting,
        wait_event_seed,
        last_escape_event,
        sector_hazards_visible: hazards,
        sector_visitation: w.sector_visitation.into_iter().map(|v| v.0).collect(),
        sector_number: w.sector_number,
        sector_is_hidden_crystal_worlds: w.sector_is_hidden_crystal_worlds.0,
        beacons: w.beacons.into_iter().map(beacon_to_domain).collect(),
        quest_events: w
            .quest_events
            .into_iter()
            .map(|q| QuestEvent {
                event_id: q.event_id.0,
                beacon_id: q.beacon_id,
            })
            .collect(),
        distant_quest_events: w.distant_quest_events.into_iter().map(|q| q.0).collect(),
    }
}

fn beacon_to_domain(b: WireBeacon) -> Beacon {
    Beacon {
        visit_count: b.visit_count,
        background: b.background.map(|g| BeaconBackground {
            starscape: g.starscape.0,
            sprite: g.sprite.0,
            sprite_x: g.sprite_x,
            sprite_y: g.sprite_y,
            sprite_rotation: g.sprite_rotation,
        }),
        seen: b.seen.0,
        enemy: b.enemy.map(|e| BeaconEnemy {
            ship_event_id: e.ship_event_id.0,
            auto_blueprint_id: e.auto_blueprint_id.0,
            ship_event_seed: e.ship_event_seed,
        }),
        fleet_presence: FleetPresence::ALL[b.fleet_presence as usize],
        under_attack: b.under_attack.0,
        store: b.store.map(|s| Store {
            shelves: s
                .shelves
                .into_iter()
                .map(|sh| StoreShelf {
                    kind: StoreItemKind::ALL[sh.kind as usize],
                    slots: sh.slots.map(|slot| {
                        slot.item.map(|i| StoreItem {
                            id: i.id.0,
                            available: slot.available,
                            extra_data: i.extra_data.unwrap_or(0),
                        })
                    }),
                })
                .collect(),
            fuel: s.fuel,
            missiles: s.missiles,
            drone_parts: s.drone_parts,
        }),
    }
}

pub fn to_wire(w: &World, f: FileFormat) -> WireWorld {
    let s = |v: &str| FtlString(v.to_owned());
    let fleet = &w.rebel_fleet;
    WireWorld {
        cargo: w.cargo.iter().map(|c| s(c)).collect(),
        sector_tree_seed: w.sector_tree_seed,
        sector_layout_seed: w.sector_layout_seed,
        rebel_fleet_offset: fleet.offset,
        rebel_fleet_fudge: fleet.fudge,
        rebel_pursuit_mod: fleet.pursuit_mod,
        sector_ae: (f >= 7).then(|| WireSectorAe {
            current_beacon_id: w.current_beacon_id,
            waiting: Bool32(w.waiting),
            wait_event_seed: w.wait_event_seed,
            last_escape_event: s(&w.last_escape_event),
            sector_hazards_visible: Bool32(w.sector_hazards_visible),
            rebel_flagship_visible: Bool32(fleet.flagship_visible),
            rebel_flagship_hop: fleet.flagship_hop,
            rebel_flagship_moving: Bool32(fleet.flagship_moving),
            rebel_flagship_retreating: Bool32(fleet.flagship_retreating),
            rebel_flagship_base_turns: fleet.flagship_base_turns,
        }),
        sector_classic: (f == 2).then_some(WireSectorClassic {
            sector_hazards_visible: Bool32(w.sector_hazards_visible),
            rebel_flagship_visible: Bool32(fleet.flagship_visible),
            rebel_flagship_hop: fleet.flagship_hop,
            rebel_flagship_moving: Bool32(fleet.flagship_moving),
        }),
        sector_visitation: w.sector_visitation.iter().map(|&v| Bool32(v)).collect(),
        sector_number: w.sector_number,
        sector_is_hidden_crystal_worlds: Bool32(w.sector_is_hidden_crystal_worlds),
        beacons: w.beacons.iter().map(|b| beacon_to_wire(b, f)).collect(),
        quest_events: w
            .quest_events
            .iter()
            .map(|q| WireQuestEvent {
                event_id: s(&q.event_id),
                beacon_id: q.beacon_id,
            })
            .collect(),
        distant_quest_events: w.distant_quest_events.iter().map(|q| s(q)).collect(),
        current_beacon_id_classic: (f == 2).then_some(w.current_beacon_id),
    }
}

fn beacon_to_wire(b: &Beacon, f: FileFormat) -> WireBeacon {
    let s = |v: &str| FtlString(v.to_owned());
    WireBeacon {
        visit_count: b.visit_count,
        background: b.background.as_ref().map(|g| WireBeaconBackground {
            starscape: s(&g.starscape),
            sprite: s(&g.sprite),
            sprite_x: g.sprite_x,
            sprite_y: g.sprite_y,
            sprite_rotation: g.sprite_rotation,
        }),
        seen: Bool32(b.seen),
        enemy_present: Bool32(b.enemy.is_some()),
        enemy: b.enemy.as_ref().map(|e| WireBeaconEnemy {
            ship_event_id: s(&e.ship_event_id),
            auto_blueprint_id: s(&e.auto_blueprint_id),
            ship_event_seed: e.ship_event_seed,
        }),
        fleet_presence: b.fleet_presence as i32,
        under_attack: Bool32(b.under_attack),
        store_present: Bool32(b.store.is_some()),
        store: b.store.as_ref().map(|st| WireStore {
            shelves: st
                .shelves
                .iter()
                .map(|sh| WireStoreShelf {
                    kind: sh.kind as i32,
                    slots: sh.slots.each_ref().map(|slot| match slot {
                        Some(i) => WireStoreSlot {
                            available: i.available,
                            item: Some(WireStoreItem {
                                id: s(&i.id),
                                extra_data: (f >= 8).then_some(i.extra_data),
                            }),
                        },
                        None => WireStoreSlot {
                            available: -1,
                            item: None,
                        },
                    }),
                })
                .collect(),
            fuel: st.fuel,
            missiles: st.missiles,
            drone_parts: st.drone_parts,
        }),
    }
}
