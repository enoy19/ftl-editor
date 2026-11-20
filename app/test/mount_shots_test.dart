// PNGs of the weapons on their hull mounts in the FTL look, over the
// fixture the Rust core reads, en and de, at 2x:
//
//   cargo build -p ftl_bridge
//   flutter test test/mount_shots_test.dart --dart-define=FTL_UI_SHOTS=<folder>
//
// ftl-mounts-kestrel: the Kestrel with its Artemis and Burst Laser II, the
// pointer on the empty mount 2. ftl-mounts-unpowered: the same with a
// pending edit that powers the Burst Laser off (it slides back under the
// hull). ftl-mounts-enemy: the pirate at the beacon. ftl-mounts-chooser:
// the chooser a click on mount 0 opens. ftl-mounts-weapons-room: the
// popover of Weapon Control with the weapon rows. Each has a -de twin.
// Without FTL_UI_SHOTS the test is skipped; without the bridge or ftl.dat
// it prints why and writes nothing. It only reads the fixture and ftl.dat.

import 'dart:typed_data';

import 'package:flutter/gestures.dart';
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

void main() {
  setUpAll(() async {
    if (shotsDir.isEmpty) return;
    await loadShotFonts();
  });

  for (final locale in const [Locale('en'), Locale('de')]) {
    for (final shot in const ['kestrel', 'unpowered', 'enemy', 'chooser', 'weapons-room']) {
      final name = 'ftl-mounts-$shot${locale.languageCode == 'en' ? '' : '-de'}';
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
        final nearby = shot == 'enemy';
        final edits = shot == 'unpowered' ? {'weapons.LASER_BURST_3.armed': 0} : <String, int>{};
        final read = await tester.runAsync(() async {
          final session = await openSave(path: fixture);
          final fieldEdits = [for (final e in edits.entries) FieldEdit(path: e.key, value: e.value)];
          return (
            floorplanOf(session: session, nearby: nearby)!,
            edits.isEmpty ? null : floorplanPreview(session: session, nearby: nearby, edits: fieldEdits),
            equipmentOf(session: session, nearby: nearby, language: language),
            edits.isEmpty
                ? null
                : equipmentPreview(session: session, nearby: nearby, edits: fieldEdits, language: language),
            crewOf(session: session, nearby: nearby),
            itemChoices(language: language),
          );
        });
        final (plan, preview, equipment, previewEquipment, crew, items) = read!;
        final refs = <GameImageRef>{
          for (final i in [plan.hull, plan.floor, for (final r in plan.rooms) ...r.interior])
            if (i != null) GameImageRef.path(i.innerPath),
          for (final s in equipment.systems) ...[
            GameImageRef.of(GameImageKind.systemIcon, s.id),
            for (final state in ['green1', 'green2', 'grey1', 'orange1', 'red1']) systemIcon(s.id, state),
          ],
          ionLock(),
          for (final c in plan.crew) crewSprite(c.lookId, anchored: true),
          for (final i in [...equipment.weapons, ...equipment.drones, ...?equipment.cargo])
            GameImageRef.of(i.drone ? GameImageKind.drone : GameImageKind.weaponFrame, i.id),
          for (final c in items) GameImageRef.of(c.drone ? GameImageKind.drone : GameImageKind.weaponFrame, c.id),
        };
        final images = <GameImageRef, Uint8List>{};
        await tester.runAsync(() async {
          for (final r in refs) {
            try {
              images[r] = await loadGameImage(r);
            } catch (_) {}
          }
        });
        final body = ListView(
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
              itemChoices: items,
              onAddItem: (_, {required toCargo}) {},
            ),
          ],
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
          size: const Size(1440, 1000),
          before: () async {
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 400)));
            await tester.pumpAndSettle();
            Future<void> settle() async {
              await tester.pumpAndSettle();
              await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
              await tester.pumpAndSettle();
            }

            switch (shot) {
              case 'kestrel':
                final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
                await mouse.addPointer(location: Offset.zero);
                await mouse.moveTo(tester.getCenter(find.byKey(const ValueKey('floor-mount-2'))));
                await settle();
              case 'chooser':
                await tester.tap(find.byKey(const ValueKey('floor-mount-0')));
                await settle();
              case 'weapons-room':
                final weapons = plan.rooms.firstWhere((r) => r.system == 'weapons');
                await tester.tap(find.byKey(ValueKey('floor-room-${weapons.id}')));
                await settle();
            }
          },
        );
      });
    }
  }
}
