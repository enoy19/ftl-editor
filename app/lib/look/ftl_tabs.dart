// The area tabs of the top bar in the original FTL look: flat FTL buttons
// (variant C of the tab gallery, picked on 2026-11-18). A closed tab
// is a dark well with a light outline and a light label; the hovered one turns
// outline and label yellow; the open tab is a solid light face whose bottom
// runs into the top line of the window below. The strip is [tabStripHeight]
// high and its last [frameGlow] rows overlap the glow of the window frame
// (img/window_outline.png: 7 px glow, then the line). docs/original-look.md
// says how FTL itself builds the tab row of its ship menu.

import 'package:material_ui/material_ui.dart';

import 'ftl_look.dart';

/// Rows of glow above the line of img/window_outline.png.
const frameGlow = 7;

/// Height of one tab, from its top glow to the frame line.
const _tabHeight = 30;

/// Height of the strip above the window: the tab minus the overlap with the
/// window's glow.
const tabStripHeight = _tabHeight - frameGlow;

/// Where the top line of the window runs straight: the window is cut by 12
/// at its corners, plus 7 px of glow and one pixel of air.
const _firstTabX = 20;

/// Pixels between two tabs.
const _gap = 4;

/// Pixels of air between a label and the edge of its tab.
const _labelPad = 10;

/// The top of every face; above it is the glow.
const _faceTop = 4;

/// The bottom of a closed face: 3 px above the frame line.
const _closedBottom = _tabHeight - 3;

/// How a tab looks: open (the page shown), idle, hovered, disabled.
enum FtlTabState { open, idle, hover, off }

/// One tab: its label, whether it can be chosen, and the key of its tap
/// target.
class FtlTab {
  const FtlTab(this.label, {this.enabled = true, this.key});

  final String label;
  final bool enabled;
  final Key? key;
}

/// A row of area tabs in the FTL look. [open] is the page shown (`-1`: none,
/// such as an Expert page); [hovered] starts the hover on one tab (the gallery
/// shot shows it without a pointer).
class FtlTabStrip extends StatefulWidget {
  const FtlTabStrip({super.key, required this.look, required this.tabs, required this.open, this.hovered, this.onOpen});

  final FtlLook look;
  final List<FtlTab> tabs;
  final int open;
  final int? hovered;
  final ValueChanged<int>? onOpen;

  @override
  State<FtlTabStrip> createState() => _FtlTabStripState();
}

class _FtlTabStripState extends State<FtlTabStrip> {
  late int? _hovered = widget.hovered;

  FtlTabState _state(int i) => switch (i) {
    _ when i == widget.open => FtlTabState.open,
    _ when !widget.tabs[i].enabled => FtlTabState.off,
    _ when i == _hovered => FtlTabState.hover,
    _ => FtlTabState.idle,
  };

  @override
  Widget build(BuildContext context) {
    final font = widget.look.font(FtlFontId.justin12Bold);
    final labels = [for (final t in widget.tabs) t.label.toUpperCase()];
    final widths = [for (final l in labels) font.width(l) + 2 * _labelPad];
    final xs = <int>[];
    for (var i = 0; i < widths.length; i++) {
      xs.add(i == 0 ? _firstTabX : xs[i - 1] + widths[i - 1] + _gap);
    }
    return SizedBox(
      width: (xs.last + widths.last).toDouble(),
      height: tabStripHeight.toDouble(),
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          for (var i = 0; i < labels.length; i++)
            Positioned(
              key: ValueKey('ftl-tab-$i'),
              left: xs[i].toDouble(),
              top: 0,
              width: widths[i].toDouble(),
              height: _tabHeight.toDouble(),
              child: MouseRegion(
                key: widget.tabs[i].key,
                cursor: _state(i) == FtlTabState.idle || _state(i) == FtlTabState.hover
                    ? SystemMouseCursors.click
                    : MouseCursor.defer,
                onEnter: (_) => setState(() => _hovered = i),
                onExit: (_) => setState(() => _hovered = _hovered == i ? null : _hovered),
                child: GestureDetector(
                  onTap: widget.tabs[i].enabled ? () => widget.onOpen?.call(i) : null,
                  child: _TabFace(font: font, state: _state(i), label: labels[i]),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _TabFace extends StatelessWidget {
  const _TabFace({required this.font, required this.state, required this.label});

  final FtlFont font;
  final FtlTabState state;
  final String label;

  @override
  Widget build(BuildContext context) {
    final bottom = state == FtlTabState.open ? _tabHeight : _closedBottom;
    // FtlText centers its capitals in the line box; center the box on the face.
    final top = (_faceTop + bottom - font.lineHeight) ~/ 2;
    final color = switch (state) {
      FtlTabState.open => FtlColors.ink,
      FtlTabState.hover => FtlColors.hover,
      FtlTabState.off => FtlColors.off,
      FtlTabState.idle => FtlColors.line,
    };
    return CustomPaint(
      painter: _TabPainter(state, bottom),
      child: Stack(
        children: [
          Positioned(
            left: _labelPad.toDouble(),
            top: top.toDouble(),
            child: FtlText(label, font: font, color: color),
          ),
        ],
      ),
    );
  }
}

/// A face with its top corners cut by 4 and the glow of FTL's frames; a
/// [chip] face stands free and has all four corners cut, from [_faceTop] to
/// [bottom].
class _TabPainter extends CustomPainter {
  _TabPainter(this.state, this.bottom, {this.chip = false});

  final FtlTabState state;
  final int bottom;
  final bool chip;

  @override
  void paint(Canvas canvas, Size size) {
    final r = Rect.fromLTRB(
      chip ? _faceTop.toDouble() : 0,
      _faceTop.toDouble(),
      size.width - (chip ? _faceTop : 0),
      bottom.toDouble(),
    );
    final path = chip
        ? cutCorners(r, 4)
        : (Path()
            ..moveTo(r.left, r.bottom)
            ..lineTo(r.left, r.top + 4)
            ..lineTo(r.left + 4, r.top)
            ..lineTo(r.right - 4, r.top)
            ..lineTo(r.right, r.top + 4)
            ..lineTo(r.right, r.bottom)
            ..close());
    for (final (width, alpha) in [(7.0, 0x10), (4.0, 0x28), (2.0, 0x45)]) {
      canvas.drawPath(
        path,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = width
          ..color = FtlColors.glow.withAlpha(alpha),
      );
    }
    if (state == FtlTabState.open) {
      canvas.drawPath(path, Paint()..color = FtlColors.line);
      return;
    }
    final edge = switch (state) {
      FtlTabState.hover => FtlColors.hover,
      FtlTabState.off => FtlColors.off,
      _ => FtlColors.line,
    };
    canvas
      ..drawPath(path, Paint()..color = FtlColors.box)
      ..drawPath(
        path,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..color = edge,
      );
  }

  @override
  bool shouldRepaint(_TabPainter old) => old.state != state || old.bottom != bottom || old.chip != chip;
}

/// Height of a chip with its glow: [_faceTop] of glow, the face, [_faceTop]
/// of glow.
const chipHeight = 30;

/// A row of choices in the FTL look that stands free inside a window, such
/// as the ship shown: the faces of the area tabs ([FtlTabStrip]) with all
/// four corners cut. [open] is the chosen one; [hovered] starts the hover
/// (a still shot). Each chip's key goes to its tap target.
class FtlChipRow extends StatefulWidget {
  const FtlChipRow({super.key, required this.look, required this.chips, required this.open, this.hovered, this.onOpen});

  final FtlLook look;
  final List<FtlTab> chips;
  final int open;
  final int? hovered;
  final ValueChanged<int>? onOpen;

  @override
  State<FtlChipRow> createState() => _FtlChipRowState();
}

class _FtlChipRowState extends State<FtlChipRow> {
  late int? _hovered = widget.hovered;

  FtlTabState _state(int i) => switch (i) {
    _ when i == widget.open => FtlTabState.open,
    _ when !widget.chips[i].enabled => FtlTabState.off,
    _ when i == _hovered => FtlTabState.hover,
    _ => FtlTabState.idle,
  };

  @override
  Widget build(BuildContext context) {
    final font = widget.look.font(FtlFontId.justin12Bold);
    const bottom = chipHeight - _faceTop;
    // FtlText centers its capitals in the line box; center the box on the face.
    final top = (_faceTop + bottom - font.lineHeight) ~/ 2;
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        // The glow margins of two chips leave 2 * _faceTop pixels between their faces.
        for (final (i, chip) in widget.chips.indexed)
          MouseRegion(
            key: chip.key,
            cursor: _state(i) == FtlTabState.idle || _state(i) == FtlTabState.hover
                ? SystemMouseCursors.click
                : MouseCursor.defer,
            onEnter: (_) => setState(() => _hovered = i),
            onExit: (_) => setState(() => _hovered = _hovered == i ? null : _hovered),
            child: GestureDetector(
              onTap: chip.enabled ? () => widget.onOpen?.call(i) : null,
              child: SizedBox(
                width: (font.width(chip.label.toUpperCase()) + 2 * _labelPad + 2 * _faceTop).toDouble(),
                height: chipHeight.toDouble(),
                child: CustomPaint(
                  painter: _TabPainter(_state(i), bottom, chip: true),
                  child: Stack(
                    children: [
                      Positioned(
                        left: (_labelPad + _faceTop).toDouble(),
                        top: top.toDouble(),
                        child: FtlText(
                          chip.label.toUpperCase(),
                          font: font,
                          color: switch (_state(i)) {
                            FtlTabState.open => FtlColors.ink,
                            FtlTabState.hover => FtlColors.hover,
                            FtlTabState.off => FtlColors.off,
                            FtlTabState.idle => FtlColors.line,
                          },
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
      ],
    );
  }
}
