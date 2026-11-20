import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:material_ui/material_ui.dart';

void main() {
  testWidgets('the gallery shows the FTL look and speaks German', (tester) async {
    tester.view.physicalSize = const Size(1280, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(const GalleryApp());
    await tester.pumpAndSettle();
    // The FTL look needs the Rust core, which unit tests do not load: it says so.
    expect(find.byKey(const ValueKey('gallery-look-ftl')), findsOneWidget);
    expect(find.textContaining('The FTL look cannot be read from ftl.dat'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.translate));
    await tester.pumpAndSettle();
    expect(find.text('UI-Varianten'), findsOneWidget);
  });
}
