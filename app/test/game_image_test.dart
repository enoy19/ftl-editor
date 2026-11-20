import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/src/rust/api/assets.dart';
import 'package:ftl_editor/ui/game_image.dart';
import 'package:material_ui/material_ui.dart';

/// The 67-byte PNG of one transparent pixel.
final pixelPng = Uint8List.fromList(const [
  0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, //
  0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
  0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D,
  0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
]);

const kestrel = GameImageRef.of(GameImageKind.ship, 'PLAYER_SHIP_HARD');

Widget host(Widget child) => MaterialApp(
  home: Scaffold(body: Center(child: child)),
);

void main() {
  testWidgets('without ftl.dat GameImage shows the fallback icon and loads nothing', (tester) async {
    var loads = 0;
    await tester.pumpWidget(
      host(
        GameDataScope(
          datPath: null,
          load: (_) async {
            loads++;
            return pixelPng;
          },
          child: const GameImage(kestrel, fallback: Icons.rocket_launch),
        ),
      ),
    );
    expect(find.byIcon(Icons.rocket_launch), findsOneWidget);
    expect(find.byType(Image), findsNothing);
    expect(loads, 0);
  });

  testWidgets('without a scope GameImage shows the fallback icon', (tester) async {
    await tester.pumpWidget(host(const GameImage(kestrel, fallback: Icons.rocket_launch)));
    expect(find.byIcon(Icons.rocket_launch), findsOneWidget);
  });

  testWidgets('with ftl.dat GameImage shows the loaded image', (tester) async {
    final asked = <GameImageRef>[];
    await tester.pumpWidget(
      host(
        GameDataScope(
          datPath: '/games/ftl.dat',
          load: (ref) async {
            asked.add(ref);
            return pixelPng;
          },
          child: const GameImage(kestrel, fallback: Icons.rocket_launch),
        ),
      ),
    );
    await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
    await tester.pump();
    expect(find.byIcon(Icons.rocket_launch), findsNothing);
    expect(tester.widget<RawImage>(find.byType(RawImage)).image, isNotNull);
    expect(asked, [kestrel]);
  });

  testWidgets('a failed load shows the fallback icon', (tester) async {
    await tester.pumpWidget(
      host(
        GameDataScope(
          datPath: '/games/ftl.dat',
          load: (_) async => throw StateError('no such entry'),
          child: const GameImage(GameImageRef.path('img/none.png'), fallback: Icons.image_not_supported),
        ),
      ),
    );
    await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 50)));
    await tester.pump();
    expect(find.byIcon(Icons.image_not_supported), findsOneWidget);
  });

  test('the provider key is the ftl.dat path and the image, not the loader', () {
    Future<Uint8List> a(GameImageRef _) async => pixelPng;
    Future<Uint8List> b(GameImageRef _) async => pixelPng;
    expect(
      GameImageProvider(datPath: '/x/ftl.dat', ref: kestrel, load: a),
      GameImageProvider(
        datPath: '/x/ftl.dat',
        ref: const GameImageRef.of(GameImageKind.ship, 'PLAYER_SHIP_HARD'),
        load: b,
      ),
    );
    expect(
      GameImageProvider(datPath: '/x/ftl.dat', ref: kestrel, load: a),
      isNot(GameImageProvider(datPath: '/y/ftl.dat', ref: kestrel, load: a)),
    );
    expect(const GameImageRef.path('img/a.png'), isNot(const GameImageRef.path('img/b.png')));
  });
}
