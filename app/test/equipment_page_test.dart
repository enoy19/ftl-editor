import 'package:flutter/gestures.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/look/ftl_look.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/equipment.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/equipment_page.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:flutter/rendering.dart';
import 'package:material_ui/material_ui.dart';

import 'fake_look.dart' show findLabel;
import 'look_test.dart' show fake;
import 'mock_equipment.dart';

/// The minimum window of the app (macos/Runner/MainFlutterWindow.swift).
const minimumWindow = Size(1100, 700);

void main() {
  late List<(String, int)> changes;
  late List<String> added;
  late List<(String, bool)> addedItems;
  // Every callback in the order of the calls: a replace removes first.
  late List<String> calls;

  Widget page({Map<String, int> edits = const {}, List<FieldValue> addedAugments = const [], Equipment? preview}) =>
      Scaffold(
        body: EquipmentPage(
          equipment: mockEquipment,
          preview: preview,
          edits: edits,
          onChanged: (p, v) {
            changes.add((p, v));
            calls.add('$p=$v');
          },
          addedAugments: addedAugments,
          choices: mockAugmentChoices,
          onAddAugment: (id) {
            added.add(id);
            calls.add('add $id');
          },
          itemChoices: mockItemChoices,
          onAddItem: (id, {required toCargo}) {
            addedItems.add((id, toCargo));
            calls.add('add $id${toCargo ? ' to cargo' : ''}');
          },
        ),
      );

  setUp(() {
    changes = [];
    added = [];
    addedItems = [];
    calls = [];
  });

  Future<void> pump(WidgetTester tester, Widget child, {FtlLook? look}) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(
      variantApp(
        look == null ? child : Theme(data: ftlLookTheme(look), child: child),
        brightness: Brightness.dark,
      ),
    );
    await tester.pumpAndSettle();
  }

  for (final ftl in [false, true]) {
    final name = ftl ? 'FTL look' : 'Material look';
    testWidgets('$name: a segment sets a level, the top one lowers it, also below the power', (tester) async {
      final look = ftl ? await tester.runAsync(() => FtlLook.load(fake)) : null;
      await pump(tester, page(), look: look);
      await tester.tap(find.byKey(const ValueKey('equip-cell-oxygen-2')));
      // The top segment lowers by one; a lower segment sets its level, also
      // below the bars a system draws (Shields 6 of 6) and the reactor below
      // the 19 bars in use.
      await tester.tap(find.byKey(const ValueKey('equip-cell-drones-3')));
      await tester.tap(find.byKey(const ValueKey('equip-cell-engines-4')));
      await tester.tap(find.byKey(const ValueKey('equip-cell-shields-1')));
      await tester.tap(find.byKey(const ValueKey('equip-cell-reactor-24')));
      await tester.tap(find.byKey(const ValueKey('equip-cell-reactor-9')));
      expect(changes, [
        ('systems.oxygen.level', 3),
        ('systems.drones.level', 3),
        ('systems.engines.level', 4),
        ('systems.shields.level', 2),
        ('reactor', 25),
        ('reactor', 10),
      ]);
      // A system that is not installed has no segment to click.
      expect(find.byKey(const ValueKey('equip-cell-clonebay-0')), findsNothing);
    });

    testWidgets('$name: a raised segment is yellow, a lowered one red, no old value, a mark at the icon', (
      tester,
    ) async {
      final look = ftl ? await tester.runAsync(() => FtlLook.load(fake)) : null;
      await pump(tester, page(edits: const {'systems.shields.level': 8, 'systems.engines.level': 2}), look: look);
      Finder seg(String id, int i, String cell) => find.byKey(ValueKey('equip-seg-$id-$i-$cell'));
      // Shields 6 to 8: two yellow segments above six green ones.
      expect(seg('shields', 5, 'powered'), findsOneWidget);
      expect(seg('shields', 6, 'raised'), findsOneWidget);
      expect(seg('shields', 7, 'raised'), findsOneWidget);
      // Engines 5 to 2: three red segments above two green ones.
      expect(seg('engines', 1, 'powered'), findsOneWidget);
      for (final i in [2, 3, 4]) {
        expect(seg('engines', i, 'lowered'), findsOneWidget);
      }
      expect(seg('engines', 5, 'empty'), findsOneWidget);
      final t = AppTokens.of(tester.element(find.byType(EquipmentPage)));
      expect(t.warn, isNot(t.accent));
      // The new value alone, in the color of the direction; no "(6)" or "(5)".
      expect(findLabel('8/8'), findsOneWidget);
      expect(findLabel('2/8'), findsOneWidget);
      expect(tester.widgetList<AppText>(find.byType(AppText)).where((t) => t.text.contains('(')), isEmpty);
      // A mark at the icon of each edited row, none at the others.
      ChangeMark mark(String id) => tester.widget<ChangeMark>(find.byKey(ValueKey('equip-mark-$id')));
      expect(mark('shields').direction, EditDirection.raised);
      expect(mark('engines').direction, EditDirection.lowered);
      expect(find.byKey(const ValueKey('equip-mark-oxygen')), findsNothing);
      expect(find.byType(ChangeMark), findsNWidgets(2));
      // A click on a red segment raises the level back to it.
      await tester.tap(find.byKey(const ValueKey('equip-cell-engines-3')));
      expect(changes, [('systems.engines.level', 4)]);
    });

    testWidgets('$name: Remove drops an augment, Add opens the picker', (tester) async {
      final look = ftl ? await tester.runAsync(() => FtlLook.load(fake)) : null;
      await pump(tester, page(), look: look);
      await tester.ensureVisible(find.byKey(const ValueKey('equip-remove-NANO_MEDBAY')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('equip-remove-NANO_MEDBAY')));
      await tester.ensureVisible(find.byKey(const ValueKey('equip-add-augment')));
      await tester.pumpAndSettle();
      expect(changes, [('augments.NANO_MEDBAY', 0)]);
      await tester.tap(find.byKey(const ValueKey('equip-add-augment')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('augment-choice-SCRAP_COLLECTOR')));
      await tester.pumpAndSettle();
      expect(added, ['SCRAP_COLLECTOR']);
    });

    testWidgets('$name: each list keeps a row per slot, Add sits right in its heading, every action is an icon', (
      tester,
    ) async {
      final look = ftl ? await tester.runAsync(() => FtlLook.load(fake)) : null;
      await pump(tester, page(), look: look);
      // Weapons and drones fill their 3 slots, the cargo holds 1 of 4, the augments 1 of 3.
      Finder empty(String kind) => find.byWidgetPredicate(
        (w) => w.key is ValueKey<String> && (w.key! as ValueKey<String>).value.startsWith('equip-empty-$kind-'),
      );
      expect(empty('weapon'), findsNothing);
      expect(empty('drone'), findsNothing);
      expect(empty('cargo'), findsNWidgets(3));
      expect(empty('augment'), findsNWidgets(2));
      // An empty row is as high as an item row: the area keeps its size.
      expect(
        tester.getSize(find.byKey(const ValueKey('equip-empty-cargo-1'))).height,
        tester.getSize(find.byKey(const ValueKey('equip-cargo-0'))).height,
      );
      // Add stands on the line of its heading, right of the list's actions.
      final panel = tester.getRect(find.byKey(const ValueKey('equip-items')));
      for (final (add, row) in [
        ('equip-add-weapon', 'equip-weapon-0'),
        ('equip-add-drone', 'equip-drone-0'),
        ('equip-add-cargo', 'equip-cargo-0'),
        ('equip-add-augment', 'equip-augment-NANO_MEDBAY-0'),
      ]) {
        final button = tester.getRect(find.byKey(ValueKey(add)));
        expect(button.right, closeTo(panel.right, 40), reason: add);
        expect(button.bottom, lessThanOrEqualTo(tester.getRect(find.byKey(ValueKey(row))).top + 1), reason: add);
      }
      // Remove of an augment is the icon button of the other rows, with its tooltip.
      final remove = tester.widget<AppButton>(
        find
            .ancestor(of: find.byKey(const ValueKey('equip-remove-NANO_MEDBAY')), matching: find.byType(AppButton))
            .first,
      );
      expect((remove.icon, remove.label, remove.tooltip), (Icons.delete_outline, null, 'Remove'));
      // The delete icons of every list share one right edge.
      final edges = {
        for (final key in [
          'equip-remove-weapon-0',
          'equip-remove-drone-0',
          'equip-remove-cargo-0',
          'equip-remove-NANO_MEDBAY',
        ])
          tester.getRect(find.byKey(ValueKey(key))).right.round(),
      };
      expect(edges, hasLength(1));
    });

    testWidgets('$name: weapons, drones and cargo set the counts of the core within its ranges', (tester) async {
      final look = ftl ? await tester.runAsync(() => FtlLook.load(fake)) : null;
      await pump(tester, page(), look: look);
      Future<void> tap(String key) async {
        await tester.ensureVisible(find.byKey(ValueKey(key)));
        await tester.tap(find.byKey(ValueKey(key)));
        await tester.pumpAndSettle();
      }

      bool enabled(String key) => buttonEnabled(tester, key);

      // The Flak has no free bar, the slots are full, the cargo has room.
      expect(enabled('equip-power-weapon-1'), isFalse);
      expect(enabled('equip-add-weapon'), isFalse);
      expect(enabled('equip-add-drone'), isFalse);
      expect(enabled('equip-to-slot-cargo-0'), isFalse);
      await tap('equip-power-weapon-0');
      await tap('equip-remove-weapon-1');
      await tap('equip-to-cargo-weapon-2');
      await tap('equip-power-drone-2');
      await tap('equip-remove-cargo-0');
      expect(changes, [
        ('weapons.ION_4.armed', 0),
        ('weapons.SHOTGUN_2', 0),
        ('weapons.BEAM_HULL', 0),
        ('cargo.BEAM_HULL', 1),
        ('drones.ANTI_DRONE.armed', 1),
        ('cargo.REPAIR', 0),
      ]);
      // Add to cargo offers weapons and drones by name.
      await tap('equip-add-cargo');
      expect(find.byKey(const ValueKey('item-choice-LASER_BURST_3')), findsOneWidget);
      await tester.enterText(find.byKey(const ValueKey('item-filter')), 'shield');
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('item-choice-LASER_BURST_3')), findsNothing);
      await tester.tap(find.byKey(const ValueKey('item-choice-DRONE_SHIELD_PLAYER')));
      await tester.pumpAndSettle();
      expect(addedItems, [('DRONE_SHIELD_PLAYER', true)]);
      expect(find.byKey(const ValueKey('equip-read-only')), findsNothing);
    });
  }

  testWidgets('the preview shows a replaced weapon with a mark and a free slot offers Add weapon', (tester) async {
    final p = mockEquipment;
    final preview = Equipment(
      reactor: p.reactor,
      systems: p.systems,
      bodyDecoded: true,
      weapons: [
        p.weapons[0],
        p.weapons[2],
        const EquipmentItem(id: 'LASER_BURST_3', drone: false, name: 'Burst Laser III', power: 4, armed: false),
      ],
      drones: p.drones.sublist(0, 2),
      augments: p.augments,
      weaponSlots: 3,
      droneSlots: 3,
      augmentSlots: 3,
      cargo: [...p.cargo!, p.weapons[1]],
      cargoSlots: 4,
      itemFields: [
        ...p.itemFields,
        mockItemFields('LASER_BURST_3', slots: (1, 1), armed: (0, 0), cargo: (0, 2)),
      ],
    );
    await pump(tester, page(preview: preview, edits: const {'weapons.SHOTGUN_2': 0, 'cargo.SHOTGUN_2': 1}));
    final laser = find.byKey(const ValueKey('equip-weapon-2'));
    expect(find.descendant(of: laser, matching: findLabel('Burst Laser III')), findsOneWidget);
    expect(
      tester.widget<ChangeMark>(find.descendant(of: laser, matching: find.byType(ChangeMark))).direction,
      EditDirection.raised,
    );
    // The Flak in the cargo is new there; the repair drone was in the file.
    expect(
      find.descendant(of: find.byKey(const ValueKey('equip-cargo-1')), matching: find.byType(ChangeMark)),
      findsOne,
    );
    expect(
      find.descendant(of: find.byKey(const ValueKey('equip-cargo-0')), matching: find.byType(ChangeMark)),
      findsNothing,
    );
    // A drone slot is free in the preview.
    expect(buttonEnabled(tester, 'equip-add-drone'), isTrue);
  });

  test('the change sheet names weapon, drone and cargo paths', () {
    final l = lookupAppLocalizations(const Locale('en'));
    expect(itemLook('weapons.ION_4', l, mockEquipment, mockItemChoices)?.$2, 'Ion Blast Mark II');
    expect(itemLook('drones.COMBAT_1.armed', l, mockEquipment, mockItemChoices)?.$2, 'Combat Drone Mark I: powered');
    expect(itemLook('cargo.LASER_BURST_3', l, mockEquipment, mockItemChoices)?.$2, 'Burst Laser III in the cargo');
    expect(itemLook('augments.NANO_MEDBAY', l, mockEquipment, mockItemChoices), isNull);
    expect(equipmentFields(mockEquipment).map((f) => f.path), contains('cargo.REPAIR'));
  });

  testWidgets('a long name keeps a gap to the broken count of its row', (tester) async {
    final look = (await tester.runAsync(() => FtlLook.load(fake)))!;
    const damage = FieldValue(path: 'systems.weapons.damage', value: 0, min: 0, max: 8);
    await pump(
      tester,
      Align(
        alignment: Alignment.topLeft,
        // The width of the floorplan room popover.
        child: SizedBox(
          width: 488,
          child: SystemLevelRow(
            id: 'weapons',
            icon: null,
            title: 'Waffensteuerung',
            field: const FieldValue(path: 'systems.weapons.level', value: 8, min: 1, max: 8),
            value: 8,
            damage: damage,
            damageValue: 3,
            onSet: (_) {},
            onDamage: (_) {},
          ),
        ),
      ),
      look: look,
    );
    final name = tester.getRect(find.byKey(const ValueKey('equip-name-weapons')));
    final count = tester.getRect(find.byKey(const ValueKey('equip-damaged-weapons')));
    expect(count.left - name.right, greaterThanOrEqualTo(Space.sm));
  });

  testWidgets('the FTL look paints a raised segment in its yellow and a lowered one in its red', (tester) async {
    final look = (await tester.runAsync(() => FtlLook.load(fake)))!;
    final boundary = GlobalKey();
    FieldValue level(String id, int value) => FieldValue(path: 'systems.$id.level', value: value, min: 1, max: 8);
    await pump(
      tester,
      RepaintBoundary(
        key: boundary,
        child: Column(
          children: [
            SystemLevelRow(id: 'up', icon: null, title: 'Up', field: level('up', 2), value: 4, onSet: (_) {}),
            SystemLevelRow(id: 'down', icon: null, title: 'Down', field: level('down', 4), value: 2, onSet: (_) {}),
          ],
        ),
      ),
      look: look,
    );
    final image = (await tester.runAsync(() async {
      final render = boundary.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      final img = await render.toImage();
      return (img.width, (await img.toByteData())!);
    }))!;
    final origin = tester.getTopLeft(find.byKey(boundary));
    Color at(String id, int i) {
      final p = tester.getCenter(find.byKey(ValueKey('equip-cell-$id-$i'))) - origin;
      final o = (p.dy.round() * image.$1 + p.dx.round()) * 4;
      final b = image.$2;
      return Color.fromARGB(b.getUint8(o + 3), b.getUint8(o), b.getUint8(o + 1), b.getUint8(o + 2));
    }

    expect(at('up', 1), powerGreen);
    expect(at('up', 2), FtlColors.hover);
    expect(at('up', 3), FtlColors.hover);
    expect(at('down', 1), powerGreen);
    expect(at('down', 2), FtlColors.warn);
    expect(at('down', 3), FtlColors.warn);
  });

  for (final ftl in [true, false]) {
    final name = ftl ? 'FTL look' : 'Material look';
    testWidgets('$name: at the minimum window the system rows stack at full segment size, titles stay whole', (
      tester,
    ) async {
      final look = ftl ? await tester.runAsync(() => FtlLook.load(fake)) : null;
      await pump(tester, page(), look: look);
      Rect rect(String key) => tester.getRect(find.byKey(ValueKey(key)));
      // Wide: the segments stand on the line of the name.
      expect(find.byKey(const ValueKey('equip-bars-shields')), findsNothing);
      final button = rect('equip-install-system-clonebay').size;
      // The minimum window of the app (macos/Runner/MainFlutterWindow.swift).
      tester.view.physicalSize = minimumWindow;
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      // Every row stacks: the segments and Install/Remove on a line of their
      // own under the name, at the size of a wide window.
      for (final id in ['reactor', 'shields', 'drones', 'clonebay']) {
        final row = rect('equip-row-$id');
        final bars = rect('equip-bars-$id');
        expect(bars.top, greaterThan(row.top + 20), reason: id);
        expect(row.bottom, greaterThan(bars.bottom), reason: '$id: space below each system');
      }
      expect(rect('equip-cell-shields-0').size, const Size(22, 18));
      expect(rect('equip-cell-reactor-0').size, const Size(8, 18));
      expect(rect('equip-install-system-clonebay').size, button, reason: 'the button keeps its size');
      // The headings of the lists keep their whole title.
      for (final title in ['WEAPONS', 'DRONES', 'CARGO', 'AUGMENTS', 'Weapons', 'Drones', 'Cargo', 'Augments']) {
        final label = findLabel(title);
        if (label.evaluate().isEmpty) continue;
        final w = label.evaluate().single.widget;
        final need = w is FtlText ? w.font.width(w.text) * w.scale : null;
        if (need != null) {
          expect(tester.getSize(label).width, greaterThanOrEqualTo(need), reason: '$title is cut');
        } else {
          final text = tester.renderObject<RenderParagraph>(
            find.descendant(of: label, matching: find.byType(RichText)),
          );
          expect(text.didExceedMaxLines, isFalse, reason: '$title is cut');
        }
      }
      // Below the window, yet built: a test can scroll to them.
      expect(find.byKey(const ValueKey('equip-add-augment')), findsOneWidget);
      expect(find.byKey(const ValueKey('equip-row-battery')), findsOneWidget);
      await tester.ensureVisible(find.byKey(const ValueKey('equip-cell-oxygen-1')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('equip-cell-oxygen-1')));
      expect(changes, [('systems.oxygen.level', 2)]);
    });

    testWidgets('$name: a click on a slot opens the chooser of its list, Empty slot first', (tester) async {
      final look = ftl ? await tester.runAsync(() => FtlLook.load(fake)) : null;
      await pump(tester, page(), look: look);
      Future<void> tap(String key) async {
        await tester.ensureVisible(find.byKey(ValueKey(key)));
        await tester.pumpAndSettle();
        await tester.tap(find.byKey(ValueKey(key)));
        await tester.pumpAndSettle();
      }

      // A filled weapon slot: the chooser marks the ion blast, lists only
      // weapons, and a pick replaces it (the removal first).
      await tap('equip-weapon-0');
      final empty = tester.getRect(find.byKey(const ValueKey('item-choice-empty')));
      expect(empty.top, lessThan(tester.getRect(find.byKey(const ValueKey('item-choice-LASER_BURST_3'))).top));
      expect(tester.widget<ListRow>(find.byKey(const ValueKey('item-choice-ION_4'))).selected, isTrue);
      expect(find.byKey(const ValueKey('item-choice-COMBAT_1')), findsNothing, reason: 'no drone in a weapon slot');
      await tap('item-choice-LASER_BURST_3');
      expect(calls, ['weapons.ION_4=0', 'add LASER_BURST_3']);
      // The chooser of a filled slot powers it off (and closes).
      calls.clear();
      await tap('equip-weapon-0');
      await tap('item-power');
      expect(calls, ['weapons.ION_4.armed=0']);
      expect(find.byKey(const ValueKey('item-filter')), findsNothing);
      // Empty slot clears a drone slot; the same item changes nothing.
      calls.clear();
      await tap('equip-drone-2');
      await tap('item-choice-empty');
      await tap('equip-drone-0');
      await tap('item-choice-COMBAT_1');
      expect(calls, ['drones.ANTI_DRONE=0']);
      // An empty cargo box fills; its Empty slot is the marked entry.
      calls.clear();
      await tap('equip-empty-cargo-2');
      expect(tester.widget<ListRow>(find.byKey(const ValueKey('item-choice-empty'))).selected, isTrue);
      await tap('item-choice-COMBAT_1');
      // An augment is replaced through the same chooser.
      await tap('equip-augment-NANO_MEDBAY-0');
      await tap('augment-choice-SCRAP_COLLECTOR');
      await tap('equip-empty-augment-2');
      await tap('augment-choice-AUTO_COOLDOWN');
      expect(calls, ['add COMBAT_1 to cargo', 'augments.NANO_MEDBAY=0', 'add SCRAP_COLLECTOR', 'add AUTO_COOLDOWN']);
      // Add is a plus with the label as its tooltip.
      final add = tester.widget<AppButton>(
        find.ancestor(of: find.byKey(const ValueKey('equip-add-cargo')), matching: find.byType(AppButton)).first,
      );
      expect((add.icon, add.label, add.tooltip), (Icons.add, null, 'Add to cargo'));
    });
  }

  testWidgets('a fourth augment is refused: Add is off with three in all', (tester) async {
    await pump(
      tester,
      page(
        edits: const {'augments.NANO_MEDBAY': 2, 'augments.SCRAP_COLLECTOR': 1},
        addedAugments: [mockAugmentField('SCRAP_COLLECTOR')],
      ),
    );
    expect(find.byKey(const ValueKey('equip-remove-SCRAP_COLLECTOR')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('equip-add-augment')), warnIfMissed: false);
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('augment-choice-AUTO_COOLDOWN')), findsNothing);
  });

  testWidgets('Install and Remove stage systems.<id>.installed where the core allows it', (tester) async {
    await pump(tester, page());
    final l = AppLocalizations.of(tester.element(find.byType(EquipmentPage)))!;
    // AppButton hands its key to the button it draws.
    AppButton button(String key) =>
        tester.widget<AppButton>(find.ancestor(of: find.byKey(ValueKey(key)), matching: find.byType(AppButton)).first);
    // The clone bay may replace the medbay; its tooltip says so.
    expect(button('equip-install-system-clonebay').onPressed, isNotNull);
    expect(find.byTooltip(l.equipReplaces('Medbay')), findsOneWidget);
    // The ship is full: teleporter and hacking stay grey with the reason.
    expect(button('equip-install-system-teleporter').onPressed, isNull);
    expect(find.byTooltip(l.equipBlockSystemLimit), findsNWidgets(2));
    // Bought systems can go, start systems have no button.
    expect(button('equip-remove-system-mind').onPressed, isNotNull);
    expect(find.byKey(const ValueKey('equip-remove-system-shields')), findsNothing);
    await tester.tap(find.byKey(const ValueKey('equip-install-system-clonebay')));
    await tester.tap(find.byKey(const ValueKey('equip-remove-system-mind')));
    expect(changes, [('systems.clonebay.installed', 1), ('systems.mind.installed', 0)]);
  });

  testWidgets(
    'a staged install shows the new level in yellow and takes a level, a staged removal shows the old in red, locked',
    (tester) async {
      EquipmentSystem withLevel(EquipmentSystem s, int level) => EquipmentSystem(
        id: s.id,
        name: s.name,
        level: FieldValue(path: 'systems.${s.id}.level', value: level, min: 1, max: 3),
        installed: FieldValue(path: s.installed.path, value: 1, min: 0, max: 1),
        block: null,
        power: 0,
        damaged: 0,
        ionized: 0,
        subsystem: false,
      );
      final preview = Equipment(
        reactor: mockEquipment.reactor,
        systems: [for (final s in mockEquipment.systems) s.id == 'clonebay' ? withLevel(s, 1) : s],
        bodyDecoded: mockEquipment.bodyDecoded,
        weapons: mockEquipment.weapons,
        drones: mockEquipment.drones,
        augments: mockEquipment.augments,
        weaponSlots: mockEquipment.weaponSlots,
        droneSlots: mockEquipment.droneSlots,
        augmentSlots: mockEquipment.augmentSlots,
        cargo: mockEquipment.cargo,
        cargoSlots: mockEquipment.cargoSlots,
        itemFields: mockEquipment.itemFields,
      );
      await pump(tester, page(edits: {'systems.clonebay.installed': 1, 'systems.mind.installed': 0}, preview: preview));
      expect(find.byKey(const ValueKey('equip-seg-clonebay-0-raised')), findsOneWidget);
      expect(
        tester.widget<ChangeMark>(find.byKey(const ValueKey('equip-mark-clonebay'))).direction,
        EditDirection.raised,
      );
      expect(find.byKey(const ValueKey('equip-seg-mind-0-lowered')), findsOneWidget);
      expect(tester.widget<ChangeMark>(find.byKey(const ValueKey('equip-mark-mind'))).direction, EditDirection.lowered);
      // The staged install takes a level at once; the staged removal is locked.
      // The buttons undo the changes.
      await tester.tap(find.byKey(const ValueKey('equip-cell-clonebay-1')));
      expect(find.byKey(const ValueKey('equip-cell-mind-0')), findsNothing);
      await tester.tap(find.byKey(const ValueKey('equip-remove-system-clonebay')));
      await tester.tap(find.byKey(const ValueKey('equip-install-system-mind')));
      expect(changes, [
        ('systems.clonebay.level', 2),
        ('systems.clonebay.installed', 0),
        ('systems.mind.installed', 1),
      ]);
    },
  );

  testWidgets('undoing a staged removal needs a free place; undoing a staged install never does', (tester) async {
    EquipmentSystem as(EquipmentSystem s, {required int? level, required (int, int) install, String? block}) =>
        EquipmentSystem(
          id: s.id,
          name: s.name,
          level: level == null ? null : FieldValue(path: 'systems.${s.id}.level', value: level, min: 1, max: 3),
          installed: FieldValue(path: s.installed.path, value: level == null ? 0 : 1, min: install.$1, max: install.$2),
          block: block,
          power: 0,
          damaged: 0,
          ionized: 0,
          subsystem: false,
        );
    // Mind control out, hacking in: the ship holds 8 systems again, so the
    // core refuses mind control back.
    final preview = Equipment(
      reactor: mockEquipment.reactor,
      systems: [
        for (final s in mockEquipment.systems)
          switch (s.id) {
            'mind' => as(s, level: null, install: (0, 0), block: 'system_limit'),
            'hacking' => as(s, level: 1, install: (0, 1)),
            _ => s,
          },
      ],
      bodyDecoded: mockEquipment.bodyDecoded,
      weapons: mockEquipment.weapons,
      drones: mockEquipment.drones,
      augments: mockEquipment.augments,
      weaponSlots: mockEquipment.weaponSlots,
      droneSlots: mockEquipment.droneSlots,
      augmentSlots: mockEquipment.augmentSlots,
      cargo: mockEquipment.cargo,
      cargoSlots: mockEquipment.cargoSlots,
      itemFields: mockEquipment.itemFields,
    );
    await pump(tester, page(edits: {'systems.mind.installed': 0, 'systems.hacking.installed': 1}, preview: preview));
    AppButton button(String key) =>
        tester.widget<AppButton>(find.ancestor(of: find.byKey(ValueKey(key)), matching: find.byType(AppButton)).first);
    expect(button('equip-install-system-mind').onPressed, isNull);
    expect(button('equip-remove-system-hacking').onPressed, isNotNull);
  });

  testWidgets('broken bars are the top segments in red, ion puts the blue outline around them', (tester) async {
    await pump(tester, page(edits: {'systems.shields.damage': 2, 'systems.shields.ion': 1}));
    // Shields at level 6: bars 4 and 5 broken, as FTL breaks the top of the column.
    for (var i = 0; i < 4; i++) {
      expect(find.byKey(ValueKey('equip-seg-shields-$i-powered')), findsOneWidget);
    }
    expect(find.byKey(const ValueKey('equip-seg-shields-4-damaged')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-seg-shields-5-damaged')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-damaged-shields')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-ion-frame-shields')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-ion-frame-engines')), findsNothing);
    expect(
      tester.widget<ChangeMark>(find.byKey(const ValueKey('equip-mark-shields'))).direction,
      EditDirection.lowered,
    );
  });

  testWidgets('a right click breaks a segment and those above, or mends a broken one and those below', (tester) async {
    await pump(tester, page());
    await tester.tap(find.byKey(const ValueKey('equip-cell-shields-3')), buttons: kSecondaryButton);
    expect(changes, [('systems.shields.damage', 3)]);
    changes.clear();
    await pump(tester, page(edits: {'systems.shields.damage': 2}));
    await tester.tap(find.byKey(const ValueKey('equip-cell-shields-4')), buttons: kSecondaryButton);
    await tester.tap(find.byKey(const ValueKey('equip-cell-shields-5')), buttons: kSecondaryButton);
    expect(changes, [('systems.shields.damage', 1), ('systems.shields.damage', 0)]);
  });

  testWidgets('a staged upgrade can be broken before the save: the damage range comes from the preview', (
    tester,
  ) async {
    // Shields 6 in the file, raised to 8 and not saved; the preview allows 8 broken bars.
    await pump(tester, page(edits: {'systems.shields.level': 8}, preview: shieldsUpgradedPreview));
    await tester.tap(find.byKey(const ValueKey('equip-cell-shields-6')), buttons: kSecondaryButton);
    expect(changes, [('systems.shields.damage', 2)]);
  });

  testWidgets('a click on the lock adds one ion up to 5, then clears it', (tester) async {
    for (final ion in [0, 3, 5]) {
      // A fresh tree for each value of the file.
      await tester.pumpWidget(const SizedBox());
      await pump(tester, page(edits: {'systems.shields.ion': ion}));
      await tester.tap(find.byKey(const ValueKey('equip-ion-shields')));
    }
    expect(changes, [('systems.shields.ion', 1), ('systems.shields.ion', 4), ('systems.shields.ion', 0)]);
  });

  testWidgets('the Zoltan shield lies over the shield bars: a click sets that many points, the top bar one less', (
    tester,
  ) async {
    await pump(tester, page());
    // Five free points over the shields, none over any other system.
    expect(find.byKey(const ValueKey('equip-zoltan-shields')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-zoltan-engines')), findsNothing);
    expect(find.byKey(const ValueKey('equip-zoltan-shields-4-free')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('equip-zoltan-cell-shields-4')));
    expect(changes, [('systems.shields.energy', 5)]);
    changes.clear();
    await pump(tester, page(edits: {'systems.shields.energy': 3}));
    expect(find.byKey(const ValueKey('equip-zoltan-shields-2-on')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-zoltan-shields-3-free')), findsOneWidget);
    expect(tester.widget<ChangeMark>(find.byKey(const ValueKey('equip-mark-shields'))).direction, EditDirection.raised);
    await tester.tap(find.byKey(const ValueKey('equip-zoltan-cell-shields-2')));
    await tester.tap(find.byKey(const ValueKey('equip-zoltan-cell-shields-0')));
    expect(changes, [('systems.shields.energy', 2), ('systems.shields.energy', 1)]);
  });

  testWidgets('the change sheet names damage and ion paths by their system', (tester) async {
    await pump(tester, page());
    final l = AppLocalizations.of(tester.element(find.byType(EquipmentPage)))!;
    expect(equipmentLook('systems.shields.damage', l, mockEquipment, const [])?.$2, l.equipSystemDamage('Shields'));
    expect(equipmentLook('nearby.systems.shields.ion', l, mockEquipment, const [])?.$2, l.equipSystemIon('Shields'));
    expect(
      equipmentLook('nearby.systems.shields.energy', l, mockEquipment, const [])?.$2,
      l.equipZoltanShield('Shields'),
    );
    expect(
      equipmentFields(mockEquipment).map((f) => f.path),
      containsAll(['systems.shields.damage', 'systems.shields.ion', 'systems.shields.energy']),
    );
  });

  testWidgets('the change sheet names an install path by its system', (tester) async {
    await pump(tester, page());
    final l = AppLocalizations.of(tester.element(find.byType(EquipmentPage)))!;
    expect(
      equipmentLook('systems.clonebay.installed', l, mockEquipment, const [])?.$2,
      l.equipSystemInstalled('Clone Bay'),
    );
    expect(installBlockText('swap_back', l), l.equipBlockSwapBack);
  });

  test('the icon state follows FTL: green, orange with damage, red when broken, grey when missing', () {
    EquipmentSystem sys(int level, int damaged) => EquipmentSystem(
      id: 'shields',
      name: 'Shields',
      level: level == 0 ? null : FieldValue(path: 'systems.shields.level', value: level, min: 0, max: 8),
      installed: FieldValue(path: 'systems.shields.installed', value: level == 0 ? 0 : 1, min: 1, max: 1),
      power: 0,
      damaged: damaged,
      ionized: 0,
      subsystem: false,
    );
    expect(systemIconState(sys(4, 0)), 'green1');
    expect(systemIconState(sys(4, 1)), 'orange1');
    expect(systemIconState(sys(4, 4)), 'red1');
    expect(systemIconState(sys(0, 0)), 'grey1');
    expect(systemIconState(sys(4, 0), hovered: true), 'green2');
    expect(systemIconState(sys(0, 0), hovered: true), 'grey2');
  });

  testWidgets('each system row shows its FTL icon; a missing system the grey one', (tester) async {
    await pump(tester, page());
    String? iconOf(String id) {
      final row = find.byKey(ValueKey('equip-row-$id'));
      // The state icon comes first; the ion lock follows it.
      final image = tester.widget<GameImage>(find.descendant(of: row, matching: find.byType(GameImage)).first);
      return image.ref.path;
    }

    expect(iconOf('shields'), 'img/icons/s_shields_green1.png');
    expect(iconOf('clonebay'), 'img/icons/s_clonebay_grey1.png');
    // The reactor row comes first and has no icon.
    expect(
      find.descendant(of: find.byKey(const ValueKey('equip-row-reactor')), matching: find.byType(GameImage)),
      findsNothing,
    );
    expect(
      tester.getTopLeft(find.byKey(const ValueKey('equip-row-reactor'))).dy,
      lessThan(tester.getTopLeft(find.byKey(const ValueKey('equip-row-shields'))).dy),
    );
  });

  testWidgets('the change sheet names equipment fields', (tester) async {
    await pump(tester, const SizedBox());
    final l = lookupAppLocalizations(const Locale('en'));
    expect(equipmentLook('reactor', l, mockEquipment, mockAugmentChoices)?.$2, l.equipReactor);
    expect(equipmentLook('systems.shields.level', l, mockEquipment, mockAugmentChoices)?.$2, 'Shields');
    expect(equipmentLook('augments.SCRAP_COLLECTOR', l, mockEquipment, mockAugmentChoices)?.$2, 'Scrap Recovery Arm');
    expect(equipmentLook('ship.scrap', l, mockEquipment, mockAugmentChoices), isNull);
  });
}

/// Whether the button an [AppButton] with [key] draws takes taps; the
/// Material and FTL buttons all carry `onPressed`.
bool buttonEnabled(WidgetTester tester, String key) =>
    (tester.widget(find.byKey(ValueKey(key))) as dynamic).onPressed != null;
