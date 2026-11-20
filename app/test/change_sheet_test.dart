import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/src/rust/api/status.dart';
import 'package:ftl_editor/ui/backup_history.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:material_ui/material_ui.dart';

import 'fake_look.dart';
import 'mock_summary.dart';

void main() {
  test('only edits that differ from the file are changes, in field order', () {
    final changes = pendingChanges(mockSummary.fields, {'ship.scrap': 999, 'ship.hull': 30, 'ship.fuel': 14});
    expect([for (final c in changes) (c.path, c.old, c.value)], [('ship.hull', 22, 30), ('ship.scrap', 214, 999)]);
  });

  group('the change sheet', () {
    late Map<String, int> edits;
    late Future<bool> answer;

    Future<void> open(WidgetTester tester, {bool steamCloud = false, List<String> notes = const []}) async {
      edits = {'ship.scrap': 999, 'ship.hull': 30};
      await tester.pumpWidget(
        variantApp(
          Builder(
            builder: (context) => TextButton(
              onPressed: () => answer = showChangeSheet(
                context,
                changes: fieldChangeRows(pendingChanges(mockSummary.fields, edits), AppLocalizations.of(context)!),
                onUndo: edits.remove,
                steamCloud: steamCloud,
                notes: notes,
              ),
              child: const Text('save'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('save'));
      await tester.pumpAndSettle();
    }

    testWidgets('lists every change with label, old and new value and icon', (tester) async {
      await open(tester);
      expect(find.text('Changes to write'), findsOneWidget);
      final hull = find.byKey(const ValueKey('change-ship.hull'));
      expect(find.descendant(of: hull, matching: find.text('Hull')), findsOneWidget);
      expect(find.descendant(of: hull, matching: findChange(22, 30)), findsOneWidget);
      expect(find.descendant(of: hull, matching: find.byIcon(Icons.favorite)), findsOneWidget);
      expect(
        find.descendant(of: find.byKey(const ValueKey('change-ship.scrap')), matching: findChange(214, 999)),
        findsOneWidget,
      );
      expect(find.byKey(const ValueKey('change-sheet-steam')), findsNothing);
    });

    testWidgets('undo removes one edit; Write answers true', (tester) async {
      await open(tester);
      await tester.tap(find.byKey(const ValueKey('undo-ship.hull')));
      await tester.pumpAndSettle();
      expect(edits, {'ship.scrap': 999});
      expect(find.byKey(const ValueKey('change-ship.hull')), findsNothing);
      expect(find.byKey(const ValueKey('change-ship.scrap')), findsOneWidget);
      await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
      await tester.pumpAndSettle();
      expect(await answer, isTrue);
    });

    testWidgets('undoing the last change closes the sheet without a write', (tester) async {
      await open(tester);
      await tester.tap(find.byKey(const ValueKey('undo-ship.hull')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('undo-ship.scrap')));
      await tester.pumpAndSettle();
      expect(edits, isEmpty);
      expect(find.byKey(const ValueKey('change-sheet')), findsNothing);
      expect(await answer, isFalse);
    });

    testWidgets('Cancel answers false; Steam Cloud adds its hint', (tester) async {
      await open(tester, steamCloud: true);
      expect(find.byKey(const ValueKey('change-sheet-steam')), findsOneWidget);
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(await answer, isFalse);
      expect(edits, hasLength(2), reason: 'Cancel keeps the edits');
    });

    testWidgets('old and new value stand around an arrow icon, never a U+2192 glyph', (tester) async {
      await open(tester);
      expect(
        find.descendant(of: find.byKey(const ValueKey('change-ship.hull')), matching: find.byIcon(Icons.arrow_forward)),
        findsOneWidget,
      );
      final texts = tester.widgetList<Text>(
        find.descendant(of: find.byKey(const ValueKey('change-sheet')), matching: find.byType(Text)),
      );
      expect(texts.where((t) => (t.data ?? '').contains('\u2192')), isEmpty);
    });

    testWidgets('the notes of an action stand under the changes', (tester) async {
      await open(tester, notes: ['2 hull breaches stay']);
      expect(find.byKey(const ValueKey('change-sheet-note-0')), findsOneWidget);
      expect(find.text('2 hull breaches stay'), findsOneWidget);
    });
  });

  group('the backup history', () {
    final newest = BackupInfo(
      path: '/b/continue_2.sav',
      modifiedMs: DateTime(2026, 11, 18, 9, 5, 7).millisecondsSinceEpoch,
      bytes: 9742,
    );
    final older = BackupInfo(
      path: '/b/continue_1.sav',
      modifiedMs: DateTime(2026, 11, 17, 20, 0).millisecondsSinceEpoch,
      bytes: 2048,
    );

    Future<List<BackupInfo>> pump(
      WidgetTester tester,
      List<BackupInfo> list,
      Future<bool> Function(BackupInfo) restore,
    ) async {
      await tester.pumpWidget(
        variantApp(
          Builder(
            builder: (context) => TextButton(
              onPressed: () =>
                  showBackupHistory(context, fileName: 'continue.sav', load: () async => list, restore: restore),
              child: const Text('history'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('history'));
      await tester.pumpAndSettle();
      return list;
    }

    testWidgets('lists the backups in the order the core answers, with time and size', (tester) async {
      await pump(tester, [newest, older], (_) async => true);
      expect(find.text('Backups of continue.sav'), findsOneWidget);
      final tiles = tester.widgetList<ListTile>(
        find.descendant(of: find.byKey(const ValueKey('backup-history')), matching: find.byType(ListTile)),
      );
      expect([for (final t in tiles) (t.title! as AppText).text], ['2026-11-18 09:05:07', '2026-11-17 20:00:00']);
      expect(find.text('9.5 KB · continue_2.sav'), findsOneWidget);
      expect(find.text('2.0 KB · continue_1.sav'), findsOneWidget);
    });

    testWidgets('no backups says so', (tester) async {
      await pump(tester, [], (_) async => true);
      expect(find.text('No backups of this file yet.'), findsOneWidget);
    });

    testWidgets('restore asks first; Cancel restores nothing, Restore restores and closes', (tester) async {
      final restored = <String>[];
      await pump(tester, [newest, older], (b) async {
        restored.add(b.path);
        return true;
      });
      Finder restoreOf(BackupInfo b) =>
          find.descendant(of: find.byKey(ValueKey('backup-${b.path}')), matching: find.text('Restore'));
      await tester.tap(restoreOf(older));
      await tester.pumpAndSettle();
      expect(find.text('Restore this backup?'), findsOneWidget);
      await tester.tap(
        find.descendant(of: find.byKey(const ValueKey('restore-confirm')), matching: find.text('Cancel')),
      );
      await tester.pumpAndSettle();
      expect(restored, isEmpty);

      await tester.tap(restoreOf(older));
      await tester.pumpAndSettle();
      await tester.tap(
        find.descendant(of: find.byKey(const ValueKey('restore-confirm')), matching: find.text('Restore')),
      );
      await tester.pumpAndSettle();
      expect(restored, [older.path]);
      expect(find.byKey(const ValueKey('backup-history')), findsNothing);
    });
  });

  test('the level of a system a staged install adds counts from 0; other unknown edits stay out', () {
    final changes = pendingChanges(mockSummary.fields, {'systems.battery.level': 2, 'crew.add': 1});
    expect([for (final c in changes) (c.path, c.old, c.value)], [('systems.battery.level', 0, 2)]);
  });
}
