// PNGs of the floorplan after a staged swap and of the Equipment lists in
// the FTL look, over the fixture the Rust core reads, en and de, at 2x:
//
//   cargo build -p ftl_bridge
//   flutter test test/ship_equipment_shots_test.dart --dart-define=FTL_UI_SHOTS=<folder>
//
// ftl-floor-swap-teleporter and ftl-floor-swap-cloaking: Cloaking and the
// Crew Teleporter installed as staged edits (the Kestrel of the fixture has
// neither), with the square menu of the teleporter room (15) and of the
// cloaking room (8) open.
// ftl-equipment-player and ftl-equipment-enemy: the lists of both ships
// with a row per slot. ftl-equipment-narrow: the player ship at the window
// minimum (1100 x 700), the system rows stacked. ftl-chooser-empty and
// ftl-chooser-filled: the slot chooser of an empty cargo box and of the
// Burst Laser's weapon slot with "la" typed into its filter (the monospace text
// field). Each has a -de twin. Without FTL_UI_SHOTS the test
// is skipped; without the bridge or ftl.dat it prints why and writes
// nothing. It only reads the fixture and ftl.dat.

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

const swap = {'systems.cloaking.installed': 1, 'systems.teleporter.installed': 1};

void main() {
  setUpAll(() async {
    if (shotsDir.isEmpty) return;
    await loadShotFonts();
  });

  const shots = [
    'floor-swap-teleporter',
    'floor-swap-cloaking',
    'equipment-player',
    'equipment-enemy',
    'equipment-narrow',
    'chooser-empty',
    'chooser-filled',
  ];
  for (final locale in const [Locale('en'), Locale('de')]) {
    for (final shot in shots) {
      final name = 'ftl-$shot${locale.languageCode == 'en' ? '' : '-de'}';
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
        final nearby = shot == 'equipment-enemy';
        final floor = shot.startsWith('floor');
        final edits = floor ? Map.of(swap) : <String, int>{};
        final read = await tester.runAsync(() async {
          final session = await openSave(path: fixture);
          final fieldEdits = [for (final e in edits.entries) FieldEdit(path: e.key, value: e.value)];
          return (
            floorplanOf(session: session, nearby: nearby)!,
            floorplanPreview(session: session, nearby: nearby, edits: fieldEdits),
            equipmentOf(session: session, nearby: nearby, language: language),
            equipmentPreview(session: session, nearby: nearby, edits: fieldEdits, language: language),
            crewOf(session: session, nearby: nearby),
            augmentChoices(language: language),
            itemChoices(language: language),
          );
        });
        final (plan, preview, equipment, previewEquipment, crew, augments, items) = read!;
        final refs = <GameImageRef>{
          for (final p in [plan, ?preview]) ...[
            for (final i in [p.hull, p.floor, for (final r in p.rooms) ...r.interior])
              if (i != null) GameImageRef.path(i.innerPath),
          ],
          for (final s in equipment.systems) ...[
            GameImageRef.of(GameImageKind.systemIcon, s.id),
            for (final state in ['green1', 'green2', 'grey1', 'grey2', 'orange1', 'red1']) systemIcon(s.id, state),
          ],
          for (final c in plan.crew) crewSprite(c.lookId, anchored: true),
          for (final c in crew) crewSprite(c.lookId),
          for (final i in [...equipment.weapons, ...equipment.drones, ...?equipment.cargo])
            GameImageRef.of(i.drone ? GameImageKind.drone : GameImageKind.weaponFrame, i.id),
          if (shot.startsWith('chooser'))
            for (final c in items) GameImageRef.of(c.drone ? GameImageKind.drone : GameImageKind.weaponFrame, c.id),
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
        final body = floor
            ? ListView(
                padding: const EdgeInsets.all(24),
                children: [
                  FloorplanCard(
                    plan: plan,
                    crew: crew,
                    equipment: equipment,
                    edits: edits,
                    onChanged: (_, _) {},
                    preview: preview,
                    previewEquipment: previewEquipment,
                  ),
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
          size: shot == 'equipment-narrow' ? const Size(1100, 700) : const Size(1280, 900),
          before: () async {
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 400)));
            await tester.pumpAndSettle();
            if (shot.startsWith('chooser')) {
              final slot = find.byKey(ValueKey(shot == 'chooser-empty' ? 'equip-empty-cargo-2' : 'equip-weapon-1'));
              await tester.ensureVisible(slot);
              await tester.pumpAndSettle();
              await tester.tap(slot);
              await tester.pumpAndSettle();
              if (shot == 'chooser-filled') {
                await tester.enterText(find.byKey(const ValueKey('item-filter')), 'la');
                await tester.pumpAndSettle();
              }
              await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 400)));
              await tester.pumpAndSettle();
            }
            if (!floor) return;
            final room = shot.endsWith('teleporter') ? 15 : 8;
            await tester.tap(find.byKey(ValueKey('floor-square-$room-0')));
            await tester.pumpAndSettle();
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
            await tester.pumpAndSettle();
          },
        );
      });
    }
  }
}
