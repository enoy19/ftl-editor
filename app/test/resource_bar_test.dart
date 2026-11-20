import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/resource_bar.dart';
import 'package:material_ui/material_ui.dart';

import 'fake_look.dart';
import 'mock_summary.dart';

/// Pumps the bar over [mockSummary]; answers the reported changes.
Future<List<(String, int)>> pumpBar(
  WidgetTester tester, {
  Map<String, int> edits = const {},
  Alignment alignment = Alignment.topLeft,
}) async {
  tester.view.physicalSize = const Size(1280, 800);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final changes = <(String, int)>[];
  await tester.pumpWidget(
    variantApp(
      Scaffold(
        body: Align(
          alignment: alignment,
          child: ResourceBar(fields: mockSummary.fields, edits: edits, onChanged: (p, v) => changes.add((p, v))),
        ),
      ),
    ),
  );
  return changes;
}

Future<void> openPopover(WidgetTester tester, String path) async {
  await tester.tap(find.byKey(ValueKey('resource-$path')));
  await tester.pumpAndSettle();
  expect(find.byKey(const ValueKey('resource-popover')), findsOneWidget);
}

final popoverField = find.descendant(
  of: find.byKey(const ValueKey('resource-popover')),
  matching: find.byType(TextField),
);

void main() {
  testWidgets('the bar shows icon and number of each resource', (tester) async {
    await pumpBar(tester);
    for (final v in ['22', '14', '3', '8', '214']) {
      expect(find.text(v), findsOneWidget, reason: v);
    }
    expect(find.byType(TextField), findsNothing, reason: 'no input field until a click');
    expect(find.byIcon(Icons.favorite), findsOneWidget);
  });

  testWidgets('a click opens the popover; Apply reports the new value', (tester) async {
    final changes = await pumpBar(tester);
    await openPopover(tester, 'ship.scrap');
    await tester.enterText(popoverField, '500');
    await tester.tap(find.text('Apply'));
    await tester.pumpAndSettle();
    expect(changes, [('ship.scrap', 500)]);
    expect(find.byKey(const ValueKey('resource-popover')), findsNothing);
  });

  testWidgets('Enter applies like the button', (tester) async {
    final changes = await pumpBar(tester);
    await openPopover(tester, 'ship.fuel');
    await tester.enterText(popoverField, '30');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    expect(changes, [('ship.fuel', 30)]);
  });

  testWidgets('a value out of range or not a number is refused', (tester) async {
    final changes = await pumpBar(tester);
    await openPopover(tester, 'ship.hull');
    await tester.enterText(popoverField, '31');
    await tester.tap(find.text('Apply'));
    await tester.pumpAndSettle();
    expect(find.text('Allowed: 0 to 30'), findsOneWidget);
    expect(find.byKey(const ValueKey('resource-popover')), findsOneWidget, reason: 'stays open');

    await tester.enterText(popoverField, 'abc');
    await tester.tap(find.text('Apply'));
    await tester.pumpAndSettle();
    expect(find.text('Not a number'), findsOneWidget);
    expect(changes, isEmpty);

    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('resource-popover')), findsNothing);
    expect(changes, isEmpty);
  });

  testWidgets('the max button fills the maximum of the field', (tester) async {
    final changes = await pumpBar(tester);
    await openPopover(tester, 'ship.scrap');
    await tester.tap(find.byTooltip('Set to maximum (99999)'));
    await tester.pump();
    expect(tester.widget<TextField>(popoverField).controller!.text, '99999');
    await tester.tap(find.text('Apply'));
    await tester.pumpAndSettle();
    expect(changes, [('ship.scrap', 99999)]);
  });

  testWidgets('an edited value shows instead of the file value', (tester) async {
    await pumpBar(tester, edits: {'ship.missiles': 40});
    expect(find.text('40'), findsOneWidget);
    expect(find.text('8'), findsNothing);
    expect(find.byTooltip('Missiles: changed, 8 in the file'), findsOneWidget);
  });

  testWidgets('near the bottom of the window the popover opens above the bar', (tester) async {
    await pumpBar(tester, alignment: Alignment.bottomRight);
    await openPopover(tester, 'ship.scrap');
    final popover = tester.getRect(find.byKey(const ValueKey('resource-popover')));
    final chip = tester.getRect(find.byKey(const ValueKey('resource-ship.scrap')));
    expect(popover.bottom, lessThanOrEqualTo(chip.top));
    expect(popover.right, lessThanOrEqualTo(1280));
    await tester.tap(find.text('Apply'));
    await tester.pumpAndSettle();
  });

  testWidgets('in the FTL look every resource is one box of one size, centered with the label', (tester) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final look = await loadFakeLook(tester);
    await tester.pumpWidget(
      variantApp(
        inFtlLook(
          look,
          Scaffold(
            body: ResourceCard(fields: mockSummary.fields, edits: const {'ship.scrap': 99999}, onChanged: (_, _) {}),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final rects = [for (final f in mockSummary.fields) tester.getRect(find.byKey(ValueKey('resource-${f.path}')))];
    for (final r in rects) {
      expect(r.size, rects.first.size, reason: 'one size for every resource');
      expect(r.center.dy, rects.first.center.dy, reason: 'one row');
    }
    // One gap between neighbours.
    final gaps = {for (var i = 1; i < rects.length; i++) rects[i].left - rects[i - 1].right};
    expect(gaps, hasLength(1));
    // The label sits on the middle line of the boxes, and the boxes in the middle of the window.
    expect(tester.getCenter(find.byIcon(Icons.inventory_2)).dy, closeTo(rects.first.center.dy, 0.5));
    final card = tester.getRect(find.byType(ResourceCard));
    expect(rects.first.center.dy, closeTo(card.center.dy, 0.5));
    // Five digits fit the box.
    expect(tester.takeException(), isNull);
  });

  testWidgets('with ftl.dat the boxes show FTL\'s own icons at their own size, the numbers centered', (tester) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final look = await loadFakeLook(tester);
    await tester.pumpWidget(
      variantApp(
        inFtlLook(
          look,
          GameDataScope(
            datPath: '/games/FTL/ftl.dat',
            load: (ref) async => throw StateError('no image in a widget test'),
            child: Scaffold(
              body: ResourceCard(fields: mockSummary.fields, edits: const {}, onChanged: (_, _) {}),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final expected = {
      'ship.hull': 'img/map/map_icon_ship.png',
      'ship.fuel': 'img/ui_icons/icon_fuel.png',
      'ship.drone_parts': 'img/ui_icons/icon_drones.png',
      'ship.missiles': 'img/ui_icons/icon_missiles.png',
      'ship.scrap': 'img/ui_icons/icon_scrap.png',
    };
    for (final f in mockSummary.fields) {
      final box = find.byKey(ValueKey('resource-${f.path}'));
      final image = tester.widget<GameImage>(find.descendant(of: box, matching: find.byType(GameImage)));
      expect(image.ref, GameImageRef.path(expected[f.path]!), reason: f.path);
      expect(image.fit, BoxFit.none, reason: 'factor 1: pixel art stays sharp');
      expect((image.width, image.height), (resourceIconSize, resourceIconSize));
      final number = find.descendant(of: box, matching: find.byType(AppText));
      expect(tester.getCenter(number).dy, tester.getCenter(box).dy, reason: 'the number of ${f.path} is centered');
      expect(tester.getRect(box).size, ftlResourceSize);
    }
  });

  testWidgets('without ftl.dat the resources keep the Material icons', (tester) async {
    await pumpBar(tester);
    expect(find.byType(GameImage), findsNothing);
    expect(find.byIcon(Icons.local_gas_station), findsOneWidget);
  });
}
