// PNGs of the shell polish of 2026-11-19 in the FTL look over the fixture
// and the installed ftl.dat, en and de, at 2x:
//
//   cargo build -p ftl_bridge
//   flutter test test/polish_shots_test.dart --dart-define=FTL_UI_SHOTS=<folder>
//
// ftl-ship-chips (the Ship page: file and ship card, the ship chips, the
// resources; the top bar with Undo and Redo left of the quick actions),
// ftl-exact (the exact points of a skill after a right click on its tile)
// and ftl-changes (the change sheet with the game icons of resources,
// systems, skills, crew, weapons, drones and a fire), each with a -de twin.
// Without FTL_UI_SHOTS the test is skipped; without the bridge or ftl.dat it
// prints why and writes nothing. It only reads the fixture and ftl.dat.

import 'dart:typed_data';

import 'package:flutter/gestures.dart' show kSecondaryButton;
import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/look/look_mock.dart';
import 'package:ftl_editor/shell/status_bar.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/crew.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/equipment.dart';
import 'package:ftl_editor/src/rust/api/floorplan.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/crew_card.dart' show skillIcon;
import 'package:ftl_editor/ui/crew_sprite.dart';
import 'package:ftl_editor/ui/crew_page.dart';
import 'package:ftl_editor/ui/equipment_page.dart';
import 'package:ftl_editor/ui/field_image.dart';
import 'package:ftl_editor/ui/floorplan/floorplan_view.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/quick_actions.dart';
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

  for (final locale in const [Locale('en'), Locale('de')]) {
    for (final page in const ['ship-chips', 'exact', 'changes']) {
      final name = 'ftl-$page${locale.languageCode == 'en' ? '' : '-de'}';
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
        final (summary, crew, equipment, plan, choices) = (await tester.runAsync(() async {
          final session = await openSave(path: fixture);
          return (
            session.summary(),
            crewOf(session: session, nearby: false),
            equipmentOf(session: session, nearby: false, language: language),
            floorplanOf(session: session, nearby: false),
            itemChoices(language: language),
          );
        }))!;

        // One edit of each kind the change sheet draws with a game icon.
        final system = equipment.systems.firstWhere((s) => s.level != null).level!;
        final weapon = equipment.itemFields.firstWhere((f) => !f.drone && f.slots != null).slots!;
        final drone = equipment.itemFields.where((f) => f.drone && f.slots != null).firstOrNull?.slots;
        final fire = floorplanFields(plan).firstWhere((f) => f.path.startsWith('fires.'));
        final edits = <String, int>{
          'ship.scrap': 999,
          'ship.fuel': 20,
          system.path: system.max,
          crew.first.skills[1].points.path: crew.first.skills[1].points.max,
          crew.first.health.path: 1,
          weapon.path: weapon.value == 0 ? 1 : 0,
          if (drone != null) drone.path: drone.value == 0 ? 1 : 0,
          fire.path: 100,
        };
        final fields = [
          ...summary.fields,
          ...crewFields(crew),
          ...equipmentFields(equipment),
          ...floorplanFields(plan),
        ];
        FieldImage? image(String path) => fieldImage(path, crew: crew, equipment: equipment, choices: choices);

        final refs = <GameImageRef>{
          GameImageRef.of(GameImageKind.ship, summary.shipBlueprintId),
          for (final f in fields) ?image(f.path)?.ref,
          for (final c in crew) crewSprite(c.lookId),
          for (final s in crew.first.skills) skillIcon(s.skill),
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
                summary: summary,
                crew: crew,
                equipment: equipment,
                nearby: false,
                edits: page == 'ship-chips' ? const {'ship.scrap': 999} : const {},
                onSave: () {},
                onUndo: () {},
                actions: QuickActionBar(onRun: (_) {}),
                status: ShellStatus(
                  path: '/Users/player/Library/Application Support/FasterThanLight/continue.sav',
                  gameData: dat,
                ),
              ),
            ),
          ),
          locale: locale,
        );
        Future<void> settle() async {
          await tester.pumpAndSettle();
          await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
          await tester.pumpAndSettle();
        }

        await shoot(
          tester,
          name,
          app,
          pixelRatio: 2,
          size: const Size(1280, 860),
          before: () async {
            await settle();
            if (page == 'exact') {
              await tester.tap(find.byKey(const ValueKey('area-crew')));
              await settle();
              final tile = find.byKey(ValueKey('skill-${crew.first.skills[1].points.path}'));
              await tester.tap(tile, buttons: kSecondaryButton);
              await settle();
            }
            if (page == 'changes') {
              final context = tester.element(find.byType(ResourceCard));
              final l = AppLocalizations.of(context)!;
              final races = {crew.first.racePath: crew.first.race == 'human' ? 'engi' : 'human'};
              showChangeSheet(
                context,
                changes: [
                  ...fieldChangeRows(
                    pendingChanges(fields, edits),
                    l,
                    look: (path, l) =>
                        itemLook(path, l, equipment, choices) ??
                        floorplanLook(path, l, plan, equipment) ??
                        fieldLook(path, l, crew: crew, equipment: equipment),
                    image: image,
                  ),
                  ...crewTextChangeRows(crew, CrewTexts(values: races), l),
                ],
                onUndo: (_) {},
              );
              await settle();
            }
          },
        );
      });
    }
  }
}
