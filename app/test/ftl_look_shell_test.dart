// The real shell, pages and dialogs in the FTL look, over a look the test
// builds (fake_look.dart). The Material look of the same widgets is covered
// by app_shell_test, crew_page_test, change_sheet_test and save_guard_test.

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/look/ftl_look.dart';
import 'package:ftl_editor/look/ftl_tabs.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/shell/status_bar.dart';
import 'package:ftl_editor/src/rust/api/status.dart';
import 'package:ftl_editor/ui/backup_history.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/crew_page.dart';
import 'package:ftl_editor/ui/editor_page.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/profile_page.dart';
import 'package:ftl_editor/ui/save_dialogs.dart';
import 'package:material_ui/material_ui.dart';

import 'fake_look.dart';
import 'mock_summary.dart';
import 'shell_fixture.dart';

Future<FtlLook> pumpFtl(WidgetTester tester, Widget Function(FtlLook look) build, {Locale? locale}) async {
  tester.view.physicalSize = const Size(1280, 800);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final look = await loadFakeLook(tester);
  await tester.pumpWidget(variantApp(inFtlLook(look, build(look)), locale: locale));
  await tester.pumpAndSettle();
  return look;
}

FtlButton button(WidgetTester tester, String key) => tester.widget<FtlButton>(find.byKey(ValueKey(key)));

void main() {
  testWidgets('the top bar is tab row C with Expert and Save; the page sits in an FTL window', (tester) async {
    await pumpFtl(tester, (_) => mockShell());
    expect(find.byType(AppBar), findsNothing);
    final strip = tester.widget<FtlTabStrip>(find.byType(FtlTabStrip));
    expect([for (final t in strip.tabs) t.label], ['Ship', 'Crew', 'Equipment', 'Map', 'Profile']);
    expect(strip.open, 0);
    expect(findLabel('EXPERT'), findsOneWidget);
    expect(findLabel('SAVE'), findsOneWidget);
    expect(find.byKey(const ValueKey('ftl-page-window')), findsOneWidget);
    expect(
      find.descendant(of: find.byKey(const ValueKey('ftl-page-window')), matching: find.byType(ShipCard)),
      findsOne,
    );
    expect(findLabel('The Kestrel'), findsOneWidget, reason: 'the ship name in the FTL font');
    expect(find.byKey(const ValueKey('resource-ship.scrap')), findsOneWidget);
    // The status bar keeps its items, in the FTL font.
    final bar = find.byKey(const ValueKey('status-bar'));
    expect(find.descendant(of: bar, matching: findLabel('FTL: unknown')), findsOneWidget);
    expect(find.descendant(of: bar, matching: find.byType(FtlText)), findsWidgets);
  });

  testWidgets('the tabs switch pages; the first row of crew cards fits above the status bar', (tester) async {
    await pumpFtl(tester, (_) => mockShell());
    await tester.tap(find.byKey(const ValueKey('area-crew')));
    await tester.pumpAndSettle();
    expect(tester.widget<FtlTabStrip>(find.byType(FtlTabStrip)).open, 1);
    expect(find.byType(CrewCard), findsNWidgets(8));
    final statusTop = tester.getTopLeft(find.byType(StatusBar)).dy;
    for (var i = 0; i < 3; i++) {
      final card = find.byKey(ValueKey('crew-card-$i'));
      expect(
        find.ancestor(of: card, matching: find.byType(FtlWindow)),
        findsWidgets,
        reason: 'card $i is an FTL window',
      );
      expect(tester.getRect(card).bottom, lessThanOrEqualTo(statusTop), reason: 'card $i ends above the status bar');
    }
    await tester.tap(find.byKey(const ValueKey('area-profile')));
    await tester.pumpAndSettle();
    expect(find.byType(ProfileBody), findsOneWidget);
    expect(findLabel('ae_prof.sav'), findsOneWidget);
    expect(find.descendant(of: find.byType(ProfileShipTile), matching: find.byType(FtlWindow)), findsWidgets);
  });

  testWidgets('Save is an FTL button: disabled without edits, it saves with them', (tester) async {
    await pumpFtl(tester, (_) => mockShell());
    expect(button(tester, 'save-button').onPressed, isNull);

    var saved = 0;
    await pumpFtl(tester, (_) => mockShell(edits: {'ship.scrap': 500}, onSave: () => saved++));
    await tester.tap(find.byKey(const ValueKey('save-button')));
    expect(saved, 1);
  });

  testWidgets('a button shows the face of its state: idle, hover, pressed, disabled', (tester) async {
    late FtlLook look;
    await pumpFtl(tester, (l) {
      look = l;
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            FtlButton(key: const ValueKey('on'), font: l.font(FtlFontId.justin12Bold), label: 'ON', onPressed: () {}),
            FtlButton(key: const ValueKey('off'), font: l.font(FtlFontId.justin12Bold), label: 'OFF'),
          ],
        ),
      );
    });
    expect(look.fonts, isNotEmpty);
    Color face(String key) =>
        (tester
                        .widget<CustomPaint>(
                          find.descendant(of: find.byKey(ValueKey(key)), matching: find.byType(CustomPaint)).first,
                        )
                        .painter!
                    as dynamic)
                .color
            as Color;
    expect(face('on'), FtlColors.line);
    expect(face('off'), FtlColors.off);

    final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
    addTearDown(mouse.removePointer);
    await mouse.addPointer(location: Offset.zero);
    await mouse.moveTo(tester.getCenter(find.byKey(const ValueKey('on'))));
    await tester.pump();
    expect(face('on'), FtlColors.hover);
    await mouse.down(tester.getCenter(find.byKey(const ValueKey('on'))));
    await tester.pump();
    expect(face('on'), FtlColors.pressed);
    await mouse.up();
    await tester.pump();
    expect(face('on'), FtlColors.hover);
  });

  testWidgets('the Expert menu opens and names the chosen item', (tester) async {
    await pumpFtl(tester, (_) => mockShell());
    await tester.tap(find.byKey(const ValueKey('expert-menu')));
    await tester.pumpAndSettle();
    await tester.tap(findLabel('State vars'));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('expert-stateVars')), findsOneWidget);
    expect(findLabel('STATE VARS'), findsOneWidget, reason: 'the menu shows the chosen item');
    expect(tester.widget<FtlTabStrip>(find.byType(FtlTabStrip)).open, -1, reason: 'no tab is open');
  });

  testWidgets('a resource opens its popover in the FTL look', (tester) async {
    await pumpFtl(tester, (_) => mockShell());
    await tester.tap(find.byKey(const ValueKey('resource-ship.fuel')));
    await tester.pumpAndSettle();
    final popover = find.byKey(const ValueKey('resource-popover'));
    expect(find.descendant(of: popover, matching: find.byType(FtlWindow)), findsOneWidget);
    expect(find.descendant(of: popover, matching: findLabel('APPLY')), findsOneWidget);
  });

  testWidgets('the file name and the game data item turn yellow under the pointer and open their chooser', (
    tester,
  ) async {
    final opened = <String>[];
    await pumpFtl(
      tester,
      (_) => mockShell(
        onChooseFile: () => opened.add('file'),
        onChooseGameData: () => opened.add('dat'),
        status: const ShellStatus(path: '/saves/continue.sav', gameData: '/games/FTL/ftl.dat'),
      ),
    );
    final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse, pointer: 1);
    addTearDown(mouse.removePointer);
    await mouse.addPointer(location: Offset.zero);
    for (final key in ['status-file', 'status-game-data']) {
      final item = find.byKey(ValueKey(key));
      final text = find.descendant(of: item, matching: find.byType(AppText));
      expect(tester.widget<AppText>(text).color, isNull, reason: key);
      await mouse.moveTo(tester.getCenter(item));
      await tester.pumpAndSettle();
      expect(tester.widget<AppText>(text).color, FtlColors.hover, reason: key);
      await tester.tap(item);
    }
    expect(opened, ['file', 'dat']);
  });

  testWidgets('the shell speaks German in the FTL look', (tester) async {
    await pumpFtl(tester, (_) => mockShell(), locale: const Locale('de'));
    final strip = tester.widget<FtlTabStrip>(find.byType(FtlTabStrip));
    expect([for (final t in strip.tabs) t.label], ['Schiff', 'Crew', 'Ausrüstung', 'Karte', 'Profil']);
    expect(findLabel('SPEICHERN'), findsOneWidget);
    expect(findLabel('FTL: unbekannt'), findsOneWidget);
  });

  group('dialogs in the FTL look', () {
    Future<void> openWith(WidgetTester tester, void Function(BuildContext context) open) async {
      await pumpFtl(
        tester,
        (_) => Scaffold(
          body: Builder(
            builder: (context) => TextButton(onPressed: () => open(context), child: const Text('open')),
          ),
        ),
      );
      await tester.tap(find.text('open'));
      await tester.pumpAndSettle();
    }

    testWidgets('a running FTL: Enter cancels, Save anyway answers true', (tester) async {
      late Future<bool> answer;
      await openWith(tester, (context) => answer = confirmFtlRunning(context));
      final dialog = find.byKey(const ValueKey('ftl-running-dialog'));
      expect(find.descendant(of: dialog, matching: find.byType(FtlWindow)), findsOneWidget);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      expect(dialog, findsNothing);
      expect(await answer, isFalse, reason: 'Cancel has the focus');

      await tester.tap(find.text('open'));
      await tester.pumpAndSettle();
      await tester.tap(findLabel('SAVE ANYWAY'));
      await tester.pumpAndSettle();
      expect(await answer, isTrue);
    });

    testWidgets('the change sheet: an FTL window, undo per row, Write answers true', (tester) async {
      final edits = {'ship.scrap': 999, 'ship.hull': 30};
      late Future<bool> answer;
      await openWith(
        tester,
        (context) => answer = showChangeSheet(
          context,
          changes: fieldChangeRows(pendingChanges(mockSummary.fields, edits), AppLocalizations.of(context)!),
          onUndo: edits.remove,
          steamCloud: true,
        ),
      );
      final sheet = find.byKey(const ValueKey('change-sheet'));
      expect(find.descendant(of: sheet, matching: find.byType(FtlWindow)), findsOneWidget);
      final hull = find.byKey(const ValueKey('change-ship.hull'));
      expect(find.descendant(of: hull, matching: findLabel('Hull')), findsOneWidget);
      expect(find.descendant(of: hull, matching: findChange(22, 30)), findsOneWidget);
      expect(find.byKey(const ValueKey('change-sheet-steam')), findsOneWidget);
      await tester.tap(find.byKey(const ValueKey('undo-ship.hull')));
      await tester.pumpAndSettle();
      expect(edits, {'ship.scrap': 999});
      expect(hull, findsNothing);
      await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
      await tester.pumpAndSettle();
      expect(await answer, isTrue);
    });

    testWidgets('the backup history: restore asks first in an FTL dialog', (tester) async {
      final backup = BackupInfo(path: '/b/continue_1.sav', bytes: 2048, modifiedMs: 0);
      final restored = <String>[];
      await openWith(
        tester,
        (context) => showBackupHistory(
          context,
          fileName: 'continue.sav',
          load: () async => [backup],
          restore: (b) async {
            restored.add(b.path);
            return true;
          },
        ),
      );
      expect(
        find.descendant(of: find.byKey(const ValueKey('backup-history')), matching: find.byType(FtlWindow)),
        findsOneWidget,
      );
      expect(findLabel(backupTime(backup)), findsOneWidget);
      await tester.tap(findLabel('RESTORE'));
      await tester.pumpAndSettle();
      expect(
        find.descendant(of: find.byKey(const ValueKey('restore-confirm')), matching: find.byType(FtlWindow)),
        findsOneWidget,
      );
      await tester.tap(
        find.descendant(of: find.byKey(const ValueKey('restore-confirm')), matching: findLabel('RESTORE')),
      );
      await tester.pumpAndSettle();
      expect(restored, ['/b/continue_1.sav']);
      expect(find.byKey(const ValueKey('backup-history')), findsNothing);
    });
  });
}
