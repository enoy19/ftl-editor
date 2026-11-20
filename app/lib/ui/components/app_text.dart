import 'package:material_ui/material_ui.dart';

import '../../look/ftl_look.dart';
import '../../look/ftl_theme.dart';
import 'tokens.dart';

/// The one text widget of the pages: [role] picks the font of the look.
///
/// In the FTL look the text is bitmap text ([FtlText]) in the font of the
/// role, unless a sign of it has no glyph (a path, a sign the fonts lack):
/// then it is system text in the size and color of the role. In the Material
/// look it is a [Text] in the style of the role. A page never names a font.
class AppText extends StatelessWidget {
  const AppText(
    this.text, {
    super.key,
    this.role = TextRole.body,
    this.color,
    this.caps,
    this.wrap = false,
    this.align = TextAlign.start,
    this.selectable = false,
  });

  final String text;
  final TextRole role;

  /// Overrides the color of the role (a marked value, a warning).
  final Color? color;

  /// Capitals in the FTL look, as FTL writes titles, names and labels; the
  /// Material look keeps the case. `null`: only [TextRole.title].
  final bool? caps;

  /// Breaks into lines to fit the width; else one line, cut at the sides.
  final bool wrap;
  final TextAlign align;

  /// The person can select and copy system text (a path). Bitmap text cannot
  /// be selected.
  final bool selectable;

  @override
  Widget build(BuildContext context) {
    final tokens = AppTokens.of(context);
    final style = tokens.style(role);
    final color = this.color ?? style.color ?? tokens.text;
    final look = FtlTheme.of(context);
    final bitmap = tokens.bitmap[role];
    if (look != null && bitmap != null) {
      final shown = (caps ?? role == TextRole.title) ? text.toUpperCase() : text;
      final (id, scale) = bitmap;
      final font = look.font(id);
      if (font.covers(shown.replaceAll('\n', ''))) {
        final ftl = FtlText(shown, font: font, scale: scale, color: color, wrap: wrap, align: align);
        // Only the sides: the dots of Ä, Ö and Ü stand above the line box.
        return wrap ? ftl : ClipRect(clipper: const FtlSideClip(), child: ftl);
      }
      return _system(shown, style, color);
    }
    return _system(text, style, color);
  }

  Widget _system(String text, TextStyle style, Color color) => selectable
      ? SelectableText(
          text,
          style: style.copyWith(color: color),
          textAlign: align,
          maxLines: wrap ? null : 1,
        )
      : Text(
          text,
          style: style.copyWith(color: color),
          textAlign: align,
          maxLines: wrap ? null : 1,
          softWrap: wrap,
          overflow: wrap ? null : TextOverflow.ellipsis,
        );
}
