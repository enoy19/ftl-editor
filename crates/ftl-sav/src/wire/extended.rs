//! The extended ship info (`readExtendedShipInfo`, `readDronePod`,
//! `readWeaponModule`).
//!
//! The file stores no count for the drone entries, no flag for the hacking,
//! mind and weapon parts, and no drone kind: the caller derives them from
//! the ship ([`ExtendedShape`]) and asks for the kind of a standalone drone
//! by its id. So [`read_extended`] is a function; writing needs nothing of
//! that and derives from the struct.

use std::io::{Read, Seek};

use binrw::{BinRead, BinResult, binrw, binwrite};
use ftl_core::DroneKind;

use super::{
    ARTILLERY, Bool32, FtlString, HACKING, MIND, WEAPONS, WireAnim, WireProjectile, WireShipBody,
    WireShipSystems, installed,
};

/// What the ship tells about the layout of its extended info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedShape {
    /// The kind of each drone of the body, in order.
    pub drone_kinds: Vec<DroneKind>,
    pub hacking: bool,
    pub mind: bool,
    /// The weapon count when the weapons system is installed.
    pub weapons: Option<usize>,
    /// Installed artillery slots.
    pub artillery: usize,
}

impl ExtendedShape {
    /// The extended shape of a ship from its systems and body. `None` when
    /// `drone_kind` does not know a drone of the body.
    pub fn of(
        systems: &WireShipSystems,
        kinds: &[usize],
        body: &WireShipBody,
        drone_kind: &dyn Fn(&str) -> Option<DroneKind>,
    ) -> Option<ExtendedShape> {
        let drone_kinds = body
            .drones
            .iter()
            .map(|d| drone_kind(&d.id.0))
            .collect::<Option<Vec<_>>>()?;
        let s = &systems.systems;
        Some(ExtendedShape {
            drone_kinds,
            hacking: installed(s, kinds, HACKING),
            mind: installed(s, kinds, MIND),
            weapons: installed(s, kinds, WEAPONS).then_some(body.weapons.len()),
            artillery: kinds
                .iter()
                .zip(s)
                .filter(|(k, sys)| **k == ARTILLERY && sys.capacity > 0)
                .count(),
        })
    }
}

#[binwrite]
#[bw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireExtendedShipInfo {
    pub drones: Vec<WireExtendedDrone>,
    pub hacking: Option<WireHackingInfo>,
    pub mind: Option<WireMindInfo>,
    #[bw(args(format))]
    pub weapon_modules: Option<WireWeaponModules>,
    #[bw(args(format))]
    pub artillery: Vec<WireWeaponModule>,
    #[bw(calc = standalone_drones.len() as u32)]
    standalone_count: u32,
    #[bw(args(format >= 11))]
    pub standalone_drones: Vec<WireStandaloneDrone>,
}

/// Reads the extended info of a ship. `Ok(None)` when `drone_kind` does not
/// know a standalone drone: the caller then keeps the bytes undecoded.
pub fn read_extended<R: Read + Seek>(
    r: &mut R,
    format: i32,
    shape: &ExtendedShape,
    drone_kind: &dyn Fn(&str) -> Option<DroneKind>,
) -> BinResult<Option<WireExtendedShipInfo>> {
    let drones = shape
        .drone_kinds
        .iter()
        .map(|&kind| WireExtendedDrone::read_le_args(r, (kind,)))
        .collect::<BinResult<Vec<_>>>()?;
    let hacking = shape
        .hacking
        .then(|| WireHackingInfo::read_le(r))
        .transpose()?;
    let mind = shape.mind.then(|| WireMindInfo::read_le(r)).transpose()?;
    let weapon_modules = match shape.weapons {
        Some(n) => {
            let pos = r.stream_position()?;
            let m = WireWeaponModules::read_le_args(r, (format,))?;
            if m.modules.len() != n {
                return Err(binrw::Error::AssertFail {
                    pos,
                    message: format!(
                        "found {} weapon modules, but the ship has {n} weapons",
                        m.modules.len()
                    ),
                });
            }
            Some(m)
        }
        None => None,
    };
    let artillery = (0..shape.artillery)
        .map(|_| WireWeaponModule::read_le_args(r, (format,)))
        .collect::<BinResult<Vec<_>>>()?;
    let count = u32::read_le(r)?;
    let mut standalone_drones = Vec::new();
    for _ in 0..count {
        let drone_id = FtlString::read_le_args(r, (format >= 11,))?;
        let Some(kind) = drone_kind(&drone_id.0) else {
            return Ok(None);
        };
        let drone_pod = WireDronePod::read_le_args(r, (kind,))?;
        let [deployed, destroyed, armed] = <[i32; 3]>::read_le(r)?;
        standalone_drones.push(WireStandaloneDrone {
            drone_id,
            drone_pod,
            deployed,
            destroyed,
            armed,
        });
    }
    Ok(Some(WireExtendedShipInfo {
        drones,
        hacking,
        mind,
        weapon_modules,
        artillery,
        standalone_drones,
    }))
}

#[binrw]
#[br(little, import(kind: DroneKind))]
#[bw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireExtendedDrone {
    pub deployed: Bool32,
    pub armed: Bool32,
    #[br(if(kind.has_pod()), args(kind))]
    pub pod: Option<WireDronePod>,
}

#[binrw]
#[br(little, import(kind: DroneKind))]
#[bw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireDronePod {
    /// Not on disk: the kind the reader was given.
    #[br(calc = kind)]
    #[bw(ignore)]
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
    pub point_target_x: i32,
    pub point_target_y: i32,
    pub next_target_x: i32,
    pub next_target_y: i32,
    pub target_speed_x: i32,
    pub target_speed_y: i32,
    pub buildup_ticks: i32,
    pub stationary_ticks: i32,
    pub cooldown_ticks: i32,
    pub orbit_angle: i32,
    pub turret_angle: i32,
    pub last_aiming_angle: i32,
    pub hops_to_live: i32,
    pub disrupted: i32,
    pub powered_last_frame: i32,
    pub overload_ticks: i32,
    pub desired_aiming_angle: i32,
    pub deployed_last_frame: i32,
    pub delta_position_x: i32,
    pub delta_position_y: i32,
    pub death_anim: WireAnim,
    #[br(args(kind))]
    pub info: WireDronePodInfo,
}

/// The kind-dependent end of a drone pod.
#[binrw]
#[br(little, import(kind: DroneKind))]
#[bw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireDronePodInfo {
    #[br(pre_assert(kind == DroneKind::Boarder))]
    Boarder {
        died_in_space: i32,
        delivered_drone: i32,
        starting_position_x: i32,
        starting_position_y: i32,
        body_health: i32,
        body_x: i32,
        body_y: i32,
        body_room_id: i32,
        body_room_square: i32,
    },
    #[br(pre_assert(kind == DroneKind::Hacking))]
    Hacking {
        attach_position_x: i32,
        attach_position_y: i32,
        arrived: i32,
        finished_setup: i32,
        landing_anim: WireAnim,
        extension_anim: WireAnim,
    },
    #[br(pre_assert(matches!(kind, DroneKind::Combat | DroneKind::Beam | DroneKind::ShipRepair)))]
    ZigZag {
        last_waypoint_x: i32,
        last_waypoint_y: i32,
        transit_ticks: i32,
        exhaust_angle: i32,
        previous_exhaust_angle: i32,
    },
    #[br(pre_assert(kind == DroneKind::Defense))]
    Empty,
    #[br(pre_assert(kind == DroneKind::Shield))]
    Shield { glow_animation: i32 },
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireHackingInfo {
    /// -1 none, else the system index (`SystemKind::ALL`).
    #[br(assert((-1..=15).contains(&target_system), "hacking target system flag {} is outside -1..=15", target_system))]
    pub target_system: i32,
    pub drone_powered: i32,
    pub drone_pod_visible: Bool32,
    pub hacking: i32,
    pub disruption_ticks: i32,
    pub disruption_ticks_goal: i32,
    pub disrupting: Bool32,
    #[br(args(DroneKind::Hacking))]
    pub drone_pod: WireDronePod,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireMindInfo {
    pub mind_control_ticks: i32,
    pub mind_control_ticks_goal: i32,
}

/// The weapon modules with their count.
#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireWeaponModules {
    #[br(temp)]
    #[bw(calc = modules.len() as u32)]
    count: u32,
    #[br(count = count, args { inner: (format,) })]
    #[bw(args(format))]
    pub modules: Vec<WireWeaponModule>,
}

#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireWeaponModule {
    pub cooldown_ticks: i32,
    pub cooldown_ticks_goal: i32,
    pub subcooldown_ticks: i32,
    pub subcooldown_ticks_goal: i32,
    pub boost: i32,
    pub charge: i32,
    #[br(temp)]
    #[bw(calc = current_targets.len() as u32)]
    current_count: u32,
    #[br(count = current_count)]
    pub current_targets: Vec<[i32; 2]>,
    #[br(temp)]
    #[bw(calc = previous_targets.len() as u32)]
    previous_count: u32,
    #[br(count = previous_count)]
    pub previous_targets: Vec<[i32; 2]>,
    pub autofire: Bool32,
    pub fire_when_ready: Bool32,
    pub target_id: i32,
    pub weapon_anim: WireAnim,
    pub protract_anim_ticks: i32,
    pub firing: Bool32,
    pub fire_shot: Bool32,
    /// Format 9 and later: anim charge and charge anim.
    #[brw(if(format >= 9))]
    pub charge_anim: Option<(i32, WireAnim)>,
    pub last_projectile_id: i32,
    #[br(temp)]
    #[bw(calc = pending_projectiles.len() as u32)]
    pending_count: u32,
    #[br(count = pending_count, args { inner: (format,) })]
    #[bw(args(format))]
    pub pending_projectiles: Vec<WireProjectile>,
}

#[binrw]
#[br(little, import(utf8: bool, kind: DroneKind))]
#[bw(little, import(utf8: bool))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireStandaloneDrone {
    #[brw(args(utf8))]
    pub drone_id: FtlString,
    #[br(args(kind))]
    pub drone_pod: WireDronePod,
    /// `Drone::GetDeployed`, `Drone::Destroyed` and `Drone::GetPowered`, in
    /// this order (`ShipManager::SaveSuperDrones`). Java: alpha to gamma.
    pub deployed: i32,
    pub destroyed: i32,
    pub armed: i32,
}
