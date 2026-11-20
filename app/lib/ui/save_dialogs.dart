import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import 'components/components.dart';

/// Asks before a save while FTL runs: FTL writes the file when it quits and
/// overwrites the edit. Cancel has the focus, so Enter does not save.
/// Answers `true` only for "Save anyway".
Future<bool> confirmFtlRunning(BuildContext context) async {
  final l = AppLocalizations.of(context)!;
  final answer = await showDialog<bool>(
    context: context,
    builder: (context) => AppDialog(
      key: const ValueKey('ftl-running-dialog'),
      icon: Icons.play_circle,
      iconColor: AppTokens.of(context).warn,
      title: l.ftlRunningTitle,
      content: AppText(l.ftlRunningBody, wrap: true),
      actions: [
        AppButton(kind: ButtonKind.text, label: l.saveAnyway, onPressed: () => Navigator.pop(context, true)),
        AppButton(
          kind: ButtonKind.primary,
          label: l.cancel,
          autofocus: true,
          onPressed: () => Navigator.pop(context, false),
        ),
      ],
    ),
  );
  return answer ?? false;
}

/// Asks before a reload throws the staged edits away. Cancel has the focus,
/// so Enter keeps the edits. Answers `true` only for "Discard and reload".
Future<bool> confirmDiscardEdits(BuildContext context) async {
  final l = AppLocalizations.of(context)!;
  final answer = await showDialog<bool>(
    context: context,
    builder: (context) => AppDialog(
      key: const ValueKey('discard-edits-dialog'),
      icon: Icons.refresh,
      iconColor: AppTokens.of(context).warn,
      title: l.discardEditsTitle,
      content: AppText(l.discardEditsBody, wrap: true),
      actions: [
        AppButton(
          key: const ValueKey('discard-edits'),
          kind: ButtonKind.text,
          label: l.discardEdits,
          onPressed: () => Navigator.pop(context, true),
        ),
        AppButton(
          kind: ButtonKind.primary,
          label: l.cancel,
          autofocus: true,
          onPressed: () => Navigator.pop(context, false),
        ),
      ],
    ),
  );
  return answer ?? false;
}
