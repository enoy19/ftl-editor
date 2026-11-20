//! After the sector map (`readSavedGame` from the encounter on):
//! `readEncounter`, `readNearbyShipAI`, `readEnvironment`, the rebel
//! flagship and the scalars between them.
//!
//! The nearby ship itself is a [`super::WireShipHead`], [`super::WireShipSystems`]
//! and [`super::WireShipBody`]; the extended infos are in [`super::extended`].
//! The order on disk is in [`WireSituation`].

use std::io::{Read, Seek, Write};

use binrw::{BinRead, BinResult, BinWrite, binrw};
use ftl_core::{DroneKind, ShipShape};

use super::{
    Bool32, ExtendedShape, FtlString, WireExtendedShipInfo, WireProjectile, WireShipBody,
    WireShipHead, WireShipSystems, read_extended,
};

/// Everything after the sector map, in file order. Read piece by piece by
/// the codec, because the nearby ship needs its layout and the extended
/// infos need drone kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireSituation {
    /// Format 7 and later.
    pub ae: Option<WireSituationAe>,
    pub nearby: Option<WireNearbyShip>,
    /// Format 7 and later.
    pub environment: Option<WireEnvironment>,
    /// Format 7 and later.
    pub projectiles: Vec<WireProjectile>,
    /// Format 7 and later: player, then nearby ship.
    pub player_extended: Option<WireExtendedShipInfo>,
    pub tail: WireSituationTail,
}

/// The system kind index of each slot and the shape of a ship.
pub type ShipFacts = (Vec<usize>, ShipShape);

/// What the reader needs from `ftl.dat`.
pub struct SituationFacts<'a> {
    /// The system kinds per slot and the shape of a ship blueprint id.
    pub ship: &'a dyn Fn(&str) -> Option<ShipFacts>,
    pub drone_kind: &'a dyn Fn(&str) -> Option<DroneKind>,
    /// The extended shape of the player ship; `None` when a drone kind is
    /// unknown.
    pub player: Option<ExtendedShape>,
}

impl WireSituation {
    /// Reads everything after the sector map. `Ok(None)` when a fact is
    /// missing (nearby ship layout, drone kind); the caller then keeps the
    /// bytes undecoded. Beside the situation it answers the kinds and the
    /// shape of the nearby ship.
    pub fn read<R: Read + Seek>(
        r: &mut R,
        format: i32,
        facts: &SituationFacts,
    ) -> BinResult<Option<(WireSituation, Option<ShipFacts>)>> {
        let ae = format >= 7;
        let head = if ae {
            Some(WireSituationAe::read_le_args(r, (format,))?)
        } else {
            None
        };
        let nearby_present = Bool32::read_le(r)?;
        let mut nearby = None;
        let mut nearby_shape = None;
        if nearby_present.0 {
            let rebel_flagship_nearby = if ae { Some(Bool32::read_le(r)?) } else { None };
            let ship_head = WireShipHead::read_le_args(r, (format,))?;
            let Some((kinds, shape)) = (facts.ship)(&ship_head.blueprint_id.0) else {
                return Ok(None);
            };
            let systems = WireShipSystems::read_le_args(r, (format, kinds.clone()))?;
            let rooms = shape
                .rooms
                .iter()
                .map(|r| (r.squares_h, r.squares_v))
                .collect();
            let body = WireShipBody::read_le_args(r, (format, rooms, shape.door_count))?;
            let ai = if ae {
                Some(WireNearbyShipAi::read_le(r)?)
            } else {
                None
            };
            nearby = Some(WireNearbyShip {
                rebel_flagship_nearby,
                head: ship_head,
                systems,
                body,
                ai,
                extended: None,
                teleport_target_room: None,
            });
            nearby_shape = Some((kinds, shape));
        }
        if !ae {
            let flagship = WireFlagshipClassic::read_le(r)?;
            return Ok(Some((
                WireSituation {
                    ae: None,
                    nearby,
                    environment: None,
                    projectiles: vec![],
                    player_extended: None,
                    tail: WireSituationTail::Classic(flagship),
                },
                nearby_shape,
            )));
        }
        let environment = WireEnvironment::read_le(r)?;
        let count = u32::read_le(r)?;
        let projectiles = (0..count)
            .map(|_| WireProjectile::read_le_args(r, (format,)))
            .collect::<BinResult<Vec<_>>>()?;
        let Some(player_shape) = &facts.player else {
            return Ok(None);
        };
        let Some(player_extended) = read_extended(r, format, player_shape, facts.drone_kind)?
        else {
            return Ok(None);
        };
        if let (Some(n), Some((kinds, _))) = (nearby.as_mut(), nearby_shape.as_ref()) {
            let Some(shape) = ExtendedShape::of(&n.systems, kinds, &n.body, facts.drone_kind)
            else {
                return Ok(None);
            };
            let Some(e) = read_extended(r, format, &shape, facts.drone_kind)? else {
                return Ok(None);
            };
            n.extended = Some(e);
        }
        let player_teleport_target_room = i32::read_le(r)?;
        if let Some(n) = nearby.as_mut() {
            n.teleport_target_room = Some(i32::read_le(r)?);
        }
        let autofire = Bool32::read_le(r)?;
        let flagship = WireFlagshipAe::read_le(r)?;
        Ok(Some((
            WireSituation {
                ae: head,
                nearby,
                environment: Some(environment),
                projectiles,
                player_extended: Some(player_extended),
                tail: WireSituationTail::Ae {
                    player_teleport_target_room,
                    autofire,
                    flagship,
                },
            },
            nearby_shape,
        )))
    }

    /// Writes the fields in the order [`WireSituation::read`] reads them.
    pub fn write<W: Write + Seek>(&self, w: &mut W, format: i32) -> BinResult<()> {
        if let Some(a) = &self.ae {
            a.write_le_args(w, (format,))?;
        }
        Bool32(self.nearby.is_some()).write_le(w)?;
        if let Some(n) = &self.nearby {
            n.rebel_flagship_nearby.write_le(w)?;
            n.head.write_le_args(w, (format,))?;
            n.systems.write_le_args(w, (format,))?;
            n.body.write_le_args(w, (format,))?;
            n.ai.write_le(w)?;
        }
        match &self.tail {
            WireSituationTail::Classic(f) => f.write_le(w),
            WireSituationTail::Ae {
                player_teleport_target_room,
                autofire,
                flagship,
            } => {
                self.environment.write_le(w)?;
                (self.projectiles.len() as u32).write_le(w)?;
                for p in &self.projectiles {
                    p.write_le_args(w, (format,))?;
                }
                if let Some(e) = &self.player_extended {
                    e.write_le_args(w, (format,))?;
                }
                if let Some(e) = self.nearby.as_ref().and_then(|n| n.extended.as_ref()) {
                    e.write_le_args(w, (format,))?;
                }
                player_teleport_target_room.write_le(w)?;
                if let Some(xi) = self.nearby.as_ref().and_then(|n| n.teleport_target_room) {
                    xi.write_le(w)?;
                }
                autofire.write_le(w)?;
                flagship.write_le(w)
            }
        }
    }
}

/// Format 7 and later: the scalars before the nearby ship flag.
#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireSituationAe {
    pub ship_event_present: i32,
    #[brw(args(format))]
    pub encounter: WireEncounter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireNearbyShip {
    /// Format 7 and later.
    pub rebel_flagship_nearby: Option<Bool32>,
    pub head: WireShipHead,
    pub systems: WireShipSystems,
    pub body: WireShipBody,
    /// Format 7 and later.
    pub ai: Option<WireNearbyShipAi>,
    /// Format 7 and later.
    pub extended: Option<WireExtendedShipInfo>,
    /// Format 7 and later. Java: xi.
    pub teleport_target_room: Option<i32>,
}

/// The fields after the extended infos (format 7+) or after the nearby
/// ship (format 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireSituationTail {
    Ae {
        player_teleport_target_room: i32,
        autofire: Bool32,
        flagship: WireFlagshipAe,
    },
    Classic(WireFlagshipClassic),
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireEncounter {
    pub ship_event_seed: i32,
    #[brw(args(format >= 11))]
    pub surrender_event_id: FtlString,
    #[brw(args(format >= 11))]
    pub escape_event_id: FtlString,
    #[brw(args(format >= 11))]
    pub destroyed_event_id: FtlString,
    #[brw(args(format >= 11))]
    pub dead_crew_event_id: FtlString,
    #[brw(args(format >= 11))]
    pub got_away_event_id: FtlString,
    #[brw(args(format >= 11))]
    pub last_event_id: FtlString,
    /// Format 11 only.
    #[brw(if(format == 11))]
    pub text_is_literal: Option<i32>,
    #[brw(args(format >= 11))]
    pub text: FtlString,
    pub affected_crew_seed: i32,
    #[br(temp)]
    #[bw(calc = choices.len() as u32)]
    choice_count: u32,
    #[br(count = choice_count)]
    pub choices: Vec<i32>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireNearbyShipAi {
    pub surrendered: Bool32,
    pub escaping: Bool32,
    pub destroyed: Bool32,
    pub surrender_threshold: i32,
    pub escape_threshold: i32,
    pub escape_ticks: i32,
    pub stalemate_triggered: Bool32,
    pub stalemate_ticks: i32,
    pub boarding_attempts: i32,
    pub boarders_needed: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireEnvironment {
    pub red_giant_present: Bool32,
    pub pulsar_present: Bool32,
    pub pds_present: Bool32,
    /// 0 player ship, 1 nearby ship, 2 both ships.
    #[br(assert((0..=2).contains(&vulnerable_ships), "environment vulnerability flag {} is outside 0..=2", vulnerable_ships))]
    pub vulnerable_ships: i32,
    pub asteroids_present: Bool32,
    #[br(if(asteroids_present.0))]
    pub asteroid_field: Option<WireAsteroidField>,
    pub solar_flare_fade_ticks: i32,
    pub havoc_ticks: i32,
    pub pds_ticks: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireAsteroidField {
    pub init_shields: i32,
    pub stray_rock_ticks: i32,
    pub state: i32,
    pub bg_drift_ticks: i32,
    pub current_target: i32,
}

/// Format 7 and later: the flagship at the end of the file.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireFlagshipAe {
    pub current_stage: i32,
    pub pending_stage: i32,
    pub power_ticks_goal: i32,
    pub power_ticks: i32,
    #[br(temp)]
    #[bw(calc = previous_occupancy.len() as u32)]
    occupancy_count: u32,
    #[br(count = occupancy_count)]
    pub previous_occupancy: Vec<i32>,
}

/// Format 2 (`readRebelFlagship`).
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireFlagshipClassic {
    pub pending_stage: i32,
    #[br(temp)]
    #[bw(calc = previous_occupancy.len() as u32)]
    occupancy_count: u32,
    #[br(count = occupancy_count)]
    pub previous_occupancy: Vec<i32>,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite};

    use super::*;

    fn int(v: i32) -> Vec<u8> {
        v.to_le_bytes().to_vec()
    }

    /// Not seen in a real file: the fixture has no asteroid field.
    #[test]
    fn an_environment_with_an_asteroid_field_round_trips() {
        let bytes = [
            int(0),
            int(0),
            int(0),
            int(2),
            int(1),
            int(10),
            int(11),
            int(12),
            int(13),
            int(1),
            int(5),
            int(6),
            int(7),
        ]
        .concat();
        let env = WireEnvironment::read(&mut Cursor::new(&bytes)).unwrap();
        let field = env.asteroid_field.as_ref().unwrap();
        assert_eq!((field.stray_rock_ticks, field.current_target), (11, 1));
        assert_eq!((env.vulnerable_ships, env.pds_ticks), (2, 7));
        let mut out = Cursor::new(Vec::new());
        env.write(&mut out).unwrap();
        assert_eq!(out.into_inner(), bytes);
    }

    /// Not seen in a real file: the fixture's flagship has no occupancy.
    #[test]
    fn a_flagship_occupancy_round_trips_in_both_forms() {
        let ae = [int(0), int(2), int(30000), int(0), int(2), int(3), int(1)].concat();
        let f = WireFlagshipAe::read(&mut Cursor::new(&ae)).unwrap();
        assert_eq!(
            (f.pending_stage, f.previous_occupancy.clone()),
            (2, vec![3, 1])
        );
        let mut out = Cursor::new(Vec::new());
        f.write(&mut out).unwrap();
        assert_eq!(out.into_inner(), ae);
        let classic = [int(1), int(1), int(4)].concat();
        let f = WireFlagshipClassic::read(&mut Cursor::new(&classic)).unwrap();
        assert_eq!(f.previous_occupancy, [4]);
    }
}
