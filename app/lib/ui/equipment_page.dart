// The Equipment page (variant C of the gallery, picked on
// 2026-11-18): one row per system with a segment per level, the reactor as
// the first row, weapons, drones, cargo and augments as lists on the right.
// Pure: the owner holds the edits, which are the same pending edits as on
// the Ship and Crew pages, so Save lists them in the change sheet. Weapons,
// drones and cargo show the core's preview of the edits (`equipmentPreview`).

import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../look/ftl_look.dart';
import '../look/ftl_theme.dart';
import '../src/rust/api/assets.dart';
import '../src/rust/api/editor.dart';
import '../src/rust/api/equipment.dart';
import 'components/components.dart';
import 'field_image.dart';
import 'game_image.dart';
import 'resource_bar.dart';
import 'ship_choice.dart';

/// The segment width of the reactor (25 levels) and of a system (8 at most).
const _reactorCell = 8.0;
const _systemCell = 22.0;

/// Green of a powered segment, from FTL's green system icons
/// (img/icons/s_*_green1.png: #47F443 to #99FD96).
const powerGreen = Color(0xFF66F962);

/// A broken bar: FTL's `COLOR_RED` (255, 50, 50), with which
/// `ShipSystem::RenderPowerBoxes` fills a damaged bar (FTL 1.6.13, static
/// initializer of ShipSystem.cpp; see `ftl_core::damage`).
const damageRed = Color(0xFFFF3232);

/// FTL's `COLOR_BLUE_ION` (133, 231, 237): the outline around the bars of a
/// system that ion damage locks.
const ionBlue = Color(0xFF85E7ED);

/// FTL's `COLOR_GREEN` (100, 255, 100): a Zoltan shield point. The player's
/// `img/statusUI/energy_shield_bar.png` (and `_bar2`) is this one color, and
/// the enemy box fills its bar with it (see `ftl_core::energy_shield`).
const zoltanGreen = Color(0xFF64FF64);

/// The lock FTL draws beside the bars of an ion-locked system; `white` for
/// a lock without a count (-1, mind control or hacking).
GameImageRef ionLock({bool white = false}) => GameImageRef.path('img/icons/locking/s_lock${white ? '_white' : ''}.png');

/// The FTL icon of a system in a state: a disc with the symbol cut out and a
/// glow, 64x64 (`img/icons/s_<id>_<state>.png`). FTL draws these images as
/// they are, one per state, with no tint and no second layer
/// (`ShipSystem::RenderSystemSymbol`, see docs/original-look.md).
GameImageRef systemIcon(String id, String state) => GameImageRef.path('img/icons/s_${id}_$state.png');

/// The state image of [s] as FTL picks it: `green` installed (the upgrade
/// screen always draws green), `orange` with some bars damaged, `red` with
/// every bar damaged, `grey` not installed. The pointer takes the lighter
/// frame `2` of the same color.
/// [installed] overrides the file: a staged install draws green, a staged
/// removal grey. [level] and [damaged] override the file's level and broken
/// bars with pending values.
String systemIconState(EquipmentSystem s, {bool hovered = false, bool? installed, int? level, int? damaged}) {
  final lvl = installed == false ? 0 : (level ?? s.level?.value ?? (installed == true ? 1 : 0));
  final broken = damaged ?? s.damaged;
  final color = lvl == 0
      ? 'grey'
      : broken >= lvl
      ? 'red'
      : broken > 0
      ? 'orange'
      : 'green';
  return '$color${hovered ? 2 : 1}';
}

/// Every editable field of [e]: the reactor, the level of each installed
/// system and whether each system is installed, the count of each augment,
/// the slot, armed and cargo counts of each weapon and drone id.
List<FieldValue> equipmentFields(Equipment? e) => e == null
    ? const []
    : [
        e.reactor,
        for (final s in e.systems) ?s.level,
        for (final s in e.systems) ?s.damage,
        for (final s in e.systems) ?s.ion,
        for (final s in e.systems) ?s.energy,
        for (final s in e.systems) s.installed,
        for (final a in e.augments) a.count,
        for (final f in e.itemFields) ...itemFieldValues(f),
      ];

/// The fields of one weapon or drone id that exist.
List<FieldValue> itemFieldValues(EquipmentItemFields f) => [?f.slots, ?f.armed, ?f.cargo];

/// Icon and label of a weapon, drone or cargo path for the change sheet;
/// `null` for any other path. Names come from [equipment] or the picker
/// [choices].
(IconData, String)? itemLook(String path, AppLocalizations l, Equipment? equipment, List<ItemChoice> choices) {
  String name(String id) =>
      [
        ...?equipment?.weapons,
        ...?equipment?.drones,
        ...?equipment?.cargo,
      ].where((i) => i.id == id).firstOrNull?.name ??
      choices.where((c) => c.id == id).firstOrNull?.name ??
      id;
  return switch (localPath(path).split('.')) {
    ['weapons', final id] => (Icons.gps_fixed, name(id)),
    ['drones', final id] => (Icons.precision_manufacturing, name(id)),
    ['weapons' || 'drones', final id, 'armed'] => (Icons.bolt, l.equipPowered(name(id))),
    ['cargo', final id] => (Icons.inventory_2, l.equipInCargo(name(id))),
    _ => null,
  };
}

/// Icon and label of an equipment path for the change sheet; `null` for
/// any other path.
(IconData, String)? equipmentLook(String path, AppLocalizations l, Equipment? equipment, List<AugmentChoice> choices) {
  final parts = localPath(path).split('.');
  if (parts case ['reactor']) return (Icons.bolt, l.equipReactor);
  if (parts case ['systems', final id, 'level']) {
    final name = equipment?.systems.where((s) => s.id == id).firstOrNull?.name;
    return (Icons.settings, name ?? id);
  }
  if (parts case ['systems', final id, 'damage']) {
    final name = equipment?.systems.where((s) => s.id == id).firstOrNull?.name;
    return (Icons.build_circle_outlined, l.equipSystemDamage(name ?? id));
  }
  if (parts case ['systems', final id, 'ion']) {
    final name = equipment?.systems.where((s) => s.id == id).firstOrNull?.name;
    return (Icons.lock_outline, l.equipSystemIon(name ?? id));
  }
  if (parts case ['systems', final id, 'energy']) {
    final name = equipment?.systems.where((s) => s.id == id).firstOrNull?.name;
    return (Icons.shield_outlined, l.equipZoltanShield(name ?? id));
  }
  if (parts case ['systems', final id, 'installed']) {
    final name = equipment?.systems.where((s) => s.id == id).firstOrNull?.name;
    return (Icons.build, l.equipSystemInstalled(name ?? id));
  }
  if (parts case ['augments', final id]) return (Icons.extension, _augmentName(id, equipment, choices));
  return null;
}

/// Why a system cannot be installed or removed, for the code of the core
/// (`ftl_core::installs::InstallBlock::code`).
String installBlockText(String code, AppLocalizations l) => switch (code) {
  'no_game_data' => l.equipBlockNoGameData,
  'no_room' => l.equipBlockNoRoom,
  'not_in_format' => l.equipBlockNotInFormat,
  'artillery' => l.equipBlockArtillery,
  'system_limit' => l.equipBlockSystemLimit,
  'subsystem_limit' => l.equipBlockSubsystemLimit,
  'starting_system' => l.equipBlockStartingSystem,
  'swap_back' => l.equipBlockSwapBack,
  'items_in_slots' => l.equipBlockItemsInSlots,
  'no_station' => l.equipBlockNoStation,
  _ => l.equipBlockNotDecoded,
};

String _augmentName(String id, Equipment? equipment, List<AugmentChoice> choices) =>
    equipment?.augments.where((a) => a.id == id).firstOrNull?.name ??
    choices.where((c) => c.id == id).firstOrNull?.name ??
    id;

class EquipmentPage extends StatefulWidget {
  const EquipmentPage({
    super.key,
    required this.equipment,
    required this.edits,
    required this.onChanged,
    required this.addedAugments,
    required this.choices,
    required this.onAddAugment,
    this.preview,
    this.previewError,
    this.itemChoices = const [],
    this.onAddItem,
    this.hovered,
    this.group,
  });

  /// `null` while no file is open.
  final Equipment? equipment;

  /// New values by path; a path without an entry shows the file value.
  final Map<String, int> edits;
  final void Function(String path, int value) onChanged;

  /// `augments.<ID>` of augments the ship did not carry when the file was
  /// read and that the person added since (`augmentField` of the core).
  final List<FieldValue> addedAugments;

  /// Every augment of the open ftl.dat, for the picker.
  final List<AugmentChoice> choices;

  /// The person picked an augment to add.
  final ValueChanged<String> onAddAugment;

  /// The equipment as the pending edits leave it (`equipmentPreview` of
  /// the core); `null` shows [equipment].
  final Equipment? preview;

  /// Why the core refused the pending edits for the preview.
  final String? previewError;

  /// Every weapon and drone of the open ftl.dat, for the pickers.
  final List<ItemChoice> itemChoices;

  /// The person picked a weapon or drone to add to the slots, or with
  /// `toCargo` to the cargo.
  final void Function(String id, {required bool toCargo})? onAddItem;

  /// The system id the pointer rests on at start (for shots).
  final String? hovered;

  /// Runs the edits of one replace (a removal and an addition) as one step
  /// of Undo; `null` runs them one by one.
  final void Function(VoidCallback edits)? group;

  @override
  State<EquipmentPage> createState() => _EquipmentPageState();
}

class _EquipmentPageState extends State<EquipmentPage> {
  late String? _hovered = widget.hovered;

  int _value(FieldValue f) => widget.edits[f.path] ?? f.value;

  /// The chooser of an augment slot: [current] is the augment of the slot
  /// clicked, `''` for an empty slot, `null` for Add. Answers the id, `''`
  /// for "Empty slot", `null` when dismissed.
  Future<String?> _pickAugment(BuildContext context, {required String title, String? current}) {
    final muted = AppTokens.of(context).muted;
    return showDialog<String>(
      context: context,
      builder: (_) => ChoicePicker(
        keyPrefix: 'augment',
        icon: Icons.extension,
        title: title,
        current: current,
        choices: [
          for (final c in widget.choices)
            PickerChoice(c.id, c.name, c.id, Icon(Icons.extension, size: 24, color: muted)),
        ],
      ),
    );
  }

  void _replace(String? chosen, {required String? current, VoidCallback? remove, required ValueChanged<String> add}) =>
      stageReplace(widget.group, chosen, current: current, remove: remove, add: add);

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final e = widget.equipment;
    if (e == null) return const SizedBox.shrink();

    // Both lists build every row: they are short, and a lazy list would
    // leave the rows below a small window unbuilt.
    Widget list(Key key, List<Widget> children) => SingleChildScrollView(
      key: key,
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: children),
    );

    // Every row stacks once the widest no longer fits beside its name, so
    // the bars of all rows keep one column and their segment size.
    final wide = [
      SystemLevelRow.wideWidth(e.reactor.max, _reactorCell, action: false),
      for (final s in e.systems) SystemLevelRow.wideWidth(s.level?.max ?? 3, _systemCell, action: true),
    ].reduce((a, b) => a > b ? a : b);
    final systems = LayoutBuilder(
      builder: (context, c) {
        final stacked = c.maxWidth < wide;
        return list(const ValueKey('equip-systems'), [
          _Heading(title: l.equipSystems, note: l.equipRowHint),
          SystemLevelRow(
            id: 'reactor',
            icon: null,
            title: l.equipReactor,
            field: e.reactor,
            value: _value(e.reactor),
            cell: _reactorCell,
            stacked: stacked,
            onSet: (v) => widget.onChanged(e.reactor.path, v),
          ),
          const SizedBox(height: Space.sm),
          for (final s in e.systems)
            SystemRow(
              system: s,
              equipment: e,
              preview: widget.preview,
              edits: widget.edits,
              onChanged: widget.onChanged,
              stacked: stacked,
              hovered: _hovered == s.id,
              onHover: (h) => setState(() => _hovered = h ? s.id : (_hovered == s.id ? null : _hovered)),
            ),
        ]);
      },
    );

    // The augments the ship carries after the edits, in file order, then the added ones.
    final augments = [
      for (final a in e.augments) (a.count, a.name ?? _augmentName(a.id, e, widget.choices)),
      for (final f in widget.addedAugments)
        if (!e.augments.any((a) => a.count.path == f.path))
          (f, _augmentName(f.path.split('.').last, e, widget.choices)),
    ];
    final used = augments.fold(0, (n, a) => n + _value(a.$1));
    final canPick = e.bodyDecoded && widget.choices.isNotEmpty;
    final canAdd = canPick && used < e.augmentSlots;

    final items = list(const ValueKey('equip-items'), [
      if (!e.bodyDecoded)
        AppText(l.equipNotDecoded, role: TextRole.caption, wrap: true)
      else ...[
        ..._itemSections(context, l, e),
        const SizedBox(height: Space.md),
        // One row per augment copy, then an empty row per free slot: the
        // area keeps its size.
        _Heading(
          title: l.equipAugments,
          note: l.equipSlots(used, e.augmentSlots),
          trailing: Builder(
            builder: (context) => AppButton(
              key: const ValueKey('equip-add-augment'),
              icon: Icons.add,
              tooltip: l.equipAddAugment,
              onPressed: canAdd
                  ? () async => _replace(
                      await _pickAugment(context, title: l.equipAddAugment),
                      current: null,
                      add: widget.onAddAugment,
                    )
                  : null,
            ),
          ),
        ),
        for (final (field, name) in augments)
          for (var copy = 0; copy < _value(field); copy++)
            _ItemRow(
              key: ValueKey('equip-augment-${field.path.split('.').last}-$copy'),
              art: Icon(Icons.extension, size: 24, color: AppTokens.of(context).muted),
              title: name,
              detail: field.path.split('.').last,
              marked: copy >= field.value,
              mark: copy >= field.value ? EditDirection.raised : null,
              onTap: canPick
                  ? (context) async => _replace(
                      await _pickAugment(context, title: l.equipAugments, current: field.path.split('.').last),
                      current: field.path.split('.').last,
                      remove: () => widget.onChanged(field.path, _value(field) - 1),
                      add: widget.onAddAugment,
                    )
                  : null,
              trailing: AppButton(
                key: ValueKey(
                  copy == 0 ? 'equip-remove-${field.path.split('.').last}' : 'equip-remove-${field.path}-$copy',
                ),
                icon: Icons.delete_outline,
                tooltip: l.equipRemove,
                onPressed: () => widget.onChanged(field.path, _value(field) - 1),
              ),
            ),
        for (var i = used; i < e.augmentSlots; i++)
          _EmptySlot(
            key: ValueKey('equip-empty-augment-$i'),
            onTap: canAdd
                ? (context) async => _replace(
                    await _pickAugment(context, title: l.equipAugments, current: ''),
                    current: '',
                    add: widget.onAddAugment,
                  )
                : null,
          ),
      ],
    ]);

    return Padding(
      padding: const EdgeInsets.all(Space.lg),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Expanded(flex: 11, child: Panel(child: systems)),
          const SizedBox(width: Space.md),
          Expanded(flex: 8, child: Panel(child: items)),
        ],
      ),
    );
  }

  /// Weapons, drones and cargo as the preview shows them ([ItemSlots]): a
  /// row per copy with its art, Power on/off, Move to cargo and Remove; a
  /// row per cargo item with Move to a slot and Remove; an Add button on
  /// each heading.
  List<Widget> _itemSections(BuildContext context, AppLocalizations l, Equipment file) {
    final slots = ItemSlots(
      file: file,
      preview: widget.preview,
      onChanged: widget.onChanged,
      onAddItem: widget.onAddItem,
      itemChoices: widget.itemChoices,
      group: widget.group,
    );
    final p = slots.now;
    final editable = p.itemFields.any((f) => f.slots != null || f.cargo != null) || widget.itemChoices.isNotEmpty;
    final cargo = p.cargo;
    final fileCargo = file.cargo ?? const [];
    final seenCargo = <String, int>{};
    return [
      ...slots.section(context, drone: false),
      const SizedBox(height: Space.md),
      ...slots.section(context, drone: true),
      if (slots.hold) ...[
        const SizedBox(height: Space.md),
        _Heading(
          title: l.equipCargo,
          note: cargo == null ? null : slotCount(l, cargo.length, p.cargoSlots),
          trailing: cargo == null
              ? null
              : slots.addButton('equip-add-cargo', l.equipAddCargo, cargo.length < p.cargoSlots),
        ),
        if (cargo == null)
          AppText(l.equipCargoUnknown, role: TextRole.caption, wrap: true)
        else ...[
          for (final (i, item) in cargo.indexed)
            () {
              final f = slots.fields(item.id);
              final copy = seenCargo[item.id] = (seenCargo[item.id] ?? -1) + 1;
              final added = copy >= fileCargo.where((x) => x.id == item.id).length;
              final inSlots = f?.slots;
              return _ItemRow(
                key: ValueKey('equip-cargo-$i'),
                art: _itemArt(item.id, drone: item.drone),
                mark: added ? EditDirection.raised : null,
                title: item.name ?? item.id,
                detail: item.power == null ? item.id : l.equipPower(item.power!),
                marked: added,
                onTap: slots.pick(l.equipCargo, item: item, count: f?.cargo),
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    AppButton(
                      key: ValueKey('equip-to-slot-cargo-$i'),
                      icon: Icons.north_west,
                      tooltip: l.equipToSlot,
                      onPressed: slots.can(f?.cargo, -1) && slots.can(inSlots, 1)
                          ? () {
                              slots.set(f?.cargo, -1);
                              slots.set(inSlots, 1);
                            }
                          : null,
                    ),
                    const SizedBox(width: Space.xs),
                    AppButton(
                      key: ValueKey('equip-remove-cargo-$i'),
                      icon: Icons.delete_outline,
                      tooltip: l.equipRemove,
                      onPressed: slots.can(f?.cargo, -1) ? () => slots.set(f?.cargo, -1) : null,
                    ),
                  ],
                ),
              );
            }(),
          ...slots.empty('cargo', cargo.length, p.cargoSlots, l.equipCargo),
        ],
      ],
      if (!editable) ...[
        const SizedBox(height: Space.xs),
        AppText(l.equipItemsLocked, role: TextRole.caption, wrap: true),
      ],
      if (widget.previewError case final error?) ...[
        const SizedBox(height: Space.xs),
        AppText(
          error,
          key: const ValueKey('equip-preview-error'),
          role: TextRole.caption,
          color: AppTokens.of(context).warn,
          wrap: true,
        ),
      ],
    ];
  }
}

/// The slots in use of all, or only the count when the ship names none.
String slotCount(AppLocalizations l, int used, int? all) => all == null ? '$used' : l.equipSlots(used, all);

/// Stages what the chooser of a slot answered: [remove] takes the copy out
/// of the slot (a filled slot), [add] puts the chosen one in, as one step of
/// Undo through [group]. The removal goes first, so the core puts the new
/// item into the freed slot (`ftl_core::slot_order`); the same item or a
/// dismissed chooser stages nothing.
void stageReplace(
  void Function(VoidCallback edits)? group,
  String? chosen, {
  required String? current,
  VoidCallback? remove,
  required ValueChanged<String> add,
}) {
  if (chosen == null || chosen == (current ?? '')) return;
  (group ?? (edits) => edits())(() {
    if (current != null && current.isNotEmpty) remove?.call();
    if (chosen.isNotEmpty) add(chosen);
  });
}

/// The weapon and drone slots of one ship and what a click on them does:
/// the one place that lists a slot's rows (art, Power, Move to cargo,
/// Remove), its empty rows and Add, and opens the chooser of a slot. The
/// Equipment page, the floorplan room popover of Weapon Control and Drone
/// Control and the weapon mounts on the hull use it. Every action sets a
/// count of the core (`weapons.<ID>`, `weapons.<ID>.armed`, `cargo.<ID>`)
/// within the range of the [preview].
class ItemSlots {
  ItemSlots({
    required this.file,
    required this.preview,
    required this.onChanged,
    required this.onAddItem,
    required this.itemChoices,
    this.group,
  });

  /// The equipment of the file and as the pending edits leave it (`null`:
  /// no edits).
  final Equipment file;
  final Equipment? preview;
  final void Function(String path, int value) onChanged;

  /// Adds a weapon or drone to the slots, or with `toCargo` to the cargo.
  final void Function(String id, {required bool toCargo})? onAddItem;
  final List<ItemChoice> itemChoices;

  /// Runs the edits of one replace as one step of Undo.
  final void Function(VoidCallback edits)? group;

  Equipment get now => preview ?? file;

  /// The cargo hold is the player's: the enemy ship shows neither the hold
  /// nor the button that moves an item there.
  bool get hold => !isNearbyPath(file.reactor.path);

  bool get canPick => onAddItem != null && itemChoices.isNotEmpty;

  EquipmentItemFields? fields(String id) => now.itemFields.where((f) => f.id == id).firstOrNull;

  bool can(FieldValue? f, int delta) => f != null && f.value + delta >= f.min && f.value + delta <= f.max;

  void set(FieldValue? f, int delta) {
    if (f != null) onChanged(f.path, f.value + delta);
  }

  /// The items of the slots of one kind as the preview shows them.
  List<EquipmentItem> items({required bool drone}) => drone ? now.drones : now.weapons;

  /// The chooser of a weapon slot, a drone slot or ([drone] `null`) a cargo
  /// box: [current] is what the slot holds (`''` empty, `null` for Add).
  /// For a slot with an [item] it offers Power on or off beside Cancel.
  Future<String?> choose(
    BuildContext context, {
    bool? drone,
    required String title,
    String? current,
    EquipmentItem? item,
  }) {
    final l = AppLocalizations.of(context)!;
    final armed = item == null || drone == null ? null : fields(item.id)?.armed;
    final delta = item?.armed == true ? -1 : 1;
    return showDialog<String>(
      context: context,
      builder: (_) => ChoicePicker(
        keyPrefix: 'item',
        icon: drone == true ? Icons.precision_manufacturing : Icons.gps_fixed,
        title: title,
        current: current,
        power: item == null || drone == null
            ? null
            : (on: item.armed, onPressed: can(armed, delta) ? () => set(armed, delta) : null),
        choices: [
          for (final c in itemChoices)
            if (drone == null || c.drone == drone)
              PickerChoice(c.id, c.name, '${l.equipPower(c.power)} · ${c.id}', _choiceArt(c.id, drone: c.drone)),
        ],
      ),
    );
  }

  /// A click on a slot: the chooser of its list; [item] is what the slot
  /// holds, [count] the field that takes it out. `null` when nothing can
  /// be picked.
  Future<void> Function(BuildContext)? pick(String title, {bool? drone, EquipmentItem? item, FieldValue? count}) {
    final add = onAddItem;
    if (add == null || !canPick || (item != null && !can(count, -1))) return null;
    return (context) async => stageReplace(
      group,
      await choose(context, drone: drone, title: title, current: item?.id ?? '', item: item),
      current: item?.id,
      remove: () => set(count, -1),
      add: (id) => add(id, toCargo: drone == null),
    );
  }

  /// Opens the chooser of weapon or drone slot [slot] (a weapon mount on
  /// the hull); nothing when it cannot be picked.
  Future<void> openSlot(BuildContext context, {required bool drone, required int slot}) async {
    final l = AppLocalizations.of(context)!;
    final list = items(drone: drone);
    final item = slot < list.length ? list[slot] : null;
    final open = pick(
      drone ? l.equipDrones : l.equipWeapons,
      drone: drone,
      item: item,
      count: item == null ? null : fields(item.id)?.slots,
    );
    await open?.call(context);
  }

  /// The Add button of a section: a plus on the right of its heading.
  Widget addButton(String key, String label, bool enabled, {bool? drone}) => Builder(
    builder: (context) {
      final open = enabled ? pick(label, drone: drone) : null;
      return AppButton(
        key: ValueKey(key),
        icon: Icons.add,
        tooltip: label,
        onPressed: open == null ? null : () => open(context),
      );
    },
  );

  /// An empty row per free slot: each section keeps the size of its slots.
  List<Widget> empty(String kind, int used, int? all, String title, {bool? drone}) => [
    for (var i = used; i < (all ?? used); i++)
      _EmptySlot(
        key: ValueKey('equip-empty-$kind-$i'),
        onTap: pick(title, drone: drone),
      ),
  ];

  /// The weapons or drones as one section: heading with the slots in use
  /// and Add, a row per copy, an empty row per free slot.
  List<Widget> section(BuildContext context, {required bool drone}) {
    final l = AppLocalizations.of(context)!;
    final kind = drone ? 'drone' : 'weapon';
    final title = drone ? l.equipDrones : l.equipWeapons;
    final now = items(drone: drone);
    final all = drone ? this.now.droneSlots : this.now.weaponSlots;
    return [
      _Heading(
        title: title,
        note: slotCount(l, now.length, all),
        trailing: addButton(
          'equip-add-$kind',
          drone ? l.equipAddDrone : l.equipAddWeapon,
          all != null && now.length < all,
          drone: drone,
        ),
      ),
      ...rows(context, drone: drone),
      ...empty(kind, now.length, all, title, drone: drone),
    ];
  }

  /// A row per weapon or drone of the preview, marked when an edit added
  /// it or changed its power.
  List<Widget> rows(BuildContext context, {required bool drone}) {
    final l = AppLocalizations.of(context)!;
    final kind = drone ? 'drone' : 'weapon';
    final title = drone ? l.equipDrones : l.equipWeapons;
    final now = items(drone: drone);
    final before = drone ? file.drones : file.weapons;
    final seen = <String, int>{};
    return [
      for (final (i, item) in now.indexed)
        () {
          final f = fields(item.id);
          final copy = seen[item.id] = (seen[item.id] ?? -1) + 1;
          final added = copy >= before.where((x) => x.id == item.id).length;
          final armedNow = now.where((x) => x.id == item.id && x.armed).length;
          final armedBefore = before.where((x) => x.id == item.id && x.armed).length;
          final direction = added || (item.armed && armedNow > armedBefore)
              ? EditDirection.raised
              : !item.armed && armedNow < armedBefore
              ? EditDirection.lowered
              : null;
          return _ItemRow(
            key: ValueKey('equip-$kind-$i'),
            art: _itemArt(item.id, drone: item.drone),
            mark: direction,
            title: item.name ?? item.id,
            detail: [item.power == null ? item.id : l.equipPower(item.power!), if (item.armed) l.equipOn].join(' · '),
            marked: direction != null,
            onTap: pick(title, drone: item.drone, item: item, count: f?.slots),
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                AppButton(
                  key: ValueKey('equip-power-$kind-$i'),
                  icon: Icons.bolt,
                  kind: item.armed ? ButtonKind.primary : ButtonKind.outlined,
                  tooltip: item.armed ? l.equipPowerOff : l.equipPowerOn,
                  onPressed: can(f?.armed, item.armed ? -1 : 1) ? () => set(f?.armed, item.armed ? -1 : 1) : null,
                ),
                if (hold) ...[
                  const SizedBox(width: Space.xs),
                  AppButton(
                    key: ValueKey('equip-to-cargo-$kind-$i'),
                    icon: Icons.inventory_2,
                    tooltip: l.equipToCargo,
                    onPressed: can(f?.slots, -1) && can(f?.cargo, 1)
                        ? () {
                            set(f?.slots, -1);
                            set(f?.cargo, 1);
                          }
                        : null,
                  ),
                ],
                const SizedBox(width: Space.xs),
                AppButton(
                  key: ValueKey('equip-remove-$kind-$i'),
                  icon: Icons.delete_outline,
                  tooltip: l.equipRemove,
                  onPressed: can(f?.slots, -1) ? () => set(f?.slots, -1) : null,
                ),
              ],
            ),
          );
        }(),
    ];
  }
}

/// One system as the Equipment page and the floorplan room popover show it:
/// its [SystemLevelRow] assembled from the file [system], the [preview] of
/// the pending edits and the [edits], and Install or Remove where the core
/// allows it (`systems.<id>.installed` of the preview). A staged install
/// shows the level FTL gives the system in yellow, a staged removal the old
/// level in red; both rows are locked until the change is saved or undone
/// with the same button. Damage and ion edit the system of the file within
/// the ranges of the preview, so a staged upgrade can be broken before the
/// save. The one builder of a system row: no page wires a
/// [SystemLevelRow] of a system on its own.
class SystemRow extends StatelessWidget {
  const SystemRow({
    super.key,
    required this.system,
    required this.equipment,
    required this.preview,
    required this.edits,
    required this.onChanged,
    this.stacked = false,
    this.hovered = false,
    this.onHover,
  });

  /// The system as the file has it (an entry of [equipment]).
  final EquipmentSystem system;

  /// The equipment of the file.
  final Equipment equipment;

  /// The equipment as the pending edits leave it; `null` shows the file.
  final Equipment? preview;
  final Map<String, int> edits;
  final void Function(String path, int value) onChanged;
  final bool stacked;
  final bool hovered;
  final ValueChanged<bool>? onHover;

  int _value(FieldValue f) => edits[f.path] ?? f.value;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final s = system;
    final p = (preview ?? equipment).systems.where((x) => x.id == s.id).firstOrNull ?? s;
    final inFile = s.installed.value == 1;
    // A staged edit says it; else the preview, where installing the Clone
    // Bay removes the Medbay without an edit of its own.
    final edited = edits.containsKey(s.installed.path);
    final now = edited || preview == null ? _value(s.installed) == 1 : p.level != null;
    final staged = now != inFile;
    final FieldValue? field;
    final int value;
    if (staged && now) {
      // A staged install can be raised at once: the level is an edit of its
      // own (the file has none, so it counts from 0) within the preview's
      // range, and the core applies it after the install.
      final level = p.level;
      field = FieldValue(
        path: '${shipPrefix(s.installed.path)}systems.${s.id}.level',
        value: 0,
        min: level?.min ?? 1,
        max: level?.max ?? 3,
      );
      value = edits[field.path] ?? level?.value ?? 1;
    } else if (staged) {
      field = s.level;
      value = 0;
    } else {
      field = s.level;
      value = s.level == null ? 0 : _value(s.level!);
    }
    // Undoing a staged install is always allowed. Undoing a staged removal
    // is an install: it needs a free place in the preview, like any other.
    final allowed = (staged && now) || (now ? p.installed.min == 0 : p.installed.max == 1);
    // Medbay and Clone Bay share a room: installing one replaces the other.
    final other = switch (s.id) {
      'medbay' => 'clonebay',
      'clonebay' => 'medbay',
      _ => null,
    };
    final replaces = now || other == null
        ? null
        : (preview ?? equipment).systems.where((x) => x.id == other && x.level != null).firstOrNull;
    final String? tip = allowed
        ? (replaces == null ? null : l.equipReplaces(replaces.name ?? replaces.id))
        : (p.block == null ? null : installBlockText(p.block!, l));
    Widget? action;
    // A system the swap of another removes changes back with that one.
    if ((!now || allowed) && (edited || !staged)) {
      final button = AppButton(
        key: ValueKey('equip-${now ? 'remove' : 'install'}-system-${s.id}'),
        label: now ? l.equipRemove : l.equipInstall,
        kind: ButtonKind.outlined,
        onPressed: allowed ? () => onChanged(s.installed.path, now ? 0 : 1) : null,
      );
      action = tip == null ? button : Tooltip(message: tip, child: button);
    }
    // Damage and ion belong to the system of the file; a staged install
    // or removal shows neither. Their ranges come from the preview, so a
    // staged upgrade can be broken before the save; the value stays the
    // file's, so the marks compare against the file.
    FieldValue? withPreviewRange(FieldValue? file, FieldValue? preview) => file == null || preview == null
        ? file
        : FieldValue(path: file.path, value: file.value, min: preview.min, max: preview.max);
    final damage = staged ? null : withPreviewRange(s.damage, p.damage);
    final ion = staged ? null : withPreviewRange(s.ion, p.ion);
    final energy = staged ? null : withPreviewRange(s.energy, p.energy);
    final damageValue = damage == null ? 0 : _value(damage).clamp(0, value);
    return SystemLevelRow(
      id: s.id,
      icon: systemIconState(s, hovered: hovered, installed: now, level: value, damaged: damageValue),
      title: s.name ?? s.id,
      field: field,
      value: value,
      hovered: hovered,
      // Only a staged removal locks the row; a staged install takes a level.
      locked: staged && !now,
      action: action,
      stacked: stacked,
      damage: damage,
      damageValue: damageValue,
      ion: ion,
      ionValue: ion == null ? 0 : _value(ion),
      onHover: onHover,
      onSet: (v) {
        if (field != null) onChanged(field.path, v);
      },
      onDamage: damage == null ? null : (v) => onChanged(damage.path, v),
      onIon: ion == null ? null : (v) => onChanged(ion.path, v),
      energy: energy,
      energyValue: energy == null ? 0 : _value(energy),
      onEnergy: energy == null ? null : (v) => onChanged(energy.path, v),
    );
  }
}

/// The art of a weapon (its first frame, turned upright) or a drone, from
/// ftl.dat; a Material icon in reduced mode.
Widget _itemArt(String id, {required bool drone}) => drone
    ? GameImage(
        GameImageRef.of(GameImageKind.drone, id),
        fallback: Icons.precision_manufacturing,
        width: 44,
        height: 44,
        iconSize: 24,
      )
    : RotatedBox(
        quarterTurns: 1,
        child: GameImage(
          GameImageRef.of(GameImageKind.weaponFrame, id),
          fallback: Icons.gps_fixed,
          width: 27,
          height: 64,
          iconSize: 24,
        ),
      );

/// The art of a weapon or drone in the chooser: a [GameIcon], cropped to
/// its visible pixels at a whole factor, the weapon turned like in the slot.
Widget _choiceArt(String id, {required bool drone}) => drone
    ? GameIcon(GameImageRef.of(GameImageKind.drone, id), fallback: Icons.precision_manufacturing, size: 36)
    : RotatedBox(
        quarterTurns: 1,
        child: GameIcon(GameImageRef.of(GameImageKind.weaponFrame, id), fallback: Icons.gps_fixed, size: 36),
      );

/// The background of the row the pointer rests on.
Color rowHoverColor(BuildContext context) => FtlTheme.of(context) == null
    ? Theme.of(context).colorScheme.surfaceContainerHighest
    : FtlColors.window.withAlpha(0x70);

/// The head of a section: its title, the slots in use, and its Add button
/// on the right.
class _Heading extends StatelessWidget {
  const _Heading({required this.title, this.note, this.trailing});

  final String title;
  final String? note;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: Space.xs, bottom: Space.sm),
    child: TitleRow(title: title, note: note, trailing: trailing),
  );
}

/// A slot that holds nothing: a dim frame where the art would be and the
/// row height of an item, so a section keeps its size. A click opens the
/// chooser of its list.
/// The dim frame of a slot that holds nothing.
class _SlotFrame extends StatelessWidget {
  const _SlotFrame();

  @override
  Widget build(BuildContext context) => Container(
    width: 40,
    height: 32,
    decoration: BoxDecoration(border: Border.all(color: AppTokens.of(context).muted.withAlpha(0x80), width: 2)),
  );
}

class _EmptySlot extends StatelessWidget {
  const _EmptySlot({super.key, this.onTap});

  /// Opens the chooser of the slot.
  final Future<void> Function(BuildContext)? onTap;

  @override
  Widget build(BuildContext context) {
    return ListRow(
      leading: const SizedBox(width: 64, height: 44, child: Center(child: _SlotFrame())),
      title: AppLocalizations.of(context)!.equipEmptySlot,
      muted: true,
      onTap: onTap == null ? null : () => onTap!(context),
    );
  }
}

/// How a segment is drawn.
enum _Cell { powered, raised, lowered, damaged, preview, empty, none }

/// A system (or the reactor) as one row: the FTL icon of its state, the
/// name, a segment per possible level, the level as a number. A click on
/// segment n sets level n; a click on the top segment of the current level
/// lowers it by one. Levels outside the range of [field] are not offered.
/// A segment an edit adds is yellow, one it removes red; a [ChangeMark] at
/// the icon says the row has a pending edit. The floorplan popover shows the
/// same row. [stacked] puts the segments and the action on a line of their
/// own under the name, at their full size (a narrow window,
/// 2026-11-19): segments scaled down to fit beside the name were unreadable.
///
/// Damage as FTL draws it (`ShipSystem::RenderPowerBoxes`, see
/// `ftl_core::damage`): the top [damageValue] segments of the level are
/// broken, [damageRed] with a crack, and the row names the count. A right
/// click on a segment breaks it and every one above it; a right click on a
/// broken one mends it and every one below it ([onDamage],
/// `systems.<id>.damage`). An ion lock ([ionValue] above 0, or -1) puts
/// FTL's blue outline around the segments; the lock after the number sets
/// `systems.<id>.ion`: a click adds one up to 5 and then clears it, a right
/// click opens the value popover ([onIon]). A segment whose damage an edit
/// changes has a yellow edge.
///
/// The Zoltan shield of the shields row ([energy], `systems.shields.energy`,
/// see `ftl_core::energy_shield`) lies over the segments as FTL lays its
/// green bars over the shield bubbles: a bar per point in [zoltanGreen] at
/// the sizes and steps of `ShipStatus::OnInit`, an outline per free point up
/// to the max. A click on bar n sets n points, a click on the top bar takes
/// it away, a right click opens the value popover ([onEnergy]).
class SystemLevelRow extends StatelessWidget {
  const SystemLevelRow({
    super.key,
    required this.id,
    required this.icon,
    required this.title,
    required this.field,
    required this.value,
    required this.onSet,
    this.hovered = false,
    this.onHover,
    this.cell = _systemCell,
    this.locked = false,
    this.action,
    this.stacked = false,
    this.damage,
    this.damageValue = 0,
    this.onDamage,
    this.ion,
    this.ionValue = 0,
    this.onIon,
    this.energy,
    this.energyValue = 0,
    this.onEnergy,
  });

  final String id;

  /// The state of the FTL icon (`green1`, `grey1`, ...); `null` for no icon.
  final String? icon;
  final String title;

  /// `null` when the system is not installed.
  final FieldValue? field;
  final int value;
  final ValueChanged<int> onSet;
  final bool hovered;
  final ValueChanged<bool>? onHover;
  final double cell;

  /// No segment takes a click: the row shows a staged install or removal.
  final bool locked;

  /// Install or Remove at the end of the row.
  final Widget? action;

  /// Name on one line, segments and action on the next.
  final bool stacked;

  /// `systems.<id>.damage` of the file and the pending broken bars;
  /// `null` for the reactor and a system that is not installed.
  final FieldValue? damage;
  final int damageValue;
  final ValueChanged<int>? onDamage;

  /// `systems.<id>.ion` of the file and the pending lock count.
  final FieldValue? ion;
  final int ionValue;
  final ValueChanged<int>? onIon;

  /// `systems.shields.energy` of the file and the pending Zoltan points;
  /// `null` on every other row.
  final FieldValue? energy;
  final int energyValue;
  final ValueChanged<int>? onEnergy;

  static const _height = 34.0;

  /// The height of the Zoltan strip above the segments: a 6 px bar and a
  /// 2 px gap. The same room stays free below, so the segments keep the
  /// middle of the row.
  static const _zoltan = 8.0;
  static const _name = 220.0;
  static const _icon = 32.0;

  /// The room the number takes after the segments ("25/25"), and the room
  /// of Install or Remove (the longer German label included).
  static const _number = 48.0;
  static const _action = 100.0;

  /// The ion outline around the segments: its distance and its 2 px line.
  /// It lies over the gaps beside the segments and takes no room.
  static const _frame = Space.xs + _frameLine;
  static const _frameLine = 2.0;

  static double _gap(double cell) => cell > 10 ? Space.xs : Space.xs / 2;

  /// The width a row of [levels] segments of [cell] needs beside its name;
  /// a narrower panel stacks it.
  static double wideWidth(int levels, double cell, {required bool action}) =>
      2 * Space.sm +
      _icon +
      Space.md +
      _name +
      levels * cell +
      (levels - 1) * _gap(cell) +
      Space.md +
      _number +
      (action ? Space.sm + _action : 0);

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final t = AppTokens.of(context);
    final field = this.field;
    final damage = this.damage;
    final ion = this.ion;
    final energy = this.energy;
    // The mark says the row has a pending edit; a level edit names its
    // direction, else damage and ion (more broken or locked is lowered).
    final direction = field == null
        ? null
        : directionOf(field.value, value) ??
              (damage == null ? null : directionOf(damageValue, damage.value)) ??
              (ion == null ? null : directionOf(ionValue.abs(), ion.value.abs())) ??
              (energy == null ? null : directionOf(energy.value, energyValue));
    final text = hovered
        ? t.accent
        : field != null
        ? t.text
        : t.muted;
    final max = field?.max ?? 3;
    final icon = this.icon;
    // The 64 px icon at its own size; its glow reaches past the row.
    final iconBox = SizedBox(
      width: _icon,
      height: _icon,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          if (icon != null)
            Positioned.fill(
              child: OverflowBox(
                maxWidth: 64,
                maxHeight: 64,
                child: GameImage(systemIcon(id, icon), fallback: Icons.settings, width: 64, height: 64, iconSize: 22),
              ),
            ),
          if (direction != null)
            Positioned(right: -2, top: -2, child: ChangeMark(direction, key: ValueKey('equip-mark-$id'))),
        ],
      ),
    );
    // The name is clipped only at its sides (dots of Ü).
    // The name is whole first; the broken count and the ion lock share
    // its column (the row keeps its width, so the bars keep their size).
    // A gap keeps a long name (WAFFENSTEUERUNG) off the broken count.
    final hasStatus = onIon != null || damage != null;
    final label = Align(
      alignment: Alignment.centerLeft,
      child: AppText(title, key: ValueKey('equip-name-$id'), role: TextRole.label, caps: true, color: text),
    );
    final name = hasStatus
        ? Row(
            children: [
              Expanded(child: label),
              const SizedBox(width: Space.sm),
              _status(context, l, t),
              const SizedBox(width: Space.sm),
            ],
          )
        : label;
    final ionLocked = ionValue != 0;
    final segments = Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (var i = 0; i < max; i++) ...[if (i > 0) SizedBox(width: _gap(cell)), _segment(i, field, t)],
      ],
    );
    final bars = Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        // FTL's blue outline around the bars of an ion-locked system (white
        // for a lock without a count), drawn outside the segments.
        Stack(
          clipBehavior: Clip.none,
          children: [
            if (energy == null)
              segments
            else
              Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _ZoltanBars(id: id, title: title, field: energy, value: energyValue, onSet: locked ? null : onEnergy),
                  segments,
                  const SizedBox(height: _zoltan),
                ],
              ),
            if (ionLocked)
              Positioned(
                left: -_frame,
                top: -_frame,
                right: -_frame,
                bottom: -_frame,
                child: IgnorePointer(
                  child: DecoratedBox(
                    key: ValueKey('equip-ion-frame-$id'),
                    decoration: BoxDecoration(
                      border: Border.all(color: ionValue < 0 ? Colors.white : ionBlue, width: _frameLine),
                    ),
                  ),
                ),
              ),
          ],
        ),
        const SizedBox(width: Space.md),
        if (field != null)
          AppText(
            '$value/$max',
            role: TextRole.number,
            // The number is the level: only a level edit colors it.
            color: switch (directionOf(field.value, value)) {
              EditDirection.raised => t.accent,
              EditDirection.lowered => t.warn,
              null => t.text,
            },
          )
        else
          AppText(l.equipNotInstalled, role: TextRole.caption, caps: true),
      ],
    );
    // FittedBox only keeps a too narrow window from overflowing; the page
    // stacks the rows before the segments would shrink.
    Widget fit(Widget child, Alignment alignment) =>
        FittedBox(fit: BoxFit.scaleDown, alignment: alignment, child: child);
    final action = this.action;
    final tail = [
      Expanded(child: fit(bars, Alignment.centerLeft)),
      // The button keeps its size: as a Flexible it got half the row and
      // shrank with the segments.
      if (action != null) ...[const SizedBox(width: Space.sm), action],
    ];
    final Widget content = stacked
        ? Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              SizedBox(
                height: _height,
                child: Row(
                  children: [
                    iconBox,
                    const SizedBox(width: Space.md),
                    Expanded(child: name),
                  ],
                ),
              ),
              SizedBox(
                key: ValueKey('equip-bars-$id'),
                height: _height,
                child: Row(children: tail),
              ),
              const SizedBox(height: Space.sm),
            ],
          )
        : SizedBox(
            height: _height,
            child: Row(
              children: [
                iconBox,
                const SizedBox(width: Space.md),
                Flexible(
                  flex: 0,
                  child: SizedBox(width: _name, child: name),
                ),
                ...tail,
              ],
            ),
          );
    return MouseRegion(
      onEnter: (_) => onHover?.call(true),
      onExit: (_) => onHover?.call(false),
      child: Container(
        key: ValueKey('equip-row-$id'),
        color: hovered ? rowHoverColor(context) : null,
        padding: const EdgeInsets.symmetric(horizontal: Space.sm),
        child: content,
      ),
    );
  }

  /// Whether segment [i] is broken with [broken] bars at level [level]:
  /// FTL breaks the top bars of the column.
  static bool _broken(int i, int level, int broken) => i < level && i >= level - broken;

  Widget _segment(int i, FieldValue? field, AppTokens t) {
    final _Cell cell;
    final damage = this.damage;
    final broken = damage != null && _broken(i, value, damageValue);
    if (field == null) {
      cell = _Cell.none;
    } else if (broken) {
      cell = _Cell.damaged;
    } else if (i < value) {
      cell = i < field.value ? _Cell.powered : _Cell.raised;
    } else if (i < field.value) {
      cell = _Cell.lowered;
    } else {
      cell = hovered && i == value ? _Cell.preview : _Cell.empty;
    }
    // A bar whose damage an edit changes, within the level it keeps.
    final changed = damage != null && i < value && i < field!.value && broken != _broken(i, field.value, damage.value);
    final paint = CustomPaint(
      key: ValueKey('equip-seg-$id-$i-${cell.name}'),
      size: Size(this.cell, 18),
      painter: _CellPainter(cell, t.muted, t.accent, t.warn, changed: changed),
    );
    if (field == null || locked) return paint;
    // The top segment of the level lowers it; any other sets its level.
    final target = i + 1 == value ? i : i + 1;
    final allowed = target >= field.min && target <= field.max;
    // A right click breaks the segment and those above it, or mends a
    // broken one and those below it.
    final onDamage = this.onDamage;
    final hurt = broken ? value - i - 1 : value - i;
    final canHurt = onDamage != null && i < value && hurt >= 0 && hurt <= damage!.max;
    return MouseRegion(
      cursor: allowed || canHurt ? SystemMouseCursors.click : MouseCursor.defer,
      child: GestureDetector(
        key: ValueKey('equip-cell-$id-$i'),
        onTap: allowed ? () => onSet(target) : null,
        onSecondaryTap: canHurt ? () => onDamage(hurt) : null,
        child: paint,
      ),
    );
  }

  /// The ion lock (a button) and the count of broken bars.
  Widget _status(BuildContext context, AppLocalizations l, AppTokens t) {
    final ion = this.ion;
    final onIon = this.onIon;
    Widget? lock;
    if (ion != null && onIon != null) {
      final on = ionValue != 0;
      final image = Opacity(
        opacity: on ? 1 : 0.35,
        child: GameImage(
          ionLock(white: ionValue < 0),
          fallback: Icons.lock_outline,
          width: 23,
          height: 27,
          iconSize: 18,
        ),
      );
      lock = Tooltip(
        message: l.equipIonTip(ionValue < 0 ? 0 : ionValue),
        child: MouseRegion(
          cursor: SystemMouseCursors.click,
          child: Builder(
            builder: (context) => GestureDetector(
              key: ValueKey('equip-ion-$id'),
              // The gap and the bitmap number take no hit of their own.
              behavior: HitTestBehavior.opaque,
              onTap: () => onIon(ionValue <= 0 || ionValue >= ion.max ? (ionValue == 0 ? 1 : 0) : ionValue + 1),
              onSecondaryTap: () async {
                final box = context.findRenderObject() as RenderBox;
                final anchor = box.localToGlobal(Offset.zero) & box.size;
                final v = await showResourcePopover(
                  context,
                  anchor: anchor,
                  field: ion,
                  current: ionValue.clamp(ion.min, ion.max),
                  look: (Icons.lock_outline, l.equipSystemIon(title)),
                  image: FieldImage(ionLock(), tint: false),
                );
                if (v != null) onIon(v);
              },
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  image,
                  if (ionValue > 0) ...[
                    const SizedBox(width: Space.xs),
                    AppText('$ionValue', role: TextRole.number, color: ionBlue),
                  ],
                ],
              ),
            ),
          ),
        ),
      );
    }
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        // The broken count: a small broken bar and the number, named in the tooltip.
        if (damageValue > 0) ...[
          Tooltip(
            message: l.equipDamaged(damageValue),
            child: Row(
              key: ValueKey('equip-damaged-$id'),
              mainAxisSize: MainAxisSize.min,
              children: [
                CustomPaint(
                  size: const Size(Space.sm, Space.lg),
                  painter: _CellPainter(_Cell.damaged, t.muted, t.accent, t.warn),
                ),
                const SizedBox(width: Space.xs),
                AppText('$damageValue', role: TextRole.number, color: damageRed),
              ],
            ),
          ),
          const SizedBox(width: Space.sm),
        ],
        ?lock,
      ],
    );
  }
}

/// The Zoltan shield points of a shields row: FTL's bars of
/// `ShipStatus::OnInit` (17 x 6 px, the third 16 px, 19 px apart; the
/// steps go on past five for the flagship's twelve), green for a point the
/// file has and keeps, yellow for one an edit adds, red for one it takes,
/// an outline for a free point.
class _ZoltanBars extends StatelessWidget {
  const _ZoltanBars({required this.id, required this.title, required this.field, required this.value, this.onSet});

  final String id;
  final String title;
  final FieldValue field;
  final int value;
  final ValueChanged<int>? onSet;

  static const _step = 19.0;
  static double _width(int i) => i == 2 ? 16 : 17;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final t = AppTokens.of(context);
    final onSet = this.onSet;
    Widget bar(int i) {
      final fill = i < value
          ? (i < field.value ? zoltanGreen : t.accent)
          : i < field.value
          ? t.warn
          : null;
      final paint = SizedBox(
        width: _step,
        height: SystemLevelRow._zoltan,
        child: Align(
          alignment: Alignment.topLeft,
          child: Container(
            key: ValueKey('equip-zoltan-$id-$i-${fill == null ? 'free' : 'on'}'),
            width: _width(i),
            height: 6,
            decoration: BoxDecoration(
              color: fill,
              border: fill == null ? Border.all(color: t.muted.withAlpha(0x80)) : null,
            ),
          ),
        ),
      );
      if (onSet == null) return paint;
      // The top bar takes its point away; any other sets its count.
      final target = i + 1 == value ? i : i + 1;
      return MouseRegion(
        cursor: SystemMouseCursors.click,
        child: Builder(
          builder: (context) => GestureDetector(
            key: ValueKey('equip-zoltan-cell-$id-$i'),
            behavior: HitTestBehavior.opaque,
            onTap: target <= field.max ? () => onSet(target) : null,
            onSecondaryTap: () async {
              final box = context.findRenderObject() as RenderBox;
              final anchor = box.localToGlobal(Offset.zero) & box.size;
              final v = await showResourcePopover(
                context,
                anchor: anchor,
                field: field,
                current: value.clamp(field.min, field.max),
                look: (Icons.shield_outlined, l.equipZoltanShield(title)),
              );
              if (v != null) onSet(v);
            },
            child: paint,
          ),
        ),
      );
    }

    return Tooltip(
      message: l.equipZoltanTip(value, field.max),
      child: Row(
        key: ValueKey('equip-zoltan-$id'),
        mainAxisSize: MainAxisSize.min,
        children: [for (var i = 0; i < field.max; i++) bar(i)],
      ),
    );
  }
}

/// One segment: green for a level the file has and keeps, yellow (FTL's
/// hover yellow) for one an edit adds, red (FTL's warning red, measured in
/// img/statusUI/top_hull_red_label.png) for one an edit removes, a yellow
/// outline where a click would go, an outline above.
class _CellPainter extends CustomPainter {
  _CellPainter(this.cell, this.off, this.hover, this.lowered, {this.changed = false});

  final _Cell cell;
  final Color off;
  final Color hover;
  final Color lowered;

  /// An edit changes the damage of the segment: a yellow edge.
  final bool changed;

  @override
  void paint(Canvas canvas, Size size) {
    final r = Offset.zero & size;
    final fill = switch (cell) {
      _Cell.powered => powerGreen,
      _Cell.raised => hover,
      _Cell.lowered => lowered,
      _Cell.damaged => damageRed,
      _ => null,
    };
    if (fill != null) {
      canvas.drawRect(r, Paint()..color = fill);
      if (cell == _Cell.damaged) {
        // A crack tells a broken bar from a level an edit removes (also red).
        final crack = Paint()
          ..color = const Color(0xFF1A0505)
          ..strokeWidth = 2
          ..style = PaintingStyle.stroke;
        canvas.drawPath(
          Path()
            ..moveTo(r.width * 0.3, 0)
            ..lineTo(r.width * 0.55, r.height * 0.45)
            ..lineTo(r.width * 0.4, r.height * 0.6)
            ..lineTo(r.width * 0.7, r.height),
          crack,
        );
      }
      if (changed) {
        canvas.drawRect(
          r.deflate(1),
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 2
            ..color = hover,
        );
      }
      return;
    }
    final edge = switch (cell) {
      _Cell.preview => hover,
      _Cell.none => off.withAlpha(0x60),
      _ => off,
    };
    canvas.drawRect(
      r.deflate(1),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = edge,
    );
  }

  @override
  bool shouldRepaint(_CellPainter old) =>
      old.cell != cell || old.off != off || old.hover != hover || old.lowered != lowered || old.changed != changed;
}

/// A weapon, drone or augment: its art, name and a detail line; [mark]
/// puts a [ChangeMark] at the art of an added or changed item.
class _ItemRow extends StatelessWidget {
  const _ItemRow({
    super.key,
    required this.art,
    required this.title,
    required this.detail,
    this.marked = false,
    this.mark,
    this.trailing,
    this.onTap,
  });

  final Widget art;
  final String title;
  final String detail;
  final bool marked;
  final EditDirection? mark;
  final Widget? trailing;

  /// Opens the chooser of the slot (its [BuildContext] lies below the look).
  final Future<void> Function(BuildContext)? onTap;

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, c) => ListRow(
      onTap: onTap == null ? null : () => onTap!(context),
      leading: SizedBox(
        width: 64,
        height: 44,
        child: Stack(
          clipBehavior: Clip.none,
          children: [
            Center(child: art),
            if (mark case final mark?) Positioned(right: 0, top: 0, child: ChangeMark(mark)),
          ],
        ),
      ),
      title: title,
      subtitle: detail,
      marked: marked,
      caps: true,
      // In a narrow panel the buttons shrink instead of overflowing the row.
      trailing: trailing == null
          ? null
          : ConstrainedBox(
              constraints: BoxConstraints(maxWidth: c.maxWidth * 0.5),
              child: FittedBox(fit: BoxFit.scaleDown, alignment: Alignment.centerRight, child: trailing),
            ),
    ),
  );
}

/// One entry of a [ChoicePicker].
class PickerChoice {
  const PickerChoice(this.id, this.name, this.detail, this.art);

  final String id;
  final String name;
  final String detail;
  final Widget art;
}

/// The one chooser of the weapon, drone, cargo and augment slots: every
/// choice of the game by name (the app language) with its art, and a
/// filter field. A slot click ([current] not `null`) lists "Empty slot"
/// first and marks what the slot holds (`''`: nothing). Answers the chosen
/// id, `''` for "Empty slot". Keys: `<keyPrefix>-filter`,
/// `<keyPrefix>-choice-<id>`, `<keyPrefix>-choice-empty`.
class ChoicePicker extends StatefulWidget {
  const ChoicePicker({
    super.key,
    required this.keyPrefix,
    required this.icon,
    required this.title,
    required this.choices,
    this.current,
    this.power,
  });

  final String keyPrefix;
  final IconData icon;
  final String title;
  final List<PickerChoice> choices;

  /// The id the clicked slot holds, `''` for an empty slot; `null` for the
  /// Add button, which offers no "Empty slot".
  final String? current;

  /// For a slot that holds a weapon or drone: Power off when [on], else
  /// Power on (`<keyPrefix>-power`), which stages it and closes the chooser;
  /// `onPressed` `null` when the bars do not allow it.
  final ({bool on, VoidCallback? onPressed})? power;

  @override
  State<ChoicePicker> createState() => ChoicePickerState();
}

class ChoicePickerState extends State<ChoicePicker> {
  String _filter = '';

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final shown = [
      for (final c in widget.choices)
        if (_filter.isEmpty || c.name.toLowerCase().contains(_filter.toLowerCase())) c,
    ];
    return AppDialog(
      icon: widget.icon,
      title: widget.title,
      content: SizedBox(
        width: 460,
        height: 420,
        child: Column(
          children: [
            AppTextField(
              key: ValueKey('${widget.keyPrefix}-filter'),
              autofocus: true,
              search: true,
              hint: l.equipFilter,
              onChanged: (v) => setState(() => _filter = v),
            ),
            const SizedBox(height: Space.sm),
            Expanded(
              child: ListView(
                children: [
                  if (widget.current case final current?)
                    ListRow(
                      key: ValueKey('${widget.keyPrefix}-choice-empty'),
                      leading: const SizedBox(width: 48, height: 40, child: Center(child: _SlotFrame())),
                      title: l.equipEmptySlot,
                      muted: current.isNotEmpty,
                      selected: current.isEmpty,
                      onTap: () => Navigator.of(context).pop(''),
                    ),
                  for (final c in shown)
                    ListRow(
                      key: ValueKey('${widget.keyPrefix}-choice-${c.id}'),
                      leading: SizedBox(width: 48, height: 40, child: Center(child: c.art)),
                      title: c.name,
                      subtitle: c.detail,
                      selected: c.id == widget.current,
                      onTap: () => Navigator.of(context).pop(c.id),
                    ),
                ],
              ),
            ),
          ],
        ),
      ),
      actions: [
        if (widget.power case final power?)
          AppButton(
            key: ValueKey('${widget.keyPrefix}-power'),
            icon: Icons.bolt,
            kind: power.on ? ButtonKind.primary : ButtonKind.outlined,
            label: power.on ? l.equipPowerOff : l.equipPowerOn,
            onPressed: power.onPressed == null
                ? null
                : () {
                    power.onPressed!();
                    Navigator.of(context).pop();
                  },
          ),
        AppButton(kind: ButtonKind.text, label: l.cancel, onPressed: () => Navigator.of(context).pop()),
      ],
    );
  }
}
