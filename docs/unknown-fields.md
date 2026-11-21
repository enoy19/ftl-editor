# The fields Java leaves unnamed

Vhati's Java editor names 66 fields of `continue.sav` only by Greek letters
(`alpha`, `beta`, …) or with a question mark. This page says what each one
is, read from FTL 1.6.13 itself, and which of them are worth editing.

Status 2026-11-19: every such field has a name in the codec now, except one
field of a beam in flight (`dh`, see Projectiles). The fixture
`continue-1.6.13-format11.sav` holds no unknown field any more
(`crates/ftl-sav/tests/raw.rs` asserts that). One field Java names is
misnamed: the drone list's `playerControlled` is `Drone::GetDeployed`
(see Ship body). The "Fixture" values come from the first fixture
(replaced on 2026-11-19); the second fixture,
`continue-1.6.13-format11-sector3-enemy-drone.sav`, adds a deployed drone
pod (`crates/ftl-sav/tests/roundtrip.rs`).

## Method and sources

- **Binary**: the local FTL 1.6.13 macOS build (`FTL.app/Contents/MacOS`,
  x86_64, with symbols), decompiled with Ghidra 12.1.4 headless. For every
  field the `SaveState`, `Save` or `SaveGame` function that writes it gives
  the offset in the object. Addresses below are function entry points of
  that binary. The Ghidra project behind them is not published, because it
  holds FTL's own code; import the binary into Ghidra and go to an address
  to check a field.
- **Names**: the struct layouts of the FTL: Hyperspace project,
  `libzhlgen/test/functions/mac/FTLGameStripped.h` (commit `1b05a07`,
  2026-10-19), © the FTL: Hyperspace authors, GPL-3.0-or-later. Only field
  names are taken. The header's offsets do not match the binary one to
  one: it declares `std::string` as 8 bytes and `std::map` as 48 bytes
  (the binary uses libc++: 24 and 24), gives a class with a primary base a
  second vtable pointer and packs some classes. A name counts as matched
  when the offset the binary writes lands on the header field after this
  shift and its known neighbours (fields Java already names) land on
  theirs too.
- **Cross-checks**: a second function of the binary that reads or writes
  the same offset with an unambiguous purpose (for example
  `Door::SetBlastDoor`), and Hyperspace's own code that touches the field
  (`src/game/...`).
- **Floats**: `FileHelper::writeFloat` (0x10014f120) stores
  `(int)(f * 1000)` (factor 1000.0 at 0x1002b48a4); `readFloat` multiplies
  by 0.001. Every field marked "×1000" below is such a float.

Confidence:

- **proven**: the binary writes the offset, the header names it with
  matching neighbours, and a second function or Hyperspace confirms the
  meaning.
- **proven (offset)**: the write and the header name match; the meaning
  comes from the name only.
- **likely**: the name rests on the header alone.

"Edit" says whether changing the value in the editor helps a player
(cheat), does nothing lasting, or is risky.

## File header and sector

| Path | Java | Meaning | Evidence | Confidence | Fixture | Edit |
|---|---|---|---|---|---|---|
| `victory` | beta | The running score is a victory (`TopScore::victory`) | `ScoreKeeper::SaveGame` 0x1000298a0 writes `ScoreKeeper+0x2e8` = `currentScore`(+0x2a8)`.victory`(+0x40); neighbours name, blueprint, sector match | proven (offset) | 0 | Risky: marks the run as won for the score list |
| `world.last_escape_event` | epsilon | `StarMap::lastEscapeEvent`, an event id | `StarMap::SaveGame` 0x1001ca4d0 writes the string at +0xc30, between `fuelEventSeed` (+0xc28, our `wait_event_seed`) and `waitedLast` (+0xc48, our `waiting`) | proven (offset) | "" | No known use |

## Crew (`player_ship.crew.<n>`, `nearby_ship.ship.crew.<n>`)

`CrewMember::SaveState` 0x10004fc20, `CrystalAlien::SaveState` 0x10004fbd0.

| Path | Java | Meaning | Evidence | Confidence | Fixture | Edit |
|---|---|---|---|---|---|---|
| `clone_dying` | lambda | Progress of dying while waiting in the clone queue, ×1000 (`fCloneDying`) | written as `(int)` of the float at +0x878; Hyperspace `CloneQueue.cpp` sets it from `CloneSystem::GetDeathProgress` | proven | 0 | No use |
| `teleporting` | nu | The crew member is teleporting (`crewAnim->status == 6`) | written as `crewAnim(+0xc0)->status(+0x60) == 6`; Hyperspace sets status 6 when a teleport starts (`CustomCrew.cpp`) | proven | false | Risky |
| `teleport_anim_reverse` | phi | The teleport animation plays backwards, which Hyperspace reads as arriving | `anims[0][6]` (+0x5a0 in the animation array) `.tracker.reverse` (+0x3d) | proven | 1 of 3 true | Risky |
| `cloned` | omega | `CrewMember::bCloned` (+0x8f0), Crystal only | `CrewMember::Clone` sets +0x8f0 to 1; only `CrystalAlien::SaveState` writes it | proven | 0 | No use |
| `lockdown_recharge_ticks(_goal)` | (named) | `CrystalAlien::powerCooldown`, ×1000, not float bits | +0x8f4, +0x8f8 via `writeFloat`; the constructor sets 0 and 50.0 | proven | — | Goal 50000 for a fresh Crystal (fixed in this lane) |

## Ship body, systems

| Path | Java | Meaning | Evidence | Confidence | Fixture | Edit |
|---|---|---|---|---|---|---|
| `body.doors.<n>.door_level` | delta | `Door::iBlast`: door level 0 to 3; picks the door health | `Door::SaveState` 0x100063c80 writes +0x28; `Door::SetBlastDoor` 0x1000640e0 sets +0x28 to 0..3 and +0x22c (our `current_max_health`) from `DOOR_HEALTH_<n>` | proven | player 1, nearby 0 | No lasting use: follows the doors system level; edit that instead |
| `body.doors.<n>.hacked` | epsilon | `Door::iHacked`: the door is hacked; `SetBlastDoor` then forces level 2 | +0x2ac, read by `SetBlastDoor`; Hyperspace `CustomCrew.cpp` sets it | proven | 0 | Risky |
| `body.doors.<n>.nominal_health` | (named) | `Door::lastbase`, the previous base health | +0x2a8 written as `(int)` float; `SetBlastDoor` stores the old base there | proven | — | — |
| `body.drones.<n>.deployed` | playerControlled (misnamed) | `Drone::GetDeployed`: the drone is out of its bay | `ShipManager::ExportShip` 0x10009ba90 writes per drone the ints of `Drone` vtable slots 0x38 (`GetPowered`, our `armed`), 0x60 (`GetDeployed`), 0x80 (`GetWorldLocation`), 0x90 (`GetDroneSlot`), 0x98 (`GetDroneHealth`); vtable `__ZTV5Drone` 0x100330c90 | proven | first fixture: 0 on three undeployed drones; second fixture: 1 on the deployed `COMBAT_BEAM` | No use: FTL deploys an armed drone itself |
| `shields.shields_down_point_x/_y` | lambda, mu | `Shields::shieldsDownPoint`, ×1000: where the last hit broke a layer | `Shields::Save` 0x1001f0b20 writes +0x2e4/+0x2e8; `Shields::Collision` stores the hit point there; `OnRenderBaseShield` draws from it | proven | 178215, -10417 | Cosmetic |
| `cloaking.turned_on` | alpha | `CloakingSystem::bTurnedOn`: cloaked now | `CloakingSystem::Save` 0x1001e98d0 writes +0x264; `SetTurnedOn` and `OnLoop` use it | proven | 0 | Cheat with care: 1 with a running timer cloaks the ship |
| `cloaking.timer_running` | beta | `TimerHelper::Running` of the cloak timer (+0x268) | same function | proven | 0 | Set together with `turned_on` |
| `extended.hacking.drone_powered` | beta | The hacking drone is powered (`Drone::powered`) | `HackingSystem::SaveState` 0x1001ed6a0 writes drone(+0x268)+0x10; `Drone::SetPowered` 0x10001dba0 writes +0x10 | proven | — | Risky |
| `extended.hacking.drone_pod_visible` | (named) | The hacking drone is deployed (`Drone::deployed`) | drone+0x18; `Drone::SetDeployed` 0x10001da50 | proven | — | Name kept |
| `extended.hacking.hacking` | delta | `HackingSystem::bHacking`: hacking its target now | +0x264; `StopHacking`, `OnLoop`, `InitiatePulse` use it | proven | — | Risky |
| `extended.weapon_modules.<n>.fire_shot` | phi | `WeaponAnimation::bFireShot` | `WeaponAnimation::SaveState` 0x10004ca30 writes +0xf0 (and `firing` = `bFiring` +0xf1) | proven (offset) | false | No use |
| `extended.weapon_modules.<n>.anim_charge` | (named) | `WeaponAnimation::boostLevel` (+0x3a0), not a charge | same function | proven (offset) | — | Name kept |

## Drone pods (`extended.drones.<n>.pod`, hacking drone, surge drones)

`SpaceDrone::SaveState` 0x100035640. `SpaceDrone` derives from `Drone`, so
the header's offsets are 8 too high; after that shift every field Java
named (`currentSpace` +0x58 … `ionStun` +0x2c4 = `overload_ticks`) matches.

| Path | Java | Meaning | Offset | Confidence | Fixture | Edit |
|---|---|---|---|---|---|---|
| `point_target_x/_y` | epsilon, zeta | `pointTarget`, the point the drone aims at, ×1000 | +0x78 | proven (offset) | `i32::MIN` (−FLT_MAX) | No use |
| `target_speed_x/_y` | iota, kappa | `targetSpeed`, ×1000; Hyperspace uses it to aim ahead (`Balance.cpp`) | +0x180 | proven | 2480, 1561 on one pod | No use |
| `last_aiming_angle` | xi | `lastAimingAngle`, ×1000 | +0x1b0 | proven | 52667, -90544, 0 | No use |
| `disrupted` | pi | `bDisrupted`: scrambled; Hyperspace `CustomAugments.cpp` sets it | +0x2bd | proven | 0 | Risky |
| `powered_last_frame` | rho | `poweredLastFrame` | +0x198 | proven (offset) | 0 | No use |
| `desired_aiming_angle` | tau | `desiredAimingAngle`, ×1000; -1 for a new drone | +0x1b4 | proven | -15944, -78549, 85193 | No use |
| `deployed_last_frame` | upsilon | `deployedLastFrame` | +0x199 | proven (offset) | 0 | No use |
| `info.died_in_space` (boarder) | alpha | `BoarderPodDrone::diedInSpace` | +0x539 (`BoarderPodDrone::SaveState` 0x100037b40) | proven (offset) | — | No use |
| `info.delivered_drone` (boarder) | beta | `bDeliveredDrone`: the boarder is on the enemy ship (Hyperspace `CustomDrones.cpp`) | +0x538 | proven | — | Risky |
| `info.starting_position_x/_y` (boarder) | gamma, delta | `startingPosition`, ×1000 | +0x3e0 | proven (offset) | — | No use |
| `info.arrived` (hacking) | gamma | `HackingDrone::arrived` | +0x3f8 (`HackingDrone::SaveState` 0x10001e570) | proven (offset) | — | Risky |
| `info.finished_setup` (hacking) | delta | `HackingDrone::finishedSetup` | +0x3f9 | proven (offset) | — | Risky |
| `info.previous_exhaust_angle` (zigzag) | epsilon | `CombatDrone::oldHeading`, ×1000 (`exhaust_angle` is `heading`) | +0x3e0 (`CombatDrone::SaveState` 0x1000355d0); Hyperspace sets `oldHeading = heading` | proven | -40056, 4212 | No use |
| `info.glow_animation` (shield) | alpha | `SuperShieldDrone::glowAnimation`, ×1000 | +0x6b0 (0x10003b2e0); Hyperspace pulses it | proven | — | Cosmetic |
| `standalone_drones.<n>.deployed/destroyed/armed` | alpha, beta, gamma | `Drone::GetDeployed`, `Drone::Destroyed`, `Drone::GetPowered` | `ShipManager::SaveSuperDrones` 0x100094250 calls vtable slots 0x60, 0x78, 0x38; `SaveState` sits at 0xd0, which fixes the slot order of the header's `VTable_Drone` (+8 for the second destructor) | proven | — | Risky |

The standalone drones are the drones of the rebel flagship's drone surge
(`BossShip::OnLoop` calls `ShipManager::PrepareSuperDrones` in stage 2).

## Encounter and situation

| Path | Java | Meaning | Evidence | Confidence | Fixture | Edit |
|---|---|---|---|---|---|---|
| `situation.ship_event_present` | mu | `ShipEvent::present`: the beacon has a ship event | `ShipEvent::Save` 0x1001bc120 writes byte +0x0 | proven (offset) | 1 | Risky |
| `situation.encounter.text_is_literal` | alpha | `TextString::isLiteral` of the event text: 1 when `text` is the text, 0 when a text id | `WorldManager::SaveGame` 0x1001e6500 writes `lastMainText.isLiteral` (+0x1768) right before `lastMainText.data` (+0x1750) | proven (offset) | 0 | Risky |
| `situation.player_teleport_target_room` | nu | `CompleteShip::teleTargetRoom` of the player ship | `CompleteShip::SaveState` 0x100008860 writes +0x1e8; `CompleteShip::InitiateTeleport` 0x100008980 stores its room argument there; Hyperspace `TeleporterBugFixes.cpp` | proven | 0 | No use |
| `situation.nearby_ship.teleport_target_room` | xi | The same for the nearby ship; `WorldManager::SaveGame` writes the player's first | as above | proven | 0 | No use |
| `environment.asteroid_field.init_shields` | alpha | `AsteroidGenerator::initShields`: the argument `Initialize` got; it takes half of it (1 to 3) as the intensity | `SpaceManager::SaveSpace` 0x10003c8a0 writes `asteroidGenerator`(+0x18)`+0x94`; `AsteroidGenerator::Initialize` 0x1000143a0 stores its 2nd argument there | proven | — | No known use |
| `environment.asteroid_field.state` | gamma | `AsteroidGenerator::iState`: 0 calm, 1 or 2 a volley | `SaveAsteroids` 0x100014b70 writes +0x7c; `UpdateState` 0x100014ae0 switches between 0 and a random 1 or 2 | proven | — | Cheat: 0 with a long `stray_rock_ticks` keeps the field calm |
| `environment.asteroid_field.stray_rock_ticks` | (named) | `fStateTimer`, ×1000: the time left in `state` | +0x88 | proven | — | See `state` |
| `environment.asteroid_field.current_target` | (unsure) | `AsteroidGenerator::currentSpace`: the ship the rocks fly at | +0x80 | proven (offset) | — | — |
| `rebel_flagship.current_stage` | alpha | `BossShip::currentStage`, 1 to 3 | `BossShip::SaveBoss` 0x100001b00 writes +0x1ec; `StartStage`, `Defeated`, `OnLoop` branch on it | proven | 0 | Risky: must match the flagship the save holds |
| `rebel_flagship.power_ticks_goal` | gamma | Goal of `BossShip::powerTimer`, ×1000 (30 s at start) | +0x1fc; the constructor sets the timer to 30000 ms; on `Done` stage 2 launches the drone surge, stage 3 the barrage or super shield | proven | 30000 | Cheat: a large goal delays the next surge |
| `rebel_flagship.power_ticks` | delta | Time on that timer, ×1000 | +0x1f8 | proven | 0 | Cheat: 0 restarts the countdown |

The nearby ship's AI fields that Java marked with a question mark are
confirmed by `ShipAI::Save` 0x10017cfa0: `destroyed`, `stalemate_triggered`
(`bStalemateTrigger`), `stalemate_ticks` (`fStalemateTimer`),
`boarding_attempts` (`iTimesTeleported`), `boarders_needed`
(`iCrewNeeded`). The environment's `solar_flare_fade_ticks` is the
progress of `SpaceManager::flashTracker`, `havoc_ticks` the time of
`flashTimer`, `pds_ticks` `randomPDSTimer`. The animation fields of every
`Anim` follow `Animation::SaveState` 0x10013fb10: `playing` is
`tracker.running`, `looping` `tracker.loop`, `x`/`y` `position`.

## Projectiles

| Path | Java | Meaning | Evidence | Confidence | Edit |
|---|---|---|---|---|---|
| `laser_or_burst.spin_angle` | alpha | `LaserBlast::spinAngle`, ×1000 (`spin` is `spinSpeed`) | `LaserBlast::SaveProjectile` 0x100021460, +0x300 | proven (offset) | No use |
| `bomb.missed` | alpha | `BombProjectile::bMissed` | `BombProjectile::SaveProjectile` 0x100018d40, +0x2f8 | proven (offset) | Risky |
| `bomb.super_shield` | gamma | `bSuperShield` | +0x30c | proven (offset) | Risky |
| `bomb.super_shield_bypass` | delta | `superShieldBypass` | +0x30d | proven (offset) | Risky |
| `pds.start_point_x/_y` | alpha, beta | `PDSFire::startPoint`, ×1000 | `PDSFire::SaveProjectile` 0x1000234b0, +0x308 | proven (offset) | No use |
| `pds.passed_target` | gamma | `PDSFire::passedTarget` | +0x310 | proven (offset) | No use |
| `pds.current_scale` | delta | `currentScale`, ×1000 | +0x314 | proven (offset) | No use |
| `pds.missed` | epsilon | `missed` | +0x318 | proven (offset) | Risky |
| `pds.explosion_anim` | zeta | `explosionAnimation` | +0x320 | proven (offset) | No use |
| `beam.final_end_x/_y` | beta | `final_end`, ×1000 | `BeamWeapon::SaveProjectile` 0x100015a80, +0x310 | proven (offset) | No use |
| `beam.lifespan` | gamma | `lifespan`, ×1000 | +0x328 | proven (offset) | No use |
| `beam.unknown_delta` | delta | `dh`; the header gives only this name | +0x330 | unknown | No use |
| `beam.last_collision_x/_y` | epsilon | point of `last_collision` | +0x338 | proven (offset) | No use |
| `beam.animation_timer` | zeta | `animationTimer`, ×1000 | +0x368 | proven (offset) | No use |
| `beam.last_damage` | eta | `lastDamage` (int) | +0x36c | proven (offset) | No use |
| `beam.timer` | iota | `timer`, a float ×1000 | +0x37c | proven (offset) | No use |
| `beam.pierced_shield` | kappa | `piercedShield` | +0x388 | proven (offset) | No use |
| `beam.damage_super_shield` | mu | `bDamageSuperShield` | +0x38a | proven (offset) | No use |
| `beam.moving_target_id` | nu | `movingTargetId` (int, a ship id) | +0x38c | proven (offset) | No use |

`beam.timer` and `beam.moving_target_id` were `Bool32` (0 or 1 only), as
Java reads them. A beam with a running timer therefore made the whole
situation fail to decode; they are `i32` now.

## Profile (`ae_prof.sav`)

`ScoreKeeper::Save` 0x100028ab0 writes, after the achievements, 12 ship
slots of two flags each: bit 0 (Type-A) and bit 2 (Type-C) of the
`unlocked` lists. FTL 1.6.13 has 10 player ships, so the last two slots
(`UNKNOWN_BETA`, `UNKNOWN_GAMMA` in format 9) have no ship: proven (offset),
likely unused. Every other profile field already had a name (Hyperspace's
rebuild in `src/game/UserInterface/SaveFile.cpp` agrees).

## What is worth editing

Few of these fields are cheats. The ones with a visible effect:

1. `rebel_flagship.power_ticks` and `power_ticks_goal`: delay the drone
   surge (stage 2) or the barrage and super shield (stage 3).
2. `environment.asteroid_field.state` with `stray_rock_ticks`: keep an
   asteroid field calm.
3. `cloaking.turned_on` with `timer_running`, `cloak_ticks` and
   `cloak_ticks_goal`: start the save cloaked.

Everything else either follows another value FTL recomputes (door
level), is cosmetic (shield-down point, glow, angles), or describes an
action in flight that a wrong value can break (teleport, hacking,
boarding, projectiles). The editor offers none of them as a field; the
Expert tree edits them raw.
