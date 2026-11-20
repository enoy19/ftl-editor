import 'package:flutter/gestures.dart';

import 'dart:ui' as ui;

import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/equipment.dart';
import 'package:ftl_editor/src/rust/api/floorplan.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/crew_sprite.dart';
import 'package:ftl_editor/ui/equipment_page.dart';
import 'package:ftl_editor/ui/floorplan/floorplan_view.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:material_ui/material_ui.dart';

import 'fake_look.dart';
import 'game_image_test.dart' show pixelPng;
import 'mock_crew.dart';
import 'mock_equipment.dart';
import 'mock_floorplan.dart';

void main() {
  WidgetController.hitTestWarningShouldBeFatal = true;

  late Map<String, int> edits;
  setUp(() => edits = {});

  Floorplan? preview;
  Equipment? previewEquipment;
  late Floorplan plan;
  late List<(String, bool)> addedItems;
  setUp(() {
    preview = null;
    previewEquipment = null;
    plan = mockFloorplan;
    addedItems = [];
  });

  Future<void> pump(WidgetTester tester, {bool ftl = false}) async {
    tester.view.physicalSize = const Size(1280, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    Widget page = StatefulBuilder(
      builder: (context, setState) => Scaffold(
        body: GameDataScope(
          datPath: '/test/ftl.dat',
          load: (_) async => pixelPng,
          child: ListView(
            children: [
              FloorplanCard(
                plan: plan,
                crew: mockCrew,
                equipment: mockEquipment,
                edits: edits,
                onChanged: (path, value) => setState(() => edits[path] = value),
                preview: preview,
                previewEquipment: previewEquipment,
                itemChoices: mockItemChoices,
                onAddItem: (id, {required toCargo}) => addedItems.add((id, toCargo)),
              ),
            ],
          ),
        ),
      ),
    );
    if (ftl) page = inFtlLook(await loadFakeLook(tester), page);
    await tester.pumpWidget(variantApp(page));
    await tester.pumpAndSettle();
  }

  testWidgets('draws hull, floor, rooms, interiors and crew at whole-number scale', (tester) async {
    await pump(tester);
    // 210 px wide fits twice into the window, not more (maxScale 2).
    expect(tester.getSize(find.byKey(const ValueKey('floorplan'))), const Size(420, 280));
    // The system symbol is the click target of the room popover, 32 px.
    final room = find.byKey(const ValueKey('floor-room-1'));
    expect(tester.getSize(room), const Size(64, 64));
    expect(tester.getSize(find.byKey(const ValueKey('floor-square-1-1'))), const Size(70, 70));
    // A room without a system has squares only.
    expect(find.byKey(const ValueKey('floor-room-2')), findsNothing);
    final crew = find.byKey(const ValueKey('floor-crew-0'));
    expect(tester.getSize(crew), const Size(70, 70));
    // Its center is the sprite center 52, 87 at scale 2.
    final origin = tester.getTopLeft(find.byKey(const ValueKey('floorplan')));
    expect(tester.getCenter(crew) - origin, const Offset(104, 174));
    final images = tester.widgetList<GameImage>(find.byType(GameImage)).map((g) => g.ref.toString()).toList();
    expect(
      images,
      containsAll([
        'img/ship/test_base.png',
        'img/ship/test_floor.png',
        'img/ship/interior/room_shields.png',
        'img/ship/interior/room_pilot.png',
        'crewLookFrame:engi.m',
        'systemIcon:shields',
        'img/effects/fire_L1_strip8.png',
        'img/effects/breach.png',
      ]),
    );
  });

  testWidgets('the rooms follow the preview: a staged install shows its system, a staged removal hides it', (
    tester,
  ) async {
    edits['systems.teleporter.installed'] = 1;
    edits['systems.pilot.installed'] = 0;
    // The core's answer for these edits: the teleporter in room 2, room 1 plain.
    preview = mockFloorplanWith({1: null, 2: 'teleporter'});
    previewEquipment = mockEquipmentWith({'teleporter': 1, 'pilot': null});
    await pump(tester);
    expect(find.byKey(const ValueKey('floor-symbol-teleporter')), findsOneWidget);
    final teleporter = tester.widget<ChangeMark>(find.byKey(const ValueKey('floor-mark-teleporter')));
    expect(teleporter.direction, EditDirection.raised);
    final images = tester.widgetList<GameImage>(find.byType(GameImage)).map((g) => g.ref.toString()).toList();
    expect(images, contains('img/ship/interior/room_teleporter.png'));
    expect(images, isNot(contains('img/ship/interior/room_pilot.png')));
    // The pilot of room 1 goes: no symbol, a lowered mark.
    expect(find.byKey(const ValueKey('floor-room-1')), findsNothing);
    expect(tester.widget<ChangeMark>(find.byKey(const ValueKey('floor-mark-pilot'))).direction, EditDirection.lowered);

    // The menu of room 2 names the teleporter and lists it at the level FTL
    // gives it. The symbol covers all but 1.5 px of the one square.
    await tester.tapAt(tester.getTopLeft(find.byKey(const ValueKey('floor-square-2-0'))) + const Offset(1.5, 1.5));
    await tester.pumpAndSettle();
    final menu = find.byKey(const ValueKey('square-menu-fires.4.1'));
    expect(menu, findsOneWidget);
    expect(find.descendant(of: menu, matching: find.text('Crew Teleporter')), findsOneWidget);
    expect(find.descendant(of: menu, matching: find.text('Crew Teleporter 1/3')), findsOneWidget);
    // Its popover takes a level at once, like the Equipment page (one SystemRow).
    await tester.tap(find.byKey(const ValueKey('square-system')));
    await tester.pumpAndSettle();
    final popover = find.byKey(const ValueKey('room-popover-2'));
    expect(popover, findsOneWidget);
    expect(tester.widget<SystemLevelRow>(find.byType(SystemLevelRow)).locked, isFalse);
    await tester.tap(find.byKey(const ValueKey('popover-close')));
    await tester.pumpAndSettle();

    // The menu of room 1 names the room, without a system.
    await tester.tap(find.byKey(const ValueKey('floor-square-1-0')));
    await tester.pumpAndSettle();
    final plain = find.byKey(const ValueKey('square-menu-fires.3.1'));
    expect(find.descendant(of: plain, matching: find.text('Room 1')), findsOneWidget);
    expect(find.descendant(of: plain, matching: find.byKey(const ValueKey('square-system'))), findsNothing);
  });

  testWidgets('without a preview the rooms show the file, whatever the edits say', (tester) async {
    edits['systems.teleporter.installed'] = 1;
    await pump(tester);
    expect(find.byKey(const ValueKey('floor-symbol-teleporter')), findsNothing);
    expect(find.byKey(const ValueKey('floor-symbol-pilot')), findsOneWidget);
  });

  testWidgets('a click on the system symbol opens its system with the level segments', (tester) async {
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('floor-room-0')));
    await tester.pumpAndSettle();
    final popover = find.byKey(const ValueKey('room-popover-0'));
    expect(popover, findsOneWidget);
    expect(find.descendant(of: popover, matching: find.byKey(const ValueKey('equip-row-shields'))), findsOneWidget);
    // The dialog route lies outside the page's GameDataScope; the popover
    // carries it along, so the system icon is the game image, not the gear.
    await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
    await tester.pump();
    expect(find.descendant(of: popover, matching: find.byIcon(Icons.settings)), findsNothing);
    // Shields are 6 of 8: the seventh segment sets level 7.
    await tester.tap(find.byKey(const ValueKey('equip-cell-shields-6')));
    await tester.pumpAndSettle();
    expect(edits, {'systems.shields.level': 7});
    expect(find.descendant(of: popover, matching: find.text('7/8')), findsOneWidget);
  });

  testWidgets('the popover row breaks and mends bars and sets the ion lock, as on the Equipment page', (tester) async {
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('floor-room-0')));
    await tester.pumpAndSettle();
    // Shields are 6 of 8: a right click on the fifth segment breaks bars 5 and 6.
    await tester.tap(find.byKey(const ValueKey('equip-cell-shields-4')), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    expect(edits, {'systems.shields.damage': 2});
    expect(find.byKey(const ValueKey('equip-seg-shields-4-damaged')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-seg-shields-5-damaged')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-damaged-shields')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('equip-ion-shields')));
    await tester.pumpAndSettle();
    expect(edits, {'systems.shields.damage': 2, 'systems.shields.ion': 1});
    expect(find.byKey(const ValueKey('equip-ion-frame-shields')), findsOneWidget);
  });

  testWidgets('a room keeps a mark of its pending level after the popover closes, red when lowered', (tester) async {
    await pump(tester);
    expect(find.byKey(const ValueKey('floor-mark-shields')), findsNothing);
    await tester.tap(find.byKey(const ValueKey('floor-room-0')));
    await tester.pumpAndSettle();
    // The second segment sets level 2, below the 6 bars the shields draw.
    await tester.tap(find.byKey(const ValueKey('equip-cell-shields-1')));
    await tester.pumpAndSettle();
    expect(edits, {'systems.shields.level': 2});
    expect(find.byKey(const ValueKey('equip-seg-shields-2-lowered')), findsOneWidget);
    expect(tester.widgetList<AppText>(find.byType(AppText)).where((t) => t.text.contains('(')), isEmpty);
    await tester.tap(find.byKey(const ValueKey('popover-close')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('room-popover-0')), findsNothing);
    final mark = find.byKey(const ValueKey('floor-mark-shields'));
    expect(tester.widget<ChangeMark>(mark).direction, EditDirection.lowered);
    // The mark lies inside the room, in its top right corner.
    // Room 0 is 70 x 70 at 35, 35, drawn at scale 2.
    final origin = tester.getTopLeft(find.byKey(const ValueKey('floorplan')));
    final room = (origin + const Offset(70, 70)) & const Size(140, 140);
    final at = tester.getRect(mark);
    expect(room.contains(at.topLeft) && room.contains(at.bottomRight), isTrue);
    expect(at.center.dx, greaterThan(room.center.dx));
    expect(at.center.dy, lessThan(room.center.dy));
    // Rooms without an edit carry none.
    expect(find.byType(ChangeMark), findsOneWidget);
  });

  testWidgets('a staged damage or ion edit marks the room and colors its floor symbol', (tester) async {
    Color symbol(String id) => tester.widget<GameImage>(find.byKey(ValueKey('floor-symbol-$id'))).color!;
    await pump(tester);
    expect(symbol('shields'), floorSymbolGray);
    // One broken bar of six: orange, and the mark of more damage (lowered).
    edits['systems.shields.damage'] = 1;
    await pump(tester);
    expect(symbol('shields'), floorSymbolOrange);
    expect(
      tester.widget<ChangeMark>(find.byKey(const ValueKey('floor-mark-shields'))).direction,
      EditDirection.lowered,
    );
    // Every bar broken: red.
    edits['systems.shields.damage'] = 6;
    await pump(tester);
    expect(symbol('shields'), floorSymbolRed);
    // An ion lock wins over damage: blue, and the pilot room is marked too.
    edits['systems.pilot.ion'] = 2;
    await pump(tester);
    expect(symbol('pilot'), floorSymbolIon);
    expect(find.byKey(const ValueKey('floor-mark-pilot')), findsOneWidget);
    // Mended again: gray, no mark.
    edits
      ..remove('systems.shields.damage')
      ..remove('systems.pilot.ion');
    await pump(tester);
    expect(symbol('shields'), floorSymbolGray);
    expect(find.byType(ChangeMark), findsNothing);
  });

  testWidgets('the room popover breaks a staged upgrade, with the row of the Equipment page', (tester) async {
    // Shields 6 in the file, raised to 8 and not saved: the preview allows 8 broken bars.
    edits['systems.shields.level'] = 8;
    previewEquipment = shieldsUpgradedPreview;
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('floor-room-0')));
    await tester.pumpAndSettle();
    SystemLevelRow shieldsRow() => tester.widget<SystemLevelRow>(
      find
          .ancestor(of: find.byKey(const ValueKey('equip-cell-shields-6')), matching: find.byType(SystemLevelRow))
          .first,
    );
    final inPopover = shieldsRow();
    await tester.tap(find.byKey(const ValueKey('equip-cell-shields-6')), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    expect(edits['systems.shields.damage'], 2);
    // The same inputs on the Equipment page draw the same row.
    await tester.pumpWidget(
      variantApp(
        Scaffold(
          body: EquipmentPage(
            equipment: mockEquipment,
            preview: shieldsUpgradedPreview,
            edits: const {'systems.shields.level': 8},
            onChanged: (_, _) {},
            addedAugments: const [],
            choices: const [],
            onAddAugment: (_) {},
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final onPage = shieldsRow();
    String describe(SystemLevelRow r) => [
      r.id,
      r.icon,
      r.title,
      r.field?.path,
      r.field?.max,
      r.value,
      r.locked,
      r.damage?.max,
      r.damageValue,
      r.ion?.max,
      r.ionValue,
      r.action != null,
    ].join('|');
    expect(describe(inPopover), describe(onPage));
  });

  testWidgets('each weapon lies on its mount under the hull, turned and mirrored as the core says', (tester) async {
    await pump(tester);
    final stack = find.descendant(of: find.byKey(const ValueKey('floorplan')), matching: find.byType(Stack)).first;
    final children = tester.widget<Stack>(stack).children;
    int indexOf(Finder f) =>
        children.indexWhere((c) => find.descendant(of: find.byWidget(c), matching: f).evaluate().isNotEmpty);
    final hull = indexOf(
      find.byWidgetPredicate((w) => w is GameImage && w.ref == const GameImageRef.path('img/ship/test_base.png')),
    );
    for (final slot in [0, 1, 2]) {
      final art = find.byKey(ValueKey('floor-weapon-$slot'));
      expect(indexOf(art), lessThan(hull), reason: 'FTL draws weapons before the hull');
      // The box of the core at scale 2.
      final m = mockFloorplan.mounts[slot];
      final origin = tester.getTopLeft(find.byKey(const ValueKey('floorplan')));
      expect(
        tester.getRect(find.byKey(ValueKey('floor-mount-$slot'))),
        Rect.fromLTWH(origin.dx + m.x * 2, origin.dy + m.y * 2, m.w * 2.0, m.h * 2.0),
      );
      expect(find.ancestor(of: art, matching: find.byType(RotatedBox)).evaluate().isNotEmpty, m.rotate);
      final flip = find.byWidgetPredicate((w) => w is Transform && w.transform.storage[0] == -1);
      expect(find.ancestor(of: art, matching: flip).evaluate().isNotEmpty, m.mirror);
    }
    expect(find.byKey(const ValueKey('floor-mount-hover-0')), findsNothing);
  });

  testWidgets('an empty mount shows under the pointer; a click opens the chooser of its slot', (tester) async {
    // The Hull Beam of slot 2 is staged for removal.
    edits['weapons.BEAM_HULL'] = 0;
    preview = mockFloorplanWith(const {}, const {}, mockMounts(weapons: ['ION_4', 'SHOTGUN_2', null]));
    previewEquipment = mockEquipmentWith(const {}, weapons: mockEquipment.weapons.take(2).toList());
    await pump(tester);
    expect(find.byKey(const ValueKey('floor-weapon-2')), findsNothing);
    expect(find.byKey(const ValueKey('floor-mount-hover-2')), findsNothing);
    final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
    addTearDown(mouse.removePointer);
    await mouse.moveTo(tester.getCenter(find.byKey(const ValueKey('floor-mount-2'))));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('floor-mount-hover-2')), findsOneWidget);
    // The chooser of the free slot marks Empty slot; a pick adds.
    await tester.tap(find.byKey(const ValueKey('floor-mount-2')));
    await tester.pumpAndSettle();
    expect(tester.widget<ListRow>(find.byKey(const ValueKey('item-choice-empty'))).selected, isTrue);
    expect(find.byKey(const ValueKey('item-power')), findsNothing, reason: 'nothing to power');
    await tester.tap(find.byKey(const ValueKey('item-choice-LASER_BURST_3')));
    await tester.pumpAndSettle();
    expect(edits, {'weapons.BEAM_HULL': 0});
    expect(addedItems, [('LASER_BURST_3', false)]);
  });

  testWidgets('a mounted weapon opens the same chooser: replace, empty, power', (tester) async {
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('floor-mount-0')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('item-choice-empty')), findsOneWidget);
    expect(tester.widget<ListRow>(find.byKey(const ValueKey('item-choice-ION_4'))).selected, isTrue);
    expect(find.byKey(const ValueKey('item-choice-COMBAT_1')), findsNothing, reason: 'weapons only');
    await tester.tap(find.byKey(const ValueKey('item-power')));
    await tester.pumpAndSettle();
    expect(edits, {'weapons.ION_4.armed': 0});
    await tester.tap(find.byKey(const ValueKey('floor-mount-1')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('item-choice-empty')));
    await tester.pumpAndSettle();
    expect(edits, {'weapons.ION_4.armed': 0, 'weapons.SHOTGUN_2': 0});
    expect(addedItems, isEmpty);
  });

  testWidgets('a mount whose weapon the preview changes carries a mark', (tester) async {
    preview = mockFloorplanWith(const {}, const {}, mockMounts(weapons: ['ION_4', null, 'LASER_BURST_3'], armed: {}));
    await pump(tester);
    expect(tester.widget<ChangeMark>(find.byKey(const ValueKey('mount-mark-0'))).direction, EditDirection.lowered);
    expect(tester.widget<ChangeMark>(find.byKey(const ValueKey('mount-mark-1'))).direction, EditDirection.lowered);
    expect(tester.widget<ChangeMark>(find.byKey(const ValueKey('mount-mark-2'))).direction, EditDirection.raised);
    // The preview's weapons are drawn: none in slot 1.
    expect(find.byKey(const ValueKey('floor-weapon-1')), findsNothing);
  });

  testWidgets('the Weapon Control popover lists the weapons with the rows of the Equipment page', (tester) async {
    plan = mockFloorplanWith(const {2: 'weapons'});
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('floor-room-2')));
    await tester.pumpAndSettle();
    final popover = find.byKey(const ValueKey('room-popover-2'));
    for (final i in [0, 1, 2]) {
      expect(find.descendant(of: popover, matching: find.byKey(ValueKey('equip-weapon-$i'))), findsOneWidget);
    }
    expect(find.descendant(of: popover, matching: find.byKey(const ValueKey('equip-drone-0'))), findsNothing);
    await tester.tap(find.byKey(const ValueKey('equip-power-weapon-0')));
    await tester.pumpAndSettle();
    expect(edits, {'weapons.ION_4.armed': 0});
    await tester.tap(find.byKey(const ValueKey('equip-remove-weapon-2')));
    await tester.pumpAndSettle();
    expect(edits['weapons.BEAM_HULL'], 0);
    expect(find.byKey(const ValueKey('equip-to-cargo-weapon-1')), findsOneWidget, reason: 'the player has a hold');
  });

  testWidgets('the Drone Control popover lists the drones', (tester) async {
    plan = mockFloorplanWith(const {2: 'drones'});
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('floor-room-2')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('equip-drone-0')), findsOneWidget);
    expect(find.byKey(const ValueKey('equip-weapon-0')), findsNothing);
  });

  testWidgets('a click on a figure opens the menu of its square; its crew row opens the card', (tester) async {
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('floor-crew-0')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('square-menu-fires.1.2')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('square-crew-0')));
    await tester.pumpAndSettle();
    final popover = find.byKey(const ValueKey('crew-popover-0'));
    expect(find.descendant(of: popover, matching: find.byKey(const ValueKey('crew-card-0'))), findsOneWidget);
    expect(find.descendant(of: popover, matching: find.text('Hollis')), findsOneWidget);
    // The card keeps the height of its content instead of the dialog's maximum.
    final card = tester.getSize(find.byKey(const ValueKey('crew-card-0'))).height;
    final screen = tester.view.physicalSize.height / tester.view.devicePixelRatio;
    expect(card, lessThan(screen / 2), reason: 'crew card $card of screen $screen');
  });

  int sheets(WidgetTester tester, String path) =>
      tester.widgetList<GameImage>(find.byType(GameImage)).where((g) => g.ref.toString() == path).length;
  const fire = 'img/effects/fire_L1_strip8.png';
  const breach = 'img/effects/breach.png';

  testWidgets('the system popover puts out each fire and seals each breach of its room', (tester) async {
    await pump(tester);
    expect(sheets(tester, fire), 1);
    await tester.tap(find.byKey(const ValueKey('floor-room-0')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('put-out-fires.2.1')));
    await tester.pumpAndSettle();
    expect(edits, {'fires.2.1': 0});
    await tester.tap(find.byKey(const ValueKey('popover-close')));
    await tester.pumpAndSettle();
    expect(sheets(tester, fire), 0);
    // The put-out square keeps a red mark until the save.
    expect(tester.widget<ChangeMark>(find.byKey(const ValueKey('square-mark-0-1'))).direction, EditDirection.lowered);
  });

  testWidgets('a click on a square opens its menu under it; a pick lights a fire or adds a breach', (tester) async {
    await pump(tester);
    final square = find.byKey(const ValueKey('floor-square-0-0'));
    await tester.tap(square);
    await tester.pumpAndSettle();
    final menu = find.byKey(const ValueKey('square-menu-fires.1.1'));
    expect(menu, findsOneWidget);
    // Anchored at the square: its top edge starts below the square.
    expect(tester.getRect(menu).top, greaterThan(tester.getRect(square).bottom));
    expect(find.descendant(of: menu, matching: find.text('Shields')), findsOneWidget);
    expect(find.byKey(const ValueKey('square-system')), findsOneWidget);
    expect(find.descendant(of: menu, matching: find.text('Light fire')), findsOneWidget);
    expect(find.descendant(of: menu, matching: find.text('Add breach')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('square-fire')));
    await tester.pumpAndSettle();
    expect(menu, findsNothing, reason: 'a pick closes the menu');
    expect(edits, {'fires.1.1': 100});
    expect(sheets(tester, fire), 2, reason: 'the new fire is drawn like the one of the file');
    expect(tester.widget<ChangeMark>(find.byKey(const ValueKey('square-mark-0-0'))).direction, EditDirection.raised);

    await tester.tap(square);
    await tester.pumpAndSettle();
    expect(find.text('Put out fire'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('square-breach')));
    await tester.pumpAndSettle();
    expect(edits, {'fires.1.1': 100, 'breaches.1.1': 100});
    expect(sheets(tester, breach), 2);

    // The breach of the file in room 2: the menu offers to seal it.
    await tester.tap(find.byKey(const ValueKey('floor-square-2-0')));
    await tester.pumpAndSettle();
    expect(find.text('Room 2'), findsOneWidget);
    expect(find.text('Seal breach'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('square-breach')));
    await tester.pumpAndSettle();
    expect(edits['breaches.4.1'], 0);
    expect(sheets(tester, breach), 1);
  });

  testWidgets('a figure on a square with more on it opens the menu, which lists the crew', (tester) async {
    edits['fires.1.2'] = 100;
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('floor-crew-0')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('crew-popover-0')), findsNothing);
    final menu = find.byKey(const ValueKey('square-menu-fires.1.2'));
    expect(find.descendant(of: menu, matching: find.text('Put out fire')), findsOneWidget);
    expect(find.descendant(of: menu, matching: find.text('Hollis')), findsOneWidget);
    // The pending fire is marked in the menu.
    expect(tester.widget<ListRow>(find.byKey(const ValueKey('square-fire'))).marked, isTrue);
    await tester.tap(find.byKey(const ValueKey('square-crew-0')));
    await tester.pumpAndSettle();
    expect(menu, findsNothing);
    expect(find.byKey(const ValueKey('crew-popover-0')), findsOneWidget);
  });

  testWidgets('the menu of a square lists the system of its room, which opens the system popover', (tester) async {
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('floor-crew-0')));
    await tester.pumpAndSettle();
    final menu = find.byKey(const ValueKey('square-menu-fires.1.2'));
    for (final key in ['square-system', 'square-fire', 'square-breach', 'square-crew-0']) {
      expect(
        find.descendant(of: menu, matching: find.byKey(ValueKey(key))),
        findsOneWidget,
        reason: key,
      );
    }
    await tester.tap(find.byKey(const ValueKey('square-system')));
    await tester.pumpAndSettle();
    expect(menu, findsNothing);
    expect(find.byType(SystemLevelRow), findsOneWidget);
  });

  testWidgets('the change sheet names fire and breach by their room', (tester) async {
    await pump(tester);
    final l = lookupAppLocalizations(const Locale('en'));
    expect(floorplanLook('fires.2.1', l, mockFloorplan, mockEquipment)?.$2, 'Fire: Shields');
    expect(floorplanLook('breaches.4.1', l, mockFloorplan, mockEquipment)?.$2, 'Hull breach: Room 2');
    expect(floorplanLook('ship.scrap', l, mockFloorplan, mockEquipment), isNull);
    expect(floorplanLook('rooms.2.oxygen', l, mockFloorplan, mockEquipment)?.$2, 'Oxygen: Room 2');
    final fields = floorplanFields(mockFloorplan);
    expect(fields.map((f) => f.path).take(4), ['rooms.0.oxygen', 'fires.1.1', 'breaches.1.1', 'fires.2.1']);
    expect(fields, hasLength(17), reason: 'three rooms with oxygen, seven squares with a fire and a breach each');
    // A search lists the oxygen of each room and only the fire and the breach the file has.
    expect(fields.where((f) => !isQuietSquare(f)).map((f) => f.path), [
      'rooms.0.oxygen',
      'fires.2.1',
      'rooms.1.oxygen',
      'rooms.2.oxygen',
      'breaches.4.1',
    ]);
  });

  testWidgets('the square menu shows the oxygen of the room and sets it through the value popover', (tester) async {
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('floor-square-0-0')));
    await tester.pumpAndSettle();
    expect(find.text('Oxygen 100%'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('square-oxygen')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('square-menu-fires.1.1')), findsNothing);
    expect(find.byKey(const ValueKey('resource-popover')), findsOneWidget);
    await tester.enterText(find.byType(EditableText), '4');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    expect(edits, {'rooms.0.oxygen': 4});
    expect(tester.widget<ChangeMark>(find.byKey(const ValueKey('oxygen-mark-0'))).direction, EditDirection.lowered);
    expect(find.byKey(const ValueKey('floor-o2-low-0')), findsOneWidget, reason: 'FTL stripes a room at 5 or less');
    expect(find.byKey(const ValueKey('floor-o2-low-1')), findsNothing);
    // The menu shows the pending value, marked.
    await tester.tap(find.byKey(const ValueKey('floor-square-0-0')));
    await tester.pumpAndSettle();
    expect(find.text('Oxygen 4%'), findsOneWidget);
    expect(tester.widget<ListRow>(find.byKey(const ValueKey('square-oxygen'))).marked, isTrue);
  });

  testWidgets('the room popover shows the oxygen with Set, which opens the value popover', (tester) async {
    edits['rooms.1.oxygen'] = 30;
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('floor-room-1')));
    await tester.pumpAndSettle();
    expect(find.text('Oxygen 30%'), findsOneWidget);
    expect(find.byKey(const ValueKey('room-oxygen-mark-1')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('oxygen-set-1')));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText).last, '100');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    expect(edits['rooms.1.oxygen'], 100);
    expect(find.text('Oxygen 100%'), findsOneWidget);
    expect(find.byKey(const ValueKey('room-oxygen-mark-1')), findsNothing);
  });

  testWidgets('a room without the oxygen system shows its oxygen but offers no edit', (tester) async {
    final plan = mockFloorplan;
    final r = plan.rooms[2];
    final bare = Floorplan(
      width: plan.width,
      height: plan.height,
      hull: plan.hull,
      floor: plan.floor,
      mounts: plan.mounts,
      rooms: [
        ...plan.rooms.take(2),
        FloorRoom(
          id: r.id,
          x: r.x,
          y: r.y,
          w: r.w,
          h: r.h,
          system: r.system,
          interior: r.interior,
          oxygen: 0,
          squares: r.squares,
        ),
      ],
      doors: plan.doors,
      crew: plan.crew,
    );
    expect(floorplanFields(bare).where((f) => f.path.startsWith('rooms.')), hasLength(2));
    await tester.pumpWidget(
      variantApp(
        Scaffold(
          body: GameDataScope(
            datPath: '/test/ftl.dat',
            load: (_) async => pixelPng,
            child: FloorplanView(
              plan: bare,
              crew: mockCrew,
              equipment: mockEquipment,
              edits: const {},
              onChanged: (_, _) {},
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('floor-o2-low-2')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('floor-square-2-0')));
    await tester.pumpAndSettle();
    expect(find.text('Oxygen 0%'), findsOneWidget);
    expect(tester.widget<ListRow>(find.byKey(const ValueKey('square-oxygen'))).onTap, isNull);
  });

  test('a room is tinted by its oxygen as FTL tints it', () {
    expect(oxygenTint(100), const Color(0xFFE6E2DB));
    expect(oxygenTint(0), const Color(0xFFFFB0A9));
    // (int)((100 - 50) * 0.25) = 12.
    expect(oxygenTint(50), const Color.fromARGB(255, 242, 202, 195));
    expect(lowOxygenStripes(35, 70).ref, const GameImageRef.path('img/effects/low_o2_stripes_1x2.png'));
    expect(lowOxygenStripes(70, 35).ref, const GameImageRef.path('img/effects/low_o2_stripes_2x1.png'));
    expect(lowOxygenStripes(105, 70).ref, const GameImageRef.path('img/effects/low_o2_stripes_2x2.png'));
  });

  testWidgets('in the FTL look a square opens its menu', (tester) async {
    await pump(tester, ftl: true);
    await tester.tap(find.byKey(const ValueKey('floor-square-1-0')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('square-menu-fires.3.1')), findsOneWidget);
    expect(findLabel('Light fire'), findsOneWidget);
  });

  testWidgets('in the FTL look a room click opens its popover too', (tester) async {
    await pump(tester, ftl: true);
    await tester.tap(find.byKey(const ValueKey('floor-room-1')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('equip-row-pilot')), findsOneWidget);
  });

  // FTL fills every room with its own floor tiles. The floor image of a ship
  // may leave squares of a room open (circle_cruiser_floor.png lacks six),
  // and there the hull showed through as a bar across the rooms.
  testWidgets('every room square is a floor tile with grid lines, also where the floor image is open', (tester) async {
    tester.view.physicalSize = const Size(400, 300);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final boundary = GlobalKey();
    await tester.pumpWidget(
      variantApp(
        Scaffold(
          body: GameDataScope(
            datPath: '/test/ftl.dat',
            // Every image, also the floor, is one transparent pixel.
            load: (_) async => pixelPng,
            child: Align(
              alignment: Alignment.topLeft,
              child: RepaintBoundary(
                key: boundary,
                child: SizedBox(
                  width: 210,
                  height: 140,
                  child: FloorplanView(
                    plan: mockFloorplan,
                    crew: mockCrew,
                    equipment: mockEquipment,
                    edits: const {},
                    onChanged: (_, _) {},
                    maxScale: 1,
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final bytes = (await tester.runAsync(() async {
      final render = boundary.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      final image = await render.toImage();
      return (await image.toByteData(format: ui.ImageByteFormat.rawRgba))!;
    }))!;
    Color at(int x, int y) {
      final i = (y * 210 + x) * 4;
      return Color.fromARGB(bytes.getUint8(i + 3), bytes.getUint8(i), bytes.getUint8(i + 1), bytes.getUint8(i + 2));
    }

    // Room 1 (pilot) is 1 x 2 squares at 105, 35: a tile in its lower square,
    // the grid line between its squares, the wall on its left.
    expect(at(115, 95), const Color(0xFFE6E2DB), reason: 'floor tile');
    expect(at(110, 70), const Color(0xFF7D7D7D), reason: 'grid line between two squares');
    expect(at(105, 95), const Color(0xFF0F0F0F), reason: 'wall');
    // Outside every room nothing is painted over the (transparent) hull.
    expect(at(10, 10).a, 0);
  });

  // FTL multiplies the white floor symbol (`s_<id>_overlay.png`) with a gray
  // (ShipSystem::OnRender, COLOR_GRAY 125,125,125); drawn as is it stood out
  // white on the light floor tiles. The test image is a white 32 x 32 square
  // with a black 1 px contour, like the overlay images.
  testWidgets('the system symbol on a room floor is FTL gray, orange, red or ion blue, its contour black', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(400, 300);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    // Earlier tests cached every symbol as the transparent pixel.
    PaintingBinding.instance.imageCache.clear();
    final symbol = (await tester.runAsync(() async {
      final recorder = ui.PictureRecorder();
      Canvas(recorder)
        ..drawRect(const Rect.fromLTWH(0, 0, 32, 32), Paint()..color = const Color(0xFF000000))
        ..drawRect(const Rect.fromLTWH(1, 1, 30, 30), Paint()..color = const Color(0xFFFFFFFF));
      final image = await recorder.endRecording().toImage(32, 32);
      return (await image.toByteData(format: ui.ImageByteFormat.png))!.buffer.asUint8List();
    }))!;

    Future<Map<Color, int>> pilotSymbol({int damaged = 0, int ionized = 0}) async {
      final equipment = Equipment(
        reactor: mockEquipment.reactor,
        systems: [
          for (final s in mockEquipment.systems)
            s.id == 'pilot'
                ? EquipmentSystem(
                    id: s.id,
                    name: s.name,
                    level: s.level,
                    installed: s.installed,
                    block: s.block,
                    power: s.power,
                    damaged: damaged,
                    ionized: ionized,
                    subsystem: s.subsystem,
                  )
                : s,
        ],
        bodyDecoded: mockEquipment.bodyDecoded,
        weapons: mockEquipment.weapons,
        drones: mockEquipment.drones,
        augments: mockEquipment.augments,
        augmentSlots: mockEquipment.augmentSlots,
        cargo: mockEquipment.cargo,
        cargoSlots: mockEquipment.cargoSlots,
        itemFields: mockEquipment.itemFields,
      );
      final boundary = GlobalKey();
      await tester.pumpWidget(
        variantApp(
          Scaffold(
            body: GameDataScope(
              datPath: '/test/ftl.dat',
              load: (ref) async => ref.kind == GameImageKind.systemIcon ? symbol : pixelPng,
              child: Align(
                alignment: Alignment.topLeft,
                child: RepaintBoundary(
                  key: boundary,
                  child: SizedBox(
                    width: 210,
                    height: 140,
                    child: FloorplanView(
                      key: UniqueKey(),
                      plan: mockFloorplan,
                      crew: mockCrew,
                      equipment: equipment,
                      edits: const {},
                      onChanged: (_, _) {},
                      maxScale: 1,
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      // The image codec decodes outside the fake clock: give it real time,
      // then paint the decoded frames.
      await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 100)));
      await tester.pumpAndSettle();
      final bytes = (await tester.runAsync(() async {
        final render = boundary.currentContext!.findRenderObject()! as RenderRepaintBoundary;
        final image = await render.toImage();
        return (await image.toByteData(format: ui.ImageByteFormat.rawRgba))!;
      }))!;
      Color at(int x, int y) {
        final i = (y * 210 + x) * 4;
        return Color.fromARGB(bytes.getUint8(i + 3), bytes.getUint8(i), bytes.getUint8(i + 1), bytes.getUint8(i + 2));
      }

      // The pilot room is 35 x 70 at 105, 35 (inside its walls from 109 to
      // 136); the 32 x 32 symbol is centered at 122.5, 70. Count the colors
      // of the box it covers, without the floor tile, the walls and the grid.
      final counts = <Color, int>{};
      for (var y = 56; y < 85; y++) {
        // Row 70 is the grid line between the two squares, in FTL gray too.
        if (y >= 69 && y <= 71) continue;
        for (var x = 110; x < 136; x++) {
          final c = at(x, y);
          if (c != const Color(0xFFE6E2DB)) counts[c] = (counts[c] ?? 0) + 1;
        }
      }
      return counts;
    }

    Color dominant(Map<Color, int> counts) => counts.entries.reduce((a, b) => a.value >= b.value ? a : b).key;
    final gray = await pilotSymbol();
    expect(dominant(gray), const Color(0xFF7D7D7D), reason: 'FTL COLOR_GRAY, not the white of the image: $gray');
    expect(gray.keys, isNot(contains(const Color(0xFFFFFFFF))), reason: 'no white pixel is left');
    expect(dominant(await pilotSymbol(damaged: 1)), const Color(0xFFFF0000), reason: 'pilot level 1, no bar left');
    expect(dominant(await pilotSymbol(ionized: 1)), const Color(0xFF85E7ED), reason: 'ion before damage');
    expect(floorSymbolColor(mockEquipment.systems.firstWhere((s) => s.id == 'shields')), floorSymbolGray);
  });

  test('the floor symbol color follows FTL: ion blue, red without bars, orange with some damage, else gray', () {
    EquipmentSystem sys(int level, int damaged, int ionized) => EquipmentSystem(
      id: 'shields',
      name: 'Shields',
      level: FieldValue(path: 'systems.shields.level', value: level, min: 0, max: 8),
      installed: const FieldValue(path: 'systems.shields.installed', value: 1, min: 1, max: 1),
      power: 0,
      damaged: damaged,
      ionized: ionized,
      subsystem: false,
    );
    expect(floorSymbolColor(sys(4, 0, 0)), const Color(0xFF7D7D7D));
    expect(floorSymbolColor(sys(4, 1, 0)), const Color(0xFFFF994C));
    expect(floorSymbolColor(sys(4, 4, 0)), const Color(0xFFFF0000));
    expect(floorSymbolColor(sys(4, 4, 2)), const Color(0xFF85E7ED));
    expect(floorSymbolColor(sys(4, 0, -1)), const Color(0xFF85E7ED));
    expect(floorSymbolColor(null), const Color(0xFF7D7D7D));
  });

  testWidgets('a female human draws the female sprite on the floorplan, as on her card', (tester) async {
    // The core's floorplan of a female human with tints: her look id.
    preview = null;
    final plan = mockFloorplan;
    final elizabeth = mockMember(0, 'Elizabeth', 'human', 100, 100, [0, 0, 0, 0, 0, 0], male: false, tints: [0, 2]);
    final female = Floorplan(
      width: plan.width,
      height: plan.height,
      hull: plan.hull,
      floor: plan.floor,
      mounts: plan.mounts,
      rooms: plan.rooms,
      doors: plan.doors,
      crew: [FloorCrew(index: 0, race: 'human', lookId: elizabeth.lookId, x: 52, y: 87)],
    );
    tester.view.physicalSize = const Size(1280, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(
      variantApp(
        Scaffold(
          body: GameDataScope(
            datPath: '/test/ftl.dat',
            load: (_) async => pixelPng,
            child: ListView(
              children: [
                FloorplanCard(
                  plan: female,
                  crew: [elizabeth],
                  equipment: mockEquipment,
                  edits: edits,
                  onChanged: (_, _) {},
                ),
              ],
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final figure = tester.widget<GameImage>(
      find.descendant(of: find.byKey(const ValueKey('floor-crew-0')), matching: find.byType(GameImage)),
    );
    expect(figure.ref.toString(), 'crewLookFrame:human.f.0.2');
    expect(figure.ref, crewSprite(elizabeth.lookId, anchored: true));
  });

  testWidgets('the crew follow the preview: a staged new member stands on the plan', (tester) async {
    final plan = mockFloorplan;
    preview = Floorplan(
      width: plan.width,
      height: plan.height,
      hull: plan.hull,
      floor: plan.floor,
      mounts: plan.mounts,
      rooms: plan.rooms,
      doors: plan.doors,
      crew: [
        ...plan.crew,
        const FloorCrew(index: 3, race: 'human', lookId: 'human.f.0.2', x: 87, y: 87),
      ],
    );
    await pump(tester);
    final figure = tester.widget<GameImage>(
      find.descendant(of: find.byKey(const ValueKey('floor-crew-3')), matching: find.byType(GameImage)),
    );
    expect(figure.ref.toString(), 'crewLookFrame:human.f.0.2');
  });
}
