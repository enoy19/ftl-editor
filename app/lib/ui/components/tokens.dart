// The one source for text styles, spacing and the few sizes the components
// share, in both looks. The FTL look (ftlLookTheme) and the Material look
// (ftlTheme) each carry an [AppTokens] as a ThemeExtension; the components
// read them and no page sets a font family, a font size or its own padding
// (test/no_own_fonts_test.dart checks the pages).

import 'package:material_ui/material_ui.dart';

import '../../look/ftl_look.dart';

/// What a text is for. The role picks the font in each look.
enum TextRole {
  /// The ship name on the ship card.
  display,

  /// The title of a panel, dialog, popover or section.
  title,

  /// A name in a row, a button label, a value next to a control.
  label,

  /// Running text: hints, notes, dialog bodies, status bar items.
  body,

  /// A second line, an id, a small detail.
  caption,

  /// A number that is a value (resources, levels, stats, scores).
  number,

  /// Raw bytes in columns (the hex view of the Expert area): system text in
  /// a monospace font in both looks, because the bitmap fonts are
  /// proportional.
  code,
}

/// The spacing scale: every gap and padding of a page is one of these.
abstract final class Space {
  static const xs = 4.0;
  static const sm = 8.0;
  static const md = 12.0;
  static const lg = 16.0;
  static const xl = 24.0;
}

/// The monospace fonts of [TextRole.code] after Menlo (macOS): Windows, Linux.
const _monospace = ['Consolas', 'DejaVu Sans Mono', 'monospace'];

/// Text styles, bitmap fonts, padding and row heights of one look.
@immutable
class AppTokens extends ThemeExtension<AppTokens> {
  const AppTokens({
    required this.styles,
    this.bitmap = const {},
    required this.text,
    required this.muted,
    required this.accent,
    required this.warn,
    required this.panelPadding,
    required this.tilePadding,
    required this.dialogPadding,
    required this.rowHeight,
    required this.denseRowHeight,
  });

  /// The system-font style of each role. In the FTL look it draws free text
  /// whose signs the bitmap font lacks.
  final Map<TextRole, TextStyle> styles;

  /// The bitmap font and its whole-number scale of each role; empty in the
  /// Material look.
  final Map<TextRole, (FtlFontId, int)> bitmap;

  /// The color of text, of muted text (second lines, hints), of a marked or
  /// hovered value and of a warning.
  final Color text;
  final Color muted;
  final Color accent;
  final Color warn;

  /// Between the frame of a panel and its content.
  final EdgeInsets panelPadding;

  /// Between the frame of a tile (a crew card, a ship, an achievement) and
  /// its content: tiles sit in a grid and carry no title in the corner.
  final EdgeInsets tilePadding;

  /// Between the frame of a dialog or popover and its content.
  final EdgeInsets dialogPadding;

  /// The one height of a [ListRow], with or without a second line.
  final double rowHeight;

  /// The height of a dense [ListRow] (one line, the command palette).
  final double denseRowHeight;

  /// The tokens of the FTL look: the bitmap fonts of ftl.dat, the measured
  /// colors, and system-font styles of the same size for free text.
  static const ftl = AppTokens(
    styles: {
      TextRole.display: TextStyle(fontSize: 26, fontWeight: FontWeight.bold, color: FtlColors.line),
      TextRole.title: TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: FtlColors.line),
      TextRole.label: TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: FtlColors.line),
      TextRole.body: TextStyle(fontSize: 13, color: FtlColors.line),
      TextRole.caption: TextStyle(fontSize: 12, color: FtlColors.off),
      TextRole.number: TextStyle(
        fontSize: 15,
        fontWeight: FontWeight.w600,
        color: FtlColors.line,
        fontFeatures: [FontFeature.tabularFigures()],
      ),
      TextRole.code: TextStyle(
        fontSize: 12,
        fontFamily: 'Menlo',
        fontFamilyFallback: _monospace,
        color: FtlColors.line,
      ),
    },
    bitmap: {
      TextRole.display: (FtlFontId.justin12Bold, 2),
      TextRole.title: (FtlFontId.justin12Bold, 1),
      TextRole.label: (FtlFontId.justin12Bold, 1),
      TextRole.body: (FtlFontId.justin10, 1),
      TextRole.caption: (FtlFontId.justin8, 1),
      TextRole.number: (FtlFontId.justin12Bold, 1),
    },
    text: FtlColors.line,
    muted: FtlColors.off,
    accent: FtlColors.hover,
    warn: FtlColors.warn,
    // The window is cut by 12 at its corners: a title 4 px under the line
    // stood in the corner.
    panelPadding: EdgeInsets.fromLTRB(Space.lg, Space.md, Space.lg, Space.md),
    tilePadding: EdgeInsets.all(Space.xs),
    dialogPadding: EdgeInsets.fromLTRB(Space.lg, Space.md, Space.lg, Space.md),
    rowHeight: 48,
    denseRowHeight: 40,
  );

  /// The tokens of the Material look, from the text theme and the colors of
  /// [theme].
  factory AppTokens.material(ThemeData theme) {
    // ThemeData.textTheme has no sizes until Theme.of localizes it; the
    // geometry of the typography fills them in.
    final t = theme.typography.englishLike.merge(theme.textTheme);
    final scheme = theme.colorScheme;
    const tabular = [FontFeature.tabularFigures()];
    return AppTokens(
      styles: {
        TextRole.display: t.titleLarge!,
        TextRole.title: t.titleMedium!,
        TextRole.label: t.titleSmall!,
        TextRole.body: t.bodyMedium!,
        TextRole.caption: t.bodySmall!.copyWith(color: scheme.outline),
        TextRole.number: t.titleMedium!.copyWith(fontFeatures: tabular),
        TextRole.code: t.bodySmall!.copyWith(fontFamily: 'Menlo', fontFamilyFallback: _monospace),
      },
      text: scheme.onSurface,
      muted: scheme.outline,
      accent: scheme.primary,
      warn: scheme.error,
      panelPadding: const EdgeInsets.all(Space.lg),
      tilePadding: const EdgeInsets.all(Space.md),
      dialogPadding: const EdgeInsets.fromLTRB(Space.lg, Space.md, Space.sm, Space.sm),
      rowHeight: 72,
      denseRowHeight: 44,
    );
  }

  /// The tokens of the theme above [context]; a theme without them (a test's
  /// plain ThemeData) gets the Material tokens of its own colors.
  static AppTokens of(BuildContext context) {
    final theme = Theme.of(context);
    return theme.extension<AppTokens>() ?? AppTokens.material(theme);
  }

  /// The style of [role].
  TextStyle style(TextRole role) => styles[role]!;

  @override
  AppTokens copyWith() => this;

  @override
  AppTokens lerp(AppTokens? other, double t) => t < 0.5 || other == null ? this : other;
}
