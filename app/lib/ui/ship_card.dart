import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../src/rust/api/assets.dart';
import '../src/rust/api/editor.dart';
import '../src/rust/api/identity.dart';
import 'change_sheet.dart';
import 'components/components.dart';
import 'crew_card.dart' show showRenamePopover;
import 'game_image.dart';

/// The edits of the ship card: the identity of the ship shown
/// (`identityOf`), the pending texts (`ship.name`, `nearby.ship.name`,
/// `ship.type`) and numbers (`difficulty`), and how to stage them. Without
/// [onText] the card shows no edit controls.
class ShipCardEdits {
  const ShipCardEdits({
    this.identity,
    this.texts = const {},
    this.edits = const {},
    this.onText,
    this.onEdit,
    this.typeChoices,
  });

  final ShipIdentity? identity;
  final Map<String, String> texts;
  final Map<String, int> edits;
  final void Function(String path, String text)? onText;
  final void Function(String path, int value)? onEdit;

  /// The hangar ships as types of the player ship (`shipTypeChoices`); empty
  /// without `ftl.dat`.
  final List<ShipTypeChoice> Function()? typeChoices;
}

/// The label of a difficulty number: 0 Easy, 1 Normal, 2 Hard.
String difficultyLabel(int n, AppLocalizations l) => switch (n) {
  0 => l.difficultyEasy,
  2 => l.difficultyHard,
  _ => l.difficultyNormal,
};

/// The text of a `TypeBlock` code of the core, with the system it names.
String typeBlockText(String code, String? system, AppLocalizations l) => switch (code) {
  'other_layout' => l.typeBlockOtherLayout,
  'system_room' => l.typeBlockSystemRoom(system ?? ''),
  'start_system' => l.typeBlockStartSystem(system ?? ''),
  'weapon_slots' => l.typeBlockWeaponSlots,
  'drone_slots' => l.typeBlockDroneSlots,
  'unknown_ship' => l.typeBlockUnknownShip,
  'no_game_data' => l.typeBlockNoGameData,
  'not_decoded' => l.typeBlockNotDecoded,
  'enemy_ship' => l.typeBlockEnemyShip,
  _ => code,
};

String difficultyBlockText(String code, AppLocalizations l) =>
    code == 'flagship' ? l.difficultyBlockFlagship : l.difficultyBlockNotDecoded;

/// The name of a ship type: the ship name and its layout letter from
/// [choices], else the blueprint id.
String shipTypeLabel(String id, List<ShipTypeChoice> choices, AppLocalizations l) {
  final c = choices.where((c) => c.id == id).firstOrNull;
  return c == null ? id : l.shipTypeLayout(c.name, c.layout);
}

/// The change sheet rows of the pending name, type and difficulty edits of
/// [player] and [nearby].
List<ChangeRow> identityChangeRows({
  required ShipIdentity? player,
  required ShipIdentity? nearby,
  required Map<String, String> texts,
  required Map<String, int> edits,
  required List<ShipTypeChoice> choices,
  required AppLocalizations l,
}) => [
  if (player != null) ...[
    if (texts[player.namePath] case final name? when name != player.name)
      ChangeRow(id: player.namePath, icon: Icons.badge, title: l.fieldShipName, change: (player.name, name)),
    if (texts[player.typePath] case final type? when type != player.blueprintId)
      ChangeRow(
        id: player.typePath,
        icon: Icons.rocket_launch,
        title: l.fieldShipType,
        change: (shipTypeLabel(player.blueprintId, choices, l), shipTypeLabel(type, choices, l)),
      ),
    if (edits[player.difficulty.path] case final d? when d != player.difficulty.value)
      ChangeRow(
        id: player.difficulty.path,
        icon: Icons.speed,
        title: l.fieldDifficulty,
        change: (difficultyLabel(player.difficulty.value, l), difficultyLabel(d, l)),
      ),
  ],
  if (nearby != null)
    if (texts[nearby.namePath] case final name? when name != nearby.name)
      ChangeRow(
        id: nearby.namePath,
        icon: Icons.badge,
        title: l.shipEnemyField(l.fieldShipName),
        change: (nearby.name, name),
      ),
];

Rect _anchor(BuildContext context) {
  final box = context.findRenderObject()! as RenderBox;
  return box.localToGlobal(Offset.zero) & box.size;
}

/// The ship of the save, or with [nearby] the enemy ship at the beacon: its
/// hull, its name with Rename, the blueprint id, and the tags of type
/// (player ship), sector and difficulty. A pending edit shows its new value
/// with a [ChangeMark]; the type and difficulty tags open their pickers.
class ShipCard extends StatelessWidget {
  const ShipCard({super.key, required this.summary, this.nearby = false, this.edits = const ShipCardEdits()});

  final SaveSummary summary;
  final bool nearby;
  final ShipCardEdits edits;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final t = AppTokens.of(context);
    final identity = edits.identity;
    final onText = edits.onText;
    final fileBlueprint = identity?.blueprintId ?? (nearby ? summary.nearbyBlueprintId ?? '' : summary.shipBlueprintId);
    final fileName = identity?.name ?? summary.shipName;
    final pendingName = identity == null ? null : edits.texts[identity.namePath];
    final nameEdited = pendingName != null && pendingName != fileName;
    final pendingType = identity == null || nearby ? null : edits.texts[identity.typePath];
    final typeEdited = pendingType != null && pendingType != fileBlueprint;
    final blueprint = typeEdited ? pendingType : fileBlueprint;
    final difficulty = identity?.difficulty;
    final fileDifficulty =
        difficulty?.value ??
        switch (summary.difficulty) {
          'easy' => 0,
          'hard' => 2,
          _ => 1,
        };
    final pendingDifficulty = difficulty == null ? null : edits.edits[difficulty.path];
    final difficultyEdited = pendingDifficulty != null && pendingDifficulty != fileDifficulty;
    final shownDifficulty = difficultyEdited ? pendingDifficulty : fileDifficulty;
    final difficultyBlock = identity?.difficultyBlock;
    final canDifficulty = difficulty != null && difficulty.max > difficulty.min && edits.onEdit != null;
    final choices = edits.typeChoices?.call() ?? const <ShipTypeChoice>[];

    Future<void> rename(BuildContext context) async {
      final name = await showRenamePopover(
        context,
        anchor: _anchor(context),
        current: nameEdited ? pendingName : fileName,
        title: l.shipRename,
      );
      if (name != null) onText!(identity!.namePath, name);
    }

    Future<void> pickDifficulty(BuildContext context) async {
      final n = await showDifficultyPicker(
        context,
        anchor: _anchor(context),
        current: shownDifficulty,
        max: difficulty!.max,
      );
      if (n != null) edits.onEdit!(difficulty.path, n);
    }

    Future<void> pickType(BuildContext context) async {
      final id = await showShipTypePicker(context, anchor: _anchor(context), current: blueprint, choices: choices);
      if (id != null) onText!(identity!.typePath, id);
    }

    final hull = GameImage(
      GameImageRef.of(GameImageKind.ship, blueprint),
      fallback: Icons.rocket_launch,
      width: 192,
      height: 120,
      iconSize: 48,
    );
    final nameRow = Row(
      children: [
        Flexible(
          child: AppText(
            nameEdited ? pendingName : fileName,
            key: const ValueKey('ship-name'),
            role: TextRole.display,
            color: nameEdited ? t.accent : null,
          ),
        ),
        if (nameEdited) ...[
          const SizedBox(width: Space.xs),
          const ChangeMark(EditDirection.raised, key: ValueKey('ship-name-mark')),
        ],
        if (onText != null && identity != null) ...[
          const SizedBox(width: Space.xs),
          Builder(
            builder: (context) => AppButton(
              key: const ValueKey('ship-rename'),
              icon: Icons.edit,
              tooltip: l.shipRename,
              onPressed: () => rename(context),
            ),
          ),
        ],
      ],
    );
    return Panel(
      child: Row(
        children: [
          hull,
          const SizedBox(width: Space.lg),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                nameRow,
                AppText(nearby ? l.shipEnemyField(blueprint) : blueprint, role: TextRole.caption),
                const SizedBox(height: Space.md),
                Wrap(
                  spacing: Space.sm,
                  runSpacing: Space.sm,
                  children: [
                    if (!nearby && identity != null)
                      Builder(
                        builder: (context) => Tag(
                          key: const ValueKey('ship-type'),
                          icon: choices.isEmpty ? Icons.rocket_launch : Icons.unfold_more,
                          label: shipTypeLabel(blueprint, choices, l),
                          marked: typeEdited,
                          tooltip: choices.isEmpty || onText == null ? null : l.shipChangeType,
                          onTap: choices.isEmpty || onText == null ? null : () => pickType(context),
                        ),
                      ),
                    Tag(
                      key: const ValueKey('ship-sector'),
                      icon: Icons.place,
                      label: l.sector(identity?.sector ?? summary.sectorNumber),
                      tooltip: l.shipSectorFixed,
                    ),
                    Builder(
                      builder: (context) => Tag(
                        key: const ValueKey('ship-difficulty'),
                        icon: canDifficulty ? Icons.unfold_more : Icons.speed,
                        label: difficultyLabel(shownDifficulty, l),
                        marked: difficultyEdited,
                        tooltip: canDifficulty
                            ? l.shipChangeDifficulty
                            : (difficultyBlock == null ? null : difficultyBlockText(difficultyBlock, l)),
                        onTap: canDifficulty ? () => pickDifficulty(context) : null,
                      ),
                    ),
                    if (summary.dlcEnabled) Tag(icon: Icons.extension, label: l.advancedEdition),
                    if (GameDataScope.maybeOf(context)?.datPath == null)
                      Tag(icon: Icons.image_not_supported, label: l.reducedMode),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// A popover under [anchor] with Easy, Normal and, up to [max] 2, Hard; the
/// row of [current] is marked. Answers the number, or `null` when dismissed.
Future<int?> showDifficultyPicker(
  BuildContext context, {
  required Rect anchor,
  required int current,
  required int max,
}) {
  final l = AppLocalizations.of(context)!;
  return showAnchoredPopover<int>(
    context,
    anchor: anchor,
    barrierLabel: l.cancel,
    width: 320,
    builder: (context) => AppPopover(
      key: const ValueKey('difficulty-picker'),
      icon: Icons.speed,
      title: l.fieldDifficulty,
      actions: [AppButton(kind: ButtonKind.text, label: l.cancel, onPressed: () => Navigator.of(context).pop())],
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          AppText(l.shipDifficultyNote, role: TextRole.caption, wrap: true),
          const SizedBox(height: Space.sm),
          for (var n = 0; n <= max; n++)
            ListRow(
              key: ValueKey('difficulty-choice-$n'),
              dense: true,
              selected: n == current,
              title: difficultyLabel(n, l),
              onTap: () => Navigator.of(context).pop(n),
            ),
        ],
      ),
    ),
  );
}

/// A popover under [anchor] with every hangar ship of [choices]: its hull,
/// its name and layout, and for a type the ship cannot take the reason
/// (the row does nothing). Answers the blueprint id, or `null`.
Future<String?> showShipTypePicker(
  BuildContext context, {
  required Rect anchor,
  required String current,
  required List<ShipTypeChoice> choices,
}) {
  final l = AppLocalizations.of(context)!;
  return showAnchoredPopover<String>(
    context,
    anchor: anchor,
    barrierLabel: l.cancel,
    width: 400,
    builder: (context) => AppPopover(
      key: const ValueKey('ship-type-picker'),
      icon: Icons.rocket_launch,
      title: l.fieldShipType,
      actions: [AppButton(kind: ButtonKind.text, label: l.cancel, onPressed: () => Navigator.of(context).pop())],
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: MediaQuery.sizeOf(context).height * 0.6),
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              AppText(l.shipTypeNote, role: TextRole.caption, wrap: true),
              const SizedBox(height: Space.sm),
              for (final c in choices)
                ListRow(
                  key: ValueKey('ship-type-choice-${c.id}'),
                  dense: true,
                  selected: c.id == current,
                  muted: c.block != null,
                  leading: GameImage(
                    GameImageRef.of(GameImageKind.ship, c.id),
                    fallback: Icons.rocket_launch,
                    width: 64,
                    height: 40,
                    iconSize: 24,
                  ),
                  title: l.shipTypeLayout(c.name, c.layout),
                  subtitle: c.block == null ? null : typeBlockText(c.block!, c.system, l),
                  onTap: c.block == null ? () => Navigator.of(context).pop(c.id) : null,
                ),
            ],
          ),
        ),
      ),
    ),
  );
}
