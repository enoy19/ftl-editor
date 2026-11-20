// A floorplan as the core answers it (`floorplanOf`), built by the test: a
// small ship of three rooms, nothing from ftl.dat. Room 0 (2x2, grid 1, 1)
// holds the shields and burns in square 1, room 1 (1x2, grid 3, 1) the
// pilot, room 2 (1x1, grid 4, 1) nothing and a breach; the blueprint gives
// room 2 the teleporter. Crew member 0 (mockCrew) stands in square 2 of
// room 0. [mockFloorplanWith] is the same ship with other systems in its
// rooms, as `floorplanPreview` answers a staged install or removal.
// Its three weapon mounts carry the weapons of mockEquipment in slot order:
// two turned mounts on the top edge and an upright one on the bottom.

import 'dart:typed_data';

import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/floorplan.dart';

FieldValue _health(String kind, int gx, int gy, int value) =>
    FieldValue(path: '$kind.$gx.$gy', value: value, min: 0, max: 100);

/// A room at pixel [x], [y] whose top left square is grid [gx], [gy];
/// [fire], [breach] and [crew] name square ids.
FloorRoom _room(
  int id,
  int x,
  int y,
  int w,
  int h,
  int gx,
  int gy,
  String? system, {
  Set<int> fire = const {},
  Set<int> breach = const {},
  Map<int, List<int>> crew = const {},
  int oxygen = 100,
}) {
  FloorImage interior(String s) => FloorImage(innerPath: 'img/ship/interior/room_$s.png', x: x, y: y, w: w, h: h);
  final cols = w ~/ 35;
  return FloorRoom(
    id: id,
    x: x,
    y: y,
    w: w,
    h: h,
    system: system,
    interior: [if (system != null) interior(system)],
    oxygen: oxygen,
    oxygenField: FieldValue(path: 'rooms.$id.oxygen', value: oxygen, min: 0, max: 100),
    squares: [
      for (var s = 0; s < cols * (h ~/ 35); s++)
        FloorSquare(
          id: s,
          x: x + (s % cols) * 35,
          y: y + (s ~/ cols) * 35,
          fire: _health('fires', gx + s % cols, gy + s ~/ cols, fire.contains(s) ? 100 : 0),
          breach: _health('breaches', gx + s % cols, gy + s ~/ cols, breach.contains(s) ? 100 : 0),
          crew: Uint32List.fromList(crew[s] ?? const []),
        ),
    ],
  );
}

final mockFloorplan = mockFloorplanWith();

/// The mounts of the mock ship with [weapons] in slot order (`null` a free
/// slot) and [armed] the powered slots.
List<FloorMount> mockMounts({
  List<String?> weapons = const ['ION_4', 'SHOTGUN_2', 'BEAM_HULL'],
  Set<int> armed = const {0, 2},
}) => [
  for (final (slot, (x, y, w, h, rotate)) in const [
    (110, 2, 60, 20, true),
    (40, 2, 60, 20, true),
    (180, 76, 20, 60, false),
  ].indexed)
    FloorMount(
      slot: slot,
      x: x,
      y: y,
      w: w,
      h: h,
      rotate: rotate,
      mirror: slot == 1,
      weapon: weapons[slot],
      armed: armed.contains(slot),
    ),
];

/// The mock ship with the system of each room in [systems] (room id to
/// system id, `null` for none) instead of the file's, and the oxygen of
/// each room in [oxygen] (room id to percent, else 100).
Floorplan mockFloorplanWith([
  Map<int, String?> systems = const {},
  Map<int, int> oxygen = const {},
  List<FloorMount>? mounts,
]) => Floorplan(
  width: 210,
  height: 140,
  hull: const FloorImage(innerPath: 'img/ship/test_base.png', x: 0, y: 0, w: 210, h: 140),
  floor: const FloorImage(innerPath: 'img/ship/test_floor.png', x: 20, y: 20, w: 170, h: 100),
  rooms: [
    _room(
      0,
      35,
      35,
      70,
      70,
      1,
      1,
      systems.containsKey(0) ? systems[0] : 'shields',
      fire: {1},
      crew: {
        2: [0],
      },
      oxygen: oxygen[0] ?? 100,
    ),
    _room(1, 105, 35, 35, 70, 3, 1, systems.containsKey(1) ? systems[1] : 'pilot', oxygen: oxygen[1] ?? 100),
    _room(2, 140, 35, 35, 35, 4, 1, systems[2], breach: {0}, oxygen: oxygen[2] ?? 100),
  ],
  doors: const [FloorDoor(x: 105, y: 35, vertical: true), FloorDoor(x: 140, y: 35, vertical: true)],
  crew: const [FloorCrew(index: 0, race: 'engi', lookId: 'engi.m', x: 52, y: 87)],
  mounts: mounts ?? mockMounts(),
);
