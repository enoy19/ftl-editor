// GameIcon: a game image as the icon of a list row, as large as the
// Material icons beside it (a 64 px system icon and a
// 35 px crew cell were squeezed into 24 px with their empty margins).

import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:material_ui/material_ui.dart';

/// A PNG of [width] x [height]: transparent, a faint glow (alpha 0x40) in
/// [glow] and opaque white in [art].
Future<Uint8List> png(int width, int height, Rect art, {Rect? glow}) async {
  final recorder = ui.PictureRecorder();
  final canvas = Canvas(recorder);
  if (glow != null) canvas.drawRect(glow, Paint()..color = const Color(0x40FFFFFF));
  canvas.drawRect(art, Paint()..color = const Color(0xFFFFFFFF));
  final image = await recorder.endRecording().toImage(width, height);
  final data = await image.toByteData(format: ui.ImageByteFormat.png);
  return data!.buffer.asUint8List();
}

void main() {
  test('the fit is a whole factor; slightly larger art stays at 1, much larger shrinks by a divisor', () {
    expect(GameIconState.fit(const Rect.fromLTWH(0, 0, 8, 12), 24), const Rect.fromLTWH(4, 0, 16, 24));
    expect(GameIconState.fit(const Rect.fromLTWH(0, 0, 20, 30), 24).size, const Size(20, 30));
    expect(GameIconState.fit(const Rect.fromLTWH(0, 0, 64, 40), 24).size, const Size(21, 13));
  });

  testWidgets('a game icon in a ListRow is as large as a Material icon in the same kind of row', (tester) async {
    // A 64 px system icon: a 12 px symbol inside a faint glow; a crew cell:
    // 35 px with 9 x 18 visible pixels.
    final images = <String, Uint8List>{};
    await tester.runAsync(() async {
      images['img/system.png'] = await png(
        64,
        64,
        const Rect.fromLTWH(26, 26, 12, 12),
        glow: const Rect.fromLTWH(10, 10, 44, 44),
      );
      images['img/crew.png'] = await png(35, 35, const Rect.fromLTWH(13, 8, 9, 18));
    });
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: GameDataScope(
            datPath: '/test/ftl.dat',
            load: (ref) async => images[ref.path]!,
            child: const Column(
              children: [
                ListRow(key: ValueKey('material'), leading: Icon(Icons.local_fire_department), title: 'Fire'),
                ListRow(
                  key: ValueKey('system'),
                  leading: GameIcon(
                    GameImageRef.path('img/system.png'),
                    key: ValueKey('system-icon'),
                    fallback: Icons.settings,
                  ),
                  title: 'Shields',
                ),
                ListRow(
                  key: ValueKey('crew'),
                  leading: GameIcon(
                    GameImageRef.path('img/crew.png'),
                    key: ValueKey('crew-icon'),
                    fallback: Icons.person,
                  ),
                  title: 'Holly',
                ),
              ],
            ),
          ),
        ),
      ),
    );
    // Decoding and measuring the pixels runs outside the fake clock.
    for (var i = 0; i < 5; i++) {
      await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await tester.pump();
    }
    final material = tester.getSize(find.byIcon(Icons.local_fire_department));
    for (final key in ['system-icon', 'crew-icon']) {
      expect(tester.getSize(find.byKey(ValueKey(key))), material, reason: '$key takes the box of a Material icon');
    }
    final system = tester.state<GameIconState>(find.byKey(const ValueKey('system-icon'))).drawn!;
    expect(system.size, const Size(24, 24), reason: 'the 12 px symbol without its glow, at factor 2');
    final crew = tester.state<GameIconState>(find.byKey(const ValueKey('crew-icon'))).drawn!;
    expect(crew.size, const Size(9, 18), reason: 'factor 1: 18 px fits 24 px once');
  });
}
