// The sector tree of the Map page, drawn like FTL's sector map
// (`StarMap::RenderSectorMap`): a line per join, a ring by state and a disc
// by type per sector, the ship marker on the current sector, the names of
// the current and the next sectors. A click on a sector opens its popover
// with the jump (`sector.jump`). The core answers the tree and every rule
// (ftl_core::sector_tree); the page stages the jump in the owner's edits.

import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../look/ftl_look.dart';
import '../src/rust/api/map.dart';
import 'change_sheet.dart';
import 'components/components.dart';
import 'game_image.dart';
import 'map_page.dart' show MapImages;

/// The path of the jump.
const sectorJumpPath = 'sector.jump';

/// FTL's colors of the sector map (`STARMAP_GREEN`, `_RED`, `_PURPLE`).
abstract final class SectorColors {
  static const civilian = Color(0xFF87C74A);
  static const hostile = Color(0xFFD63232);
  static const nebula = Color(0xFF8033D2);
  static const visited = Color(0xFFFFFF32);
  static const reachable = Color(0xFFFFFFFF);
  static const unreachable = Color(0xFF7D7D7D);

  static Color kind(String kind) => switch (kind) {
    'hostile' => hostile,
    'nebula' => nebula,
    _ => civilian,
  };
}

/// The text of a jump block or tree code of the core.
String jumpBlockText(String code, AppLocalizations l) => switch (code) {
  'jump_last_stand' => l.sectorJumpLastStand,
  'jump_crew_aboard' => l.mapMoveBlockCrew,
  'jump_distant_quests' => l.sectorJumpQuests,
  'jump_hidden_sector' => l.sectorJumpHidden,
  'tree_native_random' => l.sectorTreeNative,
  'tree_no_game_data' => l.sectorTreeNoGameData,
  'tree_count_mismatch' || 'tree_catalog' => l.sectorTreeMismatch,
  _ => l.mapMoveBlockUnknown,
};

String sectorKindLabel(String kind, AppLocalizations l) => switch (kind) {
  'hostile' => l.sectorHostile,
  'nebula' => l.sectorNebula,
  _ => l.sectorCivilian,
};

/// The change sheet row of a pending jump.
List<ChangeRow> sectorJumpChangeRows(SectorTreeView? tree, Map<String, int> edits, AppLocalizations l) {
  final to = edits[sectorJumpPath];
  final field = tree?.jumpField;
  if (tree == null || to == null || field == null || to == field.value) return const [];
  String label(int n) {
    final s = tree.sectors[n];
    return '${l.sector(s.level + 1)}: ${s.name}';
  }

  return [
    ChangeRow(
      id: sectorJumpPath,
      icon: Icons.travel_explore,
      title: l.sectorJumpField,
      change: (label(field.value), label(to)),
    ),
  ];
}

/// The margin around the tree in drawing pixels: room for the ring, the
/// ship and the names.
const _padLeft = 40.0;
const _padRight = 120.0;
const _padY = 40.0;

/// The sector tree. [tree] is the file's, [shown] the tree as the pending
/// edits leave it.
class SectorTreeCanvas extends StatelessWidget {
  const SectorTreeCanvas({
    super.key,
    required this.tree,
    required this.shown,
    required this.edits,
    required this.onChanged,
  });

  final SectorTreeView tree;
  final SectorTreeView shown;
  final Map<String, int> edits;
  final void Function(String path, int value) onChanged;

  @override
  Widget build(BuildContext context) {
    final maxX = shown.sectors.fold<int>(0, (m, s) => s.x > m ? s.x : m);
    final maxY = shown.sectors.fold<int>(0, (m, s) => s.y > m ? s.y : m);
    final canvas = Size(maxX + _padLeft + _padRight, maxY + 2 * _padY);
    return LayoutBuilder(
      builder: (context, box) {
        final scale = (box.maxWidth / canvas.width).clamp(0.2, box.maxHeight / canvas.height);
        final size = Size(box.maxWidth / scale, box.maxHeight / scale);
        final origin = Offset(_padLeft + (size.width - canvas.width) / 2, _padY + (size.height - canvas.height) / 2);
        Offset at(TreeSectorView s) => origin + Offset(s.x.toDouble(), s.y.toDouble());
        final current = shown.sectors.where((s) => s.current).firstOrNull;
        // The drawing scales by a fraction; names, marks and the click
        // targets stand above it at screen scale, so bitmap text stays sharp.
        return ClipRect(
          child: Stack(
            children: [
              const Positioned.fill(child: ColoredBox(color: FtlColors.ground)),
              Positioned.fill(
                child: FittedBox(
                  fit: BoxFit.fill,
                  child: SizedBox.fromSize(
                    size: size,
                    child: Stack(
                      children: [
                        Positioned.fill(child: CustomPaint(painter: _TreePainter(shown, at))),
                        if (current != null)
                          Positioned(
                            left: at(current).dx - 32,
                            top: at(current).dy - 58,
                            child: const IgnorePointer(
                              child: GameImage(
                                MapImages.ship,
                                fallback: Icons.navigation,
                                width: 64,
                                height: 64,
                                iconSize: 24,
                              ),
                            ),
                          ),
                      ],
                    ),
                  ),
                ),
              ),
              for (final s in shown.sectors) ..._overlay(context, s, at(s) * scale, scale),
            ],
          ),
        );
      },
    );
  }

  List<Widget> _overlay(BuildContext context, TreeSectorView s, Offset p, double scale) {
    final current = shown.sectors.where((s) => s.current).firstOrNull;
    final named = s.current || (current?.next.contains(s.index) ?? false);
    final moved = s.current && !tree.sectors[s.index].current;
    final hit = (14 * scale).clamp(12.0, 28.0);
    return [
      Positioned(
        left: p.dx - hit,
        top: p.dy - hit,
        width: 2 * hit,
        height: 2 * hit,
        child: Builder(
          builder: (context) => MouseRegion(
            cursor: SystemMouseCursors.click,
            child: GestureDetector(
              key: ValueKey('sector-${s.index}'),
              behavior: HitTestBehavior.opaque,
              onTap: () => showSectorPopover(
                context,
                anchor: _rectOf(context),
                sector: s,
                tree: tree,
                shown: shown,
                onChanged: onChanged,
              ),
            ),
          ),
        ),
      ),
      if (named)
        Positioned(
          left: p.dx + 10 * scale,
          top: p.dy + 6 * scale,
          child: IgnorePointer(
            // A dark backing keeps the name readable over the joins.
            child: ColoredBox(
              color: FtlColors.ground,
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: Space.xs),
                child: AppText(
                  s.shortName,
                  role: TextRole.caption,
                  color: s.current ? SectorColors.visited : FtlColors.line,
                ),
              ),
            ),
          ),
        ),
      if (moved)
        Positioned(left: p.dx - 22 * scale, top: p.dy - 20 * scale, child: const ChangeMark(EditDirection.raised)),
    ];
  }
}

Rect _rectOf(BuildContext context) {
  final box = context.findRenderObject()! as RenderBox;
  return box.localToGlobal(Offset.zero) & box.size;
}

class _TreePainter extends CustomPainter {
  _TreePainter(this.tree, this.at);

  final SectorTreeView tree;
  final Offset Function(TreeSectorView) at;

  @override
  void paint(Canvas canvas, Size size) {
    final sectors = tree.sectors;
    final line = Paint()..strokeWidth = 2;
    for (final s in sectors) {
      for (final n in s.next) {
        final t = sectors[n];
        line.color = s.visited && t.visited
            ? SectorColors.visited
            : s.reachable && t.reachable
            ? SectorColors.reachable
            : SectorColors.unreachable;
        canvas.drawLine(at(s), at(t), line);
      }
    }
    for (final s in sectors) {
      final ring = s.visited
          ? SectorColors.visited
          : s.reachable
          ? SectorColors.reachable
          : SectorColors.unreachable;
      canvas.drawCircle(at(s), 7, Paint()..color = ring);
      var fill = SectorColors.kind(s.kind);
      if (!s.visited && !s.reachable) {
        fill = Color.fromARGB(
          255,
          (fill.r * 255 * 0.25).round(),
          (fill.g * 255 * 0.25).round(),
          (fill.b * 255 * 0.25).round(),
        );
      }
      canvas.drawCircle(at(s), 6, Paint()..color = const Color(0xFF000000));
      canvas.drawCircle(at(s), 5, Paint()..color = fill);
    }
  }

  @override
  bool shouldRepaint(_TreePainter old) => old.tree != tree;
}

/// The legend of the sector colors and the state of the jump.
class SectorTreeBar extends StatelessWidget {
  const SectorTreeBar({super.key, required this.tree, required this.edits});

  final SectorTreeView tree;
  final Map<String, int> edits;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final tokens = AppTokens.of(context);
    Widget dot(Color c) => Container(
      width: 12,
      height: 12,
      decoration: BoxDecoration(
        color: c,
        shape: BoxShape.circle,
        border: Border.all(color: FtlColors.line),
      ),
    );
    final block = tree.jumpBlock;
    return Wrap(
      spacing: Space.sm,
      runSpacing: Space.sm,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        for (final (kind, color) in [
          ('civilian', SectorColors.civilian),
          ('hostile', SectorColors.hostile),
          ('nebula', SectorColors.nebula),
        ]) ...[dot(color), AppText(sectorKindLabel(kind, l), color: tokens.muted), const SizedBox(width: Space.sm)],
        if (block != null)
          AppText(jumpBlockText(block, l), color: tokens.warn, wrap: true)
        else
          AppText(l.sectorJumpHint, color: tokens.muted, wrap: true),
      ],
    );
  }
}

/// The popover of a sector: name, number, type, visit, and the jump there
/// or why there is none.
Future<void> showSectorPopover(
  BuildContext context, {
  required Rect anchor,
  required TreeSectorView sector,
  required SectorTreeView tree,
  required SectorTreeView shown,
  required void Function(String path, int value) onChanged,
}) {
  final l = AppLocalizations.of(context)!;
  return showAnchoredPopover<void>(
    context,
    anchor: anchor,
    barrierLabel: l.mapClose,
    width: 420,
    builder: (context) {
      final tokens = AppTokens.of(context);
      final file = tree.sectors[sector.index];
      final block = file.current ? null : (tree.jumpBlock ?? file.jumpBlock);
      final shipThere = sector.current;
      return AppPopover(
        key: ValueKey('sector-popover-${sector.index}'),
        icon: Icons.public,
        title: sector.name,
        trailing: Wrap(
          spacing: Space.xs,
          children: [
            Tag(label: l.sector(sector.level + 1)),
            Tag(label: sectorKindLabel(sector.kind, l)),
            if (shipThere) Tag(label: l.mapShipHere),
            if (sector.visited && !shipThere) Tag(label: l.mapVisited),
          ],
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            if (shipThere)
              AppText(l.sectorShipHere, color: tokens.muted, wrap: true)
            else if (block != null)
              AppText(jumpBlockText(block, l), color: tokens.warn, wrap: true)
            else ...[
              AppText(l.sectorJumpArrival, color: tokens.muted, wrap: true),
              const SizedBox(height: Space.sm),
              AppButton(
                key: ValueKey('sector-jump-${sector.index}'),
                icon: Icons.travel_explore,
                label: l.sectorJumpHere,
                onPressed: () {
                  onChanged(sectorJumpPath, sector.index);
                  Navigator.of(context).pop();
                },
              ),
            ],
          ],
        ),
      );
    },
  );
}
