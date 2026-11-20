// An FTL look for widget tests, built by the test itself: solid images in the
// sizes and insets of the real frames, and one font whose glyphs cover ASCII
// and the German letters with one atlas pixel each. Nothing comes from
// ftl.dat.

import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_look.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:material_ui/material_ui.dart';

/// Size and 9-slice insets (left, top, right, bottom) of each image, as the
/// real ftl.dat of FTL 1.6.13 answers them (docs/original-look.md).
const _images = {
  FtlImages.window: (66, 72, (31, 36, 32, 35)),
  FtlImages.box: (205, 122, (3, 3, 3, 3)),
  FtlImages.topScrap: (135, 55, (0, 0, 0, 0)),
  FtlImages.topFuel: (80, 40, (0, 0, 0, 0)),
  FtlImages.topMissiles: (80, 40, (0, 0, 0, 0)),
  FtlImages.topDrones: (75, 40, (0, 0, 0, 0)),
  FtlImages.kestrel: (64, 32, (0, 0, 0, 0)),
};

Future<Uint8List> _solidPng(int width, int height) async {
  final recorder = ui.PictureRecorder();
  Canvas(recorder)
      .drawRect(Rect.fromLTWH(0, 0, width.toDouble(), height.toDouble()), Paint()..color = const Color(0xFFEBF5E5));
  final image = await recorder.endRecording().toImage(width, height);
  return (await image.toByteData(format: ui.ImageByteFormat.png))!.buffer.asUint8List();
}

/// Glyphs for ASCII and the German letters: one atlas pixel each, 7 rows
/// high on the page, an advance of 6.
final _glyphs = [
  for (final r in [for (var c = 0x20; c < 0x7F; c++) c, ...'ÄÖÜäöüß'.runes])
    GameGlyph(codePoint: r, x: 0, y: 0, width: 1, height: 1, top: 7, left: 0, right: 5),
];

/// The source of the fake look.
final fakeLookSource = FtlLookSource(
  image: (path) {
    final (w, h, _) = _images[path]!;
    return _solidPng(w, h);
  },
  insets: (path) async {
    final (_, _, (l, t, r, b)) = _images[path]!;
    return ImageInsets(left: l, top: t, right: r, bottom: b);
  },
  font: (path) async => GameFont(atlasPng: await _solidPng(1, 1), lineHeight: 19, baseline: 12, glyphs: _glyphs),
);

/// Loads the fake look; image decoding needs real time.
Future<FtlLook> loadFakeLook(WidgetTester tester) async => (await tester.runAsync(() => FtlLook.load(fakeLookSource)))!;

/// [child] in the FTL look.
Widget inFtlLook(FtlLook look, Widget child) => Theme(data: ftlLookTheme(look), child: child);

/// A text in either look: a [Text] in the fallback font or an [FtlText].
Finder findLabel(String text) =>
    find.byWidgetPredicate((w) => (w is Text && w.data == text) || (w is FtlText && w.text == text));

/// The old and new text of a change sheet row.
Finder findText(String old, String value) =>
    find.byWidgetPredicate((w) => w is ChangeValues && w.old == old && w.value == value);

/// The old and new value of a change sheet row.
Finder findChange(int old, int value) =>
    find.byWidgetPredicate((w) => w is ChangeValues && w.old == '$old' && w.value == '$value');
