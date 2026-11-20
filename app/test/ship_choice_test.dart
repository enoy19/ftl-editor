// The enemy ship of a save: the `nearby.` paths map to the same icons and
// labels as the player's, the change sheet names the ship, and the switch
// sits on the Ship, Crew and Equipment pages only when the file has one: on
// Ship below the file and ship card, scrolling with the page; in the FTL look
// as chips with all corners cut.

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/l10n/app_localizations_de.dart';
import 'package:ftl_editor/l10n/app_localizations_en.dart';
import 'package:ftl_editor/look/ftl_tabs.dart';
import 'package:ftl_editor/shell/destination.dart';
import 'package:ftl_editor/ui/command_palette.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/editor_page.dart';
import 'package:ftl_editor/ui/equipment_page.dart';
import 'package:ftl_editor/ui/resource_bar.dart';
import 'package:ftl_editor/ui/ship_choice.dart';
import 'package:ftl_editor/ui/ship_page.dart';
import 'package:material_ui/material_ui.dart';

import 'app_shell_test.dart' show pumpShell;
import 'ftl_look_shell_test.dart' show pumpFtl;
import 'mock_equipment.dart';
import 'mock_floorplan.dart';
import 'shell_fixture.dart';

void main() {
  final AppLocalizations en = AppLocalizationsEn();
  final AppLocalizations de = AppLocalizationsDe();

  test('a nearby path is the same field on the enemy ship', () {
    expect(localPath('nearby.systems.shields.level'), 'systems.shields.level');
    expect(localPath('reactor'), 'reactor');
    expect(shipPrefix('nearby.fires.2.0'), 'nearby.');
    expect(resourceLook('nearby.ship.hull', en), resourceLook('ship.hull', en));
    expect(resourceImage('nearby.ship.scrap'), resourceImage('ship.scrap'));
    expect(equipmentLook('nearby.reactor', en, mockEquipment, const []), (Icons.bolt, en.equipReactor));
    expect(itemLook('nearby.weapons.ION_4.armed', en, null, const [])?.$1, Icons.bolt);
    expect(areaOf('nearby.fires.2.0'), Destination.ship);
    expect(areaOf('nearby.crew.0.health'), Destination.crew);
    expect(areaOf('nearby.systems.shields.level'), Destination.equipment);
    expect(fieldKind('nearby.crew.0.health', en), en.paletteHealth);
  });

  test('the change sheet names an edit of the enemy ship in both languages', () {
    expect(shipNamed((Icons.favorite, 'Hull'), 'nearby.ship.hull', en), (Icons.favorite, 'Enemy ship: Hull'));
    expect(shipNamed((Icons.favorite, 'Hülle'), 'nearby.ship.hull', de), (Icons.favorite, 'Gegnerschiff: Hülle'));
    expect(shipNamed((Icons.favorite, 'Hull'), 'ship.hull', en), (Icons.favorite, 'Hull'));
  });

  testWidgets('the switch stands above Ship, Crew and Equipment only with an enemy ship', (tester) async {
    await pumpShell(tester, mockShell());
    expect(find.byType(ShipSwitch), findsNothing);

    await pumpShell(tester, mockShell(nearby: true));
    expect(find.byType(ShipSwitch), findsOneWidget);
    expect(find.byKey(const ValueKey('ship-nearby')), findsOneWidget);
    for (final area in ['Crew', 'Equipment']) {
      await tester.tap(find.text(area));
      await tester.pumpAndSettle();
      expect(find.byType(ShipSwitch), findsOneWidget, reason: area);
    }
    await tester.tap(find.text('Map'));
    await tester.pumpAndSettle();
    expect(find.byType(ShipSwitch), findsNothing);
  });

  testWidgets('a tap on a segment reports the ship', (tester) async {
    bool? shown;
    await pumpShell(tester, Scaffold(body: ShipSwitch(nearby: false, onChanged: (v) => shown = v)));
    await tester.tap(find.byKey(const ValueKey('ship-nearby')));
    await tester.pumpAndSettle();
    expect(shown, isTrue);
  });

  testWidgets('on the Ship page the switch lies between the file card and the ship card and scrolls with the page', (
    tester,
  ) async {
    await pumpShell(tester, mockShell(nearby: false, floorplan: mockFloorplan));
    final shipSwitch = find.byKey(const ValueKey('ship-switch'));
    expect(tester.getBottomLeft(shipSwitch).dy, lessThan(tester.getTopLeft(find.byType(ShipCard)).dy));
    expect(tester.getTopLeft(shipSwitch).dy, greaterThan(tester.getBottomLeft(find.byType(FileCard)).dy));
    expect(tester.getTopLeft(shipSwitch).dy, lessThan(tester.getTopLeft(find.byType(ResourceCard)).dy));
    final before = tester.getTopLeft(shipSwitch).dy;
    await tester.drag(
      find.descendant(of: find.byType(ShipPage), matching: find.byType(Scrollable)),
      const Offset(0, -80),
    );
    await tester.pumpAndSettle();
    expect(tester.getTopLeft(shipSwitch).dy, lessThan(before), reason: 'not pinned: it scrolls with the content');
  });

  testWidgets('in the FTL look the switch is a row of chips, not a second tab strip', (tester) async {
    await pumpFtl(tester, (_) => mockShell(nearby: true));
    expect(find.byKey(const ValueKey('area-tabs')), findsOneWidget);
    expect(find.byType(FtlTabStrip), findsOneWidget, reason: 'only the area tabs');
    expect(tester.widget(find.byKey(const ValueKey('ship-switch'))), isA<FtlChipRow>());
    expect(tester.widget<FtlChipRow>(find.byKey(const ValueKey('ship-switch'))).open, 1);
  });
}
