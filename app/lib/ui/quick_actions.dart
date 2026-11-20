import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import 'command_palette.dart';
import 'components/components.dart';

/// One one-click action of the Rust core (`ftl_core::actions::Action`), by
/// the id that `plan_action` and `ftled action` take.
class QuickAction {
  const QuickAction(this.id, this.icon);

  final String id;
  final IconData icon;

  String label(AppLocalizations l) => switch (id) {
    'max-everything' => l.actionMaxEverything,
    'crew-fully-trained' => l.actionCrewFullyTrained,
    'repair-ship' => l.actionRepairShip,
    'heal-crew' => l.actionHealCrew,
    _ => id,
  };
}

/// The actions in the order of the core (`action_ids`).
const quickActions = [
  QuickAction('max-everything', Icons.keyboard_double_arrow_up),
  QuickAction('crew-fully-trained', Icons.school),
  QuickAction('repair-ship', Icons.build),
  QuickAction('heal-crew', Icons.healing),
];

/// The actions as entries of the command palette.
List<PaletteEntry> paletteActions(AppLocalizations l) => [
  for (final a in quickActions)
    PaletteEntry(kind: PaletteKind.action, id: a.id, icon: a.icon, title: a.label(l), detail: l.paletteAction),
];

/// The action buttons of the top bar: one icon button per action, its name
/// as tooltip. They sit in the top bar because an action changes several
/// areas at once (resources, systems, crew). [onRun] gets the action id;
/// `null` disables the buttons (no open file).
class QuickActionBar extends StatelessWidget {
  const QuickActionBar({super.key, required this.onRun});

  final ValueChanged<String>? onRun;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final onRun = this.onRun;
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final (i, a) in quickActions.indexed) ...[
          if (i > 0) const SizedBox(width: Space.xs),
          AppButton(
            key: ValueKey('action-${a.id}'),
            kind: ButtonKind.text,
            icon: a.icon,
            tooltip: a.label(l),
            onPressed: onRun == null ? null : () => onRun(a.id),
          ),
        ],
      ],
    );
  }
}

/// The localized text of a note of the core (`ActionNoteInfo`).
String actionNoteText(String code, int count, AppLocalizations l, {String detail = ''}) => switch (code) {
  'hull_at_editor_cap' => l.noteHullAtEditorCap(count),
  'systems_at_editor_cap' => l.noteSystemsAtEditorCap,
  'rooms_not_decoded' => l.noteRoomsNotDecoded,
  _ => detail,
};
