// Every crew sprite comes from one builder (lib/ui/crew_sprite.dart): it
// draws race, sex and tints from the core's look id, so a female human looks
// the same on the floorplan, in the square menu, on her card and in the
// pickers. This test reads the source of the app and fails when another file
// names a crew image kind of the bridge (a per-place GameImageRef.of of a
// crew race drew a female human male on the floorplan, 2026-11-19).

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// The one builder and the generated bindings that define the kinds.
const allowed = ['lib/ui/crew_sprite.dart', 'lib/src/'];

void main() {
  test('only the crew sprite builder names a crew image kind', () {
    final kinds = RegExp(r'GameImageKind\.crew\w*');
    final violations = <String>[];
    for (final f in Directory('lib').listSync(recursive: true).whereType<File>()) {
      final path = f.path.replaceAll(r'\', '/');
      if (!path.endsWith('.dart') || allowed.any(path.startsWith)) continue;
      for (final (i, line) in f.readAsLinesSync().indexed) {
        if (kinds.hasMatch(line)) violations.add('$path:${i + 1}: $line');
      }
    }
    expect(violations, isEmpty, reason: 'draw crew with CrewSprite or crewSprite of lib/ui/crew_sprite.dart');
    // The builder itself still names them, so the guard reads the right files.
    expect(kinds.hasMatch(File('lib/ui/crew_sprite.dart').readAsStringSync()), isTrue);
  });
}
