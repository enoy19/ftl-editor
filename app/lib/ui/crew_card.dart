import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../look/ftl_look.dart';
import '../src/rust/api/crew.dart';
import '../src/rust/api/editor.dart';
import 'change_sheet.dart';
import 'components/components.dart';
import 'crew_page.dart';
import 'crew_sprite.dart';
import 'field_image.dart';
import 'game_image.dart';
import 'resource_bar.dart';

/// The pending text edits of the crew (`crew.<n>.name`, `crew.<n>.race`)
/// and the races the race picker offers. The owner holds them beside the
/// number edits; `onChanged` with the file value drops the edit. Without
/// [onChanged] name and race are read-only.
@immutable
class CrewTexts {
  const CrewTexts({
    this.values = const {},
    this.races = const [],
    this.onChanged,
    this.previews = const {},
    this.looksOf,
    this.onLook,
    this.newFrom,
    this.onRemoveNew,
  });

  final Map<String, String> values;
  final List<RaceChoice> races;
  final void Function(String path, String text)? onChanged;

  /// The crew members with a pending edit as the core would leave them
  /// (`crewPreview`), by index: the card shows their look, and with a
  /// pending race their skills and health.
  final Map<int, CrewMember> previews;

  /// The looks of FTL's character creator for a race (`lookChoices`);
  /// without it the card offers no look picker.
  final List<LookChoice> Function(String race)? looksOf;

  /// Stages a look: sex and tints of [member].
  final void Function(CrewMember member, LookChoice look)? onLook;

  /// The first index of a crew member that `crew.add` stages; `null` when
  /// none is staged.
  final int? newFrom;

  /// Drops the staged new crew member of this index.
  final void Function(int index)? onRemoveNew;

  bool isNew(CrewMember c) => newFrom != null && c.index >= newFrom!;
}

/// The white skill icon of FTL (`img/people/skill_<name>_white.png`, 26x26).
GameImageRef skillIcon(String skill) => GameImageRef.path(
  'img/people/skill_${switch (skill) {
    'engine' => 'engines',
    'shield' => 'shields',
    'weapon' => 'weapons',
    final s => s,
  }}_white.png',
);

/// The color of a mastery level as FTL draws the skill icons: white, green,
/// yellow.
Color masteryColor(int level, AppTokens t) => switch (level) {
  2 => t.accent,
  1 => const Color(0xFF6BE35F),
  _ => t.text,
};

/// The title of race [id]: the name from `ftl.dat` in [races], else the
/// app's own text, else the id.
String raceTitle(String id, AppLocalizations l, {List<RaceChoice> races = const []}) =>
    races.where((r) => r.id == id).firstOrNull?.name ?? raceText(id, l);

/// The change sheet rows of the pending name and race edits, after the
/// number edits of the same crew member.
List<ChangeRow> crewTextChangeRows(List<CrewMember> crew, CrewTexts texts, AppLocalizations l) => [
  for (final c in crew) ...[
    if (texts.values[c.namePath] case final name? when name != c.name)
      ChangeRow(
        id: c.namePath,
        icon: Icons.badge,
        image: fieldImage(c.namePath, crew: crew),
        title: l.crewField(c.name, l.fieldCrewName),
        change: (c.name, name),
      ),
    if (texts.values[c.racePath] case final race? when race != c.race)
      ChangeRow(
        id: c.racePath,
        icon: Icons.groups,
        image: fieldImage(c.racePath, crew: crew),
        title: l.crewField(c.name, l.fieldCrewRace),
        change: (raceLabel(c, l), raceTitle(race, l, races: texts.races)),
      ),
  ],
];

/// One crew member: portrait, name and race (both editable), health with
/// Heal and Train fully, and the six skills as a 3x2 grid of tiles (FTL's
/// skill icon in its mastery color, the full name, two level pips). The
/// Crew page frames it as a panel; the floorplan popover puts it into its
/// own frame ([framed] false). A pending edit draws a [ChangeMark] and a
/// tile outline in the accent (raised) or warning (lowered) color.
class CrewCard extends StatelessWidget {
  const CrewCard({
    super.key,
    required this.member,
    required this.edits,
    required this.onChanged,
    this.texts = const CrewTexts(),
    this.framed = true,
  });

  final CrewMember member;
  final Map<String, int> edits;
  final void Function(String path, int value) onChanged;
  final CrewTexts texts;
  final bool framed;

  int _value(FieldValue f) => edits[f.path] ?? f.value;

  /// The look the card shows: the pending one, else the file's.
  String get _lookId => texts.previews[member.index]?.lookId ?? member.lookId;

  /// A staged `crew.<n>.remove=1`.
  bool get _removed => edits[member.remove.path] == 1;

  /// This member as a pending race would leave it, else `null`.
  CrewMember? get _preview {
    final race = texts.values[member.racePath];
    final preview = texts.previews[member.index];
    return race == null || race == member.race || preview?.race != race ? null : preview;
  }

  Future<void> _editExact(BuildContext context, FieldValue field, (IconData, String) look) async {
    final v = await showResourcePopover(
      context,
      anchor: _anchor(context),
      field: field,
      current: _value(field),
      look: look,
      image: fieldImage(field.path, crew: [member]),
    );
    if (v != null) onChanged(field.path, v);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final preview = _preview;
    final skills = [
      for (final (i, s) in member.skills.indexed)
        _Skill(
          skill: s,
          points: _value(s.points),
          preview: preview?.skills[i],
          onStep: () => onChanged(s.points.path, nextMasteryPoints(_value(s.points), s.masteryPoints)),
          onExact: (context) {
            final (icon, label) = skillLook(s.skill, l);
            _editExact(context, s.points, (icon, l.crewField(member.name, label)));
          },
        ),
    ];
    final body = Column(
      key: ValueKey('crew-card-${member.index}'),
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _Header(card: this),
        const SizedBox(height: Space.lg),
        for (final (i, row) in [skills.sublist(0, 3), skills.sublist(3)].indexed) ...[
          if (i > 0) const SizedBox(height: Space.sm),
          Row(
            spacing: Space.sm,
            children: [for (final s in row) Expanded(child: _SkillTile(s))],
          ),
        ],
      ],
    );
    final shown = _removed ? Opacity(opacity: 0.45, child: body) : body;
    return framed ? Panel(child: shown) : shown;
  }
}

Rect _anchor(BuildContext context) {
  final box = context.findRenderObject()! as RenderBox;
  return box.localToGlobal(Offset.zero) & box.size;
}

/// Portrait, name with Rename, race with its picker, health bar, Heal and
/// Train fully.
class _Header extends StatelessWidget {
  const _Header({required this.card});

  final CrewCard card;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final t = AppTokens.of(context);
    final member = card.member;
    final texts = card.texts;
    final health = member.health;
    final hp = card._value(health);
    // A pending race refits health; the bar shows the share of the new race.
    final shownHealth =
        card._preview?.health ?? FieldValue(path: health.path, value: hp, min: health.min, max: health.max);
    final pendingName = texts.values[member.namePath];
    final pendingRace = texts.values[member.racePath];
    final nameEdited = pendingName != null && pendingName != member.name;
    final raceEdited = pendingRace != null && pendingRace != member.race;
    final race = raceEdited ? pendingRace : member.race;
    final onText = texts.onChanged;

    Future<void> rename(BuildContext context) async {
      final name = await showRenamePopover(
        context,
        anchor: _anchor(context),
        current: nameEdited ? pendingName : member.name,
        title: l.crewField(member.name, l.fieldCrewName),
      );
      if (name != null) onText!(member.namePath, name);
    }

    Future<void> pickRace(BuildContext context) async {
      final id = await showRacePicker(
        context,
        anchor: _anchor(context),
        current: race,
        races: texts.races,
        title: l.crewField(member.name, l.fieldCrewRace),
      );
      if (id != null) onText!(member.racePath, id);
    }

    final looks = texts.looksOf?.call(race) ?? const <LookChoice>[];
    final onLook = texts.onLook;
    final lookId = card._lookId;
    final lookEdited = lookId != member.lookId && !raceEdited;

    Future<void> pickLook(BuildContext context) async {
      final look = await showLookPicker(
        context,
        anchor: _anchor(context),
        current: lookId,
        looks: looks,
        title: l.crewField(member.name, l.crewLook),
      );
      if (look != null) onLook!(member, look);
    }

    final nameRow = Row(
      children: [
        Flexible(
          child: AppText(
            nameEdited ? pendingName : member.name,
            key: ValueKey('crew-name-${member.index}'),
            role: TextRole.title,
            caps: false,
            color: nameEdited ? t.accent : null,
          ),
        ),
        if (nameEdited) ...[
          const SizedBox(width: Space.xs),
          ChangeMark(EditDirection.raised, key: ValueKey('crew-name-mark-${member.index}')),
        ],
        if (onText != null) ...[
          const SizedBox(width: Space.xs),
          Builder(
            builder: (context) => AppButton(
              key: ValueKey('rename-${member.index}'),
              icon: Icons.edit,
              tooltip: l.crewRename,
              onPressed: () => rename(context),
            ),
          ),
        ],
      ],
    );
    final raceTag = Builder(
      builder: (context) => Tag(
        key: ValueKey('race-${member.index}'),
        label: raceTitle(race, l, races: texts.races),
        icon: onText == null ? null : Icons.unfold_more,
        marked: raceEdited,
        tooltip: onText == null ? null : l.crewChangeRace,
        onTap: onText == null ? null : () => pickRace(context),
      ),
    );
    final lookTag = Builder(
      builder: (context) => Tag(
        key: ValueKey('look-${member.index}'),
        label: l.crewLook,
        icon: Icons.palette_outlined,
        marked: lookEdited,
        tooltip: l.crewChangeLook,
        onTap: () => pickLook(context),
      ),
    );
    final isNew = texts.isNew(member);
    final removed = card._removed;
    final remove = member.remove;
    final buttons = Row(
      mainAxisSize: MainAxisSize.min,
      spacing: Space.xs,
      children: [
        AppButton(
          key: ValueKey('heal-${member.index}'),
          icon: Icons.healing,
          tooltip: l.crewHeal,
          onPressed: hp == health.max ? null : () => card.onChanged(health.path, health.max),
        ),
        AppButton(
          key: ValueKey('train-${member.index}'),
          icon: Icons.school,
          tooltip: l.crewTrainAll,
          onPressed: () {
            for (final s in member.skills) {
              if (card._value(s.points) != s.points.max) card.onChanged(s.points.path, s.points.max);
            }
          },
        ),
        if (isNew && texts.onRemoveNew != null)
          AppButton(
            key: ValueKey('remove-${member.index}'),
            icon: Icons.person_remove_outlined,
            tooltip: l.crewRemove,
            onPressed: () => texts.onRemoveNew!(member.index),
          )
        else if (!isNew)
          AppButton(
            key: ValueKey('remove-${member.index}'),
            icon: removed ? Icons.undo : Icons.person_remove_outlined,
            tooltip: removed
                ? l.crewKeep
                : remove.max == 0
                ? l.crewRemoveLast
                : l.crewRemove,
            onPressed: removed
                ? () => card.onChanged(remove.path, 0)
                : remove.max == 0
                ? null
                : () => card.onChanged(remove.path, 1),
          ),
      ],
    );
    final healthBar = Builder(
      builder: (context) => _HealthBar(
        key: ValueKey('health-${health.path}'),
        value: shownHealth.value,
        max: shownHealth.max,
        direction: directionOf(health.value, hp),
        tooltip: l.resourceTooltip(l.fieldCrewHealth, health.min, health.max),
        onTap: () => card._editExact(context, health, (Icons.favorite, l.crewField(member.name, l.fieldCrewHealth))),
      ),
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // A new or removed member shows it through the mark and, when
            // removed, the faded card; the word is only the tooltip, so no
            // card of a grid row grows by a line.
            Tooltip(
              key: isNew || removed ? ValueKey('crew-state-${member.index}') : null,
              message: removed
                  ? l.crewRemoved
                  : isNew
                  ? l.crewNew
                  : '',
              child: _Portrait(
                key: ValueKey('crew-portrait-${member.index}'),
                lookId: lookId,
                mark: removed
                    ? EditDirection.lowered
                    : isNew || lookEdited
                    ? EditDirection.raised
                    : null,
                markKey: ValueKey('crew-look-mark-${member.index}'),
              ),
            ),
            const SizedBox(width: Space.md),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  nameRow,
                  const SizedBox(height: Space.sm),
                  Wrap(
                    spacing: Space.xs,
                    runSpacing: Space.xs,
                    children: [raceTag, if (onLook != null && looks.length > 1) lookTag],
                  ),
                  const SizedBox(height: Space.sm),
                  buttons,
                ],
              ),
            ),
          ],
        ),
        const SizedBox(height: Space.md),
        healthBar,
      ],
    );
  }
}

/// The centered standing frame at three times its size in a dark well, as
/// the crew slots of FTL's ship menu hold their crew.
class _Portrait extends StatelessWidget {
  const _Portrait({super.key, required this.lookId, this.mark, this.markKey});

  final String lookId;
  final EditDirection? mark;
  final Key? markKey;

  /// The 35 px frame at three times its size keeps the pixels sharp.
  static const size = 105.0;

  @override
  Widget build(BuildContext context) {
    final t = AppTokens.of(context);
    final well = Container(
      width: size + 8,
      height: size + 8,
      decoration: BoxDecoration(
        color: FtlColors.ground,
        border: Border.all(color: t.muted, width: 2),
      ),
      alignment: Alignment.center,
      child: CrewSprite(lookId, size: size),
    );
    final mark = this.mark;
    if (mark == null) return well;
    return Stack(
      clipBehavior: Clip.none,
      children: [
        well,
        Positioned(right: -4, top: -4, child: ChangeMark(mark, key: markKey)),
      ],
    );
  }
}

/// Health as FTL's crew box shows it: a bar in a box (`health_box.png`),
/// green, yellow below two thirds, red below one third; the number beside.
class _HealthBar extends StatelessWidget {
  const _HealthBar({
    super.key,
    required this.value,
    required this.max,
    required this.direction,
    required this.tooltip,
    required this.onTap,
  });

  final int value;
  final int max;
  final EditDirection? direction;
  final String tooltip;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final t = AppTokens.of(context);
    final fraction = max == 0 ? 0.0 : (value / max).clamp(0.0, 1.0);
    final color = healthColor(fraction);
    return Tooltip(
      message: tooltip,
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: onTap,
          child: Row(
            children: [
              Icon(Icons.favorite, size: 14, color: color),
              const SizedBox(width: Space.xs),
              Expanded(
                child: Container(
                  height: 12,
                  decoration: BoxDecoration(border: Border.all(color: t.muted, width: 2)),
                  // A transparent inner border leaves FTL's gap between box and bar.
                  child: Container(
                    decoration: BoxDecoration(border: Border.all(color: Colors.transparent, width: 2)),
                    child: Align(
                      alignment: Alignment.centerLeft,
                      child: FractionallySizedBox(
                        widthFactor: fraction,
                        child: Container(color: color),
                      ),
                    ),
                  ),
                ),
              ),
              const SizedBox(width: Space.sm),
              if (direction case final d?) ...[ChangeMark(d), const SizedBox(width: Space.xs)],
              AppText(
                '$value/$max',
                role: TextRole.number,
                color: switch (direction) {
                  EditDirection.raised => t.accent,
                  EditDirection.lowered => t.warn,
                  null => null,
                },
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// One skill with its value.
/// One skill with its value. With a pending race, [preview] is the skill as
/// the core refits it: the tile shows its points and level, and the mark
/// compares that level with the level in the file. Edits stay in the units
/// of the file's race; the core applies them before the race.
class _Skill {
  const _Skill({required this.skill, required this.points, required this.onStep, required this.onExact, this.preview});

  final CrewSkill skill;
  final int points;
  final CrewSkill? preview;
  final VoidCallback onStep;
  final void Function(BuildContext context) onExact;

  int get shownPoints => preview?.points.value ?? points;
  int get shownMax => preview?.points.max ?? skill.points.max;
  int get level => masteryOf(shownPoints, preview?.masteryPoints ?? skill.masteryPoints);

  EditDirection? get direction {
    if (preview == null) return directionOf(skill.points.value, points);
    return directionOf(masteryOf(skill.points.value, skill.masteryPoints), level);
  }
}

/// A skill tile: FTL's skill icon in its mastery color with the edit mark,
/// the full name and two level pips. Click steps the mastery, long press
/// or right click edits the exact points.
class _SkillTile extends StatelessWidget {
  const _SkillTile(this.s);

  final _Skill s;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final t = AppTokens.of(context);
    final (icon, label) = skillLook(s.skill.skill, l);
    final outline = switch (s.direction) {
      EditDirection.raised => t.accent,
      EditDirection.lowered => t.warn,
      null => t.muted,
    };
    final tile = Container(
      padding: const EdgeInsets.symmetric(vertical: Space.sm),
      decoration: BoxDecoration(border: Border.all(color: outline, width: 2)),
      child: Column(
        children: [
          SizedBox(
            width: 26,
            height: 26,
            child: Stack(
              clipBehavior: Clip.none,
              children: [
                GameImage(
                  skillIcon(s.skill.skill),
                  fallback: icon,
                  width: 26,
                  height: 26,
                  iconSize: 18,
                  color: masteryColor(s.level, t),
                ),
                if (s.direction case final d?)
                  Positioned(
                    right: -4,
                    top: -4,
                    child: ChangeMark(d, key: ValueKey('skill-mark-${s.skill.points.path}')),
                  ),
              ],
            ),
          ),
          const SizedBox(height: Space.xs),
          AppText(label, role: TextRole.caption, align: TextAlign.center),
          const SizedBox(height: Space.xs),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            spacing: Space.xs,
            children: [
              for (var i = 1; i <= 2; i++)
                Container(
                  width: 10,
                  height: 6,
                  decoration: BoxDecoration(
                    color: s.level >= i ? masteryColor(i, t) : null,
                    border: Border.all(color: t.muted),
                  ),
                ),
            ],
          ),
        ],
      ),
    );
    return Tooltip(
      message: l.crewSkillTooltip(label, s.shownPoints, s.shownMax, s.level),
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        child: GestureDetector(
          key: ValueKey('skill-${s.skill.points.path}'),
          behavior: HitTestBehavior.opaque,
          onTap: s.onStep,
          onLongPress: () => s.onExact(context),
          onSecondaryTap: () => s.onExact(context),
          child: tile,
        ),
      ),
    );
  }
}

/// A popover under [anchor] with a text field for a crew name. Answers the
/// trimmed name, or `null` on Cancel. The core's rule: 1 to 32 characters.
Future<String?> showRenamePopover(
  BuildContext context, {
  required Rect anchor,
  required String current,
  required String title,
}) {
  final l = AppLocalizations.of(context)!;
  return showAnchoredPopover<String>(
    context,
    anchor: anchor,
    barrierLabel: l.cancel,
    width: 320,
    builder: (_) => _RenamePopover(current: current, title: title),
  );
}

class _RenamePopover extends StatefulWidget {
  const _RenamePopover({required this.current, required this.title});

  final String current;
  final String title;

  @override
  State<_RenamePopover> createState() => _RenamePopoverState();
}

class _RenamePopoverState extends State<_RenamePopover> {
  late final _ctrl = TextEditingController(text: widget.current)
    ..selection = TextSelection(baseOffset: 0, extentOffset: widget.current.length);
  String? _error;

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  void _apply() {
    final name = _ctrl.text.trim();
    if (name.isEmpty || name.runes.length > 32) {
      setState(() => _error = AppLocalizations.of(context)!.crewNameRule);
      return;
    }
    Navigator.of(context).pop(name);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    return AppPopover(
      key: const ValueKey('rename-popover'),
      icon: Icons.badge,
      title: widget.title,
      actions: [
        AppButton(kind: ButtonKind.text, label: l.cancel, onPressed: () => Navigator.of(context).pop()),
        AppButton(key: const ValueKey('rename-apply'), kind: ButtonKind.tonal, label: l.apply, onPressed: _apply),
      ],
      child: AppTextField(
        key: const ValueKey('rename-field'),
        controller: _ctrl,
        autofocus: true,
        helper: l.crewNameRule,
        error: _error,
        onChanged: (_) {
          if (_error != null) setState(() => _error = null);
        },
        onSubmitted: (_) => _apply(),
      ),
    );
  }
}

/// A popover under [anchor] with every race of [races] as a row with its
/// portrait and title; the row of [current] is marked. Answers the race id,
/// or `null` when dismissed.
Future<String?> showRacePicker(
  BuildContext context, {
  required Rect anchor,
  required String current,
  required List<RaceChoice> races,
  required String title,
}) {
  final l = AppLocalizations.of(context)!;
  // The popover route lies above the page, outside its GameDataScope.
  final scope = GameDataScope.maybeOf(context);
  return showAnchoredPopover<String>(
    context,
    anchor: anchor,
    barrierLabel: l.cancel,
    width: 320,
    builder: (context) => GameDataScope(
      datPath: scope?.datPath,
      load: scope?.load ?? loadGameImage,
      child: AppPopover(
        key: const ValueKey('race-picker'),
        icon: Icons.groups,
        title: title,
        actions: [AppButton(kind: ButtonKind.text, label: l.cancel, onPressed: () => Navigator.of(context).pop())],
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            AppText(l.crewRaceNote, role: TextRole.caption, wrap: true),
            const SizedBox(height: Space.sm),
            for (final r in races)
              ListRow(
                key: ValueKey('race-choice-${r.id}'),
                dense: true,
                selected: r.id == current,
                leading: CrewSprite(r.lookId),
                title: raceTitle(r.id, l, races: races),
                onTap: () => Navigator.of(context).pop(r.id),
              ),
          ],
        ),
      ),
    ),
  );
}

/// A popover under [anchor] with every look of FTL's character creator for
/// the race ([looks]) as a grid of portraits; the look of [current] is
/// marked. Answers the picked look, or `null` when dismissed.
Future<LookChoice?> showLookPicker(
  BuildContext context, {
  required Rect anchor,
  required String current,
  required List<LookChoice> looks,
  required String title,
}) {
  final l = AppLocalizations.of(context)!;
  // The popover route lies above the page, outside its GameDataScope.
  final scope = GameDataScope.maybeOf(context);
  return showAnchoredPopover<LookChoice>(
    context,
    anchor: anchor,
    barrierLabel: l.cancel,
    width: 360,
    builder: (context) {
      final t = AppTokens.of(context);
      return GameDataScope(
        datPath: scope?.datPath,
        load: scope?.load ?? loadGameImage,
        child: AppPopover(
          key: const ValueKey('look-picker'),
          icon: Icons.palette_outlined,
          title: title,
          actions: [AppButton(kind: ButtonKind.text, label: l.cancel, onPressed: () => Navigator.of(context).pop())],
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              AppText(l.crewLookNote, role: TextRole.caption, wrap: true),
              const SizedBox(height: Space.sm),
              Wrap(
                spacing: Space.sm,
                runSpacing: Space.sm,
                children: [
                  for (final (i, look) in looks.indexed)
                    Tooltip(
                      message: look.male ? l.crewMale : l.crewFemale,
                      child: MouseRegion(
                        cursor: SystemMouseCursors.click,
                        child: GestureDetector(
                          key: ValueKey('look-choice-$i'),
                          onTap: () => Navigator.of(context).pop(look),
                          child: Container(
                            padding: const EdgeInsets.all(Space.xs),
                            decoration: BoxDecoration(
                              color: FtlColors.ground,
                              border: Border.all(color: look.lookId == current ? t.accent : t.muted, width: 2),
                            ),
                            child: CrewSprite(look.lookId, size: 70),
                          ),
                        ),
                      ),
                    ),
                ],
              ),
            ],
          ),
        ),
      );
    },
  );
}
