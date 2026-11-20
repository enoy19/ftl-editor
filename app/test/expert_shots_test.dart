// PNGs of the Expert area in the FTL look over the raw view of the fixture
// as the Rust core reads it, en and de, at 2x:
//
//   cargo build -p ftl_bridge
//   flutter test test/expert_shots_test.dart --dart-define=FTL_UI_SHOTS=<folder>
//
// ftl-expert-search (the tree searched for "resources", scrap and fuel
// staged), ftl-expert-state (the state vars, used_missile staged),
// ftl-expert-unknown (the unknown fields of crew member 0, none since
// the codec names them, a raw field staged, and the undecoded bytes) and
// ftl-expert-changes (the change sheet that names the raw paths), each with a -de twin. Without FTL_UI_SHOTS the test is
// skipped; without the bridge or ftl.dat it prints why and writes nothing.
// It only reads the fixture and ftl.dat.

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/look/look_mock.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/shell/destination.dart';
import 'package:ftl_editor/shell/status_bar.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/expert.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/expert_page.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:material_ui/material_ui.dart';

import 'shell_fixture.dart';
import 'shots.dart';

const fixture = '../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav';

void main() {
  setUpAll(() async {
    if (shotsDir.isEmpty) return;
    await loadShotFonts();
  });

  final pages = <(String, Destination, String, Map<String, int>)>[
    (
      'search',
      Destination.dump,
      'resources',
      const {'raw.player_ship.resources.scrap': 9999, 'raw.player_ship.resources.fuel': 30},
    ),
    ('state', Destination.stateVars, '', const {'state.used_missile': 3}),
    ('unknown', Destination.unknownFields, 'player_ship.crew.0', const {'raw.player_ship.crew.0.clone_dying': 5}),
    ('changes', Destination.dump, '', const {'raw.player_ship.resources.scrap': 9999, 'state.used_missile': 3}),
  ];
  for (final locale in const [Locale('en'), Locale('de')]) {
    for (final (page, item, query, edits) in pages) {
      final name = 'ftl-expert-$page${locale.languageCode == 'en' ? '' : '-de'}';
      testWidgets('shot $name', skip: shotsDir.isEmpty, (tester) async {
        if (!await loadRust(tester, 'skip $name')) return;
        final look = await tester.runAsync(loadLookFromLocatedDat);
        if (look == null) {
          // ignore: avoid_print
          print('skip $name: no ftl.dat found');
          return;
        }
        final dat = (await tester.runAsync(ftlDatStatus))!.path;
        final dump = (await tester.runAsync(() async => expertOf(session: await openSave(path: fixture))))!;
        final app = variantApp(
          Theme(
            data: ftlLookTheme(look),
            child: GameDataScope(
              datPath: dat,
              load: (ref) async => throw StateError('no image $ref'),
              child: mockShell(
                edits: edits,
                onSave: () {},
                expertDump: dump,
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
          before: () async {
            await tester.tap(find.byKey(const ValueKey('expert-menu')));
            await tester.pumpAndSettle();
            await tester.tap(find.byWidgetPredicate((w) => w is PopupMenuItem<Destination> && w.value == item));
            await tester.pumpAndSettle();
            if (query.isNotEmpty) {
              await tester.enterText(find.byKey(const ValueKey('expert-search')), query);
              await tester.pumpAndSettle();
            }
            if (page == 'changes') {
              final context = tester.element(find.byType(ExpertPage));
              final l = AppLocalizations.of(context)!;
              showChangeSheet(
                context,
                changes: fieldChangeRows(
                  pendingChanges([for (final e in dump.entries) ?e.field], edits),
                  l,
                  look: (path, l) => expertLook(path, l)!,
                ),
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
