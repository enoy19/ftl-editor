// The helpers of the PNG shots (gallery_shots_test.dart,
// expert_shots_test.dart): the shot folder, the fonts of the Flutter SDK,
// the host build of the Rust bridge and the capture of one shot.

import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/src/rust/frb_generated.dart';
import 'package:material_ui/material_ui.dart';

const shotsDir = String.fromEnvironment('FTL_UI_SHOTS');

Future<void> loadFont(String family, List<String> files) async {
  final root = Platform.environment['FLUTTER_ROOT'];
  if (root == null) throw StateError('FLUTTER_ROOT is not set; run through `flutter test`.');
  final loader = FontLoader(family);
  for (final f in files) {
    final bytes = File('$root/bin/cache/artifacts/material_fonts/$f').readAsBytesSync();
    loader.addFont(Future.value(ByteData.sublistView(bytes)));
  }
  await loader.load();
}

/// The Rust bridge, loaded once per process; `null` with a printed reason.
bool? _rust;
Future<bool> loadRust(WidgetTester tester, String shot) async {
  if (_rust case final loaded?) return loaded;
  const libDefine = String.fromEnvironment('FTL_BRIDGE_LIB');
  final lib = libDefine.isNotEmpty ? libDefine : '../target/debug/libftl_bridge.dylib';
  if (!File(lib).existsSync()) {
    // ignore: avoid_print
    print('$shot: no Rust bridge at $lib (run `cargo build -p ftl_bridge`)');
    return _rust = false;
  }
  await tester.runAsync(() => RustLib.init(externalLibrary: ExternalLibrary.open(lib)));
  return _rust = true;
}

/// Loads Roboto and the Material icons once per test file, so the PNGs show
/// text instead of the boxes of the test font, and the macOS monospace font
/// as `Menlo` (TextRole.code: text fields, the hex). Flutter cannot read
/// Menlo.ttc, so SF Mono stands in; without it the code text stays boxes.
Future<void> loadShotFonts() async {
  await loadFont('Roboto', ['Roboto-Regular.ttf', 'Roboto-Medium.ttf', 'Roboto-Bold.ttf']);
  await loadFont('MaterialIcons', ['MaterialIcons-Regular.otf']);
  final mono = File('/System/Library/Fonts/SFNSMono.ttf');
  if (mono.existsSync()) {
    await (FontLoader('Menlo')..addFont(Future.value(ByteData.sublistView(mono.readAsBytesSync())))).load();
  }
}

/// Renders [app] at [size], runs [before], and writes `<shotsDir>/<name>.png`
/// at [pixelRatio].
Future<void> shoot(
  WidgetTester tester,
  String name,
  Widget app, {
  Future<void> Function()? before,
  double pixelRatio = 1.5,
  Size size = const Size(1280, 800),
}) async {
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  // flutter_test replaces every shadow with a black outline; the shots
  // show the real shadows and restore the test default before the test ends.
  debugDisableShadows = false;
  final boundary = GlobalKey();
  await tester.pumpWidget(RepaintBoundary(key: boundary, child: app));
  await tester.pumpAndSettle();
  if (before != null) await before();
  await tester.runAsync(() async {
    final render = boundary.currentContext!.findRenderObject()! as RenderRepaintBoundary;
    final image = await render.toImage(pixelRatio: pixelRatio);
    final png = await image.toByteData(format: ui.ImageByteFormat.png);
    await Directory(shotsDir).create(recursive: true);
    await File('$shotsDir/$name.png').writeAsBytes(png!.buffer.asUint8List());
  });
  debugDisableShadows = true;
}
