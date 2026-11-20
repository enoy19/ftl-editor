//! Domain of the FTL editor.
//!
//! The dependency rule points inward: `ftl-core` knows no file format and
//! performs no IO. Adapters (`ftl-sav` for the binary format, `ftl-app` for
//! files and backups) implement the ports in [`ports`]; the CLI and the
//! Flutter bridge talk to `ftl-app` only.

pub mod actions;
pub mod beacons;
pub mod crew_list;
pub mod damage;
pub mod edit;
pub mod energy_shield;
pub mod identity;
pub mod installs;
pub mod items;
pub mod layout;
pub mod model;
pub mod ports;
pub mod profile;
pub mod raw;
pub mod sector_map;
pub mod sector_tree;
mod slot_order;

pub use actions::{Action, ActionNote, ActionPlan, ShipLimits, UnknownAction};
pub use edit::{
    Assignment, Change, EditError, Field, INDEXED_PATHS, MAX_AUGMENTS, MAX_BREACH_HEALTH,
    MAX_CREW_NAME, MAX_REACTOR, MAX_ROOM_OXYGEN, Patch, Resource, Value, fit_power_to_capacity,
    fit_power_to_reactor, reactor_power_in_use,
};
pub use layout::{
    DroneFacts, ItemSlots, KnownLayouts, RoomShape, ShipLayouts, ShipShape, SystemRoom, WeaponFacts,
};
pub use model::*;
pub use ports::{DecodeError, ProfileCodec, SaveCodec};
pub use raw::{RawEntry, RawKind};
