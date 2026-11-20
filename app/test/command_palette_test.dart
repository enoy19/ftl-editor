// Cmd+K (Ctrl+K off macOS) over the real shell and pages with mock data:
// the search finds fields, actions and areas; Enter switches the area and
// hands the entry to its owner.

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/command_palette.dart';
import 'package:ftl_editor/ui/crew_page.dart';
import 'package:ftl_editor/ui/editor_page.dart';
import 'package:ftl_editor/ui/equipment_page.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/quick_actions.dart';
import 'package:material_ui/material_ui.dart';

import 'fake_look.dart';
import 'mock_crew.dart';
import 'mock_equipment.dart';
import 'mock_summary.dart';
import 'shell_fixture.dart';

/// The entries the editor page builds, over the mock data.
List<PaletteEntry> entries(AppLocalizations l) => [
  ...paletteFields(
    [...mockSummary.fields, ...crewFields(fullCrew), ...equipmentFields(mockEquipment)],
    (path, l) => fieldLook(path, l, crew: fullCrew, equipment: mockEquipment),
    l,
  ),
  ...paletteActions(l),
];

/// A shell whose palette records the picked entries; an action shows the
/// change sheet, as the editor page does after it planned the action.
Widget shell(List<PaletteEntry> picked, {bool showSheet = false}) => Builder(
  builder: (context) => mockShell(
    palette: PaletteSource(
      entries: entries,
      onPick: (e) {
        picked.add(e);
        if (showSheet && e.kind == PaletteKind.action) {
          showChangeSheet(
            context,
            changes: const [
              ChangeRow(id: 'ship.scrap', icon: Icons.hardware, title: 'Scrap', change: ('214', '99999')),
            ],
          );
        }
      },
    ),
  ),
);

Future<void> pump(WidgetTester tester, Widget home) async {
  tester.view.physicalSize = const Size(1280, 800);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(variantApp(home));
  await tester.pumpAndSettle();
}

Future<void> openPalette(WidgetTester tester, {bool meta = false}) async {
  final modifier = meta ? LogicalKeyboardKey.metaLeft : LogicalKeyboardKey.controlLeft;
  await tester.sendKeyDownEvent(modifier);
  await tester.sendKeyEvent(LogicalKeyboardKey.keyK);
  await tester.sendKeyUpEvent(modifier);
  await tester.pumpAndSettle();
  expect(find.byKey(const ValueKey('command-palette')), findsOneWidget);
}

Future<void> search(WidgetTester tester, String query) async {
  await tester.enterText(find.byKey(const ValueKey('palette-query')), query);
  await tester.pumpAndSettle();
}

Future<void> enter(WidgetTester tester) async {
  await tester.sendKeyEvent(LogicalKeyboardKey.enter);
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('typing scrap finds the scrap field, Enter hands it over on the Ship page', (tester) async {
    final picked = <PaletteEntry>[];
    await pump(tester, shell(picked));
    await tester.tap(find.text('Crew'));
    await tester.pumpAndSettle();
    await openPalette(tester);
    await search(tester, 'scrap');
    expect(find.byKey(const ValueKey('palette-field-ship.scrap')), findsOneWidget);
    expect(find.byKey(const ValueKey('palette-field-ship.fuel')), findsNothing);
    final row = find.byKey(const ValueKey('palette-field-ship.scrap'));
    expect(find.descendant(of: row, matching: find.text('Resource')), findsOneWidget);
    expect(
      find.descendant(of: row, matching: find.text('ship.scrap')),
      findsNothing,
      reason: 'the path is no text for a player',
    );
    await enter(tester);
    expect(find.byKey(const ValueKey('command-palette')), findsNothing);
    expect(picked.single.id, 'ship.scrap');
    expect(find.byType(ShipCard), findsOneWidget, reason: 'Enter switched to the Ship page');
  });

  testWidgets('crew skills are found by the name of the crew member', (tester) async {
    final picked = <PaletteEntry>[];
    await pump(tester, shell(picked));
    await openPalette(tester);
    await search(tester, 'holly pilot');
    expect(find.byKey(const ValueKey('palette-field-crew.3.skills.pilot')), findsOneWidget);
    expect(find.byKey(const ValueKey('palette-field-crew.4.skills.pilot')), findsNothing);
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('palette-field-crew.3.skills.pilot')),
        matching: find.text('Crew · Skill'),
      ),
      findsOneWidget,
    );
    await enter(tester);
    expect(picked.single.id, 'crew.3.skills.pilot');
    expect(find.byType(CrewCard), findsWidgets, reason: 'Enter switched to the Crew page');
  });

  testWidgets('arrow keys choose among the matches', (tester) async {
    final picked = <PaletteEntry>[];
    await pump(tester, shell(picked));
    await openPalette(tester);
    await search(tester, 'shields');
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await tester.pump();
    await enter(tester);
    expect(picked, hasLength(1));
    expect(
      picked.single.id,
      isNot(entries(lookupAppLocalizations(const Locale('en'))).where((e) => e.matches('shields')).first.id),
    );
  });

  testWidgets('an area entry switches the page and hands nothing over', (tester) async {
    final picked = <PaletteEntry>[];
    await pump(tester, shell(picked));
    await openPalette(tester);
    await search(tester, 'equipment');
    expect(find.byKey(const ValueKey('palette-area-equipment')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('palette-area-equipment')));
    await tester.pumpAndSettle();
    expect(picked, isEmpty);
    expect(find.byType(EquipmentPage), findsOneWidget);
  });

  testWidgets('Enter on an action shows the change sheet', (tester) async {
    final picked = <PaletteEntry>[];
    await pump(tester, shell(picked, showSheet: true));
    await openPalette(tester);
    await search(tester, 'max every');
    expect(find.byKey(const ValueKey('palette-action-max-everything')), findsOneWidget);
    await enter(tester);
    expect(picked.single.kind, PaletteKind.action);
    expect(find.byKey(const ValueKey('change-sheet')), findsOneWidget);
  });

  testWidgets('no match says so; Escape closes', (tester) async {
    await pump(tester, shell([]));
    await openPalette(tester);
    await search(tester, 'zzzz');
    expect(find.text('No match'), findsOneWidget);
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('command-palette')), findsNothing);
  });

  testWidgets('on macOS Cmd+K opens it, Ctrl+K does not', (tester) async {
    debugDefaultTargetPlatformOverride = TargetPlatform.macOS;
    try {
      await pump(tester, shell([]));
      await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyK);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('command-palette')), findsNothing);
      await openPalette(tester, meta: true);
    } finally {
      debugDefaultTargetPlatformOverride = null;
    }
  });

  testWidgets('in the FTL look the palette is an FTL window', (tester) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final look = await loadFakeLook(tester);
    final picked = <PaletteEntry>[];
    await tester.pumpWidget(variantApp(inFtlLook(look, shell(picked))));
    await tester.pumpAndSettle();
    await openPalette(tester);
    expect(
      find.descendant(of: find.byKey(const ValueKey('command-palette')), matching: find.byType(FtlWindow)),
      findsOneWidget,
    );
    await search(tester, 'scrap');
    expect(findLabel('Scrap'), findsWidgets);
    await enter(tester);
    expect(picked.single.id, 'ship.scrap');
  });
}
