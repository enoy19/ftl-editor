// The Map page: the sector map drawn like FTL's beacon map from ftl.dat
// (the zone background, beacon icons, the rebel fleet's danger zone, the
// ship marker, STORE and QUEST labels) over the whole page, a toolbar in
// the map frame for the rebel fleet, and a popover per beacon with its
// visit, a move of the ship, its store and what waits there. The core
// answers every position (ftl_core::sector_map) and every rule
// (ftl_core::beacons); the page stages edits in the owner's edits, like
// every other page.

import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../look/ftl_look.dart';
import '../look/ftl_tabs.dart';
import '../look/ftl_theme.dart';
import '../src/rust/api/assets.dart';
import '../src/rust/api/editor.dart';
import '../src/rust/api/map.dart';
import 'change_sheet.dart';
import 'components/components.dart';
import 'crew_sprite.dart';
import 'equipment_page.dart' show ChoicePicker, PickerChoice;
import 'game_image.dart';
import 'resource_bar.dart';
import 'sector_tree.dart';

/// Every editable number field of the sector map: the fleet, the ship's
/// beacon, each beacon's seen, visited and store flags, each store slot
/// and supply.
List<FieldValue> mapFields(SectorMap? map) => [
  ?map?.fleetField,
  ?map?.shipField,
  for (final b in map?.beacons ?? const <BeaconView>[]) ...[
    b.seen,
    ?b.visited,
    b.hasStore,
    for (final i in b.store?.items ?? const <StoreItemView>[]) i.stock,
    ...?b.store?.supplies,
  ],
];

/// Icon and label of a map path; `null` for any other path.
(IconData, String)? mapLook(String path, AppLocalizations l, SectorMap? map) {
  if (path == 'fleet.offset') return (Icons.warning_amber, l.mapFleetField);
  if (path == 'beacons.current') return (Icons.navigation, l.mapShipField);
  final parts = path.split('.');
  if (parts.length < 3 || parts[0] != 'beacons') return null;
  final n = int.tryParse(parts[1]);
  if (n == null) return null;
  final shown = n + 1;
  return switch (parts.sublist(2)) {
    ['seen'] => (Icons.visibility, l.mapBeaconField(shown, l.mapSeen)),
    ['visited'] => (Icons.flag, l.mapBeaconField(shown, l.mapVisited)),
    ['store'] => (Icons.storefront, l.mapBeaconField(shown, l.mapStore)),
    ['store', final supply] => (
      resourceLook('ship.$supply', l).$1,
      l.mapStoreItemField(shown, resourceLook('ship.$supply', l).$2),
    ),
    ['store', _, _] => (Icons.storefront, l.mapStoreItemField(shown, _itemName(map, n, path) ?? l.mapStore)),
    ['store', _, _, 'item'] => (Icons.storefront, l.mapStoreItemField(shown, l.mapSwapItem)),
    _ => null,
  };
}

String? _itemName(SectorMap? map, int beacon, String path) {
  final b = map?.beacons.where((b) => b.id == beacon).firstOrNull;
  final item = b?.store?.items.where((i) => i.stock.path == path).firstOrNull;
  return item == null ? null : item.name ?? item.id;
}

/// The image of a store item: weapons and drones as in the Equipment page,
/// crew by its portrait, systems by their icon; augments have none.
GameImageRef? storeItemImage(StoreItemView i) => _kindImage(i.kind, i.id, lookId: i.lookId);

GameImageRef? _kindImage(String kind, String id, {String? lookId}) => switch (kind) {
  'weapon' => GameImageRef.of(GameImageKind.weaponFrame, id),
  'drone' => GameImageRef.of(GameImageKind.drone, id),
  'crew' => crewSprite(lookId ?? id),
  'system' => GameImageRef.of(GameImageKind.systemIcon, id),
  _ => null,
};

IconData _kindIcon(String kind) => switch (kind) {
  'weapon' => Icons.bolt,
  'drone' => Icons.precision_manufacturing,
  'crew' => Icons.person,
  'system' => Icons.memory,
  _ => Icons.extension,
};

/// One blueprint a store slot of a kind can take: id and name.
typedef StoreChoice = ({String id, String name});

/// The blueprints of a shelf kind (`weapon`, `drone`, `augment`, `crew`,
/// `system`) the page offers for a swap.
typedef StoreChoices = List<StoreChoice> Function(String kind);

/// The change sheet rows of the pending store item swaps: the item of
/// [base] (the file, or a new store as it starts), the chosen one by name.
List<ChangeRow> storeItemChangeRows(
  SectorMap? base,
  Map<String, String> texts,
  StoreChoices choices,
  AppLocalizations l,
) => [
  for (final b in base?.beacons ?? const <BeaconView>[])
    for (final i in b.store?.items ?? const <StoreItemView>[])
      if (texts[i.itemPath] case final id?)
        ChangeRow(
          id: i.itemPath,
          icon: Icons.storefront,
          title: l.mapStoreItemField(b.id + 1, l.mapSwapItem),
          change: (i.name ?? i.id, choices(i.kind).where((c) => c.id == id).firstOrNull?.name ?? id),
        ),
];

/// The map images of ftl.dat the page draws.
abstract final class MapImages {
  static const unvisited = GameImageRef.path('img/map/map_icon_diamond_yellow.png');
  static const visited = GameImageRef.path('img/map/map_icon_diamond_blue.png');
  static const overtaken = GameImageRef.path('img/map/map_icon_warning.png');
  static const ship = GameImageRef.path('img/map/map_icon_ship.png');
  static const zoneTile = GameImageRef.path('img/map/map_warningcircle_tile.png');
  static const zoneEdge = GameImageRef.path('img/map/map_warningcircle_point.png');
  static GameImageRef box(String color, int part) => GameImageRef.path('img/map/map_box_${color}_$part.png');

  /// The background of a sector: `StarMap::OnRender` draws
  /// `map/zone_<sector % 3 + 1>.png` (zero-based sector), tinted 0.75.
  static GameImageRef zone(int sector) => GameImageRef.path('img/map/zone_${(sector - 1) % 3 + 1}.png');

  static List<GameImageRef> get all => [
    unvisited,
    visited,
    overtaken,
    ship,
    zoneTile,
    zoneEdge,
    for (final s in [1, 2, 3]) zone(s),
    for (final c in ['white', 'purple'])
      for (final p in [1, 2, 3]) box(c, p),
  ];
}

/// Where the beacon area starts inside the drawing, and the drawing's size:
/// the beacons stand at 10 to 649 by 10 to 415 (`PopulateGrid`).
const _origin = Offset(Space.xl, Space.xl + Space.md);
const _canvas = Size(700, 470);

/// The tint `StarMap::OnRender` puts on the zone background.
const _zoneTint = Color(0xFFBFBFBF);

/// The edge image (200 x 996) puts the tip of its arc 181 pixels right of its
/// left side and its middle at the fudge; FTL tiles the zone left of it
/// (`StarMap::OnRender`: translate to offset + 767, tiles up to offset + 586).
const _edgeLeft = 586.0;
const _edgeHeight = 996.0;
const _edgeWidth = 200.0;

/// The fleet moves 64 pixels per jump (`StarMap::UpdateDangerZone`).
const fleetJump = 64;

typedef OnChanged = void Function(String path, int value);
typedef OnText = void Function(String path, String text);

/// The Map page. Pure: the owner holds the edits; [map] is the file,
/// [preview] the map as the pending edits leave it (the fleet flags follow a
/// moved fleet, the ship stands at its new beacon, a new store is there),
/// [previewError] why the core refused them. [texts] are the pending store
/// item swaps, [storeChoices] the blueprints a slot can take.
class MapPage extends StatefulWidget {
  const MapPage({
    super.key,
    required this.map,
    required this.edits,
    required this.onChanged,
    this.preview,
    this.previewError,
    this.texts = const {},
    this.onText,
    this.storeChoices,
    this.previewOf,
    this.tree,
    this.treePreview,
    this.treeBlock,
  });

  final SectorMap? map;
  final SectorMap? preview;
  final String? previewError;
  final Map<String, int> edits;
  final OnChanged onChanged;
  final Map<String, String> texts;
  final OnText? onText;
  final StoreChoices? storeChoices;
  final SectorMap? Function()? previewOf;

  /// The sector tree of the file and as the pending edits leave it (a
  /// staged jump); [treeBlock] the core's code when there is no tree.
  final SectorTreeView? tree;
  final SectorTreeView? treePreview;
  final String? treeBlock;

  @override
  State<MapPage> createState() => _MapPageState();
}

class _MapPageState extends State<MapPage> {
  bool _sectors = false;

  @override
  Widget build(BuildContext context) {
    final w = widget;
    final l = AppLocalizations.of(context)!;
    final viewSwitch = MapViewSwitch(sectors: _sectors, onChanged: (v) => setState(() => _sectors = v));
    final tree = w.tree;
    if (_sectors && w.map != null) {
      return Padding(
        padding: const EdgeInsets.all(Space.xl),
        child: SizedBox.expand(
          child: Panel(
            icon: Icons.account_tree,
            title: l.sectorTree,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                viewSwitch,
                const SizedBox(height: Space.sm),
                if (tree == null)
                  AppText(jumpBlockText(w.treeBlock ?? '', l), color: AppTokens.of(context).muted, wrap: true)
                else ...[
                  SectorTreeBar(tree: w.treePreview ?? tree, edits: w.edits),
                  if (w.previewError case final e?) ...[
                    const SizedBox(height: Space.sm),
                    AppText(e, color: AppTokens.of(context).warn, wrap: true),
                  ],
                  const SizedBox(height: Space.md),
                  Expanded(
                    child: SectorTreeCanvas(
                      tree: tree,
                      shown: w.treePreview ?? tree,
                      edits: w.edits,
                      onChanged: w.onChanged,
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
      );
    }
    final jumping = w.edits.containsKey(sectorJumpPath) && w.preview != null;
    return BeaconMap(
      // A staged jump shows the new sector read only: its beacons are not
      // the file's, so no beacon edit can be staged on them.
      map: jumping ? w.preview : w.map,
      preview: jumping ? null : w.preview,
      previewError: w.previewError,
      edits: jumping ? const {} : w.edits,
      onChanged: w.onChanged,
      texts: w.texts,
      onText: w.onText,
      storeChoices: w.storeChoices,
      previewOf: jumping ? () => null : w.previewOf,
      header: viewSwitch,
      locked: jumping,
    );
  }
}

/// Beacons or sectors: the two views of the Map page.
class MapViewSwitch extends StatelessWidget {
  const MapViewSwitch({super.key, required this.sectors, required this.onChanged});

  final bool sectors;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final look = FtlTheme.of(context);
    if (look != null) {
      return Align(
        alignment: Alignment.centerLeft,
        child: FtlChipRow(
          key: const ValueKey('map-view'),
          look: look,
          chips: [
            FtlTab(l.mapBeacons, key: const ValueKey('map-view-beacons')),
            FtlTab(l.sectorTree, key: const ValueKey('map-view-sectors')),
          ],
          open: sectors ? 1 : 0,
          onOpen: (i) => onChanged(i == 1),
        ),
      );
    }
    return Align(
      alignment: Alignment.centerLeft,
      child: SegmentedButton<bool>(
        key: const ValueKey('map-view'),
        showSelectedIcon: false,
        segments: [
          ButtonSegment(
            value: false,
            icon: const Icon(Icons.scatter_plot, key: ValueKey('map-view-beacons')),
            label: AppText(l.mapBeacons),
          ),
          ButtonSegment(
            value: true,
            icon: const Icon(Icons.account_tree, key: ValueKey('map-view-sectors')),
            label: AppText(l.sectorTree),
          ),
        ],
        selected: {sectors},
        onSelectionChanged: (s) => onChanged(s.first),
      ),
    );
  }
}

/// The beacon map of the Map page: the panel with the toolbar and the
/// drawing. [locked] shows a staged jump's new sector without edits.
class BeaconMap extends StatelessWidget {
  const BeaconMap({
    super.key,
    required this.map,
    required this.edits,
    required this.onChanged,
    this.preview,
    this.previewError,
    this.texts = const {},
    this.onText,
    this.storeChoices,
    this.previewOf,
    this.header,
    this.locked = false,
  });

  final Widget? header;
  final bool locked;

  final SectorMap? map;
  final SectorMap? preview;
  final String? previewError;
  final Map<String, int> edits;
  final OnChanged onChanged;
  final Map<String, String> texts;
  final OnText? onText;
  final StoreChoices? storeChoices;

  /// The preview as the edits are now: an open popover reads it after each
  /// of its edits, because it keeps the page's [preview] of its opening.
  final SectorMap? Function()? previewOf;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final map = this.map;
    if (map == null) {
      return Center(child: AppText(l.mapNone, color: AppTokens.of(context).muted));
    }
    final shown = preview ?? map;
    final actions = MapActions(
      map: map,
      edits: edits,
      onChanged: onChanged,
      texts: texts,
      onText: onText,
      storeChoices: storeChoices,
      previewOf: previewOf ?? () => preview,
      locked: locked,
    );
    final body = map.layoutBlock == null
        ? SectorCanvas(map: map, shown: shown, actions: actions)
        : _BeaconList(map: map, shown: shown, actions: actions);
    return Padding(
      padding: const EdgeInsets.all(Space.xl),
      child: SizedBox.expand(
        child: Panel(
          icon: Icons.map,
          title: l.sector(map.sector),
          note: switch (map.layoutBlock) {
            null when locked => l.sectorJumpPendingNote,
            null => l.mapSectorsNote,
            'map_native_random' => l.mapLayoutNative,
            _ => l.mapLayoutMismatch,
          },
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              ?header,
              if (header != null) const SizedBox(height: Space.sm),
              if (!locked) MapToolbar(map: map, shown: shown, edits: edits, onChanged: onChanged),
              if (previewError case final e?) ...[
                const SizedBox(height: Space.sm),
                AppText(e, color: AppTokens.of(context).warn, wrap: true),
              ],
              const SizedBox(height: Space.md),
              Expanded(child: body),
            ],
          ),
        ),
      ),
    );
  }
}

/// What a beacon's popover needs to stage edits.
class MapActions {
  const MapActions({
    required this.map,
    required this.edits,
    required this.onChanged,
    required this.texts,
    required this.onText,
    required this.storeChoices,
    required this.previewOf,
    this.locked = false,
  });

  /// A staged jump: the beacons open no popover.
  final bool locked;
  final SectorMap map;
  final Map<String, int> edits;
  final OnChanged onChanged;
  final Map<String, String> texts;
  final OnText? onText;
  final StoreChoices? storeChoices;
  final SectorMap? Function() previewOf;
}

/// The value of [f] with the pending edit.
int _now(FieldValue f, Map<String, int> edits) => edits[f.path] ?? f.value;

/// The direction of the first pending edit among a beacon's fields;
/// `null` when none.
EditDirection? _beaconMark(BeaconView b, Map<String, int> edits, {bool moved = false}) {
  if (moved) return EditDirection.raised;
  for (final f in [
    b.seen,
    ?b.visited,
    b.hasStore,
    for (final i in b.store?.items ?? const <StoreItemView>[]) i.stock,
    ...?b.store?.supplies,
  ]) {
    if (directionOf(f.value, _now(f, edits)) case final d?) return d;
  }
  return null;
}

/// The beacon map: the zone background over the whole space, the danger
/// zone, the joins of the ship's beacon, every beacon by its state, the
/// ship and the labels. The 700 x 470 drawing scales to fit the space and
/// stays centered; the background and the zone fill the rest.
class SectorCanvas extends StatelessWidget {
  const SectorCanvas({super.key, required this.map, required this.shown, required this.actions});

  final SectorMap map;
  final SectorMap shown;
  final MapActions actions;

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final scale = (box.maxWidth / _canvas.width).clamp(0.2, box.maxHeight / _canvas.height);
      final size = Size(box.maxWidth / scale, box.maxHeight / scale);
      final origin = _origin + Offset((size.width - _canvas.width) / 2, (size.height - _canvas.height) / 2);
      // The drawing scales by a fraction; the labels and change marks stand
      // above it at screen scale, so their bitmap text stays sharp (a scaled
      // STORE label blurred its letters).
      return ClipRect(
        child: Stack(
          children: [
            Positioned.fill(
              child: FittedBox(
                fit: BoxFit.fill,
                child: SizedBox.fromSize(size: size, child: _drawing(context, origin)),
              ),
            ),
            ..._overlay(context, origin, scale),
          ],
        ),
      );
    },
  );

  /// The STORE and QUEST labels and the change marks of the beacons, placed
  /// at the screen position of their beacon ([scale] of the drawing).
  List<Widget> _overlay(BuildContext context, Offset origin, double scale) {
    final l = AppLocalizations.of(context)!;
    final current = shown.beacons.where((b) => b.current).firstOrNull;
    final moved = current != null && !map.beacons[current.id].current;
    Offset at(BeaconView b) => (origin + Offset(b.x!.toDouble(), b.y!.toDouble())) * scale;
    final placed = [
      for (final b in shown.beacons)
        if (b.x != null && b.y != null) b,
    ];
    return [
      for (final b in placed) ..._labels(context, b, at(b), l, scale),
      for (final b in placed)
        if (_beaconMark(map.beacons[b.id], actions.edits, moved: moved && b.id == current.id) case final d?)
          // Left of the beacon: the labels take its right side.
          Positioned(left: at(b).dx - 22 * scale, top: at(b).dy - 18 * scale, child: ChangeMark(d)),
    ];
  }

  Widget _drawing(BuildContext context, Offset origin) {
    final current = shown.beacons.where((b) => b.current).firstOrNull;
    Offset at(BeaconView b) => origin + Offset(b.x!.toDouble(), b.y!.toDouble());
    final placed = [
      for (final b in shown.beacons)
        if (b.x != null && b.y != null) b,
    ];
    final joined = <(Offset, Offset)>[
      if (current != null && current.x != null)
        for (var i = 0; i + 1 < shown.joins.length; i += 2)
          if (shown.joins[i] == current.id || shown.joins[i + 1] == current.id)
            (at(shown.beacons[shown.joins[i]]), at(shown.beacons[shown.joins[i + 1]])),
    ];
    return Stack(
      clipBehavior: Clip.hardEdge,
      children: [
        const Positioned.fill(child: ColoredBox(color: FtlColors.ground)),
        Positioned.fill(
          child: ColorFiltered(
            colorFilter: const ColorFilter.mode(_zoneTint, BlendMode.modulate),
            child: GameImage(
              MapImages.zone(map.sector),
              fallback: Icons.blur_on,
              width: double.infinity,
              height: double.infinity,
              fit: BoxFit.cover,
              iconSize: 0,
            ),
          ),
        ),
        ..._dangerZone(context, origin),
        Positioned.fill(child: CustomPaint(painter: _JoinPainter(joined, FtlColors.line.withValues(alpha: 0.7)))),
        for (final b in placed) _beacon(context, b, at(b)),
        if (current != null && current.x != null)
          Positioned(
            left: at(current).dx - 32,
            top: at(current).dy - 32,
            child: const IgnorePointer(
              child: GameImage(MapImages.ship, fallback: Icons.navigation, width: 64, height: 64, iconSize: 20),
            ),
          ),
      ],
    );
  }

  /// FTL's zone: the tile left of the edge image, the edge image with its
  /// arc at offset + 767 and its middle at the fudge. Reduced mode tints
  /// the circle red instead.
  List<Widget> _dangerZone(BuildContext context, Offset origin) {
    final offset = shown.fleetOffset.toDouble();
    final fudge = shown.fleetFudge.toDouble();
    final left = origin.dx + offset + _edgeLeft;
    final top = origin.dy + fudge - _edgeHeight / 2;
    final scope = GameDataScope.maybeOf(context);
    final dat = scope?.datPath;
    if (scope == null || dat == null) {
      final r = shown.fleetRadius.toDouble();
      return [
        Positioned(
          left: origin.dx + offset - r,
          top: origin.dy + fudge - r,
          width: 2 * r,
          height: 2 * r,
          child: DecoratedBox(
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: AppTokens.of(context).warn.withValues(alpha: 0.18),
              border: Border.all(color: AppTokens.of(context).warn),
            ),
          ),
        ),
      ];
    }
    final r = shown.fleetRadius.toDouble();
    return [
      // The stripes fill the fleet's circle (center at offset and fudge, the
      // arc of the edge image at its right), not a rectangle left of the
      // edge image: between that rectangle and the arc no stripes showed.
      if (left + _edgeWidth > 0)
        Positioned.fill(
          child: ClipPath(
            clipper: _CircleClip(Offset(origin.dx + offset, origin.dy + fudge), r),
            child: DecoratedBox(
              decoration: BoxDecoration(
                image: DecorationImage(
                  image: GameImageProvider(datPath: dat, ref: MapImages.zoneTile, load: scope.load),
                  repeat: ImageRepeat.repeat,
                  alignment: Alignment.topRight,
                  filterQuality: FilterQuality.none,
                ),
              ),
            ),
          ),
        ),
      Positioned(
        left: left,
        top: top,
        child: const GameImage(
          MapImages.zoneEdge,
          fallback: Icons.circle_outlined,
          width: _edgeWidth,
          height: _edgeHeight,
          fit: BoxFit.none,
        ),
      ),
    ];
  }

  Widget _beacon(BuildContext context, BeaconView b, Offset p) {
    final file = map.beacons[b.id];
    final edits = actions.edits;
    final visited = b.current || file.visited == null || _now(file.visited!, edits) > 0;
    final seen = _now(file.seen, edits) > 0;
    final (ref, icon) = b.fleet == 'rebel' || b.fleet == 'both'
        ? (MapImages.overtaken, Icons.report)
        : visited
        ? (MapImages.visited, Icons.circle)
        : (MapImages.unvisited, Icons.circle_outlined);
    return Positioned(
      left: p.dx - 16,
      top: p.dy - 16,
      child: Builder(
        builder: (context) => MouseRegion(
          cursor: SystemMouseCursors.click,
          child: GestureDetector(
            key: ValueKey('map-beacon-${b.id}'),
            behavior: HitTestBehavior.opaque,
            onTap: actions.locked
                ? null
                : () => showBeaconPopover(context, anchor: _rectOf(context), beacon: file, shown: b, actions: actions),
            child: Opacity(
              opacity: seen ? 1 : 0.45,
              child: GameImage(ref, fallback: icon, width: 32, height: 32, iconSize: 14),
            ),
          ),
        ),
      ),
    );
  }

  /// STORE and QUEST beside a beacon, in FTL's map boxes: the corner of the
  /// box points at the beacon.
  /// [p] is the beacon on screen; the labels keep their own size and step
  /// off the beacon by its scaled radius.
  List<Widget> _labels(BuildContext context, BeaconView b, Offset p, AppLocalizations l, double scale) => [
    if (b.store != null)
      Positioned(
        left: p.dx + 2 * scale,
        top: p.dy - 30,
        child: MapLabel(color: 'white', text: l.mapStore),
      ),
    if (b.quest != null)
      Positioned(
        left: p.dx + 2 * scale,
        top: p.dy - (b.store != null ? 56 : 30),
        child: MapLabel(color: 'purple', text: l.mapQuest),
      ),
  ];
}

Rect _rectOf(BuildContext context) {
  final box = context.findRenderObject()! as RenderBox;
  return box.localToGlobal(Offset.zero) & box.size;
}

/// The fleet's circle: [center] and [radius] in the coordinates of the map.
class _CircleClip extends CustomClipper<Path> {
  _CircleClip(this.center, this.radius);

  final Offset center;
  final double radius;

  @override
  Path getClip(Size size) => Path()..addOval(Rect.fromCircle(center: center, radius: radius));

  @override
  bool shouldReclip(_CircleClip old) => old.center != center || old.radius != radius;
}

class _JoinPainter extends CustomPainter {
  _JoinPainter(this.joins, this.color);

  final List<(Offset, Offset)> joins;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..strokeWidth = 2;
    for (final (a, b) in joins) {
      // FTL's dotted line (img/map/dotted_line.png): 6 on, 6 off.
      final d = b - a;
      final length = d.distance;
      final step = d / length;
      for (var s = 16.0; s < length - 16; s += 12) {
        canvas.drawLine(a + step * s, a + step * (s + 6).clamp(0, length - 16), paint);
      }
    }
  }

  @override
  bool shouldRepaint(_JoinPainter old) => old.joins != joins || old.color != color;
}

/// A label of FTL's map: `map_box_<color>_1/2/3.png` around the text.
class MapLabel extends StatelessWidget {
  const MapLabel({super.key, required this.color, required this.text});

  final String color;
  final String text;

  @override
  Widget build(BuildContext context) {
    final label = AppText(text, role: TextRole.caption, caps: true, color: FtlColors.line);
    if (FtlTheme.of(context) == null || GameDataScope.maybeOf(context)?.datPath == null) {
      return IgnorePointer(child: Tag(label: text));
    }
    // The box is 32 high; its lowest 6 pixels are the corner that points
    // at the beacon, so the text sits in the 26 above.
    return IgnorePointer(
      child: SizedBox(
        height: 32,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            GameImage(MapImages.box(color, 1), fallback: Icons.label, width: 19, height: 32),
            IntrinsicWidth(
              child: Stack(
                children: [
                  Positioned.fill(
                    child: GameImage(
                      MapImages.box(color, 2),
                      fallback: Icons.label,
                      width: 1,
                      height: 32,
                      fit: BoxFit.fill,
                    ),
                  ),
                  // The dark face of map_box_<color>_2.png is rows 8 to 17
                  // (light edges at 6-7 and 18-19, measured in ftl.dat); the
                  // capitals center on that face. Centered in all 26 rows
                  // the text sat on the bottom edge and its R read as B.
                  // The stack is the full 32 rows, so the middle image keeps
                  // its own height instead of the text's.
                  const SizedBox(height: 32),
                  Padding(
                    padding: const EdgeInsets.only(top: Space.sm),
                    child: SizedBox(
                      height: 10,
                      child: OverflowBox(
                        maxHeight: double.infinity,
                        child: Center(child: label),
                      ),
                    ),
                  ),
                ],
              ),
            ),
            GameImage(MapImages.box(color, 3), fallback: Icons.label, width: 15, height: 32),
          ],
        ),
      ),
    );
  }
}

/// The beacons as a list, when the positions are unknown: each row opens
/// the same popover as a beacon of the map.
class _BeaconList extends StatelessWidget {
  const _BeaconList({required this.map, required this.shown, required this.actions});

  final SectorMap map;
  final SectorMap shown;
  final MapActions actions;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    return ListView(
      children: [
        for (final b in map.beacons)
          Builder(
            builder: (context) {
              final s = shown.beacons[b.id];
              return ListRow(
                key: ValueKey('map-beacon-${b.id}'),
                dense: true,
                leading: switch (_beaconMark(b, actions.edits, moved: s.current && !b.current)) {
                  final d? => ChangeMark(d),
                  null => const Icon(Icons.circle_outlined),
                },
                title: l.mapBeacon(b.id + 1),
                subtitle: [
                  if (s.current) l.mapShipHere,
                  if (s.store != null) l.mapStore,
                  if (s.quest != null) l.mapQuest,
                ].join(' · '),
                onTap: () =>
                    showBeaconPopover(context, anchor: _rectOf(context), beacon: b, shown: s, actions: actions),
              );
            },
          ),
      ],
    );
  }
}

/// The toolbar in the map frame: how many beacons the rebel fleet holds,
/// and the fleet moved back to the sector start, one jump back or one jump
/// forward ([fleetJump]) as icon buttons with their names as tooltips, so
/// the bar stays one line at 1100 x 700; why the fleet stays when it does.
class MapToolbar extends StatelessWidget {
  const MapToolbar({super.key, required this.map, required this.shown, required this.edits, required this.onChanged});

  final SectorMap map;
  final SectorMap shown;
  final Map<String, int> edits;
  final OnChanged onChanged;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final tokens = AppTokens.of(context);
    final field = map.fleetField;
    final overtaken = shown.beacons.where((b) => b.fleet == 'rebel' || b.fleet == 'both').length;
    final mark = field == null ? null : directionOf(field.value, _now(field, edits));
    final now = field == null ? 0 : _now(field, edits);
    void move(int to) => onChanged(field!.path, to.clamp(field.min, field.max));
    return Wrap(
      spacing: Space.sm,
      runSpacing: Space.sm,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        Icon(Icons.warning_amber, size: 18, color: tokens.warn),
        AppText(l.mapFleet, role: TextRole.label, caps: true),
        // The buttons stand before the count and the mark: a count that grows
        // or a mark that appears must not move the button under the pointer.
        if (field == null)
          AppText(
            map.fleetBlock == 'last_stand' ? l.mapFleetLastStand : l.mapFleetUnknown,
            color: tokens.muted,
            wrap: true,
          )
        else ...[
          AppButton(
            key: const ValueKey('fleet-start'),
            icon: Icons.first_page,
            tooltip: l.mapFleetStart,
            onPressed: now <= field.min ? null : () => move(field.min),
          ),
          AppButton(
            key: const ValueKey('fleet-back'),
            icon: Icons.keyboard_double_arrow_left,
            tooltip: l.mapFleetBack,
            onPressed: now <= field.min ? null : () => move(now - fleetJump),
          ),
          AppButton(
            key: const ValueKey('fleet-forward'),
            icon: Icons.keyboard_double_arrow_right,
            tooltip: l.mapFleetForward,
            onPressed: now >= field.max ? null : () => move(now + fleetJump),
          ),
        ],
        const SizedBox(width: Space.md),
        AppText(
          key: const ValueKey('fleet-overtaken'),
          l.mapFleetOvertaken(overtaken, shown.beacons.length),
          color: tokens.muted,
        ),
        ?(mark == null ? null : ChangeMark(mark)),
      ],
    );
  }
}

/// The text of a move block code of the core.
String moveBlockText(String code, AppLocalizations l) => switch (code) {
  'last_stand' => l.mapMoveBlockLastStand,
  'move_crew_aboard' => l.mapMoveBlockCrew,
  'move_fleet_beacon' => l.mapMoveBlockFleet,
  'move_quest_beacon' => l.mapMoveBlockQuest,
  _ => l.mapMoveBlockUnknown,
};

/// The popover of a beacon: its visit and reveal, a move of the ship, the
/// fleet, the ship event and quest waiting there, its store (added or
/// removed here) with every item (image, name, for sale or sold out, a
/// swap) and the supplies, and why other kinds cannot be set. [beacon] is
/// the file's beacon, [shown] the preview's. It redraws after each edit.
Future<void> showBeaconPopover(
  BuildContext context, {
  required Rect anchor,
  required BeaconView beacon,
  required BeaconView shown,
  required MapActions actions,
}) {
  final l = AppLocalizations.of(context)!;
  return showAnchoredPopover<void>(
    context,
    anchor: anchor,
    barrierLabel: l.mapClose,
    width: 500,
    builder: (context) => StatefulBuilder(
      builder: (context, setState) => _BeaconPopover(
        key: ValueKey('beacon-popover-${beacon.id}'),
        beacon: beacon,
        shown: shown,
        actions: MapActions(
          map: actions.map,
          edits: actions.edits,
          onChanged: (path, value) {
            actions.onChanged(path, value);
            setState(() {});
          },
          texts: actions.texts,
          onText: actions.onText == null
              ? null
              : (path, text) {
                  actions.onText!(path, text);
                  setState(() {});
                },
          storeChoices: actions.storeChoices,
          previewOf: actions.previewOf,
        ),
      ),
    ),
  );
}

class _BeaconPopover extends StatelessWidget {
  const _BeaconPopover({super.key, required this.beacon, required this.shown, required this.actions});

  final BeaconView beacon;
  final BeaconView shown;
  final MapActions actions;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final tokens = AppTokens.of(context);
    final edits = actions.edits;
    final onChanged = actions.onChanged;
    final b = beacon;
    final shown = actions.previewOf()?.beacons.where((s) => s.id == b.id).firstOrNull ?? this.shown;
    final visited = b.visited;
    final isVisited = visited == null || _now(visited, edits) > 0;
    final seen = _now(b.seen, edits) > 0;
    Widget? mark(FieldValue f) => switch (directionOf(f.value, _now(f, edits))) {
      final d? => ChangeMark(d),
      null => null,
    };
    final ship = actions.map.shipField;
    final shipHere = (ship == null ? b.current : _now(ship, edits) == b.id);
    final moveBlock = actions.map.shipBlock ?? b.moveBlock;
    final hasStore = _now(b.hasStore, edits) > 0;
    // A new store exists only in the preview; a store of the file keeps
    // its own fields, so their marks compare with the file.
    final store = hasStore ? (b.store ?? shown.store) : null;
    return AppPopover(
      icon: Icons.place,
      title: l.mapBeacon(b.id + 1),
      trailing: Wrap(
        spacing: Space.xs,
        children: [
          if (shipHere) Tag(label: l.mapShipHere),
          if (store != null) Tag(label: l.mapStore),
        ],
      ),
      actions: [
        AppButton(key: const ValueKey('beacon-close'), label: l.mapClose, onPressed: () => Navigator.of(context).pop()),
      ],
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: MediaQuery.sizeOf(context).height * 0.7),
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              if (!b.current || ship != null)
                ListRow(
                  dense: true,
                  leading: shipHere && !b.current
                      ? const ChangeMark(EditDirection.raised)
                      : const Icon(Icons.navigation, size: 18),
                  title: shipHere ? l.mapShipHere : l.mapShipField,
                  subtitle: shipHere ? null : (moveBlock == null ? l.mapMoveHint : moveBlockText(moveBlock, l)),
                  trailing: shipHere || ship == null || moveBlock != null
                      ? null
                      : AppButton(
                          key: ValueKey('beacon-move-${b.id}'),
                          icon: Icons.navigation,
                          label: l.mapMoveShip,
                          onPressed: () => onChanged(ship.path, b.id),
                        ),
                ),
              ListRow(
                dense: true,
                leading: (visited == null ? null : mark(visited)) ?? const Icon(Icons.flag, size: 18),
                title: isVisited ? l.mapVisited : l.mapNotVisited,
                subtitle: visited == null ? l.mapVisitKept : (visited.value > 0 ? l.mapUnvisitHint : null),
                trailing: visited == null || visited.max == 0
                    ? null
                    : AppButton(
                        key: ValueKey('beacon-unvisit-${b.id}'),
                        label: isVisited ? l.mapUnvisit : l.mapVisited,
                        onPressed: () => onChanged(visited.path, isVisited ? 0 : 1),
                      ),
              ),
              ListRow(
                dense: true,
                leading: mark(b.seen) ?? const Icon(Icons.visibility, size: 18),
                title: seen ? l.mapSeen : l.mapHidden,
                trailing: AppButton(
                  key: ValueKey('beacon-seen-${b.id}'),
                  label: seen ? l.mapHide : l.mapReveal,
                  onPressed: () => onChanged(b.seen.path, seen ? 0 : 1),
                ),
              ),
              if (shown.fleet == 'rebel' || shown.fleet == 'both')
                ListRow(
                  dense: true,
                  leading: Icon(Icons.report, size: 18, color: tokens.warn),
                  title: l.mapOvertaken,
                ),
              if (b.enemy case final e?)
                ListRow(dense: true, leading: const Icon(Icons.rocket, size: 18), title: l.mapEnemy, subtitle: e),
              if (b.quest case final q?)
                ListRow(dense: true, leading: const Icon(Icons.assignment, size: 18), title: l.mapQuest, subtitle: q),
              const SizedBox(height: Space.sm),
              TitleRow(
                icon: Icons.storefront,
                title: l.mapStore,
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    ?mark(b.hasStore),
                    const SizedBox(width: Space.xs),
                    AppButton(
                      key: ValueKey('beacon-store-${b.id}'),
                      icon: hasStore ? Icons.remove_circle_outline : Icons.add_business,
                      label: hasStore ? l.mapStoreRemove : l.mapStoreAdd,
                      onPressed: () => onChanged(b.hasStore.path, hasStore ? 0 : 1),
                    ),
                  ],
                ),
              ),
              if (store == null)
                AppText(l.mapStoreAddHint, color: tokens.muted, wrap: true)
              else ...[
                for (final i in store.items) _StoreItemRow(item: i, actions: actions),
                for (final s in store.supplies) _SupplyRow(field: s, edits: edits, onChanged: onChanged),
              ],
              const SizedBox(height: Space.sm),
              AppText(l.mapKindsNote, color: tokens.muted, wrap: true),
            ],
          ),
        ),
      ),
    );
  }
}

class _StoreItemRow extends StatelessWidget {
  const _StoreItemRow({required this.item, required this.actions});

  final StoreItemView item;
  final MapActions actions;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final edits = actions.edits;
    final f = item.stock;
    final forSale = _now(f, edits) > 0;
    final swapped = actions.texts[item.itemPath];
    final choices = actions.storeChoices?.call(item.kind) ?? const <StoreChoice>[];
    final id = swapped ?? item.id;
    final name = swapped == null
        ? (item.name ?? item.id)
        : (choices.where((c) => c.id == swapped).firstOrNull?.name ?? swapped);
    final image = swapped == null ? storeItemImage(item) : _kindImage(item.kind, swapped);
    final d = swapped != null && swapped != item.id ? EditDirection.raised : directionOf(f.value, _now(f, edits));
    final onText = actions.onText;
    return ListRow(
      key: ValueKey('store-item-${f.path}'),
      caps: true,
      leading: SizedBox(
        width: 48,
        height: 40,
        child: Stack(
          children: [
            Center(
              child: image == null
                  ? Icon(_kindIcon(item.kind))
                  : Opacity(
                      opacity: forSale ? 1 : 0.4,
                      child: GameImage(image, fallback: _kindIcon(item.kind), width: 48, height: 40, iconSize: 22),
                    ),
            ),
            if (d != null) Positioned(right: 0, top: 0, child: ChangeMark(d)),
          ],
        ),
      ),
      title: name,
      subtitle: forSale ? l.mapForSale : l.mapSoldOut,
      // Icon buttons keep the item name readable in a narrow popover.
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          AppButton(
            key: ValueKey('store-swap-${item.itemPath}'),
            icon: Icons.swap_horiz,
            tooltip: choices.isEmpty ? l.mapNoChoices : l.mapSwapItem,
            onPressed: onText == null || choices.isEmpty
                ? null
                : () async {
                    final picked = await _pickStoreItem(context, item.kind, choices, id);
                    if (picked != null && picked.isNotEmpty && picked != id) onText(item.itemPath, picked);
                  },
          ),
          const SizedBox(width: Space.xs),
          AppButton(
            key: ValueKey('store-stock-${f.path}'),
            icon: forSale ? Icons.remove_shopping_cart : Icons.add_shopping_cart,
            tooltip: forSale ? l.mapSellOut : l.mapRestock,
            onPressed: () => actions.onChanged(f.path, forSale ? 0 : 1),
          ),
        ],
      ),
    );
  }
}

/// The chooser of a store slot: every blueprint of the shelf's kind.
Future<String?> _pickStoreItem(BuildContext context, String kind, List<StoreChoice> choices, String current) {
  final l = AppLocalizations.of(context)!;
  return showDialog<String>(
    context: context,
    builder: (_) => ChoicePicker(
      keyPrefix: 'store',
      icon: _kindIcon(kind),
      title: l.mapSwapItem,
      choices: [
        for (final c in choices)
          PickerChoice(c.id, c.name, c.id, switch (_kindImage(kind, c.id)) {
            final ref? => GameIcon(ref, fallback: _kindIcon(kind), size: 36),
            null => Icon(_kindIcon(kind), size: 24),
          }),
      ],
    ),
  );
}

class _SupplyRow extends StatelessWidget {
  const _SupplyRow({required this.field, required this.edits, required this.onChanged});

  final FieldValue field;
  final Map<String, int> edits;
  final OnChanged onChanged;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final supply = field.path.split('.').last;
    final (icon, label) = resourceLook('ship.$supply', l);
    final now = _now(field, edits);
    final d = directionOf(field.value, now);
    return ListRow(
      key: ValueKey('store-supply-${field.path}'),
      dense: true,
      leading: d == null
          ? ResourceIcon(path: 'ship.$supply', fallback: icon, color: AppTokens.of(context).text)
          : ChangeMark(d),
      title: label,
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          AppButton(
            key: ValueKey('store-supply-minus-${field.path}'),
            icon: Icons.remove,
            onPressed: now <= field.min ? null : () => onChanged(field.path, now - 1),
          ),
          SizedBox(
            width: 40,
            child: Center(child: AppText('$now', role: TextRole.number)),
          ),
          AppButton(
            key: ValueKey('store-supply-plus-${field.path}'),
            icon: Icons.add,
            onPressed: now >= field.max ? null : () => onChanged(field.path, now + 1),
          ),
        ],
      ),
    );
  }
}
