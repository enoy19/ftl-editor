import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../look/ftl_look.dart';
import '../look/ftl_theme.dart';
import '../src/rust/api/editor.dart';
import 'components/components.dart';
import 'field_image.dart';
import 'resource_bar.dart';

/// Icon and label of a field path.
typedef FieldLook = (IconData, String) Function(String path, AppLocalizations l);

/// One edit as the change sheet lists it: the field, its value in the file
/// and the new value.
class PendingChange {
  const PendingChange({required this.path, required this.old, required this.value});

  final String path;
  final int old;
  final int value;
}

/// One line of the change sheet. [id] names the change for [onUndo] of
/// [showChangeSheet] and for the keys `change-<id>` and `undo-<id>`.
class ChangeRow {
  const ChangeRow({required this.id, required this.icon, required this.title, this.image, this.subtitle, this.change});

  final String id;

  /// The Material icon; it stands in for [image] without `ftl.dat`.
  final IconData icon;

  /// The game image of the change ([fieldImage]); `null` keeps [icon].
  final FieldImage? image;
  final String title;
  final String? subtitle;

  /// The old and the new value of a field edit as shown (a number's
  /// digits, a crew name, a race title), drawn as [ChangeValues].
  final (String, String)? change;
}

/// The rows of field edits: label, icon, old and new value; the id is the
/// field path. [look] names a path; the default is [resourceLook]. [image]
/// answers its game image; the default is [fieldImage] without crew and
/// equipment.
List<ChangeRow> fieldChangeRows(
  List<PendingChange> changes,
  AppLocalizations l, {
  FieldLook look = resourceLook,
  FieldImageOf image = fieldImage,
}) => [for (final c in changes) _fieldRow(c, l, look, image)];

ChangeRow _fieldRow(PendingChange c, AppLocalizations l, FieldLook look, FieldImageOf image) {
  final (icon, label) = look(c.path, l);
  // The ship's beacon is an index; the map names beacons from 1.
  final shift = c.path == 'beacons.current' ? 1 : 0;
  return ChangeRow(
    id: c.path,
    icon: icon,
    image: image(c.path),
    title: label,
    change: ('${c.old + shift}', '${c.value + shift}'),
  );
}

/// The old value, an arrow and the new value. The arrow is an icon: the
/// fonts of both looks lack U+2192 and would draw a box.
class ChangeValues extends StatelessWidget {
  const ChangeValues({super.key, required this.old, required this.value, this.color});

  final String old;
  final String value;
  final Color? color;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        AppText(old, color: color),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: Space.xs),
          child: Icon(Icons.arrow_forward, size: 14, color: color),
        ),
        AppText(value, color: color),
      ],
    );
  }
}

/// The edits that differ from the file, in the order of [fields].
/// The level of a system that a staged install adds is no field of the file;
/// its edit counts from 0. (Other edits outside [fields] have rows of their
/// own: crew lists, identity, store items.)
List<PendingChange> pendingChanges(List<FieldValue> fields, Map<String, int> edits) {
  final known = {for (final f in fields) f.path};
  final newLevel = RegExp(r'^(nearby\.)?systems\.[a-z_]+\.level$');
  return [
    for (final f in fields)
      if (edits[f.path] case final v? when v != f.value) PendingChange(path: f.path, old: f.value, value: v),
    for (final MapEntry(key: path, value: v) in edits.entries)
      if (!known.contains(path) && newLevel.hasMatch(path)) PendingChange(path: path, old: 0, value: v),
  ];
}

/// Shows every change before a save. Answers `true` when the person writes
/// the file. With [onUndo] each row has an undo that drops its change at
/// once; when the last change is undone, the sheet closes with `false`.
/// Without it the rows are one edit (such as "Unlock all ships") that only
/// Cancel drops.
/// [steamCloud] adds the hint that Steam Cloud can replace the file.
/// [notes] lists what an action could not do, one info line each.
Future<bool> showChangeSheet(
  BuildContext context, {
  required List<ChangeRow> changes,
  ValueChanged<String>? onUndo,
  bool steamCloud = false,
  List<String> notes = const [],
}) async {
  // A centered modal: in the FTL look an FTL window, in the Material look a
  // Material dialog. At most 85 % of the window high; the list scrolls.
  final ftl = FtlTheme.of(context) != null;
  final write = await showDialog<bool>(
    context: context,
    builder: (context) => Dialog(
      backgroundColor: ftl ? Colors.transparent : null,
      elevation: ftl ? 0 : null,
      shadowColor: ftl ? Colors.transparent : null,
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: 640, maxHeight: MediaQuery.sizeOf(context).height * 0.85),
        child: ChangeSheet(changes: changes, onUndo: onUndo, steamCloud: steamCloud, notes: notes),
      ),
    ),
  );
  return write ?? false;
}

class ChangeSheet extends StatefulWidget {
  const ChangeSheet({super.key, required this.changes, this.onUndo, this.steamCloud = false, this.notes = const []});

  final List<ChangeRow> changes;
  final ValueChanged<String>? onUndo;
  final bool steamCloud;
  final List<String> notes;

  @override
  State<ChangeSheet> createState() => _ChangeSheetState();
}

class _ChangeSheetState extends State<ChangeSheet> {
  late final List<ChangeRow> _changes = [...widget.changes];

  void _undo(ChangeRow c, ValueChanged<String> onUndo) {
    onUndo(c.id);
    setState(() => _changes.remove(c));
    if (_changes.isEmpty) Navigator.pop(context, false);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final tokens = AppTokens.of(context);
    final onUndo = widget.onUndo;
    final body = Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        AppText(l.changesTitle, role: TextRole.title),
        const SizedBox(height: Space.xs),
        AppText(l.changesHint, wrap: true),
        const SizedBox(height: Space.sm),
        Flexible(
          child: ListView(
            shrinkWrap: true,
            children: [
              for (final c in _changes)
                // One row height whether the row has a second line or not.
                ListRow(
                  key: ValueKey('change-${c.id}'),
                  leading: FieldIcon(image: c.image, fallback: c.icon),
                  title: c.title,
                  second: switch (c.change) {
                    (final old, final value) => ChangeValues(old: old, value: value, color: tokens.muted),
                    _ => null,
                  },
                  subtitle: c.subtitle,
                  trailing: onUndo == null
                      ? null
                      : AppButton(
                          key: ValueKey('undo-${c.id}'),
                          kind: ButtonKind.text,
                          icon: Icons.undo,
                          tooltip: l.undo,
                          onPressed: () => _undo(c, onUndo),
                        ),
                ),
            ],
          ),
        ),
        for (final (i, note) in widget.notes.indexed)
          _Note(key: ValueKey('change-sheet-note-$i'), icon: Icons.info_outline, color: tokens.muted, text: note),
        if (widget.steamCloud)
          _Note(
            key: const ValueKey('change-sheet-steam'),
            icon: Icons.cloud_sync,
            color: tokens.warn,
            text: l.statusSteamCloudHint,
            textColor: tokens.warn,
          ),
        const SizedBox(height: Space.md),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            AppButton(kind: ButtonKind.text, label: l.cancel, onPressed: () => Navigator.pop(context, false)),
            const SizedBox(width: Space.sm),
            AppButton(
              key: const ValueKey('change-sheet-write'),
              kind: ButtonKind.primary,
              icon: Icons.save,
              label: l.writeFile,
              autofocus: true,
              onPressed: () => Navigator.pop(context, true),
            ),
          ],
        ),
      ],
    );
    // In the FTL look the sheet is an FTL window; in the Material look the
    // dialog is the frame.
    final look = FtlTheme.of(context);
    return SafeArea(
      child: Padding(
        key: const ValueKey('change-sheet'),
        padding: look == null ? const EdgeInsets.all(Space.xl) : EdgeInsets.zero,
        child: look == null
            ? body
            : FtlWindow(look: look, fill: FtlColors.window, padding: tokens.dialogPadding, child: body),
      ),
    );
  }
}

/// A note under the changes: an icon and a line of text.
class _Note extends StatelessWidget {
  const _Note({super.key, required this.icon, required this.color, required this.text, this.textColor});

  final IconData icon;
  final Color color;
  final String text;
  final Color? textColor;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: Space.sm),
    child: Row(
      children: [
        Icon(icon, color: color),
        const SizedBox(width: Space.md),
        Expanded(child: AppText(text, color: textColor, wrap: true)),
      ],
    ),
  );
}
