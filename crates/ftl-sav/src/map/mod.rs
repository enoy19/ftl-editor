//! Wire structs to domain model and back. One module per area of the file.

mod body;
mod crew;
mod encounter;
mod extended;
mod projectile;
mod systems;
mod world;

use ftl_core::{
    DecodeError, Difficulty, Resources, SavedGame, SessionStats, Ship, ShipShape, StartingCrew,
    StateVar,
};

use crate::wire::{
    Bool32, FtlString, ShipFacts, WireJumpState, WireSave, WireShipBody, WireShipHead,
    WireShipSystems, WireSituation, WireStartingCrew, WireStateVar, WireWorld,
};

/// The read passes of a saved game (see [`crate::wire`]) and the bytes after
/// the last one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireGame {
    /// Its `rest` is empty: the bytes after the head are split into the
    /// fields below.
    pub head: WireSave,
    pub systems: WireShipSystems,
    /// System kind index (into `SystemKind::ALL`) of each system slot.
    pub kinds: Vec<usize>,
    /// The body with the shape it was read with.
    pub body: Option<(WireShipBody, ShipShape)>,
    /// Read only after a body.
    pub world: Option<WireWorld>,
    /// Read only after a world, with the system kinds and the shape of the
    /// nearby ship.
    pub situation: Option<(WireSituation, Option<ShipFacts>)>,
    pub rest: Vec<u8>,
}

/// The wire form of every pass that [`to_wire`] answers, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireParts {
    pub head: WireSave,
    pub systems: WireShipSystems,
    pub body: Option<WireShipBody>,
    pub world: Option<WireWorld>,
    pub situation: Option<WireSituation>,
}

pub fn to_domain(g: WireGame) -> Result<SavedGame, DecodeError> {
    let WireGame {
        head: w,
        systems: sys,
        kinds,
        body: wire_body,
        world: wire_world,
        situation: wire_situation,
        rest,
    } = g;
    let difficulty = match (difficulty_flag(w.difficulty, w.format), w.format) {
        (0, _) => Difficulty::Easy,
        (1, _) => Difficulty::Normal,
        (2, f) if f >= 7 => Difficulty::Hard,
        (v, f) => {
            return Err(DecodeError {
                // format, [random_native], [dlc_enabled], then difficulty.
                offset: 4 + 4 * usize::from(f == 11) + 4 * usize::from(f >= 7),
                field: "difficulty",
                problem: format!("unknown difficulty flag {v} for file format {f}"),
            });
        }
    };
    let mut player_ship = ship_to_domain(w.player_ship, sys, &kinds, wire_body)?;
    let situation = match wire_situation {
        Some((s, nearby_shape)) => {
            let (situation, player_extended) = encounter::to_domain(s, nearby_shape)?;
            player_ship.extended = player_extended;
            Some(situation)
        }
        None => None,
    };
    Ok(SavedGame {
        file_format: w.format,
        random_native: w.random_native.is_none_or(|b| b.0),
        dlc_enabled: w.dlc_enabled.is_some_and(|b| b.0),
        difficulty,
        stats: SessionStats {
            ships_defeated: w.ships_defeated,
            beacons_explored: w.beacons_explored,
            scrap_collected: w.scrap_collected,
            crew_hired: w.crew_hired,
        },
        player_ship_name: w.player_ship_name.0,
        player_ship_blueprint_id: w.player_ship_blueprint_id.0,
        sector_number: w.sector_number,
        victory: w.victory,
        state_vars: w
            .state_vars
            .into_iter()
            .map(|v| StateVar {
                id: v.id.0,
                value: v.value,
            })
            .collect(),
        player_ship,
        world: wire_world.map(world::to_domain),
        situation,
        undecoded: rest,
    })
}

/// A ship from its three wire parts. `kinds` names the system kind of each
/// slot; the body comes with the shape it was read with. The extended info
/// stays `None`: the file stores it apart from the ship.
pub fn ship_to_domain(
    head: WireShipHead,
    sys: WireShipSystems,
    kinds: &[usize],
    wire_body: Option<(WireShipBody, ShipShape)>,
) -> Result<Ship, DecodeError> {
    let jump = head.jump.unwrap_or(WireJumpState {
        hostile: Bool32(false),
        jump_charge_ticks: 0,
        jumping: Bool32(false),
        jump_anim_ticks: 0,
    });
    Ok(Ship {
        blueprint_id: head.blueprint_id.0,
        name: head.name.0,
        gfx_base_name: head.gfx_base_name.0,
        starting_crew: head
            .starting_crew
            .into_iter()
            .map(|c| StartingCrew {
                race: c.race.0,
                name: c.name.0,
            })
            .collect(),
        hostile: jump.hostile.0,
        jump_charge_ticks: jump.jump_charge_ticks,
        jumping: jump.jumping.0,
        jump_anim_ticks: jump.jump_anim_ticks,
        resources: Resources {
            hull: head.hull,
            fuel: head.fuel,
            drone_parts: head.drone_parts,
            missiles: head.missiles,
            scrap: head.scrap,
        },
        crew: head.crew.into_iter().map(crew::to_domain).collect(),
        reactor_power: sys.reactor_power,
        systems: systems::systems_to_domain(sys.systems, kinds)?,
        clonebay: sys.clonebay.map(systems::clonebay_to_domain),
        battery: sys.battery.map(systems::battery_to_domain),
        shields: sys.shields.map(systems::shields_to_domain),
        cloaking: sys.cloaking.map(systems::cloaking_to_domain),
        body: wire_body.map(|(b, shape)| body::to_domain(b, &shape)),
        extended: None,
    })
}

/// The wire form of every pass. The writer needs no system kinds and no
/// body shape.
pub fn to_wire(g: &SavedGame) -> WireParts {
    let f = g.file_format;
    let (player_head, systems, body) = ship_to_wire(&g.player_ship, f);
    WireParts {
        head: head_to_wire(g, player_head),
        systems,
        body,
        world: g.world.as_ref().map(|w| world::to_wire(w, f)),
        situation: g
            .situation
            .as_ref()
            .map(|s| encounter::to_wire(s, g.player_ship.extended.as_ref(), f)),
    }
}

/// The three wire parts of a ship. Its extended info is written apart.
pub fn ship_to_wire(ship: &Ship, f: i32) -> (WireShipHead, WireShipSystems, Option<WireShipBody>) {
    let s = |v: &str| FtlString(v.to_owned());
    let r = &ship.resources;
    let head = WireShipHead {
        blueprint_id: s(&ship.blueprint_id),
        name: s(&ship.name),
        gfx_base_name: s(&ship.gfx_base_name),
        starting_crew: ship
            .starting_crew
            .iter()
            .map(|c| WireStartingCrew {
                race: s(&c.race),
                name: s(&c.name),
            })
            .collect(),
        jump: (f >= 7).then_some(WireJumpState {
            hostile: Bool32(ship.hostile),
            jump_charge_ticks: ship.jump_charge_ticks,
            jumping: Bool32(ship.jumping),
            jump_anim_ticks: ship.jump_anim_ticks,
        }),
        hull: r.hull,
        fuel: r.fuel,
        drone_parts: r.drone_parts,
        missiles: r.missiles,
        scrap: r.scrap,
        crew: ship.crew.iter().map(|c| crew::to_wire(c, f)).collect(),
    };
    let systems = WireShipSystems {
        reactor_power: ship.reactor_power,
        systems: systems::systems_to_wire(&ship.systems, f),
        clonebay: ship.clonebay.as_ref().map(systems::clonebay_to_wire),
        battery: ship.battery.as_ref().map(systems::battery_to_wire),
        shields: ship.shields.as_ref().map(systems::shields_to_wire),
        cloaking: ship.cloaking.as_ref().map(systems::cloaking_to_wire),
    };
    let body = ship.body.as_ref().map(|b| body::to_wire(b, f));
    (head, systems, body)
}

/// The stored difficulty flag as FTL 1.6.13 means it, and back: from format
/// 3 on 0 Easy, 1 Normal, 2 Hard; below it the flag is inverted, 0 Normal
/// and 1 Easy (`ScoreKeeper::LoadGame`, macOS 0x100029940:
/// `difficulty = (flag == 0)` unless the format is above 2). The mapping is
/// its own inverse for 0 and 1. Java's `SavedGameParser` reads 0 as Easy in
/// every format.
fn difficulty_flag(flag: i32, format: i32) -> i32 {
    match (format, flag) {
        (..=2, 0) => 1,
        (..=2, 1) => 0,
        _ => flag,
    }
}

fn head_to_wire(g: &SavedGame, player_ship: WireShipHead) -> WireSave {
    let f = g.file_format;
    let s = |v: &str| FtlString(v.to_owned());
    WireSave {
        format: f,
        random_native: (f == 11).then_some(Bool32(g.random_native)),
        dlc_enabled: (f >= 7).then_some(Bool32(g.dlc_enabled)),
        difficulty: difficulty_flag(
            match g.difficulty {
                Difficulty::Easy => 0,
                Difficulty::Normal => 1,
                Difficulty::Hard => 2,
            },
            f,
        ),
        ships_defeated: g.stats.ships_defeated,
        beacons_explored: g.stats.beacons_explored,
        scrap_collected: g.stats.scrap_collected,
        crew_hired: g.stats.crew_hired,
        player_ship_name: s(&g.player_ship_name),
        player_ship_blueprint_id: s(&g.player_ship_blueprint_id),
        sector_number: g.sector_number,
        victory: g.victory,
        state_vars: g
            .state_vars
            .iter()
            .map(|v| WireStateVar {
                id: s(&v.id),
                value: v.value,
            })
            .collect(),
        player_ship,
        rest: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::difficulty_flag;

    #[test]
    fn format_2_stores_the_difficulty_inverted() {
        // ScoreKeeper::LoadGame: below format 3, flag 0 is Normal, 1 Easy.
        assert_eq!(difficulty_flag(0, 2), 1);
        assert_eq!(difficulty_flag(1, 2), 0);
        for format in [7, 9, 11] {
            for flag in 0..=2 {
                assert_eq!(difficulty_flag(flag, format), flag);
            }
        }
        // The mapping is its own inverse, so decode and encode share it.
        for flag in 0..=1 {
            assert_eq!(difficulty_flag(difficulty_flag(flag, 2), 2), flag);
        }
    }
}
