import 'package:material_ui/material_ui.dart';

import '../../look/ftl_look.dart';
import '../../look/ftl_theme.dart';
import 'app_text.dart';
import 'panel.dart';
import 'tokens.dart';

/// A dialog: [icon] and [title] in a [TitleRow], then [content], then
/// [actions] on the right. Centered and as wide as its content, at most
/// [maxWidth]: content that wants a width says so itself (a list in a
/// SizedBox). In the FTL look it is an FTL window, in the Material look an
/// [AlertDialog].
class AppDialog extends StatelessWidget {
  const AppDialog({
    super.key,
    required this.icon,
    this.iconColor,
    required this.title,
    required this.content,
    required this.actions,
    this.maxWidth = 640,
  });

  final IconData icon;
  final Color? iconColor;
  final String title;
  final Widget content;
  final List<Widget> actions;
  final double maxWidth;

  @override
  Widget build(BuildContext context) {
    final tokens = AppTokens.of(context);
    final look = FtlTheme.of(context);
    if (look == null) {
      return AlertDialog(
        icon: Icon(icon, color: iconColor),
        title: AppText(title, role: TextRole.title, wrap: true, align: TextAlign.center),
        content: ConstrainedBox(
          constraints: BoxConstraints(maxWidth: maxWidth),
          child: content,
        ),
        actions: actions,
      );
    }
    return Dialog(
      backgroundColor: Colors.transparent,
      elevation: 0,
      shadowColor: Colors.transparent,
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: maxWidth),
        // Sized to the content: without it the row of actions, which spans
        // the dialog, stretched the window over the whole page.
        child: IntrinsicWidth(
          child: FtlWindow(
            look: look,
            fill: FtlColors.window,
            padding: tokens.dialogPadding,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                TitleRow(icon: icon, iconColor: iconColor ?? tokens.text, title: title),
                const SizedBox(height: Space.md),
                Flexible(child: content),
                const SizedBox(height: Space.lg),
                _Actions(actions),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// A small window that edits one thing, placed by its caller (under an
/// anchor, or as a dialog): an optional [TitleRow] with [trailing], [child],
/// then [actions] on the right. [leading] replaces [icon] in the title.
class AppPopover extends StatelessWidget {
  const AppPopover({
    super.key,
    this.icon,
    this.leading,
    this.title,
    this.trailing,
    required this.child,
    this.actions = const [],
  });

  final IconData? icon;
  final Widget? leading;
  final String? title;
  final Widget? trailing;
  final Widget child;
  final List<Widget> actions;

  @override
  Widget build(BuildContext context) {
    final tokens = AppTokens.of(context);
    final title = this.title;
    final body = Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (title != null) ...[
          TitleRow(icon: icon, leading: leading, iconColor: tokens.text, title: title, trailing: trailing),
          const SizedBox(height: Space.sm),
        ],
        child,
        if (actions.isNotEmpty) ...[const SizedBox(height: Space.md), _Actions(actions)],
      ],
    );
    if (FtlTheme.of(context) case final look?) {
      return Material(
        type: MaterialType.transparency,
        child: FtlWindow(look: look, fill: FtlColors.window, padding: tokens.dialogPadding, child: body),
      );
    }
    final scheme = Theme.of(context).colorScheme;
    return Material(
      elevation: 3,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: scheme.outlineVariant),
      ),
      color: scheme.surfaceContainerHigh,
      child: Padding(padding: tokens.dialogPadding, child: body),
    );
  }
}

/// The buttons of a popover on the right. They wrap to a second line when
/// their labels are wider than the popover (German in the FTL look:
/// ABBRECHEN and ÜBERNEHMEN overflowed the 280 px value popover).
class _Actions extends StatelessWidget {
  const _Actions(this.actions);

  final List<Widget> actions;

  @override
  Widget build(BuildContext context) => Align(
    alignment: Alignment.centerRight,
    child: Wrap(alignment: WrapAlignment.end, spacing: Space.sm, runSpacing: Space.sm, children: actions),
  );
}

/// Opens [builder] (an [AppPopover], usually) under [anchor], or above it
/// when the window has no room below, kept inside the window and [width]
/// wide at most. A click beside it closes it with `null`. The page's themes
/// (the FTL look among them) come along; unlike showDialog,
/// showGeneralDialog does not carry them into its route.
Future<T?> showAnchoredPopover<T>(
  BuildContext context, {
  required Rect anchor,
  required String barrierLabel,
  required WidgetBuilder builder,
  double width = 280,
}) {
  final themes = InheritedTheme.capture(from: context, to: Navigator.of(context, rootNavigator: true).context);
  return showGeneralDialog<T>(
    context: context,
    barrierDismissible: true,
    barrierLabel: barrierLabel,
    barrierColor: Colors.transparent,
    transitionDuration: const Duration(milliseconds: 120),
    pageBuilder: (context, _, _) => themes.wrap(
      CustomSingleChildLayout(
        delegate: _AnchoredLayout(anchor, width),
        child: Builder(builder: builder),
      ),
    ),
    transitionBuilder: (context, animation, _, child) => FadeTransition(opacity: animation, child: child),
  );
}

class _AnchoredLayout extends SingleChildLayoutDelegate {
  const _AnchoredLayout(this.anchor, this.width);

  final Rect anchor;
  final double width;
  static const gap = 6.0;
  static const margin = 8.0;

  @override
  BoxConstraints getConstraintsForChild(BoxConstraints constraints) =>
      BoxConstraints.loose(Size(width, constraints.maxHeight - 2 * margin));

  @override
  Offset getPositionForChild(Size size, Size childSize) {
    final left = anchor.left.clamp(margin, (size.width - childSize.width - margin).clamp(margin, double.infinity));
    final below = anchor.bottom + gap;
    final fitsBelow = below + childSize.height <= size.height - margin;
    final top = fitsBelow ? below : anchor.top - gap - childSize.height;
    return Offset(left, top.clamp(margin, (size.height - childSize.height - margin).clamp(margin, double.infinity)));
  }

  @override
  bool shouldRelayout(_AnchoredLayout old) => old.anchor != anchor || old.width != width;
}
