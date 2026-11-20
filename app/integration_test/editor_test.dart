// Flow tests over the real app and the real Rust core (macOS):
//
//   flutter test integration_test -d macos
//
// The native file dialog cannot be driven by a test, so each test passes a
// file chooser that answers a fixed path. Preferences live in memory, the
// default save file and the editor's config folder point into the test's temp
// folder, so no test touches the real FTL folder, the real app preferences or
// the real editor config. Tests that need the game's ftl.dat read the
// installed one and print a skip line without it; nothing extracted is kept.

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_look.dart';
import 'package:ftl_editor/look/ftl_tabs.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/main.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/crew.dart';
import 'package:ftl_editor/src/rust/api/equipment.dart';
import 'package:ftl_editor/src/rust/api/floorplan.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/status.dart';
import 'package:ftl_editor/src/rust/frb_generated.dart';
import 'package:ftl_editor/ui/crew_page.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/editor_page.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/save_location.dart';
import 'package:integration_test/integration_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:shared_preferences/shared_preferences.dart';

const fixture = '../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav';

/// The old and new value of a change sheet row.
Finder findChange(int old, int value) =>
    find.byWidgetPredicate((w) => w is ChangeValues && w.old == '$old' && w.value == '$value');

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // A tap that misses its widget fails the test instead of printing a warning:
  // the window size of the macOS test runner varies between runs.
  WidgetController.hitTestWarningShouldBeFatal = true;
  setUpAll(() async => RustLib.init());
  setUp(() => SharedPreferences.setMockInitialValues({}));

  late Directory dir;
  late File save;
  setUp(() async {
    dir = await Directory.systemTemp.createTemp('ftl-editor-it');
    save = File('${dir.path}/continue.sav');
    await File(fixture).copy(save.path);
  });
  tearDown(() async => dir.delete(recursive: true));

  String configDir() => '${dir.path}/config';

  /// No ftl.dat anywhere: the reduced mode.
  Future<FtlDatStatus> nothingFound() async => const FtlDatStatus(tried: ['/nowhere/ftl.dat']);

  /// The warnings the injected status source answers; empty is "FTL closed,
  /// no Steam Cloud". A user's running FTL never reaches a test.
  var warnings = <FileWarning>[];
  var warningReads = 0;
  setUp(() {
    warnings = [];
    warningReads = 0;
  });
  Future<List<FileWarning>> fakeWarnings(String path) async {
    warningReads++;
    return warnings;
  }

  /// The search for ftl.dat answers "nothing found" unless [realLocate] runs
  /// the real search with the test's config folder. The warnings come from
  /// [fakeWarnings] unless [realWarnings] asks the core.
  Widget app({String? pick, bool realLocate = false, String? pickDat, bool realWarnings = false}) => FtlEditorApp(
    home: EditorPage(
      backupDir: '${dir.path}/backups',
      defaultFile: '${dir.path}/missing/continue.sav',
      pickFile: (_, _) async => pick,
      assetsConfigDir: configDir(),
      locateFtlDat: realLocate ? null : nothingFound,
      pickFtlDat: (_) async => pickDat,
      warningsOf: realWarnings ? (path) => saveWarnings(path: path) : fakeWarnings,
    ),
  );

  final popover = find.byKey(const ValueKey('resource-popover'));
  final popoverField = find.descendant(of: popover, matching: find.byType(TextField));

  /// Opens the popover of the resource [path] in the resource bar.
  Future<void> openResource(WidgetTester tester, String path) async {
    final chip = find.byKey(ValueKey('resource-$path'));
    await tester.ensureVisible(chip);
    await tester.pumpAndSettle();
    await tester.tap(chip);
    await tester.pumpAndSettle();
    expect(popover, findsOneWidget);
  }

  /// Writes the file from the open change sheet.
  Future<void> writeFromSheet(WidgetTester tester) async {
    expect(find.byKey(const ValueKey('change-sheet')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await tester.pumpAndSettle();
  }

  Future<void> apply(WidgetTester tester) async {
    await tester.tap(find.descendant(of: popover, matching: find.text('Apply')));
    await tester.pumpAndSettle();
  }

  /// The installed ftl.dat as the real search finds it, or `null` with a
  /// printed skip reason.
  Future<String?> installedDat(String test) async {
    final status = await ftlDatStatus(configDir: '${dir.path}/probe-config');
    // The skip line goes to the test output on purpose, like the Rust tests.
    // ignore: avoid_print
    if (status.path == null) print('skip "$test": no ftl.dat installed; searched ${status.tried}');
    return status.path;
  }

  /// Pumps real time until [done] holds; a Rust future is no frame, so
  /// pumpAndSettle does not wait for it.
  Future<void> pumpUntil(WidgetTester tester, bool Function() done, String what) async {
    for (var i = 0; i < 200 && !done(); i++) {
      await Future<void>.delayed(const Duration(milliseconds: 50));
      await tester.pump();
    }
    expect(done(), isTrue, reason: 'waited 10 s for $what');
  }

  /// Writes [edits] into the test's copy of the fixture through the core
  /// before the app opens it, for a test that needs a state the fixture lacks.
  /// Its backup lands apart from the folder the tests count.
  Future<void> prepare(List<FieldEdit> edits) async {
    final session = await openSave(path: save.path);
    session.apply(edits: edits, texts: const []);
    await session.save(backupDir: '${dir.path}/prepare-backups');
  }

  Finder shipImage() => find.descendant(of: find.byType(ShipCard), matching: find.byType(RawImage));
  bool shipImageShown(WidgetTester tester) =>
      shipImage().evaluate().isNotEmpty && tester.widget<RawImage>(shipImage()).image != null;

  /// A text in either look: a [Text] in the fallback font or an [FtlText].
  /// With ftl.dat open the app wears the FTL look, so tests that open it
  /// find texts, tabs and buttons by this and by keys.
  Finder findLabel(String text) =>
      find.byWidgetPredicate((w) => (w is Text && w.data == text) || (w is FtlText && w.text == text));
  final saveButton = find.byKey(const ValueKey('save-button'));
  final chooseFile = find.byKey(const ValueKey('choose-file'));
  Finder tab(String area) => find.byKey(ValueKey('area-$area'));

  Future<void> tapVisible(WidgetTester tester, Finder finder) async {
    await tester.ensureVisible(finder);
    await tester.pumpAndSettle();
    await tester.tap(finder);
  }

  testWidgets('one button chooses a file and loads it; save writes the edit', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    expect(find.text('The Kestrel'), findsOneWidget);
    // The file card and the status bar name the file.
    expect(find.text('continue.sav'), findsNWidgets(2));
    expect(find.text('No backup in this session'), findsOneWidget);
    final prefs = await SharedPreferences.getInstance();
    expect(prefs.getString(SaveLocation.lastFileKey), save.path);

    final saveButton = find.widgetWithIcon(FilledButton, Icons.save);
    expect(tester.widget<FilledButton>(saveButton).onPressed, isNull, reason: 'nothing edited yet');

    await openResource(tester, 'ship.scrap');
    await tester.enterText(popoverField, '9999');
    await apply(tester);
    expect(find.text('9999'), findsOneWidget);
    expect(tester.widget<FilledButton>(saveButton).onPressed, isNotNull);
    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    await writeFromSheet(tester);

    final reread = await openSave(path: save.path);
    expect(reread.summary().fields.firstWhere((f) => f.path == 'ship.scrap').value, 9999);
    final backups = Directory('${dir.path}/backups').listSync();
    expect(backups, hasLength(1));
    expect(find.textContaining('Last backup: '), findsOneWidget, reason: 'the status bar names the backup');
    expect(find.byTooltip(backups.single.path), findsOneWidget);
    expect(await File(backups.single.path).readAsBytes(), await File(fixture).readAsBytes());
  });

  Finder crewSprite(int index) =>
      find.descendant(of: find.byKey(ValueKey('crew-portrait-$index')), matching: find.byType(RawImage));
  bool crewSpriteShown(WidgetTester tester, int index) =>
      crewSprite(index).evaluate().isNotEmpty && tester.widget<RawImage>(crewSprite(index)).image != null;

  /// Steps the shield ring of Elizabeth (a human, 0 points, mastery 0) to
  /// mastery 1, saves through the change sheet and reads the crew back
  /// through the core.
  Future<void> stepAndSave(WidgetTester tester) async {
    await tester.tap(tab('crew'));
    await tester.pumpAndSettle();
    expect(find.byType(CrewCard), findsNWidgets(3));
    await tapVisible(tester, find.byKey(const ValueKey('skill-crew.2.skills.shield')));
    await tester.pumpAndSettle();
    expect(find.byTooltip('Shields: 50 of 100, level 1 of 2'), findsOneWidget);

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    final row = find.byKey(const ValueKey('change-crew.2.skills.shield'));
    expect(find.descendant(of: row, matching: findLabel('Elizabeth: Shields')), findsOneWidget);
    expect(find.descendant(of: row, matching: findChange(0, 50)), findsOneWidget);
    await writeFromSheet(tester);

    final crew = crewOf(session: await openSave(path: save.path), nearby: false);
    final shield = crew[2].skills.firstWhere((s) => s.skill == 'shield');
    expect((shield.points.value, shield.mastery), (50, 1));
    expect(Directory('${dir.path}/backups').listSync(), hasLength(1));
  }

  testWidgets('a click on a skill ring steps the mastery; Save writes it (reduced mode)', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    await stepAndSave(tester);
    expect(
      find.descendant(of: find.byKey(const ValueKey('crew-card-2')), matching: find.byIcon(Icons.person)),
      findsOneWidget,
    );
    expect(crewSprite(2), findsNothing);
    expect(find.text('Human'), findsNWidgets(3), reason: 'the app names the race without ftl.dat');
  });

  testWidgets('Rename and a new race of a crew member; Save writes both and the race fits the skills', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    await tester.tap(tab('crew'));
    await tester.pumpAndSettle();

    // Elizabeth has no mastery yet: a click on the shield ring gives her one,
    // so the new race has a level to keep.
    await tapVisible(tester, find.byKey(const ValueKey('skill-crew.2.skills.shield')));
    await tester.pumpAndSettle();
    await tapVisible(tester, find.byKey(const ValueKey('rename-2')));
    await tester.pumpAndSettle();
    await tester.enterText(find.byKey(const ValueKey('rename-field')), ' Ripley ');
    await tester.tap(find.byKey(const ValueKey('rename-apply')));
    await tester.pumpAndSettle();
    await tapVisible(tester, find.byKey(const ValueKey('race-2')));
    await tester.pumpAndSettle();
    await tapVisible(tester, find.byKey(const ValueKey('race-choice-crystal')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('crew-name-mark-2')), findsOneWidget);
    // The core previews the race before the save: a Crystal has 125 health.
    expect(
      find.descendant(of: find.byKey(const ValueKey('crew-card-2')), matching: findLabel('125/125')),
      findsOneWidget,
    );

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    final name = find.byKey(const ValueKey('change-crew.2.name'));
    expect(find.descendant(of: name, matching: findLabel('Elizabeth: Name')), findsOneWidget);
    expect(
      find.descendant(of: name, matching: find.byWidgetPredicate((w) => w is ChangeValues && w.value == 'Ripley')),
      findsOneWidget,
    );
    expect(find.byKey(const ValueKey('change-crew.2.race')), findsOneWidget);
    await writeFromSheet(tester);

    final ripley = crewOf(session: await openSave(path: save.path), nearby: false)[2];
    expect((ripley.name, ripley.race, ripley.health.max), ('Ripley', 'crystal', 125));
    final shield = ripley.skills.firstWhere((s) => s.skill == 'shield');
    expect(shield.mastery, 1, reason: 'the race keeps the mastery level');
    expect(find.text('Ripley'), findsWidgets, reason: 'the page shows the file as written');
  });

  testWidgets('the ship card renames the ship and sets Hard; Save writes both', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();

    await tapVisible(tester, find.byKey(const ValueKey('ship-rename')));
    await tester.pumpAndSettle();
    await tester.enterText(find.byKey(const ValueKey('rename-field')), ' Red Dwarf ');
    await tester.tap(find.byKey(const ValueKey('rename-apply')));
    await tester.pumpAndSettle();
    await tapVisible(tester, find.byKey(const ValueKey('ship-difficulty')));
    await tester.pumpAndSettle();
    await tapVisible(tester, find.byKey(const ValueKey('difficulty-choice-2')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('ship-name-mark')), findsOneWidget);

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    final name = find.byKey(const ValueKey('change-ship.name'));
    expect(
      find.descendant(of: name, matching: find.byWidgetPredicate((w) => w is ChangeValues && w.value == 'Red Dwarf')),
      findsOneWidget,
    );
    expect(find.byKey(const ValueKey('change-difficulty')), findsOneWidget);
    await writeFromSheet(tester);

    final written = (await openSave(path: save.path)).summary();
    expect((written.shipName, written.difficulty), ('Red Dwarf', 'hard'));
  });

  testWidgets('the plus card adds a crew member; a new name, a look and a removal; Save writes all', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    await tester.tap(tab('crew'));
    await tester.pumpAndSettle();
    final add = find.byKey(const ValueKey('crew-add'));
    if (add.evaluate().isEmpty) {
      // ignore: avoid_print
      print('skip "add crew": the rooms are not decoded without the ship layouts of ftl.dat');
      return;
    }
    await tapVisible(tester, add);
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('crew-card-3')), findsOneWidget, reason: 'the core places a default human');
    await tapVisible(tester, find.byKey(const ValueKey('rename-3')));
    await tester.pumpAndSettle();
    await tester.enterText(find.byKey(const ValueKey('rename-field')), 'Vasquez');
    await tester.tap(find.byKey(const ValueKey('rename-apply')));
    await tester.pumpAndSettle();
    // The look picker of a human: four colors male, then four female.
    await tapVisible(tester, find.byKey(const ValueKey('look-3')));
    await tester.pumpAndSettle();
    await tapVisible(tester, find.byKey(const ValueKey('look-choice-6')));
    await tester.pumpAndSettle();
    await tapVisible(tester, find.byKey(const ValueKey('remove-0')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('crew-state-0')), findsOneWidget);

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('change-crew.add')), findsOneWidget);
    expect(find.byKey(const ValueKey('change-crew.0.remove')), findsOneWidget);
    expect(find.byKey(const ValueKey('change-crew.3.male')), findsOneWidget);
    expect(find.byKey(const ValueKey('change-crew.3.tints.1')), findsOneWidget);
    expect(find.byKey(const ValueKey('change-crew.3.name')), findsOneWidget);
    await writeFromSheet(tester);

    final crew = crewOf(session: await openSave(path: save.path), nearby: false);
    expect([for (final c in crew) c.name], ['Chris Malott', 'Elizabeth', 'Vasquez']);
    final vasquez = crew[2];
    expect((vasquez.race, vasquez.male, vasquez.lookId), ('human', false, 'human.f.0.2'));
  });

  testWidgets('with ftl.dat the crew cards show the race sprites and titles', (tester) async {
    final dat = await installedDat('crew sprites');
    if (dat == null) return;
    await rememberFtlDat(configDir: configDir(), picked: dat);
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(realLocate: true));
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
    await tester.tap(tab('crew'));
    await tester.pumpAndSettle();
    for (final i in [0, 1, 2]) {
      await pumpUntil(tester, () => crewSpriteShown(tester, i), 'the sprite of crew $i');
    }
    expect(findLabel('Human'), findsNWidgets(3));
    await tester.tap(tab('ship'));
    await tester.pumpAndSettle();
    await stepAndSave(tester);
  });

  testWidgets('the Equipment page raises a system level and adds an augment; Save writes both', (tester) async {
    final dat = await installedDat('equipment');
    if (dat == null) return;
    await rememberFtlDat(configDir: configDir(), picked: dat);
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(realLocate: true));
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
    await tester.tap(tab('equipment'));
    await tester.pumpAndSettle();
    expect(findLabel('OXYGEN'), findsOneWidget);

    // Oxygen is at level 1 of 3: the second segment sets level 2.
    await tapVisible(tester, find.byKey(const ValueKey('equip-cell-oxygen-1')));
    await tester.pumpAndSettle();
    await tapVisible(tester, find.byKey(const ValueKey('equip-add-augment')));
    await tester.pumpAndSettle();
    await tester.enterText(find.byKey(const ValueKey('augment-filter')), 'Scrap Recovery');
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('augment-choice-SCRAP_COLLECTOR')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('equip-remove-SCRAP_COLLECTOR')), findsOneWidget);

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('change-systems.oxygen.level')), findsOneWidget);
    expect(find.byKey(const ValueKey('change-augments.SCRAP_COLLECTOR')), findsOneWidget);
    await writeFromSheet(tester);

    final equipment = equipmentOf(
      session: await openSave(path: save.path),
      nearby: false,
      language: 'en',
    );
    expect(equipment.systems.firstWhere((s) => s.id == 'oxygen').level!.value, 2);
    expect([for (final a in equipment.augments) (a.id, a.count.value)], [('SCRAP_COLLECTOR', 1)]);
    expect(Directory('${dir.path}/backups').listSync(), hasLength(1));
  });

  testWidgets('the enemy ship switch edits the nearby ship; the change sheet names it and Save writes it', (
    tester,
  ) async {
    final dat = await installedDat('enemy');
    if (dat == null) return;
    await rememberFtlDat(configDir: configDir(), picked: dat);
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(realLocate: true));
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');

    // The fixture has an enemy ship: the switch shows above the Ship page.
    await tapVisible(tester, find.byKey(const ValueKey('ship-nearby')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('resource-nearby.ship.hull')), findsOneWidget);
    expect(find.byKey(const ValueKey('resource-ship.hull')), findsNothing);

    // The switch stays on the enemy ship on the Equipment page: shields 2 go to 3.
    await tester.tap(tab('equipment'));
    await tester.pumpAndSettle();
    await tapVisible(tester, find.byKey(const ValueKey('equip-cell-shields-2')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('equip-to-cargo-weapon-0')), findsNothing);

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('change-nearby.systems.shields.level')), findsOneWidget);
    expect(findChange(2, 3), findsOneWidget);
    await writeFromSheet(tester);

    final session = await openSave(path: save.path);
    final enemy = equipmentOf(session: session, nearby: true, language: 'en');
    final player = equipmentOf(session: session, nearby: false, language: 'en');
    expect(enemy.systems.firstWhere((s) => s.id == 'shields').level!.value, 3);
    expect(player.systems.firstWhere((s) => s.id == 'shields').level!.value, 2);
  });

  testWidgets('the Equipment page moves a weapon to the cargo, adds a weapon and powers a drone; Save writes all', (
    tester,
  ) async {
    final dat = await installedDat('weapons');
    if (dat == null) return;
    // The Kestrel of the fixture has no drone system: the drone control, two
    // unpowered drones and two free reactor bars come first.
    await prepare(const [
      FieldEdit(path: 'systems.drones.installed', value: 1),
      FieldEdit(path: 'reactor', value: 10),
      FieldEdit(path: 'drones.COMBAT_1', value: 1),
      FieldEdit(path: 'drones.DEFENSE_1', value: 1),
    ]);
    await rememberFtlDat(configDir: configDir(), picked: dat);
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(realLocate: true));
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
    await tester.tap(tab('equipment'));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('equip-read-only')), findsNothing);

    // The Burst Laser (second weapon) goes to the cargo, which frees a slot.
    await tapVisible(tester, find.byKey(const ValueKey('equip-to-cargo-weapon-1')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('equip-cargo-0')), findsOneWidget);
    await tapVisible(tester, find.byKey(const ValueKey('equip-add-weapon')));
    await tester.pumpAndSettle();
    final beam = itemChoices(language: 'en').firstWhere((c) => c.id == 'BEAM_1');
    await tester.enterText(find.byKey(const ValueKey('item-filter')), beam.name);
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('item-choice-BEAM_1')));
    await tester.pumpAndSettle();
    // The new beam takes the slot the laser freed, with the mark of an added item.
    final second = find.descendant(of: find.byKey(const ValueKey('equip-weapon-1')), matching: find.byType(ChangeMark));
    expect(tester.widget<ChangeMark>(second).direction, EditDirection.raised);
    // The defense drone (second) takes the two free reactor bars.
    await tapVisible(tester, find.byKey(const ValueKey('equip-power-drone-1')));
    await tester.pumpAndSettle();

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    for (final path in ['weapons.LASER_BURST_3', 'cargo.LASER_BURST_3', 'weapons.BEAM_1', 'drones.DEFENSE_1.armed']) {
      expect(find.byKey(ValueKey('change-$path')), findsOneWidget);
    }
    await writeFromSheet(tester);

    final equipment = equipmentOf(
      session: await openSave(path: save.path),
      nearby: false,
      language: 'en',
    );
    expect([for (final w in equipment.weapons) (w.id, w.armed)], [('MISSILES_2_PLAYER', true), ('BEAM_1', false)]);
    expect([for (final d in equipment.drones) d.armed], [false, true]);
    expect([for (final c in equipment.cargo!) c.id], ['LASER_BURST_3']);
    expect(Directory('${dir.path}/backups').listSync(), hasLength(1));
  });

  testWidgets('a click on a weapon slot replaces it in place; one Undo takes the replace back; Save writes it', (
    tester,
  ) async {
    final dat = await installedDat('replace');
    if (dat == null) return;
    await rememberFtlDat(configDir: configDir(), picked: dat);
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(realLocate: true));
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
    await tester.tap(tab('equipment'));
    await tester.pumpAndSettle();
    Finder title(String key) => find.byKey(ValueKey(key));
    Future<void> replaceLaser() async {
      await tapVisible(tester, title('equip-weapon-1'));
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('item-choice-empty')), findsOneWidget);
      final beam = itemChoices(language: 'en').firstWhere((c) => c.id == 'BEAM_1');
      await tester.enterText(find.byKey(const ValueKey('item-filter')), beam.name);
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('item-choice-BEAM_1')));
      await tester.pumpAndSettle();
    }

    await replaceLaser();
    final mark = find.descendant(of: title('equip-weapon-1'), matching: find.byType(ChangeMark));
    expect(tester.widget<ChangeMark>(mark).direction, EditDirection.raised);
    // One Undo takes back both edits of the replace.
    await tester.tap(find.byKey(const ValueKey('undo-button')));
    await tester.pumpAndSettle();
    expect(find.descendant(of: title('equip-weapon-1'), matching: find.byType(ChangeMark)), findsNothing);
    // The FTL look draws Undo as an FtlButton.
    expect(tester.widget<FtlButton>(find.byKey(const ValueKey('undo-button'))).onPressed, isNull);
    await tester.tap(find.byKey(const ValueKey('redo-button')));
    await tester.pumpAndSettle();
    expect(find.descendant(of: title('equip-weapon-1'), matching: find.byType(ChangeMark)), findsOneWidget);

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    await writeFromSheet(tester);
    final equipment = equipmentOf(
      session: await openSave(path: save.path),
      nearby: false,
      language: 'en',
    );
    expect([for (final w in equipment.weapons) w.id], ['MISSILES_2_PLAYER', 'BEAM_1']);
  });

  testWidgets(
    'the Equipment page removes mind control, installs the teleporter and swaps in the clone bay; Save writes all',
    (tester) async {
      final dat = await installedDat('install');
      if (dat == null) return;
      // The Kestrel of the fixture has five systems: mind control, cloaking
      // and the drone control make eight, the most FTL's store sells.
      await prepare(const [
        FieldEdit(path: 'systems.mind.installed', value: 1),
        FieldEdit(path: 'systems.cloaking.installed', value: 1),
        FieldEdit(path: 'systems.drones.installed', value: 1),
      ]);
      await rememberFtlDat(configDir: configDir(), picked: dat);
      SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
      await tester.pumpWidget(app(realLocate: true));
      await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
      await tester.tap(tab('equipment'));
      await tester.pumpAndSettle();

      Finder button(String key) => find.byKey(ValueKey(key));
      bool enabled(String key) =>
          tester.widget<AppButton>(find.ancestor(of: button(key), matching: find.byType(AppButton)).first).onPressed !=
          null;
      // The ship holds 8 systems: the teleporter waits for a place.
      expect(enabled('equip-install-system-teleporter'), isFalse);
      await tapVisible(tester, button('equip-remove-system-mind'));
      await tester.pumpAndSettle();
      expect(enabled('equip-install-system-teleporter'), isTrue);
      await tapVisible(tester, button('equip-install-system-teleporter'));
      await tester.pumpAndSettle();
      await tapVisible(tester, button('equip-install-system-clonebay'));
      await tester.pumpAndSettle();
      expect(
        tester.widget<ChangeMark>(find.byKey(const ValueKey('equip-mark-medbay'))).direction,
        EditDirection.lowered,
      );

      await tester.tap(saveButton);
      await tester.pumpAndSettle();
      for (final id in ['mind', 'teleporter', 'clonebay']) {
        expect(find.byKey(ValueKey('change-systems.$id.installed')), findsOneWidget);
      }
      await writeFromSheet(tester);

      final equipment = equipmentOf(
        session: await openSave(path: save.path),
        nearby: false,
        language: 'en',
      );
      int? level(String id) => equipment.systems.firstWhere((s) => s.id == id).level?.value;
      expect([level('mind'), level('teleporter'), level('clonebay'), level('medbay')], [null, 1, 1, null]);
    },
  );

  testWidgets('a freed place takes a new system whatever order the clicks staged; Save writes it', (tester) async {
    final dat = await installedDat('install order');
    if (dat == null) return;
    // The Kestrel of the fixture has five systems: mind control, cloaking
    // and the drone control make eight, the most FTL's store sells.
    await prepare(const [
      FieldEdit(path: 'systems.mind.installed', value: 1),
      FieldEdit(path: 'systems.cloaking.installed', value: 1),
      FieldEdit(path: 'systems.drones.installed', value: 1),
    ]);
    await rememberFtlDat(configDir: configDir(), picked: dat);
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(realLocate: true));
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
    await tester.tap(tab('equipment'));
    await tester.pumpAndSettle();

    Finder button(String key) => find.byKey(ValueKey(key));
    bool enabled(String key) =>
        tester.widget<AppButton>(find.ancestor(of: button(key), matching: find.byType(AppButton)).first).onPressed !=
        null;
    Future<void> click(String key) async {
      expect(enabled(key), isTrue, reason: key);
      await tapVisible(tester, button(key));
      await tester.pumpAndSettle();
    }

    // Swap mind control for the teleporter, free a second place, then put
    // mind control back: the teleporter install now stands before the
    // removals that make room for it.
    await click('equip-remove-system-mind');
    await click('equip-install-system-teleporter');
    await click('equip-remove-system-cloaking');
    await click('equip-install-system-mind');
    expect(find.byKey(const ValueKey('equip-preview-error')), findsNothing);
    expect(enabled('equip-install-system-hacking'), isFalse, reason: 'eight systems again');
    // Removing mind control once more frees a place: hacking can go there.
    await click('equip-remove-system-mind');
    expect(find.byKey(const ValueKey('equip-preview-error')), findsNothing);
    expect(enabled('equip-install-system-cloaking'), isTrue);
    await click('equip-install-system-hacking');
    expect(enabled('equip-install-system-cloaking'), isFalse, reason: 'eight systems again');

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    await writeFromSheet(tester);
    final equipment = equipmentOf(
      session: await openSave(path: save.path),
      nearby: false,
      language: 'en',
    );
    int? level(String id) => equipment.systems.firstWhere((s) => s.id == id).level?.value;
    expect([level('teleporter'), level('hacking'), level('mind'), level('cloaking')], [1, 1, null, null]);
  });

  testWidgets('the Equipment page lowers the reactor below 8 and a system below its power; Save writes both', (
    tester,
  ) async {
    final dat = await installedDat('lowering');
    if (dat == null) return;
    await rememberFtlDat(configDir: configDir(), picked: dat);
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(realLocate: true));
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
    await tester.tap(tab('equipment'));
    await tester.pumpAndSettle();

    // The fixture: reactor 8 with 8 bars in use, shields level 2 drawing 2.
    // The fifth reactor segment sets 5, the first shields segment level 1.
    await tapVisible(tester, find.byKey(const ValueKey('equip-cell-reactor-4')));
    await tester.pumpAndSettle();
    await tapVisible(tester, find.byKey(const ValueKey('equip-cell-shields-0')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('equip-seg-shields-1-lowered')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-seg-reactor-5-lowered')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-mark-shields')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-mark-reactor')), findsOneWidget);

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('change-reactor')), findsOneWidget);
    expect(find.byKey(const ValueKey('change-systems.shields.level')), findsOneWidget);
    await writeFromSheet(tester);

    final equipment = equipmentOf(
      session: await openSave(path: save.path),
      nearby: false,
      language: 'en',
    );
    expect(equipment.reactor.value, 5);
    final shields = equipment.systems.firstWhere((s) => s.id == 'shields');
    expect(shields.level!.value, 1);
    expect(shields.power, lessThanOrEqualTo(1));
    final inUse = equipment.systems.where((s) => !s.subsystem).fold(0, (n, s) => n + s.power);
    expect(inUse, lessThanOrEqualTo(5));
    expect(Directory('${dir.path}/backups').listSync(), hasLength(1));
  });

  testWidgets(
    'with ftl.dat the Ship page draws the floorplan; a room opens its system, a square its menu; Save writes',
    (tester) async {
      final dat = await installedDat('floorplan');
      if (dat == null) return;
      await rememberFtlDat(configDir: configDir(), picked: dat);
      SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
      await tester.pumpWidget(app(realLocate: true));
      final plan = find.byKey(const ValueKey('floorplan'));
      await pumpUntil(tester, () => plan.evaluate().isNotEmpty, 'the floorplan');
      Finder planImage(String ref) =>
          find.descendant(of: plan, matching: find.byWidgetPredicate((w) => w is GameImage && w.ref.toString() == ref));
      bool drawn(String ref) => find
          .descendant(of: planImage(ref), matching: find.byType(RawImage))
          .evaluate()
          .any((e) => (e.widget as RawImage).image != null);
      await pumpUntil(tester, () => drawn('img/ship/kestral_base.png'), 'the hull of the floorplan');
      await pumpUntil(tester, () => drawn('img/ship/interior/room_shields.png'), 'the shields interior');
      // A human's look id carries its tints: ask the core for Berko's.
      final berko = crewOf(session: await openSave(path: save.path), nearby: false)[0].lookId;
      await pumpUntil(tester, () => drawn('crewLookFrame:$berko'), 'the crew frames');
      expect(find.byKey(const ValueKey('floor-crew-2')), findsOneWidget);

      // Room 5 holds the shields; its system symbol opens the popover.
      await tapVisible(tester, find.byKey(const ValueKey('floor-room-5')));
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('room-popover-5')), findsOneWidget);
      await tester.tap(find.byKey(const ValueKey('equip-cell-shields-2')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('popover-close')));
      await tester.pumpAndSettle();
      // The room keeps a mark of the pending level after the popover closed.
      expect(find.byKey(const ValueKey('floor-mark-shields')), findsOneWidget);

      await tapVisible(tester, find.byKey(const ValueKey('floor-crew-2')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('square-crew-2')));
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('crew-popover-2')), findsOneWidget);
      expect(findLabel('Elizabeth'), findsWidgets);
      await tester.tapAt(const Offset(4, 4));
      await tester.pumpAndSettle();

      // The bottom right square of the shields (grid 9, 6) gets a fire and a
      // breach through its menu.
      final square = find.byKey(const ValueKey('floor-square-5-3'));
      for (final pick in ['square-fire', 'square-breach']) {
        await tapVisible(tester, square);
        await tester.pumpAndSettle();
        await tester.tap(find.byKey(ValueKey(pick)));
        await tester.pumpAndSettle();
      }
      expect(find.byKey(const ValueKey('square-mark-5-3')), findsOneWidget);

      // The menu of the same square sets the oxygen of the shields room.
      await tapVisible(tester, square);
      await tester.pumpAndSettle();
      expect(findLabel('Oxygen 100%'), findsOneWidget);
      await tester.tap(find.byKey(const ValueKey('square-oxygen')));
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(EditableText), '10');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('oxygen-mark-5')), findsOneWidget);
      expect(find.byKey(const ValueKey('floor-o2-low-5')), findsNothing, reason: 'FTL stripes only 5 or less');

      await tester.tap(saveButton);
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('change-systems.shields.level')), findsOneWidget);
      expect(find.byKey(const ValueKey('change-fires.9.6')), findsOneWidget);
      expect(find.byKey(const ValueKey('change-breaches.9.6')), findsOneWidget);
      expect(find.byKey(const ValueKey('change-rooms.5.oxygen')), findsOneWidget);
      await writeFromSheet(tester);
      final written = await openSave(path: save.path);
      final equipment = equipmentOf(session: written, nearby: false, language: 'en');
      expect(equipment.systems.firstWhere((s) => s.id == 'shields').level!.value, 3);
      final q = floorplanOf(session: written, nearby: false)!.rooms[5].squares[3];
      expect((q.fire.value, q.breach.value), (100, 100));
      expect(floorplanOf(session: written, nearby: false)!.rooms[5].oxygen, 10);
      expect(Directory('${dir.path}/backups').listSync(), hasLength(1));
    },
  );

  testWidgets('the enemy ship lists its weapons and drones in slots; one is added, one unpowered; Save writes both', (
    tester,
  ) async {
    final dat = await installedDat('enemy weapons');
    if (dat == null) return;
    await rememberFtlDat(configDir: configDir(), picked: dat);
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(realLocate: true));
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
    await tapVisible(tester, find.byKey(const ValueKey('ship-nearby')));
    await tester.pumpAndSettle();
    await tester.tap(tab('equipment'));
    await tester.pumpAndSettle();

    // JELLY_BUTTON_P_DLC: two weapons in FTL's default four slots, no drones in four.
    for (final i in [0, 1]) {
      expect(find.byKey(ValueKey('equip-weapon-$i')), findsOneWidget);
    }
    for (final i in [2, 3]) {
      expect(find.byKey(ValueKey('equip-empty-weapon-$i')), findsOneWidget);
    }
    expect(find.byKey(const ValueKey('equip-empty-drone-3')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-add-cargo')), findsNothing);

    await tapVisible(tester, find.byKey(const ValueKey('equip-add-weapon')));
    await tester.pumpAndSettle();
    final ion = itemChoices(language: 'en').firstWhere((c) => c.id == 'ION_1');
    await tester.enterText(find.byKey(const ValueKey('item-filter')), ion.name);
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('item-choice-ION_1')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('equip-weapon-2')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-empty-weapon-2')), findsNothing);
    await tapVisible(tester, find.byKey(const ValueKey('equip-power-weapon-0')));
    await tester.pumpAndSettle();

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    for (final path in ['nearby.weapons.ION_1', 'nearby.weapons.LASER_BURST_1.armed']) {
      expect(find.byKey(ValueKey('change-$path')), findsOneWidget, reason: path);
    }
    await writeFromSheet(tester);
    final enemy = equipmentOf(
      session: await openSave(path: save.path),
      nearby: true,
      language: 'en',
    );
    expect(
      [for (final w in enemy.weapons) (w.id, w.armed)],
      [('LASER_BURST_1', false), ('BEAM_1', true), ('ION_1', false)],
    );
  });

  testWidgets(
    'a weapon mount powers the Kestrel weapon off; the enemy Weapon Control popover unpowers its laser; Save writes both',
    (tester) async {
      final dat = await installedDat('mounts');
      if (dat == null) return;
      await rememberFtlDat(configDir: configDir(), picked: dat);
      SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
      await tester.pumpWidget(app(realLocate: true));
      final plan = find.byKey(const ValueKey('floorplan'));
      await pumpUntil(tester, () => plan.evaluate().isNotEmpty, 'the floorplan');
      // The Kestrel: Artemis on mount 0, Burst Laser II on mount 1, two free.
      for (final slot in [0, 1]) {
        expect(find.byKey(ValueKey('floor-weapon-$slot')), findsOneWidget);
      }
      expect(find.byKey(const ValueKey('floor-weapon-2')), findsNothing);
      expect(find.byKey(const ValueKey('floor-mount-3')), findsOneWidget);
      await tapVisible(tester, find.byKey(const ValueKey('floor-mount-1')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('item-power')));
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('mount-mark-1')), findsOneWidget);

      // The pirate: its Weapon Control popover lists the weapons with the
      // Equipment rows; the Basic Laser goes off.
      await tapVisible(tester, find.byKey(const ValueKey('ship-nearby')));
      await tester.pumpAndSettle();
      await pumpUntil(
        tester,
        () => find.byKey(const ValueKey('floor-mount-0')).evaluate().isNotEmpty,
        'the enemy mounts',
      );
      final weapons = floorplanOf(
        session: await openSave(path: save.path),
        nearby: true,
      )!.rooms.firstWhere((r) => r.system == 'weapons');
      await tapVisible(tester, find.byKey(ValueKey('floor-room-${weapons.id}')));
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('equip-weapon-1')), findsOneWidget);
      expect(find.byKey(const ValueKey('equip-to-cargo-weapon-0')), findsNothing, reason: 'no hold on the enemy ship');
      await tester.tap(find.byKey(const ValueKey('equip-power-weapon-0')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('popover-close')));
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('mount-mark-0')), findsOneWidget);

      await tester.tap(saveButton);
      await tester.pumpAndSettle();
      for (final path in ['weapons.LASER_BURST_3.armed', 'nearby.weapons.LASER_BURST_1.armed']) {
        expect(find.byKey(ValueKey('change-$path')), findsOneWidget, reason: path);
      }
      await writeFromSheet(tester);
      final written = await openSave(path: save.path);
      final player = equipmentOf(session: written, nearby: false, language: 'en');
      final enemy = equipmentOf(session: written, nearby: true, language: 'en');
      expect(
        [for (final w in player.weapons) (w.id, w.armed)],
        [('MISSILES_2_PLAYER', true), ('LASER_BURST_3', false)],
      );
      expect([for (final w in enemy.weapons) (w.id, w.armed)], [('LASER_BURST_1', false), ('BEAM_1', true)]);
    },
  );

  testWidgets('the floorplan follows staged edits: Cloaking out, the teleporter in, both room menus say so', (
    tester,
  ) async {
    final dat = await installedDat('floorplan preview');
    if (dat == null) return;
    // The Kestrel of the fixture has no cloaking: it comes first.
    await prepare(const [FieldEdit(path: 'systems.cloaking.installed', value: 1)]);
    await rememberFtlDat(configDir: configDir(), picked: dat);
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(realLocate: true));
    final plan = find.byKey(const ValueKey('floorplan'));
    await pumpUntil(tester, () => plan.evaluate().isNotEmpty, 'the floorplan');
    // Room 8 holds the cloaking of the file, room 15 is the teleporter room
    // of the blueprint (data/blueprints.xml, PLAYER_SHIP_HARD).
    expect(find.byKey(const ValueKey('floor-symbol-cloaking')), findsOneWidget);

    await tester.tap(tab('equipment'));
    await tester.pumpAndSettle();
    for (final key in ['equip-remove-system-cloaking', 'equip-install-system-teleporter']) {
      await tapVisible(tester, find.byKey(ValueKey(key)));
      await tester.pumpAndSettle();
    }
    await tester.tap(tab('ship'));
    await tester.pumpAndSettle();

    expect(find.byKey(const ValueKey('floor-symbol-cloaking')), findsNothing);
    expect(find.byKey(const ValueKey('floor-symbol-teleporter')), findsOneWidget);
    final name = equipmentOf(
      session: await openSave(path: save.path),
      nearby: false,
      language: 'en',
    ).systems.firstWhere((s) => s.id == 'teleporter').name!;
    Finder titled(String text) => find.byWidgetPredicate(
      (w) =>
          (w is Text && w.data?.toLowerCase() == text.toLowerCase()) ||
          (w is FtlText && w.text.toLowerCase() == text.toLowerCase()),
    );

    // The teleporter room names the teleporter and lists it.
    await tapVisible(tester, find.byKey(const ValueKey('floor-square-15-0')));
    await tester.pumpAndSettle();
    final menu = find.byWidgetPredicate(
      (w) => w.key is ValueKey<String> && (w.key! as ValueKey<String>).value.startsWith('square-menu-'),
    );
    expect(menu, findsOneWidget);
    expect(find.descendant(of: menu, matching: titled(name)), findsOneWidget);
    expect(find.descendant(of: menu, matching: find.byKey(const ValueKey('square-system'))), findsOneWidget);
    await tester.tapAt(const Offset(4, 4));
    await tester.pumpAndSettle();

    // The old cloaking room is a plain room now.
    await tapVisible(tester, find.byKey(const ValueKey('floor-square-8-0')));
    await tester.pumpAndSettle();
    expect(menu, findsOneWidget);
    expect(find.descendant(of: menu, matching: titled('Room 8')), findsOneWidget);
    expect(find.descendant(of: menu, matching: find.byKey(const ValueKey('square-system'))), findsNothing);
  });

  testWidgets('the core checks the open file for FTL and Steam Cloud', (tester) async {
    await tester.pumpWidget(app(pick: save.path, realWarnings: true));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await pumpUntil(tester, () => find.text('Steam Cloud off').evaluate().isNotEmpty, 'the warnings');
    expect(find.text('FTL: unknown'), findsNothing, reason: 'the process list of macOS is readable');

    await File('${dir.path}/steam_autocloud.vdf').writeAsString('');
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await pumpUntil(tester, () => find.text('Steam Cloud on').evaluate().isNotEmpty, 'the warnings after resume');
  });

  testWidgets('a running FTL asks before a save; Cancel writes nothing', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    expect(find.text('FTL is closed'), findsOneWidget);

    // FTL starts while the editor is in the background.
    warnings = [const FileWarning(code: 'ftl_running', message: 'FTL is running (pid 4242)')];
    await openResource(tester, 'ship.scrap');
    await tester.enterText(popoverField, '9999');
    await apply(tester);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pumpAndSettle();
    expect(find.text('FTL is running'), findsOneWidget, reason: 'the focus came back');
    expect(find.text('9999'), findsOneWidget, reason: 'a resume keeps the edit');

    final reads = warningReads;
    final saveButton = find.widgetWithIcon(FilledButton, Icons.save);
    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    expect(warningReads, reads + 1, reason: 'a save reads the warnings again');
    expect(find.byKey(const ValueKey('ftl-running-dialog')), findsOneWidget);
    await tester.tap(
      find.descendant(of: find.byKey(const ValueKey('ftl-running-dialog')), matching: find.text('Cancel')),
    );
    await tester.pumpAndSettle();
    expect(await File(save.path).readAsBytes(), await File(fixture).readAsBytes());
    expect(Directory('${dir.path}/backups').existsSync(), isFalse);

    await tester.tap(saveButton);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Save anyway'));
    await tester.pumpAndSettle();
    await writeFromSheet(tester);
    final reread = await openSave(path: save.path);
    expect(reread.summary().fields.firstWhere((f) => f.path == 'ship.scrap').value, 9999);
  });

  testWidgets('Steam Cloud does not stop a save and names its risk', (tester) async {
    warnings = [const FileWarning(code: 'steam_cloud', message: 'Steam Cloud synchronises this folder')];
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    expect(find.text('Steam Cloud on'), findsOneWidget);
    await openResource(tester, 'ship.scrap');
    await tester.enterText(popoverField, '9999');
    await apply(tester);
    await tester.tap(find.widgetWithIcon(FilledButton, Icons.save));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('ftl-running-dialog')), findsNothing);
    expect(find.byKey(const ValueKey('change-sheet-steam')), findsOneWidget);
    await writeFromSheet(tester);
    final reread = await openSave(path: save.path);
    expect(reread.summary().fields.firstWhere((f) => f.path == 'ship.scrap').value, 9999);
  });

  int valueIn(SaveSession s, String path) => s.summary().fields.firstWhere((f) => f.path == path).value;

  testWidgets('Save lists every change; an undo drops one; Write saves the rest', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    final inFile = await openSave(path: save.path);

    await openResource(tester, 'ship.scrap');
    await tester.enterText(popoverField, '9999');
    await apply(tester);
    await openResource(tester, 'ship.fuel');
    await tester.enterText(popoverField, '77');
    await apply(tester);
    await tester.tap(find.widgetWithIcon(FilledButton, Icons.save));
    await tester.pumpAndSettle();

    final fuel = find.byKey(const ValueKey('change-ship.fuel'));
    expect(find.descendant(of: fuel, matching: findChange(valueIn(inFile, 'ship.fuel'), 77)), findsOneWidget);
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('change-ship.scrap')),
        matching: findChange(valueIn(inFile, 'ship.scrap'), 9999),
      ),
      findsOneWidget,
    );
    expect(await File(save.path).readAsBytes(), await File(fixture).readAsBytes(), reason: 'the sheet writes nothing');
    await tester.tap(find.byKey(const ValueKey('undo-ship.fuel')));
    await tester.pumpAndSettle();
    expect(fuel, findsNothing);
    await writeFromSheet(tester);

    final reread = await openSave(path: save.path);
    expect(valueIn(reread, 'ship.scrap'), 9999);
    expect(valueIn(reread, 'ship.fuel'), valueIn(inFile, 'ship.fuel'), reason: 'the undone edit is not written');
    expect(Directory('${dir.path}/backups').listSync(), hasLength(1));
  });

  testWidgets('Undo and Redo walk every staged edit, a quick action in one step; a save clears them', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    final inFile = await openSave(path: save.path);
    final undo = find.byKey(const ValueKey('undo-button'));
    final redo = find.byKey(const ValueKey('redo-button'));
    bool enabled(Finder f) => tester.widget<IconButton>(f).onPressed != null;
    Finder shown(String path, int value) =>
        find.descendant(of: find.byKey(ValueKey('resource-$path')), matching: find.text('$value'));
    expect((enabled(undo), enabled(redo)), (false, false));

    await openResource(tester, 'ship.scrap');
    await tester.enterText(popoverField, '9999');
    await apply(tester);
    await openResource(tester, 'ship.fuel');
    await tester.enterText(popoverField, '77');
    await apply(tester);
    // Max everything stages its edits as one step; its change sheet is cancelled.
    await tester.tap(find.byKey(const ValueKey('action-max-everything')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    final maxed = shown('ship.fuel', 77).evaluate().isEmpty;
    expect(maxed, isTrue, reason: 'max everything raised the fuel');

    await tester.tap(undo);
    await tester.pumpAndSettle();
    expect(shown('ship.fuel', 77), findsOneWidget, reason: 'one undo takes back the whole action');
    expect(shown('ship.scrap', 9999), findsOneWidget);
    await tester.tap(undo);
    await tester.pumpAndSettle();
    expect(shown('ship.fuel', valueIn(inFile, 'ship.fuel')), findsOneWidget);
    expect(shown('ship.scrap', 9999), findsOneWidget);
    expect(enabled(redo), isTrue);
    await tester.tap(redo);
    await tester.pumpAndSettle();
    expect(shown('ship.fuel', 77), findsOneWidget);

    // A new edit drops the redo step.
    await tester.tap(redo);
    await tester.pumpAndSettle();
    await tester.tap(undo);
    await tester.pumpAndSettle();
    await openResource(tester, 'ship.missiles');
    await tester.enterText(popoverField, '9');
    await apply(tester);
    expect(enabled(redo), isFalse);

    await tester.tap(find.widgetWithIcon(FilledButton, Icons.save));
    await tester.pumpAndSettle();
    await writeFromSheet(tester);
    final reread = await openSave(path: save.path);
    expect(valueIn(reread, 'ship.fuel'), 77);
    expect(valueIn(reread, 'ship.missiles'), 9);
    expect((enabled(undo), enabled(redo)), (false, false), reason: 'a save clears the steps');
  });

  testWidgets('the backup history restores a backup and backs up the current file', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    await openResource(tester, 'ship.scrap');
    await tester.enterText(popoverField, '9999');
    await apply(tester);
    await tester.tap(find.widgetWithIcon(FilledButton, Icons.save));
    await tester.pumpAndSettle();
    await writeFromSheet(tester);
    final edited = await File(save.path).readAsBytes();
    final first = Directory('${dir.path}/backups').listSync().single.path;

    expect(find.byType(SnackBar), findsOneWidget, reason: 'the save message opens above the status bar');
    await tester.tap(find.byKey(const ValueKey('status-backup')));
    await pumpUntil(tester, () => find.byKey(ValueKey('backup-$first')).evaluate().isNotEmpty, 'the backup list');
    await tester.tap(find.descendant(of: find.byKey(ValueKey('backup-$first')), matching: find.text('Restore')));
    await tester.pumpAndSettle();
    await tester.tap(
      find.descendant(of: find.byKey(const ValueKey('restore-confirm')), matching: find.text('Restore')),
    );
    await pumpUntil(tester, () => find.byKey(const ValueKey('backup-history')).evaluate().isEmpty, 'the restore');

    expect(await File(save.path).readAsBytes(), await File(fixture).readAsBytes());
    final backups = Directory('${dir.path}/backups').listSync().map((e) => e.path).toList();
    expect(backups, hasLength(2));
    final current = backups.singleWhere((p) => p != first);
    expect(await File(current).readAsBytes(), edited, reason: 'the replaced file is backed up first');
    expect(find.byTooltip(current), findsOneWidget, reason: 'the status bar names the new backup');
    expect(find.text('9999'), findsNothing, reason: 'the page shows the restored file');
  });

  testWidgets('the max button fills the maximum; the file value again clears the edit', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    final saveButton = find.widgetWithIcon(FilledButton, Icons.save);
    final inFile = (await openSave(path: save.path)).summary().fields.firstWhere((f) => f.path == 'ship.scrap').value;

    await openResource(tester, 'ship.scrap');
    await tester.tap(find.byTooltip('Set to maximum (99999)'));
    await tester.pump();
    expect(tester.widget<TextField>(popoverField).controller!.text, '99999');
    await apply(tester);
    expect(find.text('99999'), findsOneWidget);
    expect(tester.widget<FilledButton>(saveButton).onPressed, isNotNull);

    await openResource(tester, 'ship.scrap');
    await tester.enterText(popoverField, '$inFile');
    await apply(tester);
    expect(tester.widget<FilledButton>(saveButton).onPressed, isNull, reason: 'back to the file value');
  });

  testWidgets('a value out of range is refused and changes nothing', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();

    await openResource(tester, 'ship.scrap');
    await tester.enterText(popoverField, '100000');
    await apply(tester);
    expect(find.text('Allowed: 0 to 99999'), findsOneWidget);
    await tester.tap(find.descendant(of: popover, matching: find.text('Cancel')));
    await tester.pumpAndSettle();
    expect(tester.widget<FilledButton>(find.widgetWithIcon(FilledButton, Icons.save)).onPressed, isNull);
  });

  testWidgets('a missing save shows the localized text, not the Rust message', (tester) async {
    await tester.pumpWidget(app());
    await tester.pumpAndSettle();

    // At start the page opens the default file, which does not exist.
    // ARB text (capital N); the Rust message starts with a lowercase "no".
    expect(find.textContaining('No saved game at ${dir.path}/missing/continue.sav'), findsOneWidget);
  });

  testWidgets('the last opened file opens again at start', (tester) async {
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app());
    await tester.pumpAndSettle();

    expect(find.text('The Kestrel'), findsOneWidget);
    expect(find.text(dir.path), findsOneWidget);
  });

  testWidgets('a cancelled dialog keeps the open file', (tester) async {
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(pick: null));
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    expect(find.text('The Kestrel'), findsOneWidget);
  });

  testWidgets('a click on the file name in the status bar opens the saved-game chooser', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    expect(find.text('The Kestrel'), findsNothing);

    await tester.tap(find.byKey(const ValueKey('status-file')));
    await tester.pumpAndSettle();
    expect(find.text('The Kestrel'), findsOneWidget);
    expect(
      find.descendant(of: find.byKey(const ValueKey('status-file')), matching: find.text('continue.sav')),
      findsOneWidget,
    );
  });

  testWidgets('the editor runs under a German system locale', (tester) async {
    // material_ui widgets need material_ui's own localization delegates;
    // without them every AppBar and TextField throws under `de`.
    tester.platformDispatcher.localesTestValue = const [Locale('de')];
    addTearDown(tester.platformDispatcher.clearLocalesTestValue);
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();

    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    expect(find.text('The Kestrel'), findsOneWidget);
    expect(find.text('Speichern'), findsOneWidget);
    expect(find.text('Ressourcen'), findsOneWidget);
  });

  testWidgets('without ftl.dat the editor runs in reduced mode and says so', (tester) async {
    await tester.pumpWidget(app(pick: save.path));
    await tester.pumpAndSettle();
    expect(find.text('No ftl.dat found'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    expect(find.text('The Kestrel'), findsOneWidget);
    expect(find.descendant(of: find.byType(ShipCard), matching: find.byIcon(Icons.rocket_launch)), findsOneWidget);
    expect(find.text('Reduced mode: no ftl.dat'), findsOneWidget);
    expect(shipImage(), findsNothing);
    expect(find.byType(FtlTabStrip), findsNothing, reason: 'without ftl.dat the app keeps the Material look');
    expect(find.byType(AppBar), findsOneWidget);
    expect(File('${configDir()}/config.json').existsSync(), isFalse);
    expect(find.byKey(const ValueKey('floorplan')), findsNothing, reason: 'the Ship page as without the floorplan');
  });

  testWidgets('a found ftl.dat is confirmed once and the ship card shows the hull', (tester) async {
    final dat = await installedDat('confirm');
    if (dat == null) return;
    await tester.pumpWidget(app(pick: save.path, realLocate: true));
    await pumpUntil(tester, () => find.text('Game data found').evaluate().isNotEmpty, 'the located ftl.dat');
    expect(find.text(dat), findsOneWidget);

    await tapVisible(tester, find.text('Use this ftl.dat'));
    await pumpUntil(tester, () => find.byType(GameDataCard).evaluate().isEmpty, 'the game data to open');
    final config = await File('${configDir()}/config.json').readAsString();
    expect(config, contains(dat));

    await tester.tap(chooseFile);
    await tester.pumpAndSettle();
    expect(findLabel('The Kestrel'), findsOneWidget);
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
    expect(find.text('Reduced mode: no ftl.dat'), findsNothing);
  });

  testWidgets('a remembered ftl.dat opens at start without a question', (tester) async {
    final dat = await installedDat('remembered');
    if (dat == null) return;
    await rememberFtlDat(configDir: configDir(), picked: dat);
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(realLocate: true));
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
    expect(find.byType(GameDataCard), findsNothing);
  });

  testWidgets('with ftl.dat the editor wears the original FTL look', (tester) async {
    final dat = await installedDat('ftl look');
    if (dat == null) return;
    await rememberFtlDat(configDir: configDir(), picked: dat);
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(realLocate: true));
    await pumpUntil(tester, () => find.byType(FtlTabStrip).evaluate().isNotEmpty, 'the FTL tab row');
    expect(find.byType(AppBar), findsNothing);
    // The area tabs; the switch of the enemy ship is a second tab strip.
    final areaTabs = find.byKey(const ValueKey('area-tabs'));
    final strip = tester.widget<FtlTabStrip>(areaTabs);
    expect(strip.tabs, hasLength(5));
    expect(strip.open, 0, reason: 'the Ship page is open');
    // The frames and the fonts come from the real ftl.dat.
    final look = FtlTheme.of(tester.element(areaTabs))!;
    final window = look.image(FtlImages.window);
    expect((window.image.width, window.image.height), (66, 72));
    expect(look.font(FtlFontId.justin12Bold).glyphs, isNotEmpty);
    final page = find.byKey(const ValueKey('ftl-page-window'));
    expect(find.descendant(of: page, matching: find.byType(FtlFrame)), findsWidgets);
    expect(find.descendant(of: page, matching: find.byType(ShipCard)), findsOneWidget);
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
    expect(findLabel('The Kestrel'), findsOneWidget, reason: 'the ship name in the FTL font');
    expect(tester.getRect(find.byType(FtlFrame).first).width, greaterThan(66), reason: 'the frame stretches');
  });

  testWidgets('without a found ftl.dat the person picks FTL.app and it is remembered', (tester) async {
    final dat = await installedDat('pick');
    if (dat == null) return;
    // .../FTL.app/Contents/Resources/ftl.dat -> .../FTL.app (macOS; elsewhere the folder of ftl.dat).
    final resources = File(dat).parent;
    final picked = resources.path.endsWith('/Contents/Resources') ? resources.parent.parent.path : resources.path;
    await tester.pumpWidget(app(pick: save.path, pickDat: picked));
    await tester.pumpAndSettle();

    await tapVisible(tester, find.text('Choose ftl.dat'));
    await pumpUntil(tester, () => find.byType(GameDataCard).evaluate().isEmpty, 'the game data to open');
    final status = await ftlDatStatus(configDir: configDir());
    expect((status.path, status.source), (dat, DatSource.remembered));

    await tester.tap(chooseFile);
    await tester.pumpAndSettle();
    await pumpUntil(tester, () => shipImageShown(tester), 'the hull image');
  });

  testWidgets('a click on the game data item picks another ftl.dat and remembers it; the resources wear its icons', (
    tester,
  ) async {
    final dat = await installedDat('status game data');
    if (dat == null) return;
    final resources = File(dat).parent;
    final picked = resources.path.endsWith('/Contents/Resources') ? resources.parent.parent.path : resources.path;
    SharedPreferences.setMockInitialValues({SaveLocation.lastFileKey: save.path});
    await tester.pumpWidget(app(pickDat: picked));
    await tester.pumpAndSettle();
    expect(find.byIcon(Icons.local_gas_station), findsOneWidget, reason: 'reduced mode: the Material icon');

    await tester.tap(find.byKey(const ValueKey('status-game-data')));
    await pumpUntil(tester, () => find.byType(FtlTabStrip).evaluate().isNotEmpty, 'the game data to open');
    final status = await ftlDatStatus(configDir: configDir());
    expect((status.path, status.source), (dat, DatSource.remembered));
    expect(findLabel('Game data: ftl.dat'), findsOneWidget);

    // The five resources show FTL's own icons from the picked ftl.dat.
    for (final path in ['ship.hull', 'ship.fuel', 'ship.drone_parts', 'ship.missiles', 'ship.scrap']) {
      final image = find.descendant(of: find.byKey(ValueKey('resource-$path')), matching: find.byType(RawImage));
      await pumpUntil(
        tester,
        () => image.evaluate().isNotEmpty && tester.widget<RawImage>(image).image != null,
        'the game icon of $path',
      );
    }
    expect(find.byIcon(Icons.local_gas_station), findsNothing);
  });
}
