import 'dart:io';

import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../src/rust/api/crew.dart';
import '../src/rust/api/editor.dart';
import '../src/rust/api/equipment.dart';
import '../src/rust/api/floorplan.dart';
import 'components/components.dart';
import 'crew_page.dart';
import 'editor_page.dart';
import 'floorplan/floorplan_view.dart';
import 'resource_bar.dart';
import 'ship_card.dart';

/// The body of the Ship page: the open file, an error, the ship and its
/// resources. Pure: the [EditorPage] owns the file and the edits.
class ShipPage extends StatelessWidget {
  const ShipPage({
    super.key,
    required this.path,
    required this.busy,
    required this.onChoose,
    this.onReload,
    required this.error,
    required this.summary,
    required this.edits,
    required this.onChanged,
    this.floorplan,
    this.preview,
    this.previewEquipment,
    this.crew = const [],
    this.equipment,
    this.texts = const CrewTexts(),
    this.nearby = false,
    this.shipSwitch,
    this.card = const ShipCardEdits(),
    this.itemChoices = const [],
    this.onAddItem,
    this.group,
  });

  /// The identity of the ship shown and its edits, for the ship card.
  final ShipCardEdits card;

  /// The weapon slot chooser of the mounts and of the Weapon Control and
  /// Drone Control popovers: every weapon and drone of ftl.dat, how one is
  /// added, and how a replace becomes one step of Undo.
  final List<ItemChoice> itemChoices;
  final void Function(String id, {required bool toCargo})? onAddItem;
  final void Function(VoidCallback edits)? group;

  /// The choice of the ship shown, below the file and ship card and above
  /// the resources; `null` without an enemy ship.
  final Widget? shipSwitch;

  final String? path;
  final bool busy;
  final VoidCallback onChoose;

  /// Reads the open file again (the editor asks first when edits wait).
  final VoidCallback? onReload;

  /// The localized text of the last error, or `null`.
  final String? error;
  final SaveSummary? summary;
  final Map<String, int> edits;
  final void Function(String path, int value) onChanged;

  /// The ship from `ftl.dat`; `null` in reduced mode or without a decoded
  /// ship body, and the page shows no floorplan.
  final Floorplan? floorplan;

  /// The floorplan and the equipment as the pending edits leave them; the
  /// rooms show a staged install or removal from here.
  final Floorplan? preview;
  final Equipment? previewEquipment;

  /// The crew and the equipment the popovers of the floorplan edit.
  final List<CrewMember> crew;
  final Equipment? equipment;

  /// Name and race edits of the crew card in the floorplan popover.
  final CrewTexts texts;

  /// Whether the page shows the enemy ship at the beacon: its card, its
  /// resources (`SaveSummary.nearbyFields`) and the [floorplan], [crew] and
  /// [equipment] the owner reads for it.
  final bool nearby;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final summary = this.summary;
    final floorplan = this.floorplan;
    final error = this.error;
    final shipSwitch = this.shipSwitch;
    return HintedList(
      hints: [if (summary != null) HintRow(text: l.closeFtlHint)],
      children: [
        FileCard(path: path, busy: busy, onChoose: onChoose, onReload: onReload),
        if (error != null) ...[const SizedBox(height: Space.lg), ErrorCard(text: error)],
        if (summary != null) ...[
          // The switch picks the ship every card below shows, its own card included.
          if (shipSwitch != null) ...[const SizedBox(height: Space.lg), shipSwitch],
          const SizedBox(height: Space.lg),
          ShipCard(summary: summary, nearby: nearby, edits: card),
          const SizedBox(height: Space.lg),
          ResourceCard(fields: nearby ? summary.nearbyFields : summary.fields, edits: edits, onChanged: onChanged),
          if (floorplan != null) ...[
            const SizedBox(height: Space.lg),
            FloorplanCard(
              plan: floorplan,
              crew: crew,
              equipment: equipment,
              edits: edits,
              onChanged: onChanged,
              preview: preview,
              previewEquipment: previewEquipment,
              texts: texts,
              itemChoices: itemChoices,
              onAddItem: onAddItem,
              group: group,
            ),
          ],
        ],
      ],
    );
  }
}

/// A scrolling page: [children], then [hints] at the bottom of the visible
/// area while the content is shorter than it, else after the content.
class HintedList extends StatelessWidget {
  const HintedList({super.key, required this.children, this.hints = const []});

  final List<Widget> children;
  final List<Widget> hints;

  static const padding = EdgeInsets.all(Space.xl);

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) => SingleChildScrollView(
      padding: padding,
      child: ConstrainedBox(
        constraints: BoxConstraints(
          minHeight: box.hasBoundedHeight ? (box.maxHeight - padding.vertical).clamp(0, double.infinity) : 0,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: children),
            if (hints.isNotEmpty)
              Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  for (final (i, h) in hints.indexed) ...[SizedBox(height: i == 0 ? Space.lg : Space.xs), h],
                ],
              ),
          ],
        ),
      ),
    ),
  );
}

/// One line of advice under a page: an info icon and a small text.
class HintRow extends StatelessWidget {
  const HintRow({super.key, required this.text});

  final String text;

  @override
  Widget build(BuildContext context) => Row(
    children: [
      Icon(Icons.info_outline, size: 18, color: AppTokens.of(context).muted),
      const SizedBox(width: Space.sm),
      Expanded(child: AppText(text, role: TextRole.body, wrap: true)),
    ],
  );
}

/// A file in a panel: [icon], the file [name], its [folder] as a second
/// line, then [actions]. The Ship and the Profile page open their files here.
class FilePanel extends StatelessWidget {
  const FilePanel({super.key, required this.icon, required this.name, required this.folder, required this.actions});

  final IconData icon;
  final String name;
  final String? folder;
  final List<Widget> actions;

  @override
  Widget build(BuildContext context) {
    final folder = this.folder;
    return Panel(
      child: Row(
        children: [
          Icon(icon, size: 32, color: AppTokens.of(context).text),
          const SizedBox(width: Space.md),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                AppText(name, role: TextRole.label),
                if (folder != null) AppText(folder, role: TextRole.caption),
              ],
            ),
          ),
          for (final a in actions) ...[const SizedBox(width: Space.sm), a],
        ],
      ),
    );
  }
}

/// The opened file, a reload of it from disk and the action to choose another.
class FileCard extends StatelessWidget {
  const FileCard({super.key, required this.path, required this.busy, required this.onChoose, this.onReload});

  final String? path;
  final bool busy;
  final VoidCallback onChoose;

  /// Reads the open file again; `null` without a file.
  final VoidCallback? onReload;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final path = this.path;
    final file = path == null ? null : File(path);
    return FilePanel(
      icon: Icons.description_outlined,
      name: file == null ? l.noFileOpen : file.uri.pathSegments.last,
      folder: file?.parent.path,
      actions: [
        if (onReload != null)
          AppButton(
            key: const ValueKey('reload-file'),
            kind: ButtonKind.tonal,
            icon: Icons.refresh,
            tooltip: l.reloadFile,
            onPressed: busy ? null : onReload,
          ),
        AppButton(
          key: const ValueKey('choose-file'),
          kind: ButtonKind.tonal,
          icon: Icons.folder_open,
          label: l.chooseFile,
          onPressed: busy ? null : onChoose,
        ),
      ],
    );
  }
}

class ErrorCard extends StatelessWidget {
  const ErrorCard({super.key, required this.text});

  final String text;

  @override
  Widget build(BuildContext context) {
    final warn = AppTokens.of(context).warn;
    return Panel(
      child: Row(
        children: [
          Icon(Icons.error_outline, color: warn),
          const SizedBox(width: Space.md),
          Expanded(
            child: AppText(text, role: TextRole.body, color: warn, wrap: true),
          ),
        ],
      ),
    );
  }
}
