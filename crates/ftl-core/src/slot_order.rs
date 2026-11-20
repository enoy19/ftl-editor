//! The slot a new weapon, drone, augment or cargo item takes within one
//! patch: the first slot a removal of the same patch freed, else the end.
//!
//! FTL's weapon and drone slots are places: `WeaponSystem::AddWeapon(bp,
//! -1)` and `DroneSystem::AddDrone(d, -1)` put a new item into the next free
//! slot, and a removed item leaves its slot free while the others stay
//! where they are ([`crate::items`] module doc). The file stores the slots
//! in order without gaps, so the editor keeps that order: a removal leaves
//! a hole for the rest of the patch, and an addition fills the first hole.
//! A replace (remove `A`, add `B`) therefore puts `B` into `A`'s slot. The
//! cargo boxes and the augment list follow the same rule.
//!
//! The count edits ([`crate::items::set_count`], the augment and cargo
//! counts) only remove copies and append new ones. [`Holes::settle`] compares
//! each list before and after one assignment: the items that left are
//! holes, the items appended at the end move into the first hole. Holes
//! that no addition fills close at the end of the patch.

use crate::model::{Drone, ExtendedDroneInfo, SavedGame, ShipId, Weapon, WeaponModule};

/// One list with slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum List {
    Weapons(ShipId),
    Drones(ShipId),
    Augments(ShipId),
    Cargo,
}

const LISTS: [List; 7] = [
    List::Weapons(ShipId::Player),
    List::Drones(ShipId::Player),
    List::Augments(ShipId::Player),
    List::Weapons(ShipId::Nearby),
    List::Drones(ShipId::Nearby),
    List::Augments(ShipId::Nearby),
    List::Cargo,
];

/// One item with the state the file keeps beside it, so that two copies
/// of a blueprint differ when their state differs.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Item {
    Weapon(Weapon, Option<WeaponModule>),
    Drone(Drone, Option<ExtendedDroneInfo>),
    Id(String),
}

/// The items of every list of a game, in slot order.
#[derive(Debug, Clone)]
pub(crate) struct Snapshot(Vec<(List, Vec<Item>)>);

impl Snapshot {
    pub(crate) fn of(game: &SavedGame) -> Self {
        Snapshot(
            LISTS
                .iter()
                .filter_map(|&l| items(game, l).map(|i| (l, i)))
                .collect(),
        )
    }
}

fn items(game: &SavedGame, list: List) -> Option<Vec<Item>> {
    let body = |id: ShipId| game.ship(id).and_then(|s| s.body.as_ref());
    let extended = |id: ShipId| game.ship(id).and_then(|s| s.extended.as_ref());
    Some(match list {
        List::Weapons(id) => {
            let modules = extended(id).and_then(|e| e.weapon_modules.as_ref());
            body(id)?
                .weapons
                .iter()
                .enumerate()
                .map(|(i, w)| Item::Weapon(w.clone(), modules.and_then(|m| m.get(i)).cloned()))
                .collect()
        }
        List::Drones(id) => {
            let infos = extended(id).map(|e| &e.drones);
            body(id)?
                .drones
                .iter()
                .enumerate()
                .map(|(i, d)| Item::Drone(d.clone(), infos.and_then(|e| e.get(i)).cloned()))
                .collect()
        }
        List::Augments(id) => body(id)?.augments.iter().cloned().map(Item::Id).collect(),
        List::Cargo => game
            .world
            .as_ref()?
            .cargo
            .iter()
            .cloned()
            .map(Item::Id)
            .collect(),
    })
}

/// Moves the item at `from` to `to` in `list`, with the state the file
/// keeps beside it in a parallel list.
fn move_item(game: &mut SavedGame, list: List, from: usize, to: usize) {
    fn shift<T>(v: &mut Vec<T>, from: usize, to: usize) {
        if from < v.len() && to <= v.len() {
            let x = v.remove(from);
            v.insert(to, x);
        }
    }
    match list {
        List::Weapons(id) => {
            if let Some(s) = game.ship_mut(id) {
                if let Some(b) = s.body.as_mut() {
                    shift(&mut b.weapons, from, to);
                }
                if let Some(m) = s.extended.as_mut().and_then(|e| e.weapon_modules.as_mut()) {
                    shift(m, from, to);
                }
            }
        }
        List::Drones(id) => {
            if let Some(s) = game.ship_mut(id) {
                if let Some(b) = s.body.as_mut() {
                    shift(&mut b.drones, from, to);
                }
                if let Some(e) = s.extended.as_mut() {
                    shift(&mut e.drones, from, to);
                }
            }
        }
        List::Augments(id) => {
            if let Some(b) = game.ship_mut(id).and_then(|s| s.body.as_mut()) {
                shift(&mut b.augments, from, to);
            }
        }
        List::Cargo => {
            if let Some(w) = game.world.as_mut() {
                shift(&mut w.cargo, from, to);
            }
        }
    }
}

/// The free slots of every list within one patch, as the index a new item
/// takes in the list as it is now.
#[derive(Debug, Clone, Default)]
pub(crate) struct Holes(Vec<(List, Vec<usize>)>);

impl Holes {
    fn of(&mut self, list: List) -> &mut Vec<usize> {
        let i = match self.0.iter().position(|(l, _)| *l == list) {
            Some(i) => i,
            None => {
                self.0.push((list, vec![]));
                self.0.len() - 1
            }
        };
        &mut self.0[i].1
    }

    /// Records the slots one assignment freed in `game` since `before`, and
    /// moves the items it appended into the first free slots.
    pub(crate) fn settle(&mut self, before: &Snapshot, game: &mut SavedGame) {
        for (list, old) in &before.0 {
            let Some(new) = items(game, *list) else {
                continue;
            };
            if new.len() < old.len() {
                self.removed(*list, old, &new);
            } else if new.len() > old.len() && new[..old.len()] == old[..] {
                self.appended(*list, old.len(), new.len(), game);
            }
        }
    }

    /// `new` is `old` without some items: each one that left frees its slot.
    fn removed(&mut self, list: List, old: &[Item], new: &[Item]) {
        let mut j = 0;
        let mut gone = vec![];
        for (i, item) in old.iter().enumerate() {
            if j < new.len() && new[j] == *item {
                j += 1;
            } else {
                gone.push(i);
            }
        }
        if j != new.len() {
            // Not a plain removal; no count edit does this.
            return;
        }
        let holes = self.of(list);
        for h in holes.iter_mut() {
            *h -= gone.iter().filter(|&&g| g < *h).count();
        }
        // The k-th item gone (in order) left a slot at its index minus the
        // k items gone before it.
        holes.extend(gone.iter().enumerate().map(|(k, g)| g - k));
    }

    /// The items from `from` to `to` are new: each takes the first free slot.
    fn appended(&mut self, list: List, from: usize, to: usize, game: &mut SavedGame) {
        for i in from..to {
            let holes = self.of(list);
            let Some((at, &slot)) = holes.iter().enumerate().min_by_key(|(_, h)| **h) else {
                return;
            };
            holes.remove(at);
            for h in holes.iter_mut() {
                if *h >= slot {
                    *h += 1;
                }
            }
            move_item(game, list, i, slot);
        }
    }
}
