// PNGs of the Map page in the FTL look over a sector map the Rust core
// reads, en and de, at 2x:
//
//   cargo build -p ftl_bridge
//   flutter test test/map_shots_test.dart --dart-define=FTL_UI_SHOTS=<folder>
//
// Every state comes from the fixture through edit paths: ftl-map (the
// file at 1100 x 700, the smallest window), ftl-map-wide (1920 x 1080, the
// fleet three jumps forward), ftl-map-moved (the ship moved to beacon 3),
// ftl-map-store (beacon 19 made a store, an item sold out, one swapped, its
// popover open) and ftl-map-changes (the change sheet of all of it), each
// with a -de twin. Without FTL_UI_SHOTS the test is skipped; without the
// bridge or ftl.dat it prints why and writes nothing. It only reads the
// fixture and ftl.dat.

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/look/look_mock.dart';
import 'package:ftl_editor/shell/status_bar.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/equipment.dart';
import 'package:ftl_editor/src/rust/api/map.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/map_page.dart';
import 'package:ftl_editor/ui/resource_bar.dart';
import 'package:material_ui/material_ui.dart';

import 'shell_fixture.dart';
import 'shots.dart';

const fixture = '../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav';

void main() {
  setUpAll(() async {
    if (shotsDir.isEmpty) return;
    await loadShotFonts();
  });

  const store = {'beacons.18.store': 1, 'beacons.18.store.0.1': 0, 'beacons.18.store.fuel': 10};
  const swap = {'beacons.18.store.0.2.item': 'LASER_BURST_3'};
  const small = Size(1100, 700);
  final pages = <(String, Size, Map<String, int>, Map<String, String>)>[
    ('', small, const {}, const {}),
    ('-wide', const Size(1920, 1080), const {'fleet.offset': -831 + 3 * fleetJump}, const {}),
    ('-moved', small, const {'beacons.current': 2}, const {}),
    ('-store', const Size(1280, 860), store, swap),
    ('-changes', const Size(1280, 860), const {'beacons.current': 2, ...store}, swap),
  ];
  for (final locale in const [Locale('en'), Locale('de')]) {
    for (final (page, size, edits, texts) in pages) {
      final name = 'ftl-map$page${locale.languageCode == 'en' ? '' : '-de'}';
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
        final (map, preview, base, choices) = (await tester.runAsync(() async {
          final session = await openSave(path: fixture);
          return (
            sectorMapOf(session: session, language: language),
            sectorMapPreview(
              session: session,
              edits: [for (final e in edits.entries) FieldEdit(path: e.key, value: e.value)],
              texts: [for (final e in texts.entries) TextEdit(path: e.key, text: e.value)],
              language: language,
            ),
            sectorMapPreview(
              session: session,
              edits: [if (edits.containsKey('beacons.18.store')) const FieldEdit(path: 'beacons.18.store', value: 1)],
              texts: const [],
              language: language,
            ),
            itemChoices(language: language),
          );
        }))!;
        final refs = {
          ...MapImages.all,
          for (final b in preview.beacons)
            for (final i in b.store?.items ?? const <StoreItemView>[]) ?storeItemImage(i),
          for (final s in ['fuel', 'missiles', 'drone_parts']) ?resourceImage('ship.$s'),
        };
        final images = <GameImageRef, Uint8List>{};
        await tester.runAsync(() async {
          for (final r in refs) {
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
          size: size,
          before: () async {
            await tester.tap(find.byKey(const ValueKey('area-map')));
            await tester.pumpAndSettle();
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 400)));
            await tester.pumpAndSettle();
            if (page == '-store') {
              await tester.tap(find.byKey(const ValueKey('map-beacon-18')), warnIfMissed: false);
              await tester.pumpAndSettle();
              await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
              await tester.pumpAndSettle();
            }
            if (page == '-changes') {
              final context = tester.element(find.byType(MapPage));
              final l = AppLocalizations.of(context)!;
              showChangeSheet(
                context,
                changes: [
                  ...fieldChangeRows(
                    pendingChanges([
                      ...mapFields(map),
                      for (final f in mapFields(base))
                        if (!mapFields(map).any((g) => g.path == f.path)) f,
                    ], edits),
                    l,
                    look: (path, l) => mapLook(path, l, map)!,
                  ),
                  ...storeItemChangeRows(
                    base,
                    texts,
                    (kind) => [
                      for (final c in choices)
                        if (c.drone == (kind == 'drone')) (id: c.id, name: c.name),
                    ],
                    l,
                  ),
                ],
                onUndo: (_) {},
              );
              await tester.pumpAndSettle();
            }
          },
        );
      });
    }
  }
}
