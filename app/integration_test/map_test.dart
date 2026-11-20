// The Map page over the real app and the real Rust core (macOS): the fleet
// pushed back to the sector start and a beacon marked unvisited, named in
// the change sheet and written; the file then holds no fleet flag and the
// beacon without its visit.
//
//   bun scripts/integration-test.ts integration_test/map_test.dart
//
// Like editor_test.dart: the file chooser answers a fixed path, preferences
// live in memory, the save, its backups and the editor config lie in the
// test's temp folder, and the warnings come from a fake.

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/main.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/map.dart';
import 'package:ftl_editor/src/rust/frb_generated.dart';
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
    dir = await Directory.systemTemp.createTemp('ftl-editor-map-it');
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

  Future<void> tap(WidgetTester tester, Finder f) async {
    await tester.ensureVisible(f);
    await tester.pumpAndSettle();
    await tester.tap(f);
    await tester.pumpAndSettle();
  }

  testWidgets('the fleet goes back to the sector start and a beacon loses its visit; Save writes both', (tester) async {
    await tester.pumpWidget(app());
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    await tap(tester, find.byKey(const ValueKey('area-map')));

    await tap(tester, find.byKey(const ValueKey('fleet-start')));
    expect(find.text('0 of 21 beacons overtaken'), findsOneWidget);
    await tap(tester, find.byKey(const ValueKey('map-beacon-2')));
    expect(find.byKey(const ValueKey('beacon-popover-2')), findsOneWidget);
    await tap(tester, find.byKey(const ValueKey('beacon-unvisit-2')));
    await tap(tester, find.byKey(const ValueKey('beacon-close')));
    expect(find.byKey(const ValueKey('beacon-popover-2')), findsNothing);
    // The ship's beacon keeps its visit.
    await tap(tester, find.byKey(const ValueKey('map-beacon-8')));
    expect(find.byKey(const ValueKey('beacon-unvisit-8')), findsNothing);
    await tap(tester, find.byKey(const ValueKey('beacon-close')));

    await tap(tester, find.byKey(const ValueKey('save-button')));
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('change-fleet.offset')),
        matching: find.text('Rebel fleet position'),
      ),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('change-beacons.2.visited')),
        matching: find.text('Beacon 3: Visited'),
      ),
      findsOneWidget,
    );
    await tap(tester, find.byKey(const ValueKey('change-sheet-write')));

    final map = sectorMapOf(
      session: await openSave(path: save.path),
      language: 'en',
    );
    expect(map.fleetOffset, -959);
    expect(map.beacons.every((b) => b.fleet == 'none'), isTrue);
    expect(map.beacons[2].visited!.value, 0);
    expect(Directory('${dir.path}/backups').listSync(), hasLength(1));
  });

  testWidgets('the ship moves to beacon 3 and beacon 4 becomes a store; Save writes both', (tester) async {
    await tester.pumpWidget(app());
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    await tap(tester, find.byKey(const ValueKey('area-map')));
    final before = sectorMapOf(
      session: await openSave(path: save.path),
      language: 'en',
    );
    if (before.shipField == null) {
      // The nearby ship decodes only with the layouts of the installed ftl.dat.
      // ignore: avoid_print
      print('skip: ${before.shipBlock}');
      return;
    }

    await tap(tester, find.byKey(const ValueKey('map-beacon-2')));
    await tap(tester, find.byKey(const ValueKey('beacon-move-2')));
    await tap(tester, find.byKey(const ValueKey('beacon-close')));
    await tap(tester, find.byKey(const ValueKey('map-beacon-3')));
    await tap(tester, find.byKey(const ValueKey('beacon-store-3')));
    await tap(tester, find.byKey(const ValueKey('store-stock-beacons.3.store.0.0')));
    await tap(tester, find.byKey(const ValueKey('beacon-close')));

    await tap(tester, find.byKey(const ValueKey('save-button')));
    expect(find.byKey(const ValueKey('change-beacons.current')), findsOneWidget);
    expect(find.byKey(const ValueKey('change-beacons.3.store')), findsOneWidget);
    expect(find.byKey(const ValueKey('change-beacons.3.store.0.0')), findsOneWidget);
    await tap(tester, find.byKey(const ValueKey('change-sheet-write')));

    final map = sectorMapOf(
      session: await openSave(path: save.path),
      language: 'en',
    );
    expect(map.beacons[2].current, isTrue);
    expect(map.beacons[8].current, isFalse);
    final store = map.beacons[3].store!;
    expect(store.items, hasLength(9));
    expect(store.items[0].stock.value, 0);
  });

  testWidgets('the ship jumps to sector 3 in the sector tree; Save writes the new sector', (tester) async {
    await tester.pumpWidget(app());
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    await tap(tester, find.byKey(const ValueKey('area-map')));
    final SectorTreeView before;
    try {
      before = sectorTreeOf(
        session: await openSave(path: save.path),
        language: 'en',
      );
    } on EditorError catch (e) {
      // The tree needs the sector lists of the installed ftl.dat.
      // ignore: avoid_print
      print('skip: ${e.code}');
      return;
    }
    if (before.jumpField == null) {
      // ignore: avoid_print
      print('skip: ${before.jumpBlock}');
      return;
    }

    await tap(tester, find.byKey(const ValueKey('map-view-sectors')));
    await tap(tester, find.byKey(const ValueKey('sector-3')));
    await tap(tester, find.byKey(const ValueKey('sector-jump-3')));
    await tap(tester, find.byKey(const ValueKey('save-button')));
    expect(find.byKey(const ValueKey('change-sector.jump')), findsOneWidget);
    await tap(tester, find.byKey(const ValueKey('change-sheet-write')));

    final session = await openSave(path: save.path);
    final tree = sectorTreeOf(session: session, language: 'en');
    expect(tree.sectors[3].current, isTrue);
    expect(tree.sectors[0].visited, isTrue);
    final map = sectorMapOf(session: session, language: 'en');
    expect(map.sector, 3);
    expect(map.fleetOffset, -959);
    expect(map.layoutBlock, isNull);
  });
}
