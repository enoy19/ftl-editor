//! The Zoltan shield of a ship (`systems.shields.energy`, also as
//! `nearby.systems.shields.energy`): the green layer over the shield
//! bubbles that FTL calls the super shield.
//!
//! Evidence (FTL 1.6.13, macOS binary, symbols read with Ghidra 12.1.4 on
//! 2026-11-19; names from Hyperspace `FTLGameStripped.h`):
//!
//! - Save. `Shields::Save` (0x1001f0b20) writes `shields.power.first`
//!   (+0x290, our `shield_layers`), `power.super.first` (+0x298, our
//!   `energy_shield_layers`: the Zoltan shield points now),
//!   `power.super.second` (+0x29c, our `energy_shield_max`), the charge
//!   float of the normal layers (+0x28c, `writeFloat` x1000, our
//!   `shield_recharge_ticks`) and three animations. The Zoltan shield has
//!   no charge of its own: our `energy_shield_anim_*` is the tracker at
//!   +0x368 that `AddSuperShield` starts when a point appears.
//! - Load. `Shields::Load` (0x1001f0c20) stores both ints as read, without
//!   a clamp. `ShipManager::ImportShip` calls it only when the ship has a
//!   shields system (`shieldSystem`, `ShipManager+0x80`), and
//!   `ShipManager::GetShieldPower` answers 0 without one: the Zoltan shield
//!   needs the shields system.
//! - Max. `Shields::OnLoop` (0x1001f1ff0) sets the max to 5 every frame
//!   while the ship has the augment `ENERGY_SHIELD` (`HasEquipment`), and
//!   the points to 5 when the max was below 1. `Shields::Jump` sets the
//!   points to the max with the augment, else to 0; `InstantCharge` sets 5
//!   with it. `AddSuperShield` adds one point up to 5 and sets the max to
//!   5 when it was 0. The rebel flagship's `PowerSuperShields` writes 12
//!   and 12. No function lowers points above the max; damage takes them
//!   first (`Shields::Collision`).
//! - Who has it. The augment sits in every Zoltan cruiser
//!   (`PLAYER_SHIP_ENERGY`, `_2`, `_3`) and the enemy blueprints
//!   `ZOLTAN_FIGHTER` and `ZOLTAN_BOMBER` (and their `_DLC` copies);
//!   `ShipGenerator::CreateShip` drops it from an enemy in the first sector
//!   on Easy. A store sells it as Zoltan Shield.
//! - Draw. The player's `ShipStatus::RenderShields` (0x100116480) draws the
//!   charge bar while the points are below 1, else one image per point
//!   from five that `ShipStatus::OnInit` builds:
//!   `statusUI/energy_shield_bar.png` (17 x 6) at x + 3, 22, 41 (that one
//!   `energy_shield_bar2.png`, 16 x 6), 59 and 78. A sixth point would read
//!   past those five primitives, so the player keeps at most 5. The enemy
//!   box (`CombatControl::RenderShipStatus`) draws the frame
//!   `combatUI/box_hostiles_shield_super5.png` for max 5 and `_super12` for
//!   max 12 (else a dark box 10 px per point) and over it a bar of 10 px
//!   per point in `COLOR_GREEN` (100, 255, 100). `Shields::RenderBase`
//!   tints the shield bubble with the same green, its alpha the points
//!   over `max(5, max)`.
//! - Fixtures. The sector-3 save carries the augment and stores 5 and 5;
//!   the sector-1 Kestrel and both enemy ships store 0 and 0.
//!
//! The editor offers 0 to 5 points on the player ship and 0 to `max(5,
//! max)` on the enemy (12 on the flagship). A point given to a ship whose
//! max is 0 sets the max to 5, as `AddSuperShield` does; without the
//! augment the points last until the next jump.

use crate::EditError;
use crate::edit::{Field, ship, ship_mut};
use crate::model::{SavedGame, ShieldsInfo, ShipId, SystemKind};

/// The points FTL's own paths give: the `ENERGY_SHIELD` augment and
/// `AddSuperShield`; also the most the player's status bar draws.
pub const ZOLTAN_POINTS: i32 = 5;

/// The shield state the field edits, or why it cannot: the shields system
/// must be installed and the file must store the state (format 7 on).
pub(crate) fn info<'g>(game: &'g SavedGame, field: &Field) -> Result<&'g ShieldsInfo, EditError> {
    let ship = ship(game, field)?;
    let installed = ship
        .systems
        .iter()
        .any(|s| s.kind == SystemKind::Shields && s.installed.is_some());
    if !installed {
        return Err(EditError::NotInstalled {
            field: field.clone(),
            system: SystemKind::Shields,
        });
    }
    ship.shields.as_ref().ok_or_else(|| EditError::NotDecoded {
        field: field.clone(),
        part: "the shield state (format 7 and later)",
    })
}

/// The most points the ship may hold: 5 on the player ship, `max(5, max)`
/// on the enemy ship.
pub fn cap(on: ShipId, info: &ShieldsInfo) -> i32 {
    match on {
        ShipId::Player => ZOLTAN_POINTS,
        ShipId::Nearby => info.energy_shield_max.max(ZOLTAN_POINTS),
    }
}

/// Sets the points (checked by the caller) and returns the old value; a
/// max of 0 becomes 5 when points appear.
pub(crate) fn set(game: &mut SavedGame, field: &Field, value: i32) -> i32 {
    let info = ship_mut(game, field)
        .shields
        .as_mut()
        .expect("info() checked the shield state");
    if value > 0 && info.energy_shield_max <= 0 {
        info.energy_shield_max = ZOLTAN_POINTS;
    }
    std::mem::replace(&mut info.energy_shield_layers, value)
}
