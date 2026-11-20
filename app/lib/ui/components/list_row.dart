import 'package:material_ui/material_ui.dart';

import '../../look/ftl_theme.dart';
import 'app_text.dart';
import 'tokens.dart';

/// One row of a list: [leading], the [title], an optional second line
/// ([subtitle] as text or [second] as a widget) and [trailing]. The row has
/// one height whether the second line exists or not ([AppTokens.rowHeight],
/// or [AppTokens.denseRowHeight] when [dense]), so rows of a list line up.
/// [selected] and [marked] draw the title in the accent color; [selected]
/// also frames the row in the FTL look.
class ListRow extends StatelessWidget {
  const ListRow({
    super.key,
    this.leading,
    required this.title,
    this.subtitle,
    this.second,
    this.trailing,
    this.onTap,
    this.selected = false,
    this.marked = false,
    this.dense = false,
    this.caps = false,
    this.muted = false,
  });

  final Widget? leading;
  final String title;
  final String? subtitle;
  final Widget? second;
  final Widget? trailing;
  final VoidCallback? onTap;
  final bool selected;
  final bool marked;
  final bool dense;

  /// The title in capitals in the FTL look (FTL's item names).
  final bool caps;

  /// The title in the muted color: a placeholder row, such as an empty slot.
  final bool muted;

  @override
  Widget build(BuildContext context) {
    final tokens = AppTokens.of(context);
    final height = dense ? tokens.denseRowHeight : tokens.rowHeight;
    final subtitle = this.subtitle;
    final secondLine = second ?? (subtitle == null ? null : AppText(subtitle, role: TextRole.caption));
    final titleColor = selected || marked
        ? tokens.accent
        : muted
        ? tokens.muted
        : null;
    final title = AppText(this.title, role: TextRole.label, color: titleColor, caps: caps);
    if (FtlTheme.of(context) == null) {
      return ListTile(
        dense: dense,
        minTileHeight: height,
        selected: selected,
        leading: leading,
        title: title,
        subtitle: secondLine == null ? null : Align(alignment: AlignmentDirectional.centerStart, child: secondLine),
        trailing: trailing,
        onTap: this.onTap,
      );
    }
    Widget row = SizedBox(
      height: height,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: Space.sm),
        child: Row(
          children: [
            if (leading case final leading?) ...[
              IconTheme.merge(
                data: IconThemeData(color: selected ? tokens.accent : tokens.text),
                child: leading,
              ),
              const SizedBox(width: Space.md),
            ],
            Expanded(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [title, ?secondLine],
              ),
            ),
            if (trailing case final trailing?) ...[const SizedBox(width: Space.sm), trailing],
          ],
        ),
      ),
    );
    if (selected) row = FtlBox(outline: tokens.accent, child: row);
    final onTap = this.onTap;
    if (onTap == null) return row;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: onTap, child: row),
    );
  }
}
