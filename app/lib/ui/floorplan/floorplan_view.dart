// The floorplan of the Ship page: the player ship drawn from its layout and
// the images of ftl.dat, as `ftl_app::floorplan` places them. Pixel art
// scales by whole numbers only. A click on the system symbol of a room opens
// its popover (system level, fire, breaches); a click on a square opens the
// menu of that square (its system, fire, breach, the crew of the room); a
// click on a crew member opens the menu of their square too. Each weapon
// sits on its mount of the hull as FTL draws it (`ftl_app::floorplan::
// place_weapon`); a click on it, or on an empty mount the pointer shows,
// opens the chooser of its Equipment slot.
// Pure: the owner holds the edits, the same pending edits as on every page.

import 'package:flutter/foundation.dart' show mapEquals;
import 'package:material_ui/material_ui.dart';

import '../../l10n/app_localizations.dart';
import '../../look/ftl_look.dart';
import '../../src/rust/api/assets.dart';
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
import '../ship_choice.dart';
import 'floorplan_popovers.dart';

/// Every editable field of [plan]: the oxygen of each room (on a ship with
/// the oxygen system), the fire and the breach of each square.
List<FieldValue> floorplanFields(Floorplan? plan) => plan == null
    ? const []
    : [
        for (final r in plan.rooms) ...[
          ?r.oxygenField,
          for (final q in r.squares) ...[q.fire, q.breach],
        ],
      ];

/// Whether [f] is the fire or breach of a square that has none in the file:
/// an edit can light or add it, but a search does not list every square.
bool isQuietSquare(FieldValue f) =>
    (localPath(f.path).startsWith('fires.') || localPath(f.path).startsWith('breaches.')) && f.value == 0;

/// Icon and label of a floorplan path for the change sheet; `null` for any
/// other path. The room names its system when it has one.
(IconData, String)? floorplanLook(String path, AppLocalizations l, Floorplan? plan, Equipment? equipment) {
  if (plan == null) return null;
  String roomName(int? id) {
    final room = plan.rooms.where((r) => r.id == id).firstOrNull;
    final system = room?.system;
    final name = system == null ? null : equipment?.systems.where((s) => s.id == system).firstOrNull?.name;
    return name ?? system ?? l.floorRoom(id ?? -1);
  }

  for (final r in plan.rooms) {
    if (r.oxygenField?.path == path) return (Icons.air, l.fieldOxygen(roomName(r.id)));
    for (final q in r.squares) {
      if (q.fire.path == path) return (Icons.local_fire_department, l.fieldFire(roomName(r.id)));
      if (q.breach.path == path) return (Icons.broken_image_outlined, l.fieldBreach(roomName(r.id)));
    }
  }
  return null;
}

/// The first frame of `fire_large` and the last of `breach`
/// (data/animations.xml: `fire_L1_strip8.png` in 32x32 frames, `breach.png`
/// in seven 22x19 frames), as the Java editor draws them.
const fireSheet = GameImageRef.path('img/effects/fire_L1_strip8.png');
const breachSheet = GameImageRef.path('img/effects/breach.png');

/// The colors FTL multiplies the floor symbol of a system with
/// (`img/icons/s_<id>_overlay.png`, white with a black contour). Read from
/// `ShipSystem::OnRender` in the macOS binary of FTL 1.6.13: it draws the
/// overlay primitive with `GL_RenderPrimitiveWithColor` and the static
/// colors `COLOR_GRAY` 125,125,125, `COLOR_BLUE_ION` 133,231,237,
/// `COLOR_RED_PLAIN` 255,0,0 and `COLOR_ORANGE` 255,153,76.
const floorSymbolGray = Color(0xFF7D7D7D);
const floorSymbolIon = Color(0xFF85E7ED);
const floorSymbolRed = Color(0xFFFF0000);
const floorSymbolOrange = Color(0xFFFF994C);

/// The color of the floor symbol of [s] as FTL picks it: blue while ion
/// damage locks bars (FTL blinks it, the editor holds it), red with no bar
/// left, orange with some bars damaged, else gray. Gray also while the
/// systems are not read yet. The color-blind palette of FTL's settings is
/// not followed. [level], [damaged] and [ionized] override the file's
/// values with pending ones.
Color floorSymbolColor(EquipmentSystem? s, {int? level, int? damaged, int? ionized}) {
  if (s == null) return floorSymbolGray;
  final lvl = level ?? s.level?.value ?? 0;
  final broken = damaged ?? s.damaged;
  final ion = ionized ?? s.ionized;
  if (ion > 0 || ion == -1) return floorSymbolIon;
  if (broken >= lvl) return floorSymbolRed;
  if (broken > 0) return floorSymbolOrange;
  return floorSymbolGray;
}

/// The floor color FTL gives a room with [oxygen] percent.
///
/// FTL 1.6.13 `Room::OnRenderFloor` (macOS binary 0x100068890, Ghidra, read
/// 2026-11-19) draws the floor with `i = (int)((100 - oxygen) * 0.25)` and
/// the color `(230 + i, 226 - 2i, 219 - 2i)`; its alpha is that of the ship
/// floor (1 unless the ship fades). A full room is `E6E2DB`, an empty one
/// `FFB0A9`. A ship without the oxygen system stores 0 and FTL draws it so.
Color oxygenTint(int oxygen) {
  final i = ((100 - oxygen.clamp(0, 100)) * 0.25).truncate();
  return Color.fromARGB(255, 230 + i, 226 - 2 * i, 219 - 2 * i);
}

/// At or below this oxygen FTL draws the low-oxygen stripes over a room
/// (`Room::OnRenderO2Low`, the constant 5.0 at 0x1002b41f0).
const lowOxygen = 5;

/// The stripes `Room::Room` picks for a room of [w] x [h] pixels: 1x2 when
/// it is narrower than 36, else 2x1 when lower than 36, else 2x2. FTL draws
/// them at the top left of the room at half alpha (35 x 70, 70 x 35 and
/// 70 x 70 pixels).
({GameImageRef ref, int w, int h}) lowOxygenStripes(int w, int h) {
  final (name, sw, sh) = w < 36
      ? ('1x2', 35, 70)
      : h < 36
      ? ('2x1', 70, 35)
      : ('2x2', 70, 70);
  return (ref: GameImageRef.path('img/effects/low_o2_stripes_$name.png'), w: sw, h: sh);
}

/// The floorplan at the largest whole scale up to [maxScale] that fits the
/// width.
class FloorplanView extends StatefulWidget {
  const FloorplanView({
    super.key,
    required this.plan,
    required this.crew,
    required this.equipment,
    required this.edits,
    required this.onChanged,
    this.preview,
    this.previewEquipment,
    this.texts = const CrewTexts(),
    this.maxScale = 2,
    this.itemChoices = const [],
    this.onAddItem,
    this.group,
    this.hoveredMount,
  });

  /// Every weapon and drone of the open ftl.dat, for the slot chooser.
  final List<ItemChoice> itemChoices;

  /// Adds a weapon or drone to the slots ([ItemSlots.onAddItem]).
  final void Function(String id, {required bool toCargo})? onAddItem;

  /// Runs the edits of one replace as one step of Undo.
  final void Function(VoidCallback edits)? group;

  /// The mount the pointer rests on at start (for shots).
  final int? hoveredMount;

  /// The ship of the file: its squares carry the fields and file values.
  final Floorplan plan;
  final List<CrewMember> crew;

  /// Names and levels of the systems; `null` until read.
  final Equipment? equipment;

  /// The ship as the pending edits leave it (`floorplanPreview` of the
  /// core): the system of each room and its interior come from here, so a
  /// staged install or removal shows in the room, its symbol, its menu and
  /// its popover. `null` without edits or when the core refused them.
  final Floorplan? preview;

  /// The equipment as the pending edits leave it (`equipmentPreview`): name
  /// and level of a system the edits install.
  final Equipment? previewEquipment;
  final Map<String, int> edits;
  final void Function(String path, int value) onChanged;

  /// Name and race edits of the crew card.
  final CrewTexts texts;
  final int maxScale;

  @override
  State<FloorplanView> createState() => _FloorplanViewState();
}

class _FloorplanViewState extends State<FloorplanView> {
  int? _hovered;
  late int? _hoveredMount = widget.hoveredMount;

  /// The weapon and drone slots of the ship as the Equipment page has them;
  /// `null` until the equipment is read.
  ItemSlots? _slots() {
    final file = widget.equipment;
    if (file == null) return null;
    return ItemSlots(
      file: file,
      preview: widget.previewEquipment,
      onChanged: widget.onChanged,
      onAddItem: widget.onAddItem,
      itemChoices: widget.itemChoices,
      group: widget.group,
    );
  }

  int _value(FieldValue f) => widget.edits[f.path] ?? f.value;

  /// The oxygen [r] has with the pending edits: from the preview, else (no
  /// preview yet, or the core refused the edits) the pending value.
  int _oxygen(FloorRoom r) {
    if (widget.preview != null) return _now(r).oxygen;
    final f = r.oxygenField;
    return f == null ? r.oxygen : _value(f);
  }

  /// [r] as the pending edits leave it.
  FloorRoom _now(FloorRoom r) => widget.preview?.rooms.where((p) => p.id == r.id).firstOrNull ?? r;

  /// Whether the pending edits install a system in [r] that the file lacks
  /// there (also the Clone Bay that replaces the Medbay).
  bool _staged(FloorRoom r) => _now(r).system != null && _now(r).system != r.system;

  /// The system [r] holds with the pending edits: from the file equipment
  /// while it is the file's system (its level field compares the file value
  /// with the edit), from the preview equipment when the edits install it.
  EquipmentSystem? _system(FloorRoom r) {
    final id = _now(r).system;
    if (id == null) return null;
    final source = id == r.system ? widget.equipment : (widget.previewEquipment ?? widget.equipment);
    return source?.systems.where((s) => s.id == id).firstOrNull;
  }

  /// The system [r] shows with the pending edits and its interior.
  ({String id, List<FloorImage> interior, bool staged})? _shown(FloorRoom r) {
    final now = _now(r);
    final id = now.system;
    return id == null ? null : (id: id, interior: now.interior, staged: _staged(r));
  }

  /// The popover of [room]: the system it shows with the pending edits, as
  /// the file has it (a staged install is a system the file lists as not
  /// installed).
  void _openRoom(FloorRoom room) {
    final id = _now(room).system;
    showRoomPopover(
      context,
      room: room,
      system: id == null ? null : widget.equipment?.systems.where((s) => s.id == id).firstOrNull,
      equipment: widget.equipment,
      preview: () => widget.previewEquipment,
      items: switch (id) {
        'weapons' => (slots: _slots, drones: false),
        'drones' => (slots: _slots, drones: true),
        _ => null,
      },
      edits: widget.edits,
      onChanged: widget.onChanged,
    );
  }

  void _openCrew(int index) {
    final member = widget.crew.where((m) => m.index == index).firstOrNull;
    if (member != null) {
      showCrewPopover(context, member: member, edits: widget.edits, onChanged: widget.onChanged, texts: widget.texts);
    }
  }

  /// The menu of [q] under [square], the widget the click hit.
  Future<void> _openSquare(FloorRoom room, FloorSquare q, BuildContext square) async {
    final box = square.findRenderObject()! as RenderBox;
    final anchor = box.localToGlobal(Offset.zero) & box.size;
    final l = AppLocalizations.of(context)!;
    final name = _system(room)?.name ?? _now(room).system ?? l.floorRoom(room.id);
    // The crew of the room, those on this square first.
    final here = q.crew.toSet();
    final inRoom = [
      ...q.crew,
      for (final o in room.squares)
        for (final i in o.crew)
          if (!here.contains(i)) i,
    ];
    final pick = await showSquareMenu(
      context,
      anchor: anchor,
      title: name,
      square: q,
      room: room,
      oxygen: _oxygen(room),
      system: _system(room),
      crew: [for (final i in inRoom) ?widget.crew.where((m) => m.index == i).firstOrNull],
      edits: widget.edits,
      onChanged: widget.onChanged,
    );
    if (!mounted) return;
    switch (pick) {
      case SystemPick():
        _openRoom(room);
      case OxygenPick():
        final f = room.oxygenField!;
        final v = await showResourcePopover(
          context,
          anchor: anchor,
          field: f,
          current: _value(f),
          look: (Icons.air, l.fieldOxygen(name)),
          image: fieldImage(f.path),
        );
        if (v != null) widget.onChanged(f.path, v);
      case CrewPick(:final index):
        _openCrew(index);
      case null:
    }
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final plan = widget.plan;
      final s = (box.maxWidth / plan.width).floor().clamp(1, widget.maxScale);
      final d = s.toDouble();
      Widget image(GameImageRef ref, int x, int y, int w, int h, IconData fallback) => Positioned(
        left: x * d,
        top: y * d,
        width: w * d,
        height: h * d,
        child: IgnorePointer(
          child: GameImage(ref, fallback: fallback, width: w * d, height: h * d),
        ),
      );
      final hull = plan.hull;
      final floor = plan.floor;
      // The crew as the pending edits leave it: a new member stands on the
      // plan, a staged look shows, a staged removal is gone.
      final people = widget.preview ?? plan;
      // The weapons as the pending edits leave them, on the mounts of the file.
      final mounts = (widget.preview ?? plan).mounts;
      final squareOf = {
        for (final r in people.rooms)
          for (final q in r.squares)
            for (final c in q.crew) c: (r, q),
      };
      return Center(
        child: SizedBox(
          key: const ValueKey('floorplan'),
          width: plan.width * d,
          height: plan.height * d,
          child: Stack(
            clipBehavior: Clip.none,
            children: [
              // FTL draws the weapons before the hull: an unpowered one
              // slides back under it.
              for (final m in mounts)
                if (m.weapon != null) _mountArt(m, d),
              if (hull != null)
                image(GameImageRef.path(hull.innerPath), hull.x, hull.y, hull.w, hull.h, Icons.rocket_launch),
              if (floor != null)
                image(GameImageRef.path(floor.innerPath), floor.x, floor.y, floor.w, floor.h, Icons.grid_on),
              Positioned.fill(
                child: IgnorePointer(
                  child: CustomPaint(painter: _FloorPainter(plan, d, {for (final r in plan.rooms) r.id: _oxygen(r)})),
                ),
              ),
              for (final r in plan.rooms)
                if (_oxygen(r) <= lowOxygen) _stripes(r, d),
              for (final r in plan.rooms)
                for (final i in _shown(r)?.interior ?? const <FloorImage>[])
                  image(GameImageRef.path(i.innerPath), i.x, i.y, i.w, i.h, Icons.settings),
              Positioned.fill(
                child: IgnorePointer(child: CustomPaint(painter: _WallPainter(plan, d, _hovered))),
              ),
              for (final r in plan.rooms)
                for (final q in r.squares) _square(r, q, d),
              for (final r in plan.rooms) ..._symbol(r, d),
              for (final r in plan.rooms) ?_oxygenMark(r, d),
              for (final r in plan.rooms)
                for (final q in r.squares) ...[
                  if (_value(q.fire) > 0) _frame(fireSheet, 256, 32, 0, 32, 32, q.x + 1, q.y + 1, d),
                  if (_value(q.breach) > 0) _frame(breachSheet, 154, 19, 6 * 22, 22, 19, q.x + 6, q.y + 8, d),
                  ?_squareMark(r, q, d),
                ],
              for (final c in people.crew) _crew(c, squareOf[c.index], d),
              for (final m in mounts) ..._mount(m, d),
            ],
          ),
        ),
      );
    },
  );

  /// The first frame of the weapon on mount [m]: mirrored first, then
  /// turned a quarter clockwise, filling the box the core placed.
  Widget _mountArt(FloorMount m, double d) {
    final (fw, fh) = m.rotate ? (m.h, m.w) : (m.w, m.h);
    Widget art = GameImage(
      GameImageRef.of(GameImageKind.weaponFrame, m.weapon!),
      key: ValueKey('floor-weapon-${m.slot}'),
      fallback: Icons.gps_fixed,
      width: fw * d,
      height: fh * d,
    );
    if (m.mirror) art = Transform.flip(flipX: true, child: art);
    if (m.rotate) art = RotatedBox(quarterTurns: 1, child: art);
    return Positioned(
      left: m.x * d,
      top: m.y * d,
      width: m.w * d,
      height: m.h * d,
      child: IgnorePointer(child: art),
    );
  }

  /// The click target of mount [m] (`floor-mount-<slot>`): it opens the
  /// chooser of its weapon slot, as a click on the slot of the Equipment
  /// page. The pointer outlines it, so an empty mount shows where it is. A
  /// [ChangeMark] says the pending edits put another weapon there or
  /// change its power: raised for a new or powered weapon, lowered for a
  /// removed or unpowered one.
  List<Widget> _mount(FloorMount m, double d) {
    final file = widget.plan.mounts.where((f) => f.slot == m.slot).firstOrNull;
    final direction = file == null || (file.weapon == m.weapon && file.armed == m.armed)
        ? null
        : m.weapon == null || (file.weapon == m.weapon && !m.armed)
        ? EditDirection.lowered
        : EditDirection.raised;
    final hovered = _hoveredMount == m.slot;
    return [
      Positioned(
        left: m.x * d,
        top: m.y * d,
        width: m.w * d,
        height: m.h * d,
        child: MouseRegion(
          cursor: SystemMouseCursors.click,
          onEnter: (_) => setState(() => _hoveredMount = m.slot),
          onExit: (_) => setState(() => _hoveredMount = _hoveredMount == m.slot ? null : _hoveredMount),
          child: Builder(
            builder: (context) => GestureDetector(
              key: ValueKey('floor-mount-${m.slot}'),
              behavior: HitTestBehavior.opaque,
              onTap: () => _slots()?.openSlot(context, drone: false, slot: m.slot),
              child: hovered
                  ? DecoratedBox(
                      key: ValueKey('floor-mount-hover-${m.slot}'),
                      decoration: BoxDecoration(
                        color: m.weapon == null ? FtlColors.hover.withAlpha(0x30) : null,
                        border: Border.all(color: FtlColors.hover, width: d),
                      ),
                    )
                  : null,
            ),
          ),
        ),
      ),
      if (direction != null)
        Positioned(
          left: (m.x + m.w - 8) * d,
          top: m.y * d,
          child: IgnorePointer(
            child: ChangeMark(direction, key: ValueKey('mount-mark-${m.slot}'), size: 8 * d),
          ),
        ),
    ];
  }

  /// The click target of one square: it opens the menu of the square and
  /// highlights its room while the pointer is over it.
  Widget _square(FloorRoom r, FloorSquare q, double d) => Positioned(
    left: q.x * d,
    top: q.y * d,
    width: 35 * d,
    height: 35 * d,
    child: MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _hovered = r.id),
      onExit: (_) => setState(() => _hovered = _hovered == r.id ? null : _hovered),
      child: Builder(
        builder: (square) => GestureDetector(
          key: ValueKey('floor-square-${r.id}-${q.id}'),
          behavior: HitTestBehavior.opaque,
          onTap: () => _openSquare(r, q, square),
        ),
      ),
    ),
  );

  /// The symbol of the system [r] shows ([_shown]) in the middle of the
  /// room, the click target of its popover, in the color of its pending
  /// level, damage and ion ([floorSymbolColor]), and a [ChangeMark] in the
  /// top right corner of the room while the level, the damage or the ion of
  /// that system has a pending edit (the mark of its Equipment row: more
  /// damage or ion is lowered), or its install (raised) or removal
  /// (lowered) is staged. The popover of a staged system shows its level
  /// locked until the save.
  List<Widget> _symbol(FloorRoom r, double d) {
    final shown = _shown(r);
    final removed = r.system != null && _now(r).system != r.system;
    if (shown == null && !removed) return const [];
    final system = shown == null || shown.staged ? null : _system(r);
    final level = system?.level;
    final damage = system?.damage;
    final ion = system?.ion;
    final levelNow = level == null ? null : _value(level);
    final damageNow = damage == null ? null : _value(damage).clamp(0, levelNow ?? 0);
    final ionNow = ion == null ? null : _value(ion);
    final direction = shown == null
        ? EditDirection.lowered
        : shown.staged
        ? EditDirection.raised
        : (level == null ? null : directionOf(level.value, levelNow!)) ??
              (damage == null ? null : directionOf(damageNow!, damage.value)) ??
              (ion == null ? null : directionOf(ionNow!.abs(), ion.value.abs()));
    final markId = shown?.id ?? r.system!;
    return [
      if (shown != null)
        Positioned(
          left: (r.x + (r.w - 32) / 2) * d,
          top: (r.y + (r.h - 32) / 2) * d,
          width: 32 * d,
          height: 32 * d,
          child: MouseRegion(
            cursor: SystemMouseCursors.click,
            onEnter: (_) => setState(() => _hovered = r.id),
            onExit: (_) => setState(() => _hovered = _hovered == r.id ? null : _hovered),
            child: GestureDetector(
              key: ValueKey('floor-room-${r.id}'),
              behavior: HitTestBehavior.opaque,
              onTap: () => _openRoom(r),
              child: GameImage(
                GameImageRef.of(GameImageKind.systemIcon, shown.id),
                key: ValueKey('floor-symbol-${shown.id}'),
                fallback: Icons.settings,
                width: 32 * d,
                height: 32 * d,
                color: floorSymbolColor(system, level: levelNow, damaged: damageNow, ionized: ionNow),
              ),
            ),
          ),
        ),
      // Inside the 4 px wall, at the scale of the plan.
      if (direction != null)
        Positioned(
          left: (r.x + r.w - 6 - 8) * d,
          top: (r.y + 6) * d,
          child: IgnorePointer(
            child: ChangeMark(direction, key: ValueKey('floor-mark-$markId'), size: 8 * d),
          ),
        ),
    ];
  }

  /// FTL's low-oxygen stripes over [r] at half alpha, clipped to the room
  /// (FTL draws them at their own size from the top left of the room).
  Widget _stripes(FloorRoom r, double d) {
    final s = lowOxygenStripes(r.w, r.h);
    return Positioned(
      left: r.x * d,
      top: r.y * d,
      width: r.w * d,
      height: r.h * d,
      child: IgnorePointer(
        child: ClipRect(
          child: OverflowBox(
            alignment: Alignment.topLeft,
            minWidth: s.w * d,
            maxWidth: s.w * d,
            minHeight: s.h * d,
            maxHeight: s.h * d,
            child: Opacity(
              key: ValueKey('floor-o2-low-${r.id}'),
              opacity: 0.5,
              child: GameImage(s.ref, fallback: Icons.air, width: s.w * d, height: s.h * d),
            ),
          ),
        ),
      ),
    );
  }

  /// A [ChangeMark] in the top left corner of a room whose oxygen has a
  /// pending edit: raised when the edit adds oxygen.
  Widget? _oxygenMark(FloorRoom r, double d) {
    final f = r.oxygenField;
    final direction = f == null ? null : directionOf(f.value, _value(f));
    if (direction == null) return null;
    return Positioned(
      left: (r.x + 6) * d,
      top: (r.y + 6) * d,
      child: IgnorePointer(
        child: ChangeMark(direction, key: ValueKey('oxygen-mark-${r.id}'), size: 8 * d),
      ),
    );
  }

  /// A [ChangeMark] in the bottom left corner of a square whose fire or
  /// breach has a pending edit: raised when the edit lights or adds one.
  Widget? _squareMark(FloorRoom r, FloorSquare q, double d) {
    final direction = directionOf(q.fire.value, _value(q.fire)) ?? directionOf(q.breach.value, _value(q.breach));
    if (direction == null) return null;
    return Positioned(
      left: (q.x + 5) * d,
      top: (q.y + 35 - 5 - 8) * d,
      child: IgnorePointer(
        child: ChangeMark(direction, key: ValueKey('square-mark-${r.id}-${q.id}'), size: 8 * d),
      ),
    );
  }

  /// The standing frame of a crew member, centered on its sprite position.
  /// A click opens the menu of their square, which lists every option there.
  Widget _crew(FloorCrew c, (FloorRoom, FloorSquare)? at, double d) => Positioned(
    left: (c.x - 17.5) * d,
    top: (c.y - 17.5) * d,
    width: 35 * d,
    height: 35 * d,
    child: MouseRegion(
      cursor: SystemMouseCursors.click,
      child: Builder(
        builder: (figure) => GestureDetector(
          key: ValueKey('floor-crew-${c.index}'),
          behavior: HitTestBehavior.opaque,
          onTap: () => at == null ? _openCrew(c.index) : _openSquare(at.$1, at.$2, figure),
          child: CrewSprite(c.lookId, anchored: true, size: 35 * d),
        ),
      ),
    ),
  );

  /// One frame of a sprite sheet: the sheet at scale [d], moved left by the
  /// frame and clipped to it, its top left corner at [x], [y].
  Widget _frame(GameImageRef sheet, int sheetW, int sheetH, int frameX, int fw, int fh, int x, int y, double d) =>
      Positioned(
        left: x * d,
        top: y * d,
        width: fw * d,
        height: fh * d,
        child: IgnorePointer(
          child: ClipRect(
            child: OverflowBox(
              alignment: Alignment.topLeft,
              minWidth: sheetW * d,
              maxWidth: sheetW * d,
              minHeight: sheetH * d,
              maxHeight: sheetH * d,
              child: Transform.translate(
                offset: Offset(-frameX * d, 0),
                child: GameImage(sheet, fallback: Icons.warning, width: sheetW * d, height: sheetH * d),
              ),
            ),
          ),
        ),
      );
}

/// The floor of every room as FTL draws it: one tile per square in the floor
/// color of its oxygen ([oxygenTint]), a grid line between neighbouring squares. FTL draws the rooms
/// itself; the floor image of the ship (`<gfx>_floor.png`) lies below and
/// only frames them, and it may leave squares open where the layout changed
/// after the image was drawn (six squares of the Engi cruiser).
/// The grid line is the color of the Java editor (RoomSprite,
/// ShipInteriorComponent); a full room is its tile color `E6E2DB` too.
class _FloorPainter extends CustomPainter {
  _FloorPainter(this.plan, this.d, this.oxygen);

  final Floorplan plan;
  final double d;

  /// The oxygen each room shows, by room id.
  final Map<int, int> oxygen;

  static const line = Color(0xFF7D7D7D);

  @override
  void paint(Canvas canvas, Size size) {
    final lines = Paint()..color = line;
    for (final r in plan.rooms) {
      final tiles = Paint()..color = oxygenTint(oxygen[r.id] ?? 100);
      canvas.drawRect(Rect.fromLTWH(r.x * d, r.y * d, r.w * d, r.h * d), tiles);
      for (var y = r.y + 35; y < r.y + r.h; y += 35) {
        canvas.drawRect(Rect.fromLTWH((r.x + 1) * d, y * d, (r.w - 2) * d, d), lines);
      }
      for (var x = r.x + 35; x < r.x + r.w; x += 35) {
        canvas.drawRect(Rect.fromLTWH(x * d, (r.y + 1) * d, d, (r.h - 2) * d), lines);
      }
    }
  }

  @override
  bool shouldRepaint(_FloorPainter old) => old.plan != plan || old.d != d || !mapEquals(old.oxygen, oxygen);
}

/// The walls of every room and the doors in them: walls 4 pixels thick on
/// the room edges, as the Java editor draws them. A door gaps the wall and
/// shows as a bar across the gap.
class _WallPainter extends CustomPainter {
  _WallPainter(this.plan, this.d, this.hovered);

  final Floorplan plan;
  final double d;
  final int? hovered;

  static const wall = Color(0xFF0F0F0F);
  static const door = Color(0xFFE79A3B);

  @override
  void paint(Canvas canvas, Size size) {
    final walls = Paint()
      ..color = wall
      ..strokeWidth = 4 * d
      ..style = PaintingStyle.stroke;
    for (final r in plan.rooms) {
      canvas.drawRect(Rect.fromLTWH(r.x * d, r.y * d, r.w * d, r.h * d), walls);
    }
    final bars = Paint()..color = door;
    final gap = Paint()..color = wall;
    for (final dr in plan.doors) {
      // The middle 23 of 35 pixels, 4 thick, across the wall line.
      final rect = dr.vertical
          ? Rect.fromLTWH((dr.x - 2) * d, (dr.y + 6) * d, 4 * d, 23 * d)
          : Rect.fromLTWH((dr.x + 6) * d, (dr.y - 2) * d, 23 * d, 4 * d);
      canvas.drawRect(rect.inflate(d), gap);
      canvas.drawRect(rect, bars);
    }
    final hovered = plan.rooms.where((r) => r.id == this.hovered).firstOrNull;
    if (hovered != null) {
      canvas.drawRect(
        Rect.fromLTWH(hovered.x * d, hovered.y * d, hovered.w * d, hovered.h * d).deflate(d),
        Paint()
          ..color = FtlColors.hover
          ..strokeWidth = 2 * d
          ..style = PaintingStyle.stroke,
      );
    }
  }

  @override
  bool shouldRepaint(_WallPainter old) => old.plan != plan || old.d != d || old.hovered != hovered;
}

/// The floorplan in a window of the current look, with its title and hint.
class FloorplanCard extends StatelessWidget {
  const FloorplanCard({
    super.key,
    required this.plan,
    required this.crew,
    required this.equipment,
    required this.edits,
    required this.onChanged,
    this.preview,
    this.previewEquipment,
    this.texts = const CrewTexts(),
    this.itemChoices = const [],
    this.onAddItem,
    this.group,
  });

  final Floorplan plan;
  final List<CrewMember> crew;
  final Equipment? equipment;
  final Map<String, int> edits;
  final void Function(String path, int value) onChanged;
  final Floorplan? preview;
  final Equipment? previewEquipment;
  final CrewTexts texts;
  final List<ItemChoice> itemChoices;
  final void Function(String id, {required bool toCargo})? onAddItem;
  final void Function(VoidCallback edits)? group;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    return Panel(
      title: l.floorTitle,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          FloorplanView(
            plan: plan,
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
          const SizedBox(height: Space.sm),
          AppText(l.floorHint, role: TextRole.caption, wrap: true),
        ],
      ),
    );
  }
}
