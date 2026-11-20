import 'package:material_ui/material_ui.dart';

import '../../look/ftl_theme.dart';
import 'app_text.dart';
import 'tokens.dart';

/// A small fact as a tag: an optional [icon], then [label]. With [onTap] it
/// is a button; [marked] draws it in the accent color (an edited value). An
/// [FtlBox] in the FTL look, a [Chip] in the Material look.
class Tag extends StatelessWidget {
  const Tag({super.key, required this.label, this.icon, this.iconColor, this.marked = false, this.onTap, this.tooltip});

  final String label;
  final IconData? icon;
  final Color? iconColor;
  final bool marked;
  final VoidCallback? onTap;
  final String? tooltip;

  @override
  Widget build(BuildContext context) {
    final tokens = AppTokens.of(context);
    final icon = this.icon;
    final onTap = this.onTap;
    final tooltip = this.tooltip;
    if (FtlTheme.of(context) == null) {
      final scheme = Theme.of(context).colorScheme;
      final avatar = icon == null ? null : Icon(icon, size: 16, color: iconColor);
      final text = AppText(label, role: TextRole.body);
      if (onTap == null) {
        final chip = Chip(
          visualDensity: VisualDensity.compact,
          materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
          avatar: avatar,
          label: text,
        );
        return tooltip == null ? chip : Tooltip(message: tooltip, child: chip);
      }
      return ActionChip(
        visualDensity: VisualDensity.compact,
        materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
        tooltip: tooltip,
        side: marked ? BorderSide(color: scheme.primary) : null,
        backgroundColor: marked ? scheme.primaryContainer : null,
        avatar: avatar,
        label: text,
        onPressed: onTap,
      );
    }
    final color = marked ? tokens.accent : tokens.text;
    Widget tag = FtlBox(
      outline: marked ? tokens.accent : tokens.muted,
      padding: const EdgeInsets.symmetric(horizontal: Space.sm - 2, vertical: Space.xs - 2),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (icon != null) ...[Icon(icon, size: 14, color: iconColor ?? color), const SizedBox(width: Space.xs)],
          AppText(label, role: TextRole.body, color: color),
        ],
      ),
    );
    if (onTap != null) {
      tag = MouseRegion(
        cursor: SystemMouseCursors.click,
        child: GestureDetector(onTap: onTap, child: tag),
      );
    }
    return tooltip == null ? tag : Tooltip(message: tooltip, child: tag);
  }
}
