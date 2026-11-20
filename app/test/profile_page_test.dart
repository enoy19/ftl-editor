import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/src/rust/api/profile.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/profile_page.dart';
import 'package:material_ui/material_ui.dart';

import 'mock_profile.dart';

void main() {
  WidgetController.hitTestWarningShouldBeFatal = true;

  Future<void> pump(WidgetTester tester, Widget body, {Locale? locale}) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(variantApp(Scaffold(body: body), locale: locale));
    await tester.pumpAndSettle();
  }

  /// The page is a lazy list: scroll until [finder] is built and on screen.
  Future<void> reveal(WidgetTester tester, Finder finder) async {
    await tester.scrollUntilVisible(finder, 200, scrollable: find.byType(Scrollable).first);
    await tester.pumpAndSettle();
  }

  ProfileBody body({
    ProfileSummary? summary,
    ProfileSummary? file,
    VoidCallback? onUnlockAll,
    String? error,
    void Function(String, String, bool)? onLayout,
    void Function(String, int)? onAchievement,
  }) => ProfileBody(
    path: summary?.path,
    busy: false,
    onChoose: () {},
    error: error,
    summary: summary,
    file: file,
    catalog: mockCatalog,
    onUnlockAll: onUnlockAll,
    onLayout: onLayout,
    onAchievement: onAchievement,
  );

  Finder tooltipStarting(String text) =>
      find.byWidgetPredicate((w) => w is Tooltip && (w.message ?? '').startsWith(text));

  testWidgets('the grid shows every ship with its hull and the A, B, C state as icons', (tester) async {
    await pump(tester, body(summary: mockProfile));
    expect(find.byType(ProfileShipTile), findsNWidgets(10));
    final kestrel = find.byKey(const ValueKey('profile-ship-PLAYER_SHIP_HARD'));
    expect(find.descendant(of: kestrel, matching: find.text('The Kestrel')), findsOneWidget);
    expect(
      find.descendant(of: kestrel, matching: find.byIcon(Icons.rocket_launch)),
      findsOneWidget,
      reason: 'hull',
    );
    expect(find.descendant(of: kestrel, matching: find.byTooltip('Type-A unlocked')), findsOneWidget);
    expect(find.descendant(of: kestrel, matching: find.byTooltip('Type-C unlocked')), findsOneWidget);

    final nesasio = find.byKey(const ValueKey('profile-ship-PLAYER_SHIP_STEALTH'));
    expect(find.descendant(of: nesasio, matching: tooltipStarting('Type-B locked')), findsOneWidget);
    final crystal = find.byKey(const ValueKey('profile-ship-PLAYER_SHIP_CRYSTAL'));
    expect(find.descendant(of: crystal, matching: find.byTooltip('This ship has no Type-C')), findsOneWidget);
  });

  testWidgets('achievements are image tiles with their difficulty; stats and scores are compact', (tester) async {
    await pump(tester, body(summary: mockProfile));
    expect(find.text('Achievements (4)'), findsOneWidget);
    final tile = find.byKey(const ValueKey('profile-achievement-ACH_MANTIS_SLAUGHTER'));
    await reveal(tester, tile);
    expect(find.descendant(of: tile, matching: find.text('Hard')), findsOneWidget);
    // The name and description of ftl.dat, never the id.
    expect(find.byTooltip('Mock Brawl\nWin a made-up brawl with the test ship.'), findsOneWidget);
    expect(find.byTooltip('ACH_MANTIS_SLAUGHTER'), findsNothing);

    await reveal(tester, find.text('3120'));
    expect(find.text('3120'), findsOneWidget);
    expect(find.text('Ships defeated'), findsOneWidget);
    expect(find.text('412'), findsOneWidget);
  });

  testWidgets('the unlock button calls back; without a profile it is off', (tester) async {
    var taps = 0;
    await pump(tester, body(summary: mockProfile, onUnlockAll: () => taps++));
    final button = find.widgetWithText(FilledButton, 'Unlock all ships');
    await reveal(tester, button);
    await tester.tap(button);
    expect(taps, 1);

    await pump(tester, body(error: 'No profile at /x/ae_prof.sav.'));
    expect(find.widgetWithText(FilledButton, 'Unlock all ships'), findsNothing);
    expect(find.text('No profile at /x/ae_prof.sav.'), findsOneWidget);
    expect(find.text('No profile open'), findsOneWidget);
  });

  testWidgets('a click on a lock asks to unlock or lock that one layout; Type-B names its rule', (tester) async {
    final calls = <(String, String, bool)>[];
    await pump(tester, body(summary: mockProfile, onLayout: (s, l, u) => calls.add((s, l, u))));
    await tester.tap(find.byKey(const ValueKey('profile-layout-PLAYER_SHIP_STEALTH-b')));
    await tester.tap(find.byKey(const ValueKey('profile-layout-PLAYER_SHIP_HARD-c')));
    await tester.tap(find.byKey(const ValueKey('profile-layout-PLAYER_SHIP_CRYSTAL-c')));
    expect(calls, [
      ('PLAYER_SHIP_STEALTH', 'b', true),
      ('PLAYER_SHIP_HARD', 'c', false),
    ], reason: 'no Type-C on the Crystal');
    final b = find.byKey(const ValueKey('profile-layout-PLAYER_SHIP_STEALTH-b'));
    final tip = tester.widget<Tooltip>(find.descendant(of: b, matching: find.byType(Tooltip)).first).message!;
    expect(tip, contains('two of the ship'));
    expect(tip, contains('Quiet Hunter, Slip Away, ACH_STEALTH_TACTICAL'));
  });

  testWidgets('staged edits show at once with the pending look; removed achievements stay marked', (tester) async {
    await pump(tester, body(summary: mockStaged, file: mockProfile));
    final pending = AppTokens.of(tester.element(find.byType(ProfileBody))).accent;
    bool marked(String key) => find
        .descendant(of: find.byKey(ValueKey(key)), matching: find.byKey(const ValueKey('pending')))
        .evaluate()
        .isNotEmpty;
    expect(marked('profile-layout-PLAYER_SHIP_STEALTH-b'), isTrue);
    expect(marked('profile-layout-PLAYER_SHIP_HARD-c'), isTrue);
    expect(marked('profile-layout-PLAYER_SHIP_HARD-a'), isFalse);
    final lockC = find.descendant(
      of: find.byKey(const ValueKey('profile-layout-PLAYER_SHIP_HARD-c')),
      matching: find.byType(Icon),
    );
    expect(tester.widget<Icon>(lockC).icon, Icons.lock, reason: 'staged lock');
    final removed = find.byKey(const ValueKey('profile-achievement-ACH_MANTIS_SLAUGHTER'));
    await reveal(tester, removed);
    expect(find.descendant(of: removed, matching: find.text('Removed')), findsOneWidget);
    final added = find.byKey(const ValueKey('profile-achievement-ACH_SECTOR_5'));
    expect(find.descendant(of: added, matching: find.text('Hard')), findsOneWidget);
    expect(tester.widget<AppText>(find.descendant(of: added, matching: find.byType(AppText))).color, pending);
  });

  testWidgets('an achievement opens its editor: difficulty and remove', (tester) async {
    final calls = <(String, int)>[];
    await pump(
      tester,
      GameDataScope(
        datPath: '/x/ftl.dat',
        load: (_) async => throw StateError('no image'),
        child: body(summary: mockProfile, onAchievement: (id, v) => calls.add((id, v))),
      ),
    );
    final tile = find.byKey(const ValueKey('profile-achievement-ACH_MANTIS_SLAUGHTER'));
    await reveal(tester, tile);
    await tester.tap(tile);
    await tester.pumpAndSettle();
    final description = find.text('Win a made-up brawl with the test ship.');
    expect(description, findsOneWidget);
    // The dialog route lies above the page: the game data comes along for the image.
    expect(GameDataScope.maybeOf(tester.element(description))?.datPath, '/x/ftl.dat');
    await tester.tap(find.byKey(const ValueKey('achievement-difficulty-normal')));
    await tester.pumpAndSettle();
    await tester.tap(tile);
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('achievement-remove')));
    await tester.pumpAndSettle();
    expect(calls, [('ACH_MANTIS_SLAUGHTER', 2), ('ACH_MANTIS_SLAUGHTER', 0)]);
  });

  testWidgets('Add achievement picks from the names of ftl.dat the profile lacks', (tester) async {
    final calls = <(String, int)>[];
    await pump(tester, body(summary: mockProfile, onAchievement: (id, v) => calls.add((id, v))));
    final add = find.byKey(const ValueKey('profile-add-achievement'));
    await reveal(tester, add);
    await tester.tap(add);
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('achievement-choice-ACH_MANTIS_SLAUGHTER')), findsNothing, reason: 'present');
    await tester.enterText(find.byKey(const ValueKey('achievement-filter')), 'sector');
    await tester.pumpAndSettle();
    expect(find.text('Quiet Hunter'), findsNothing);
    await tester.tap(find.text('First Steps'));
    await tester.pumpAndSettle();
    expect(calls, [('ACH_SECTOR_5', 1)]);
  });

  late Future<bool> answer;
  final undone = <String>[];

  Future<void> openSheet(WidgetTester tester, {Locale? locale}) async {
    undone.clear();
    await pump(
      tester,
      Builder(
        builder: (context) => TextButton(
          onPressed: () => answer = showChangeSheet(
            context,
            changes: profileChangeRows(
              mockEditChanges,
              AppLocalizations.of(context)!,
              ships: mockStaged.ships,
              catalog: mockCatalog,
            ),
            onUndo: undone.add,
          ),
          child: const Text('go'),
        ),
      ),
      locale: locale,
    );
    await tester.tap(find.text('go'));
    await tester.pumpAndSettle();
  }

  testWidgets('the change sheet names layouts and achievements from ftl.dat, each with undo', (tester) async {
    await openSheet(tester);
    expect(find.text('Changes to write'), findsOneWidget);
    final b = find.byKey(const ValueKey('change-ships.PLAYER_SHIP_STEALTH.b'));
    expect(find.descendant(of: b, matching: find.text('The Nesasio: unlock Type-B')), findsOneWidget);
    expect(find.descendant(of: b, matching: find.textContaining('Quiet Hunter, Slip Away')), findsOneWidget);
    expect(find.text('The Kestrel: lock Type-C'), findsOneWidget);
    expect(find.text('Remove Mock Brawl'), findsOneWidget);
    expect(find.text('Add First Steps on Hard'), findsOneWidget);
    expect(find.text('The Torus: unlock Type-A'), findsOneWidget);
    expect(find.textContaining('ACH_'), findsNothing, reason: 'names, never ids');
    await tester.tap(find.byKey(const ValueKey('undo-achievements.ACH_SECTOR_5')));
    await tester.pumpAndSettle();
    expect(undone, ['achievements.ACH_SECTOR_5']);
  });

  testWidgets('Write answers true, Cancel answers false', (tester) async {
    await openSheet(tester);
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await tester.pumpAndSettle();
    expect(await answer, isTrue);

    await openSheet(tester);
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    expect(await answer, isFalse);
  });

  testWidgets('the change sheet speaks German', (tester) async {
    await openSheet(tester, locale: const Locale('de'));
    expect(find.text('The Nesasio: Typ B freischalten'), findsOneWidget);
    expect(find.text('Mock Brawl entfernen'), findsOneWidget);
    expect(find.text('First Steps auf Schwer hinzufügen'), findsOneWidget);
  });

  testWidgets('the backups button opens the history of the profile', (tester) async {
    var opened = 0;
    await pump(
      tester,
      ProfileBody(
        path: mockProfile.path,
        busy: false,
        onChoose: () {},
        error: null,
        summary: mockProfile,
        onUnlockAll: () {},
        onBackups: () => opened++,
      ),
    );
    await tester.tap(find.byKey(const ValueKey('profile-backups')));
    expect(opened, 1);
    await pump(tester, body(summary: mockProfile));
    expect(tester.widget<OutlinedButton>(find.byKey(const ValueKey('profile-backups'))).onPressed, isNull);
  });

  testWidgets('the profile page speaks German', (tester) async {
    await pump(tester, body(summary: mockProfile), locale: const Locale('de'));
    expect(find.text('Spielerschiffe'), findsOneWidget);
    expect(find.text('Alle Schiffe freischalten'), findsOneWidget);
    expect(find.byTooltip('Typ A freigeschaltet'), findsWidgets);
  });
}
