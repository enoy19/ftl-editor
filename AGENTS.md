# AGENTS.md — ftl-editor

Editor for FTL: Faster Than Light saved games (`continue.sav`) and the player
profile (`ae_prof.sav`), single player.
A Rust core with a CLI (`ftled`) and a Flutter desktop app over the same API.
Target platforms: macOS, Windows, Linux. Development and the integration tests
run on macOS; GitHub Actions checks and builds all three ("Continuous integration").

## Layout and dependency rule

Dependencies point inward. A crate may use only the crates above it in this list.

| Path | Role | Knows |
|---|---|---|
| `crates/ftl-core` | Domain model, edits as data (`Patch`, `profile::unlock_all_ships`), ports (`SaveCodec`, `ProfileCodec`) | nothing: no IO, no file format |
| `crates/ftl-sav` | Binary codecs of `continue.sav` and, in `profile/`, `ae_prof.sav` (adapters for both ports, sharing `FtlString` and `Bool32`) | `ftl-core`, `binrw` |
| `crates/ftl-assets` | Reads `ftl.dat`: blueprints, names, ship layouts, sprite PNGs as a `GameData` value | `ftldat`, `quick-xml` |
| `crates/ftl-app` | The one API: `SaveFile` and `ProfileFile` open, edit, verify, back up, save through the same helpers; OS paths; `assets`: find, remember and load `ftl.dat` | `ftl-core`, `ftl-sav`, `ftl-assets` |
| `crates/ftl-cli` (package `ftled`) | `ftled`, a clap adapter | `ftl-app` only |
| `app/rust` (`ftl_bridge`) | flutter_rust_bridge adapter, DTOs and an opaque `SaveSession` | `ftl-app` only |
| `app/lib` | Flutter UI. Collects input, shows what Rust answers | the generated bindings only |
| `reference/ftl-profile-editor` | Submodule of [Vhati's Java editor](https://github.com/Vhati/ftl-profile-editor), the public upstream at a pinned commit: format reference and oracle. `reference/patches/` holds the Java 21 build and the two round-trip probes; `scripts/oracle.ts` applies them | — |

A rule (ranges, verify, backup, changed-on-disk check) lives in `ftl-core` or
`ftl-app`, never in the CLI, the bridge or Dart. The CLI and the app call the
same `ftl-app` functions.

## UI direction

The Java editor is the format reference, never the UI reference: its tabs
mirror the file (Dump, State Vars, Sector Tree). The app follows the tasks of
a player instead: one top bar with the area tabs Ship, Crew, Equipment, Map
and Profile on the left and Expert and Save on the right; raw data only in a collapsed Expert area; game sprites and the
floorplan from the local `ftl.dat`; one-click actions with limits from the
blueprints; a change list with undo before every save; a backup history; a
status bar for file, running FTL, Steam Cloud and last backup. Settle each
new view on rendered variants before it is built.

## The codec contract

- `encode(decode(bytes)) == bytes` for every file FTL writes. Each fixture
  under `crates/ftl-sav/tests/fixtures/` is a file written by FTL itself.
  The main one, `continue-1.6.13-format11.sav`, is a sector-1 Kestrel
  (`PLAYER_SHIP_HARD`, three humans, two weapons, no drone, cargo,
  augment, fire or breach) with the pirate `JELLY_BUTTON_P_DLC` nearby. A
  test that needs more builds it through the edit paths (`Patch`, for
  example `systems.drones.installed=1`, `drones.COMBAT_1=1`,
  `crew.2.race=anaerobic`), never with a second edited file. The fake
  layout port `crates/ftl-sav/tests/common/layouts.rs` describes these
  two ships. A second FTL-written save,
  `continue-1.6.13-format11-sector3-enemy-drone.sav`, is the evidence for
  what the main one lacks: The Kestrel in sector 3 (four armed
  `ARTILLERY_BOSS_*`, three augments) with the auto ship `AUTO_ASSAULT`
  nearby, whose `COMBAT_BEAM` is deployed, in an asteroid field with one
  rock in flight. `drone_fixture_layouts()` in the fake decodes it to its
  end; `tests/roundtrip.rs` reads the drone pod, `tests/sector_map.rs` the
  sector-3 layout. It has no quest, store, fleet flag, Cloaking, Battery
  or Mind Control either; an FTL-written save with those is still wanted.
- The codec decodes front to back. Everything after the last decoded field is
  `SavedGame::undecoded` and goes back unchanged. Today the codec decodes the
  header, the state vars, the whole player ship (`readShip`), after the ship
  body the world (`SavedGame::world`: cargo, sector seeds, rebel fleet,
  beacons with stores, quests) and after the world the situation
  (`SavedGame::situation`: encounter, nearby ship with AI, environment,
  projectiles, rebel flagship; the extended ship info lands in
  `Ship::extended`). With the layouts of `ftl.dat` the fixture leaves
  `undecoded` empty. The situation decodes as one piece or not at all.
- Rooms and doors need the ship shape from `ftl.dat` (room count, squares per
  room, door count, systems in several rooms): the bytes do not say where
  they end. `SaveCodec::decode` takes the port `ftl_core::ShipLayouts`, keyed
  by ship blueprint id. A ship it does not know decodes up to its systems
  (one slot per system); its rooms and everything after them stay in
  `undecoded` (`Ship::body` is `None`). The same port answers the kind of a
  drone blueprint (`drone_kind`), because the extended ship info stores a
  drone pod whose fields depend on it; an unknown nearby ship or drone kind
  keeps the situation undecoded. The adapter is
  `ftl_app::layouts::GameLayouts` over ftl-assets: rooms by room id, doors by
  distinct (x, y, vertical) like Java's door map, one system slot per
  `systemList` entry. `SaveFile::open` loads the located `ftl.dat` once per
  call and falls back to no layouts when none is found or readable;
  `SaveFile::layout_source` and `ftled show` say which. Tests pass layouts
  with `open_with_layouts`; the fake in `crates/ftl-sav/tests/common/layouts.rs`
  equals the real shape (`crates/ftl-app/tests/real_layouts.rs`).
- Field names follow the FTL 1.6.13 binary: `docs/unknown-fields.md` names
  the save function, address and offset of every field Java leaves as a
  Greek letter, and how to check a new one. `FileHelper::writeFloat` stores
  `(int)(f * 1000)`, never float bits; a field FTL writes as a float holds
  that fixed point value. Only `beam.unknown_delta` is still unknown.
- A new decoded field moves bytes out of `undecoded` into a typed field. The
  layout reference is `reference/ftl-profile-editor/src/main/java/net/blerf/ftl/parser/SavedGameParser.java`
  (`readSavedGame`, `readShip`).
- `wire/` mirrors the bytes with binrw, one struct field per field on disk.
  `map/` converts wire to domain. Keep the two layers apart; each has one
  module per area of the file (`crew.rs`, `systems.rs`, `body.rs`). The
  codec reads in passes (head, systems, body, world, situation; see
  `wire/mod.rs`).
- Edits are paths in `ftl-core::edit`: `ship.<resource>`,
  `crew.<n>.skills.<skill>`, `crew.<n>.health` (1 to `max_health(race)`,
  a table from the game binary via the Java editor, Crystal 125),
  `crew.<n>.name` and `crew.<n>.race` (text: `Value::Text`; a name has 1
  to 32 characters and is stored trimmed; a race is an id of `RACES`, the
  eight playable races; `CrewMember::change_race` keeps the health share
  and each skill's mastery level, gives one tint per color layer of the new
  race and the lockdown fields of a fresh Crystal; the evidence from
  `ShipManager::ImportShip` and Hyperspace is in its doc),
  `crew.add` (how many default humans the patch appends), `crew.<n>.remove`
  (0 or 1), `crew.<n>.tints.<layer>` and `crew.<n>.male` (the look) live in
  `ftl_core::crew_list`; its module doc names the FTL 1.6.13 functions
  (Ghidra) behind a new member's fields, its square (pilot station, a free
  square, the nearest room), the limit (8 player, 10 enemy, `IsCrewFull`)
  and the look of the character creator. A new member's name is the first
  free name of `names.xml` (port `crew_names`). A removal is staged on the
  member (`CrewMember::removed`) and done at the end of `Patch::apply_in`,
  so every index of a patch is a file index; `Patch::stage_in` (and
  `SaveFile::preview`) keeps it marked; the last crew member of a side
  stays. The bridge's `patch_of` puts the crew look after the texts
  (`Field::is_crew_look`), so a look picked for a pending race stays,
  `reactor`
  (0 to 25), `systems.<id>.level` (1 to the system maximum),
  `systems.<id>.damage` (broken bars, 0 to the level) and
  `systems.<id>.ion` (ion lock count, 0 to 5, a fresh lock timer;
  `ftl_core::damage` names the FTL 1.6.13 save, load, power and draw
  functions behind both; a progress field there is a plain int 0 to 100,
  the lock timer seconds x 5000, neither `writeFloat`),
  `systems.shields.energy` (Zoltan shield points, 0 to 5 on the player
  ship, up to `max(5, max)` on the enemy; a point on a ship with max 0
  sets the max to 5; `ftl_core::energy_shield` holds the FTL 1.6.13
  evidence; the sector-3 fixture is FTL's own 5 of 5 with the
  `ENERGY_SHIELD` augment). Every value
  can be lowered: a level below the power of its system lowers the power
  (`fit_power_to_capacity`: power at most level minus damaged bars, battery
  bars last); a reactor below the bars in use (`reactor_power_in_use`: the
  non-subsystems without battery bars) takes reactor bars from the systems,
  the last of the file first (`fit_power_to_reactor`). Both mirror FTL
  1.6.13 `ShipManager::ImportShip`, which powers the systems bar by bar in
  file order until the reactor runs out, and `ShipSystem::GetMaxPower`;
  armed weapons and drones stay as they are (FTL's `PowerWeapon` refuses
  what the power does not cover). `augments.<ID>` (how
  many of that augment, at most 3 in all; needs the decoded body) and
  `beacons.<n>.seen` (0 or 1; needs the decoded world), `fires.<x>.<y>`
  and `breaches.<x>.<y>` (health 0 to 100 at a room square of the ship
  grid, layout offset included, the coordinates the file stores for a
  breach; `Room` carries its grid position from the layout). 0 puts the
  fire out (fire health and ignition; the ignition of the whole room once
  none of it burns) or removes the breach; 100 lights a fire (fire health
  only, extinguishment -1) or appends a breach, as Java's `addFire` and
  `addBreach` do. A square no room covers is refused. `rooms.<n>.oxygen`
  (0 to 100 percent, by room id): FTL saves `(int)` of the float per room,
  0 for a ship without the oxygen system, and ignores the value at load
  then (`ShipManager::ExportShip`/`ImportShip`), so the field exists only
  with the system (`Field::RoomOxygen` holds the evidence). `weapons.<ID>`,
  `drones.<ID>` (copies in the slots, within `weaponSlots`/`droneSlots`),
  `weapons.<ID>.armed`, `drones.<ID>.armed` (armed copies within the free
  bars of system and reactor; arming moves bars, because a system draws
  exactly its armed items) and `cargo.<ID>` (4 boxes) live in
  `ftl_core::items`; its module doc names the FTL 1.6.13 functions (macOS
  binary, Ghidra) behind the module and pod a new weapon or drone carries.
  A count edit removes the last copies and appends new ones; within one
  patch a new weapon, drone, augment or cargo item takes the first slot a
  removal freed (`ftl_core::slot_order`, FTL's next free slot), so a
  replace (remove `A`, then add `B`) keeps `B` in `A`'s slot.
  They need the port's `slots`, `weapon` and `drone` (ftl.dat): `Patch::apply_in`
  and `Field::range_in` take it, `SaveFile::apply` and `preview` pass the
  file's layouts. A deployed drone, a crew drone's arming and the kinds FTL
  never puts in a slot (hacking, beam) are refused. Parsing,
  ranges and the all-or-nothing rule live there; `ftled fields` lists the
  paths.
- The enemy ship: `nearby.<path>` (`Field::Nearby`, `Field::on(ShipId)`)
  is every ship path above on the ship at the beacon (`nearby.ship.hull`,
  `nearby.systems.shields.level`, `nearby.crew.0.health`, the text fields
  `nearby.crew.0.name` and `nearby.crew.0.race`, `nearby.fires.2.0`, ...);
  `beacons` and `cargo` belong to the player and
  take no prefix. One implementation: the helpers of `edit`, `items` and
  `installs` resolve the ship of the field (`SavedGame::ship`). It needs the
  decoded situation with a ship at the beacon. FTL loads both ships with the
  same `ImportShip`; the two differences are in the module docs of `items`
  (a new drone starts in its owner's space, 1) and `installs` (a manned
  system other than the pilot has a random station on an enemy ship:
  `InstallBlock::NoStation`). Enemy blueprints mostly name no slots: FTL's
  default is 4 weapons and 4 drones (`layouts::DEFAULT_ITEM_SLOTS`).
  `roster`, `loadout`, `item_fields`, `augment_count` and `floorplan` take
  the `ShipId`; the bridge takes `nearby: bool` (also `crew_preview`), the summary carries
  `nearby_blueprint_id` and `nearby_fields`; `ftled crew list --nearby`,
  `ftled equipment list --nearby`. The app reads both ships, so the edits
  of both join `_fields` and the change sheet; `ShipSwitch` (components;
  in the FTL look `FtlChipRow`, the tab faces with all four corners cut)
  picks the ship shown: part of the scrolling content of Ship (below the
  file and ship card) and Crew (`shipSwitch`), above the two panels of
  Equipment (`ShipSwitchFrame`, `app/lib/ui/ship_choice.dart`); never
  pinned. A label of an
  enemy field reads "Enemy ship: ..." (`shipNamed`). Path-to-label maps
  strip the prefix with `localPath`. The Equipment page shows no cargo hold
  for the enemy ship. A save without a ship at the beacon shows no switch.
- The crew: `ftl_app::crew::roster` answers each crew member with its fields,
  ranges and mastery levels (`SkillKind::mastery_level`, `mastery_points`);
  `ftled crew list` and the bridge (`api/crew.rs`, `crew_of`) call it;
  `race_choices` answers the races of `RACES` named from `ftl.dat` in the app
  language. One `CrewCard` (`app/lib/ui/crew_card.dart`, variant B of
  2026-11-19) serves the Crew page and the floorplan popover: the centered
  portrait in a dark well, name with Rename (a popover with a text field),
  race as a tag that opens the race picker (portraits and titles), an FTL
  health bar, Heal and Train fully, and a 3x2 grid of skill tiles (FTL's
  `img/people/skill_<name>_white.png` multiplied with the mastery color,
  the full name, two level pips). A click on a tile sets the next of the
  three `mastery_points` (0 → 1 → 2 → 0), a long press or a right click the exact points; an
  edited tile has a yellow (raised) or red (lowered) outline and a
  `ChangeMark`. Number edits join the pending edits of the Ship page; name
  and race are text edits (`CrewTexts`, `EditorPage._texts`) that the
  bridge applies with `SaveSession.apply(edits, texts)` (`TextEdit`) and
  the change sheet shows as words (`crewTextChangeRows`). A female human
  wears the `female` portrait. A pending race shows the new portrait and
  title, and the skills and health of `crew_preview` (the core applies every
  pending edit to a copy, numbers before texts, as `patch_of` in the bridge);
  skill edits stay in the units of the file's race. Every crew sprite (card,
  floorplan figure, square menu, race and look pickers, store crew) comes
  from `crewSprite`/`CrewSprite` (`app/lib/ui/crew_sprite.dart`) with the
  core's look id (race, sex, tints; `ImageId::CrewLook`, anchored
  `CrewLookFrame` on the floorplan); `test/crew_sprite_guard_test.dart`
  fails on any other file naming a crew image kind. The Crew page ends with
  a card with a plus (`crew-add`, stages `crew.add`); the editor reads the
  new members from `crewPreview` with only that edit and lists them after
  the file's crew (their edits move down when one is dropped). Each card
  has Remove (`remove-<n>`) and a Look tag next to the race (`look-<n>`,
  `lookChoices`); the change sheet names these edits in words
  (`crewListChangeRows`). The floorplan draws the crew of the preview.
- The equipment: `ftl_app::equipment::loadout` answers reactor, systems
  (installed ones with `systems.<id>.level`, the level range from 1 to the
  cap of the ship blueprint; listed but missing ones without),
  weapons, drones and augments with names and power from `ftl.dat`;
  `augment_count` answers `augments.<ID>` also for an augment the ship lacks,
  `augment_choices` every augment by name; `item_fields` the weapon, drone
  and cargo paths of one id with their ranges, `item_choices` every weapon
  and drone FTL equips. `ftled equipment list|augments|set`
  and the bridge (`api/equipment.rs`, `equipment_of`, `equipment_preview`,
  `augment_field`, `augment_choices`, `item_fields`, `item_choices`) call
  them. `systems.<id>.installed` (0 or 1) installs or removes a system
  (`ftl_core::installs`; its module doc names the FTL 1.6.13 functions):
  only in a room of the ship blueprint (`ShipLayouts::system_rooms`, the
  `systemList` with `start` and `<slot>`; FTL crashes loading a system
  without one), at the level FTL's `AddSystem` gives, within the store's
  limits of 8 systems and 4 subsystems; start systems stay (FTL builds
  them again at load), Medbay and Clone Bay swap like the store swaps
  them, a weapons or drone system with items stays. Each `SystemEntry`
  carries the field (0 to 0 when blocked) and the `InstallBlock` (stable
  `code()`, ARB `equipBlock*`). The Equipment page shows Install / Remove
  per row; a staged install or removal locks the row's segments. The
  floorplan reads its rooms from the core's preview of the pending edits
  (`floorplan_preview` beside `equipment_preview`): system, interior,
  symbol, square menu title and system entry, and the popover (locked for
  a staged system) follow it; fields and marks stay on the file's plan.
  Dart never derives an install from the edits itself.
  The Equipment page (`app/lib/ui/equipment_page.dart`, variant C) puts
  its edits into the pending edits of the Ship page. A system row is
  `SystemRow`, the one builder for the page and the floorplan room popover:
  file system, preview and edits give level, damage and ion (ranges from
  the preview, values from the file), icon state, Install/Remove and marks;
  `test/system_row_guard_test.dart` fails on a `SystemLevelRow` built
  elsewhere. Its `SystemLevelRow` draws a segment an edit adds yellow (accent,
  `FtlColors.hover`) and one it removes red (warn, `FtlColors.warn` from
  `top_hull_red_label.png`), shows only the new value, and puts a
  `ChangeMark` (components) at the icon; the floorplan room of an edited
  system carries the same mark. The same row draws damage as FTL's
  `RenderPowerBoxes` does: the top `damage` segments of the level broken
  (`damageRed` 255,50,50 with a dark crack, so a level an edit removes
  stays plain red), FTL's `COLOR_BLUE_ION` outline around the segments
  while ion locks them (white for -1), and in the name column the broken
  count and the lock `img/icons/locking/s_lock.png`. A right click on a
  segment breaks it and those above or mends it and those below; a click
  on the lock adds an ion up to 5, then clears it, a right click opens the
  value popover; a segment whose damage an edit changes has a yellow edge
  and the number stays the level's; the shields row carries the Zoltan
  shield above its segments as FTL lays it over the bubbles
  (`_ZoltanBars`, `zoltanGreen`: a bar per point, outlines up to the max,
  a click sets that many, the top bar one less, a right click the value
  popover; shots `test/zoltan_shots_test.dart`); Add augment picks from
  `augment_choices` and the editor asks `augment_field` for the new path.
  Weapons, drones and cargo show `equipment_preview` of the pending edits
  (the editor stages the latest edit last, because the core applies them in
  order; an assignment that fails waits for a later one that makes it
  possible, so pending installs and removals apply in any click order,
  `Patch::apply_in`); each row sets a count path within the preview's range, Add opens
  the same picker with images (a dialog takes `GameDataScope` along).
  Weapons, drones, cargo and augments are fixed areas: a row per slot, a
  free slot an empty row (`equip-empty-<kind>-<n>`), Add a plus icon button
  with its label as tooltip on the right of the section heading, every row
  action an icon button with a tooltip in one right-aligned column (an
  augment is a row per copy). `ItemSlots` owns the weapon and drone rows,
  empty rows, Add and the slot chooser, for the page, the Weapon Control and
  Drone Control popovers and the hull mounts. A click on a slot, empty or filled, opens the
  one chooser of its list (`ChoicePicker`: "Empty slot" first as
  `<prefix>-choice-empty`, then the blueprints with `GameIcon` art; a
  filled weapon or drone slot adds Power on/off, `item-power`); a pick
  on a filled slot stages the removal, then the addition, as one step of
  Undo (`EquipmentPage.group`, `EditorPage._group`), and the core keeps
  the slot. The enemy ship gets the same lists without the cargo. When the
  systems panel is narrower than its widest row
  (`SystemLevelRow.wideWidth`), every row is `stacked`: segments and
  Install/Remove on a line under the name at full size; a `TitleRow` with a
  note gives the title its whole width first.
  Names follow the app language (`de` reads `text-de.xml`). Text in the
  FTL look is clipped only at its sides (`FtlSideClip`): the dots of Ü stand
  above the line box of the bitmap font.
- One-click actions: `ftl_core::actions` plans `max-everything`,
  `crew-fully-trained`, `repair-ship` and `heal-crew` as a `Patch` from the
  game and `ShipLimits` (hull and system caps of the ship blueprint);
  `ftl_app::actions::limits` fills them from `ftl.dat`. An action only
  improves the ship and lists only changed fields; `repair-ship` puts out
  every fire (`fires.<x>.<y>`), seals every breach
  (`breaches.<x>.<y>`) and refills every room (`rooms.<n>.oxygen=100`, as
  `OxygenSystem::Restart`), mends every broken bar and clears every ion
  lock with a count (`systems.<id>.damage=0`, `systems.<id>.ion=0`).
  `ActionNote`s (stable `code()`) name an editor cap it fell back to or
  undecoded rooms. `ftled action <name>` and the
  bridge (`api/actions.rs`, `plan_action`) call it. The app shows the
  actions as icon buttons in the top bar left of Expert (`QuickActionBar`,
  keys `action-<id>`), because an action changes several areas; their
  edits join the pending edits and the change sheet opens with the notes.
  `SaveFile::open` opens the located `ftl.dat` as the shared game data for
  the layouts, so the bridge plans with the real blueprint even while the
  app shows the reduced mode; tests read the expected values from
  `plan_action` instead of assuming the editor caps.
- The sector map: the file stores no beacon position and no fleet circle.
  `ftl_core::sector_map` builds both as FTL 1.6.13 does (`GenerateMap`,
  `PopulateGrid`, `random32`; Ghidra evidence in the module doc): positions
  from the layout seed, rebuilt until the joins (neighbor cells, at most
  165 px) reach every beacon. Java's `RandomSectorMapGenerator` guesses
  that test (isolation over 165) and differs on 85 of the seeds 1 to 3000;
  the Java oracle fixes the positions where both agree. The circle is all
  within 767 px of (offset, fudge); the fleet flags of the first fixture
  (sector 7, replaced on 2026-11-19) proved it; the current ones (sector 1
  and sector 3) have none, so `tests/sector_map.rs` moves the fleet
  forward first. A
  header with `random_native` (the OS RNG) has no positions (`NoLayout`).
  Edits (`ftl_core::beacons`): `fleet.offset` (-959 to 60; sets every
  beacon's fleet flag as `SaveGame` would; refused in the Last Stand and
  without positions), `beacons.<n>.visited` (1 to 0 only, never the
  current beacon), `beacons.<n>.store.<shelf>.<slot>` (0 sold out, 1 for
  sale) and `beacons.<n>.store.<fuel|missiles|drone_parts>` (0 to 99).
  Prices are not in the file. `beacons.current` moves the ship inside
  the sector as an arrival without a ship (no nearby ship, encounter
  event, projectiles or hazard; visit count at least 2, so FTL clears the
  beacon's event); refused in the Last Stand, with crew aboard the other
  ship, at a fleet or quest beacon (`beacons::MoveBlock`, stable codes).
  `beacons.<n>.store` (0 or 1) adds `beacons::new_store` or removes the
  stored store; `beacons.<n>.store.<shelf>.<slot>.item` (text) swaps a
  slot to a blueprint of the shelf kind (port `weapon`, `drone`,
  `is_augment`; races; systems). The `beacons` module doc holds the
  evidence: only the current sector has beacons in the file (the other
  sectors are the tree, see below), a stored
  stock is never generated from the seed, and quest, distress, nebula and
  other events are not stored. `ftl_app::map::sector_view`, `ftled map
  list|set`, the bridge `sector_map_of`/`sector_map_preview` (edits and
  texts); the Map page (`app/lib/ui/map_page.dart`) fills the page: the
  700 x 470 drawing scales to fit over `map/zone_<sector % 3 + 1>.png`
  tinted 0.75, a `MapToolbar` in the frame (`fleet-start`, `fleet-back`,
  `fleet-forward`, 64 px per jump), FTL's map icons, the zone from
  `map_warningcircle_tile`/`_point` at offset + 586, and a popover per
  beacon (`map-beacon-<n>`: `beacon-move-<n>`, `beacon-store-<n>`,
  `store-swap-<item path>` with the equipment's `ChoicePicker`). An open
  popover reads the current preview (`MapPage.previewOf`). The editor
  takes the fields of a store the edits add from a preview with only those
  edits (`_mapBase`). Java's `readStoreShelf` drops an empty middle slot,
  so its round trip of such a store differs; FTL keeps the slot.
- The sector tree (`ftl_core::sector_tree`; Ghidra evidence of
  `GenerateSectorMap`, `AddSectorColumn`, `GetSectorDescription`,
  `RenderSectorMap` and the arrival in its module doc): built from the tree
  seed and `data/sector_data.xml` (`ftl_assets::SectorData`, port
  `ShipLayouts::sector_catalog`, names by `sector_text_in`). It matches the
  Java oracle (`RandomSectorTreeGenerator`, types and ids; Java names the
  first sector `CIVILIAN_SECTOR`, FTL `STANDARD_SPACE`) and the visited
  flags of an FTL save form a route through it. `sector.jump` (a tree
  index) writes an arrival: route flags, world and header sector, a new
  layout seed (`random32` after the old one), GenerateMap's beacons with
  the ship on the start beacon (`SectorLayout::start`, two visits, the old
  background), fleet -959; refused in or into the Last Stand, in the hidden
  sector, with crew aboard the other ship, with distant quests
  (`JumpBlock`, stable codes). `ftl_app::map::sector_tree_view`, `ftled map
  tree`, the bridge `sector_tree_of`/`sector_tree_preview`; the Map page
  switches Beacons / Sector tree (`map-view-*`, `ui/sector_tree.dart`:
  `sector-<n>`, `sector-jump-<n>`). A staged jump shows the new sector's
  beacon map read only and drops pending edits of the old map. Jumped
  files pass the Java oracle IDENTICAL (`crates/ftl-app/tests/real_sectors.rs`
  writes `fe-sectors-jump-*.sav` to the test tmp dir).
- The identity of a run (`ftl_core::identity`; its module doc holds the
  FTL 1.6.13 evidence and the design note of a full type change):
  `ship.name` and `nearby.ship.name` (text, the crew name rule; the
  player's header copy too), `ship.type` (text, player ship only: a
  blueprint of the same layout file whose system rooms match, whose start
  systems are installed and whose slots hold the items; writes id, header
  id and `gfx_base_name` from the port's `ship_gfx`; `TypeBlock`, stable
  `code()`), `difficulty` (0 to 2, Hard from format 7; refused while the
  rebel flagship is on the map, because `GenerateMap` rebuilds the Last
  Stand's beacons by difficulty) and `sector` (read only alone:
  `EditError::SectorFixed`; a jump is `sector.jump`). No two hangar ships share a
  layout, so in 1.6.13 every hangar type is refused (`other_layout`); only
  the unused `PLAYER_SHIP_EASY` fits Kestrel A. Below format 3 the stored
  difficulty flag is inverted (0 Normal, 1 Easy). The bridge is
  `api/identity.rs` (`identity_of`, `ship_type_choices`); the ship card
  (`app/lib/ui/ship_card.dart`) shows Rename (`ship-rename`), the type,
  sector and difficulty tags (`ship-type`, `ship-sector`,
  `ship-difficulty`) and their pickers; the difficulty is staged outside
  `_fields` (no palette entry), and `identityChangeRows` names the edits.
- Cmd+K (Ctrl+K off macOS) opens the command palette (`ui/command_palette.dart`)
  over the areas, every field of `_fields` (label from `fieldLook`, its
  localized kind from `fieldKind` as detail; the path is only a search key)
  and the actions; every word of the query must occur. Enter switches
  to the area of the entry, then a field opens its value popover over the
  page and an action runs. `AppShell` takes `actions` and `palette`.
- The Expert area: `ftl_core::raw` serializes the `SavedGame` through its
  serde derives into a tree (no list per field; the model derives
  `Deserialize` for the way back) and answers every decoded field as a
  `RawEntry` (dotted serde path, `RawKind`, value, range of its type,
  `unknown` for `unknown_*`). `raw.<path>` (`Field::Raw`) edits a number
  (`i32`, `u32` up to `i32::MAX`) or a flag; texts, enum variants and list
  lengths stay, so only the verify step of a save guards what FTL accepts
  (`raw.file_format=2` is refused there). `state.<id>` (`Field::StateVar`)
  edits a state var by name. `SavedGame::undecoded` is shown as hex
  (`raw::hex_lines`), never edited. `ftled dump [--json]` prints the tree;
  the bridge is `api/expert.rs` (`expert_of`). The three items of the Expert
  menu show `ExpertPage` (`app/lib/ui/expert_page.dart`, `ExpertView`): the
  folded tree, the state vars, the unknown fields with the hex; a search
  lists matches flat; a number opens the value popover and its edit joins
  `_fields` through `EditorPage._rawFields`; `expertLook` names it in the
  change sheet ("Raw: <path>", "State var: <id>"). `TextRole.code` is the
  monospace role of the hex. Shots: `test/expert_shots_test.dart` (helpers
  in `test/shots.dart`).
- Undo and Redo (`app/lib/ui/edit_history.dart`, `EditHistory`): the
  editor page records a snapshot of `_edits` and `_texts` before every
  change that alters them (`_change`: a field, a text, a quick action as
  one step, an undo row of the change sheet); `ProfileEdits` keeps its own
  steps (Unlock all as one, `group`). A newly read file (open, save,
  restore) clears them. The shell shows `UndoRedoBar` (keys `undo-button`,
  `redo-button`) left of the quick actions and routes them by area like
  Save; Cmd+Z and Shift+Cmd+Z (Ctrl off macOS) press them unless an
  `EditableText` has the focus, so a text field keeps its own undo.
- Game icons of a field (`app/lib/ui/field_image.dart`): `fieldImage`
  maps a path to its image in `ftl.dat` (resources `resourceImage`,
  systems `s_<id>_overlay`, skills `skillIcon`, crew fields the portrait,
  weapons and drones their blueprint art, fires `s_fire2`); `FieldIcon`
  draws it and the Material icon in reduced mode. A game image beside
  Material icons (a `ListRow` leading, the square menu) is a `GameIcon`
  (`game_image.dart`): cropped to its pixels with alpha at least 0x80 (no
  glow, no empty cell margin) and drawn at a whole factor of the icon size,
  art up to 1.5 times that size at factor 1. The change sheet
  (`ChangeRow.image`), the value popover (`showResourcePopover(image:)`)
  and the palette (`PaletteEntry.image`) use it; `EditorPage._image`
  passes the crew and equipment of the field's ship.
- Each row of the crew grid is one `IntrinsicHeight` row: every card, the
  plus card too, is as high as the tallest. A card shows its states (new,
  removed, a pending race) through marks and outline only, never through
  an extra line.
- The macOS window opens at least 1280 x 860 and keeps 1100 x 700
  (`MainFlutterWindow.swift`): below that the actions of the top bar cover
  the area tabs.
- A page's hint (`HintRow`) sits in `HintedList` (`ship_page.dart`): at
  the bottom of the visible area while the content is shorter.
- Text never relies on U+2192: the fonts of both looks lack it. The change
  sheet draws old and new value around an arrow icon (`ChangeValues`).
- The floorplan: `ftl_app::floorplan` places hull, floor, rooms, system
  interiors, doors, burning squares, breaches and crew in game pixels, as
  `SavedGameFloorplanPanel.addBundle` does (the mapping is in the module
  doc), moved so the drawing starts at 0, 0. An interior is the blueprint
  `img`, else `room_<system>` when `ftl.dat` has it (the Java editor draws
  only `img`). The bridge is `api/floorplan.rs` (`floorplan_of`); the app
  asks for it only after it opened `ftl.dat`, because `SaveFile::open` loads
  the game data for the layouts on its own. The Ship page draws it in
  `app/lib/ui/floorplan/` at a whole scale up to 2: the system symbol of a
  room opens a popover with the `SystemLevelRow` of the Equipment page and
  Put out / Seal per square; every square (`floor-square-<room>-<id>`)
  opens a menu anchored under it (`showAnchoredPopover` of the components)
  with Light fire / Put out fire, Add breach / Seal breach and an entry per
  crew member standing there (the core answers each square with its paths
  and crew); a crew member opens the `CrewCard` when they are all their
  square holds, else the menu. Fires and breaches are drawn from the pending
  values, so an added one looks like one of the file; a square with a
  pending fire or breach carries a `ChangeMark`. Cmd+K lists only the fires
  and breaches of the file (`isQuietSquare`). A dialog route lies above the
  page, so a popover takes `GameDataScope` along, like `showResourcePopover`
  takes the theme. Fire and breach frames: first frame of `fire_large`,
  last of `breach` (`data/animations.xml`). FTL draws every room floor
  itself: a tile per square, a grid line between squares, walls 4 px on the
  room edges (colors of the Java editor: `E6E2DB`, `7D7D7D`, `0F0F0F`).
  The system symbol on a floor is `s_<id>_overlay.png` multiplied with the
  state color of `ShipSystem::OnRender` (`floorSymbolColor`: gray `7D7D7D`,
  ion blue, red, orange; docs/original-look.md), never drawn white.
  `<gfx>_floor.png` only lies below and can leave room squares open (six on
  the Engi cruiser, two on the Kestrel); without the tiles the hull shows through there.
  The tile color is the room's oxygen as `Room::OnRenderFloor` tints it
  (`oxygenTint`, `E6E2DB` at 100), with `low_o2_stripes` at 5 % or less
  (docs/original-look.md); it follows the preview of the pending edits.
  `PlanRoom` carries `oxygen` and `oxygen_field`; the square menu
  (`square-oxygen`) and the room popover (`oxygen-set-<room>`) show the
  percentage and open the value popover; a pending edit carries
  `oxygen-mark-<room>` in the room's top left corner. The floor symbol and
  the room mark follow staged level, damage and ion edits.
  Weapons on the hull: weapon slot `n` sits on mount `n` of
  `<weaponMounts>` (`ShipLayout::weapon_mounts`, up to the blueprint's
  slots, none without a weapons system). `floorplan::place_weapon` turns,
  mirrors and slides the first frame about its `<mountPoint>` as FTL
  1.6.13's `WeaponAnimation::OnRender` does (Ghidra evidence in its doc):
  an unpowered weapon sits half its box back under the hull, which FTL
  draws over the weapons. The app draws the mounts of the preview
  (`floor-weapon-<slot>`), each a click target (`floor-mount-<slot>`) that
  opens `ItemSlots.openSlot`; the pointer outlines a mount, so an empty one
  shows; a changed mount carries `mount-mark-<slot>`. An open popover
  reads the preview through a getter, never a value captured at open, and
  builds again a frame after each edit.
- The resources of the Ship page in the FTL look are one `FtlBox` of
  `ftlResourceSize` each (icon and number); FTL's `top_*` status boxes
  differ in size and stay in the look mock only. With `ftl.dat` open the
  icon is FTL's own (`resourceImage`: `img/ui_icons/icon_*`, the hull the
  map marker `img/map/map_icon_ship.png`; docs/original-look.md), drawn at
  factor 1 (`GameImage.fit` `BoxFit.none`, which crops) and tinted with the
  text color; Material icons only in reduced mode.
- The profile codec (`ftl-sav/src/profile/`) follows the same contract. It
  decodes every field of formats 9 and 4; `Profile::undecoded` stays empty
  in every observed file. Its fixtures live in `tests/fixtures/profile/`, apart
  from the saves, because each round-trip test reads every `.sav` of its
  folder. The layout reference is `ProfileParser.java`. Format 9 stores three
  layout flags after each `PLAYER_SHIP_*_VICTORY` achievement only.
- Type-B of a ship has no flag in the profile: FTL unlocks it from two of
  the ship's three achievements (`ftl_core::profile::PLAYER_SHIPS`).
- Profile edits are paths in `ftl_core::profile` (`ProfilePatch`, all or
  none): `ships.<SHIP_ID>.<a|b|c>` (0 or 1) and `achievements.<ID>` (0
  removes, 1 Easy, 2 Normal, 3 Hard). `ships.<ID>.b=1` adds missing ship
  achievements on Easy until two are present, `=0` removes them until one
  is left; an added ship victory counts as won with Type-A (format 9 stores
  its layout flags). Type-C needs format 9 and a ship with one; quests and
  victories need format 9. "Unlock all" is `unlock_all_patch`. `ftled
  profile set|fields` and the bridge (`preview`, `save` with the edits,
  `unlock_all_edits`) call `ProfileFile::preview` and `apply`.
- binrw passes a struct field as a reference when it writes. An attribute that
  reads another field needs separate `#[br(...)]` and `#[bw(...)]` forms with
  `*field` in the write form.

## Commands

```sh
bun scripts/codegen.ts                      # the generated files (see "Generated files"); --install builds the pinned codegen
cargo test                                  # core, codecs, app
cargo clippy --all-targets
cargo fmt --all --check                    # one rustfmt baseline (rustfmt.toml)
cargo run -p ftled -- --help              # ftled
cd app && flutter analyze
cd app && flutter test test                         # Dart unit tests
bun scripts/integration-test.ts                    # real app + real Rust core; one run per machine (lock)
bun scripts/integration-test.ts integration_test/actions_test.dart   # only the files named
cd app && flutter run -d macos
```

`rustfmt.toml` holds the one Rust baseline; `cargo fmt --all` formats and
the gate checks it. rustfmt skips the generated `frb_generated.rs`
(`#[rustfmt::skip]` in `app/rust/src/lib.rs`), so the check passes with or
without the file and no `cargo fmt` follows the codegen. Dart uses width
120: `dart format -l 120 <file>` (the default 80 reflows lines nobody
changed).

While a change is under work, run only the tests that cover it:
`cargo test -p <crate> <filter>`, `flutter test test/<file>.dart`,
`bun scripts/integration-test.ts integration_test/<file>.dart`. A full
integration run takes minutes and holds the machine lock.

Gate once before a pull request: `cargo test`, `cargo
clippy --all-targets`, `cargo fmt --all --check` and `flutter analyze`
without findings, `flutter test test` and the full integration test green.
A Dart-only change skips the cargo steps.

Oracle check against [Vhati's Java editor](https://github.com/Vhati/ftl-profile-editor)
(needs FTL installed, Java 21 or later and Maven):

```sh
bun scripts/oracle.ts <file.sav>                     # SaveRoundTrip
bun scripts/oracle.ts --profile [<ae_prof.sav>]      # ProfileRoundTrip
```

The script (its `--help` is the contract) checks out the submodule
`reference/ftl-profile-editor` at its pinned upstream commit, applies
`reference/patches/*.patch` idempotently (0001: Java 21 build and
`SaveRoundTrip`, 0002: `ProfileRoundTrip`; upstream has the parsers they
call), builds with `mvn -q -B package -DskipTests` and runs the probe with
the FTL resources the app finds (or `--resources <dir>`). The probe prints
the resources it reads and `IDENTICAL` or `DIFFER`; the exit code is its
verdict. A file written by `ftled` must pass it. The profile probe also
lists the player ships with their achievements and B/C layouts. After a run
the submodule shows as modified (`m`): the applied patches and `target/`
are local, never committed. A new probe is a new patch over the pinned
commit: `git diff` in the submodule, written to `reference/patches/` with
a one-line `#` header (git apply skips text before the first `diff`).

## Generated files

`app/rust/src/frb_generated.rs`, `app/lib/src/rust/` (flutter_rust_bridge)
and `app/lib/l10n/app_localizations*.dart` (`flutter gen-l10n`) are
generated and gitignored; never edit them by hand. `bun scripts/codegen.ts`
(its `--help` is the contract) writes all of them:
`flutter_rust_bridge_codegen generate`, then `flutter gen-l10n`, both in
`app/`. Run it after a clone and after a change in `app/rust/src/api` or an
ARB file; `flutter pub get` regenerates the localizations as well
(`generate: true` in `app/pubspec.yaml`). The codegen version is pinned:
`flutter_rust_bridge` in `Cargo.lock` and `app/pubspec.lock` must agree and
the installed `flutter_rust_bridge_codegen --version` must answer it, else
the script refuses; `--install` runs `cargo install
flutter_rust_bridge_codegen --version <pin> --locked`. The codegen needs Rust
and Flutter. By hand: `cd app && flutter_rust_bridge_codegen generate &&
flutter gen-l10n`.

Each build path generates them before it needs them:
`scripts/integration-test.ts` runs the script before the lock,
`scripts/build-dmg.ts` in its export (or unpacks `--generated <tar>`), CI
in `codegen.yml` once for every job ("Continuous integration").
`scripts/export-public.ts` exports `git archive`, so the public tree carries
none of them either.

`Cargo.lock` and `app/pubspec.lock` stay in git: they pin the dependencies of
an application, and every build runs with `--locked` and
`--enforce-lockfile`.

## Platform notes

- The macOS app runs without App Sandbox (`macos/Runner/*.entitlements`). The
  sandbox blocks `~/Library/Application Support/FasterThanLight`.
- `app/rust/rust-toolchain.toml` pins an exact Rust version (required by
  `native_toolchain_rust`) and the six desktop targets.
- A failing Xcode build that names `DVTDownloads` or `IDESimulatorFoundation`
  needs `xcodebuild -runFirstLaunch` once after an Xcode update.
- FTL deletes `continue.sav` when a run ends and writes it when the game
  quits. Close FTL before a save; `ftl-app` refuses a save when the file
  changed since it was opened.
- The file dialog is `file_selector` (first-party, native on all three
  desktops). One button chooses and loads a file; there is no path field and
  no reload button. The dialog opens in the folder of the current file, else
  in FTL's save folder (`app/lib/ui/save_location.dart`). The last file that
  opened without error lives in `shared_preferences` and opens again at
  start. When the window gets the focus back and no field is edited, the page
  reads the file again. Tests replace the dialog through `EditorPage.pickFile`.
  The file name of the status bar opens the same chooser (`_choose`), its
  game data item the `ftl.dat` chooser (`_pickFtlDat`, remembered through
  `rememberFtlDat`); a clickable `StatusItem` sets the click cursor itself,
  because `InkWell` defaults to `adaptiveClickable`, the arrow on desktop.
- User texts in the app go through `app/lib/l10n/*.arb` (English template,
  German). An error reaches Dart as `EditorError` with the stable code of
  `AppError::code`; `app/lib/ui/error_text.dart` maps each code to an ARB
  text and keeps the English core message only as detail. A new `AppError`
  variant needs a code, an ARB text in both files and a case there.
- Backups: a save copies the old bytes to `<stem>_<time>[_<n>].sav` in the
  backup folder. `ftl_app::list_backups(dir, save)` lists them newest first;
  `SaveFile::restore` and `ProfileFile::restore` write one through the save
  path (`restore_checked` in `backups.rs`: the backup must decode and encode
  back to its bytes, the changed-on-disk check, a backup of the current file
  first). `ftled backups list|restore` (`--profile` for `ae_prof.sav`) and
  the app's histories (the backup item of the status bar for the save, the
  Backups button of the Profile page) call them. The app shows every change
  in the change sheet (`ChangeRow`s) before it writes, every edit with an
  undo per row. Tests and probes
  pass their own backup folder (`--backup-dir`, `EditorPage.backupDir`).
- The Profile page (`app/lib/ui/profile_page.dart`) opens `ae_prof.sav` on
  its own (`ProfileView`, preference `lastProfileFile`); the status bar keeps
  the saved game. A missing profile is `AppError::NoProfile` (`no_profile`).
  The bridge is `app/rust/src/api/profile.rs`; ship and achievement names
  come from the open `ftl.dat` in the app language (`achievement_catalog`)
  and are read again when it opens. A click on a layout lock, an
  achievement (difficulty, remove), Add achievement or "Unlock all ships"
  stages an edit in `ProfileEdits`, which `EditorPage` owns, so the edits
  outlive a tab visit; the page shows the core's `preview` with the pending
  look (accent frame). Save in the top bar follows the area
  (`AppShell.onSaveProfile` on the Profile page) and opens the change sheet.
  A dialog of the page takes `GameDataScope` along (`_showGameDialog`).
- `ftl_app::warnings` answers `Warning`s that do not stop a save, each with a
  stable `code()`: `ftl_running` (a process named `FTL`, `FTLGame.exe`,
  `FTL.amd64` or `FTL.x86`, through the port `ProcessList`), `steam_cloud`
  (`steam_autocloud.vdf` next to the save file) and
  `process_list_unavailable`. `ftled show` and `ftled set` print them to
  stderr; `set` still writes. The app reads them through
  `ftl_bridge::api::status::save_warnings` at open, on focus and before a
  save (`SaveGuard.of` maps the codes for the status bar); a running FTL
  asks before the save with Cancel as default; the Profile page asks the
  same before Unlock all and a restore. Tests inject
  `EditorPage.warningsOf`, so a user's running FTL never reaches them.

## Continuous integration

GitHub Actions on the public repository (`.github/workflows/`); both pin
Rust 1.99.0 (as `app/rust/rust-toolchain.toml`, clippy lints change per
version), Flutter 3.50.0 stable and every action to an exact version.

- `codegen.yml`, a reusable workflow that both call first: on Ubuntu
  `bun scripts/codegen.ts --install --pack generated.tar` (the pinned codegen,
  `~/.cargo/bin` cached by rust-cache) and the artifact `generated`. The
  output is platform independent; every other job needs it and unpacks the
  tar (`tar -xf`, on Windows through `cygpath -u`) into its checkout or
  export.
- `ci.yml`, every push and pull request to `main`: `cargo fmt --all
  --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on
  Ubuntu, Windows and macOS; `flutter analyze` and `flutter test test` on
  Ubuntu. The integration tests drive a real window and stay the local gate.
  The runners have no `ftl.dat`: a Rust test passes with and without one. A
  test that needs the decoded world opens the fixture with the fake layout
  port (`open_with_layouts`; in the bridge `open_fixture`), a test that needs
  the real game data skips with a `skip:` line. Check that locally with
  `HOME=<empty dir> cargo test` (keep `CARGO_HOME` and `RUSTUP_HOME`).
- `release.yml`, a pushed tag `v<version>` or a manual run with input
  `tag`: builds on ubuntu, windows and macos-latest and attaches to the
  release of the tag (created for a pushed tag when missing):
  `FTL-Save-Editor-linux-x64.tar.gz` (the Flutter bundle),
  `-windows-x64.zip` (the runner `Release` folder plus `msvcp140.dll`,
  `vcruntime140.dll`, `vcruntime140_1.dll`, because the app and the bridge
  link the MSVC runtime dynamically), `-macos-universal.dmg`
  (`scripts/build-dmg.ts`, renamed: Flutter and the bridge build arm64 and
  x86_64), `ftled-<os>-<arch>` archives (macOS: a `lipo` of both targets)
  and `SHA256SUMS.txt`. The names carry no version: the README links
  `releases/latest/download/<asset>`, which stay valid for each release.
  A manual run takes `dry_run` to build and scan without an upload.
- crates.io: the crates publish as `ftled-core`, `ftled-sav`,
  `ftled-assets`, `ftled-app` and `ftled` (the CLI; `ftl-core` and
  `ftl-cli` belong to others there). The folders and the crate names in
  code stay (`ftl_core`, ...): each lib sets `[lib] name`, the workspace
  dependencies rename with `package =`. A package ships `src/` only
  (`include`); tests and fixtures stay here. The job `publish-crates` of
  `release.yml` publishes them in that order after the builds and skips a
  version crates.io has; auth is Trusted Publishing, else the secret
  `CARGO_REGISTRY_TOKEN` (the first publish of a crate). Check before a
  release: `cargo publish --dry-run -p ftled-core -p ftled-sav -p
  ftled-assets -p ftled-app -p ftled`. The bridge `app/rust` stays
  `publish = false`. A manual run builds the ref it runs
  on (default `main`). Public binaries come from this workflow only, never
  from a maintainer machine.
- Build paths: a binary keeps the absolute paths of its sources (Rust
  panic locations name the cargo registry and rustup, the Dart AOT snapshot
  the file URI of `dart_plugin_registrant.dart`, Swift and C objects
  `__FILE__` and their object paths, an exe its PDB path). So every release
  build runs in a `git archive` export under a neutral folder (`/tmp/...`,
  `C:/ftl-editor-ci`, macOS `$TMPDIR`) with its own `PUB_CACHE`;
  `app/hook/build.dart` passes `--remap-path-prefix` for cargo home, rustup
  home and repository to the bridge build (stable Cargo has no
  `trim-paths`), the workflow the same as `RUSTFLAGS` to `ftled`. Then
  `bun scripts/scan-binaries.ts <path>...` (its `--help` is the contract)
  scans every file of each artifact, single-byte and UTF-16LE, for the home
  folder, the checkout, `/Users/`, `C:\Users\` and the maintainer's names;
  the workflow scans the staged folders, the mounted DMG and, before the
  upload, the downloaded archives again. Both refuse when `app/pubspec.yaml` does not carry the
  tag's version. An upload never replaces an asset (`gh release upload`
  without `--clobber`), so a second run for the same tag fails at the upload.
- Linux needs the GTK desktop packages (`clang cmake ninja-build pkg-config
  libgtk-3-dev liblzma-dev libstdc++-12-dev`); Native Assets builds the
  bridge through `native_toolchain_rust` on every OS.
- FTL's save folder per OS (`ftl_app::default_save_dir`): macOS
  `~/Library/Application Support/FasterThanLight`, Linux (native build)
  `~/.local/share/FasterThanLight`, Windows `Documents\My Games\FasterThanLight`.
  Only the macOS path is checked against an installed FTL; the other two
  follow the Java editor and the PCGamingWiki "Save game data location".

## Libraries

| Need | Library | Status |
|---|---|---|
| Binary codec, read and write from one declaration | `binrw` | in use |
| Flutter bridge | `flutter_rust_bridge` 2.13, native-assets backend | in use |
| CLI | `clap` derive | in use |
| OS folders | `directories`; timestamps `jiff` | in use |
| Process list (running FTL) | `sysinfo`, feature `system` only | in use |
| `ftl.dat` (PKG since FTL 1.6.1, DAT before) | `ftldat` 0.1.1 (itb-community/ftldat-rs, GPL-3.0) | in use |
| Blueprint XML inside `ftl.dat` | `quick-xml` 0.42 with serde, feature `overlapped-lists` | in use |
| Sprite frames (decode, crop, encode PNG) | `image` 0.25, feature `png` only | in use |

## Game data (`ftl-assets`)

- `GameData::open` reads the package once and drops it: `ftldat::Package`
  holds an `Rc` to a memory map and is not `Send`. Only the PNGs the
  accessors answer are copied (crew `_base` plus `female`, system `_overlay`,
  weapon strips; `WeaponArt::mount_point` is the `<mountPoint>` of the
  `weaponAnim`). `crew_frame` and `weapon_frame` cut one frame with the
  `image` crate (crew: 35x35 at 0,0, as `DefaultSpriteImageProvider`; ghost
  uses the human sheet) and answer PNG bytes. `crew_look` and
  `crew_look_frame` draw a look as `CrewAnimation::SetupStrips` does: the
  race sheet (`female` for a female human) with each
  `<sheet>_layer<k>.png` multiplied by its `colorList` color. FTL anchors the cell, not the
  art, and the art sits up to 2 px off the cell's middle (Lanius left), so
  the crew card takes `crew_portrait` (`ImageId::CrewPortrait`): the same
  frame with its opaque pixels centered; the floorplan keeps `crew_frame`.
- ftldat panics on some corrupt packages and refuses deflated PKG entries
  (FTL 1.6.13 has none). `open` catches the panic and returns an error.
- The XML repairs (no single root, `<FTL>` root, stray text, repeated
  elements) are listed at the top of `crates/ftl-assets/src/xml.rs`. A mixed
  starting crew repeats `crewCount`, once per race.
- `ftl_app::assets::AssetsConfig` keeps the confirmed `ftl.dat` path in
  `config.json` of the editor's OS config folder (`ProjectDirs`). `locate`
  tries it first, then the candidates of `FTLUtilities.findFtlDatsDir`, and
  says whether the path was remembered or only found. Tests inject a temp
  folder (`AssetsConfig::in_dir`); a CLI probe of `--remember` sets `HOME`
  to a scratch folder.
- Names in German: `GameData::display_name_in(id, "de")` answers the text of
  `data/text-de.xml` (same text names as `text_*.xml`, read at open for the
  blueprints that name their title by id), else the English name. Weapon and
  drone blueprints carry their `power`.
- `GameData` keeps only the sprites above. Ship hulls, drones and
  achievements are read on demand with `ftl_assets::read_entry`, which opens
  the package per call; `ship_image_paths` and `drone_image_paths` answer
  candidates in order (`img/ship` then `img/ships_glow`; `_base` then `_off`,
  the shield drone has no `_base`). Augments have no image in `ftl.dat`.
- Achievements: `GameData::achievements` answers `achievements.xml` (51 in
  1.6.13) plus the ship quests and victories FTL hard-codes (`QUEST_SHIP_IDS`,
  as `DefaultDataManager`), with name and description (`achievement_text_in`
  for `de`). A quest or victory takes the hangar marker `S_Q_on` / `S_V_on`
  as image and `quest_ach_title` / `victory_ach_title` with the ship class
  as name; without this their tiles showed the fallback icon.
- `ftl_app::assets::GameAssets` holds one `Arc<GameData>` and an LRU cache of
  128 images (by inner path or `ImageId`); a hit does not open the package.
  `open_shared` keeps one per process, so a save can share its `GameData`.
  Tests count opens through the `Archive` seam. `dat_in` resolves a picked
  folder or `FTL.app` (a macOS dialog does not open the bundle) to `ftl.dat`.
- The app opens a remembered `ftl.dat` at start; a found one waits for
  "Use this ftl.dat", none shows "Choose ftl.dat". Until then it runs in
  reduced mode: `GameImage` shows its Material icon and the ship card says
  "Reduced mode". `GameDataScope` above the page tells every `GameImage`
  which `ftl.dat` is open; `GameImageProvider` keys the Flutter image cache
  by that path and the image. Integration tests pass `assetsConfigDir` and
  `locateFtlDat`, so they never write the real editor config, and wait for a
  Rust future with real time (`pumpAndSettle` does not see it).
- `ftled assets ls [prefix]` lists the inner paths of `ftl.dat`, `ftled
  assets cat <inner> -o <file>` writes one entry to a file outside the repo:
  the tools for evidence about the package.
- The original look (`docs/original-look.md`): `ftl_assets::BitmapFont`
  decodes `fonts/*.font` (header, 16-byte glyph records, a `TEX` block of
  1-bit coverage); `measure_insets` measures the 9-slice insets of a frame
  image; `GameAssets::font` and `insets` answer both from the cache, the
  bridge as `game_font` and `game_image_insets`. `app/lib/look/` draws text,
  frames and buttons from them; the measured colors are `FtlColors`. The
  gallery shot of the look loads the real `ftl.dat` through a host build of
  the bridge (`cargo build -p ftl_bridge`), because `flutter test` builds no
  native assets.
- The app wears the look when `ftl.dat` is open: `EditorPage` loads an
  `FtlLook` with the game data and puts `ftlLookTheme(look)` above the page,
  a `ThemeData` of the measured colors that carries the look as the
  extension `FtlTheme`. `FtlTheme.of(context)` `null` is the Material look
  (reduced mode, or a look that cannot be read; the status bar names why).
  `showDialog` and `showModalBottomSheet` carry the theme into their routes;
  `showGeneralDialog` does not, so `showAnchoredPopover` captures it.
  `GameDataScope` is an `InheritedTheme`, so both carry the open `ftl.dat`
  into a dialog route as well; the explicit wraps that predate it
  (`withScope`, `_showGameDialog`, the floorplan and map popovers) are
  redundant.
- Pages build only from the components in `app/lib/ui/components/`
  (`AppText`, `Panel`, `TitleRow`, `AppDialog`, `AppPopover`, `AppButton`,
  `ListRow`, `Tag`); each draws itself in either look. Both themes carry
  `AppTokens` (text roles title, label, body, caption, number, display with
  their bitmap font, colors, panel, tile and dialog padding, row heights);
  `Space` is the spacing scale. A page never names a font, a text style or
  a padding number: `test/no_own_fonts_test.dart` fails on it. `AppText`
  draws bitmap text unless a sign has no glyph, then system text of the same
  role; the middle dot is the full stop raised. `FtlText` centers the
  capitals in its line box (the fonts' own baseline put text too high), can
  wrap, and cuts a too-wide line with `..`. Bitmap text scales by whole
  numbers only. Every text field is `AppTextField` (filters, rename, exact
  values, Cmd+K): typed text and hint in `TextRole.code`, the monospace
  system font, because the bitmap fonts draw no cursor;
  the guard test fails on a bare `TextField` in a page. `AppButton` passes its key to the button it draws. Tests find
  a text of either look with `findLabel` (`test/fake_look.dart`) and the
  controls by keys (`area-<name>`, `save-button`, `choose-file`); widget
  tests of the look use `fakeLookSource`, never game data.
- Never commit XML or images extracted from the real `ftl.dat`. The tests pack
  their own fixture at run time; `tests/real.rs` reads the installed game
  (`FTL_DAT` overrides the path) and prints a skip line without it.

## Brand, icons and the DMG

- `brand/logo.svg` is the active logo and the only hand-edited icon source. It
  is own work (no FTL logo, no shape or font from `ftl.dat`).
  `brand/logo-*.svg` are the variants: A to C in a purple space look (A dart
  ship at warp, B pen nib, C ship with a wrench); D to F in the box-art look of
  FTL, a dark planet with a pale pink-white rim glow on black (D a chunky E,
  E a floppy disk, F a wrench with a fleet); F2 and F3 are F without stars,
  ships or texture, and F3 (larger, steeper wrench, tighter crop) is active.
  A swap is one copy onto `brand/logo.svg`, then the icon render.
- `bun scripts/icon-render.ts` renders it with the resvg CLI (`brew install
  resvg`, pinned 0.50.0) to the macOS appiconset, `app_icon.ico` of Windows,
  `app/linux/runner/resources/app_icon.png` and `brand/logo.png`; `--check`
  proves the committed files are the render. The Linux runner does not load
  its PNG yet.
- `bun scripts/build-dmg.ts [--verify]` exports HEAD to
  `$TMPDIR/ftl-editor-release-src` (uncommitted changes are not built),
  runs `scripts/codegen.ts` there (CI passes `--generated <tar>` instead),
  builds the release app there, signs it ad-hoc, scans it with
  `scripts/scan-binaries.ts` (a hit stops before the DMG), packs `dist/FTL-Save-Editor-<version>-macos-arm64.dmg` (app plus an
  `/Applications` link; `dist/` is gitignored) and writes the receipt
  `dist/<date>-dmg-macos-arm64.json`. `--verify` mounts the DMG
  read-only, compares the bundled icon, starts the app with `open -n` and
  ends only that PID. Not notarized: the first start needs right click, Open.

## License

GPL-3.0-or-later. The Java
editor ships only the GPLv2 text and names no version in any file, so GPLv2
§9 allows any FSF version; `ftldat` is GPL-3.0, so the combined work is
GPL-3.0. A new dependency needs a GPL-3.0-compatible license.
