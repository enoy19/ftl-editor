import 'package:flutter/gestures.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/look/ftl_tabs.dart';
import 'package:ftl_editor/shell/status_bar.dart';
import 'package:ftl_editor/ui/crew_page.dart';
import 'package:ftl_editor/ui/editor_page.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/profile_page.dart';
import 'package:material_ui/material_ui.dart';

import 'shell_fixture.dart';

Future<void> pumpShell(WidgetTester tester, Widget shell, {Locale? locale}) async {
  tester.view.physicalSize = const Size(1280, 800);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(variantApp(shell, locale: locale));
  await tester.pumpAndSettle();
}

final saveButton = find.widgetWithIcon(FilledButton, Icons.save);

void main() {
  testWidgets('one top bar holds the five area tabs, Expert and Save', (tester) async {
    await pumpShell(tester, mockShell());
    expect(find.byType(AppBar), findsOneWidget);
    expect(find.byType(FtlTabStrip), findsNothing, reason: 'without an FtlTheme the shell keeps the Material look');
    expect(find.byType(Tab), findsNWidgets(5));
    for (final area in ['Ship', 'Crew', 'Equipment', 'Map', 'Profile']) {
      expect(
        find.descendant(of: find.byType(AppBar), matching: find.text(area)),
        findsOneWidget,
        reason: area,
      );
    }
    expect(find.descendant(of: find.byType(AppBar), matching: find.text('Expert')), findsOneWidget);
    expect(find.descendant(of: find.byType(AppBar), matching: saveButton), findsOneWidget);
    expect(find.byType(ShipCard), findsOneWidget, reason: 'the Ship page is the start page');
  });

  // The crew card of 2026-11-19 (variant B) trades the second row for
  // spacing and full skill names: the first row fits, the page scrolls.
  testWidgets('the first row of crew cards fits at 1280x800; the page scrolls to the eighth', (tester) async {
    await pumpShell(tester, mockShell());
    await tester.tap(find.text('Crew'));
    await tester.pumpAndSettle();
    final statusTop = tester.getTopLeft(find.byType(StatusBar)).dy;
    for (var i = 0; i < 4; i++) {
      final card = tester.getRect(find.byKey(ValueKey('crew-card-$i')));
      expect(card.bottom, lessThanOrEqualTo(statusTop), reason: 'card $i ends above the status bar');
    }
    await tester.scrollUntilVisible(
      find.byKey(const ValueKey('crew-card-7')),
      200,
      scrollable: find.descendant(of: find.byType(CrewPage), matching: find.byType(Scrollable)),
    );
    await tester.ensureVisible(find.byKey(const ValueKey('crew-card-7')));
    await tester.pumpAndSettle();
    expect(tester.getRect(find.byKey(const ValueKey('crew-card-7'))).bottom, lessThanOrEqualTo(statusTop));
  });

  testWidgets('the tabs switch pages; other areas name the step that brings them', (tester) async {
    await pumpShell(tester, mockShell());
    await tester.tap(find.text('Crew'));
    await tester.pumpAndSettle();
    expect(find.byType(ShipCard), findsNothing);
    expect(find.byType(CrewCard), findsNWidgets(8));

    await tester.tap(find.text('Profile'));
    await tester.pumpAndSettle();
    expect(find.byType(ProfileBody), findsOneWidget);
    expect(find.text('ae_prof.sav'), findsOneWidget, reason: 'the page names its own file');
    expect(
      find.descendant(of: find.byKey(const ValueKey('status-bar')), matching: find.text('continue.sav')),
      findsOneWidget,
      reason: 'the status bar keeps the saved game',
    );

    await tester.tap(find.text('Ship'));
    await tester.pumpAndSettle();
    expect(find.byType(ShipCard), findsOneWidget);
  });

  testWidgets('the Expert menu opens and names the chosen item', (tester) async {
    await pumpShell(tester, mockShell());
    expect(find.text('State vars'), findsNothing, reason: 'Expert is collapsed');
    await tester.tap(find.byKey(const ValueKey('expert-menu')));
    await tester.pumpAndSettle();
    for (final item in ['Raw dump', 'State vars', 'Unknown fields']) {
      expect(find.text(item), findsOneWidget, reason: item);
    }
    await tester.tap(find.text('State vars'));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('expert-stateVars')), findsOneWidget);
    expect(find.descendant(of: find.byType(AppBar), matching: find.text('State vars')), findsOneWidget);
    expect(find.text('Expert'), findsNothing, reason: 'the menu shows the chosen item instead');
  });

  testWidgets('Save is disabled without edits and saves with them', (tester) async {
    await pumpShell(tester, mockShell());
    expect(tester.widget<FilledButton>(saveButton).onPressed, isNull);

    var saved = 0;
    await pumpShell(tester, mockShell(edits: {'ship.scrap': 500}, onSave: () => saved++));
    await tester.tap(saveButton);
    expect(saved, 1);
  });

  testWidgets('on the Profile page Save writes the profile, not the saved game', (tester) async {
    var saved = 0;
    var profileSaved = 0;
    await pumpShell(tester, mockShell(edits: {'ship.scrap': 500}, onSave: () => saved++));
    await tester.tap(find.byKey(const ValueKey('area-profile')));
    await tester.pumpAndSettle();
    expect(tester.widget<FilledButton>(saveButton).onPressed, isNull, reason: 'no staged profile edit');

    await pumpShell(
      tester,
      mockShell(edits: {'ship.scrap': 500}, onSave: () => saved++, onSaveProfile: () => profileSaved++),
    );
    await tester.tap(find.byKey(const ValueKey('area-profile')));
    await tester.pumpAndSettle();
    await tester.tap(saveButton);
    expect((saved, profileSaved), (0, 1));
    await tester.tap(find.byKey(const ValueKey('area-ship')));
    await tester.pumpAndSettle();
    await tester.tap(saveButton);
    expect((saved, profileSaved), (1, 1));
  });

  testWidgets('the status bar shows the file, unchecked warnings and no backup', (tester) async {
    await pumpShell(tester, mockShell());
    final bar = find.byKey(const ValueKey('status-bar'));
    expect(find.descendant(of: bar, matching: find.text('continue.sav')), findsOneWidget);
    expect(find.byTooltip('/Users/player/Library/Application Support/FasterThanLight'), findsOneWidget);
    expect(find.text('FTL: unknown'), findsOneWidget);
    expect(find.text('Steam Cloud: unknown'), findsOneWidget);
    expect(find.text('No backup in this session'), findsOneWidget);
    expect(find.descendant(of: bar, matching: find.text('Reduced mode')), findsOneWidget);
  });

  testWidgets('a banner sits above every page', (tester) async {
    await pumpShell(tester, mockShell(banner: const Text('ftl.dat question')));
    expect(find.text('ftl.dat question'), findsOneWidget);
    await tester.tap(find.text('Map'));
    await tester.pumpAndSettle();
    expect(find.text('ftl.dat question'), findsOneWidget);
  });

  testWidgets('the status bar warns of a running FTL and Steam Cloud and names the backup', (tester) async {
    await pumpShell(
      tester,
      mockShell(
        status: ShellStatus(
          path: '/saves/continue.sav',
          ftlRunning: true,
          steamCloud: true,
          lastBackup: LastBackup(path: '/backups/continue-1.sav', time: DateTime(2026, 11, 18, 9, 5)),
          gameData: '/games/FTL/ftl.dat',
        ),
      ),
    );
    expect(find.text('FTL is running'), findsOneWidget);
    expect(find.text('Steam Cloud on'), findsOneWidget);
    expect(find.text('Last backup: 09:05'), findsOneWidget);
    expect(find.byTooltip('/backups/continue-1.sav'), findsOneWidget);
    expect(find.text('Game data: ftl.dat'), findsOneWidget);
    expect(find.byTooltip('/games/FTL/ftl.dat'), findsOneWidget);
  });

  testWidgets('a click on the backup item opens the backup history', (tester) async {
    var opened = 0;
    await pumpShell(tester, mockShell(onBackups: () => opened++));
    expect(find.byTooltip('Show the backups of this file'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('status-backup')));
    expect(opened, 1);
  });

  testWidgets('a click on the file name or the game data item opens its chooser; hover marks them', (tester) async {
    final opened = <String>[];
    await pumpShell(
      tester,
      mockShell(onChooseFile: () => opened.add('file'), onChooseGameData: () => opened.add('dat')),
    );
    final file = find.byKey(const ValueKey('status-file'));
    expect(
      find.byTooltip('/Users/player/Library/Application Support/FasterThanLight\nClick to open another saved game.'),
      findsOneWidget,
    );
    expect(
      find.byTooltip(
        'No ftl.dat is open: the editor shows icons instead of game images.\nClick to choose another ftl.dat.',
      ),
      findsOneWidget,
    );
    await tester.tap(file);
    await tester.tap(find.byKey(const ValueKey('status-game-data')));
    expect(opened, ['file', 'dat']);

    // Under the pointer the item turns the primary color and shows the click cursor.
    Color? textColor() => tester.widget<AppText>(find.descendant(of: file, matching: find.byType(AppText))).color;
    expect(textColor(), isNull);
    final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse, pointer: 1);
    addTearDown(mouse.removePointer);
    await mouse.addPointer(location: Offset.zero);
    await mouse.moveTo(tester.getCenter(file));
    await tester.pumpAndSettle();
    final scheme = Theme.of(tester.element(file)).colorScheme;
    expect(textColor(), scheme.primary);
    expect(RendererBinding.instance.mouseTracker.debugDeviceActiveCursor(1), SystemMouseCursors.click);
  });

  testWidgets('without a chooser the file name is plain text', (tester) async {
    await pumpShell(tester, mockShell());
    expect(
      find.descendant(of: find.byKey(const ValueKey('status-file')), matching: find.byType(InkWell)),
      findsNothing,
    );
  });

  testWidgets('the shell speaks German', (tester) async {
    await pumpShell(tester, mockShell(), locale: const Locale('de'));
    for (final t in ['Schiff', 'Crew', 'Ausrüstung', 'Karte', 'Profil', 'Experte', 'Speichern', 'FTL: unbekannt']) {
      expect(find.text(t), findsWidgets, reason: t);
    }
  });
}
