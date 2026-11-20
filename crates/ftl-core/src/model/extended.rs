//! The extended ship info (`readExtendedShipInfo`): drone pods, hacking,
//! mind control, weapon modules, artillery and standalone drones.
//!
//! FTL stores it after the projectiles, for the player ship and then for
//! the nearby ship, not with the ship itself.

use serde::{Deserialize, Serialize};

use crate::model::{Anim, Projectile, SystemKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtendedShipInfo {
    /// One per drone of the ship body, in the same order.
    pub drones: Vec<ExtendedDroneInfo>,
    /// When the hacking system is installed.
    pub hacking: Option<HackingInfo>,
    /// When the mind control system is installed.
    pub mind: Option<MindInfo>,
    /// When the weapons system is installed: one per weapon of the body.
    pub weapon_modules: Option<Vec<WeaponModule>>,
    /// One per installed artillery slot (the flagship has several).
    pub artillery: Vec<WeaponModule>,
    /// Drones without a drone slot: the flagship's surge drones.
    pub standalone_drones: Vec<StandaloneDrone>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtendedDroneInfo {
    pub deployed: bool,
    pub armed: bool,
    /// Every drone kind but repair and battle (anti-personnel) has a pod.
    pub pod: Option<DronePod>,
}

/// The kind of a drone, from its blueprint (`type` in the drone XML). It
/// decides which pod fields the file stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DroneKind {
    Battle,
    Repair,
    Boarder,
    Hacking,
    Combat,
    Beam,
    Defense,
    Shield,
    ShipRepair,
}

impl DroneKind {
    pub const ALL: [DroneKind; 9] = [
        DroneKind::Battle,
        DroneKind::Repair,
        DroneKind::Boarder,
        DroneKind::Hacking,
        DroneKind::Combat,
        DroneKind::Beam,
        DroneKind::Defense,
        DroneKind::Shield,
        DroneKind::ShipRepair,
    ];

    /// The `type` value of the drone blueprint.
    pub fn id(self) -> &'static str {
        match self {
            DroneKind::Battle => "BATTLE",
            DroneKind::Repair => "REPAIR",
            DroneKind::Boarder => "BOARDER",
            DroneKind::Hacking => "HACKING",
            DroneKind::Combat => "COMBAT",
            DroneKind::Beam => "BEAM",
            DroneKind::Defense => "DEFENSE",
            DroneKind::Shield => "SHIELD",
            DroneKind::ShipRepair => "SHIP_REPAIR",
        }
    }

    pub fn from_id(id: &str) -> Option<DroneKind> {
        DroneKind::ALL.into_iter().find(|k| k.id() == id)
    }

    /// Whether a deployed drone of this kind has a pod in the file.
    pub fn has_pod(self) -> bool {
        !matches!(self, DroneKind::Repair | DroneKind::Battle)
    }
}

/// A drone body outside the ship (`readDronePod`), as `SpaceDrone::SaveState`
/// of FTL 1.6.13 (macOS 0x100035640) writes it. Floats are stored times 1000
/// (`FileHelper::writeFloat`). The names Java leaves as Greek letters come
/// from the `SpaceDrone` struct of the FTL: Hyperspace headers
/// (`FTLGameStripped.h`, GPL-3.0-or-later), matched by the offsets the binary
/// writes; `docs/unknown-fields.md` lists the evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DronePod {
    /// Not stored: the drone blueprint names it.
    pub kind: DroneKind,
    pub mourning_ticks: i32,
    pub current_space: i32,
    pub destination_space: i32,
    pub current_position_x: i32,
    pub current_position_y: i32,
    pub previous_position_x: i32,
    pub previous_position_y: i32,
    pub goal_position_x: i32,
    pub goal_position_y: i32,
    /// `SpaceDrone::pointTarget` (+0x78), the point the drone aims at.
    /// Java: epsilon and zeta.
    pub point_target_x: i32,
    pub point_target_y: i32,
    pub next_target_x: i32,
    pub next_target_y: i32,
    /// `SpaceDrone::targetSpeed` (+0x180), the speed of the target, for
    /// aiming ahead. Java: iota and kappa.
    pub target_speed_x: i32,
    pub target_speed_y: i32,
    pub buildup_ticks: i32,
    pub stationary_ticks: i32,
    pub cooldown_ticks: i32,
    pub orbit_angle: i32,
    pub turret_angle: i32,
    /// `SpaceDrone::lastAimingAngle` (+0x1b0). Java: xi.
    pub last_aiming_angle: i32,
    pub hops_to_live: i32,
    /// `SpaceDrone::bDisrupted` (+0x2bd): the drone is scrambled. Java: pi.
    pub disrupted: i32,
    /// `SpaceDrone::poweredLastFrame` (+0x198). Java: rho.
    pub powered_last_frame: i32,
    pub overload_ticks: i32,
    /// `SpaceDrone::desiredAimingAngle` (+0x1b4). Java: tau.
    pub desired_aiming_angle: i32,
    /// `SpaceDrone::deployedLastFrame` (+0x199). Java: upsilon.
    pub deployed_last_frame: i32,
    pub delta_position_x: i32,
    pub delta_position_y: i32,
    pub death_anim: Anim,
    /// The part that depends on the kind.
    pub info: DronePodInfo,
}

/// The kind-dependent end of a drone pod.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DronePodInfo {
    /// Boarding drone (`BoarderPodDrone::SaveState`, 0x100037b40).
    Boarder {
        /// `diedInSpace` (+0x539): shot down before it reached the hull.
        /// Java: alpha.
        died_in_space: i32,
        /// `bDeliveredDrone` (+0x538): the boarder is on the enemy ship.
        /// Java: beta.
        delivered_drone: i32,
        /// `startingPosition` (+0x3e0), times 1000. Java: gamma and delta.
        starting_position_x: i32,
        starting_position_y: i32,
        body_health: i32,
        body_x: i32,
        body_y: i32,
        body_room_id: i32,
        body_room_square: i32,
    },
    /// Hacking drone.
    Hacking {
        attach_position_x: i32,
        attach_position_y: i32,
        /// `HackingDrone::arrived` (+0x3f8): it reached the hull. Java: gamma.
        arrived: i32,
        /// `HackingDrone::finishedSetup` (+0x3f9): it is attached and
        /// extended. Java: delta.
        finished_setup: i32,
        landing_anim: Anim,
        extension_anim: Anim,
    },
    /// Combat, beam and hull repair drones fly a zigzag.
    ZigZag {
        last_waypoint_x: i32,
        last_waypoint_y: i32,
        transit_ticks: i32,
        exhaust_angle: i32,
        /// `CombatDrone::oldHeading` (+0x3e0); `exhaust_angle` is `heading`.
        /// Java: epsilon.
        previous_exhaust_angle: i32,
    },
    /// Defense drone: no more fields.
    Empty,
    /// Shield drone.
    Shield {
        /// `SuperShieldDrone::glowAnimation` (+0x6b0), times 1000. Java: alpha.
        glow_animation: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HackingInfo {
    /// The system the hacking drone sits on; `None` before it lands.
    pub target_system: Option<SystemKind>,
    /// The hacking drone is powered (`Drone::powered`, drone +0x10;
    /// `HackingSystem::SaveState`, 0x1001ed6a0). Java: beta.
    pub drone_powered: i32,
    /// The hacking drone is deployed (`Drone::deployed`, drone +0x18).
    pub drone_pod_visible: bool,
    /// `HackingSystem::bHacking` (+0x264): the system hacks its target now.
    /// Java: delta.
    pub hacking: i32,
    pub disruption_ticks: i32,
    pub disruption_ticks_goal: i32,
    pub disrupting: bool,
    /// The hacking drone.
    pub drone_pod: DronePod,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MindInfo {
    pub mind_control_ticks: i32,
    pub mind_control_ticks_goal: i32,
}

/// The charge state of a weapon (`readWeaponModule`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponModule {
    pub cooldown_ticks: i32,
    pub cooldown_ticks_goal: i32,
    pub subcooldown_ticks: i32,
    pub subcooldown_ticks_goal: i32,
    pub boost: i32,
    pub charge: i32,
    pub current_targets: Vec<Reticle>,
    pub previous_targets: Vec<Reticle>,
    pub autofire: bool,
    pub fire_when_ready: bool,
    pub target_id: i32,
    pub weapon_anim: Anim,
    pub protract_anim_ticks: i32,
    pub firing: bool,
    /// `WeaponAnimation::bFireShot` (+0xf0; `firing` is `bFiring`, +0xf1),
    /// `WeaponAnimation::SaveState`, 0x10004ca30. Java: phi.
    pub fire_shot: bool,
    /// Format 9 and later. `WeaponAnimation::boostLevel` (+0x3a0).
    pub anim_charge: i32,
    /// Format 9 and later.
    pub charge_anim: Anim,
    pub last_projectile_id: i32,
    /// Projectiles queued before firing.
    pub pending_projectiles: Vec<Projectile>,
}

/// A target reticle on the enemy ship, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reticle {
    pub x: i32,
    pub y: i32,
}

/// A drone of the rebel flagship's drone surge (stage 2), outside any drone
/// system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandaloneDrone {
    pub drone_id: String,
    pub drone_pod: DronePod,
    /// `Drone::GetDeployed` (`ShipManager::SaveSuperDrones`, 0x100094250).
    /// Java: alpha.
    pub deployed: i32,
    /// `Drone::Destroyed`. Java: beta.
    pub destroyed: i32,
    /// `Drone::GetPowered`. Java: gamma.
    pub armed: i32,
}
