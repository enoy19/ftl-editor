// Undo and Redo: the stack of staged-edit snapshots and its use by the
// profile edits. The editor page's own use runs in integration_test/editor_test.dart.

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/ui/edit_history.dart';
import 'package:ftl_editor/ui/profile_page.dart';

void main() {
  test('undo walks back, redo forward, a new change drops the redo steps', () {
    final h = EditHistory<int>();
    expect((h.canUndo, h.canRedo), (false, false));
    expect(h.undo(0), isNull);
    h.record(0); // 0 -> 1
    h.record(1); // 1 -> 2
    expect(h.undo(2), 1);
    expect(h.undo(1), 0);
    expect(h.canUndo, isFalse);
    expect(h.redo(0), 1);
    expect(h.canRedo, isTrue);
    h.record(1); // 1 -> 5
    expect(h.canRedo, isFalse);
    h.clear();
    expect((h.canUndo, h.canRedo), (false, false));
  });

  test('the profile edits undo and redo each change; Unlock all is one step; clear drops the steps', () {
    final e = ProfileEdits();
    var notified = 0;
    e.addListener(() => notified++);
    e.replace({'ships.A.b': 1});
    e.replace({'ships.A.b': 1, 'achievements.X': 2});
    e.undo('ships.A.b'); // the undo of a change sheet row is a step too
    expect(e.values, {'achievements.X': 2});
    expect(e.undoStep(), isTrue);
    expect(e.values, {'ships.A.b': 1, 'achievements.X': 2});
    expect(e.undoStep(), isTrue);
    expect(e.undoStep(), isTrue);
    expect(e.values, isEmpty);
    expect(e.undoStep(), isFalse);
    expect(e.redoStep(), isTrue);
    expect(e.values, {'ships.A.b': 1});

    e.group(() {
      e.replace({'ships.A.b': 1, 'ships.B.b': 1});
      e.replace({'ships.A.b': 1, 'ships.B.b': 1, 'ships.C.b': 1});
    });
    expect(e.canRedo, isFalse);
    expect(e.undoStep(), isTrue);
    expect(e.values, {'ships.A.b': 1}, reason: 'Unlock all comes back in one step');

    e.clear();
    expect((e.canUndo, e.canRedo, e.isEmpty), (false, false, true));
    expect(notified, greaterThan(0));
  });
}
