// The one lookup from an edit path to its game image in `ftl.dat`, shared by
// the change sheet, the exact-value popover and the command palette. A path
// without a game image keeps the Material icon of its look.

import 'package:material_ui/material_ui.dart';

import '../src/rust/api/assets.dart';
import '../src/rust/api/crew.dart';
import '../src/rust/api/equipment.dart';
import 'crew_card.dart';
import 'crew_sprite.dart';
import 'game_image.dart';
import 'resource_bar.dart';
import 'ship_choice.dart';

/// A game image that stands for a field: [ref], whether it is one of FTL's
/// white symbols that take the color of the text ([tint]), and the quarter
/// turns it needs to stand upright (a weapon frame lies on its side).
@immutable
class FieldImage {
  const FieldImage(this.ref, {this.tint = true, this.quarterTurns = 0});

  final GameImageRef ref;
  final bool tint;
  final int quarterTurns;

  @override
  bool operator ==(Object other) =>
      other is FieldImage && other.ref == ref && other.tint == tint && other.quarterTurns == quarterTurns;

  @override
  int get hashCode => Object.hash(ref, tint, quarterTurns);
}

/// Answers the game image of a field path, or `null`.
typedef FieldImageOf = FieldImage? Function(String path);

/// FTL's fire symbol of the HUD (`img/icons/s_fire2.png`, 32x32), drawn in
/// its own red.
const fireImage = FieldImage(GameImageRef.path('img/icons/s_fire2.png'), tint: false);

/// The game image of the field at [path] of either ship:
///
/// - a resource: FTL's `ui_icons` ([resourceImage]),
/// - a system level or install: the white symbol `s_<id>_overlay.png`
///   ([GameImageKind.systemIcon]),
/// - a skill: the skill icon of the crew card ([skillIcon]),
/// - health, name and race of a crew member of [crew]: their portrait,
/// - a weapon or drone count, armed count or cargo box: its blueprint art
///   (a cargo id is a drone when [equipment] or [choices] say so),
/// - a fire: FTL's fire symbol,
/// - the oxygen of a room: the symbol of the oxygen system.
///
/// `null` for every other path (reactor, augments, breaches, the map, the
/// Expert fields): `ftl.dat` has no small image for them.
FieldImage? fieldImage(
  String path, {
  List<CrewMember> crew = const [],
  Equipment? equipment,
  List<ItemChoice> choices = const [],
}) {
  if (resourceImage(path) case final ref?) return FieldImage(ref);
  bool isDrone(String id) =>
      [
        ...?equipment?.weapons,
        ...?equipment?.drones,
        ...?equipment?.cargo,
      ].where((i) => i.id == id).firstOrNull?.drone ??
      choices.where((c) => c.id == id).firstOrNull?.drone ??
      false;
  FieldImage item(String id, {required bool drone}) => drone
      ? FieldImage(GameImageRef.of(GameImageKind.drone, id), tint: false)
      : FieldImage(GameImageRef.of(GameImageKind.weaponFrame, id), tint: false, quarterTurns: 1);
  switch (localPath(path).split('.')) {
    case ['systems', final id, 'level' || 'installed' || 'damage']:
      return FieldImage(GameImageRef.of(GameImageKind.systemIcon, id));
    case ['systems', _, 'ion']:
      return const FieldImage(GameImageRef.path('img/icons/locking/s_lock.png'), tint: false);
    case ['crew', final n, 'skills', final skill]:
      return int.tryParse(n) == null ? null : FieldImage(skillIcon(skill));
    case ['crew', final n, 'health' || 'name' || 'race']:
      final c = crew.where((c) => '${c.index}' == n).firstOrNull;
      return c == null ? null : FieldImage(crewSprite(c.lookId), tint: false);
    case ['weapons', final id] || ['weapons', final id, 'armed']:
      return item(id, drone: false);
    case ['drones', final id] || ['drones', final id, 'armed']:
      return item(id, drone: true);
    case ['cargo', final id]:
      return item(id, drone: isDrone(id));
    case ['fires', _, _]:
      return fireImage;
    case ['rooms', _, 'oxygen']:
      return const FieldImage(GameImageRef.of(GameImageKind.systemIcon, 'oxygen'));
  }
  return null;
}

/// The icon of a field: its [image] from the open `ftl.dat` as a [GameIcon]
/// (cropped to its visible pixels, a whole factor of the size of the icons
/// around it), a white symbol tinted with [color]; else (reduced mode, no
/// image, an image that cannot be read) the Material [fallback].
class FieldIcon extends StatelessWidget {
  const FieldIcon({super.key, required this.image, required this.fallback, this.size, this.color});

  final FieldImage? image;
  final IconData fallback;

  /// `null`: the size of the Material icons around it.
  final double? size;

  /// The color of the text beside it; `null` takes the icon theme.
  final Color? color;

  @override
  Widget build(BuildContext context) {
    final image = this.image;
    final color = this.color ?? IconTheme.of(context).color;
    if (image == null || GameDataScope.maybeOf(context)?.datPath == null) {
      return Icon(fallback, size: size, color: color);
    }
    final game = GameIcon(image.ref, fallback: fallback, size: size, color: image.tint ? color : null);
    return image.quarterTurns == 0 ? game : RotatedBox(quarterTurns: image.quarterTurns, child: game);
  }
}
