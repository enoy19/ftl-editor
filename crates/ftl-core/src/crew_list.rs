//! New crew members, removed crew members and the look of a crew member:
//! `crew.add`, `crew.<n>.remove`, `crew.<n>.tints.<layer>` and
//! `crew.<n>.male`, each also with `nearby.` for the ship at the beacon.
//!
//! Evidence (FTL 1.6.13, macOS binary, symbols read with Ghidra 12.1.4 on
//! 2026-11-19; field offsets from Hyperspace `FTLGameStripped.h`; the crew
//! written by FTL in `continue-1.6.13-format11.sav`):
//!
//! - Writing. `ShipManager::ExportShip` writes the crew of
//!   `CrewMemberFactory::GetCrewList`: every crew member whose current ship
//!   is this ship, plus the dead ones of its side waiting for a clone. Per
//!   member: `GetLongName`, the species, `IsDrone`, the integer health, the
//!   position (`GetLocation`, the sprite center), room and square, `iShipId
//!   == 0` (player controlled), `clone_ready`, `iDeathNumber`, one tint index
//!   per color layer (`CrewBlueprint::colorChoices`), `bMindControlled`, the
//!   saved position (square, then room), the six skill points, `male`, the
//!   five `SCrewStats` counters, then `CrewMember::SaveState`: `fStunTime`
//!   (`writeFloat`), `healthBoost`, `iDeathNumber` again, `fMindDamageBoost`
//!   (`writeFloat`, which stores `(int)(f * 1000)`), `fCloneDying` (the
//!   same), the static `iDeathCount`, the
//!   twelve mastery flags (`skillsEarned`), whether the crew animation is in
//!   state 6, the teleport animation and one more animation flag.
//! - A new member. `ShipManager::AddCrewMember(CrewBlueprint, ...)` (hire
//!   and event crew) asks `BlueprintManager::GetUnusedCrewName` when the
//!   blueprint has no name, builds the member with
//!   `CrewMemberFactory::CreateCrewmember` (`human` is a plain
//!   `CrewMember`) and appends it to the crew list, so it is written last.
//!   `CrewMember::CrewMember` sets: health full, `clone_ready` and
//!   `bMindControlled` false, `iDeathNumber` -1, the five counters 0,
//!   `healthBoost` 0, `fMindDamageBoost` 1.0 (1000 in the file),
//!   `fCloneDying` 0, the saved position -1, -1, `fStunTime` 0 and the
//!   twelve mastery flags false. `CrewBlueprint::CrewBlueprint` sets `male`
//!   true and the skills 0; `CrewBlueprint::ChooseColors` picks a random
//!   index per color layer, so index 0 is one of FTL's own choices. The nearby
//!   ship's crew of the fixture, which FTL built and never moved, carries
//!   exactly these values (saved position -1, -1; teleport animation
//!   playing, frame 0, scale 1000; animation flags off) and the same
//!   `universal_death_count` as every other member: FTL writes its one
//!   static counter for all ([`new_member`]).
//! - Where it stands. `AddCrewMember` with room -1 takes the room of the
//!   pilot system (`systemKey[6]`); `CrewMember::SetRoom` asks
//!   `Ship::GetAvailableRoom`, which keeps a room that is not full, else
//!   takes a connected room, else the free room whose center lies nearest;
//!   without a pilot room it takes the first free room in room order.
//!   `Room::GetEmptySlot` gives a member of the ship's own side the station
//!   square (`primarySlot`) while it is free, else the first free square.
//!   `ShipGraph::GetSlotWorldPosition` is the square's center: grid square
//!   times 35 plus 17, as every crew member of the fixture shows. The
//!   editor skips the connected-rooms step (the file has no door graph) and
//!   counts every crew member of the ship as taking its square ([`place`]).
//! - Limits. `ShipManager::IsCrewFull` is true when
//!   `CrewMemberFactory::GetCrewCount` of the ship's side reaches 8 for the
//!   player ship (`iShipId` 0) or 10 for the enemy (`8 + 2 * (id != 0)`), or
//!   when the flag after `bJumping` is set (Hyperspace: `bAutomated`, a
//!   drone ship). The count runs over both ships by owner, without drones.
//!   The crew store (`CrewStoreBox::CanHold`) and events ask it; the load
//!   (`ShipManager::ImportShip`) reads every member of the file without a
//!   limit, so a file beyond it loads, and `IsCrewOverFull` only answers
//!   more than the limit. The editor adds within the limit ([`crew_limit`])
//!   and needs a crew member of the ship's side already on the nearby ship:
//!   the file stores no automated flag, and a drone ship carries no crew.
//! - Removing. A member that leaves the file is simply not loaded; nothing
//!   else in the file refers to a crew member by index. But
//!   `CompleteShip::DeadCrew` is true once `GetCrewCount` of a side is 0 on
//!   a crewed ship: the game over for the player, the "crew killed" outcome
//!   for the enemy. The editor keeps one crew member of each side on its
//!   ship ([`EditError::LastCrewMember`]). A removal is staged on the member
//!   ([`CrewMember::removed`]) and done at the end of a patch, so every
//!   index of a patch is an index of the file ([`drop_removed`]).
//! - Look. FTL's character creator (`CrewCustomizeBox::MouseClick`) has two
//!   arrows. Each toggles `CrewMember::SetSex` and, except when a human just
//!   changed sex in the arrow's direction, calls `CycleColorLayer(+1 or
//!   -1)`, which moves the index of every color layer with colors by one,
//!   wrapping. `CrewAnimation::SetupStrips` draws `people/<race>_base.png`
//!   and `people/<race>_layer<k>.png` tinted with color `k` (from 1), and a
//!   female human (`male` false) as `female`. So the look is one tint index
//!   per layer ([`crate::Race::color_layers`]) and, for a human, the sex.
//!   `crew.<n>.tints.<layer>` sets one index within its layer;
//!   `crew.<n>.male` sets the sex of any race, as the creator does, but only
//!   a human looks different.

use crate::edit::{EditError, Field, ship, ship_mut};
use crate::installs;
use crate::layout::ShipLayouts;
use crate::model::{
    Anim, CrewMember, Room, SavedGame, Ship, ShipId, Skills, SystemKind, max_health, race,
};

/// The first file format that stores the tints ([`CrewMember::sprite_tints`]).
const TINT_FORMAT: i32 = 7;

/// The crew FTL lets a side hold: 8 for the player, 10 for the enemy
/// (`ShipManager::IsCrewFull`).
pub fn crew_limit(on: ShipId) -> usize {
    match on {
        ShipId::Player => 8,
        ShipId::Nearby => 10,
    }
}

/// Whether a crew member is of the side of ship `on`: the player's crew is
/// player controlled, wherever it stands.
fn of_side(c: &CrewMember, on: ShipId) -> bool {
    c.player_controlled == (on == ShipId::Player)
}

/// A crew member as FTL counts one: no boarding drone, not staged for removal.
fn counted(c: &CrewMember) -> bool {
    !c.enemy_boarding_drone && c.race != "battle" && !c.removed
}

/// The crew of the side of ship `on` on both ships, as `GetCrewCount` counts.
pub fn side_count(game: &SavedGame, on: ShipId) -> usize {
    [ShipId::Player, ShipId::Nearby]
        .into_iter()
        .filter_map(|id| game.ship(id))
        .flat_map(|s| &s.crew)
        .filter(|c| counted(c) && of_side(c, on))
        .count()
}

/// How many crew members `crew.add` can append to ship `on`: the limit of
/// its side minus the side's crew, 0 for a nearby ship without crew of its
/// own side (see the module doc).
pub fn free_places(game: &SavedGame, on: ShipId) -> usize {
    let own_aboard = game
        .ship(on)
        .is_some_and(|s| s.crew.iter().any(|c| counted(c) && of_side(c, on)));
    if on == ShipId::Nearby && !own_aboard {
        return 0;
    }
    crew_limit(on).saturating_sub(side_count(game, on))
}

/// The range of `crew.add`.
pub(crate) fn add_range(game: &SavedGame, field: &Field) -> Result<(i32, i32), EditError> {
    body_of(game, field)?;
    Ok((0, free_places(game, field.ship()) as i32))
}

/// Appends `count` default humans to the ship of `field`, each with a free
/// name of `data` and on a free square; returns 0, the old value.
pub(crate) fn add(
    game: &mut SavedGame,
    field: &Field,
    count: i32,
    data: &dyn ShipLayouts,
) -> Result<i32, EditError> {
    let names = data.crew_names(true);
    if names.is_empty() && count > 0 {
        return Err(EditError::NoCrewNames {
            field: field.clone(),
        });
    }
    let on = field.ship();
    let pilot_room = pilot_room(ship(game, field)?, data);
    for _ in 0..count {
        let name = free_name(game, &names);
        let universal = universal_death_count(game);
        let ship = ship_mut(game, field);
        let body = ship.body.as_ref().expect("the range checked the body");
        let (room, square) =
            place(&body.rooms, &ship.crew, pilot_room).ok_or_else(|| EditError::NoFreeSquare {
                field: field.clone(),
            })?;
        let member = new_member(name, on, &body.rooms[room], room, square, universal);
        ship.crew.push(member);
    }
    Ok(0)
}

/// The crew member FTL builds for a default human (module doc): full
/// health, no skills, male, tint 0 of each layer, at the center of `square`.
pub fn new_member(
    name: String,
    on: ShipId,
    room: &Room,
    room_id: usize,
    square: usize,
    universal_death_count: i32,
) -> CrewMember {
    let human = race("human").expect("human is a race");
    let (gx, gy) = room.grid_square(square);
    CrewMember {
        name,
        race: human.id.to_owned(),
        enemy_boarding_drone: false,
        health: max_health(human.id),
        sprite_x: gx * SQUARE + SQUARE / 2,
        sprite_y: gy * SQUARE + SQUARE / 2,
        room_id: room_id as i32,
        room_square: square as i32,
        player_controlled: on == ShipId::Player,
        clone_ready: 0,
        death_order: -1,
        sprite_tints: vec![0; human.color_layers.len()],
        mind_controlled: false,
        saved_room_square: -1,
        saved_room_id: -1,
        skills: Skills::default(),
        male: true,
        repairs: 0,
        combat_kills: 0,
        piloted_evasions: 0,
        jumps_survived: 0,
        skill_masteries_earned: 0,
        stun_ticks: 0,
        health_boost: 0,
        clonebay_priority: -1,
        damage_boost: 1000,
        clone_dying: 0,
        universal_death_count,
        teleporting: false,
        teleport_anim: Anim {
            playing: true,
            looping: false,
            current_frame: 0,
            progress_ticks: 0,
            scale: 1000,
            x: 0,
            y: 0,
        },
        teleport_anim_reverse: false,
        lockdown_recharge_ticks: 0,
        lockdown_recharge_ticks_goal: 0,
        cloned: 0,
        removed: false,
    }
}

/// Pixels per square of the ship grid.
const SQUARE: i32 = 35;

/// FTL's one counter of crew deaths, written into every crew member: the
/// value of the first member of either ship, else 0.
fn universal_death_count(game: &SavedGame) -> i32 {
    [ShipId::Player, ShipId::Nearby]
        .into_iter()
        .filter_map(|id| game.ship(id))
        .flat_map(|s| &s.crew)
        .map(|c| c.universal_death_count)
        .next()
        .unwrap_or(0)
}

/// The first name of `names` no crew member of either ship carries, else
/// the first name (FTL's `GetUnusedCrewName` also falls back to a used one).
fn free_name(game: &SavedGame, names: &[String]) -> String {
    let used: Vec<&str> = [ShipId::Player, ShipId::Nearby]
        .into_iter()
        .filter_map(|id| game.ship(id))
        .flat_map(|s| &s.crew)
        .map(|c| c.name.as_str())
        .collect();
    names
        .iter()
        .find(|n| !used.contains(&n.as_str()))
        .or(names.first())
        .cloned()
        .unwrap_or_default()
}

/// The room of the installed pilot system, when the blueprint names one.
fn pilot_room(ship: &Ship, data: &dyn ShipLayouts) -> Option<usize> {
    if !installs::is_installed(ship, SystemKind::Pilot) {
        return None;
    }
    data.system_rooms(&ship.blueprint_id)?
        .into_iter()
        .find(|r| r.kind == SystemKind::Pilot)
        .map(|r| r.room)
}

/// Where a new crew member stands: room and square id (module doc).
pub fn place(rooms: &[Room], crew: &[CrewMember], pilot: Option<usize>) -> Option<(usize, usize)> {
    let taken = |room: usize, square: usize| {
        crew.iter()
            .any(|c| !c.removed && c.room_id == room as i32 && c.room_square == square as i32)
    };
    let free_square = |room: usize| -> Option<usize> {
        let r = &rooms[room];
        let station = r
            .station
            .map(|s| s.square)
            .filter(|&s| s >= 0 && (s as usize) < r.squares.len())
            .map(|s| s as usize);
        station
            .filter(|&s| !taken(room, s))
            .or_else(|| (0..r.squares.len()).find(|&s| !taken(room, s)))
    };
    let Some(pilot) = pilot.filter(|&p| p < rooms.len()) else {
        return (0..rooms.len()).find_map(|r| free_square(r).map(|s| (r, s)));
    };
    if let Some(s) = free_square(pilot) {
        return Some((pilot, s));
    }
    let center = |r: &Room| {
        (
            f64::from(r.x) + f64::from(r.squares_h) / 2.0,
            f64::from(r.y) + f64::from(r.squares_v) / 2.0,
        )
    };
    let (px, py) = center(&rooms[pilot]);
    let mut best: Option<(f64, usize, usize)> = None;
    for (id, room) in rooms.iter().enumerate() {
        let Some(s) = free_square(id) else { continue };
        let (x, y) = center(room);
        let d = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
        if best.is_none_or(|(b, ..)| d < b) {
            best = Some((d, id, s));
        }
    }
    best.map(|(_, r, s)| (r, s))
}

fn body_of<'g>(game: &'g SavedGame, field: &Field) -> Result<&'g [Room], EditError> {
    ship(game, field)?
        .body
        .as_ref()
        .map(|b| b.rooms.as_slice())
        .ok_or_else(|| EditError::NotDecoded {
            field: field.clone(),
            part: "the room list of the ship",
        })
}

/// The range of `crew.<n>.remove`: 0 to 1, 0 to 0 for the last crew member
/// of its side on a ship.
pub(crate) fn remove_range(game: &SavedGame, field: &Field, crew: usize) -> (i32, i32) {
    (0, i32::from(!is_last(game, field, crew)))
}

fn is_last(game: &SavedGame, field: &Field, crew: usize) -> bool {
    let on = field.ship();
    let Some(ship) = game.ship(on) else {
        return false;
    };
    let member = &ship.crew[crew];
    if !counted(member) {
        return false;
    }
    let side = member.player_controlled;
    ship.crew
        .iter()
        .enumerate()
        .filter(|(i, c)| *i != crew && counted(c) && c.player_controlled == side)
        .count()
        == 0
}

/// Stages or unstages the removal of crew member `crew`; returns the old
/// value (1 when it was staged).
pub(crate) fn set_removed(
    game: &mut SavedGame,
    field: &Field,
    crew: usize,
    value: i32,
) -> Result<i32, EditError> {
    if value == 1 && is_last(game, field, crew) {
        return Err(EditError::LastCrewMember {
            field: field.clone(),
        });
    }
    let member = &mut ship_mut(game, field).crew[crew];
    let old = i32::from(member.removed);
    member.removed = value == 1;
    Ok(old)
}

/// Removes every crew member staged for removal from both ships.
pub fn drop_removed(game: &mut SavedGame) {
    for id in [ShipId::Player, ShipId::Nearby] {
        if let Some(ship) = game.ship_mut(id) {
            ship.crew.retain(|c| !c.removed);
        }
    }
}

/// The tint layers of a crew member: those of its race, none before format 7.
pub fn tint_layers(game: &SavedGame, member: &CrewMember) -> &'static [usize] {
    if game.file_format < TINT_FORMAT {
        return &[];
    }
    race(&member.race).map_or(&[], |r| r.color_layers)
}

/// The tint index of `layer`; a layer the file lacks reads 0.
pub(crate) fn tint(member: &CrewMember, layer: usize) -> i32 {
    member.sprite_tints.get(layer).copied().unwrap_or(0)
}

/// The range of `crew.<n>.tints.<layer>`: 0 to the last color of the layer.
pub(crate) fn tint_range(
    game: &SavedGame,
    field: &Field,
    member: &CrewMember,
    layer: usize,
) -> Result<(i32, i32), EditError> {
    let layers = tint_layers(game, member);
    let size = layers.get(layer).ok_or_else(|| EditError::NoTintLayer {
        field: field.clone(),
        race: member.race.clone(),
        layers: layers.len(),
    })?;
    Ok((0, *size as i32 - 1))
}

/// Writes one tint index; a shorter tint list grows to the layers of the
/// race first. Returns the old index.
pub(crate) fn set_tint(member: &mut CrewMember, layers: usize, layer: usize, value: i32) -> i32 {
    if member.sprite_tints.len() < layers {
        member.sprite_tints.resize(layers, 0);
    }
    std::mem::replace(&mut member.sprite_tints[layer], value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Square, Station, StationDirection};

    fn room(x: i32, y: i32, w: u32, h: u32, station: Option<i32>) -> Room {
        Room {
            x,
            y,
            squares_h: w,
            squares_v: h,
            oxygen: 100,
            squares: vec![
                Square {
                    fire_health: 0,
                    ignition_progress: 0,
                    extinguishment_progress: -1
                };
                (w * h) as usize
            ],
            station: station.map(|square| Station {
                square,
                direction: StationDirection::Up,
            }),
        }
    }

    fn at(room: usize, square: usize) -> CrewMember {
        let mut c = new_member("X".into(), ShipId::Player, &room_of(), room, square, 0);
        c.room_id = room as i32;
        c.room_square = square as i32;
        c
    }

    fn room_of() -> Room {
        room(0, 0, 2, 2, None)
    }

    #[test]
    fn a_new_member_takes_the_pilot_station_then_a_free_square_then_the_nearest_room() {
        let rooms = [
            room(0, 0, 2, 2, None),
            room(2, 0, 2, 1, Some(1)),
            room(10, 10, 1, 1, None),
        ];
        assert_eq!(place(&rooms, &[], Some(1)), Some((1, 1)));
        assert_eq!(place(&rooms, &[at(1, 1)], Some(1)), Some((1, 0)));
        // Pilot room full: room 0 lies nearer than room 2.
        assert_eq!(place(&rooms, &[at(1, 0), at(1, 1)], Some(1)), Some((0, 0)));
        // No pilot room: the first free room in room order.
        assert_eq!(place(&rooms, &[at(0, 0)], None), Some((0, 1)));
        let mut gone = at(0, 0);
        gone.removed = true;
        assert_eq!(place(&rooms, &[gone], None), Some((0, 0)));
    }

    #[test]
    fn a_new_member_stands_at_the_center_of_its_grid_square() {
        // Elizabeth of the fixture: square 1 of room 10, a 2x2 room at grid
        // 4, 4, stands at 192, 157.
        let r = room(4, 4, 2, 2, None);
        let c = new_member("Kirby".into(), ShipId::Player, &r, 10, 1, 114);
        assert_eq!((c.sprite_x, c.sprite_y), (192, 157));
        assert_eq!((c.room_id, c.room_square), (10, 1));
        assert_eq!((c.saved_room_id, c.saved_room_square), (-1, -1));
        assert_eq!((c.health, c.race.as_str(), c.male), (100, "human", true));
        assert_eq!(c.sprite_tints, [0, 0]);
        assert_eq!((c.death_order, c.clonebay_priority), (-1, -1));
        assert_eq!((c.damage_boost, c.universal_death_count), (1000, 114));
        assert!(c.player_controlled);
        let enemy = new_member("Jon".into(), ShipId::Nearby, &r, 0, 0, 0);
        assert!(!enemy.player_controlled);
    }
}
