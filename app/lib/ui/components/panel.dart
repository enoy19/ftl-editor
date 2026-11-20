import 'package:material_ui/material_ui.dart';

import '../../look/ftl_look.dart';
import '../../look/ftl_theme.dart';
import 'app_text.dart';
import 'tokens.dart';

/// A title line: an optional [icon] (or a [leading] widget in its place,
/// such as a game image), the [title], an optional [note] after it and an
/// optional [trailing] at the end. The head of a [Panel], a dialog, a
/// popover or a section of a page.
class TitleRow extends StatelessWidget {
  const TitleRow({super.key, this.icon, this.leading, this.iconColor, required this.title, this.note, this.trailing});

  final IconData? icon;
  final Widget? leading;
  final Color? iconColor;
  final String title;
  final String? note;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    final icon = this.icon;
    final leading = this.leading;
    final note = this.note;
    return Row(
      children: [
        if (leading != null) ...[
          leading,
          const SizedBox(width: Space.sm),
        ] else if (icon != null) ...[
          Icon(icon, color: iconColor ?? AppTokens.of(context).text),
          const SizedBox(width: Space.sm),
        ],
        Expanded(
          child: note == null
              ? Align(
                  alignment: Alignment.centerLeft,
                  child: AppText(title, role: TextRole.title),
                )
              // The title takes its whole width first; the note gets the rest
              // and is cut before the title ('WEAPO..' beside a wide button).
              // Only with a note: a LayoutBuilder has no intrinsic width,
              // which a dialog title needs.
              : LayoutBuilder(
                  builder: (context, c) => Row(
                    children: [
                      ConstrainedBox(
                        constraints: BoxConstraints(maxWidth: c.maxWidth),
                        child: AppText(title, role: TextRole.title),
                      ),
                      Flexible(
                        child: Padding(
                          padding: const EdgeInsets.only(left: Space.md),
                          child: AppText(note, role: TextRole.caption),
                        ),
                      ),
                    ],
                  ),
                ),
        ),
        if (trailing case final trailing?) ...[const SizedBox(width: Space.sm), trailing],
      ],
    );
  }
}

/// A frame around [child]: an FTL window or a Material card, with the padding
/// of the tokens and an optional [TitleRow] (with [title]) above the child.
/// [fill] is the FTL fill (the inner box by default). A [tile] (a card in a
/// grid) has the smaller tile padding.
class Panel extends StatelessWidget {
  const Panel({
    super.key,
    this.icon,
    this.title,
    this.note,
    this.trailing,
    this.fill = FtlColors.box,
    this.tile = false,
    required this.child,
  });

  final IconData? icon;
  final String? title;
  final String? note;
  final Widget? trailing;
  final Color fill;
  final bool tile;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final tokens = AppTokens.of(context);
    final title = this.title;
    final padding = tile ? tokens.tilePadding : tokens.panelPadding;
    final body = title == null
        ? child
        : Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              TitleRow(icon: icon, title: title, note: note, trailing: trailing),
              const SizedBox(height: Space.md),
              Flexible(fit: FlexFit.loose, child: child),
            ],
          );
    if (FtlTheme.of(context) case final look?) {
      return FtlWindow(look: look, fill: fill, padding: padding, child: body);
    }
    return Card(
      margin: EdgeInsets.zero,
      clipBehavior: Clip.antiAlias,
      child: Padding(padding: padding, child: body),
    );
  }
}
