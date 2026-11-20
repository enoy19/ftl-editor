// PNGs of the sector tree of the Map page in the FTL look over the tree the
// Rust core builds from the fixture and the real ftl.dat, en and de, at 2x:
//
//   cargo build -p ftl_bridge
//   flutter test test/sector_shots_test.dart --dart-define=FTL_UI_SHOTS=<folder>
//
// ftl-sectors (the fixture's tree, the ship in sector 1),
// ftl-sectors-popover (the popover of sector 3 with Jump here),
// ftl-sectors-jump (a staged jump to sector index 9: the ship marker there,
// the visited route) and ftl-sectors-jump-map (the beacon map of the new
// sector, read only), each with a -de twin. Without FTL_UI_SHOTS the test
// is skipped; without the bridge or ftl.dat it prints why and writes
// nothing. It only reads the fixture and ftl.dat.

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/look/look_mock.dart';
import 'package:ftl_editor/shell/status_bar.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/map.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/map_page.dart';
import 'package:material_ui/material_ui.dart';

import 'shell_fixture.dart';
import 'shots.dart';

const fixture = '../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav';

void main() {
  setUpAll(() async {
    if (shotsDir.isEmpty) return;
    await loadShotFonts();
  });

  const jump = {'sector.jump': 9};
  final pages = <(String, Map<String, int>)>[
    ('', const {}),
    ('-popover', const {}),
    ('-jump', jump),
    ('-jump-map', jump),
  ];
  for (final locale in const [Locale('en'), Locale('de')]) {
    for (final (page, edits) in pages) {
      final name = 'ftl-sectors$page${locale.languageCode == 'en' ? '' : '-de'}';
      testWidgets('shot $name', skip: shotsDir.isEmpty, (tester) async {
        if (!await loadRust(tester, 'skip $name')) return;
        final look = await tester.runAsync(loadLookFromLocatedDat);
        if (look == null) {
          // ignore: avoid_print
          print('skip $name: no ftl.dat found');
          return;
        }
        imageCache
          ..clear()
          ..clearLiveImages();
        final dat = (await tester.runAsync(ftlDatStatus))!.path;
        final language = locale.languageCode;
        final fieldEdits = [for (final e in edits.entries) FieldEdit(path: e.key, value: e.value)];
        final (map, preview, tree, treePreview) = (await tester.runAsync(() async {
          final session = await openSave(path: fixture);
          return (
            sectorMapOf(session: session, language: language),
            edits.isEmpty
                ? null
                : sectorMapPreview(session: session, edits: fieldEdits, texts: const [], language: language),
            sectorTreeOf(session: session, language: language),
            edits.isEmpty
                ? null
                : sectorTreePreview(session: session, edits: fieldEdits, texts: const [], language: language),
          );
        }))!;
        final images = <GameImageRef, Uint8List>{};
        await tester.runAsync(() async {
          for (final r in MapImages.all) {
            try {
              images[r] = await loadGameImage(r);
            } catch (_) {}
          }
        });
        final app = variantApp(
          Theme(
            data: ftlLookTheme(look),
            child: GameDataScope(
              datPath: dat,
              load: (ref) async => images[ref] ?? (throw StateError('no image $ref')),
              child: mockShell(
                edits: Map.of(edits),
                onSave: () {},
                map: map,
                mapPreview: preview,
                tree: tree,
                treePreview: treePreview,
                status: ShellStatus(
                  path: '/Users/player/Library/Application Support/FasterThanLight/continue.sav',
                  gameData: dat,
                ),
              ),
            ),
          ),
          locale: locale,
        );
        await shoot(
          tester,
          name,
          app,
          pixelRatio: 2,
          size: const Size(1280, 860),
          before: () async {
            await tester.tap(find.byKey(const ValueKey('area-map')));
            await tester.pumpAndSettle();
            if (page != '-jump-map') {
              await tester.tap(find.byKey(const ValueKey('map-view-sectors')));
              await tester.pumpAndSettle();
            }
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 400)));
            await tester.pumpAndSettle();
            if (page == '-popover') {
              await tester.tap(find.byKey(const ValueKey('sector-3')), warnIfMissed: false);
              await tester.pumpAndSettle();
            }
          },
        );
      });
    }
  }
}
