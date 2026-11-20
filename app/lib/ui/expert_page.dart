import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../shell/destination.dart';
import '../src/rust/api/editor.dart';
import '../src/rust/api/expert.dart';
import 'components/components.dart';
import 'resource_bar.dart';

/// What the Expert area shows: the whole tree, the state vars, or the
/// fields nobody knows the meaning of with the undecoded bytes.
enum ExpertView {
  dump,
  stateVars,
  unknownFields;

  /// The view of an item of the Expert menu.
  static ExpertView of(Destination item) => switch (item) {
    Destination.stateVars => stateVars,
    Destination.unknownFields => unknownFields,
    _ => dump,
  };
}

/// Icon and label of an edit of the Expert area: `raw.<path>` names its raw
/// path, `state.<id>` the state var; `null` for any other path.
(IconData, String)? expertLook(String path, AppLocalizations l) {
  if (path.startsWith('raw.')) return (Icons.data_object, l.fieldRaw(path.substring(4)));
  if (path.startsWith('state.')) return (Icons.tune, l.fieldStateVar(path.substring(6)));
  return null;
}

/// Whether every word of [query] occurs in the path, the value or the edit
/// path of [e], ignoring case.
bool expertMatches(ExpertEntry e, String query) {
  final words = query.toLowerCase().split(RegExp(r'\s+')).where((w) => w.isNotEmpty);
  final text = '${e.path} ${e.value} ${e.field?.path ?? ''}'.toLowerCase();
  return words.every(text.contains);
}

/// The entries of [view] the page lists, before the search: the tree keeps
/// every entry, the state vars their values, the unknown view the unknown
/// fields.
List<ExpertEntry> expertEntries(ExpertDump dump, ExpertView view) => switch (view) {
  ExpertView.dump => dump.entries,
  ExpertView.stateVars => [
    for (final e in dump.entries)
      if (e.field?.path.startsWith('state.') ?? false) e,
  ],
  ExpertView.unknownFields => [
    for (final e in dump.entries)
      if (e.unknown) e,
  ],
};

/// The Expert area: every decoded field of the save as the core answers it
/// (`expertOf`), searchable, with the edits of numbers and flags staged
/// like every other edit. Pure: the owner holds the edits; [onChanged]
/// stages one with the field it belongs to, so the change sheet can list it.
///
/// A warning banner says that raw edits can break the save. The tree
/// view folds records and lists; a search lists the matching entries flat
/// with their whole path. The unknown view ends with the undecoded bytes as
/// hex (read-only).
class ExpertPage extends StatefulWidget {
  const ExpertPage({super.key, required this.view, required this.dump, required this.edits, required this.onChanged});

  final ExpertView view;

  /// `null` while no file is open.
  final ExpertDump? dump;

  /// New values by path; a field without an entry shows the file value.
  final Map<String, int> edits;
  final void Function(FieldValue field, int value) onChanged;

  @override
  State<ExpertPage> createState() => _ExpertPageState();
}

class _ExpertPageState extends State<ExpertPage> {
  final _search = TextEditingController();

  /// The paths of the open records and lists of the tree.
  final Set<String> _open = {};

  static const _rowHeight = 28.0;

  @override
  void dispose() {
    _search.dispose();
    super.dispose();
  }

  /// The rows on screen: with a search the matches, flat; else the tree
  /// with the children of the closed groups left out (the other views are
  /// flat already).
  List<ExpertEntry> _rows(List<ExpertEntry> entries) {
    final query = _search.text.trim();
    if (query.isNotEmpty) return entries.where((e) => expertMatches(e, query)).toList();
    if (widget.view != ExpertView.dump) return entries;
    final rows = <ExpertEntry>[];
    int? closedDepth;
    for (final e in entries) {
      if (closedDepth != null && e.depth > closedDepth) continue;
      closedDepth = null;
      rows.add(e);
      if ((e.kind == 'record' || e.kind == 'list') && !_open.contains(e.path)) closedDepth = e.depth;
    }
    return rows;
  }

  Future<void> _edit(BuildContext rowContext, ExpertEntry e) async {
    final field = e.field;
    if (field == null) return;
    final l = AppLocalizations.of(context)!;
    final box = rowContext.findRenderObject()! as RenderBox;
    final at = box.localToGlobal(Offset.zero) & box.size;
    final v = await showResourcePopover(
      context,
      anchor: Rect.fromLTWH(at.right - 280, at.top, 280, at.height),
      field: field,
      current: widget.edits[field.path] ?? field.value,
      look: expertLook(field.path, l),
    );
    if (v != null) widget.onChanged(field, v);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final tokens = AppTokens.of(context);
    final dump = widget.dump;
    if (dump == null) return Center(child: AppText(l.expertNoFile, color: tokens.muted));
    final entries = expertEntries(dump, widget.view);
    final rows = _rows(entries);
    final searching = _search.text.trim().isNotEmpty;
    final flat = searching || widget.view != ExpertView.dump;
    final hex = widget.view == ExpertView.unknownFields;
    return Padding(
      padding: const EdgeInsets.all(Space.lg),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Panel(
            key: const ValueKey('expert-warning'),
            icon: Icons.warning_amber,
            title: l.expertWarningTitle,
            child: AppText(
              widget.view == ExpertView.unknownFields ? '${l.expertWarning} ${l.expertUnknownHint}' : l.expertWarning,
              color: tokens.warn,
              wrap: true,
            ),
          ),
          const SizedBox(height: Space.md),
          Row(
            children: [
              Expanded(
                child: AppTextField(
                  key: const ValueKey('expert-search'),
                  controller: _search,
                  search: true,
                  hint: l.expertSearch,
                  onChanged: (_) => setState(() {}),
                ),
              ),
              const SizedBox(width: Space.md),
              AppText(l.expertShown(rows.length, entries.length), role: TextRole.caption),
            ],
          ),
          const SizedBox(height: Space.md),
          Expanded(
            child: Panel(
              child: rows.isEmpty && !hex
                  ? Center(child: AppText(l.expertNoMatch, color: tokens.muted))
                  : CustomScrollView(
                      key: const ValueKey('expert-list'),
                      slivers: [
                        SliverFixedExtentList(
                          itemExtent: _rowHeight,
                          delegate: SliverChildBuilderDelegate(
                            (context, i) => _EntryRow(
                              entry: rows[i],
                              flat: flat,
                              open: _open.contains(rows[i].path),
                              tagUnknown: widget.view != ExpertView.unknownFields,
                              pending: switch (rows[i].field) {
                                final f? => widget.edits[f.path],
                                null => null,
                              },
                              onTap: (rowContext) {
                                final e = rows[i];
                                if (e.field != null) {
                                  _edit(rowContext, e);
                                } else if (!flat && (e.kind == 'record' || e.kind == 'list')) {
                                  setState(() => _open.contains(e.path) ? _open.remove(e.path) : _open.add(e.path));
                                }
                              },
                            ),
                            childCount: rows.length,
                          ),
                        ),
                        if (hex) SliverToBoxAdapter(child: _Undecoded(dump: dump)),
                      ],
                    ),
            ),
          ),
        ],
      ),
    );
  }
}

/// One entry: the chevron of a group, its name (the whole path when [flat]),
/// its type, a tag for an unknown field and its value; an edited value in
/// the accent color with a [ChangeMark].
class _EntryRow extends StatelessWidget {
  const _EntryRow({
    required this.entry,
    required this.flat,
    required this.open,
    required this.pending,
    required this.onTap,
    this.tagUnknown = true,
  });

  /// Tags an unknown field; the unknown view lists only those and leaves it out.
  final bool tagUnknown;

  final ExpertEntry entry;
  final bool flat;
  final bool open;

  /// The staged value of the entry's field, if any.
  final int? pending;
  final void Function(BuildContext rowContext) onTap;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final tokens = AppTokens.of(context);
    final e = entry;
    final group = e.kind == 'record' || e.kind == 'list';
    final field = e.field;
    final pending = this.pending;
    final direction = field != null && pending != null ? directionOf(field.value, pending) : null;
    final edited = direction != null;
    final shown = !edited
        ? e.value
        : e.kind == 'bool'
        ? '${pending != 0}'
        : '$pending';
    // A state var goes by its name, the rest by the whole path (flat) or
    // the last segment (tree).
    final stateVar = field?.path.startsWith('state.') ?? false;
    final name = stateVar && flat
        ? field!.path.substring(6)
        : flat
        ? e.path
        : e.path.split('.').last;
    final editColor = direction == EditDirection.lowered ? tokens.warn : tokens.accent;
    final clickable = field != null || (group && !flat);
    Widget row = Row(
      children: [
        if (!flat) SizedBox(width: e.depth * Space.lg),
        SizedBox(
          width: Space.lg + Space.xs,
          child: group && !flat
              ? Icon(open ? Icons.expand_more : Icons.chevron_right, size: 18, color: tokens.text)
              : null,
        ),
        Expanded(
          child: AppText(name, role: TextRole.body, color: edited ? editColor : null),
        ),
        if (e.unknown && tagUnknown) ...[
          Tag(icon: Icons.help_outline, iconColor: tokens.warn, label: l.expertUnknownTag),
          const SizedBox(width: Space.sm),
        ],
        SizedBox(
          width: 3 * Space.xl,
          child: AppText(e.kind, role: TextRole.caption),
        ),
        SizedBox(
          width: 220,
          child: Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              if (direction != null) ...[
                ChangeMark(direction, key: ValueKey('expert-mark-${e.path}')),
                const SizedBox(width: Space.xs),
              ],
              Flexible(
                child: AppText(
                  group ? (e.kind == 'list' ? '[${e.value}]' : '') : shown,
                  role: field != null ? TextRole.label : TextRole.body,
                  color: edited ? editColor : (field == null ? tokens.muted : null),
                  align: TextAlign.end,
                ),
              ),
              if (field != null) ...[const SizedBox(width: Space.xs), Icon(Icons.edit, size: 14, color: tokens.muted)],
            ],
          ),
        ),
      ],
    );
    row = Padding(
      padding: const EdgeInsets.symmetric(horizontal: Space.xs),
      child: row,
    );
    if (!clickable) return KeyedSubtree(key: ValueKey('expert-row-${e.path}'), child: row);
    return Builder(
      key: ValueKey('expert-row-${e.path}'),
      builder: (rowContext) => MouseRegion(
        cursor: SystemMouseCursors.click,
        child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => onTap(rowContext), child: row),
      ),
    );
  }
}

/// The undecoded bytes as hex, read-only, or the sentence that there are none.
class _Undecoded extends StatelessWidget {
  const _Undecoded({required this.dump});

  final ExpertDump dump;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    return Padding(
      key: const ValueKey('expert-undecoded'),
      padding: const EdgeInsets.only(top: Space.lg),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          TitleRow(icon: Icons.memory, title: l.expertUndecoded(dump.undecodedBytes)),
          const SizedBox(height: Space.sm),
          AppText(dump.undecodedHex.isEmpty ? l.expertAllDecoded : l.expertUndecodedHint, wrap: true),
          if (dump.undecodedHex.isNotEmpty) ...[
            const SizedBox(height: Space.sm),
            AppText(dump.undecodedHex.join('\n'), role: TextRole.code, wrap: true, selectable: true),
          ],
        ],
      ),
    );
  }
}
