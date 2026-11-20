// The sector tree of the Map page over a mock tree: the view switch shows
// it, a sector's popover stages the jump, a blocked sector names its
// reason, and the change sheet names a jump in words. No Rust core.

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations_en.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/map.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/map_page.dart';
import 'package:ftl_editor/ui/sector_tree.dart';
import 'package:material_ui/material_ui.dart';

import 'map_page_test.dart' show mockMap;

TreeSectorView sector(int index, int level, int y, List<int> next, {bool current = false, String? block}) =>
    TreeSectorView(
      index: index,
      kind: index.isEven ? 'civilian' : 'hostile',
      level: level,
      x: level * 66,
      y: y,
      description: 'S$index',
      name: 'Sector name $index',
      shortName: 'Short $index',
      next: Uint32List.fromList(next),
      visited: current,
      current: current,
      reachable: true,
      jumpBlock: block,
    );

final mockTree = SectorTreeView(
  sectors: [
    sector(0, 0, 75, [1, 2], current: true),
    sector(1, 1, 50, [3]),
    sector(2, 1, 100, [3]),
    sector(3, 2, 75, [], block: 'jump_last_stand'),
  ],
  jumpField: const FieldValue(path: sectorJumpPath, value: 0, min: 0, max: 3),
  hiddenSector: false,
);

void main() {
  Future<Map<String, int>> pump(WidgetTester tester) async {
    tester.view.physicalSize = const Size(1400, 1000);
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
              tree: mockTree,
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('map-view-sectors')));
    await tester.pumpAndSettle();
    return edits;
  }

  testWidgets('a sector popover stages the jump', (tester) async {
    final edits = await pump(tester);
    expect(find.byKey(const ValueKey('sector-0')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('sector-2')));
    await tester.pumpAndSettle();
    expect(find.text('Sector name 2'), findsWidgets);
    await tester.tap(find.byKey(const ValueKey('sector-jump-2')));
    await tester.pumpAndSettle();
    expect(edits, {sectorJumpPath: 2});
  });

  testWidgets('the Last Stand names why it refuses the jump', (tester) async {
    final edits = await pump(tester);
    await tester.tap(find.byKey(const ValueKey('sector-3')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('sector-jump-3')), findsNothing);
    expect(find.text(AppLocalizationsEn().sectorJumpLastStand), findsOneWidget);
    expect(edits, isEmpty);
  });

  test('the change sheet names the jump with the sector names', () {
    final l = AppLocalizationsEn();
    final rows = sectorJumpChangeRows(mockTree, {sectorJumpPath: 1}, l);
    expect(rows.single.id, sectorJumpPath);
    expect(rows.single.change, ('Sector 1: Sector name 0', 'Sector 2: Sector name 1'));
    expect(sectorJumpChangeRows(mockTree, const {}, l), isEmpty);
  });
}
