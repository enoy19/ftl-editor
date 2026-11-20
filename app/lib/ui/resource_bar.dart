import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../look/ftl_look.dart';
import '../look/ftl_theme.dart';
import 'components/components.dart';
import '../src/rust/api/editor.dart';
import 'field_image.dart';
import 'game_image.dart';
import 'ship_choice.dart';

/// Icon and label of a resource path of either ship, as FTL shows them in
/// its top bar.
(IconData, String) resourceLook(String path, AppLocalizations l) => switch (localPath(path)) {
  'ship.hull' => (Icons.favorite, l.fieldHull),
  'ship.fuel' => (Icons.local_gas_station, l.fieldFuel),
  'ship.drone_parts' => (Icons.precision_manufacturing, l.fieldDroneParts),
  'ship.missiles' => (Icons.rocket, l.fieldMissiles),
  'ship.scrap' => (Icons.hardware, l.fieldScrap),
  _ => (Icons.tune, path),
};

/// FTL's own icon of a resource in `ftl.dat`, drawn at its own size (factor
/// 1) in a box of [resourceIconSize]; `null` for a path without one.
///
/// Fuel, drone parts, missiles and scrap are the 32x32 white icons of
/// `img/ui_icons/`. FTL has no hull icon (its top bar writes HULL on
/// `top_hull_label.png`), so the hull takes the player ship marker of the map,
/// `img/map/map_icon_ship.png` (64x64, the ship 16x25 in the middle), cropped
/// to the box. docs/original-look.md lists the paths.
GameImageRef? resourceImage(String path) => switch (localPath(path)) {
  'ship.hull' => const GameImageRef.path('img/map/map_icon_ship.png'),
  'ship.fuel' => const GameImageRef.path('img/ui_icons/icon_fuel.png'),
  'ship.drone_parts' => const GameImageRef.path('img/ui_icons/icon_drones.png'),
  'ship.missiles' => const GameImageRef.path('img/ui_icons/icon_missiles.png'),
  'ship.scrap' => const GameImageRef.path('img/ui_icons/icon_scrap.png'),
  _ => null,
};

/// The box of a resource icon: the size of FTL's `ui_icons`.
const resourceIconSize = 32.0;

/// The icon of a resource: FTL's own from the open `ftl.dat`
/// ([resourceImage]) in a box of [resourceIconSize], else (reduced mode, or
/// an image that cannot be read) the Material icon of [resourceLook]. The
/// game icons are white, so [color] tints them like the Material icon.
class ResourceIcon extends StatelessWidget {
  const ResourceIcon({super.key, required this.path, required this.fallback, required this.color});

  final String path;
  final IconData fallback;
  final Color color;

  @override
  Widget build(BuildContext context) {
    final ref = resourceImage(path);
    final icon = Icon(fallback, size: 20, color: color);
    if (ref == null || GameDataScope.maybeOf(context)?.datPath == null) return icon;
    return GameImage(
      ref,
      fallback: fallback,
      width: resourceIconSize,
      height: resourceIconSize,
      iconSize: 20,
      color: color,
      fit: BoxFit.none,
    );
  }
}

/// The resources of the ship as one compact row of icon and number.
///
/// A click on a resource opens a small popover to edit it. The popover
/// accepts only a whole number inside the range the core answers, so every
/// value in [edits] is valid. [edits] holds only values that differ from the
/// file; [onChanged] reports a new value, and the owner drops it again when
/// it equals the file value.
class ResourceBar extends StatelessWidget {
  const ResourceBar({super.key, required this.fields, required this.edits, required this.onChanged});

  final List<FieldValue> fields;

  /// New values by path; a path without an entry shows the file value.
  final Map<String, int> edits;

  final void Function(String path, int value) onChanged;

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: Space.sm,
      runSpacing: Space.sm,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        for (final f in fields) _ResourceChip(field: f, value: edits[f.path], onChanged: (v) => onChanged(f.path, v)),
      ],
    );
  }
}

class _ResourceChip extends StatelessWidget {
  const _ResourceChip({required this.field, required this.value, required this.onChanged});

  final FieldValue field;

  /// The edited value, or `null` when the file value stands.
  final int? value;
  final ValueChanged<int> onChanged;

  Future<void> _edit(BuildContext context) async {
    final box = context.findRenderObject()! as RenderBox;
    final anchor = box.localToGlobal(Offset.zero) & box.size;
    final v = await showResourcePopover(context, anchor: anchor, field: field, current: value ?? field.value);
    if (v != null) onChanged(v);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final scheme = Theme.of(context).colorScheme;
    final (icon, label) = resourceLook(field.path, l);
    final edited = value != null;
    final tooltip = edited ? l.resourceEdited(label, field.value) : l.resourceTooltip(label, field.min, field.max);
    if (FtlTheme.of(context) != null) {
      return Tooltip(
        message: tooltip,
        child: MouseRegion(
          cursor: SystemMouseCursors.click,
          child: GestureDetector(
            key: ValueKey('resource-${field.path}'),
            onTap: () => _edit(context),
            child: _FtlResource(path: field.path, icon: icon, value: value ?? field.value, edited: edited),
          ),
        ),
      );
    }
    return Tooltip(
      message: tooltip,
      child: Material(
        key: ValueKey('resource-${field.path}'),
        color: edited ? scheme.primaryContainer : scheme.surfaceContainerHigh,
        shape: StadiumBorder(side: BorderSide(color: edited ? scheme.primary : scheme.outlineVariant)),
        clipBehavior: Clip.antiAlias,
        child: InkWell(
          onTap: () => _edit(context),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: Space.md, vertical: Space.sm),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                ResourceIcon(
                  path: field.path,
                  fallback: icon,
                  color: edited ? scheme.primary : scheme.onSurfaceVariant,
                ),
                const SizedBox(width: Space.sm),
                AppText(
                  '${value ?? field.value}',
                  role: TextRole.number,
                  color: edited ? scheme.onPrimaryContainer : null,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// Size of every resource box in the FTL look: one height for the row that
/// holds a [resourceIconSize] icon inside the 2 px outline, one width that
/// holds the icon and five digits (scrap goes up to 99999).
const ftlResourceSize = Size(108, 36);

/// One resource in the FTL look: an [FtlBox] of [ftlResourceSize] with icon
/// and number, yellow when the value is edited. All five resources share it;
/// FTL's own status boxes (img/statusUI/top_*) differ in size and style.
class _FtlResource extends StatelessWidget {
  const _FtlResource({required this.path, required this.icon, required this.value, required this.edited});

  final String path;
  final IconData icon;
  final int value;
  final bool edited;

  @override
  Widget build(BuildContext context) {
    final color = edited ? FtlColors.hover : FtlColors.line;
    return SizedBox.fromSize(
      size: ftlResourceSize,
      child: FtlBox(
        outline: color,
        padding: const EdgeInsets.symmetric(horizontal: Space.xs),
        child: Row(
          children: [
            ResourceIcon(path: path, fallback: icon, color: color),
            const SizedBox(width: Space.xs),
            Expanded(
              child: AppText('$value', role: TextRole.number, color: color),
            ),
          ],
        ),
      ),
    );
  }
}

/// Opens the popover under [anchor]; answers the accepted value or `null`
/// when the person dismisses it. [look] names the field; the default is
/// [resourceLook]. [image] is its game image in the title; the default is
/// [fieldImage] of the path (a resource, a system, a skill).
Future<int?> showResourcePopover(
  BuildContext context, {
  required Rect anchor,
  required FieldValue field,
  required int current,
  (IconData, String)? look,
  FieldImage? image,
}) {
  final l = AppLocalizations.of(context)!;
  return showAnchoredPopover<int>(
    context,
    anchor: anchor,
    barrierLabel: l.cancel,
    // Wide enough for a crew name with its skill in the title and the two
    // German buttons (ABBRECHEN, ÜBERNEHMEN) on one line in the FTL look.
    width: 360,
    builder: (_) =>
        _ResourcePopover(field: field, current: current, look: look, image: image ?? fieldImage(field.path)),
  );
}

class _ResourcePopover extends StatefulWidget {
  const _ResourcePopover({required this.field, required this.current, this.look, this.image});

  final FieldValue field;
  final int current;
  final (IconData, String)? look;
  final FieldImage? image;

  @override
  State<_ResourcePopover> createState() => _ResourcePopoverState();
}

class _ResourcePopoverState extends State<_ResourcePopover> {
  late final _ctrl = TextEditingController(text: '${widget.current}')
    ..selection = TextSelection(baseOffset: 0, extentOffset: '${widget.current}'.length);
  String? _error;

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  void _apply() {
    final l = AppLocalizations.of(context)!;
    final f = widget.field;
    final v = int.tryParse(_ctrl.text.trim());
    final error = v == null
        ? l.notANumber
        : v < f.min || v > f.max
        ? l.outOfRange(f.min, f.max)
        : null;
    if (error != null) {
      setState(() => _error = error);
      return;
    }
    Navigator.of(context).pop(v);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final f = widget.field;
    final (icon, label) = widget.look ?? resourceLook(f.path, l);
    final field = AppTextField(
      controller: _ctrl,
      autofocus: true,
      keyboardType: TextInputType.number,
      helper: '${f.min}–${f.max}',
      error: _error,
      onChanged: (_) {
        if (_error != null) setState(() => _error = null);
      },
      onSubmitted: (_) => _apply(),
    );
    final max = AppButton(
      icon: Icons.keyboard_double_arrow_up,
      kind: ButtonKind.text,
      tooltip: l.setToMax(f.max),
      onPressed: () => setState(() {
        _ctrl.text = '${f.max}';
        _error = null;
      }),
    );
    return AppPopover(
      key: const ValueKey('resource-popover'),
      icon: icon,
      leading: FieldIcon(image: widget.image, fallback: icon, color: AppTokens.of(context).text),
      title: label,
      trailing: max,
      actions: [
        AppButton(kind: ButtonKind.text, label: l.cancel, onPressed: () => Navigator.of(context).pop()),
        AppButton(kind: ButtonKind.tonal, label: l.apply, onPressed: _apply),
      ],
      child: field,
    );
  }
}

/// The resource bar in a card with its title, for the Ship page.
class ResourceCard extends StatelessWidget {
  const ResourceCard({super.key, required this.fields, required this.edits, required this.onChanged});

  /// The resources of the ship shown (`SaveSummary.fields` or `nearbyFields`).
  final List<FieldValue> fields;
  final Map<String, int> edits;
  final void Function(String path, int value) onChanged;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final bar = ResourceBar(fields: fields, edits: edits, onChanged: onChanged);
    return Panel(
      child: Row(
        children: [
          Icon(Icons.inventory_2, color: AppTokens.of(context).text),
          const SizedBox(width: Space.sm),
          AppText(l.resources, role: TextRole.title),
          const SizedBox(width: Space.xl),
          Expanded(child: bar),
        ],
      ),
    );
  }
}
