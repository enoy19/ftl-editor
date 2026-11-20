// The action buttons of the top bar in both looks. The Rust core plans the
// patch; here the shell only has to call the runner with the action id.

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_look.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/quick_actions.dart';
import 'package:material_ui/material_ui.dart';

import 'fake_look.dart';
import 'shell_fixture.dart';

const ids = ['max-everything', 'crew-fully-trained', 'repair-ship', 'heal-crew'];

Future<void> pump(WidgetTester tester, Widget shell) async {
  tester.view.physicalSize = const Size(1280, 800);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(variantApp(shell));
  await tester.pumpAndSettle();
}

void main() {
  test('the buttons follow the order of the core actions', () {
    expect(quickActions.map((a) => a.id), ids);
  });

  testWidgets('each action button in the Material top bar runs its action', (tester) async {
    final ran = <String>[];
    await pump(tester, mockShell(actions: QuickActionBar(onRun: ran.add)));
    for (final id in ids) {
      final button = find.byKey(ValueKey('action-$id'));
      expect(
        find.descendant(of: find.byType(AppBar), matching: button),
        findsOneWidget,
        reason: id,
      );
      await tester.tap(button);
      await tester.pumpAndSettle();
    }
    expect(ran, ids);
    expect(find.byTooltip('Max everything'), findsOneWidget);
    expect(find.byTooltip('Heal crew'), findsOneWidget);
  });

  testWidgets('without an open file the buttons are disabled', (tester) async {
    await pump(tester, mockShell(actions: const QuickActionBar(onRun: null)));
    final button = tester.widget<IconButton>(find.byKey(const ValueKey('action-repair-ship')));
    expect(button.onPressed, isNull);
  });

  testWidgets('in the FTL look the actions are FTL icon buttons left of Expert', (tester) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final look = await loadFakeLook(tester);
    final ran = <String>[];
    await tester.pumpWidget(variantApp(inFtlLook(look, mockShell(actions: QuickActionBar(onRun: ran.add)))));
    await tester.pumpAndSettle();
    for (final id in ids) {
      final button = tester.widget<FtlButton>(find.byKey(ValueKey('action-$id')));
      expect(button.icon, isNotNull, reason: id);
      expect(button.label, isNull, reason: 'an icon button, the name is its tooltip');
    }
    final expert = tester.getTopLeft(find.byKey(const ValueKey('expert-menu')));
    expect(tester.getTopRight(find.byKey(const ValueKey('action-heal-crew'))).dx, lessThan(expert.dx));
    await tester.tap(find.byKey(const ValueKey('action-crew-fully-trained')));
    expect(ran, ['crew-fully-trained']);
  });
}
