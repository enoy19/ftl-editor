// PNGs of the Zoltan shield of the shields row in the FTL look, over the
// fixture the Rust core reads, en and de, at 2x:
//
//   cargo build -p ftl_bridge
//   flutter test test/zoltan_shots_test.dart --dart-define=FTL_UI_SHOTS=<folder>
//
// The file gets 3 Zoltan points through the edit path, saved to a copy of
// the fixture in a temp folder; a pending edit raises them to 5, so the row
// shows FTL's green bars and the two yellow ones of the edit.
// ftl-zoltan-equipment: the Equipment page. ftl-zoltan-popover: the
// floorplan popover of the shields room. Each has a -de twin. Without
// FTL_UI_SHOTS the test is skipped; without the bridge or ftl.dat it prints
// why and writes nothing. It reads ftl.dat and writes only its temp folder.

import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/look/look_mock.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/crew.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/equipment.dart';
import 'package:ftl_editor/src/rust/api/floorplan.dart';
import 'package:ftl_editor/ui/crew_sprite.dart';
import 'package:ftl_editor/ui/equipment_page.dart';
import 'package:ftl_editor/ui/floorplan/floorplan_view.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:material_ui/material_ui.dart';

import 'shots.dart';

const fixture = '../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav';

const inFile = [FieldEdit(path: 'systems.shields.energy', value: 3)];

const pending = {'systems.shields.energy': 5};

void main() {
  setUpAll(() async {
    if (shotsDir.isEmpty) return;
    await loadShotFonts();
  });

  for (final locale in const [Locale('en'), Locale('de')]) {
    for (final shot in const ['equipment', 'popover']) {
      final name = 'ftl-zoltan-$shot${locale.languageCode == 'en' ? '' : '-de'}';
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
        final edits = Map.of(pending);
        final read = await tester.runAsync(() async {
          final dir = await Directory.systemTemp.createTemp('fe-zoltan-shots');
          final copy = '${dir.path}/continue.sav';
          await File(fixture).copy(copy);
          final damaged = await openSave(path: copy);
          damaged.apply(edits: inFile, texts: const []);
          await damaged.save(backupDir: '${dir.path}/backups');
          final session = await openSave(path: copy);
          final fieldEdits = [for (final e in edits.entries) FieldEdit(path: e.key, value: e.value)];
          final result = (
            floorplanOf(session: session, nearby: false)!,
            equipmentOf(session: session, nearby: false, language: language),
            equipmentPreview(session: session, nearby: false, edits: fieldEdits, language: language),
            crewOf(session: session, nearby: false),
            augmentChoices(language: language),
            itemChoices(language: language),
          );
          await dir.delete(recursive: true);
          return result;
        });
        final (plan, equipment, previewEquipment, crew, augments, items) = read!;
        final refs = <GameImageRef>{
          for (final i in [plan.hull, plan.floor, for (final r in plan.rooms) ...r.interior])
            if (i != null) GameImageRef.path(i.innerPath),
          for (final s in equipment.systems) ...[
            GameImageRef.of(GameImageKind.systemIcon, s.id),
            for (final state in ['green1', 'green2', 'grey1', 'grey2', 'orange1', 'orange2', 'red1', 'red2'])
              systemIcon(s.id, state),
          ],
          ionLock(),
          ionLock(white: true),
          for (final c in plan.crew) crewSprite(c.lookId, anchored: true),
          for (final c in crew) crewSprite(c.lookId),
          for (final i in [...equipment.weapons, ...equipment.drones, ...?equipment.cargo])
            GameImageRef.of(i.drone ? GameImageKind.drone : GameImageKind.weaponFrame, i.id),
          fireSheet,
          breachSheet,
        };
        final images = <GameImageRef, Uint8List>{};
        await tester.runAsync(() async {
          for (final r in refs) {
            try {
              images[r] = await loadGameImage(r);
            } catch (_) {}
          }
        });
        final popover = shot == 'popover';
        final body = popover
            ? ListView(
                padding: const EdgeInsets.all(24),
                children: [
                  FloorplanCard(plan: plan, crew: crew, equipment: equipment, edits: edits, onChanged: (_, _) {}),
                ],
              )
            : EquipmentPage(
                equipment: equipment,
                preview: previewEquipment,
                edits: edits,
                onChanged: (_, _) {},
                addedAugments: const [],
                choices: augments,
                onAddAugment: (_) {},
                itemChoices: items,
                onAddItem: (_, {required toCargo}) {},
              );
        final app = variantApp(
          Theme(
            data: ftlLookTheme(look),
            child: GameDataScope(
              datPath: dat,
              load: (ref) async => images[ref] ?? (throw StateError('no image $ref')),
              child: Builder(builder: (context) => Scaffold(body: body)),
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
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 400)));
            await tester.pumpAndSettle();
            if (!popover) return;
            final shields = plan.rooms.firstWhere((r) => r.system == 'shields');
            await tester.tap(find.byKey(ValueKey('floor-room-${shields.id}')));
            await tester.pumpAndSettle();
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
            await tester.pumpAndSettle();
          },
        );
      });
    }
  }
}
