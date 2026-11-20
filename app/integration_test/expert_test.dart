// The Expert area over the real app and the real Rust core (macOS): a state
// var and a raw field edited through the raw view, named in the change
// sheet and written; a raw edit that breaks the file is refused by the
// verify step of the save and leaves the file as it was.
//
//   bun scripts/integration-test.ts integration_test/expert_test.dart
//
// Like editor_test.dart: the file chooser answers a fixed path, preferences
// live in memory, the save, its backups and the editor config lie in the
// test's temp folder, and the warnings come from a fake.

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/main.dart';
import 'package:ftl_editor/shell/destination.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/expert.dart';
import 'package:ftl_editor/src/rust/frb_generated.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/editor_page.dart';
import 'package:integration_test/integration_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:shared_preferences/shared_preferences.dart';

const fixture = '../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  WidgetController.hitTestWarningShouldBeFatal = true;
  setUpAll(() async => RustLib.init());
  setUp(() => SharedPreferences.setMockInitialValues({}));

  late Directory dir;
  late File save;
  setUp(() async {
    dir = await Directory.systemTemp.createTemp('ftl-editor-expert-it');
    save = File('${dir.path}/continue.sav');
    await File(fixture).copy(save.path);
  });
  tearDown(() async => dir.delete(recursive: true));

  Widget app() => FtlEditorApp(
    home: EditorPage(
      backupDir: '${dir.path}/backups',
      defaultFile: '${dir.path}/missing/continue.sav',
      pickFile: (_, _) async => save.path,
      assetsConfigDir: '${dir.path}/config',
      locateFtlDat: () async => const FtlDatStatus(tried: ['/nowhere/ftl.dat']),
      pickFtlDat: (_) async => null,
      warningsOf: (_) async => [],
    ),
  );

  final popover = find.byKey(const ValueKey('resource-popover'));
  Finder row(String path) => find.byKey(ValueKey('expert-row-$path'));

  Future<void> open(WidgetTester tester) async {
    await tester.pumpWidget(app());
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
  }

  Future<void> goTo(WidgetTester tester, Destination item) async {
    await tester.tap(find.byKey(const ValueKey('expert-menu')));
    await tester.pumpAndSettle();
    await tester.tap(find.byWidgetPredicate((w) => w is PopupMenuItem<Destination> && w.value == item));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('expert-warning')), findsOneWidget);
  }

  Future<void> search(WidgetTester tester, String query) async {
    await tester.enterText(find.byKey(const ValueKey('expert-search')), query);
    await tester.pumpAndSettle();
  }

  Future<void> setValue(WidgetTester tester, String path, int value) async {
    await tester.ensureVisible(row(path));
    await tester.pumpAndSettle();
    await tester.tap(row(path));
    await tester.pumpAndSettle();
    expect(popover, findsOneWidget);
    await tester.enterText(find.descendant(of: popover, matching: find.byType(TextField)), '$value');
    await tester.tap(find.descendant(of: popover, matching: find.text('Apply')));
    await tester.pumpAndSettle();
  }

  Finder changeRow(String path) => find.byKey(ValueKey('change-$path'));

  testWidgets('a state var and a raw field are edited in the Expert area; Save names and writes both', (tester) async {
    await open(tester);
    await goTo(tester, Destination.stateVars);
    await search(tester, 'used_missile');
    expect(find.descendant(of: row('state_vars.2.value'), matching: find.text('used_missile')), findsOneWidget);
    await setValue(tester, 'state_vars.2.value', 3);
    expect(find.byKey(const ValueKey('expert-mark-state_vars.2.value')), findsOneWidget);

    // The codec names every unknown field of the fixture; a named raw
    // field takes the same path through the dump.
    await goTo(tester, Destination.dump);
    await search(tester, 'player_ship.crew.0.clone_dying');
    await setValue(tester, 'player_ship.crew.0.clone_dying', 5);

    await tester.tap(find.byKey(const ValueKey('save-button')));
    await tester.pumpAndSettle();
    expect(
      find.descendant(of: changeRow('state.used_missile'), matching: find.text('State var: used_missile')),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: changeRow('raw.player_ship.crew.0.clone_dying'),
        matching: find.text('Raw: player_ship.crew.0.clone_dying'),
      ),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: changeRow('state.used_missile'),
        matching: find.byWidgetPredicate((w) => w is ChangeValues && w.old == '2' && w.value == '3'),
      ),
      findsOneWidget,
    );
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await tester.pumpAndSettle();

    final dump = expertOf(session: await openSave(path: save.path));
    String value(String path) => dump.entries.firstWhere((e) => e.path == path).value;
    expect(value('state_vars.2.value'), '3');
    expect(value('player_ship.crew.0.clone_dying'), '5');
    expect(Directory('${dir.path}/backups').listSync(), hasLength(1));
  });

  testWidgets('a raw edit that breaks the file is refused by the verify and writes nothing', (tester) async {
    final before = await save.readAsBytes();
    await open(tester);
    await goTo(tester, Destination.dump);
    await search(tester, 'file_format');
    await setValue(tester, 'file_format', 2);
    await tester.tap(find.byKey(const ValueKey('save-button')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await tester.pumpAndSettle();
    expect(find.textContaining('would not read back correctly'), findsOneWidget);
    expect(await save.readAsBytes(), before);
    expect(Directory('${dir.path}/backups').existsSync(), isFalse);
  });
}
