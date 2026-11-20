// The Map page over a mock sector map: beacons by key, the popover of a
// store beacon stages restock, supply and swap edits, a beacon becomes a
// store or takes the ship, the toolbar moves the fleet back and forward,
// the map fills the page, and every map path has a label. No Rust core.

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/map.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/map_page.dart';
import 'package:material_ui/material_ui.dart';

FieldValue fv(String path, int value, int min, int max) => FieldValue(path: path, value: value, min: min, max: max);

BeaconView beacon(int id, int x, int y, {bool current = false, StoreView? store, String fleet = 'none'}) => BeaconView(
  id: id,
  x: x,
  y: y,
  seen: fv('beacons.$id.seen', 1, 0, 1),
  visited: current ? null : fv('beacons.$id.visited', 1, 0, 1),
  fleet: fleet,
  underAttack: false,
  current: current,
  store: store,
  hasStore: fv('beacons.$id.store', store == null ? 0 : 1, 0, 1),
  moveBlock: fleet == 'rebel' ? 'move_fleet_beacon' : null,
);

final mockMap = SectorMap(
  sector: 3,
  beacons: [
    beacon(0, 40, 60, fleet: 'rebel'),
    beacon(1, 200, 150, current: true),
    beacon(
      2,
      330,
      200,
      store: StoreView(
        items: [
          StoreItemView(
            lookId: null,
            id: 'BEAM_1',
            kind: 'weapon',
            name: 'Mini Beam',
            stock: fv('beacons.2.store.0.0', 0, 0, 1),
            itemPath: 'beacons.2.store.0.0.item',
          ),
        ],
        supplies: [
          fv('beacons.2.store.fuel', 3, 0, 99),
          fv('beacons.2.store.missiles', 1, 0, 99),
          fv('beacons.2.store.drone_parts', 0, 0, 99),
        ],
      ),
    ),
  ],
  joins: Uint32List.fromList([1, 2, 0, 1]),
  fleetOffset: -400,
  fleetFudge: 120,
  fleetRadius: 767,
  fleetField: fv('fleet.offset', -400, -959, 60),
  shipField: fv('beacons.current', 1, 0, 2),
  width: 640,
  height: 488,
);

void main() {
  final texts = <String, String>{};
  Future<Map<String, int>> pump(WidgetTester tester, {Size size = const Size(1400, 1000)}) async {
    texts.clear();
    tester.view.physicalSize = size;
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final edits = <String, int>{};
    await tester.pumpWidget(
      variantApp(
        Scaffold(
          body: StatefulBuilder(
            builder: (context, setState) => MapPage(
              map: mockMap,
              edits: edits,
              onChanged: (path, value) => setState(() => edits[path] = value),
              texts: texts,
              onText: (path, text) => setState(() => texts[path] = text),
              storeChoices: (kind) => kind == 'weapon'
                  ? const [(id: 'BEAM_1', name: 'Mini Beam'), (id: 'ION_2', name: 'Heavy Ion')]
                  : const [],
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    return edits;
  }

  testWidgets('a store beacon opens its popover and restocks', (tester) async {
    final edits = await pump(tester);
    expect(find.byKey(const ValueKey('map-beacon-0')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('map-beacon-2')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('beacon-popover-2')), findsOneWidget);
    expect(find.text('Mini Beam'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('store-stock-beacons.2.store.0.0')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('store-supply-plus-beacons.2.store.fuel')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('beacon-unvisit-2')));
    await tester.pumpAndSettle();
    expect(edits, {'beacons.2.store.0.0': 1, 'beacons.2.store.fuel': 4, 'beacons.2.visited': 0});
  });

  testWidgets('the current beacon keeps its visit', (tester) async {
    await pump(tester);
    await tester.tap(find.byKey(const ValueKey('map-beacon-1')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('beacon-unvisit-1')), findsNothing);
  });

  testWidgets('the fleet goes back one jump, then to the sector start', (tester) async {
    final edits = await pump(tester);
    await tester.tap(find.byKey(const ValueKey('fleet-back')));
    await tester.pumpAndSettle();
    expect(edits['fleet.offset'], -400 - fleetJump);
    await tester.tap(find.byKey(const ValueKey('fleet-start')));
    await tester.pumpAndSettle();
    expect(edits['fleet.offset'], -959);
    await tester.tap(find.byKey(const ValueKey('fleet-forward')));
    await tester.pumpAndSettle();
    expect(edits['fleet.offset'], -959 + fleetJump);
  });

  testWidgets('the ship moves to a beacon; a fleet beacon says why it cannot', (tester) async {
    final edits = await pump(tester);
    await tester.tap(find.byKey(const ValueKey('map-beacon-2')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('beacon-move-2')));
    await tester.pumpAndSettle();
    expect(edits['beacons.current'], 2);
    await tester.tap(find.byKey(const ValueKey('beacon-close')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('map-beacon-0')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('beacon-move-0')), findsNothing);
    final l = AppLocalizations.of(tester.element(find.byType(MapPage)))!;
    expect(find.text(l.mapMoveBlockFleet), findsOneWidget);
  });

  testWidgets('a beacon becomes a store and a store item is swapped', (tester) async {
    final edits = await pump(tester);
    await tester.tap(find.byKey(const ValueKey('map-beacon-0')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('beacon-store-0')));
    await tester.pumpAndSettle();
    expect(edits['beacons.0.store'], 1);
    await tester.tap(find.byKey(const ValueKey('beacon-close')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('map-beacon-2')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('store-swap-beacons.2.store.0.0.item')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('store-choice-ION_2')));
    await tester.pumpAndSettle();
    expect(texts, {'beacons.2.store.0.0.item': 'ION_2'});
    expect(find.text('Heavy Ion'), findsOneWidget);
  });

  testWidgets('the map fills the page at 1100 x 700', (tester) async {
    await pump(tester, size: const Size(1100, 700));
    final canvas = tester.getSize(find.byType(SectorCanvas));
    expect(canvas.width, greaterThan(1000));
    expect(canvas.height, greaterThan(440));
    expect(find.byKey(const ValueKey('fleet-forward')), findsOneWidget);
  });

  testWidgets('every map path has a label', (tester) async {
    await pump(tester);
    final l = AppLocalizations.of(tester.element(find.byType(MapPage)))!;
    final labels = {for (final f in mapFields(mockMap)) f.path: mapLook(f.path, l, mockMap)?.$2};
    expect(labels.values, everyElement(isNotNull));
    expect(labels['beacons.2.store.0.0'], 'Beacon 3 store: Mini Beam');
    expect(labels['fleet.offset'], 'Rebel fleet position');
  });
}
