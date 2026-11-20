// The reload of the open file: the button on the file card and the question
// before staged edits are thrown away.

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/save_dialogs.dart';
import 'package:ftl_editor/ui/ship_page.dart';
import 'package:material_ui/material_ui.dart';

void main() {
  testWidgets('the file card reloads the open file and has no reload without one', (tester) async {
    var reloads = 0;
    await tester.pumpWidget(
      variantApp(
        Scaffold(
          body: Column(
            children: [
              FileCard(path: '/tmp/continue.sav', busy: false, onChoose: () {}, onReload: () => reloads++),
              FileCard(key: const ValueKey('none'), path: null, busy: false, onChoose: () {}),
            ],
          ),
        ),
      ),
    );
    expect(find.byKey(const ValueKey('reload-file')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('reload-file')));
    expect(reloads, 1);
  });

  for (final (label, discard) in [('Cancel keeps the edits', false), ('Discard answers yes', true)]) {
    testWidgets('the discard question: $label', (tester) async {
      Future<bool>? answer;
      await tester.pumpWidget(
        variantApp(
          Scaffold(
            body: Builder(
              builder: (context) =>
                  TextButton(onPressed: () => answer = confirmDiscardEdits(context), child: const Text('reload')),
            ),
          ),
        ),
      );
      await tester.tap(find.text('reload'));
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('discard-edits-dialog')), findsOneWidget);
      await tester.tap(discard ? find.byKey(const ValueKey('discard-edits')) : find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(await answer, discard);
    });
  }
}
