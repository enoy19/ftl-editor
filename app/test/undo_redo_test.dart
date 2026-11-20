// Undo and Redo in the top bar: left of the one-click actions with a gap,
// disabled without a step, Cmd+Z and Shift+Cmd+Z, and on the Profile page
// the steps of the profile.

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_look.dart';
import 'package:ftl_editor/ui/quick_actions.dart';
import 'package:material_ui/material_ui.dart';

import 'app_shell_test.dart' show pumpShell;
import 'ftl_look_shell_test.dart' show pumpFtl;
import 'shell_fixture.dart';

void main() {
  WidgetController.hitTestWarningShouldBeFatal = true;

  Future<void> cmd(WidgetTester tester, {bool shift = false}) async {
    await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    if (shift) await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyZ);
    if (shift) await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await tester.pumpAndSettle();
  }

  testWidgets('Undo and Redo sit left of Max everything with a gap and press their handler', (tester) async {
    final calls = <String>[];
    await pumpFtl(
      tester,
      (_) => mockShell(
        actions: QuickActionBar(onRun: (_) {}),
        onUndo: () => calls.add('undo'),
        onRedo: () => calls.add('redo'),
      ),
    );
    final undo = find.byKey(const ValueKey('undo-button'));
    final redo = find.byKey(const ValueKey('redo-button'));
    final max = find.byKey(const ValueKey('action-max-everything'));
    expect(tester.widget<FtlButton>(undo).onPressed, isNotNull);
    expect(tester.getTopRight(undo).dx, lessThan(tester.getTopLeft(redo).dx));
    final gap = tester.getTopLeft(max).dx - tester.getTopRight(redo).dx;
    expect(
      gap,
      greaterThan(tester.getTopLeft(redo).dx - tester.getTopRight(undo).dx),
      reason: 'a gap before the actions',
    );
    await tester.tap(undo);
    await tester.tap(redo);
    expect(calls, ['undo', 'redo']);
  });

  testWidgets('without a step both are disabled', (tester) async {
    await pumpShell(tester, mockShell());
    for (final key in ['undo-button', 'redo-button']) {
      expect(tester.widget<IconButton>(find.byKey(ValueKey(key))).onPressed, isNull, reason: key);
    }
  });

  testWidgets('Cmd+Z undoes, Shift+Cmd+Z redoes; a text field keeps the keys for its text', (tester) async {
    debugDefaultTargetPlatformOverride = TargetPlatform.macOS;
    addTearDown(() => debugDefaultTargetPlatformOverride = null);
    final calls = <String>[];
    await pumpShell(tester, mockShell(onUndo: () => calls.add('undo'), onRedo: () => calls.add('redo')));
    await cmd(tester);
    await cmd(tester, shift: true);
    expect(calls, ['undo', 'redo']);

    // The Expert search field holds the focus: the keys stay with it.
    await tester.tap(find.byKey(const ValueKey('expert-menu')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Raw dump').last);
    await tester.pumpAndSettle();
    await tester.tap(find.byType(TextField).first);
    await tester.pumpAndSettle();
    await cmd(tester);
    expect(calls, ['undo', 'redo']);
  });

  testWidgets('on the Profile page Undo and Redo walk the profile edits', (tester) async {
    debugDefaultTargetPlatformOverride = TargetPlatform.macOS;
    addTearDown(() => debugDefaultTargetPlatformOverride = null);
    final calls = <String>[];
    await pumpShell(
      tester,
      mockShell(
        onUndo: () => calls.add('save'),
        onUndoProfile: () => calls.add('profile'),
        onRedoProfile: () => calls.add('profile redo'),
      ),
    );
    await tester.tap(find.text('Profile'));
    await tester.pumpAndSettle();
    await cmd(tester);
    await tester.tap(find.byKey(const ValueKey('redo-button')));
    await tester.pumpAndSettle();
    expect(calls, ['profile', 'profile redo']);
  });
}
