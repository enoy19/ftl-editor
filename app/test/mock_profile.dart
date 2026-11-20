// A profile summary as the core would answer it for a player with a few runs.
// Widget tests and shots use it instead of the Rust core.

import 'package:ftl_editor/src/rust/api/profile.dart';

ProfileShip _ship(String id, String? name, {bool a = false, bool b = false, bool c = false, bool hasC = true}) =>
    ProfileShip(
      id: id,
      name: name,
      layoutA: a,
      layoutB: b,
      layoutC: c,
      hasLayoutC: hasC,
      achievements: _shipAchievements[id] ?? const [],
    );

const _shipAchievements = {
  'PLAYER_SHIP_HARD': ['ACH_UNITED_FEDERATION', 'ACH_FULL_ARSENAL', 'ACH_TOUGH_SHIP'],
  'PLAYER_SHIP_STEALTH': ['ACH_STEALTH_DESTROY', 'ACH_STEALTH_AVOID', 'ACH_STEALTH_TACTICAL'],
};

final mockProfile = ProfileSummary(
  path: '/Users/player/Library/Application Support/FasterThanLight/ae_prof.sav',
  fileFormat: 9,
  ships: [
    _ship('PLAYER_SHIP_HARD', 'The Kestrel', a: true, b: true, c: true),
    _ship('PLAYER_SHIP_STEALTH', 'The Nesasio', a: true),
    _ship('PLAYER_SHIP_MANTIS', 'The Gila Monster', a: true, b: true),
    _ship('PLAYER_SHIP_CIRCLE', 'The Torus'),
    _ship('PLAYER_SHIP_FED', 'The Osprey'),
    _ship('PLAYER_SHIP_JELLY', 'Man of War'),
    _ship('PLAYER_SHIP_ROCK', 'The Bulwark'),
    _ship('PLAYER_SHIP_ENERGY', 'The Adjudicator'),
    _ship('PLAYER_SHIP_CRYSTAL', 'The Carnelian', hasC: false),
    _ship('PLAYER_SHIP_ANAEROBIC', 'The Kruos', a: true, hasC: false),
  ],
  achievements: const [
    ProfileAchievement(id: 'ACH_UNITED_FEDERATION', difficulty: 'normal'),
    ProfileAchievement(id: 'ACH_FULL_ARSENAL', difficulty: 'easy'),
    ProfileAchievement(id: 'ACH_MANTIS_SLAUGHTER', difficulty: 'hard'),
    ProfileAchievement(id: 'PLAYER_SHIP_HARD_VICTORY', difficulty: 'normal', victoryLayouts: ['normal', null, null]),
  ],
  topScores: const [
    ProfileScore(
      shipName: 'Kestrel',
      shipId: 'PLAYER_SHIP_HARD',
      value: 3120,
      sector: 8,
      victory: true,
      difficulty: 'normal',
    ),
    ProfileScore(
      shipName: 'Gila',
      shipId: 'PLAYER_SHIP_MANTIS',
      value: 1405,
      sector: 5,
      victory: false,
      difficulty: 'easy',
    ),
  ],
  stats: const ProfileStats(
    mostShipsDefeated: 61,
    totalShipsDefeated: 412,
    mostBeaconsExplored: 88,
    totalBeaconsExplored: 903,
    mostScrapCollected: 2875,
    totalScrapCollected: 19230,
    mostCrewHired: 7,
    totalCrewHired: 54,
    totalGamesPlayed: 23,
    totalVictories: 1,
  ),
  dirty: false,
);

/// Achievements of the game as the core names them from ftl.dat.
const mockCatalog = [
  AchievementInfo(
    id: 'ACH_UNITED_FEDERATION',
    name: 'Mixed Company',
    description: 'Fly with crew of many kinds (made-up test text).',
    kind: 'ship',
    shipId: 'PLAYER_SHIP_HARD',
  ),
  AchievementInfo(
    id: 'ACH_FULL_ARSENAL',
    name: 'Loaded Racks',
    description: 'Fill the racks with guns and drones (made-up test text).',
    kind: 'ship',
    shipId: 'PLAYER_SHIP_HARD',
  ),
  AchievementInfo(
    id: 'ACH_MANTIS_SLAUGHTER',
    name: 'Mock Brawl',
    description: 'Win a made-up brawl with the test ship.',
    kind: 'ship',
    shipId: 'PLAYER_SHIP_MANTIS',
  ),
  AchievementInfo(
    id: 'PLAYER_SHIP_HARD_VICTORY',
    name: 'Test Cruiser Victory',
    description: 'Win a made-up run with any layout of the test cruiser',
    kind: 'victory',
    shipId: 'PLAYER_SHIP_HARD',
  ),
  AchievementInfo(
    id: 'ACH_STEALTH_DESTROY',
    name: 'Quiet Hunter',
    description: 'Win made-up fights unseen (test text).',
    kind: 'ship',
    shipId: 'PLAYER_SHIP_STEALTH',
  ),
  AchievementInfo(
    id: 'ACH_STEALTH_AVOID',
    name: 'Slip Away',
    description: 'Dodge a made-up volley (test text).',
    kind: 'ship',
    shipId: 'PLAYER_SHIP_STEALTH',
  ),
  AchievementInfo(id: 'ACH_SECTOR_5', name: 'First Steps', description: 'Reach a made-up sector (test text).', kind: 'general'),
];

/// [mockProfile] with staged edits: Nesasio Type-B and Type-C unlocked (B
/// adds two achievements), Kestrel Type-C locked, Mock Brawl removed and
/// First Steps added on Hard.
final mockStaged = ProfileSummary(
  path: mockProfile.path,
  fileFormat: 9,
  ships: [
    for (final s in mockProfile.ships)
      switch (s.id) {
        'PLAYER_SHIP_HARD' => _ship(s.id, s.name, a: true, b: true),
        'PLAYER_SHIP_STEALTH' => _ship(s.id, s.name, a: true, b: true, c: true),
        _ => s,
      },
  ],
  achievements: [
    for (final a in mockProfile.achievements)
      if (a.id != 'ACH_MANTIS_SLAUGHTER') a,
    const ProfileAchievement(id: 'ACH_STEALTH_DESTROY', difficulty: 'easy'),
    const ProfileAchievement(id: 'ACH_STEALTH_AVOID', difficulty: 'easy'),
    const ProfileAchievement(id: 'ACH_SECTOR_5', difficulty: 'hard'),
  ],
  topScores: mockProfile.topScores,
  stats: mockProfile.stats,
  dirty: true,
);

/// What the core answers for the edits of [mockStaged].
const mockEditChanges = [
  ProfileEditChange(
    path: 'ships.PLAYER_SHIP_STEALTH.b',
    old: 0,
    value: 1,
    addedAchievements: ['ACH_STEALTH_DESTROY', 'ACH_STEALTH_AVOID'],
    removedAchievements: [],
  ),
  ProfileEditChange(
    path: 'ships.PLAYER_SHIP_STEALTH.c',
    old: 0,
    value: 1,
    addedAchievements: [],
    removedAchievements: [],
  ),
  ProfileEditChange(path: 'ships.PLAYER_SHIP_HARD.c', old: 1, value: 0, addedAchievements: [], removedAchievements: []),
  ProfileEditChange(
    path: 'achievements.ACH_MANTIS_SLAUGHTER',
    old: 3,
    value: 0,
    addedAchievements: [],
    removedAchievements: [],
  ),
  ProfileEditChange(
    path: 'achievements.ACH_SECTOR_5',
    old: 0,
    value: 3,
    addedAchievements: [],
    removedAchievements: [],
  ),
  ProfileEditChange(
    path: 'ships.PLAYER_SHIP_CIRCLE.a',
    old: 0,
    value: 1,
    addedAchievements: [],
    removedAchievements: [],
  ),
];
