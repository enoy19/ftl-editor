//! After the sector map: wire to domain and back.

use ftl_core::{
    AsteroidField, DecodeError, Encounter, Environment, ExtendedShipInfo, FileFormat,
    HazardVulnerability, NearbyShip, NearbyShipAi, RebelFlagship, Situation,
};

use super::{extended, projectile, ship_to_domain, ship_to_wire};
use crate::wire::{
    Bool32, FtlString, ShipFacts, WireAsteroidField, WireEncounter, WireEnvironment,
    WireFlagshipAe, WireFlagshipClassic, WireNearbyShip, WireNearbyShipAi, WireSituation,
    WireSituationAe, WireSituationTail,
};

/// The situation and the extended info of the player ship, which the file
/// stores inside it. `nearby_shape` is what the nearby ship was read with.
pub fn to_domain(
    w: WireSituation,
    nearby_shape: Option<ShipFacts>,
) -> Result<(Situation, Option<ExtendedShipInfo>), DecodeError> {
    let nearby_ship = match (w.nearby, nearby_shape) {
        (Some(n), Some((kinds, shape))) => {
            let mut ship = ship_to_domain(n.head, n.systems, &kinds, Some((n.body, shape)))?;
            ship.extended = n.extended.map(extended::to_domain).transpose()?;
            Some(NearbyShip {
                rebel_flagship_nearby: n.rebel_flagship_nearby.is_some_and(|b| b.0),
                ship,
                ai: n.ai.map(ai_to_domain),
                teleport_target_room: n.teleport_target_room.unwrap_or(0),
            })
        }
        _ => None,
    };
    let (player_teleport_target_room, autofire, rebel_flagship) = match w.tail {
        WireSituationTail::Ae {
            player_teleport_target_room,
            autofire,
            flagship,
        } => (
            player_teleport_target_room,
            autofire.0,
            RebelFlagship {
                current_stage: flagship.current_stage,
                pending_stage: flagship.pending_stage,
                power_ticks_goal: flagship.power_ticks_goal,
                power_ticks: flagship.power_ticks,
                previous_occupancy: flagship.previous_occupancy,
            },
        ),
        WireSituationTail::Classic(f) => (
            0,
            false,
            RebelFlagship {
                current_stage: 0,
                pending_stage: f.pending_stage,
                power_ticks_goal: 0,
                power_ticks: 0,
                previous_occupancy: f.previous_occupancy,
            },
        ),
    };
    let (ship_event_present, encounter) = match w.ae {
        Some(a) => (a.ship_event_present, Some(encounter_to_domain(a.encounter))),
        None => (0, None),
    };
    let situation = Situation {
        ship_event_present,
        encounter,
        nearby_ship,
        environment: w.environment.map(environment_to_domain),
        projectiles: w
            .projectiles
            .into_iter()
            .map(projectile::to_domain)
            .collect(),
        player_teleport_target_room,
        autofire,
        rebel_flagship,
    };
    let player = w.player_extended.map(extended::to_domain).transpose()?;
    Ok((situation, player))
}

pub fn to_wire(
    s: &Situation,
    player_extended: Option<&ExtendedShipInfo>,
    f: FileFormat,
) -> WireSituation {
    let ae = f >= 7;
    let nearby = s.nearby_ship.as_ref().map(|n| {
        let (head, systems, body) = ship_to_wire(&n.ship, f);
        WireNearbyShip {
            rebel_flagship_nearby: ae.then_some(Bool32(n.rebel_flagship_nearby)),
            head,
            systems,
            body: body.expect("a decoded nearby ship has a body"),
            ai: if ae {
                n.ai.as_ref().map(ai_to_wire)
            } else {
                None
            },
            extended: if ae {
                n.ship.extended.as_ref().map(extended::to_wire)
            } else {
                None
            },
            teleport_target_room: ae.then_some(n.teleport_target_room),
        }
    });
    let flag = &s.rebel_flagship;
    WireSituation {
        ae: s
            .encounter
            .as_ref()
            .filter(|_| ae)
            .map(|e| WireSituationAe {
                ship_event_present: s.ship_event_present,
                encounter: encounter_to_wire(e, f),
            }),
        nearby,
        environment: if ae {
            s.environment.as_ref().map(environment_to_wire)
        } else {
            None
        },
        projectiles: s.projectiles.iter().map(projectile::to_wire).collect(),
        player_extended: if ae {
            player_extended.map(extended::to_wire)
        } else {
            None
        },
        tail: if ae {
            WireSituationTail::Ae {
                player_teleport_target_room: s.player_teleport_target_room,
                autofire: Bool32(s.autofire),
                flagship: WireFlagshipAe {
                    current_stage: flag.current_stage,
                    pending_stage: flag.pending_stage,
                    power_ticks_goal: flag.power_ticks_goal,
                    power_ticks: flag.power_ticks,
                    previous_occupancy: flag.previous_occupancy.clone(),
                },
            }
        } else {
            WireSituationTail::Classic(WireFlagshipClassic {
                pending_stage: flag.pending_stage,
                previous_occupancy: flag.previous_occupancy.clone(),
            })
        },
    }
}

fn encounter_to_domain(e: WireEncounter) -> Encounter {
    Encounter {
        ship_event_seed: e.ship_event_seed,
        surrender_event_id: e.surrender_event_id.0,
        escape_event_id: e.escape_event_id.0,
        destroyed_event_id: e.destroyed_event_id.0,
        dead_crew_event_id: e.dead_crew_event_id.0,
        got_away_event_id: e.got_away_event_id.0,
        last_event_id: e.last_event_id.0,
        text_is_literal: e.text_is_literal.unwrap_or(0),
        text: e.text.0,
        affected_crew_seed: e.affected_crew_seed,
        choices: e.choices,
    }
}

fn encounter_to_wire(e: &Encounter, f: FileFormat) -> WireEncounter {
    let s = |v: &str| FtlString(v.to_owned());
    WireEncounter {
        ship_event_seed: e.ship_event_seed,
        surrender_event_id: s(&e.surrender_event_id),
        escape_event_id: s(&e.escape_event_id),
        destroyed_event_id: s(&e.destroyed_event_id),
        dead_crew_event_id: s(&e.dead_crew_event_id),
        got_away_event_id: s(&e.got_away_event_id),
        last_event_id: s(&e.last_event_id),
        text_is_literal: (f == 11).then_some(e.text_is_literal),
        text: s(&e.text),
        affected_crew_seed: e.affected_crew_seed,
        choices: e.choices.clone(),
    }
}

fn ai_to_domain(a: WireNearbyShipAi) -> NearbyShipAi {
    NearbyShipAi {
        surrendered: a.surrendered.0,
        escaping: a.escaping.0,
        destroyed: a.destroyed.0,
        surrender_threshold: a.surrender_threshold,
        escape_threshold: a.escape_threshold,
        escape_ticks: a.escape_ticks,
        stalemate_triggered: a.stalemate_triggered.0,
        stalemate_ticks: a.stalemate_ticks,
        boarding_attempts: a.boarding_attempts,
        boarders_needed: a.boarders_needed,
    }
}

fn ai_to_wire(a: &NearbyShipAi) -> WireNearbyShipAi {
    WireNearbyShipAi {
        surrendered: Bool32(a.surrendered),
        escaping: Bool32(a.escaping),
        destroyed: Bool32(a.destroyed),
        surrender_threshold: a.surrender_threshold,
        escape_threshold: a.escape_threshold,
        escape_ticks: a.escape_ticks,
        stalemate_triggered: Bool32(a.stalemate_triggered),
        stalemate_ticks: a.stalemate_ticks,
        boarding_attempts: a.boarding_attempts,
        boarders_needed: a.boarders_needed,
    }
}

fn environment_to_domain(e: WireEnvironment) -> Environment {
    Environment {
        red_giant_present: e.red_giant_present.0,
        pulsar_present: e.pulsar_present.0,
        pds_present: e.pds_present.0,
        // The reader asserts 0..=2.
        vulnerable_ships: HazardVulnerability::ALL[e.vulnerable_ships as usize],
        asteroid_field: e.asteroid_field.map(|a| AsteroidField {
            init_shields: a.init_shields,
            stray_rock_ticks: a.stray_rock_ticks,
            state: a.state,
            bg_drift_ticks: a.bg_drift_ticks,
            current_target: a.current_target,
        }),
        solar_flare_fade_ticks: e.solar_flare_fade_ticks,
        havoc_ticks: e.havoc_ticks,
        pds_ticks: e.pds_ticks,
    }
}

fn environment_to_wire(e: &Environment) -> WireEnvironment {
    WireEnvironment {
        red_giant_present: Bool32(e.red_giant_present),
        pulsar_present: Bool32(e.pulsar_present),
        pds_present: Bool32(e.pds_present),
        vulnerable_ships: e.vulnerable_ships as i32,
        asteroids_present: Bool32(e.asteroid_field.is_some()),
        asteroid_field: e.asteroid_field.as_ref().map(|a| WireAsteroidField {
            init_shields: a.init_shields,
            stray_rock_ticks: a.stray_rock_ticks,
            state: a.state,
            bg_drift_ticks: a.bg_drift_ticks,
            current_target: a.current_target,
        }),
        solar_flare_fade_ticks: e.solar_flare_fade_ticks,
        havoc_ticks: e.havoc_ticks,
        pds_ticks: e.pds_ticks,
    }
}
