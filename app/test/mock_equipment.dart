// The equipment as the core answers it (`equipmentOf`), for widget tests and
// shots without the Rust core: a made-up Engi cruiser with drones, cargo and
// an augment, which the Kestrel of the fixture lacks, with the names and caps
// of FTL 1.6.13. Every
// level starts at 1 and the reactor at 0: the core lowers the power with it.

import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/equipment.dart';

/// [install] is `systems.<id>.installed` as (min, max), [block] why the
/// core refuses the other value, as the core answers such a ship.
EquipmentSystem _system(
  String id,
  String name,
  int level,
  int power,
  int max, {
  bool sub = false,
  (int, int) install = (1, 1),
  String? block = 'starting_system',
}) => EquipmentSystem(
  id: id,
  name: name,
  level: level == 0 ? null : FieldValue(path: 'systems.$id.level', value: level, min: 1, max: max),
  installed: FieldValue(path: 'systems.$id.installed', value: level == 0 ? 0 : 1, min: install.$1, max: install.$2),
  block: block,
  power: power,
  damaged: 0,
  ionized: 0,
  damage: level == 0 ? null : FieldValue(path: 'systems.$id.damage', value: 0, min: 0, max: level),
  ion: level == 0 ? null : FieldValue(path: 'systems.$id.ion', value: 0, min: 0, max: 5),
  energy: level == 0 || id != 'shields'
      ? null
      : const FieldValue(path: 'systems.shields.energy', value: 0, min: 0, max: 5),
  subsystem: sub,
);

final mockEquipment = Equipment(
  reactor: const FieldValue(path: 'reactor', value: 21, min: 0, max: 25),
  systems: [
    _system('shields', 'Shields', 6, 6, 8),
    _system('engines', 'Engines', 5, 5, 8),
    _system('oxygen', 'Oxygen', 1, 1, 3),
    _system('weapons', 'Weapon Control', 5, 5, 8),
    _system('drones', 'Drone Control', 4, 0, 8),
    _system('medbay', 'Medbay', 1, 0, 3, block: 'swap_back'),
    _system('teleporter', 'Crew Teleporter', 0, 0, 3, install: (0, 0), block: 'system_limit'),
    _system('cloaking', 'Cloaking', 1, 1, 3, install: (0, 1), block: null),
    _system('clonebay', 'Clone Bay', 0, 0, 3, install: (0, 1), block: null),
    _system('mind', 'Mind Control', 1, 1, 3, install: (0, 1), block: null),
    _system('hacking', 'Hacking', 0, 0, 3, install: (0, 0), block: 'system_limit'),
    _system('pilot', 'Piloting', 1, 1, 3, sub: true),
    _system('sensors', 'Sensors', 2, 2, 3, sub: true),
    _system('doors', 'Door System', 2, 2, 3, sub: true),
    _system('battery', 'Backup Battery', 1, 1, 2, sub: true, install: (0, 1), block: null),
  ],
  bodyDecoded: true,
  weapons: const [
    EquipmentItem(id: 'ION_4', drone: false, name: 'Ion Blast Mark II', power: 3, armed: true),
    EquipmentItem(id: 'SHOTGUN_2', drone: false, name: 'Flak Gun Mark II', power: 3, armed: false),
    EquipmentItem(id: 'BEAM_HULL', drone: false, name: 'Hull Beam', power: 2, armed: true),
  ],
  drones: const [
    EquipmentItem(id: 'COMBAT_1', drone: true, name: 'Combat Drone Mark I', power: 2, armed: false),
    EquipmentItem(id: 'COMBAT_1', drone: true, name: 'Combat Drone Mark I', power: 2, armed: false),
    EquipmentItem(id: 'ANTI_DRONE', drone: true, name: 'Anti-Combat Drone', power: 1, armed: false),
  ],
  augments: const [
    EquipmentAugment(
      id: 'NANO_MEDBAY',
      name: 'Engi Med-bot Dispersal',
      count: FieldValue(path: 'augments.NANO_MEDBAY', value: 1, min: 0, max: 3),
    ),
  ],
  weaponSlots: 3,
  droneSlots: 3,
  augmentSlots: 3,
  cargo: const [EquipmentItem(id: 'REPAIR', drone: true, name: 'System Repair Drone', power: 1, armed: false)],
  cargoSlots: 4,
  // The ranges the core answers for such a ship:
  // every slot full, 2 free reactor bars, 3 of 4 cargo boxes free.
  itemFields: [
    mockItemFields('ION_4', slots: (1, 1), armed: (1, 1), cargo: (0, 3)),
    mockItemFields('SHOTGUN_2', slots: (1, 1), armed: (0, 0), cargo: (0, 3)),
    mockItemFields('BEAM_HULL', slots: (1, 1), armed: (1, 1), cargo: (0, 3)),
    mockItemFields('COMBAT_1', drone: true, slots: (2, 2), armed: (0, 1), cargo: (0, 3)),
    mockItemFields('ANTI_DRONE', drone: true, slots: (1, 1), armed: (0, 1), cargo: (0, 3)),
    mockItemFields('REPAIR', drone: true, slots: (0, 0), armed: (0, 0), cargo: (1, 4)),
  ],
);

/// The fields of one weapon or drone id as `(value, max)` pairs; every
/// minimum is 0.
EquipmentItemFields mockItemFields(
  String id, {
  bool drone = false,
  required (int, int) slots,
  required (int, int) armed,
  required (int, int) cargo,
}) {
  final kind = drone ? 'drones' : 'weapons';
  return EquipmentItemFields(
    id: id,
    drone: drone,
    slots: FieldValue(path: '$kind.$id', value: slots.$1, min: 0, max: slots.$2),
    armed: FieldValue(path: '$kind.$id.armed', value: armed.$1, min: 0, max: armed.$2),
    cargo: FieldValue(path: 'cargo.$id', value: cargo.$1, min: 0, max: cargo.$2),
  );
}

const mockItemChoices = [
  ItemChoice(id: 'LASER_BURST_3', drone: false, name: 'Burst Laser III', power: 4),
  ItemChoice(id: 'ION_4', drone: false, name: 'Ion Blast Mark II', power: 3),
  ItemChoice(id: 'COMBAT_1', drone: true, name: 'Combat Drone Mark I', power: 2),
  ItemChoice(id: 'DRONE_SHIELD_PLAYER', drone: true, name: 'Shield Drone', power: 2),
];

const mockAugmentChoices = [
  AugmentChoice(id: 'AUTO_COOLDOWN', name: 'Automated Re-loader'),
  AugmentChoice(id: 'NANO_MEDBAY', name: 'Engi Med-bot Dispersal'),
  AugmentChoice(id: 'SCRAP_COLLECTOR', name: 'Scrap Recovery Arm'),
];

/// `augmentField` of the core for an augment the ship lacks.
FieldValue mockAugmentField(String id) => FieldValue(path: 'augments.$id', value: 0, min: 0, max: 3);

/// [mockEquipment] as a preview answers it: each system of [levels] at that
/// level, `null` for not installed (`equipmentPreview` of staged installs);
/// [weapons] instead of the file's.
Equipment mockEquipmentWith(Map<String, int?> levels, {List<EquipmentItem>? weapons}) => Equipment(
  reactor: mockEquipment.reactor,
  systems: [
    for (final s in mockEquipment.systems)
      if (!levels.containsKey(s.id))
        s
      else
        EquipmentSystem(
          id: s.id,
          name: s.name,
          level: levels[s.id] == null
              ? null
              : FieldValue(path: 'systems.${s.id}.level', value: levels[s.id]!, min: 1, max: s.level?.max ?? 3),
          installed: FieldValue(path: s.installed.path, value: levels[s.id] == null ? 0 : 1, min: 0, max: 1),
          block: null,
          power: 0,
          damaged: 0,
          ionized: 0,
          subsystem: s.subsystem,
        ),
  ],
  bodyDecoded: mockEquipment.bodyDecoded,
  weapons: weapons ?? mockEquipment.weapons,
  drones: mockEquipment.drones,
  augments: mockEquipment.augments,
  weaponSlots: mockEquipment.weaponSlots,
  droneSlots: mockEquipment.droneSlots,
  augmentSlots: mockEquipment.augmentSlots,
  cargo: mockEquipment.cargo,
  cargoSlots: mockEquipment.cargoSlots,
  itemFields: mockEquipment.itemFields,
);

/// The preview of the core after `systems.shields.level=8` (6 in the file):
/// the level and the damage range of a staged upgrade.
final shieldsUpgradedPreview = Equipment(
  reactor: mockEquipment.reactor,
  systems: [
    for (final s in mockEquipment.systems)
      s.id == 'shields'
          ? EquipmentSystem(
              id: s.id,
              name: s.name,
              level: const FieldValue(path: 'systems.shields.level', value: 8, min: 1, max: 8),
              installed: s.installed,
              block: s.block,
              power: s.power,
              damaged: 0,
              ionized: 0,
              damage: const FieldValue(path: 'systems.shields.damage', value: 0, min: 0, max: 8),
              ion: s.ion,
              energy: s.energy,
              subsystem: false,
            )
          : s,
  ],
  bodyDecoded: mockEquipment.bodyDecoded,
  weapons: mockEquipment.weapons,
  drones: mockEquipment.drones,
  augments: mockEquipment.augments,
  weaponSlots: mockEquipment.weaponSlots,
  droneSlots: mockEquipment.droneSlots,
  augmentSlots: mockEquipment.augmentSlots,
  cargo: mockEquipment.cargo,
  cargoSlots: mockEquipment.cargoSlots,
  itemFields: mockEquipment.itemFields,
);
