//! The saved game (`continue.sav`) as the editor understands it.
//!
//! The codec decodes the file front to back. Whatever it does not decode yet
//! stays in [`SavedGame::undecoded`] and is written back unchanged, so an edit
//! never loses bytes. Each slice of the rewrite moves fields out of that tail
//! into typed fields.

use serde::{Deserialize, Serialize, Serializer};

mod body;
mod crew;
mod encounter;
mod extended;
mod projectile;
mod systems;
mod world;

pub use body::*;
pub use crew::*;
pub use encounter::*;
pub use extended::*;
pub use projectile::*;
pub use systems::*;
pub use world::*;

/// File format number, the first int of the file.
///
/// | value | FTL version |
/// |---|---|
/// | 2 | 1.01 to 1.03.3 |
/// | 7, 8, 9 | 1.5.4 to 1.5.13 (Advanced Edition) |
/// | 11 | 1.6.1 and later (UTF-8 strings) |
pub type FileFormat = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedGame {
    pub file_format: FileFormat,
    /// Format 11 only: whether the sector map uses the native RNG of the OS.
    pub random_native: bool,
    /// Advanced Edition content; absent (false) in format 2.
    pub dlc_enabled: bool,
    pub difficulty: Difficulty,
    pub stats: SessionStats,
    /// Redundant copy of the player ship name in the file header.
    pub player_ship_name: String,
    /// Redundant copy of the player ship blueprint id in the file header.
    pub player_ship_blueprint_id: String,
    /// One-based sector number, redundant in the file.
    pub sector_number: i32,
    /// `TopScore::victory` of the running score (`ScoreKeeper+0x2e8`, written
    /// by `ScoreKeeper::SaveGame`, FTL 1.6.13 macOS 0x1000298a0): the run is
    /// won. 0 in a running game. Java: beta.
    pub victory: i32,
    pub state_vars: Vec<StateVar>,
    pub player_ship: Ship,
    /// Cargo, sector map, beacons and quests. `None` when the player ship
    /// body is not decoded: these bytes follow it on disk and then stay in
    /// [`SavedGame::undecoded`].
    pub world: Option<World>,
    /// Encounter, nearby ship, environment, projectiles and the rebel
    /// flagship. `None` when the world is not decoded, or when the layout
    /// port does not know the nearby ship or a drone kind the file needs.
    pub situation: Option<Situation>,
    /// Bytes after the last decoded field, kept verbatim.
    #[serde(serialize_with = "len_only", skip_deserializing)]
    pub undecoded: Vec<u8>,
}

/// One of the two ships a saved game holds.
///
/// FTL 1.6.13 loads both with the same code (`WorldManager::LoadGame`: the
/// nearby ship is `new ShipManager(1)`, then `ShipManager::ImportShip` and
/// `ImportBattleState` as for the player ship, id 0), so every edit of a
/// ship applies to either.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShipId {
    #[default]
    Player,
    /// The ship at the current beacon, `nearby.` in an edit path.
    Nearby,
}

impl ShipId {
    /// FTL's `iShipId`: 0 for the player ship, 1 for the nearby ship. A
    /// drone a ship builds starts in that ship's space (`SpaceDrone`).
    pub fn number(self) -> i32 {
        match self {
            ShipId::Player => 0,
            ShipId::Nearby => 1,
        }
    }

    /// What an edit path of this ship starts with: nothing or `nearby.`.
    pub fn prefix(self) -> &'static str {
        match self {
            ShipId::Player => "",
            ShipId::Nearby => "nearby.",
        }
    }

    /// "the player ship" or "the nearby ship", for error texts.
    pub fn name(self) -> &'static str {
        match self {
            ShipId::Player => "the player ship",
            ShipId::Nearby => "the nearby ship",
        }
    }
}

impl SavedGame {
    /// The ship `id`; `None` for the nearby ship when the situation is not
    /// decoded or no ship is at the beacon.
    pub fn ship(&self, id: ShipId) -> Option<&Ship> {
        match id {
            ShipId::Player => Some(&self.player_ship),
            ShipId::Nearby => self
                .situation
                .as_ref()?
                .nearby_ship
                .as_ref()
                .map(|n| &n.ship),
        }
    }

    pub fn ship_mut(&mut self, id: ShipId) -> Option<&mut Ship> {
        match id {
            ShipId::Player => Some(&mut self.player_ship),
            ShipId::Nearby => self
                .situation
                .as_mut()?
                .nearby_ship
                .as_mut()
                .map(|n| &mut n.ship),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionStats {
    pub ships_defeated: i32,
    pub beacons_explored: i32,
    pub scrap_collected: i32,
    pub crew_hired: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateVar {
    pub id: String,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ship {
    pub blueprint_id: String,
    pub name: String,
    pub gfx_base_name: String,
    pub starting_crew: Vec<StartingCrew>,
    /// Format 7 and later. FTL writes `true` for the player ship as well.
    pub hostile: bool,
    pub jump_charge_ticks: i32,
    pub jumping: bool,
    pub jump_anim_ticks: i32,
    pub resources: Resources,
    pub crew: Vec<CrewMember>,
    /// Reactor bars (Java: reserve power capacity).
    pub reactor_power: i32,
    /// Every system slot in file order: 16 in format 7 and later, 12 before.
    pub systems: Vec<System>,
    /// Format 7 and later, when the clonebay is installed.
    pub clonebay: Option<ClonebayInfo>,
    /// Format 7 and later, when the backup battery is installed.
    pub battery: Option<BatteryInfo>,
    /// Format 7 and later, for every ship.
    pub shields: Option<ShieldsInfo>,
    /// Format 7 and later, when the cloaking is installed.
    pub cloaking: Option<CloakingInfo>,
    /// Everything from the rooms on. `None` when the layout port did not
    /// know the ship: those bytes then stay in [`SavedGame::undecoded`].
    pub body: Option<ShipBody>,
    /// Format 7 and later, decoded with [`SavedGame::situation`]: FTL stores
    /// it after the projectiles.
    pub extended: Option<ExtendedShipInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartingCrew {
    pub race: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resources {
    pub hull: i32,
    pub fuel: i32,
    pub drone_parts: i32,
    pub missiles: i32,
    pub scrap: i32,
}

fn len_only<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&format!("{} bytes", bytes.len()))
}
