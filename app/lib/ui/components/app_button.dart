import 'package:material_ui/material_ui.dart';

import '../../look/ftl_look.dart';
import '../../look/ftl_theme.dart';
import 'tokens.dart';

/// How much a button stands out in the Material look. The FTL look has one
/// button face for all of them.
enum ButtonKind {
  /// The one main action of a page or dialog (Save, Write file).
  primary,

  /// A secondary action next to it (Choose file).
  tonal,

  /// An action on a row or card (Remove, Backups).
  outlined,

  /// A quiet action (Cancel, Close, the top-bar actions).
  text,
}

/// The one button of the pages: an [FtlButton] in the FTL look, the Material
/// button of [kind] in the Material look. With a [label] the FTL face shows
/// only the label in capitals (FTL's own buttons carry no icon next to
/// text); without one it shows [icon]. Its [key] goes to the button it
/// draws, so a test reads the FtlButton or the Material button by that key.
class AppButton extends StatelessWidget {
  // The key is named `key` for the callers but goes to the drawn button.
  // ignore: use_key_in_widget_constructors
  const AppButton({
    Key? key,
    this.label,
    this.icon,
    required this.onPressed,
    this.kind = ButtonKind.outlined,
    this.autofocus = false,
    this.tooltip,
  }) : _key = key, // ignore: prefer_initializing_formals
       assert(label != null || icon != null);

  final Key? _key;

  final String? label;
  final IconData? icon;

  /// `null` disables the button.
  final VoidCallback? onPressed;
  final ButtonKind kind;
  final bool autofocus;
  final String? tooltip;

  @override
  Widget build(BuildContext context) {
    final label = this.label;
    final icon = this.icon;
    final tooltip = this.tooltip;
    // While a theme change animates, the look and the tokens can disagree
    // for a frame: the FTL face needs both.
    final look = FtlTheme.of(context);
    final bitmap = AppTokens.of(context).bitmap[TextRole.label];
    if (look != null && bitmap != null) {
      final (id, _) = bitmap;
      return FtlButton(
        key: _key,
        font: look.font(id),
        label: label?.toUpperCase(),
        icon: label == null ? icon : null,
        tooltip: tooltip,
        autofocus: autofocus,
        onPressed: onPressed,
      );
    }
    if (label == null) {
      // Compact: icon buttons sit in rows of tags and cards.
      const density = VisualDensity.compact;
      final button = switch (kind) {
        ButtonKind.primary => IconButton.filled(
          key: _key,
          visualDensity: density,
          icon: Icon(icon),
          tooltip: tooltip,
          autofocus: autofocus,
          onPressed: onPressed,
        ),
        ButtonKind.tonal || ButtonKind.outlined => IconButton.filledTonal(
          key: _key,
          visualDensity: density,
          icon: Icon(icon),
          tooltip: tooltip,
          autofocus: autofocus,
          onPressed: onPressed,
        ),
        ButtonKind.text => IconButton(
          key: _key,
          visualDensity: density,
          icon: Icon(icon),
          tooltip: tooltip,
          autofocus: autofocus,
          onPressed: onPressed,
        ),
      };
      return button;
    }
    final text = Text(label);
    final Widget button = switch ((kind, icon)) {
      (ButtonKind.primary, final i?) => FilledButton.icon(
        key: _key,
        autofocus: autofocus,
        onPressed: onPressed,
        icon: Icon(i),
        label: text,
      ),
      (ButtonKind.primary, null) => FilledButton(key: _key, autofocus: autofocus, onPressed: onPressed, child: text),
      (ButtonKind.tonal, final i?) => FilledButton.tonalIcon(
        key: _key,
        autofocus: autofocus,
        onPressed: onPressed,
        icon: Icon(i),
        label: text,
      ),
      (ButtonKind.tonal, null) => FilledButton.tonal(
        key: _key,
        autofocus: autofocus,
        onPressed: onPressed,
        child: text,
      ),
      (ButtonKind.outlined, final i?) => OutlinedButton.icon(
        key: _key,
        autofocus: autofocus,
        onPressed: onPressed,
        icon: Icon(i),
        label: text,
      ),
      (ButtonKind.outlined, null) => OutlinedButton(key: _key, autofocus: autofocus, onPressed: onPressed, child: text),
      (ButtonKind.text, final i?) => TextButton.icon(
        key: _key,
        autofocus: autofocus,
        onPressed: onPressed,
        icon: Icon(i),
        label: text,
      ),
      (ButtonKind.text, null) => TextButton(key: _key, autofocus: autofocus, onPressed: onPressed, child: text),
    };
    return tooltip == null ? button : Tooltip(message: tooltip, child: button);
  }
}
