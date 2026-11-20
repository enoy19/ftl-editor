import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_look.dart';
import 'package:ftl_editor/look/ftl_tabs.dart';
import 'package:ftl_editor/look/look_mock.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:material_ui/material_ui.dart';

import 'game_image_test.dart' show pixelPng;

/// A look of one-pixel images and a font whose only glyph is `?`.
final fake = FtlLookSource(
  image: (path) async => pixelPng,
  insets: (path) async => const ImageInsets(left: 0, top: 0, right: 0, bottom: 0),
  font: (path) async => GameFont(
    atlasPng: pixelPng,
    lineHeight: 3,
    baseline: 2,
    glyphs: const [GameGlyph(codePoint: 0x3F, x: 0, y: 0, width: 1, height: 1, top: 1, left: 0, right: 1)],
  ),
);

void main() {
  test('a font measures a text by the advances of its glyphs, unknown signs as ?', () async {
    final glyphs = const [
      GameGlyph(codePoint: 0x41, x: 0, y: 0, width: 5, height: 7, top: 7, left: 0, right: 1),
      GameGlyph(codePoint: 0x3F, x: 6, y: 0, width: 4, height: 7, top: 7, left: 1, right: 1),
    ];
    final look = await FtlLook.load(fake);
    final font = FtlFont(look.font(FtlFontId.hl2).atlas, 9, 7, glyphs);
    expect(font.width('AA'), 12);
    expect(font.width('A€'), 6 + 6, reason: '€ is drawn as ?');
  });

  testWidgets('the mock screen shows the tabs, Expert, Save and the resources', (tester) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final look = (await tester.runAsync(() => FtlLook.load(fake)))!;
    expect(look.fonts.keys, FtlFontId.values);
    expect(look.images.keys, FtlImages.all);
    await tester.pumpWidget(variantApp(LookMockScreen(look: look), brightness: Brightness.dark));
    await tester.pumpAndSettle();
    for (final label in ['SHIP', 'CREW', 'EQUIPMENT', 'MAP', 'PROFILE', 'EXPERT', 'SAVE', '47', '16', '8', '2']) {
      expect(find.bySemanticsLabel(label), findsOneWidget, reason: label);
    }
    // The first tab starts inside the window (the old mock cut it off).
    final strip = tester.getTopLeft(find.byType(FtlTabStrip));
    expect(tester.getTopLeft(find.byKey(const ValueKey('ftl-tab-0'))).dx, greaterThanOrEqualTo(strip.dx));

    // A click opens a tab.
    await tester.tap(find.bySemanticsLabel('MAP'));
    await tester.pumpAndSettle();
    expect(tester.widget<FtlTabStrip>(find.byType(FtlTabStrip)).open, 3);
  });

  testWidgets('the loader says so when no ftl.dat exists', (tester) async {
    await tester.pumpWidget(variantApp(LookMockLoader(load: () async => null)));
    await tester.pumpAndSettle();
    expect(find.textContaining('No ftl.dat found'), findsOneWidget);
  });
}
