// Flow tests of the Profile page over the real app and the real Rust core
// (macOS):
//
//   flutter test integration_test -d macos
//
// The profile is a copy of the fixture in the test's temp folder, with its
// own backup folder. Preferences live in memory and the search for ftl.dat
// answers "nothing found" or the installed ftl.dat as remembered, so no test
// touches the real FTL folder, the real app preferences or the real editor
// config. The test with ftl.dat prints a skip line without it.

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/main.dart';
import 'package:ftl_editor/shell/app_shell.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/src/rust/api/profile.dart';
import 'package:ftl_editor/src/rust/frb_generated.dart';
import 'package:ftl_editor/ui/editor_page.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:ftl_editor/ui/profile_page.dart';
import 'package:integration_test/integration_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:shared_preferences/shared_preferences.dart';

const fixture = '../crates/ftl-sav/tests/fixtures/profile/ae_prof-1.6.13.sav';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  WidgetController.hitTestWarningShouldBeFatal = true;
  setUpAll(() async => RustLib.init());
  setUp(() => SharedPreferences.setMockInitialValues({}));

  late Directory dir;
  late File profile;
  setUp(() async {
    dir = await Directory.systemTemp.createTemp('ftl-editor-profile-it');
    profile = File('${dir.path}/ae_prof.sav');
    await File(fixture).copy(profile.path);
  });
  tearDown(() async => dir.delete(recursive: true));

  /// The app without ftl.dat, or with [dat] as the remembered ftl.dat
  /// (nothing is written to the editor config).
  Widget app({String? profileFile, String? dat}) => FtlEditorApp(
    home: EditorPage(
      backupDir: '${dir.path}/backups',
      defaultFile: '${dir.path}/missing/continue.sav',
      defaultProfileFile: profileFile ?? profile.path,
      pickFile: (_, _) async => null,
      assetsConfigDir: '${dir.path}/config',
      locateFtlDat: () async => dat == null
          ? const FtlDatStatus(tried: ['/nowhere/ftl.dat'])
          : FtlDatStatus(path: dat, source: DatSource.remembered, tried: const []),
      pickFtlDat: (_) async => null,
      // No running FTL and no Steam Cloud: the real process list of the
      // machine must not decide the flow.
      warningsOf: (_) async => const [],
    ),
  );

  /// The installed ftl.dat, or `null` with a printed skip reason.
  Future<String?> installedDat(String test) async {
    final status = await ftlDatStatus(configDir: '${dir.path}/probe-config');
    // ignore: avoid_print
    if (status.path == null) print('skip "$test": no ftl.dat installed; searched ${status.tried}');
    return status.path;
  }

  /// Pumps real time until [done] holds; a Rust future is no frame.
  Future<void> pumpUntil(WidgetTester tester, bool Function() done, String what) async {
    for (var i = 0; i < 200 && !done(); i++) {
      await Future<void>.delayed(const Duration(milliseconds: 50));
      await tester.pump();
    }
    expect(done(), isTrue, reason: 'waited 10 s for $what');
  }

  Future<void> tapVisible(WidgetTester tester, Finder finder) async {
    await tester.ensureVisible(finder);
    await tester.pumpAndSettle();
    await tester.tap(finder);
    await tester.pumpAndSettle();
  }

  /// The page is a lazy list: scroll until [finder] is built, then tap it.
  Future<void> tapInPage(WidgetTester tester, Finder finder) async {
    await tester.scrollUntilVisible(
      finder,
      200,
      scrollable: find.descendant(of: find.byType(ProfileBody), matching: find.byType(Scrollable)).first,
    );
    await tapVisible(tester, finder);
  }

  Future<void> openProfileTab(WidgetTester tester, {String? dat}) async {
    await tester.pumpWidget(app(dat: dat));
    await tester.pumpAndSettle();
    if (dat != null) {
      await pumpUntil(
        tester,
        () => GameDataScope.maybeOf(tester.element(find.byType(AppShell)))?.datPath != null,
        'ftl.dat',
      );
      await tester.pumpAndSettle();
    }
    // The tab bar scrolls: in a narrow test window the Profile tab can sit outside it.
    await tapVisible(tester, find.byKey(const ValueKey('area-profile')));
    await pumpUntil(tester, () => find.byType(ProfileShipTile).evaluate().isNotEmpty, 'the profile');
  }

  /// Whether the lock of [layout] of [ship] shows as locked.
  bool locked(WidgetTester tester, String ship, String layout) =>
      tester
          .widget<Icon>(
            find.descendant(of: find.byKey(ValueKey('profile-layout-$ship-$layout')), matching: find.byType(Icon)),
          )
          .icon ==
      Icons.lock;

  bool pending(String key) => find
      .descendant(of: find.byKey(ValueKey(key)), matching: find.byKey(const ValueKey('pending')))
      .evaluate()
      .isNotEmpty;

  final unlockButton = find.byKey(const ValueKey('profile-unlock-all'));
  final saveButton = find.byKey(const ValueKey('save-button'));
  final sheet = find.byKey(const ValueKey('change-sheet'));

  Future<ProfileSummary> reread() async => (await openProfile(path: profile.path)).summary(language: 'en');

  testWidgets('unlock all stages the layouts; Save shows them, writes with a backup and reads back', (tester) async {
    await openProfileTab(tester);
    expect(locked(tester, 'PLAYER_SHIP_STEALTH', 'b'), isTrue, reason: 'the fixture has locked layouts');
    // The status bar keeps the saved game; the page names its own file.
    expect(find.text('continue.sav'), findsOneWidget);
    expect(find.text('ae_prof.sav'), findsOneWidget);

    await tapVisible(tester, unlockButton);
    // Staged, not written: the locks open with the pending look, no sheet yet.
    expect(sheet, findsNothing);
    expect(locked(tester, 'PLAYER_SHIP_STEALTH', 'b'), isFalse);
    expect(pending('profile-layout-PLAYER_SHIP_STEALTH-b'), isTrue);
    expect(await profile.readAsBytes(), await File(fixture).readAsBytes());

    await tapVisible(tester, saveButton);
    expect(sheet, findsOneWidget);
    expect(find.textContaining('Adds on Easy: '), findsWidgets, reason: 'Type-B adds achievements');
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await pumpUntil(tester, () => !pending('profile-layout-PLAYER_SHIP_STEALTH-b'), 'the save');
    await tester.pumpAndSettle();

    for (final s in (await reread()).ships) {
      expect(s.layoutA && s.layoutB && s.layoutC == s.hasLayoutC, isTrue, reason: s.id);
    }
    final backups = Directory('${dir.path}/backups').listSync();
    expect(backups, hasLength(1));
    expect(await File(backups.single.path).readAsBytes(), await File(fixture).readAsBytes());

    // A second unlock has nothing to stage.
    await tapVisible(tester, unlockButton);
    expect(find.text('Every layout of every ship is already unlocked.'), findsOneWidget);
    expect(tester.widget<FilledButton>(saveButton).onPressed, isNull);

    // The backup history restores the profile before the unlock.
    final backupsButton = find.byKey(const ValueKey('profile-backups'));
    await tester.scrollUntilVisible(
      backupsButton,
      -300,
      scrollable: find.descendant(of: find.byType(ProfileBody), matching: find.byType(Scrollable)).first,
    );
    await tester.pumpAndSettle();
    await tester.tap(backupsButton);
    await pumpUntil(tester, () => find.text('Restore').evaluate().isNotEmpty, 'the backup history');
    expect(find.text('Backups of ae_prof.sav'), findsOneWidget);
    await tester.tap(find.text('Restore'));
    await tester.pumpAndSettle();
    await tester.tap(
      find.descendant(of: find.byKey(const ValueKey('restore-confirm')), matching: find.text('Restore')),
    );
    await pumpUntil(tester, () => locked(tester, 'PLAYER_SHIP_STEALTH', 'b'), 'the restore');
    expect(await profile.readAsBytes(), await File(fixture).readAsBytes());
    expect(Directory('${dir.path}/backups').listSync(), hasLength(2), reason: 'the unlocked file is backed up first');
  });

  testWidgets('one layout unlocks on its own, another locks; Save writes only those, reopen shows them', (
    tester,
  ) async {
    await openProfileTab(tester);
    await tapVisible(tester, find.byKey(const ValueKey('profile-layout-PLAYER_SHIP_CIRCLE-c')));
    await tapVisible(tester, find.byKey(const ValueKey('profile-layout-PLAYER_SHIP_HARD-a')));
    expect(locked(tester, 'PLAYER_SHIP_CIRCLE', 'c'), isFalse);
    expect(locked(tester, 'PLAYER_SHIP_HARD', 'a'), isTrue);
    // A second click on the same lock takes the edit back.
    await tapVisible(tester, find.byKey(const ValueKey('profile-layout-PLAYER_SHIP_HARD-a')));
    expect(pending('profile-layout-PLAYER_SHIP_HARD-a'), isFalse);

    // The staged edit outlives a visit of another tab.
    await tapVisible(tester, find.byKey(const ValueKey('area-ship')));
    await tapVisible(tester, find.byKey(const ValueKey('area-profile')));
    await pumpUntil(tester, () => find.byType(ProfileShipTile).evaluate().isNotEmpty, 'the profile again');
    expect(pending('profile-layout-PLAYER_SHIP_CIRCLE-c'), isTrue);

    await tapVisible(tester, saveButton);
    expect(find.byKey(const ValueKey('change-ships.PLAYER_SHIP_CIRCLE.c')), findsOneWidget);
    expect(find.byKey(const ValueKey('change-ships.PLAYER_SHIP_HARD.a')), findsNothing);
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await pumpUntil(tester, () => !pending('profile-layout-PLAYER_SHIP_CIRCLE-c'), 'the save');

    final again = await reread();
    final circle = again.ships.firstWhere((s) => s.id == 'PLAYER_SHIP_CIRCLE');
    expect((circle.layoutA, circle.layoutB, circle.layoutC), (true, true, true));
    expect(again.ships.where((s) => s.layoutC).map((s) => s.id), ['PLAYER_SHIP_CIRCLE']);
    expect(again.achievements.length, 10, reason: 'Type-C adds no achievement');
  });

  testWidgets('cancel keeps the file and the staged edits', (tester) async {
    await openProfileTab(tester);
    await tapVisible(tester, unlockButton);
    await tapVisible(tester, saveButton);
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    expect(await profile.readAsBytes(), await File(fixture).readAsBytes());
    expect(Directory('${dir.path}/backups').existsSync(), isFalse);
    expect(pending('profile-layout-PLAYER_SHIP_STEALTH-b'), isTrue, reason: 'like the edits of a saved game');
  });

  testWidgets('an achievement is added by its ftl.dat name, set to Hard and saved', (tester) async {
    final dat = await installedDat('add achievement');
    if (dat == null) return;
    await openProfileTab(tester, dat: dat);
    await tapInPage(tester, find.byKey(const ValueKey('profile-add-achievement')));
    await tester.enterText(find.byKey(const ValueKey('achievement-filter')), 'Pacifism');
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('achievement-choice-ACH_PACIFIST')));
    await tester.pumpAndSettle();
    await tapInPage(tester, find.byKey(const ValueKey('profile-achievement-ACH_PACIFIST')));
    await tester.tap(find.byKey(const ValueKey('achievement-difficulty-hard')));
    await tester.pumpAndSettle();
    // One edit for the achievement, its name in the sheet, never the id.
    await tapVisible(tester, saveButton);
    final row = find.byKey(const ValueKey('change-achievements.ACH_PACIFIST'));
    expect(row, findsOneWidget);
    expect(find.descendant(of: row, matching: find.textContaining('ACH_')), findsNothing);
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await pumpUntil(tester, () => sheet.evaluate().isEmpty, 'the save');
    await pumpUntil(tester, () => Directory('${dir.path}/backups').existsSync(), 'the backup');

    final added = (await reread()).achievements.where((a) => a.id == 'ACH_PACIFIST');
    expect(added.map((a) => a.difficulty), ['hard']);
  });

  testWidgets('a missing profile shows the localized text, not the Rust message', (tester) async {
    await tester.pumpWidget(app(profileFile: '${dir.path}/none/ae_prof.sav'));
    await tester.pumpAndSettle();
    await tapVisible(tester, find.byKey(const ValueKey('area-profile')));
    final text = 'No profile at ${dir.path}/none/ae_prof.sav';
    await pumpUntil(tester, () => find.textContaining(text).evaluate().isNotEmpty, 'the error');
    expect(unlockButton, findsNothing);
  });
}
