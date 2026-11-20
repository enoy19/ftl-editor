// Renders PNGs of the app shell (Ship and Crew page) over mock data, light
// and dark, and of the FTL look, without a person:
//
//   flutter test test/gallery_shots_test.dart --dart-define=FTL_UI_SHOTS=<folder>
//
// Without FTL_UI_SHOTS the test is skipped, so `flutter test test` writes
// nothing. The PNGs use Roboto and the Material icons from the Flutter SDK
// (FLUTTER_ROOT); the test font would draw every glyph as a box.
//
// The FTL look (look-ftl and look-ftl-de, the mock; ftl-ship, ftl-crew,
// ftl-equipment (with staged weapon, drone and cargo edits), ftl-augment (the augment picker), ftl-item (the Add to cargo picker), ftl-profile (with staged edits), ftl-profile-lower (stats and scores),
// ftl-profile-changes (the change sheet of the staged profile), ftl-profile-achievement (its editor), ftl-profile-add (the picker), ftl-changes (the change sheet), ftl-palette (Cmd+K),
// ftl-actions (the sheet of Max everything on the fixture), ftl-enemy-ship, ftl-enemy-equipment,
// ftl-enemy-changes (the enemy ship of the fixture with staged edits) and ftl-status-file-hover,
// ftl-status-dat-hover (the pointer over a status bar item) with -de twins, the real shell in the look; dark only, at 2x so
// one font pixel is two image pixels) and the crew sprites of the Crew page
// read the local ftl.dat through the Rust core. `flutter test` does not build it, so build it
// first with `cargo build -p ftl_bridge` (the dylib in ../target/debug;
// FTL_BRIDGE_LIB overrides the path). Without the library or without ftl.dat
// the look shot prints why and writes nothing, and the Crew page shows the
// fallback icons.

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_look.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/look/look_mock.dart';
import 'package:ftl_editor/shell/status_bar.dart';
import 'package:ftl_editor/src/rust/api/actions.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/crew.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/equipment.dart';
import 'package:ftl_editor/src/rust/api/floorplan.dart';
import 'package:ftl_editor/src/rust/api/profile.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/ui/crew_sprite.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/command_palette.dart';
import 'package:ftl_editor/ui/crew_card.dart';
import 'package:ftl_editor/ui/crew_page.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/equipment_page.dart';
import 'package:ftl_editor/ui/floorplan/floorplan_view.dart';
import 'package:ftl_editor/ui/profile_page.dart';
import 'package:ftl_editor/ui/ship_choice.dart';
import 'package:ftl_editor/ui/ship_page.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/quick_actions.dart';
import 'package:ftl_editor/ui/resource_bar.dart';
import 'package:material_ui/material_ui.dart';

import 'mock_crew.dart';
import 'mock_equipment.dart';
import 'mock_profile.dart';
import 'mock_summary.dart';
import 'shell_fixture.dart';
import 'shots.dart';

void main() {
  setUpAll(() async {
    if (shotsDir.isEmpty) return;
    await loadShotFonts();
  });

  for (final b in Brightness.values) {
    final suffix = b == Brightness.light ? '' : '-dark';
    testWidgets('shot crew$suffix', skip: shotsDir.isEmpty, (tester) async {
      // The real sprites when the bridge and ftl.dat exist, else the fallback icons.
      String? dat;
      final sprites = <GameImageRef, Uint8List>{};
      if (await loadRust(tester, 'crew sprites')) {
        await tester.runAsync(() async {
          dat = (await ftlDatStatus()).path;
          if (dat == null) return;
          await openGameData(path: dat!);
          for (final r in crewCardImages(fullCrew)) {
            sprites[r] = await loadGameImage(r);
          }
        });
      }
      // ignore: avoid_print
      if (dat == null) print('crew sprites: no ftl.dat found, fallback icons');
      final app = variantApp(
        GameDataScope(
          datPath: dat,
          load: (ref) async => sprites[ref]!,
          child: mockShell(
            edits: {'crew.2.skills.shield': 110, 'crew.6.health': 70},
            onSave: () {},
            status: ShellStatus(
              path: '/Users/player/Library/Application Support/FasterThanLight/continue.sav',
              gameData: dat,
            ),
          ),
        ),
        brightness: b,
      );
      await shoot(
        tester,
        'crew$suffix',
        app,
        before: () async {
          await tester.tap(find.text('Crew'));
          await tester.pumpAndSettle();
          // Image decoding runs outside the fake clock of the test.
          await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
          await tester.pumpAndSettle();
        },
      );
    });
    testWidgets('shot shell$suffix', skip: shotsDir.isEmpty, (tester) async {
      await shoot(
        tester,
        'shell$suffix',
        variantApp(
          mockShell(edits: {'ship.scrap': 500}, onSave: () {}),
          brightness: b,
        ),
      );
    });
    testWidgets('shot shell-profile$suffix', skip: shotsDir.isEmpty, (tester) async {
      await shoot(
        tester,
        'shell-profile$suffix',
        variantApp(mockShell(), brightness: b),
        before: () async {
          await tester.tap(find.text('Profile'));
          await tester.pumpAndSettle();
        },
      );
    });
    testWidgets('shot shell-profile-lower$suffix', skip: shotsDir.isEmpty, (tester) async {
      await shoot(
        tester,
        'shell-profile-lower$suffix',
        variantApp(mockShell(), brightness: b),
        before: () async {
          await tester.tap(find.text('Profile'));
          await tester.pumpAndSettle();
          await tester.drag(find.byType(Scrollable).first, const Offset(0, -2000));
          await tester.pumpAndSettle();
        },
      );
    });
    testWidgets('shot shell-popover$suffix', skip: shotsDir.isEmpty, (tester) async {
      await shoot(
        tester,
        'shell-popover$suffix',
        variantApp(mockShell(), brightness: b),
        before: () async {
          await tester.tap(find.byKey(const ValueKey('resource-ship.fuel')));
          await tester.pumpAndSettle();
        },
      );
    });
  }

  // The real shell in the FTL look, with every game image of the mock data.
  for (final locale in const [Locale('en'), Locale('de')]) {
    final de = locale.languageCode == 'en' ? '' : '-de';
    for (final (page, tab) in const [
      ('ship', null),
      ('crew', 'area-crew'),
      ('crew-race', 'area-crew'),
      ('equipment', 'area-equipment'),
      ('install', 'area-equipment'),
      ('augment', 'area-equipment'),
      ('item', 'area-equipment'),
      ('profile', 'area-profile'),
      ('profile-lower', 'area-profile'),
      ('profile-changes', 'area-profile'),
      ('profile-achievement', 'area-profile'),
      ('profile-add', 'area-profile'),
      ('changes', null),
      ('palette', null),
      ('actions', null),
      ('status-file-hover', null),
      ('status-dat-hover', null),
    ]) {
      final name = 'ftl-$page$de';
      testWidgets('shot $name', skip: shotsDir.isEmpty, (tester) async {
        if (!await loadRust(tester, 'skip $name')) return;
        final look = await tester.runAsync(loadLookFromLocatedDat);
        if (look == null) {
          // ignore: avoid_print
          print('skip $name: no ftl.dat found');
          return;
        }
        final refs = {
          GameImageRef.of(GameImageKind.ship, mockSummary.shipBlueprintId),
          ...crewCardImages(fullCrew),
          for (final s in mockProfile.ships) GameImageRef.of(GameImageKind.ship, s.id),
          for (final a in [...mockProfile.achievements, ...mockStaged.achievements])
            GameImageRef.of(GameImageKind.achievement, a.id),
          for (final s in mockProfile.topScores) GameImageRef.of(GameImageKind.ship, s.shipId),
          for (final s in mockEquipment.systems)
            for (final state in ['green1', 'green2', 'grey1', 'grey2', 'orange1', 'red1']) systemIcon(s.id, state),
          for (final w in mockEquipment.weapons) GameImageRef.of(GameImageKind.weaponFrame, w.id),
          for (final d in mockEquipment.drones) GameImageRef.of(GameImageKind.drone, d.id),
          for (final f in mockSummary.fields) ?resourceImage(f.path),
        };
        // The Burst Laser to the cargo, a Mini Beam into its slot, the drone
        // control with a Combat drone (the Kestrel of the fixture has none)
        // and a Shield drone into the cargo.
        const itemEdits = {
          'weapons.LASER_BURST_3': 0,
          'cargo.LASER_BURST_3': 1,
          'weapons.BEAM_1': 1,
          'systems.drones.installed': 1,
          'drones.COMBAT_1': 1,
          'cargo.DRONE_SHIELD_PLAYER': 1,
        };
        // Mind control and the teleporter in, the clone bay instead of the
        // medbay.
        const installEdits = {
          'systems.mind.installed': 1,
          'systems.teleporter.installed': 1,
          'systems.clonebay.installed': 1,
        };
        // Engines raised (yellow), shields and the reactor lowered (red).
        const edits = {
          'ship.scrap': 500,
          'crew.2.skills.shield': 110,
          'crew.6.health': 70,
          'systems.engines.level': 6,
          'systems.shields.level': 1,
          'reactor': 6,
        };
        // The image cache outlives a test, and an image of an earlier test can
        // come back disposed: each shot loads its own.
        imageCache
          ..clear()
          ..clearLiveImages();
        final images = <GameImageRef, Uint8List>{};
        final dat = (await tester.runAsync(ftlDatStatus))!.path;
        // The Equipment page shows the fixture as the core reads it, names in
        // the language of the shot; it only reads the file.
        final equipment = await tester.runAsync(() async {
          final session = await openSave(path: '../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav');
          return (
            equipmentOf(session: session, nearby: false, language: locale.languageCode),
            augmentChoices(language: locale.languageCode),
            planAction(session: session, action: 'max-everything'),
            session.summary().fields,
            crewOf(session: session, nearby: false),
            achievementCatalog(language: locale.languageCode),
            equipmentPreview(
              session: session,
              nearby: false,
              edits: [for (final e in itemEdits.entries) FieldEdit(path: e.key, value: e.value)],
              language: locale.languageCode,
            ),
            itemChoices(language: locale.languageCode),
            equipmentPreview(
              session: session,
              nearby: false,
              edits: [for (final e in installEdits.entries) FieldEdit(path: e.key, value: e.value)],
              language: locale.languageCode,
            ),
          );
        });
        // The weapons, drones and cargo of the preview and every choice of the picker.
        refs.addAll([
          for (final i in [...?equipment?.$7.weapons, ...?equipment?.$7.drones, ...?equipment?.$7.cargo])
            GameImageRef.of(i.drone ? GameImageKind.drone : GameImageKind.weaponFrame, i.id),
          for (final c in equipment?.$8 ?? const <ItemChoice>[])
            GameImageRef.of(c.drone ? GameImageKind.drone : GameImageKind.weaponFrame, c.id),
        ]);
        // The picker of Add achievement shows every achievement of the game.
        refs.addAll([
          for (final a in equipment?.$6 ?? const <AchievementInfo>[]) GameImageRef.of(GameImageKind.achievement, a.id),
        ]);
        await tester.runAsync(() async {
          for (final r in refs) {
            try {
              images[r] = await loadGameImage(r);
            } catch (_) {
              // The image of this id does not exist: the fallback icon shows.
            }
          }
        });
        final app = variantApp(
          Theme(
            data: ftlLookTheme(look),
            child: GameDataScope(
              datPath: dat,
              load: (ref) async => images[ref] ?? (throw StateError('no image $ref')),
              child: mockShell(
                edits: page == 'install'
                    ? installEdits
                    : page == 'equipment' || page == 'item'
                    ? {...edits, ...itemEdits}
                    : edits,
                onSave: () {},
                equipmentHovered: 'shields',
                equipment: equipment?.$1,
                equipmentPreview: page == 'install'
                    ? equipment?.$9
                    : page == 'equipment' || page == 'item'
                    ? equipment?.$7
                    : null,
                augmentChoices: equipment?.$2,
                itemChoices: equipment?.$8 ?? const [],
                profileStaged: true,
                // A staged name (Juno to June) and race (Hollis to Human).
                crewTexts: const CrewTexts(
                  values: {'crew.1.name': 'June', 'crew.0.race': 'human'},
                  races: mockRaces,
                  onChanged: _ignoreText,
                ),
                onSaveProfile: () {},
                onChooseFile: () {},
                onChooseGameData: () {},
                achievementCatalog: equipment?.$6 ?? mockCatalog,
                actions: QuickActionBar(onRun: (_) {}),
                palette: PaletteSource(
                  entries: (l) => [
                    ...paletteFields(
                      [...mockSummary.fields, ...crewFields(fullCrew), ...equipmentFields(mockEquipment)],
                      (path, l) => fieldLook(path, l, crew: fullCrew, equipment: mockEquipment),
                      l,
                    ),
                    ...paletteActions(l),
                  ],
                  onPick: (_) {},
                ),
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
            if (tab != null) {
              await tester.tap(find.byKey(ValueKey(tab)));
              await tester.pumpAndSettle();
            }
            if (page == 'profile-lower') {
              await tester.drag(find.byType(Scrollable).first, const Offset(0, -2000));
              await tester.pumpAndSettle();
            }
            if (page == 'profile-changes') {
              // The sheet Save opens for the staged edits of the profile.
              final context = tester.element(find.byType(ProfileBody));
              showChangeSheet(
                context,
                changes: profileChangeRows(
                  mockEditChanges,
                  AppLocalizations.of(context)!,
                  ships: mockStaged.ships,
                  catalog: equipment?.$6 ?? mockCatalog,
                ),
                onUndo: (_) {},
              );
              await tester.pumpAndSettle();
            }
            if (page == 'profile-achievement') {
              final tile = find.byKey(const ValueKey('profile-achievement-ACH_SECTOR_5'));
              await tester.scrollUntilVisible(tile, 200, scrollable: find.byType(Scrollable).first);
              await tester.pumpAndSettle();
              await tester.tap(tile);
              await tester.pumpAndSettle();
            }
            if (page == 'profile-add') {
              final add = find.byKey(const ValueKey('profile-add-achievement'));
              await tester.scrollUntilVisible(add, 200, scrollable: find.byType(Scrollable).first);
              await tester.pumpAndSettle();
              await tester.tap(add);
              await tester.pumpAndSettle();
            }
            if (page.startsWith('status-')) {
              // The pointer over the file name or the game data item of the status bar.
              final item = find.byKey(ValueKey(page == 'status-file-hover' ? 'status-file' : 'status-game-data'));
              final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
              addTearDown(mouse.removePointer);
              await mouse.addPointer(location: Offset.zero);
              await mouse.moveTo(tester.getCenter(item));
              await tester.pumpAndSettle();
            }
            if (page == 'crew-race') {
              await tester.tap(find.byKey(const ValueKey('race-2')));
              await tester.pumpAndSettle();
            }
            if (page == 'item') {
              final add = find.byKey(const ValueKey('equip-add-cargo'));
              await tester.ensureVisible(add);
              await tester.pumpAndSettle();
              await tester.tap(add);
              await tester.pumpAndSettle();
            }
            if (page == 'augment') {
              await tester.tap(find.byKey(const ValueKey('equip-add-augment')));
              await tester.pumpAndSettle();
            }
            if (page == 'changes') {
              // The mock shell saves nothing; the editor opens this sheet on Save.
              final context = tester.element(find.byType(ShipPage));
              final l = AppLocalizations.of(context)!;
              showChangeSheet(
                context,
                changes: fieldChangeRows(
                  pendingChanges([
                    ...mockSummary.fields,
                    ...crewFields(fullCrew),
                    ...equipmentFields(mockEquipment),
                  ], edits),
                  l,
                  look: (path, l) => fieldLook(path, l, crew: fullCrew, equipment: mockEquipment),
                ),
                onUndo: (_) {},
              );
              await tester.pumpAndSettle();
            }
            if (page == 'palette') {
              await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
              await tester.sendKeyEvent(LogicalKeyboardKey.keyK);
              await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
              await tester.pumpAndSettle();
              await tester.enterText(find.byKey(const ValueKey('palette-query')), 'shield');
              await tester.pumpAndSettle();
            }
            if (page == 'actions' && equipment != null) {
              // The sheet the editor opens after Max everything on the fixture.
              final (e, choices, plan, fields, crew, _, _, _, _) = equipment;
              final context = tester.element(find.byType(ShipPage));
              final l = AppLocalizations.of(context)!;
              final old = {
                for (final f in [...fields, ...crewFields(crew), ...equipmentFields(e)]) f.path: f.value,
              };
              showChangeSheet(
                context,
                changes: fieldChangeRows(
                  [for (final c in plan.edits) PendingChange(path: c.path, old: old[c.path] ?? 0, value: c.value)],
                  l,
                  look: (path, l) => fieldLook(path, l, crew: crew, equipment: e, choices: choices),
                ),
                onUndo: (_) {},
                notes: [for (final n in plan.notes) actionNoteText(n.code, n.count, l, detail: n.message)],
              );
              await tester.pumpAndSettle();
            }
            // Image decoding runs outside the fake clock of the test.
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 400)));
            await tester.pumpAndSettle();
          },
        );
      });
    }
  }

  // The Ship page with the floorplan of the fixture (the Kestrel) from the
  // real ftl.dat, and the popover of the shields room. The shot adds fires
  // and breaches to the shields (room 5) and a breach to the doors (room 2)
  // (the fixture has none) so they show; the popover offers to put out and
  // seal them. The shields are lowered and the engines raised as pending
  // edits: the rooms carry their marks, the popover of each shows red or
  // yellow segments.
  // The square variants: the menu of an empty square of the weapons room
  // (room 10, grid 4..5, 4..5), the menu Elizabeth's square opens once a fire
  // burns there (room 10, square 1, grid 5, 4), and a fire lit and a breach
  // added by the editor in the weapons room, with their marks.
  const levels = {'systems.shields.level': 1, 'systems.engines.level': 7};
  const oxygen = {'rooms.0.oxygen': 0, 'rooms.5.oxygen': 3, 'rooms.10.oxygen': 40, 'rooms.12.oxygen': 75};
  final variants = <(String, String?, Map<String, int>)>[
    ('', null, levels),
    ('-room', 'system:shields', levels),
    ('-room-raised', 'system:engines', levels),
    ('-square-empty', 'floor-square-10-2', const {}),
    ('-square-crew-fire', 'floor-crew-2', const {'fires.5.4': 100}),
    ('-fire-lit', null, const {'fires.4.5': 100, 'breaches.5.5': 100}),
    // Low oxygen as pending edits: the pilot room empty (its popover), the
    // shields nearly so (both with FTL's stripes), the weapons room at 40
    // (its square menu), a corridor at 75; each room tinted as FTL tints it.
    ('-oxygen', 'system:pilot', oxygen),
    ('-oxygen-menu', 'floor-square-10-2', oxygen),
    // The crew card of Elizabeth from the square menu, shields raised.
    ('-crew-card', 'floor-crew-2,square-crew-2', const {'crew.2.skills.shield': 50}),
    // A staged install: the teleporter and mind control in their rooms, the
    // clone bay in the medbay's room.
    (
      '-install',
      null,
      const {'systems.mind.installed': 1, 'systems.teleporter.installed': 1, 'systems.clonebay.installed': 1},
    ),
  ];
  for (final locale in const [Locale('en'), Locale('de')]) {
    for (final (suffix, tap, edits) in variants) {
      final name = 'ftl-floorplan$suffix${locale.languageCode == 'en' ? '' : '-de'}';
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
        final read = await tester.runAsync(() async {
          final session = await openSave(path: '../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav');
          return (
            floorplanOf(session: session, nearby: false)!,
            equipmentOf(session: session, nearby: false, language: locale.languageCode),
            session.summary(),
            crewOf(session: session, nearby: false),
          );
        });
        // The ship card shows the ship of the floorplan: the fixture's own
        // summary, and the square menus and crew cards its own crew.
        final (fixturePlan, equipment, summary, crew) = read!;
        final plan = withHazards(fixturePlan);
        final refs = {
          for (final i in [plan.hull, plan.floor, for (final r in plan.rooms) ...r.interior])
            if (i != null) GameImageRef.path(i.innerPath),
          for (final s in equipment.systems) GameImageRef.of(GameImageKind.systemIcon, s.id),
          for (final c in plan.crew) crewSprite(c.lookId, anchored: true),
          ...crewCardImages(crew),
          for (final r in plan.rooms)
            if (r.system case final s?) GameImageRef.of(GameImageKind.systemIcon, s),
          for (final s in equipment.systems) systemIcon(s.id, systemIconState(s)),
          fireSheet,
          breachSheet,
          for (final (w, h) in const [(35, 70), (70, 35), (70, 70)]) lowOxygenStripes(w, h).ref,
          GameImageRef.of(GameImageKind.systemIcon, 'oxygen'),
          GameImageRef.of(GameImageKind.ship, summary.shipBlueprintId),
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
                onSave: () {},
                edits: edits,
                equipment: equipment,
                floorplan: plan,
                summary: summary,
                crew: crew,
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
          size: const Size(1280, 1240),
          before: () async {
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 400)));
            await tester.pumpAndSettle();
            // Each tap in turn: a room by its system symbol, else a widget by its key.
            for (final tap in tap?.split(',') ?? const <String>[]) {
              final key = tap.startsWith('system:')
                  ? 'floor-room-${plan.rooms.firstWhere((r) => r.system == tap.substring(7)).id}'
                  : tap;
              final target = find.byKey(ValueKey(key));
              await tester.ensureVisible(target);
              await tester.pumpAndSettle();
              await tester.tap(target);
              await tester.pumpAndSettle();
              await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 200)));
              await tester.pumpAndSettle();
            }
          },
        );
      });
    }
  }

  // The enemy ship of the fixture (JELLY_BUTTON_P_DLC) from the real
  // ftl.dat with staged edits: on the Ship page (switch, card, resources,
  // floorplan with a fire and a breach the editor adds), on the Equipment
  // page (shields and reactor raised, a Burst Laser out, an Ion Blast in,
  // mind control installed: its blueprint has no teleporter room) and in the
  // change sheet, which names each edit as one of the enemy ship.
  const enemyEdits = {
    'nearby.ship.hull': 5,
    'nearby.crew.0.health': 40,
    'nearby.systems.shields.level': 8,
    'nearby.reactor': 12,
    'nearby.fires.2.0': 100,
    'nearby.breaches.0.3': 100,
    'nearby.weapons.LASER_BURST_1': 0,
    'nearby.weapons.ION_4': 1,
    'nearby.systems.mind.installed': 1,
  };
  for (final locale in const [Locale('en'), Locale('de')]) {
    for (final page in const ['ship', 'equipment', 'changes']) {
      final name = 'ftl-enemy-$page${locale.languageCode == 'en' ? '' : '-de'}';
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
        final read = await tester.runAsync(() async {
          final session = await openSave(path: '../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav');
          return (
            session.summary(),
            floorplanOf(session: session, nearby: true)!,
            equipmentOf(session: session, nearby: true, language: language),
            crewOf(session: session, nearby: true),
            equipmentPreview(
              session: session,
              nearby: true,
              edits: [for (final e in enemyEdits.entries) FieldEdit(path: e.key, value: e.value)],
              language: language,
            ),
            itemChoices(language: language),
            // The Ion Blast the edits add, as the editor reads it on Add weapon.
            itemFields(session: session, nearby: true, id: 'ION_4')!,
          );
        });
        final (summary, plan, equipment, crew, preview, choices, ion) = read!;
        final refs = {
          for (final i in [plan.hull, plan.floor, for (final r in plan.rooms) ...r.interior])
            if (i != null) GameImageRef.path(i.innerPath),
          for (final s in equipment.systems) GameImageRef.of(GameImageKind.systemIcon, s.id),
          for (final c in plan.crew) crewSprite(c.lookId, anchored: true),
          for (final s in equipment.systems)
            for (final state in ['green1', 'green2', 'grey1', 'grey2', 'orange1', 'red1']) systemIcon(s.id, state),
          for (final i in [...preview.weapons, ...preview.drones, ...equipment.weapons])
            GameImageRef.of(i.drone ? GameImageKind.drone : GameImageKind.weaponFrame, i.id),
          for (final f in summary.nearbyFields) ?resourceImage(f.path),
          fireSheet,
          breachSheet,
          GameImageRef.of(GameImageKind.ship, summary.nearbyBlueprintId!),
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
                onSave: () {},
                edits: enemyEdits,
                summary: summary,
                floorplan: plan,
                crew: crew,
                equipment: equipment,
                equipmentPreview: preview,
                itemChoices: choices,
                nearby: true,
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
          size: Size(1280, page == 'ship' ? 1240 : 800),
          before: () async {
            if (page == 'equipment') {
              await tester.tap(find.byKey(const ValueKey('area-equipment')));
              await tester.pumpAndSettle();
            }
            if (page == 'changes') {
              final context = tester.element(find.byType(ShipPage));
              final l = AppLocalizations.of(context)!;
              final fields = [
                ...summary.nearbyFields,
                ...crewFields(crew),
                ...equipmentFields(equipment),
                ...floorplanFields(plan),
                ...itemFieldValues(ion),
              ];
              final unique = {for (final f in fields) f.path: f}.values.toList();
              showChangeSheet(
                context,
                changes: fieldChangeRows(
                  pendingChanges(unique, enemyEdits),
                  l,
                  look: (path, l) => shipNamed(
                    floorplanLook(path, l, plan, equipment) ??
                        itemLook(path, l, equipment, choices) ??
                        fieldLook(path, l, crew: crew, equipment: equipment),
                    path,
                    l,
                  ),
                ),
                onUndo: (_) {},
              );
              await tester.pumpAndSettle();
            }
            await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 400)));
            await tester.pumpAndSettle();
          },
        );
      });
    }
  }

  for (final locale in const [Locale('en'), Locale('de')]) {
    final name = 'look-ftl${locale.languageCode == 'en' ? '' : '-de'}';
    testWidgets('shot $name', skip: shotsDir.isEmpty, (tester) async {
      if (!await loadRust(tester, 'skip $name')) return;
      final look = await tester.runAsync(loadLookFromLocatedDat);
      if (look == null) {
        // ignore: avoid_print
        print('skip $name: no ftl.dat found');
        return;
      }
      await shoot(
        tester,
        name,
        variantApp(
          LookMockScreen(look: look),
          brightness: Brightness.dark,
          locale: locale,
        ),
        pixelRatio: 2,
      );
      expect(look.fonts.length, FtlFontId.values.length);
    });
  }
}

/// [plan] with a fire in the second square row of the shields room (room 5
/// of the Kestrel) and a breach in room 2 and room 5, as a burning ship
/// would answer it.
Floorplan withHazards(Floorplan plan) {
  FieldValue on(FieldValue f) => FieldValue(path: f.path, value: 100, min: f.min, max: f.max);
  FloorSquare square(FloorRoom r, FloorSquare q) => FloorSquare(
    id: q.id,
    x: q.x,
    y: q.y,
    fire: r.id == 5 && q.id >= 2 ? on(q.fire) : q.fire,
    breach: (r.id == 2 && q.id == 0) || (r.id == 5 && q.id == 1) ? on(q.breach) : q.breach,
    crew: q.crew,
  );
  return Floorplan(
    width: plan.width,
    height: plan.height,
    hull: plan.hull,
    floor: plan.floor,
    mounts: plan.mounts,
    rooms: [
      for (final r in plan.rooms)
        FloorRoom(
          id: r.id,
          x: r.x,
          y: r.y,
          w: r.w,
          h: r.h,
          system: r.system,
          interior: r.interior,
          oxygen: r.oxygen,
          oxygenField: r.oxygenField,
          squares: [for (final q in r.squares) square(r, q)],
        ),
    ],
    doors: plan.doors,
    crew: plan.crew,
  );
}

/// The images of the crew cards of [crew]: portraits and skill icons.
Set<GameImageRef> crewCardImages(List<CrewMember> crew) => {
  for (final c in crew) crewSprite(c.lookId),
  // The race picker shows every race.
  for (final r in mockRaces) crewSprite(r.lookId),
  for (final s in ['pilot', 'engine', 'shield', 'weapon', 'repair', 'combat']) skillIcon(s),
};

void _ignoreText(String path, String text) {}
