//! Everything after the sector map (`readSavedGame` from the encounter on):
//! the encounter, the nearby ship with its AI, the environment, the
//! projectiles in flight and the rebel flagship.
//!
//! The codec decodes it as one piece, and only when it knows every ship
//! layout and drone kind it needs: the nearby ship's shape and the drone
//! kind of every drone with a drone pod. Otherwise the bytes stay in
//! [`crate::SavedGame::undecoded`].

use serde::{Deserialize, Serialize};

use crate::model::{Projectile, Ship};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Situation {
    /// Format 7 and later. `ShipEvent::present` (`ShipEvent::Save`, FTL 1.6.13
    /// macOS 0x1001bc120): the beacon has a ship event. Java: mu.
    pub ship_event_present: i32,
    /// Format 7 and later.
    pub encounter: Option<Encounter>,
    pub nearby_ship: Option<NearbyShip>,
    /// Format 7 and later.
    pub environment: Option<Environment>,
    /// Projectiles in flight between the ships. Format 7 and later.
    pub projectiles: Vec<Projectile>,
    /// Format 7 and later. `CompleteShip::teleTargetRoom` of the player ship
    /// (+0x1e8, `CompleteShip::SaveState` 0x100008860; set by
    /// `CompleteShip::InitiateTeleport`): the room the teleporter sends to.
    /// Java: nu.
    pub player_teleport_target_room: i32,
    /// Format 7 and later: the autofire toggle of the player.
    pub autofire: bool,
    pub rebel_flagship: RebelFlagship,
}

/// The ship at the current beacon (`readShip` with auto = true).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearbyShip {
    /// Format 7 and later: the nearby ship is the rebel flagship.
    pub rebel_flagship_nearby: bool,
    /// Its body is always decoded: the codec decodes a nearby ship only
    /// with its layout. Its extended info is set in format 7 and later.
    pub ship: Ship,
    /// Format 7 and later.
    pub ai: Option<NearbyShipAi>,
    /// Format 7 and later. `CompleteShip::teleTargetRoom` of the nearby ship,
    /// written after the player's. Java: xi.
    pub teleport_target_room: i32,
}

/// The event state at the current beacon (`readEncounter`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encounter {
    pub ship_event_seed: i32,
    pub surrender_event_id: String,
    pub escape_event_id: String,
    pub destroyed_event_id: String,
    pub dead_crew_event_id: String,
    pub got_away_event_id: String,
    pub last_event_id: String,
    /// Format 11 only. `TextString::isLiteral` of the event text
    /// (`WorldManager::SaveGame`, 0x1001e6500, writes it before the text):
    /// 1 when `text` is the text itself, 0 when it is a text id. Java: alpha.
    pub text_is_literal: i32,
    /// Text id of the event text, for example `event_FLEET_EASY_DLC_text`.
    pub text: String,
    pub affected_crew_seed: i32,
    /// Indexes of the choices the player took in the last event.
    pub choices: Vec<i32>,
}

/// The behaviour of the nearby ship (`readNearbyShipAI`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearbyShipAi {
    pub surrendered: bool,
    pub escaping: bool,
    /// `ShipAI::destroyed` (`ShipAI::Save`, 0x10017cfa0).
    pub destroyed: bool,
    /// Hull at which the ship offers surrender.
    pub surrender_threshold: i32,
    /// Hull at which the ship starts to escape.
    pub escape_threshold: i32,
    /// Counts down to the escape jump.
    pub escape_ticks: i32,
    /// `ShipAI::bStalemateTrigger`.
    pub stalemate_triggered: bool,
    /// `ShipAI::fStalemateTimer`, times 1000.
    pub stalemate_ticks: i32,
    /// `ShipAI::iTimesTeleported`.
    pub boarding_attempts: i32,
    /// `ShipAI::iCrewNeeded`.
    pub boarders_needed: i32,
}

/// Hazards at the current beacon (`readEnvironment`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    pub red_giant_present: bool,
    pub pulsar_present: bool,
    pub pds_present: bool,
    /// Which ships the planetary defense system targets.
    pub vulnerable_ships: HazardVulnerability,
    pub asteroid_field: Option<AsteroidField>,
    /// Progress of `SpaceManager::flashTracker` (the solar flare flash),
    /// times 1000 (`SpaceManager::SaveSpace`, 0x10003c8a0).
    pub solar_flare_fade_ticks: i32,
    /// `SpaceManager::flashTimer` time, times 1000: the red giant, pulsar or
    /// PDS cycle.
    pub havoc_ticks: i32,
    /// `SpaceManager::randomPDSTimer`, times 1000.
    pub pds_ticks: i32,
}

/// Stored as 0 to 2 in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardVulnerability {
    PlayerShip,
    NearbyShip,
    BothShips,
}

impl HazardVulnerability {
    pub const ALL: [HazardVulnerability; 3] = [
        HazardVulnerability::PlayerShip,
        HazardVulnerability::NearbyShip,
        HazardVulnerability::BothShips,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsteroidField {
    /// `AsteroidGenerator::initShields` (`SpaceManager::SaveSpace`,
    /// 0x10003c8a0): the value `AsteroidGenerator::Initialize` got, from
    /// which it takes the intensity of the field (half of it, 1 to 3).
    /// Java: alpha.
    pub init_shields: i32,
    pub stray_rock_ticks: i32,
    /// `AsteroidGenerator::iState`: 0 calm, 1 or 2 a volley
    /// (`AsteroidGenerator::UpdateState`); `stray_rock_ticks` is the time
    /// left in it (`fStateTimer`). Java: gamma.
    pub state: i32,
    pub bg_drift_ticks: i32,
    /// `AsteroidGenerator::currentSpace`: the ship the next rock flies at.
    pub current_target: i32,
}

/// The rebel flagship between its fights.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebelFlagship {
    /// Format 7 and later. `BossShip::currentStage` (+0x1ec,
    /// `BossShip::SaveBoss` 0x100001b00): the stage being fought, 1 to 3.
    /// Java: alpha.
    pub current_stage: i32,
    /// The next flagship stage the player meets (1 to 3).
    pub pending_stage: i32,
    /// Format 7 and later. Goal of `BossShip::powerTimer` (+0x1fc), times
    /// 1000: when it runs out, stage 2 launches its drone surge and stage 3
    /// its barrage or super shield (`BossShip::OnLoop`). Java: gamma.
    pub power_ticks_goal: i32,
    /// Format 7 and later. Time on `BossShip::powerTimer` (+0x1f8), times
    /// 1000. Java: delta.
    pub power_ticks: i32,
    /// Crew per room of the flagship as last seen, by room id.
    pub previous_occupancy: Vec<i32>,
}
