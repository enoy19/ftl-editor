// Every system row comes from one builder, `SystemRow` of
// lib/ui/equipment_page.dart: it assembles level, damage and ion (with the
// ranges of the preview), the icon state, Install or Remove and the marks.
// The floorplan room popover wired its own SystemLevelRow and could not
// break a staged upgrade the Equipment page could (2026-11-19). This
// test reads the source of the app and fails when another file builds a
// SystemLevelRow.

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

const builder = 'lib/ui/equipment_page.dart';

void main() {
  test('only the Equipment page builds a SystemLevelRow; every other place uses SystemRow', () {
    final row = RegExp(r'\bSystemLevelRow\(');
    final violations = <String>[];
    for (final f in Directory('lib').listSync(recursive: true).whereType<File>()) {
      final path = f.path.replaceAll(r'\', '/');
      if (!path.endsWith('.dart') || path == builder) continue;
      for (final (i, line) in f.readAsLinesSync().indexed) {
        if (row.hasMatch(line)) violations.add('$path:${i + 1}: $line');
      }
    }
    expect(violations, isEmpty, reason: 'show a system with SystemRow of $builder');
    expect(row.hasMatch(File(builder).readAsStringSync()), isTrue);
  });
}
