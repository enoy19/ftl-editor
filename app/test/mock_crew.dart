// A crew as the core answers it (`crewOf`), for widget tests and shots
// without the Rust core. The first three are made up: two Engi and a Lanius,
// so the tests see races and masteries the all-human crew of the fixture
// lacks; the others fill two rows of four.

import 'dart:typed_data';

import 'package:ftl_editor/src/rust/api/crew.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/ui/crew_card.dart';

const _skills = ['pilot', 'engine', 'shield', 'weapon', 'repair', 'combat'];
const _human = [13, 13, 50, 58, 16, 7];
const _other = [15, 15, 55, 65, 18, 8];

CrewMember mockMember(
  int index,
  String name,
  String race,
  int health,
  int maxHealth,
  List<int> points, {
  String? raceName,
  bool male = true,
  List<int> tints = const [],
  bool lastOfSide = false,
}) {
  final intervals = race == 'human' ? _human : _other;
  return CrewMember(
    index: index,
    name: name,
    namePath: 'crew.$index.name',
    race: race,
    racePath: 'crew.$index.race',
    raceName: raceName,
    male: male,
    sex: FieldValue(path: 'crew.$index.male', value: male ? 1 : 0, min: 0, max: 1),
    tints: [for (final (k, t) in tints.indexed) FieldValue(path: 'crew.$index.tints.$k', value: t, min: 0, max: 3)],
    remove: FieldValue(path: 'crew.$index.remove', value: 0, min: 0, max: lastOfSide ? 0 : 1),
    removed: false,
    lookId: mockLookId(race, male: male, tints: tints),
    health: FieldValue(path: 'crew.$index.health', value: health, min: 1, max: maxHealth),
    skills: [
      for (var i = 0; i < 6; i++)
        CrewSkill(
          skill: _skills[i],
          points: FieldValue(path: 'crew.$index.skills.${_skills[i]}', value: points[i], min: 0, max: 2 * intervals[i]),
          mastery: (points[i] ~/ intervals[i]).clamp(0, 2),
          masteryPoints: Int32List.fromList([0, intervals[i], 2 * intervals[i]]),
        ),
    ],
  );
}

/// The look id the core builds (`crew_look_id`): race, `m` or `f`, the tints.
String mockLookId(String race, {bool male = true, List<int> tints = const []}) =>
    [race, male ? 'm' : 'f', for (final t in tints) '$t'].join('.');

/// The crew of the fixture.
final mockCrew = [
  mockMember(0, 'Hollis', 'engi', 100, 100, [0, 0, 0, 130, 21, 1]),
  mockMember(1, 'Juno', 'engi', 100, 100, [30, 0, 0, 0, 10, 1]),
  mockMember(2, 'Tamsin', 'anaerobic', 100, 100, [0, 0, 56, 0, 3, 10]),
];

/// A full ship: eight crew members of several races.
final fullCrew = [
  ...mockCrew,
  mockMember(3, 'Holly', 'human', 64, 100, [26, 13, 30, 4, 9, 3], male: false),
  mockMember(4, 'Kazaar', 'mantis', 84, 100, [2, 5, 0, 70, 6, 16]),
  mockMember(5, 'Granit', 'rock', 150, 150, [6, 0, 110, 30, 14, 8]),
  mockMember(6, 'Zap', 'energy', 30, 70, [15, 30, 0, 0, 18, 0]),
  mockMember(7, 'Shard', 'crystal', 125, 125, [0, 0, 55, 65, 0, 8]),
];

/// The races of the picker as the core answers them in reduced mode.
const mockRaces = [
  RaceChoice(id: 'human', name: null, lookId: 'human.m'),
  RaceChoice(id: 'engi', name: null, lookId: 'engi.m'),
  RaceChoice(id: 'mantis', name: null, lookId: 'mantis.m'),
  RaceChoice(id: 'slug', name: null, lookId: 'slug.m'),
  RaceChoice(id: 'rock', name: null, lookId: 'rock.m'),
  RaceChoice(id: 'energy', name: null, lookId: 'energy.m'),
  RaceChoice(id: 'crystal', name: null, lookId: 'crystal.m'),
  RaceChoice(id: 'anaerobic', name: null, lookId: 'anaerobic.m'),
];

void _ignoreText(String path, String text) {}

/// Name and race editable, nothing staged.
const mockCrewTexts = CrewTexts(races: mockRaces, onChanged: _ignoreText);
