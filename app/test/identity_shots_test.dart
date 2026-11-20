// PNG shots of the ship card in the FTL look, with the real ftl.dat and the
// real core (host build of the bridge): the card with a staged name,
// difficulty and type (the unused Kestrel PLAYER_SHIP_EASY, the one type of
// the same layout), the difficulty picker and the type picker with the
// reason of each refused hangar ship. Run with
// `--dart-define=FTL_UI_SHOTS=<dir>`; without the bridge or ftl.dat each shot
// prints why and writes nothing.

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/look/look_mock.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/identity.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/ship_card.dart';
import 'package:material_ui/material_ui.dart';

import 'shots.dart';

const _fixture = '../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav';

void main() {
  setUpAll(() async {
    if (shotsDir.isEmpty) return;
    await loadShotFonts();
  });

  for (final locale in const [Locale('en'), Locale('de')]) {
    final lang = locale.languageCode;
    for (final shot in ['card', 'staged', 'difficulty', 'type']) {
      final name = 'fe-identity-$shot-$lang';
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
        final staged = shot == 'staged';
        final dat = (await tester.runAsync(ftlDatStatus))!.path;
        final data = (await tester.runAsync(() async {
          final session = await openSave(path: _fixture);
          // The edits reach the core as the app sends them: they must apply.
          if (staged) {
            final s = await openSave(path: _fixture);
            s.apply(
              edits: const [FieldEdit(path: 'difficulty', value: 2)],
              texts: const [
                TextEdit(path: 'ship.name', text: 'Red Dwarf'),
                TextEdit(path: 'ship.type', text: 'PLAYER_SHIP_EASY'),
              ],
            );
          }
          return (
            session.summary(),
            identityOf(session: session, nearby: false),
            shipTypeChoices(session: session, language: lang),
          );
        }))!;
        final (summary, identity, choices) = data;
        final refs = {
          GameImageRef.of(GameImageKind.ship, identity.blueprintId),
          GameImageRef.of(GameImageKind.ship, 'PLAYER_SHIP_EASY'),
          for (final c in choices) GameImageRef.of(GameImageKind.ship, c.id),
        };
        final images = <GameImageRef, Uint8List>{};
        await tester.runAsync(() async {
          for (final r in refs) {
            try {
              images[r] = await loadGameImage(r);
            } catch (_) {
              // The fallback icon shows.
            }
          }
        });
        final edits = ShipCardEdits(
          identity: identity,
          texts: staged ? const {'ship.name': 'Red Dwarf', 'ship.type': 'PLAYER_SHIP_EASY'} : const {},
          edits: staged ? const {'difficulty': 2} : const {},
          onText: (_, _) {},
          onEdit: (_, _) {},
          typeChoices: () => choices,
        );
        final app = variantApp(
          Theme(
            data: ftlLookTheme(look),
            child: GameDataScope(
              datPath: dat,
              load: (ref) async => images[ref] ?? (throw StateError('no image $ref')),
              child: Builder(
                builder: (context) => Scaffold(
                  backgroundColor: Theme.of(context).colorScheme.surface,
                  body: ListView(
                    padding: const EdgeInsets.all(Space.xl),
                    children: [ShipCard(summary: summary, edits: edits)],
                  ),
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
          size: const Size(1100, 900),
          before: () async {
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
            await tester.pumpAndSettle();
            final tag = switch (shot) {
              'difficulty' => 'ship-difficulty',
              'type' => 'ship-type',
              _ => null,
            };
            if (tag != null) {
              await tester.tap(find.byKey(ValueKey(tag)));
              await tester.pumpAndSettle();
              await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
              await tester.pumpAndSettle();
            }
          },
        );
      });
    }
  }
}
