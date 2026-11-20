// One mock screen in the original FTL look, for the gallery: the top bar
// (area tabs left, Expert and Save right, decision
// ftl-editor-uses-top-tabs-and-portrait-crew-cards) and a Ship panel with
// the resource bar inside a window frame. Example values only; the real
// pages wear the look through FtlTheme (ftl_theme.dart).

import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../src/rust/api/assets.dart';
import 'ftl_look.dart';
import 'ftl_tabs.dart';

/// The gallery entry: loads the look from the local ftl.dat, then shows
/// [LookMockScreen]; without ftl.dat it says why it shows nothing.
class LookMockLoader extends StatefulWidget {
  const LookMockLoader({super.key, this.load = loadLookFromLocatedDat});

  final Future<FtlLook?> Function() load;

  @override
  State<LookMockLoader> createState() => _LookMockLoaderState();
}

/// Locates ftl.dat (read-only, nothing is remembered), opens it and loads
/// the look; `null` when no ftl.dat exists.
Future<FtlLook?> loadLookFromLocatedDat() async {
  final status = await ftlDatStatus();
  final path = status.path;
  if (path == null) return null;
  await openGameData(path: path);
  return FtlLook.load();
}

class _LookMockLoaderState extends State<LookMockLoader> {
  late final Future<FtlLook?> _look = widget.load();

  @override
  Widget build(BuildContext context) => FutureBuilder(
    future: _look,
    builder: (context, snap) {
      final l = AppLocalizations.of(context)!;
      final look = snap.data;
      if (look != null) return LookMockScreen(look: look);
      final text = switch (snap) {
        AsyncSnapshot(hasError: true) => l.lookUnreadable('${snap.error}'),
        AsyncSnapshot(connectionState: ConnectionState.done) => l.lookNoFtlDat,
        _ => null,
      };
      return Center(child: text == null ? const CircularProgressIndicator() : Text(text));
    },
  );
}

/// Example values of a Kestrel run.
const _scrap = 47, _fuel = 16, _missiles = 8, _drones = 2;

/// The mock: Ship is open and the pointer rests on Equipment, so the shot
/// shows the open, hovered and idle tabs.
class LookMockScreen extends StatefulWidget {
  const LookMockScreen({super.key, required this.look, this.open = 0, this.hovered = 2});

  final FtlLook look;
  final int open;
  final int? hovered;

  @override
  State<LookMockScreen> createState() => _LookMockScreenState();
}

class _LookMockScreenState extends State<LookMockScreen> {
  late int _open = widget.open;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final look = widget.look;
    final labelFont = look.font(FtlFontId.justin12Bold);
    final areas = [l.areaShip, l.areaCrew, l.areaEquipment, l.areaMap, l.areaProfile];
    const bar = tabStripHeight * 1.0;
    return ColoredBox(
      color: FtlColors.ground,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(16, 12, 16, 16),
        child: Stack(
          clipBehavior: Clip.none,
          children: [
            Positioned.fill(
              top: bar,
              child: _ShipPanel(look: look),
            ),
            // After the window, so the open tab covers the glow of its line.
            Positioned(
              left: 0,
              top: 0,
              child: FtlTabStrip(
                look: look,
                tabs: [for (final a in areas) FtlTab(a)],
                open: _open,
                hovered: widget.hovered,
                onOpen: (i) => setState(() => _open = i),
              ),
            ),
            Positioned(
              right: 4,
              // Like a closed tab: the face ends 3 px above the frame line.
              top: bar + frameGlow - 3 - 28,
              child: Row(
                children: [
                  FtlButton(label: l.areaExpert.toUpperCase(), font: labelFont, onPressed: () {}),
                  const SizedBox(width: 8),
                  // No edits yet: Save is disabled.
                  FtlButton(label: l.save.toUpperCase(), font: labelFont),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _ShipPanel extends StatelessWidget {
  const _ShipPanel({required this.look});

  final FtlLook look;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final kestrel = look.image(FtlImages.kestrel).image;
    return FtlFrame(
      image: look.image(FtlImages.window),
      fill: FtlColors.window,
      inset: 9,
      cut: 12,
      padding: const EdgeInsets.fromLTRB(28, 24, 28, 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          FtlText('THE KESTREL', font: look.font(FtlFontId.justin12Bold), scale: 2),
          const SizedBox(height: 4),
          FtlText('Kestrel Cruiser, Type A', font: look.font(FtlFontId.justin10)),
          const SizedBox(height: 16),
          Row(
            children: [
              _Resource(
                look: look,
                image: FtlImages.topScrap,
                value: _scrap,
                font: FtlFontId.numbers,
                left: 52,
                top: 14,
              ),
              const SizedBox(width: 8),
              _Resource(look: look, image: FtlImages.topFuel, value: _fuel, left: 38, top: 13),
              _Resource(look: look, image: FtlImages.topMissiles, value: _missiles, left: 38, top: 13),
              _Resource(look: look, image: FtlImages.topDrones, value: _drones, left: 34, top: 13),
            ],
          ),
          const SizedBox(height: 8),
          FtlText(
            '${l.fieldScrap}, ${l.fieldFuel}, ${l.fieldMissiles}, ${l.fieldDroneParts}'.toUpperCase(),
            font: look.font(FtlFontId.hl1),
            color: FtlColors.off,
          ),
          const SizedBox(height: 16),
          Expanded(
            child: FtlFrame(
              image: look.image(FtlImages.window),
              fill: FtlColors.box,
              inset: 9,
              cut: 12,
              child: Center(
                child: RawImage(image: kestrel, filterQuality: FilterQuality.none),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// One box of the resource bar: the status image (icon and box) with the
/// value drawn into the box at [left], [top].
class _Resource extends StatelessWidget {
  const _Resource({
    required this.look,
    required this.image,
    required this.value,
    required this.left,
    required this.top,
    this.font = FtlFontId.justin12Bold,
  });

  final FtlLook look;
  final String image;
  final int value;
  final double left;
  final double top;
  final FtlFontId font;

  @override
  Widget build(BuildContext context) {
    final img = look.image(image).image;
    return SizedBox(
      width: img.width.toDouble(),
      height: img.height.toDouble(),
      child: Stack(
        children: [
          Positioned.fill(
            child: RawImage(image: img, filterQuality: FilterQuality.none),
          ),
          Positioned(
            left: left,
            top: top,
            child: FtlText('$value', font: look.font(font)),
          ),
        ],
      ),
    );
  }
}
