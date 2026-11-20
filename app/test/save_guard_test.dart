import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/shell/status_bar.dart';
import 'package:ftl_editor/src/rust/api/status.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/save_dialogs.dart';
import 'package:material_ui/material_ui.dart';

import 'app_shell_test.dart' show pumpShell;
import 'shell_fixture.dart';

const running = FileWarning(code: 'ftl_running', message: 'FTL is running (pid 4242)');
const steam = FileWarning(code: 'steam_cloud', message: 'Steam Cloud synchronises this folder');
const noList = FileWarning(code: 'process_list_unavailable', message: 'cannot read the process list (denied)');

Widget shellWith(List<FileWarning> warnings) => mockShell(
  status: ShellStatus.withGuard(path: '/saves/continue.sav', guard: SaveGuard.of(warnings)),
);

void main() {
  test('the warning codes map to running FTL, Steam Cloud and unknown', () {
    final none = SaveGuard.of(const []);
    expect((none.ftlRunning, none.steamCloud, none.ftlUnknownReason), (false, false, null));
    final both = SaveGuard.of(const [running, steam]);
    expect((both.ftlRunning, both.steamCloud), (true, true));
    final unknown = SaveGuard.of(const [noList, steam]);
    expect((unknown.ftlRunning, unknown.steamCloud, unknown.ftlUnknownReason), (null, true, noList.message));
  });

  testWidgets('no warning: FTL is closed and Steam Cloud is off', (tester) async {
    await pumpShell(tester, shellWith(const []));
    expect(find.text('FTL is closed'), findsOneWidget);
    expect(find.text('Steam Cloud off'), findsOneWidget);
    expect(find.byIcon(Icons.stop_circle_outlined), findsOneWidget);
    expect(find.byIcon(Icons.cloud_off_outlined), findsOneWidget);
  });

  testWidgets('a running FTL and Steam Cloud show with their icons and hints', (tester) async {
    await pumpShell(tester, shellWith(const [running, steam]));
    expect(find.text('FTL is running'), findsOneWidget);
    expect(find.text('Steam Cloud on'), findsOneWidget);
    expect(find.byIcon(Icons.play_circle), findsOneWidget);
    expect(find.byIcon(Icons.cloud_sync), findsOneWidget);
    expect(find.byTooltip('Close FTL before you save: the game writes this file when it quits.'), findsOneWidget);
  });

  testWidgets('an unreadable process list keeps FTL unknown and says why', (tester) async {
    await pumpShell(tester, shellWith(const [noList]));
    expect(find.text('FTL: unknown'), findsOneWidget);
    expect(find.text('Steam Cloud off'), findsOneWidget);
    expect(
      find.byTooltip(
        'The editor cannot read the process list, so a running FTL is unknown. '
        'Quit FTL before you save. Detail: ${noList.message}',
      ),
      findsOneWidget,
    );
  });

  testWidgets('without a file nothing is checked', (tester) async {
    await pumpShell(tester, mockShell(status: ShellStatus.withGuard(path: null, guard: null)));
    expect(find.text('FTL: unknown'), findsOneWidget);
    expect(find.text('Steam Cloud: unknown'), findsOneWidget);
    expect(find.byTooltip('Open a file to check this.'), findsNWidgets(2));
  });

  group('the dialog for a running FTL', () {
    Future<Future<bool>> open(WidgetTester tester) async {
      late Future<bool> answer;
      await tester.pumpWidget(
        variantApp(
          Builder(
            builder: (context) =>
                TextButton(onPressed: () => answer = confirmFtlRunning(context), child: const Text('save')),
          ),
        ),
      );
      await tester.tap(find.text('save'));
      await tester.pumpAndSettle();
      expect(
        find.text('FTL is running and overwrites this file when it quits. Quit FTL first, then save.'),
        findsOneWidget,
      );
      return answer;
    }

    testWidgets('Enter cancels: Cancel has the focus', (tester) async {
      final answer = await open(tester);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('ftl-running-dialog')), findsNothing);
      expect(await answer, isFalse);
    });

    testWidgets('Save anyway answers true', (tester) async {
      final answer = await open(tester);
      await tester.tap(find.text('Save anyway'));
      await tester.pumpAndSettle();
      expect(await answer, isTrue);
    });
  });
}
