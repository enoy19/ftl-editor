// The one lookup from an edit path to its game image, and its use by the
// change sheet, the value popover and the command palette.

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/command_palette.dart';
import 'package:ftl_editor/ui/crew_card.dart';
import 'package:ftl_editor/ui/crew_sprite.dart';
import 'package:ftl_editor/ui/field_image.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/resource_bar.dart';
import 'package:material_ui/material_ui.dart';

import 'game_image_test.dart' show pixelPng;
import 'mock_crew.dart';
import 'mock_equipment.dart';
import 'mock_summary.dart';

void main() {
  test('resources, systems, skills, crew, items and fires map to their game image', () {
    expect(fieldImage('ship.scrap'), FieldImage(resourceImage('ship.scrap')!));
    expect(fieldImage('nearby.ship.fuel'), FieldImage(resourceImage('ship.fuel')!));
    for (final p in ['systems.shields.level', 'nearby.systems.shields.installed']) {
      expect(fieldImage(p), const FieldImage(GameImageRef.of(GameImageKind.systemIcon, 'shields')), reason: p);
    }
    expect(fieldImage('crew.0.skills.engine'), FieldImage(skillIcon('engine')));
    expect(
      fieldImage('crew.3.health', crew: fullCrew),
      FieldImage(crewSprite(fullCrew[3].lookId), tint: false),
      reason: 'the look of Holly (female, her tints)',
    );
    expect(fieldImage('crew.9.health', crew: fullCrew), isNull);
    const laser = FieldImage(GameImageRef.of(GameImageKind.weaponFrame, 'ION_4'), tint: false, quarterTurns: 1);
    expect(fieldImage('weapons.ION_4'), laser);
    expect(fieldImage('weapons.ION_4.armed'), laser);
    expect(fieldImage('cargo.ION_4', choices: mockItemChoices), laser);
    const drone = FieldImage(GameImageRef.of(GameImageKind.drone, 'COMBAT_1'), tint: false);
    expect(fieldImage('nearby.drones.COMBAT_1'), drone);
    expect(fieldImage('cargo.COMBAT_1', choices: mockItemChoices), drone);
    expect(fieldImage('fires.2.0'), fireImage);
    for (final p in ['reactor', 'augments.SCRAP_COLLECTOR', 'breaches.1.1', 'fleet.offset', 'raw.x', 'state.y']) {
      expect(fieldImage(p), isNull, reason: p);
    }
  });

  Widget scoped(Widget child, {String? datPath = '/test/ftl.dat'}) =>
      GameDataScope(datPath: datPath, load: (_) async => pixelPng, child: child);

  testWidgets('FieldIcon: the game image with ftl.dat, the Material icon without', (tester) async {
    const image = FieldImage(GameImageRef.path('img/ui_icons/icon_scrap.png'));
    await tester.pumpWidget(
      MaterialApp(
        home: scoped(const FieldIcon(image: image, fallback: Icons.hardware)),
      ),
    );
    expect(find.byType(GameIcon), findsOneWidget);
    await tester.pumpWidget(
      MaterialApp(
        home: scoped(const FieldIcon(image: image, fallback: Icons.hardware), datPath: null),
      ),
    );
    expect(find.byType(GameIcon), findsNothing);
    expect(find.byIcon(Icons.hardware), findsOneWidget);
  });

  testWidgets('the change sheet, a dialog route above the page, shows the game images of the page', (tester) async {
    await tester.pumpWidget(
      variantApp(
        scoped(
          Builder(
            builder: (context) => TextButton(
              onPressed: () => showChangeSheet(
                context,
                changes: fieldChangeRows(
                  pendingChanges(mockSummary.fields, {'ship.scrap': 999}),
                  AppLocalizations.of(context)!,
                ),
                onUndo: (_) {},
              ),
              child: const Text('save'),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('save'));
    await tester.pumpAndSettle();
    final row = find.byKey(const ValueKey('change-ship.scrap'));
    final image = tester.widget<GameIcon>(find.descendant(of: row, matching: find.byType(GameIcon)));
    expect(image.ref, resourceImage('ship.scrap'));
  });

  testWidgets('the value popover shows the game image of its field in the title', (tester) async {
    const field = FieldValue(path: 'crew.0.skills.pilot', value: 0, min: 0, max: 26);
    await tester.pumpWidget(
      variantApp(
        scoped(
          Builder(
            builder: (context) => TextButton(
              onPressed: () => showResourcePopover(
                context,
                anchor: const Rect.fromLTWH(100, 100, 10, 10),
                field: field,
                current: 0,
                look: (Icons.navigation, 'Pilot'),
              ),
              child: const Text('open'),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();
    final popover = find.byKey(const ValueKey('resource-popover'));
    final image = tester.widget<GameIcon>(find.descendant(of: popover, matching: find.byType(GameIcon)));
    expect(image.ref, skillIcon('pilot'));
  });

  testWidgets('the palette shows the game image of a field', (tester) async {
    final l = await AppLocalizations.delegate.load(const Locale('en'));
    final entries = paletteFields(mockSummary.fields, resourceLook, l);
    expect(entries.first.image, fieldImage(mockSummary.fields.first.path));
    await tester.pumpWidget(
      variantApp(
        scoped(
          Builder(
            builder: (context) =>
                TextButton(onPressed: () => showCommandPalette(context, entries), child: const Text('open')),
          ),
        ),
      ),
    );
    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();
    final row = find.byKey(const ValueKey('palette-field-ship.scrap'));
    expect(find.descendant(of: row, matching: find.byType(GameIcon)), findsOneWidget);
  });
}
