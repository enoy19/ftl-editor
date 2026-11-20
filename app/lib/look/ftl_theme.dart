// The FTL look as a theme: a ThemeExtension that carries the look read from
// ftl.dat, inside a ThemeData of the measured colors. One mechanism: a Theme
// above the page reaches every page, and showDialog and showModalBottomSheet
// capture it for their routes. Without ftl.dat there is no extension and the
// app keeps its Material look.

import 'package:material_ui/material_ui.dart';

import '../ui/components/tokens.dart';
import 'ftl_look.dart';

/// The look of FTL for the widgets below a [Theme] built by [ftlLookTheme].
class FtlTheme extends ThemeExtension<FtlTheme> {
  const FtlTheme(this.look);

  final FtlLook look;

  /// The FTL look, or `null` for the Material look.
  static FtlLook? of(BuildContext context) => Theme.of(context).extension<FtlTheme>()?.look;

  @override
  FtlTheme copyWith({FtlLook? look}) => FtlTheme(look ?? this.look);

  @override
  FtlTheme lerp(FtlTheme? other, double t) => t < 0.5 || other == null ? this : other;
}

/// Ground of the bars and fields: the inner box color without its alpha.
const _field = Color(0xFF1C1817);

/// The Material theme of the FTL look: the measured colors on every Material
/// widget the pages keep (text fields, tooltips, menus, progress lines,
/// snack bars), the system font as the fallback for free text, and [look]
/// as [FtlTheme].
ThemeData ftlLookTheme(FtlLook look) {
  const scheme = ColorScheme(
    brightness: Brightness.dark,
    primary: FtlColors.hover,
    onPrimary: FtlColors.ink,
    primaryContainer: Color(0xFF4A4322),
    onPrimaryContainer: FtlColors.hover,
    secondary: FtlColors.line,
    onSecondary: FtlColors.ink,
    tertiary: FtlColors.glow,
    onTertiary: FtlColors.ink,
    error: FtlColors.warn,
    onError: FtlColors.ink,
    errorContainer: Color(0xFF4A1C18),
    onErrorContainer: FtlColors.warn,
    surface: _field,
    onSurface: FtlColors.line,
    onSurfaceVariant: FtlColors.off,
    surfaceContainerLowest: FtlColors.ground,
    surfaceContainerLow: _field,
    surfaceContainer: _field,
    surfaceContainerHigh: Color(0xFF292422),
    surfaceContainerHighest: Color(0xFF3A3331),
    outline: FtlColors.off,
    outlineVariant: Color(0xFF55504C),
    inverseSurface: FtlColors.line,
    onInverseSurface: FtlColors.ink,
  );
  final base = ThemeData(colorScheme: scheme, useMaterial3: true);
  return base.copyWith(
    scaffoldBackgroundColor: FtlColors.ground,
    textTheme: base.textTheme.apply(bodyColor: FtlColors.line, displayColor: FtlColors.line),
    iconTheme: const IconThemeData(color: FtlColors.line),
    tooltipTheme: const TooltipThemeData(
      decoration: BoxDecoration(
        color: Color(0xF6292422),
        border: Border.fromBorderSide(BorderSide(color: FtlColors.line)),
      ),
      textStyle: TextStyle(color: FtlColors.line, fontSize: 12),
    ),
    popupMenuTheme: const PopupMenuThemeData(
      color: Color(0xFF292422),
      shape: BeveledRectangleBorder(
        side: BorderSide(color: FtlColors.line, width: 2),
        borderRadius: BorderRadius.all(Radius.circular(4)),
      ),
    ),
    snackBarTheme: const SnackBarThemeData(
      backgroundColor: Color(0xFF292422),
      contentTextStyle: TextStyle(color: FtlColors.line),
      behavior: SnackBarBehavior.fixed,
    ),
    progressIndicatorTheme: const ProgressIndicatorThemeData(color: FtlColors.hover),
    inputDecorationTheme: const InputDecorationTheme(
      filled: true,
      fillColor: FtlColors.ground,
      border: OutlineInputBorder(borderRadius: BorderRadius.zero),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.zero,
        borderSide: BorderSide(color: FtlColors.line),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.zero,
        borderSide: BorderSide(color: FtlColors.hover, width: 2),
      ),
    ),
    textSelectionTheme: const TextSelectionThemeData(cursorColor: FtlColors.hover),
    extensions: [FtlTheme(look), AppTokens.ftl],
  );
}

/// An FTL window: the 9-slice of img/window_outline.png around [child],
/// filled with [fill] ([FtlColors.window] for a page or dialog,
/// [FtlColors.box] for a card on it). [padding] is the room between the
/// frame line and [child].
class FtlWindow extends StatelessWidget {
  const FtlWindow({
    super.key,
    required this.look,
    this.fill = FtlColors.box,
    this.padding = EdgeInsets.zero,
    required this.child,
  });

  /// From the image edge to the inside of the line: 7 px glow, 4 px line.
  static const line = 11.0;

  final FtlLook look;
  final Color fill;
  final EdgeInsets padding;
  final Widget child;

  @override
  Widget build(BuildContext context) => FtlFrame(
    image: look.image(FtlImages.window),
    fill: fill,
    inset: 9,
    cut: 12,
    padding: padding + const EdgeInsets.all(line),
    child: child,
  );
}

/// A small FTL box drawn from the measured colors: the inner box fill, a
/// 2 px outline and corners cut by 4. For tiles too small for [FtlWindow]
/// and for tags. [outline] is the line color.
class FtlBox extends StatelessWidget {
  const FtlBox({super.key, this.outline = FtlColors.line, this.padding = EdgeInsets.zero, required this.child});

  final Color outline;
  final EdgeInsets padding;
  final Widget child;

  @override
  Widget build(BuildContext context) => CustomPaint(
    painter: _BoxPainter(outline),
    child: Padding(padding: padding + const EdgeInsets.all(2), child: child),
  );
}

class _BoxPainter extends CustomPainter {
  _BoxPainter(this.outline);

  final Color outline;

  @override
  void paint(Canvas canvas, Size size) {
    final path = cutCorners((Offset.zero & size).deflate(1), 4);
    canvas
      ..drawPath(path, Paint()..color = FtlColors.box)
      ..drawPath(
        path,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..color = outline,
      );
  }

  @override
  bool shouldRepaint(_BoxPainter old) => old.outline != outline;
}
