// PNG shots of new crew and the crew look in the FTL look, with the real
// ftl.dat and the real core (host build of the bridge): the Crew page with
// the card with the plus, a staged new member (a woman with another hair
// color) and a staged removal, the look picker, and the floorplan that draws
// the new member as she looks on her card. Run with
// `--dart-define=FTL_UI_SHOTS=<dir>`; without the bridge or ftl.dat each shot
// prints why and writes nothing.

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/look/look_mock.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/crew.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/equipment.dart';
import 'package:ftl_editor/src/rust/api/floorplan.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/crew_page.dart';
import 'package:ftl_editor/ui/crew_sprite.dart';
import 'package:ftl_editor/ui/floorplan/floorplan_view.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/game_image.dart';
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
    for (final shot in ['crew-add', 'crew-new', 'crew-look', 'floor-new']) {
      final name = 'fe-crewadd-$shot-$lang';
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
        final staged = shot != 'crew-add';
        // A new human (crew 3) who is a woman with the third hair color, and
        // Berko leaves the ship.
        final edits = staged
            ? {'crew.add': 1, 'crew.3.male': 0, 'crew.3.tints.1': 2, 'crew.0.remove': 1}
            : <String, int>{};
        final dat = (await tester.runAsync(ftlDatStatus))!.path;
        final data = (await tester.runAsync(() async {
          final session = await openSave(path: _fixture);
          final fieldEdits = [for (final e in edits.entries) FieldEdit(path: e.key, value: e.value)];
          final file = crewOf(session: session, nearby: false);
          final preview = staged
              ? crewPreview(session: session, nearby: false, edits: fieldEdits, texts: const [])
              : file;
          final added = staged
              ? crewPreview(
                  session: session,
                  nearby: false,
                  edits: [const FieldEdit(path: 'crew.add', value: 1)],
                  texts: const [],
                ).sublist(file.length)
              : const <CrewMember>[];
          return (
            file,
            preview,
            added,
            crewAddField(session: session, nearby: false),
            raceChoices(language: lang),
            floorplanOf(session: session, nearby: false),
            staged ? floorplanPreview(session: session, nearby: false, edits: fieldEdits) : null,
            equipmentOf(session: session, nearby: false, language: lang),
          );
        }))!;
        final (file, preview, added, add, races, plan, planPreview, equipment) = data;
        final crew = [...file, ...added];
        final looks = {for (final r in races) r.id: lookChoices(race: r.id)};
        final refs = {
          for (final c in [...crew, ...preview]) crewSprite(c.lookId),
          for (final r in races) crewSprite(r.lookId),
          for (final l in looks.values.expand((l) => l)) crewSprite(l.lookId),
          for (final c in [...plan!.crew, ...?planPreview?.crew]) crewSprite(c.lookId, anchored: true),
          for (final s in ['pilot', 'engines', 'shields', 'weapons', 'repair', 'combat'])
            GameImageRef.path('img/people/skill_${s}_white.png'),
          for (final p in [...plan.rooms.expand((r) => r.interior), ?plan.hull, ?plan.floor])
            GameImageRef.path(p.innerPath),
          for (final r in plan.rooms)
            if (r.system case final s?) GameImageRef.of(GameImageKind.systemIcon, s),
          for (final s in equipment.systems) GameImageRef.of(GameImageKind.systemIcon, s.id),
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
        final texts = CrewTexts(
          races: races,
          onChanged: (_, _) {},
          previews: staged ? {for (final c in preview) c.index: c} : const {},
          looksOf: (race) => looks[race] ?? const [],
          onLook: (_, _) {},
          newFrom: added.firstOrNull?.index,
          onRemoveNew: (_) {},
        );
        final Widget body = shot == 'floor-new'
            ? ListView(
                padding: const EdgeInsets.all(Space.xl),
                children: [
                  FloorplanCard(
                    plan: plan,
                    crew: crew,
                    equipment: equipment,
                    edits: edits,
                    onChanged: (_, _) {},
                    preview: planPreview,
                    texts: texts,
                  ),
                ],
              )
            : CrewPage(crew: crew, edits: edits, onChanged: (_, _) {}, texts: texts, add: add);
        final app = variantApp(
          Theme(
            data: ftlLookTheme(look),
            child: GameDataScope(
              datPath: dat,
              load: (ref) async => images[ref] ?? (throw StateError('no image $ref')),
              child: Builder(
                builder: (context) => Scaffold(backgroundColor: Theme.of(context).colorScheme.surface, body: body),
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
          size: const Size(1280, 900),
          before: () async {
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
            await tester.pumpAndSettle();
            if (shot == 'crew-look') {
              await tester.tap(find.byKey(const ValueKey('look-3')));
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
