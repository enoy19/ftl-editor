import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:material_ui/material_ui.dart';

import '../src/rust/api/assets.dart';

/// A game image: an inner path of `ftl.dat` (`img/...`) or a blueprint id.
@immutable
class GameImageRef {
  const GameImageRef.path(String this.path) : kind = null, id = null;

  const GameImageRef.of(GameImageKind this.kind, String this.id) : path = null;

  final String? path;
  final GameImageKind? kind;
  final String? id;

  @override
  bool operator ==(Object other) => other is GameImageRef && other.path == path && other.kind == kind && other.id == id;

  @override
  int get hashCode => Object.hash(path, kind, id);

  @override
  String toString() => path ?? '${kind!.name}:$id';
}

/// Answers the PNG bytes of a game image.
typedef GameImageLoader = Future<Uint8List> Function(GameImageRef ref);

/// The Rust core: it reads the open `ftl.dat` and keeps a cache of its own.
Future<Uint8List> loadGameImage(GameImageRef ref) {
  final path = ref.path;
  return path != null ? gameImage(innerPath: path) : gameImageOf(kind: ref.kind!, id: ref.id!);
}

/// Feeds a game image into the Flutter image cache. The key is the `ftl.dat`
/// path and the image; the loader is not part of it.
@immutable
class GameImageProvider extends ImageProvider<GameImageProvider> {
  const GameImageProvider({required this.datPath, required this.ref, this.load = loadGameImage});

  final String datPath;
  final GameImageRef ref;
  final GameImageLoader load;

  @override
  Future<GameImageProvider> obtainKey(ImageConfiguration configuration) => SynchronousFuture(this);

  @override
  ImageStreamCompleter loadImage(GameImageProvider key, ImageDecoderCallback decode) =>
      MultiFrameImageStreamCompleter(codec: _codec(key, decode), scale: 1, debugLabel: '$ref');

  static Future<ui.Codec> _codec(GameImageProvider key, ImageDecoderCallback decode) async {
    final bytes = await key.load(key.ref);
    return decode(await ui.ImmutableBuffer.fromUint8List(bytes));
  }

  @override
  bool operator ==(Object other) => other is GameImageProvider && other.datPath == datPath && other.ref == ref;

  @override
  int get hashCode => Object.hash(datPath, ref);
}

/// Tells the [GameImage]s below which `ftl.dat` is open. `datPath == null`
/// is the reduced mode: every [GameImage] shows its Material icon.
///
/// An [InheritedTheme]: `showDialog` and `showAnchoredPopover` capture it
/// with the themes, so a dialog route above the page (the change sheet, the
/// palette, a value popover) sees the same `ftl.dat`.
class GameDataScope extends InheritedTheme {
  const GameDataScope({super.key, required this.datPath, this.load = loadGameImage, required super.child});

  final String? datPath;
  final GameImageLoader load;

  static GameDataScope? maybeOf(BuildContext context) => context.dependOnInheritedWidgetOfExactType<GameDataScope>();

  @override
  Widget wrap(BuildContext context, Widget child) => GameDataScope(datPath: datPath, load: load, child: child);

  @override
  bool updateShouldNotify(GameDataScope oldWidget) => datPath != oldWidget.datPath || load != oldWidget.load;
}

/// A game image from the local `ftl.dat`, or [fallback] in reduced mode and
/// when the image cannot be read. Pixel art scales without smoothing.
class GameImage extends StatelessWidget {
  const GameImage(
    this.ref, {
    super.key,
    required this.fallback,
    this.width = 48,
    this.height = 48,
    this.iconSize,
    this.color,
    this.fit = BoxFit.contain,
  });

  final GameImageRef ref;
  final IconData fallback;
  final double width;
  final double height;

  /// Size of the fallback icon; defaults to the smaller side.
  final double? iconSize;

  /// A color every pixel is multiplied with, channel by channel, as FTL's
  /// `GL_RenderPrimitiveWithColor` does (`BlendMode.modulate`: black stays
  /// black, transparent stays transparent); `null` draws the image as is.
  final Color? color;

  /// How the image fills [width] and [height]. [BoxFit.none] draws it at its
  /// own size (factor 1, pixel art stays sharp) and crops it to the box,
  /// centered.
  final BoxFit fit;

  @override
  Widget build(BuildContext context) {
    final scope = GameDataScope.maybeOf(context);
    final datPath = scope?.datPath;
    final icon = SizedBox(
      width: width,
      height: height,
      child: Center(
        child: Icon(fallback, size: iconSize ?? (width < height ? width : height), color: color),
      ),
    );
    if (scope == null || datPath == null) return icon;
    return Image(
      image: GameImageProvider(datPath: datPath, ref: ref, load: scope.load),
      width: width,
      height: height,
      fit: fit,
      filterQuality: FilterQuality.none,
      color: color,
      colorBlendMode: color == null ? null : BlendMode.modulate,
      errorBuilder: (_, _, _) => icon,
    );
  }
}

/// A game image as the icon of a list row, beside Material icons: cropped
/// to its visible pixels (alpha at least 0x80, so the glow of a system icon
/// is left out) and drawn at a whole factor that fits [size] (default: the
/// size of the icons around it, `IconTheme`), pixel-sharp ([GameIconState.fit]).
/// A 64 px system icon and a 35 px crew
/// cell thus fill the row like a 24 px Material icon instead of shrinking
/// with their empty margins. [color] multiplies a white symbol like
/// [GameImage.color]. In reduced mode, and when the image cannot be read,
/// it is the Material [fallback].
class GameIcon extends StatefulWidget {
  const GameIcon(this.ref, {super.key, required this.fallback, this.size, this.color});

  final GameImageRef ref;
  final IconData fallback;
  final double? size;
  final Color? color;

  @override
  State<GameIcon> createState() => GameIconState();
}

class GameIconState extends State<GameIcon> {
  /// The visible pixels of each image, measured once per image.
  static final Map<GameImageProvider, Rect> _visible = {};

  GameImageProvider? _provider;
  ImageStream? _stream;
  ImageStreamListener? _listener;
  ImageInfo? _info;
  Rect? _box;
  bool _failed = false;

  /// Where the last frame drew the visible pixels, inside the icon box.
  @visibleForTesting
  Rect? drawn;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _resolve();
  }

  @override
  void didUpdateWidget(GameIcon oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.ref != widget.ref) _resolve();
  }

  void _resolve() {
    final scope = GameDataScope.maybeOf(context);
    final datPath = scope?.datPath;
    final provider = scope == null || datPath == null
        ? null
        : GameImageProvider(datPath: datPath, ref: widget.ref, load: scope.load);
    if (provider == _provider) return;
    _stopListening();
    _provider = provider;
    _box = null;
    _failed = false;
    if (provider == null) return;
    final stream = provider.resolve(createLocalImageConfiguration(context));
    final listener = ImageStreamListener(
      (info, _) => _loaded(provider, info),
      onError: (_, _) {
        if (mounted) setState(() => _failed = true);
      },
    );
    stream.addListener(listener);
    _stream = stream;
    _listener = listener;
  }

  Future<void> _loaded(GameImageProvider provider, ImageInfo info) async {
    _info?.dispose();
    _info = info;
    final box = _visible[provider] ?? await _measure(info.image);
    if (box == null) {
      if (mounted) setState(() => _failed = true);
      return;
    }
    _visible[provider] = box;
    if (mounted && _provider == provider) setState(() => _box = box);
  }

  /// The bounding box of the pixels with alpha at least 0x80.
  static Future<Rect?> _measure(ui.Image image) async {
    final data = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
    if (data == null) return null;
    var (left, top, right, bottom) = (image.width, image.height, -1, -1);
    for (var y = 0; y < image.height; y++) {
      for (var x = 0; x < image.width; x++) {
        if (data.getUint8((y * image.width + x) * 4 + 3) < 0x80) continue;
        if (x < left) left = x;
        if (x > right) right = x;
        if (y < top) top = y;
        if (y > bottom) bottom = y;
      }
    }
    if (right < 0) return Rect.fromLTWH(0, 0, image.width.toDouble(), image.height.toDouble());
    return Rect.fromLTRB(left.toDouble(), top.toDouble(), right + 1.0, bottom + 1.0);
  }

  void _stopListening() {
    final listener = _listener;
    if (listener != null) _stream?.removeListener(listener);
    _stream = null;
    _listener = null;
  }

  @override
  void dispose() {
    _stopListening();
    _info?.dispose();
    super.dispose();
  }

  /// [box] scaled by a whole factor to a square of [size], centered on whole
  /// pixels. Art up to half again as large as [size] stays at factor 1 and
  /// reaches a little past the box (a crew figure, a system disc): a smaller
  /// copy would lose pixels. Larger art shrinks by a whole divisor.
  static Rect fit(Rect box, double size) {
    final longest = box.width > box.height ? box.width : box.height;
    final scale = longest <= size
        ? (size / longest).floorToDouble()
        : longest <= size * 1.5
        ? 1.0
        : 1 / (longest / size).ceil();
    final w = (box.width * scale).roundToDouble();
    final h = (box.height * scale).roundToDouble();
    return Rect.fromLTWH(((size - w) / 2).roundToDouble(), ((size - h) / 2).roundToDouble(), w, h);
  }

  @override
  Widget build(BuildContext context) {
    final size = widget.size ?? IconTheme.of(context).size ?? 24;
    final color = widget.color;
    final image = _info?.image;
    final box = _box;
    if (_provider == null || _failed || image == null || box == null) {
      drawn = null;
      return Icon(widget.fallback, size: size, color: color);
    }
    final dst = fit(box, size);
    drawn = dst;
    return SizedBox.square(
      dimension: size,
      child: CustomPaint(painter: _IconPainter(image, box, dst, color)),
    );
  }
}

class _IconPainter extends CustomPainter {
  _IconPainter(this.image, this.src, this.dst, this.color);

  final ui.Image image;
  final Rect src;
  final Rect dst;
  final Color? color;

  @override
  void paint(Canvas canvas, Size size) {
    final color = this.color;
    final paint = Paint()..filterQuality = FilterQuality.none;
    if (color != null) paint.colorFilter = ColorFilter.mode(color, BlendMode.modulate);
    canvas.drawImageRect(image, src, dst, paint);
  }

  @override
  bool shouldRepaint(_IconPainter old) => old.image != image || old.src != src || old.dst != dst || old.color != color;
}
