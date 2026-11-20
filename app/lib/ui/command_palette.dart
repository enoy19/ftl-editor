import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../shell/destination.dart';
import '../src/rust/api/editor.dart';
import 'change_sheet.dart';
import 'components/components.dart';
import 'field_image.dart';
import 'ship_choice.dart';

/// What a palette entry stands for.
enum PaletteKind { area, field, action }

/// One line of the command palette. [id] is the area name, the field path or
/// the action id: a search key the row never shows. [detail] is the
/// localized kind of the entry ("Crew · Skill"); [area] is the page Enter
/// switches to (`null` for an action, which works on every page).
class PaletteEntry {
  const PaletteEntry({
    required this.kind,
    required this.id,
    required this.icon,
    this.image,
    required this.title,
    this.detail,
    this.area,
  });

  final PaletteKind kind;
  final String id;
  final IconData icon;

  /// The game image of a field ([fieldImage]); `null` keeps [icon].
  final FieldImage? image;
  final String title;
  final String? detail;
  final Destination? area;

  /// Every word of [query] occurs in the title, the detail or the id,
  /// ignoring case: "holly pilot" finds "Holly: Pilot".
  bool matches(String query) {
    final text = '$title ${detail ?? ''} $id'.toLowerCase();
    return query.toLowerCase().split(RegExp(r'\s+')).where((w) => w.isNotEmpty).every(text.contains);
  }
}

/// The entries and the handler a page owner gives the shell. The shell adds
/// the areas, switches the page on Enter and then calls [onPick] with a
/// field or action entry.
class PaletteSource {
  const PaletteSource({required this.entries, required this.onPick});

  final List<PaletteEntry> Function(AppLocalizations l) entries;
  final ValueChanged<PaletteEntry> onPick;
}

/// The page that shows the field at [path].
/// A field of the enemy ship lies on the same page as on the player ship.
Destination areaOf(String path) => switch (localPath(path).split('.').first) {
  'ship' || 'fires' || 'breaches' || 'rooms' => Destination.ship,
  'crew' => Destination.crew,
  'beacons' || 'fleet' => Destination.map,
  _ => Destination.equipment,
};

/// The five areas as entries.
List<PaletteEntry> paletteAreas(AppLocalizations l) => [
  for (final d in Destination.areas)
    PaletteEntry(kind: PaletteKind.area, id: d.name, icon: d.icon, title: d.label(l), detail: l.paletteArea, area: d),
];

/// The localized kind of the field at [path], for the palette row.
String fieldKind(String path, AppLocalizations l) => switch (localPath(path).split('.')) {
  ['ship', _] => l.paletteResource,
  ['crew', _, 'skills', _] => l.paletteSkill,
  ['crew', _, 'health'] => l.paletteHealth,
  ['reactor'] || ['systems', _, _] => l.paletteSystem,
  ['augments', _] => l.paletteAugment,
  ['fires', _, _] => l.paletteFire,
  ['breaches', _, _] => l.paletteBreach,
  ['rooms', _, 'oxygen'] => l.paletteOxygen,
  ['beacons', ...] || ['fleet', _] => l.paletteBeacon,
  _ => '',
};

/// Every editable field as an entry: its label from [look], its game image
/// from [image], its kind as detail; the path stays the hidden search key.
List<PaletteEntry> paletteFields(
  List<FieldValue> fields,
  FieldLook look,
  AppLocalizations l, {
  FieldImageOf image = fieldImage,
}) => [
  for (final f in fields)
    if (look(f.path, l) case (final icon, final label))
      PaletteEntry(
        kind: PaletteKind.field,
        id: f.path,
        icon: icon,
        image: image(f.path),
        title: label,
        detail: fieldKind(f.path, l),
        area: areaOf(f.path),
      ),
];

/// Cmd+K on macOS, Ctrl+K elsewhere.
SingleActivator paletteActivator() => switch (defaultTargetPlatform) {
  TargetPlatform.macOS || TargetPlatform.iOS => const SingleActivator(LogicalKeyboardKey.keyK, meta: true),
  _ => const SingleActivator(LogicalKeyboardKey.keyK, control: true),
};

/// Opens the palette over [entries]; answers the chosen entry or `null`.
Future<PaletteEntry?> showCommandPalette(BuildContext context, List<PaletteEntry> entries) => showDialog<PaletteEntry>(
  context: context,
  builder: (_) => CommandPalette(entries: entries),
);

/// A search field over [entries]: typing filters, the arrow keys move the
/// choice, Enter or a click answers it, Escape closes.
class CommandPalette extends StatefulWidget {
  const CommandPalette({super.key, required this.entries});

  final List<PaletteEntry> entries;

  @override
  State<CommandPalette> createState() => _CommandPaletteState();
}

class _CommandPaletteState extends State<CommandPalette> {
  /// The height of one row ([AppTokens.denseRowHeight]) and of the nine
  /// rows the list shows, set by [build] for the scroll math of [_move].
  double _rowHeight = 44;
  double get _listHeight => 9 * _rowHeight;

  final _query = TextEditingController();
  final _scroll = ScrollController();
  int _selected = 0;

  List<PaletteEntry> get _matches => [
    for (final e in widget.entries)
      if (e.matches(_query.text)) e,
  ];

  @override
  void dispose() {
    _query.dispose();
    _scroll.dispose();
    super.dispose();
  }

  void _move(int by) {
    final count = _matches.length;
    if (count == 0) return;
    setState(() => _selected = (_selected + by).clamp(0, count - 1));
    // Keep the chosen row inside the visible part of the list.
    final top = _selected * _rowHeight;
    if (!_scroll.hasClients) return;
    if (top < _scroll.offset) {
      _scroll.jumpTo(top);
    } else if (top + _rowHeight > _scroll.offset + _listHeight) {
      _scroll.jumpTo(top + _rowHeight - _listHeight);
    }
  }

  void _pick([PaletteEntry? entry]) {
    final matches = _matches;
    final chosen = entry ?? (matches.isEmpty ? null : matches[_selected.clamp(0, matches.length - 1)]);
    if (chosen != null) Navigator.of(context).pop(chosen);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final tokens = AppTokens.of(context);
    _rowHeight = tokens.denseRowHeight;
    final matches = _matches;
    final field = CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.arrowDown): () => _move(1),
        const SingleActivator(LogicalKeyboardKey.arrowUp): () => _move(-1),
        const SingleActivator(LogicalKeyboardKey.enter): _pick,
        const SingleActivator(LogicalKeyboardKey.numpadEnter): _pick,
      },
      child: AppTextField(
        key: const ValueKey('palette-query'),
        controller: _query,
        autofocus: true,
        search: true,
        hint: l.paletteHint,
        onChanged: (_) => setState(() => _selected = 0),
        onSubmitted: (_) => _pick(),
      ),
    );
    final list = SizedBox(
      height: _listHeight,
      child: matches.isEmpty
          ? Center(child: AppText(l.paletteNoMatch, color: tokens.muted))
          : ListView.builder(
              controller: _scroll,
              itemExtent: _rowHeight,
              itemCount: matches.length,
              itemBuilder: (context, i) {
                final e = matches[i];
                final detail = e.detail;
                return ListRow(
                  key: ValueKey('palette-${e.kind.name}-${e.id}'),
                  dense: true,
                  selected: i == _selected,
                  leading: FieldIcon(image: e.image, fallback: e.icon),
                  title: e.title,
                  trailing: detail == null ? null : AppText(detail, role: TextRole.caption),
                  onTap: () => _pick(e),
                );
              },
            ),
    );
    return Dialog(
      key: const ValueKey('command-palette'),
      alignment: Alignment.topCenter,
      // Below the top bar, so the tabs stay in view.
      insetPadding: const EdgeInsets.fromLTRB(Space.xl, kToolbarHeight + Space.lg, Space.xl, Space.xl),
      backgroundColor: Colors.transparent,
      elevation: 0,
      shadowColor: Colors.transparent,
      child: SizedBox(
        width: 560,
        child: AppPopover(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              field,
              const SizedBox(height: Space.sm),
              list,
            ],
          ),
        ),
      ),
    );
  }
}
