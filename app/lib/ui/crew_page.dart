import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../src/rust/api/crew.dart';
import '../src/rust/api/editor.dart';
import '../src/rust/api/equipment.dart';
import '../look/ftl_look.dart';
import 'change_sheet.dart';
import 'components/components.dart';
import 'crew_card.dart';
import 'equipment_page.dart';
import 'resource_bar.dart';
import 'ship_page.dart';

export 'crew_card.dart' show CrewCard, CrewTexts, crewTextChangeRows;

/// The mastery level 0, 1 or 2 of [points], given the points of each level
/// as the core answers them (`CrewSkill.masteryPoints`).
int masteryOf(int points, List<int> masteryPoints) => points >= masteryPoints[2]
    ? 2
    : points >= masteryPoints[1]
    ? 1
    : 0;

/// The points one click on a skill ring sets: the next mastery level, and
/// level 2 back to 0, each on the interval value of the race.
int nextMasteryPoints(int points, List<int> masteryPoints) => masteryPoints[(masteryOf(points, masteryPoints) + 1) % 3];

/// Every editable field of the crew, in card order: health, the skills,
/// then the removal (`crew.<n>.remove`).
List<FieldValue> crewFields(List<CrewMember> crew) => [
  for (final c in crew) ...[c.health, for (final s in c.skills) s.points, c.remove],
];

final _crewListPath = RegExp(r'^(nearby\.)?crew\.(add|\d+\.(remove|male|tints\.\d+))$');

/// Whether [path] adds or removes crew or sets a crew look: the change sheet
/// names these in words ([crewListChangeRows]), not as numbers.
bool isCrewListPath(String path) => _crewListPath.hasMatch(path);

/// The change sheet rows of new and removed crew members and of the crew
/// look in [edits], for the crew of one ship (file crew and staged new
/// members) and its `crew.add` field.
List<ChangeRow> crewListChangeRows(List<CrewMember> crew, FieldValue? add, Map<String, int> edits, AppLocalizations l) {
  final prefix = add?.path.startsWith('nearby.') == true ? 'nearby.' : '';
  return [
    if (add != null && (edits[add.path] ?? 0) > 0)
      ChangeRow(
        id: add.path,
        icon: Icons.person_add_outlined,
        title: l.fieldCrewAdd,
        change: ('0', '${edits[add.path]}'),
      ),
    for (final c in crew) ...[
      if (edits[c.remove.path] == 1)
        ChangeRow(
          id: c.remove.path,
          icon: Icons.person_remove_outlined,
          title: l.crewField(c.name, l.fieldCrewRemove),
          change: (l.crewAboard, l.crewRemoved),
        ),
      if (edits['${prefix}crew.${c.index}.male'] case final male? when male != c.sex.value)
        ChangeRow(
          id: '${prefix}crew.${c.index}.male',
          icon: Icons.wc,
          title: l.crewField(c.name, l.fieldCrewSex),
          change: (c.male ? l.crewMale : l.crewFemale, male == 1 ? l.crewMale : l.crewFemale),
        ),
      for (final e in edits.entries)
        if (_tintLayer(e.key, '${prefix}crew.${c.index}.tints.') case final layer?)
          ChangeRow(
            id: e.key,
            icon: Icons.palette_outlined,
            title: l.crewField(c.name, l.fieldCrewColor(layer + 1)),
            change: (layer < c.tints.length ? l.crewColor(c.tints[layer].value + 1) : '-', l.crewColor(e.value + 1)),
          ),
    ],
  ];
}

int? _tintLayer(String path, String base) => path.startsWith(base) ? int.tryParse(path.substring(base.length)) : null;

/// Icon and label of a skill id of the core.
(IconData, String) skillLook(String skill, AppLocalizations l) => switch (skill) {
  'pilot' => (Icons.navigation, l.skillPilot),
  'engine' => (Icons.local_fire_department, l.skillEngines),
  'shield' => (Icons.shield, l.skillShields),
  'weapon' => (Icons.gps_fixed, l.skillWeapons),
  'repair' => (Icons.build, l.skillRepair),
  'combat' => (Icons.sports_mma, l.skillCombat),
  _ => (Icons.star_outline, skill),
};

/// The race title: from `ftl.dat` when it is open, else the app's own text,
/// else the race id.
String raceLabel(CrewMember c, AppLocalizations l) => c.raceName ?? raceText(c.race, l);

/// The app's own title of race [id], else the id.
String raceText(String id, AppLocalizations l) => switch (id) {
  'human' => l.raceHuman,
  'engi' => l.raceEngi,
  'mantis' => l.raceMantis,
  'rock' => l.raceRock,
  'energy' => l.raceZoltan,
  'slug' => l.raceSlug,
  'crystal' => l.raceCrystal,
  'anaerobic' => l.raceLanius,
  'ghost' => l.raceGhost,
  'battle' => l.raceDrone,
  _ => id,
};

/// Icon and label of any field path: a crew field names the crew member, an
/// equipment field its system or augment ([equipmentLook]), every other path
/// is a resource ([resourceLook]).
(IconData, String) fieldLook(
  String path,
  AppLocalizations l, {
  List<CrewMember> crew = const [],
  Equipment? equipment,
  List<AugmentChoice> choices = const [],
}) {
  for (final c in crew) {
    if (path == c.health.path) return (Icons.favorite_border, l.crewField(c.name, l.fieldCrewHealth));
    if (path == c.namePath) return (Icons.badge, l.crewField(c.name, l.fieldCrewName));
    if (path == c.racePath) return (Icons.groups, l.crewField(c.name, l.fieldCrewRace));
    if (path == c.remove.path) return (Icons.person_remove_outlined, l.crewField(c.name, l.fieldCrewRemove));
    for (final s in c.skills) {
      if (path == s.points.path) {
        final (icon, label) = skillLook(s.skill, l);
        return (icon, l.crewField(c.name, label));
      }
    }
  }
  return equipmentLook(path, l, equipment, choices) ?? resourceLook(path, l);
}

/// Colors of the three mastery levels, as FTL shows them (white, green, yellow).
Color levelColor(int level, ColorScheme scheme) => switch (level) {
  2 => const Color(0xFFF2C12E),
  1 => const Color(0xFF5DBB63),
  _ => scheme.outline,
};

Color healthColor(double fraction) => fraction > 0.66
    ? const Color(0xFF5DBB63)
    : fraction > 0.33
    ? const Color(0xFFF2C12E)
    : const Color(0xFFE5533D);

/// The Crew page: one [CrewCard] per crew member, up to four per row
/// (decision ftl-editor-uses-top-tabs-and-portrait-crew-cards). Pure: the
/// owner holds the edits, which are the same pending edits as on the Ship
/// page, and the text edits of name and race ([texts]), so Save lists them
/// in the change sheet.
class CrewPage extends StatelessWidget {
  const CrewPage({
    super.key,
    required this.crew,
    required this.edits,
    required this.onChanged,
    this.texts = const CrewTexts(),
    this.add,
    this.shipSwitch,
  });

  /// The choice of the ship shown, above the cards; `null` without an
  /// enemy ship.
  final Widget? shipSwitch;

  /// `null` while no file is open. Staged new crew members come last.
  final List<CrewMember>? crew;

  /// `crew.add` of the ship shown; `null` hides the card with the plus.
  final FieldValue? add;

  /// New values by path; a path without an entry shows the file value.
  final Map<String, int> edits;
  final void Function(String path, int value) onChanged;
  final CrewTexts texts;

  static const gap = Space.xl;
  static const minCardWidth = 280.0;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final crew = this.crew;
    final shipSwitch = this.shipSwitch;
    if (crew == null) return const SizedBox.shrink();
    if (crew.isEmpty) {
      return HintedList(
        children: [
          if (shipSwitch != null) ...[shipSwitch, const SizedBox(height: Space.lg)],
          AppText(l.crewNone, color: AppTokens.of(context).muted, align: TextAlign.center),
        ],
      );
    }
    return LayoutBuilder(
      builder: (context, box) {
        final width = box.maxWidth - HintedList.padding.horizontal;
        final columns = ((width + gap) ~/ (minCardWidth + gap)).clamp(1, 4);
        final cardWidth = (width - (columns - 1) * gap) / columns;
        return HintedList(
          hints: [HintRow(text: l.crewSkillHint)],
          children: [
            if (shipSwitch != null) ...[shipSwitch, const SizedBox(height: Space.lg)],
            // Rows of [columns] cards, each row as high as its tallest card:
            // the plus card and a card with fewer lines fill the row.
            for (final (i, row) in _rows([
              for (final c in crew) CrewCard(member: c, edits: edits, onChanged: onChanged, texts: texts),
              if (add case final add?)
                _AddCrewCard(
                  staged: edits[add.path] ?? 0,
                  max: add.max,
                  onAdd: () => onChanged(add.path, (edits[add.path] ?? 0) + 1),
                ),
            ], columns).indexed) ...[
              if (i > 0) const SizedBox(height: gap),
              IntrinsicHeight(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  spacing: gap,
                  children: [for (final card in row) SizedBox(width: cardWidth, child: card)],
                ),
              ),
            ],
          ],
        );
      },
    );
  }
}

/// [cards] in rows of [columns].
List<List<Widget>> _rows(List<Widget> cards, int columns) => [
  for (var i = 0; i < cards.length; i += columns)
    cards.sublist(i, i + columns > cards.length ? cards.length : i + columns),
];

/// The card at the end of the crew grid: a plus in the portrait well of a
/// crew card. A click stages one more default human (`crew.add`); at FTL's
/// crew limit the card is disabled and says why.
class _AddCrewCard extends StatelessWidget {
  const _AddCrewCard({required this.staged, required this.max, required this.onAdd});

  final int staged;
  final int max;
  final VoidCallback onAdd;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final t = AppTokens.of(context);
    final enabled = staged < max;
    final card = Panel(
      child: ConstrainedBox(
        constraints: const BoxConstraints(minHeight: 220),
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 113,
                height: 113,
                decoration: BoxDecoration(
                  color: FtlColors.ground,
                  border: Border.all(color: enabled ? t.accent : t.muted, width: 2),
                ),
                alignment: Alignment.center,
                child: Icon(Icons.add, size: 56, color: enabled ? t.accent : t.muted),
              ),
              const SizedBox(height: Space.md),
              AppText(l.crewAdd, role: TextRole.label, color: enabled ? null : t.muted),
            ],
          ),
        ),
      ),
    );
    return Tooltip(
      message: enabled ? l.crewAdd : l.crewAddFull,
      child: MouseRegion(
        cursor: enabled ? SystemMouseCursors.click : SystemMouseCursors.basic,
        child: GestureDetector(
          key: const ValueKey('crew-add'),
          behavior: HitTestBehavior.opaque,
          onTap: enabled ? onAdd : null,
          child: card,
        ),
      ),
    );
  }
}
