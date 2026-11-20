// The popovers of the floorplan: a room shows the level segments of its
// system (the row of the Equipment page), its fires and its breaches; a
// square shows a small menu under it (fire, breach, the crew standing there);
// a crew member shows the crew card of the Crew page. They edit the pending
// edits of the owner and redraw at once, because they read the same edits map.

import 'package:material_ui/material_ui.dart';

import '../../l10n/app_localizations.dart';
import '../../src/rust/api/crew.dart';
import '../../src/rust/api/editor.dart';
import '../../src/rust/api/equipment.dart';
import '../../src/rust/api/floorplan.dart';
import '../components/components.dart';
import '../crew_card.dart';
import '../crew_sprite.dart';
import '../equipment_page.dart';
import '../field_image.dart';
import '../game_image.dart';
import '../resource_bar.dart';
import 'floorplan_view.dart' show lowOxygen;

typedef OnChanged = void Function(String path, int value);

/// A dialog in the current look whose [build] runs again after every edit.
/// Without [framed] the body brings its own card.
Future<void> _showPopover(
  BuildContext context, {
  required Key key,
  required double width,
  bool framed = true,
  required Widget Function(BuildContext context, VoidCallback changed) build,
}) {
  // The dialog route sits above the page, outside its GameDataScope.
  final scope = GameDataScope.maybeOf(context);
  return showDialog<void>(
    context: context,
    builder: (context) => GameDataScope(
      datPath: scope?.datPath,
      load: scope?.load ?? loadGameImage,
      child: StatefulBuilder(
        builder: (context, setState) {
          // After an edit the page builds the new preview; the popover
          // builds again once that frame is done, so it reads it too.
          void changed() {
            setState(() {});
            WidgetsBinding.instance.addPostFrameCallback((_) {
              if (context.mounted) setState(() {});
            });
          }

          final body = build(context, changed);
          return Dialog(
            key: key,
            backgroundColor: Colors.transparent,
            elevation: 0,
            child: SizedBox(
              width: width,
              child: framed ? AppPopover(child: body) : body,
            ),
          );
        },
      ),
    ),
  );
}

/// The system of [room] with its level segments, each fire of the file with
/// "Put out" and each breach with "Seal". The system row is the
/// [SystemRow] of the Equipment page: [system] is the system the room shows
/// as the file has it (a staged install shows the level FTL gives it,
/// locked until the save), [equipment] the file's, [preview] answers the
/// equipment as the pending edits leave it now (it changes with every edit
/// made in the popover). [items] lists the weapons or drones of the ship
/// under Weapon Control or Drone Control with the rows of the Equipment
/// page ([ItemSlots.section]): `slots` answers them as the edits leave
/// them now.
Future<void> showRoomPopover(
  BuildContext context, {
  required FloorRoom room,
  required EquipmentSystem? system,
  required Equipment? equipment,
  required Equipment? Function() preview,
  ({ItemSlots? Function() slots, bool drones})? items,
  required Map<String, int> edits,
  required OnChanged onChanged,
}) => _showPopover(
  context,
  key: ValueKey('room-popover-${room.id}'),
  width: 520,
  build: (context, changed) {
    final l = AppLocalizations.of(context)!;
    void set(String path, int value) {
      onChanged(path, value);
      changed();
    }

    final oxygen = room.oxygenField;
    final name = system?.name ?? system?.id ?? l.floorRoom(room.id);
    // The slots edit through this popover, so it redraws after each edit.
    final base = items?.slots();
    final add = base?.onAddItem;
    final slots = base == null
        ? null
        : ItemSlots(
            file: base.file,
            preview: base.preview,
            onChanged: set,
            onAddItem: add == null
                ? null
                : (id, {required toCargo}) {
                    add(id, toCargo: toCargo);
                    changed();
                  },
            itemChoices: base.itemChoices,
            group: base.group,
          );
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // A system names the room in its own row.
        if (system == null || equipment == null)
          TitleRow(title: l.floorRoom(room.id))
        else
          SystemRow(system: system, equipment: equipment, preview: preview(), edits: edits, onChanged: set),
        if (slots != null) ...[const SizedBox(height: Space.sm), ...slots.section(context, drone: items!.drones)],
        _Oxygen(
          room: room,
          value: oxygen == null ? room.oxygen : edits[oxygen.path] ?? oxygen.value,
          onSet: oxygen == null
              ? null
              : (anchor) async {
                  final v = await showResourcePopover(
                    context,
                    anchor: anchor,
                    field: oxygen,
                    current: edits[oxygen.path] ?? oxygen.value,
                    look: (Icons.air, l.fieldOxygen(name)),
                    image: fieldImage(oxygen.path),
                  );
                  if (v != null) set(oxygen.path, v);
                },
        ),
        for (final q in room.squares)
          if (q.fire.value > 0)
            _Hazard(
              icon: Icons.local_fire_department,
              label: l.floorFire,
              action: l.floorPutOut,
              buttonKey: ValueKey('put-out-${q.fire.path}'),
              done: (edits[q.fire.path] ?? q.fire.value) == 0,
              onPressed: () => set(q.fire.path, 0),
            ),
        for (final q in room.squares)
          if (q.breach.value > 0)
            _Hazard(
              icon: Icons.broken_image_outlined,
              label: l.floorBreach,
              action: l.floorSeal,
              buttonKey: ValueKey('seal-${q.breach.path}'),
              done: (edits[q.breach.path] ?? q.breach.value) == 0,
              onPressed: () => set(q.breach.path, 0),
            ),
        const SizedBox(height: Space.sm),
        Align(
          alignment: Alignment.centerRight,
          child: AppButton(
            key: const ValueKey('popover-close'),
            label: l.floorClose,
            onPressed: () => Navigator.of(context).pop(),
          ),
        ),
      ],
    );
  },
);

/// What a pick in the square menu asks for next.
sealed class SquarePick {
  const SquarePick();
}

/// Open the system popover of the room.
class SystemPick extends SquarePick {
  const SystemPick();
}

/// Open the value popover of the oxygen of the room.
class OxygenPick extends SquarePick {
  const OxygenPick();
}

/// Open the card of the crew member [index].
class CrewPick extends SquarePick {
  const CrewPick(this.index);
  final int index;
}

/// The menu of one square, anchored under it: every option of the square at
/// once. The system of its room (when it has one), the [oxygen] of [room]
/// in percent, light or put out the fire, add or seal the breach, then the
/// crew of the room, those on [square] first. Fire and breach edit at once
/// and close the menu; the system, the oxygen (on a ship with the oxygen
/// system) and a crew member answer with a [SquarePick] for the caller to
/// open.
Future<SquarePick?> showSquareMenu(
  BuildContext context, {
  required Rect anchor,
  required String title,
  required FloorSquare square,
  required FloorRoom room,
  required int oxygen,
  required EquipmentSystem? system,
  required List<CrewMember> crew,
  required Map<String, int> edits,
  required OnChanged onChanged,
}) {
  final l = AppLocalizations.of(context)!;
  // The menu route sits above the page, outside its GameDataScope.
  final scope = GameDataScope.maybeOf(context);
  return showAnchoredPopover<SquarePick>(
    context,
    anchor: anchor,
    barrierLabel: l.cancel,
    width: 300,
    builder: (context) {
      final fire = edits[square.fire.path] ?? square.fire.value;
      final breach = edits[square.breach.path] ?? square.breach.value;
      void set(FieldValue f, int value) {
        onChanged(f.path, value);
        Navigator.of(context).pop();
      }

      final level = system?.level;
      return GameDataScope(
        datPath: scope?.datPath,
        load: scope?.load ?? loadGameImage,
        child: AppPopover(
          key: ValueKey('square-menu-${square.fire.path}'),
          title: title,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              if (system != null)
                ListRow(
                  key: const ValueKey('square-system'),
                  dense: true,
                  leading: GameIcon(systemIcon(system.id, systemIconState(system)), fallback: Icons.settings),
                  // The level beside the name: the title of the menu names the system already.
                  title: level == null
                      ? system.name ?? system.id
                      : '${system.name ?? system.id} ${edits[level.path] ?? level.value}/${level.max}',
                  marked: level != null && edits.containsKey(level.path),
                  onTap: () => Navigator.of(context).pop(const SystemPick()),
                ),
              ListRow(
                key: const ValueKey('square-oxygen'),
                dense: true,
                leading: const Icon(Icons.air),
                title: l.floorOxygenValue(oxygen),
                marked: room.oxygenField != null && edits.containsKey(room.oxygenField!.path),
                onTap: room.oxygenField == null ? null : () => Navigator.of(context).pop(const OxygenPick()),
              ),
              ListRow(
                key: const ValueKey('square-fire'),
                dense: true,
                leading: const Icon(Icons.local_fire_department),
                title: fire > 0 ? l.squarePutOutFire : l.squareLightFire,
                marked: edits.containsKey(square.fire.path),
                onTap: () => set(square.fire, fire > 0 ? 0 : square.fire.max),
              ),
              ListRow(
                key: const ValueKey('square-breach'),
                dense: true,
                leading: const Icon(Icons.broken_image_outlined),
                title: breach > 0 ? l.squareSealBreach : l.squareAddBreach,
                marked: edits.containsKey(square.breach.path),
                onTap: () => set(square.breach, breach > 0 ? 0 : square.breach.max),
              ),
              for (final c in crew)
                ListRow(
                  key: ValueKey('square-crew-${c.index}'),
                  dense: true,
                  leading: GameIcon(crewSprite(c.lookId), fallback: Icons.person),
                  title: c.name,
                  onTap: () => Navigator.of(context).pop(CrewPick(c.index)),
                ),
            ],
          ),
        ),
      );
    },
  );
}

/// The oxygen of a room in percent with a [ChangeMark] while it has a
/// pending edit, and Set, which opens the value popover under it; without
/// [onSet] (no oxygen system: FTL ignores the value) the row only shows it.
class _Oxygen extends StatelessWidget {
  const _Oxygen({required this.room, required this.value, required this.onSet});

  final FloorRoom room;
  final int value;
  final void Function(Rect anchor)? onSet;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final t = AppTokens.of(context);
    final f = room.oxygenField;
    final direction = f == null ? null : directionOf(f.value, value);
    final onSet = this.onSet;
    return Padding(
      padding: const EdgeInsets.only(top: Space.sm),
      child: Row(
        children: [
          Icon(Icons.air, color: value <= lowOxygen ? t.warn : t.accent),
          const SizedBox(width: Space.md),
          Expanded(
            child: AppText(
              l.floorOxygenValue(value),
              key: ValueKey('room-oxygen-${room.id}'),
              role: TextRole.label,
              caps: true,
            ),
          ),
          if (direction != null) ...[
            ChangeMark(direction, key: ValueKey('room-oxygen-mark-${room.id}')),
            const SizedBox(width: Space.sm),
          ],
          Builder(
            builder: (button) => AppButton(
              key: ValueKey('oxygen-set-${room.id}'),
              label: l.floorOxygenSet,
              onPressed: onSet == null
                  ? null
                  : () {
                      final box = button.findRenderObject()! as RenderBox;
                      onSet(box.localToGlobal(Offset.zero) & box.size);
                    },
            ),
          ),
        ],
      ),
    );
  }
}

/// One fire or breach: its name and the button that clears it; once
/// cleared, both are greyed out until the save.
class _Hazard extends StatelessWidget {
  const _Hazard({
    required this.icon,
    required this.label,
    required this.action,
    required this.buttonKey,
    required this.done,
    required this.onPressed,
  });

  final IconData icon;
  final String label;
  final String action;
  final Key buttonKey;
  final bool done;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final t = AppTokens.of(context);
    return Padding(
      padding: const EdgeInsets.only(top: Space.sm),
      child: Row(
        children: [
          Icon(icon, color: done ? t.muted : t.accent),
          const SizedBox(width: Space.md),
          Expanded(
            child: AppText(label, role: TextRole.label, caps: true, color: done ? t.muted : t.text),
          ),
          AppButton(key: buttonKey, label: action, onPressed: done ? null : onPressed),
        ],
      ),
    );
  }
}

/// The crew card of [member], as on the Crew page.
Future<void> showCrewPopover(
  BuildContext context, {
  required CrewMember member,
  required Map<String, int> edits,
  required OnChanged onChanged,
  CrewTexts texts = const CrewTexts(),
}) => _showPopover(
  context,
  key: ValueKey('crew-popover-${member.index}'),
  width: 360,
  build: (context, changed) => CrewCard(
    member: member,
    edits: edits,
    framed: false,
    onChanged: (path, value) {
      onChanged(path, value);
      changed();
    },
    texts: CrewTexts(
      values: texts.values,
      races: texts.races,
      onChanged: switch (texts.onChanged) {
        final set? => (path, text) {
          set(path, text);
          changed();
        },
        null => null,
      },
    ),
  ),
);
