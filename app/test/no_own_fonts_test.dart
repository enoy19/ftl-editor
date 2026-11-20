// The pages build from the components (lib/ui/components) and never name a
// font or their own padding: the components read both from the AppTokens of
// the look. This test reads the source of every page and fails on a widget
// that sets a font family, a font size, a text style or a bare Text, and on
// EdgeInsets with a number that is not from the Space scale. The components,
// the look primitives (lib/look), the generated code and the localizations
// are where fonts are allowed.

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

/// Folders below lib/ whose code may name fonts and sizes.
const allowed = ['lib/ui/components/', 'lib/look/', 'lib/src/', 'lib/l10n/'];

/// What a page must not write, with the reason the failure prints.
final forbidden = <(RegExp, String)>[
  (RegExp(r'\bfontSize\b'), 'a font size: use AppText with a TextRole'),
  (RegExp(r'\bfontFamily\b'), 'a font family: use AppText with a TextRole'),
  (RegExp(r'\bfontWeight\b'), 'a font weight: use AppText with a TextRole'),
  (RegExp(r'\bTextStyle\('), 'an own TextStyle: use AppText or AppTokens.style'),
  (RegExp(r'\btextTheme\b'), 'a style of the text theme: use AppText with a TextRole'),
  (RegExp(r'\bFtlFontId\b'), 'a bitmap font: use AppText with a TextRole'),
  (RegExp(r'\bFtlText\('), 'bitmap text: use AppText'),
  (RegExp(r'\bFtlButton\('), 'an FTL button: use AppButton'),
  (RegExp(r'(?<![A-Za-z_.])(Selectable)?Text\('), 'a bare Text: use AppText'),
  (RegExp(r'(?<![A-Za-z_.])TextField\('), 'a bare TextField: use AppTextField (monospace typing)'),
];

/// The argument text of each `EdgeInsets.<ctor>(...)` call in [source].
Iterable<(int, String)> edgeInsets(String source) sync* {
  for (final m in RegExp(r'EdgeInsets\.\w+\(').allMatches(source)) {
    var depth = 1;
    var i = m.end;
    while (i < source.length && depth > 0) {
      if (source[i] == '(') depth++;
      if (source[i] == ')') depth--;
      i++;
    }
    yield (m.start, source.substring(m.end, i - 1));
  }
}

void main() {
  test('no page sets its own font, text style or padding', () {
    final violations = <String>[];
    final files = Directory('lib')
        .listSync(recursive: true)
        .whereType<File>()
        .where((f) => f.path.endsWith('.dart'))
        .where((f) => !allowed.any((a) => f.path.replaceAll(r'\', '/').startsWith(a)));
    for (final file in files) {
      final source = file.readAsStringSync();
      final lines = source.split('\n');
      int lineOf(int offset) => '\n'.allMatches(source.substring(0, offset)).length + 1;
      for (final (i, line) in lines.indexed) {
        final code = line.split('//').first;
        for (final (pattern, why) in forbidden) {
          if (pattern.hasMatch(code)) violations.add('${file.path}:${i + 1}: $why\n    ${line.trim()}');
        }
      }
      for (final (offset, args) in edgeInsets(source)) {
        // Only Space.<step> and 0: any other number is padding of its own.
        if (RegExp(r'(?<![\w.])(\d*[1-9]\d*(\.\d+)?|\d+\.\d*[1-9]\d*)(?![\w])').hasMatch(args)) {
          violations.add('${file.path}:${lineOf(offset)}: padding of its own, use Space\n    EdgeInsets...($args)');
        }
      }
    }
    expect(violations, isEmpty, reason: violations.join('\n'));
  });

  test('the guard sees a page that sets its own font or padding', () {
    const page = '''
      Text('x', style: TextStyle(fontSize: 12));
      Padding(padding: const EdgeInsets.fromLTRB(10, 0, Space.lg, 0));
      Padding(padding: const EdgeInsets.all(Space.lg));
    ''';
    final fonts = [
      for (final (pattern, _) in forbidden)
        if (pattern.hasMatch(page)) pattern.pattern,
    ];
    expect(fonts, hasLength(3), reason: 'Text(, TextStyle( and fontSize');
    final padding = [
      for (final (_, args) in edgeInsets(page))
        if (RegExp(r'(?<![\w.])(\d*[1-9]\d*(\.\d+)?|\d+\.\d*[1-9]\d*)(?![\w])').hasMatch(args)) args,
    ];
    expect(padding, ['10, 0, Space.lg, 0']);
  });
}
