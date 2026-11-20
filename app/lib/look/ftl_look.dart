// The original FTL look, read once from the local ftl.dat: the bitmap fonts,
// the frame and button images with their 9-slice insets, and the colors
// measured from those images (docs/original-look.md). Nothing of it is
// stored in the repository.

import 'dart:async';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:material_ui/material_ui.dart';

import '../src/rust/api/assets.dart';

/// Colors measured from the UI images of FTL 1.6.13 (docs/original-look.md).
abstract final class FtlColors {
  /// Outline of every frame, button face "on", text on dark ground.
  static const line = Color(0xFFEBF5E5);

  /// Button face "off" (disabled).
  static const off = Color(0xFFA4ABA0);

  /// Button face "select2" (hover, pressed).
  static const hover = Color(0xFFFFE65E);

  /// Ink of the labels on a button face.
  static const ink = Color(0xFF193133);

  /// Glow around frames (its alpha falls off outwards; 0x45 next to the line).
  static const glow = Color(0xFF79F2FD);

  /// Fill of a window (img/window_base.png, mean of its hex pattern).
  static const window = Color(0xFF7A6463);

  /// Fill of an inner box (img/customizeUI/box_crew_details.png, alpha 0xF6).
  static const box = Color(0xF6292422);

  /// Ground behind the windows (inside img/main_menus/button_cancel_base.png).
  static const ground = Color(0xFF090909);

  /// Warnings (img/statusUI/top_hull_red_label.png).
  static const warn = Color(0xFFFD5446);

  /// A pressed button: FTL draws no own face for it, so the hover face moves
  /// a fifth towards the ink (`Color.lerp(hover, ink, 0.2)`).
  static const pressed = Color(0xFFD1C255);
}

/// The fonts of FTL 1.6.13 that the look uses.
enum FtlFontId {
  justin8('fonts/JustinFont8.font'),
  justin10('fonts/JustinFont10.font'),
  justin11Bold('fonts/JustinFont11Bold.font'),
  justin12Bold('fonts/JustinFont12Bold.font'),
  hl1('fonts/HL1.font'),
  hl2('fonts/HL2.font'),
  numbers('fonts/num_font.font');

  const FtlFontId(this.path);

  final String path;
}

/// The frame and button images the look uses, by inner path.
abstract final class FtlImages {
  static const window = 'img/window_outline.png';
  static const box = 'img/customizeUI/box_crew_details.png';
  static const topScrap = 'img/statusUI/top_scrap.png';
  static const topFuel = 'img/statusUI/top_fuel_on.png';
  static const topMissiles = 'img/statusUI/top_missiles_on.png';
  static const topDrones = 'img/statusUI/top_drones_on.png';
  static const kestrel = 'img/ship/kestral_base.png';

  static const all = [window, box, topScrap, topFuel, topMissiles, topDrones, kestrel];
}

/// A bitmap font: the atlas image and its glyphs by code point.
class FtlFont {
  FtlFont(this.atlas, this.lineHeight, this.baseline, Iterable<GameGlyph> glyphs)
    : glyphs = {for (final g in glyphs) g.codePoint: g};

  final ui.Image atlas;
  final int lineHeight;
  final int baseline;
  final Map<int, GameGlyph> glyphs;

  /// The glyph of [rune]; `?` for a sign the font lacks. The fonts lack the
  /// middle dot (U+00B7) that separates facts ("Sector 8 · Easy"): it is the
  /// full stop raised to the middle of the capitals.
  GameGlyph? glyph(int rune) => glyphs[rune] ?? (rune == 0xB7 ? _middleDot : null) ?? glyphs[0x3F];

  late final GameGlyph? _middleDot = () {
    final dot = glyphs[0x2E];
    if (dot == null) return null;
    final cap = glyphs[0x48]?.top ?? baseline;
    return GameGlyph(
      codePoint: 0xB7,
      x: dot.x,
      y: dot.y,
      width: dot.width,
      height: dot.height,
      top: (cap + dot.height) ~/ 2,
      left: dot.left,
      right: dot.right,
    );
  }();

  /// Whether every sign of [text] has its own glyph (no `?` stands in).
  bool covers(String text) => text.runes.every((r) => glyphs.containsKey(r) || (r == 0xB7 && _middleDot != null));

  /// Rows from the top of the line box down to the baseline as [FtlText]
  /// draws it: the capital letters (the height of "H") stand in the middle
  /// of the line box. The fonts put the baseline at 12 of 19 rows, so text
  /// drawn at the font's own baseline sits too high beside an icon or in a
  /// box; centering the capitals is the one fix for every text.
  late final int drawBaseline = () {
    final cap = (glyphs[0x48] ?? glyph(0x48))?.top ?? baseline;
    return (lineHeight - cap) ~/ 2 + cap;
  }();

  /// The width of [text] in font pixels.
  int width(String text) => text.runes.fold(0, (w, r) {
    final g = glyph(r);
    return g == null ? w : w + g.left + g.width + g.right;
  });
}

/// An image with the insets a 9-slice must not stretch.
class FtlImage {
  const FtlImage(this.image, this.insets);

  final ui.Image image;
  final ImageInsets insets;

  /// The stretchable middle, for [Canvas.drawImageNine].
  Rect get center => Rect.fromLTRB(
    insets.left.toDouble(),
    insets.top.toDouble(),
    (image.width - insets.right).toDouble(),
    (image.height - insets.bottom).toDouble(),
  );
}

/// Where the look comes from: the Rust core over the open ftl.dat, or a
/// test's fake.
class FtlLookSource {
  const FtlLookSource({required this.image, required this.insets, required this.font});

  /// The bridge to the open ftl.dat.
  static const rust = FtlLookSource(image: _image, insets: _insets, font: _font);

  final Future<Uint8List> Function(String path) image;
  final Future<ImageInsets> Function(String path) insets;
  final Future<GameFont> Function(String path) font;
}

Future<Uint8List> _image(String path) => gameImage(innerPath: path);
Future<ImageInsets> _insets(String path) => gameImageInsets(innerPath: path);
Future<GameFont> _font(String path) => gameFont(innerPath: path);

Future<ui.Image> _decode(Uint8List png) async {
  final codec = await ui.instantiateImageCodec(png);
  return (await codec.getNextFrame()).image;
}

/// Everything the look draws, decoded once.
class FtlLook {
  FtlLook(this.fonts, this.images);

  final Map<FtlFontId, FtlFont> fonts;
  final Map<String, FtlImage> images;

  FtlFont font(FtlFontId id) => fonts[id]!;
  FtlImage image(String path) => images[path]!;

  static Future<FtlLook> load([FtlLookSource source = FtlLookSource.rust]) async {
    final fonts = <FtlFontId, FtlFont>{};
    for (final id in FtlFontId.values) {
      final f = await source.font(id.path);
      fonts[id] = FtlFont(await _decode(f.atlasPng), f.lineHeight, f.baseline, f.glyphs);
    }
    final images = <String, FtlImage>{};
    for (final path in FtlImages.all) {
      images[path] = FtlImage(await _decode(await source.image(path)), await source.insets(path));
    }
    return FtlLook(fonts, images);
  }
}

/// Text in an FTL bitmap font. One font pixel is [scale] logical pixels, a
/// whole number, so the pixels stay square; glyphs are drawn without
/// smoothing and tinted with [color]. The capitals stand in the vertical
/// middle of the line box ([FtlFont.drawBaseline]), so a centered [FtlText]
/// is centered by its glyphs. Pages use `AppText`, which picks the font.
class FtlText extends StatelessWidget {
  const FtlText(
    this.text, {
    super.key,
    required this.font,
    this.color = FtlColors.line,
    this.scale = 1,
    this.wrap = false,
    this.align = TextAlign.start,
  });

  final String text;
  final FtlFont font;
  final Color color;
  final int scale;

  /// Breaks the text into lines at spaces to fit the width; [align] places
  /// each line. Without it the text is one line of its own width.
  final bool wrap;
  final TextAlign align;

  @override
  Widget build(BuildContext context) => Semantics(
    label: text,
    child: wrap
        ? _FtlParagraph(text: text, font: font, color: color, scale: scale, align: align)
        : CustomPaint(
            size: Size((font.width(text) * scale).toDouble(), (font.lineHeight * scale).toDouble()),
            painter: _TextPainter(text, font, color, scale),
          ),
  );
}

/// Draws one line of [text] with its line box at [origin].
void _paintLine(Canvas canvas, FtlFont font, String text, Offset origin, int scale, Paint paint) {
  var pen = 0;
  for (final r in text.runes) {
    final g = font.glyph(r);
    if (g == null) continue;
    final src = Rect.fromLTWH(g.x.toDouble(), g.y.toDouble(), g.width.toDouble(), g.height.toDouble());
    final dst = Rect.fromLTWH(
      origin.dx + ((pen + g.left) * scale).toDouble(),
      origin.dy + ((font.drawBaseline - g.top) * scale).toDouble(),
      (g.width * scale).toDouble(),
      (g.height * scale).toDouble(),
    );
    canvas.drawImageRect(font.atlas, src, dst, paint);
    pen += g.left + g.width + g.right;
  }
}

Paint _glyphPaint(Color color) => Paint()
  ..filterQuality = FilterQuality.none
  ..colorFilter = ColorFilter.mode(color, BlendMode.srcIn);

class _TextPainter extends CustomPainter {
  _TextPainter(this.text, this.font, this.color, this.scale);

  final String text;
  final FtlFont font;
  final Color color;
  final int scale;

  /// [text], or as much of it as fits [width] followed by `..` (the fonts
  /// have no ellipsis sign), as Material cuts a line.
  String _fit(double width) {
    if (font.width(text) * scale <= width) return text;
    final runes = text.runes.toList();
    for (var n = runes.length - 1; n > 0; n--) {
      final cut = '${String.fromCharCodes(runes.take(n)).trimRight()}..';
      if (font.width(cut) * scale <= width) return cut;
    }
    return '';
  }

  @override
  void paint(Canvas canvas, Size size) =>
      _paintLine(canvas, font, _fit(size.width), Offset.zero, scale, _glyphPaint(color));

  @override
  bool shouldRepaint(_TextPainter old) =>
      old.text != text || old.font != font || old.color != color || old.scale != scale;
}

/// Bitmap text broken into lines at spaces. A render object of its own, so
/// it answers its intrinsic sizes (a dialog sized to its content asks them).
class _FtlParagraph extends LeafRenderObjectWidget {
  const _FtlParagraph({
    required this.text,
    required this.font,
    required this.color,
    required this.scale,
    required this.align,
  });

  final String text;
  final FtlFont font;
  final Color color;
  final int scale;
  final TextAlign align;

  @override
  RenderObject createRenderObject(BuildContext context) => _RenderFtlParagraph(text, font, color, scale, align);

  @override
  void updateRenderObject(BuildContext context, _RenderFtlParagraph r) => r
    ..text = text
    ..font = font
    ..color = color
    ..scale = scale
    ..align = align;
}

class _RenderFtlParagraph extends RenderBox {
  _RenderFtlParagraph(this._text, this._font, this._color, this._scale, this._align);

  String _text;
  FtlFont _font;
  Color _color;
  int _scale;
  TextAlign _align;
  List<String> _lines = const [];

  set text(String v) {
    if (v == _text) return;
    _text = v;
    markNeedsLayout();
  }

  set font(FtlFont v) {
    if (v == _font) return;
    _font = v;
    markNeedsLayout();
  }

  set color(Color v) {
    if (v == _color) return;
    _color = v;
    markNeedsPaint();
  }

  set scale(int v) {
    if (v == _scale) return;
    _scale = v;
    markNeedsLayout();
  }

  set align(TextAlign v) {
    if (v == _align) return;
    _align = v;
    markNeedsPaint();
  }

  double _width(String s) => (_font.width(s) * _scale).toDouble();
  double get _lineHeight => (_font.lineHeight * _scale).toDouble();

  /// Greedy lines no wider than [maxWidth]; a word wider than that keeps a
  /// line of its own.
  List<String> _break(double maxWidth) {
    final lines = <String>[];
    for (final paragraph in _text.split('\n')) {
      var line = '';
      for (final word in paragraph.split(' ')) {
        final next = line.isEmpty ? word : '$line $word';
        if (line.isNotEmpty && _width(next) > maxWidth) {
          lines.add(line);
          line = word;
        } else {
          line = next;
        }
      }
      lines.add(line);
    }
    return lines;
  }

  @override
  double computeMinIntrinsicWidth(double height) =>
      _text.split(RegExp(r'[ \n]')).fold(0.0, (w, word) => w > _width(word) ? w : _width(word));

  @override
  double computeMaxIntrinsicWidth(double height) =>
      _text.split('\n').fold(0.0, (w, line) => w > _width(line) ? w : _width(line));

  @override
  double computeMinIntrinsicHeight(double width) => _break(width).length * _lineHeight;

  @override
  double computeMaxIntrinsicHeight(double width) => computeMinIntrinsicHeight(width);

  @override
  Size computeDryLayout(BoxConstraints constraints) {
    final lines = _break(constraints.maxWidth);
    final w = lines.fold(0.0, (w, l) => w > _width(l) ? w : _width(l));
    return constraints.constrain(Size(w, lines.length * _lineHeight));
  }

  @override
  void performLayout() {
    _lines = _break(constraints.maxWidth);
    size = computeDryLayout(constraints);
  }

  @override
  void paint(PaintingContext context, Offset offset) {
    final paint = _glyphPaint(_color);
    for (final (i, line) in _lines.indexed) {
      final free = size.width - _width(line);
      final dx = switch (_align) {
        TextAlign.center => free / 2,
        TextAlign.right || TextAlign.end => free,
        _ => 0.0,
      };
      _paintLine(context.canvas, _font, line, offset + Offset(dx.floorToDouble(), i * _lineHeight), _scale, paint);
    }
  }
}

/// Clips the sides of an [FtlText] that may be too wide, never its top and
/// bottom: the dots of Ä, Ö and Ü stand above the line box of the font, and a
/// plain [ClipRect] cuts them off.
class FtlSideClip extends CustomClipper<Rect> {
  const FtlSideClip();

  @override
  Rect getClip(Size size) => Rect.fromLTRB(0, -size.height, size.width, 2 * size.height);

  @override
  bool shouldReclip(FtlSideClip oldClipper) => false;
}

/// A rectangle whose corners are cut by [cut] pixels, the shape of FTL's
/// windows and buttons.
Path cutCorners(Rect r, double cut) => Path()
  ..moveTo(r.left + cut, r.top)
  ..lineTo(r.right - cut, r.top)
  ..lineTo(r.right, r.top + cut)
  ..lineTo(r.right, r.bottom - cut)
  ..lineTo(r.right - cut, r.bottom)
  ..lineTo(r.left + cut, r.bottom)
  ..lineTo(r.left, r.bottom - cut)
  ..lineTo(r.left, r.top + cut)
  ..close();

/// A 9-slice of an FTL frame image behind [child]. [fill] paints the inside
/// first (the outline images have a transparent middle): a rectangle
/// [inset] pixels inside the image edge with corners cut by [cut].
class FtlFrame extends StatelessWidget {
  const FtlFrame({
    super.key,
    required this.image,
    this.fill,
    this.inset = 0,
    this.cut = 0,
    this.padding = EdgeInsets.zero,
    this.child,
  });

  final FtlImage image;
  final Color? fill;
  final double inset;
  final double cut;
  final EdgeInsets padding;
  final Widget? child;

  @override
  Widget build(BuildContext context) {
    final i = image.insets;
    // Smaller than its corners, a 9-slice would overlap them.
    return ConstrainedBox(
      constraints: BoxConstraints(
        minWidth: (i.left + i.right + 1).toDouble(),
        minHeight: (i.top + i.bottom + 1).toDouble(),
      ),
      child: CustomPaint(
        painter: _NinePainter(image, fill, inset, cut),
        child: Padding(padding: padding, child: child),
      ),
    );
  }
}

class _NinePainter extends CustomPainter {
  _NinePainter(this.image, this.fill, this.inset, this.cut);

  final FtlImage image;
  final Color? fill;
  final double inset;
  final double cut;

  @override
  void paint(Canvas canvas, Size size) {
    final dst = Offset.zero & size;
    final fill = this.fill;
    if (fill != null) canvas.drawPath(cutCorners(dst.deflate(inset), cut), Paint()..color = fill);
    canvas.drawImageNine(image.image, image.center, dst, Paint()..filterQuality = FilterQuality.none);
  }

  @override
  bool shouldRepaint(_NinePainter old) =>
      old.image != image || old.fill != fill || old.inset != inset || old.cut != cut;
}

/// The faces of an FTL button: `_on` (idle), `_off` (disabled) and
/// `_select2` (hover and focus). FTL has no own pressed face; the pressed one
/// is the hover face a step towards the ink.
enum FtlButtonState {
  on(FtlColors.line),
  off(FtlColors.off),
  hover(FtlColors.hover),
  pressed(FtlColors.pressed);

  const FtlButtonState(this.face);

  final Color face;
}

/// A text button as FTL draws one (the faces of img/customizeUI/button_accept_*):
/// a face with corners cut by 2 pixels, the label in [FtlColors.ink], an
/// optional [icon] before it. [onPressed] `null` disables it. Enter and Space
/// press it while it has the focus; [autofocus] takes the focus at once.
/// [state] fixes the face for a still shot.
class FtlButton extends StatefulWidget {
  const FtlButton({
    super.key,
    this.label,
    required this.font,
    this.icon,
    this.onPressed,
    this.autofocus = false,
    this.tooltip,
    this.state,
  });

  final String? label;
  final FtlFont font;
  final IconData? icon;
  final VoidCallback? onPressed;
  final bool autofocus;
  final String? tooltip;
  final FtlButtonState? state;

  @override
  State<FtlButton> createState() => _FtlButtonState();
}

class _FtlButtonState extends State<FtlButton> {
  bool _hover = false;
  bool _focus = false;
  bool _down = false;

  bool get _enabled => widget.onPressed != null;

  FtlButtonState get _face =>
      widget.state ??
      switch (null) {
        _ when !_enabled => FtlButtonState.off,
        _ when _down => FtlButtonState.pressed,
        _ when _hover || _focus => FtlButtonState.hover,
        _ => FtlButtonState.on,
      };

  void _press() => widget.onPressed?.call();

  @override
  Widget build(BuildContext context) {
    final label = widget.label;
    final icon = widget.icon;
    final face = _face;
    Widget button = FocusableActionDetector(
      enabled: _enabled,
      autofocus: widget.autofocus,
      mouseCursor: _enabled ? SystemMouseCursors.click : MouseCursor.defer,
      onFocusChange: (v) => setState(() => _focus = v),
      actions: {ActivateIntent: CallbackAction<ActivateIntent>(onInvoke: (_) => _press())},
      // A MouseRegion, not onShowHoverHighlight: that one waits for the
      // traditional focus highlight mode.
      child: MouseRegion(
        onEnter: (_) => setState(() => _hover = true),
        onExit: (_) => setState(() => _hover = false),
        child: GestureDetector(
          onTapDown: _enabled ? (_) => setState(() => _down = true) : null,
          onTapUp: _enabled ? (_) => setState(() => _down = false) : null,
          onTapCancel: _enabled ? () => setState(() => _down = false) : null,
          onTap: _enabled ? _press : null,
          child: Semantics(
            button: true,
            enabled: _enabled,
            child: CustomPaint(
              painter: _FacePainter(face.face),
              child: Padding(
                padding: EdgeInsets.fromLTRB(label == null ? 6 : 10, 5, label == null ? 6 : 10, 4),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    if (icon != null)
                      SizedBox(
                        height: widget.font.lineHeight.toDouble(),
                        child: Icon(icon, size: 16, color: FtlColors.ink),
                      ),
                    if (icon != null && label != null) const SizedBox(width: 6),
                    if (label != null) FtlText(label, font: widget.font, color: FtlColors.ink),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
    final tooltip = widget.tooltip;
    if (tooltip != null) button = Tooltip(message: tooltip, child: button);
    return button;
  }
}

class _FacePainter extends CustomPainter {
  _FacePainter(this.color);

  final Color color;

  @override
  void paint(Canvas canvas, Size size) => canvas.drawPath(cutCorners(Offset.zero & size, 2), Paint()..color = color);

  @override
  bool shouldRepaint(_FacePainter old) => old.color != color;
}
