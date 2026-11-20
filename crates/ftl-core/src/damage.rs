//! Damage and ion damage of a system (`systems.<id>.damage` and
//! `systems.<id>.ion`, also as `nearby.systems.<id>.*`).
//!
//! Evidence (FTL 1.6.13, macOS binary, symbols read with Ghidra 12.1.4 on
//! 2026-11-19; field names from Hyperspace `FTLGameStripped.h`):
//!
//! - Save. `ShipManager::ExportShip` (0x10009c3d3 in its system loop) writes
//!   per installed system: capacity (`+0x64`), power (`+0x60`), the damaged
//!   bars as `healthState.second - healthState.first` (`+0x188` max health
//!   minus `+0x184` health), `GetLockCount` (`iLockCount` `+0x1ac`, our
//!   `ionized_bars`), the lock timer's `currTime * 5000.0` (our
//!   `deionization_ticks`), `(int) fRepairOverTime` (`+0x190`, our
//!   `repair_progress`) and `(int) fDamageOverTime` (`+0x18c`, our
//!   `damage_progress`). The two progress floats are written as plain ints,
//!   not `writeFloat`'s x1000: they run from 0 to 100 (`PartialRepair` and
//!   `DamageOverTime` mend or break one bar at 100.0 and start again at 0).
//! - Load. `ShipManager::ImportShip` upgrades the system to the capacity,
//!   powers it bar by bar, then calls the virtual `AddDamage(damaged)` (vtable
//!   slot 0x100), `AddLock(ion)`, `SetLockTimer(ticks * 0.0002)`,
//!   `SetRepairOverTime(repair)` and `SetDamageOverTime(damage)`.
//!   `UpgradeSystem` raises capacity, max health and health alike, so a
//!   system loads with health = capacity. `AddDamage` sets health to
//!   `clamp(health - n, 0, max health)`: damage above the capacity loads as
//!   every bar broken, a negative one as none. The editor keeps damage
//!   within 0 to the level.
//! - Power. `AddDamage` does not lower the power; `CheckMaxPower` (vtable
//!   0xf0, called by `ShipSystem::OnLoop` every frame) lowers it to
//!   `GetMaxPower` = `min(temporary cap, capacity - damaged - loss)`, so a
//!   damaged bar holds no power. The editor does the same at once
//!   ([`crate::fit_power_to_capacity`]).
//! - Ion. `IonDamage(n)` sets the lock count to n when the system is not
//!   locked and else adds n, at most 5; it restarts the lock timer. The
//!   timer (`lockTimer` +0x1b0, built with 5000 ms) runs 5 s and repeats;
//!   each time it ends `OnLoop` takes one from the lock count. -1 is a lock
//!   without a count (`GetLocked` is true for > 0 or -1; mind control and
//!   hacking set it): the editor shows it and offers 0 to 5. Ion keeps the
//!   power: `ShipSystem::RenderPowerBoxes` draws the powered bars
//!   `COLOR_BLUE_ION` (133, 231, 237) and a blue outline with
//!   `icons/locking/s_lock.png` (`s_lock_white.png` for -1) while locked.
//!   An edit writes a fresh timer: 0 ticks.
//! - Draw. `RenderPowerBoxes` draws a bar below `capacity - damaged` that
//!   holds no power as an outline and a damaged bar filled with `COLOR_RED`
//!   (255, 50, 50) and the red glow; the bar under repair shows the repair
//!   progress over it. The damaged bars are the top ones of the column.
//! - The fixture stores 0 for every one of these fields on every system.

use crate::fit_power_to_capacity;
use crate::model::SystemState;

/// The highest lock count FTL's `IonDamage` stacks to.
pub const MAX_ION: i32 = 5;

/// Sets the damaged bars (0 to the level, checked by the caller) and
/// returns the old count. A repair or new damage starts the repair and
/// damage progress anew, as FTL does when a bar breaks or is mended; the
/// power follows the bars left ([`fit_power_to_capacity`]).
pub fn set_damage(s: &mut SystemState, value: i32) -> i32 {
    let old = std::mem::replace(&mut s.damaged_bars, value);
    if old != value {
        s.repair_progress = 0;
        s.damage_progress = 0;
    }
    fit_power_to_capacity(s);
    old
}

/// Sets the ion lock count (0 to [`MAX_ION`], checked by the caller) with
/// a fresh lock timer and returns the old count.
pub fn set_ion(s: &mut SystemState, value: i32) -> i32 {
    let old = std::mem::replace(&mut s.ionized_bars, value);
    if old != value {
        s.deionization_ticks = 0;
    }
    old
}
