// Flow tests of the one-click actions and the command palette over the real
// app and the real Rust core (macOS), in reduced mode (no ftl.dat in the UI):
//
//   bun scripts/integration-test.ts
//
// Each test copies the fixture into its own temp folder with its own backup
// folder and config folder, so no test touches the FTL save folder, the real
// preferences or the editor's data folder.

import 'dart:io';

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/main.dart';
import 'package:ftl_editor/src/rust/api/actions.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/equipment.dart';
import 'package:ftl_editor/src/rust/api/status.dart';
import 'package:ftl_editor/src/rust/frb_generated.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/editor_page.dart';
import 'package:ftl_editor/ui/quick_actions.dart';
import 'package:integration_test/integration_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:shared_preferences/shared_preferences.dart';

const fixture = '../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav';

/// The old and new value of a change sheet row.
Finder findChange(int old, int value) =>
    find.byWidgetPredicate((w) => w is ChangeValues && w.old == '$old' && w.value == '$value');

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  WidgetController.hitTestWarningShouldBeFatal = true;
  setUpAll(() async => RustLib.init());

  late Directory dir;
  late File save;
  setUp(() async {
    dir = await Directory.systemTemp.createTemp('ftl-editor-actions-it');
    save = File('${dir.path}/continue.sav');
    await File(fixture).copy(save.path);
    SharedPreferences.setMockInitialValues({});
  });
  tearDown(() async => dir.delete(recursive: true));

  Widget app() => FtlEditorApp(
    home: EditorPage(
      backupDir: '${dir.path}/backups',
      defaultFile: save.path,
      pickFile: (_, _) async => null,
      assetsConfigDir: '${dir.path}/config',
      locateFtlDat: () async => const FtlDatStatus(tried: ['/nowhere/ftl.dat']),
      pickFtlDat: (_) async => null,
      warningsOf: (_) async => const <FileWarning>[],
    ),
  );

  Future<void> start(WidgetTester tester) async {
    await tester.pumpWidget(app());
    await tester.pumpAndSettle();
    expect(find.text('The Kestrel'), findsOneWidget, reason: 'the default file opened');
  }

  int field(SaveSession s, String path) => s.summary().fields.firstWhere((f) => f.path == path).value;

  test('the buttons name the actions of the core in its order', () {
    expect(quickActions.map((a) => a.id), actionIds());
  });

  testWidgets('max everything goes through the change sheet and the file holds it', (tester) async {
    await start(tester);
    // The core plans with the blueprint of an installed ftl.dat even in
    // reduced mode (SaveFile::open loads it for the ship layouts); without
    // one the hull goes to the editor cap and a note says so.
    final planned = planAction(
      session: await openSave(path: save.path),
      action: 'max-everything',
    );
    final hull = planned.edits.firstWhere((e) => e.path == 'ship.hull').value;
    await tester.tap(find.byKey(const ValueKey('action-max-everything')));
    await tester.pumpAndSettle();

    expect(find.byKey(const ValueKey('change-sheet')), findsOneWidget);
    // A centered modal in the look of the page: a quick action once opened a
    // bottom sheet in the Material look from a context above the look's Theme.
    expect(find.byType(BottomSheet), findsNothing);
    expect(
      find.ancestor(of: find.byKey(const ValueKey('change-sheet')), matching: find.byType(Dialog)),
      findsOneWidget,
    );
    expect(
      FtlTheme.of(tester.element(find.byKey(const ValueKey('change-sheet')))) != null,
      FtlTheme.of(tester.element(find.byType(QuickActionBar))) != null,
      reason: 'the change sheet has the look of the page',
    );
    // The list builds only the rows in view: scroll each row into view first.
    final rows = find.descendant(of: find.byKey(const ValueKey('change-sheet')), matching: find.byType(Scrollable));
    Future<Finder> row(String path) async {
      final f = find.byKey(ValueKey('change-$path'));
      await tester.scrollUntilVisible(f, 60, scrollable: rows.first);
      return f;
    }

    final scrap = await row('ship.scrap');
    expect(find.descendant(of: scrap, matching: findChange(53, 99999)), findsOneWidget);
    expect(await row('reactor'), findsOneWidget);
    expect(await row('systems.shields.level'), findsOneWidget);
    expect(find.byKey(ValueKey('change-sheet-note-${planned.notes.length}')), findsNothing);
    for (var i = 0; i < planned.notes.length; i++) {
      expect(find.byKey(ValueKey('change-sheet-note-$i')), findsOneWidget, reason: planned.notes[i].code);
    }
    if (planned.notes.any((n) => n.code == 'hull_at_editor_cap')) {
      expect(find.textContaining('editor cap 999'), findsOneWidget);
    }
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await tester.pumpAndSettle();

    final reread = await openSave(path: save.path);
    expect(field(reread, 'ship.scrap'), 99999);
    expect(field(reread, 'ship.hull'), hull);
    final e = equipmentOf(session: reread, nearby: false, language: 'en');
    expect(e.reactor.value, 25);
    expect(e.systems.firstWhere((s) => s.id == 'shields').level!.value, 8);
    expect(
      planAction(session: reread, action: 'max-everything').edits,
      isEmpty,
      reason: 'nothing left to raise',
    );
    expect(Directory('${dir.path}/backups').listSync(), hasLength(1));
  });

  testWidgets('a right click breaks shield bars, the lock ionizes them; Repair ship mends both', (tester) async {
    await start(tester);
    await tester.tap(find.byKey(const ValueKey('area-equipment')));
    await tester.pumpAndSettle();
    // Shields at level 2: a right click on the first segment breaks both bars.
    final cell = find.byKey(const ValueKey('equip-cell-shields-0'));
    await tester.ensureVisible(cell);
    await tester.pumpAndSettle();
    await tester.tap(cell, buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('equip-seg-shields-1-damaged')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('equip-ion-engines')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('equip-ion-frame-engines')), findsOneWidget);

    await tester.tap(find.byKey(const ValueKey('save-button')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('change-systems.shields.damage')), findsOneWidget);
    expect(findChange(0, 2), findsOneWidget);
    expect(find.byKey(const ValueKey('change-systems.engines.ion')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await tester.pumpAndSettle();

    var e = equipmentOf(
      session: await openSave(path: save.path),
      nearby: false,
      language: 'en',
    );
    final shields = e.systems.firstWhere((s) => s.id == 'shields');
    expect((shields.damage!.value, shields.power), (2, 0), reason: 'a broken bar holds no power');
    expect(e.systems.firstWhere((s) => s.id == 'engines').ion!.value, 1);

    // The page reads the file again after the save: Repair ship mends it.
    await tester.tap(find.byKey(const ValueKey('action-repair-ship')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('change-systems.shields.damage')), findsOneWidget);
    expect(find.byKey(const ValueKey('change-systems.engines.ion')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await tester.pumpAndSettle();
    e = equipmentOf(
      session: await openSave(path: save.path),
      nearby: false,
      language: 'en',
    );
    expect(e.systems.every((s) => (s.damage?.value ?? 0) == 0 && (s.ion?.value ?? 0) == 0), isTrue);
  });

  testWidgets('Cmd+K finds ship.scrap by typing scrap; Enter opens its editor', (tester) async {
    await start(tester);
    await tester.tap(find.byKey(const ValueKey('area-crew')));
    await tester.pumpAndSettle();
    await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyK);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await tester.pumpAndSettle();
    await tester.enterText(find.byKey(const ValueKey('palette-query')), 'scrap');
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('palette-field-ship.scrap')), findsOneWidget);
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pumpAndSettle();

    expect(find.byType(ShipCard), findsOneWidget, reason: 'Enter switched to the Ship page');
    final popover = find.byKey(const ValueKey('resource-popover'));
    expect(popover, findsOneWidget);
    await tester.enterText(find.descendant(of: popover, matching: find.byType(TextField)), '5000');
    await tester.tap(find.descendant(of: popover, matching: find.text('Apply')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('save-button')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('change-ship.scrap')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await tester.pumpAndSettle();
    expect(field(await openSave(path: save.path), 'ship.scrap'), 5000);
  });
}
