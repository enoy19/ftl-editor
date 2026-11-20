import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart' show SingleActivator;

/// The steps behind Undo and Redo: snapshots of the staged edits, each the
/// state before one change. A new change drops the steps Undo walked back;
/// a save or a newly read file clears both stacks ([clear]).
class EditHistory<S> {
  final List<S> _back = [];
  final List<S> _forward = [];

  bool get canUndo => _back.isNotEmpty;
  bool get canRedo => _forward.isNotEmpty;

  /// Remembers [before], the state before a change.
  void record(S before) {
    _back.add(before);
    _forward.clear();
  }

  /// The state before the last change, or `null` without one; [current]
  /// becomes the step Redo returns to.
  S? undo(S current) {
    if (_back.isEmpty) return null;
    _forward.add(current);
    return _back.removeLast();
  }

  /// The state the last Undo left, or `null` without one.
  S? redo(S current) {
    if (_forward.isEmpty) return null;
    _back.add(current);
    return _forward.removeLast();
  }

  void clear() {
    _back.clear();
    _forward.clear();
  }
}

/// Cmd+Z on macOS, Ctrl+Z elsewhere.
SingleActivator undoActivator() => switch (defaultTargetPlatform) {
  TargetPlatform.macOS || TargetPlatform.iOS => const SingleActivator(LogicalKeyboardKey.keyZ, meta: true),
  _ => const SingleActivator(LogicalKeyboardKey.keyZ, control: true),
};

/// Shift+Cmd+Z on macOS, Shift+Ctrl+Z elsewhere.
SingleActivator redoActivator() => switch (defaultTargetPlatform) {
  TargetPlatform.macOS || TargetPlatform.iOS => const SingleActivator(LogicalKeyboardKey.keyZ, meta: true, shift: true),
  _ => const SingleActivator(LogicalKeyboardKey.keyZ, control: true, shift: true),
};
