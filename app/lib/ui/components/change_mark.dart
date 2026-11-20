import 'package:material_ui/material_ui.dart';

import '../../look/ftl_look.dart';
import 'tokens.dart';

/// The direction of a pending edit against the value of the file.
enum EditDirection { raised, lowered }

/// The direction of [value] against the file value [old]; `null` when equal.
EditDirection? directionOf(int old, int value) => value == old
    ? null
    : value > old
    ? EditDirection.raised
    : EditDirection.lowered;

/// A small triangle that marks a value with a pending edit: pointing up in
/// the accent color (FTL's yellow) when the edit raises it, pointing down in
/// the warning color (FTL's red) when it lowers it, with a dark contour so it
/// reads on the light floor tiles and on the dark window alike.
class ChangeMark extends StatelessWidget {
  const ChangeMark(this.direction, {super.key, this.size = 10});

  final EditDirection direction;
  final double size;

  @override
  Widget build(BuildContext context) {
    final t = AppTokens.of(context);
    return CustomPaint(
      size: Size.square(size),
      painter: _TrianglePainter(direction, direction == EditDirection.raised ? t.accent : t.warn),
    );
  }
}

class _TrianglePainter extends CustomPainter {
  _TrianglePainter(this.direction, this.color);

  final EditDirection direction;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final w = size.width;
    final h = size.height;
    final path = direction == EditDirection.raised
        ? (Path()
            ..moveTo(w / 2, 0)
            ..lineTo(w, h)
            ..lineTo(0, h)
            ..close())
        : (Path()
            ..moveTo(0, 0)
            ..lineTo(w, 0)
            ..lineTo(w / 2, h)
            ..close());
    canvas.drawPath(path, Paint()..color = color);
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1
        ..color = FtlColors.ground,
    );
  }

  @override
  bool shouldRepaint(_TrianglePainter old) => old.direction != direction || old.color != color;
}
