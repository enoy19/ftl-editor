import 'package:material_ui/material_ui.dart';

import '../src/rust/api/assets.dart';
import 'game_image.dart';

/// The one place that turns a crew look into a game image. A look id comes
/// from the core (`CrewMember.lookId`, `FloorCrew.lookId`, `RaceChoice.lookId`,
/// `LookChoice.lookId`, `StoreItemView.lookId`) and names race, sex and the
/// tint of each color layer, so a female human wears FTL's `female` sprite
/// and every tint shows wherever crew is drawn. [anchored] keeps FTL's
/// anchoring of the 35 px cell (the floorplan); a portrait centers the art.
/// `test/crew_sprite_guard_test.dart` fails when another file names a crew
/// image kind.
GameImageRef crewSprite(String lookId, {bool anchored = false}) =>
    GameImageRef.of(anchored ? GameImageKind.crewLookFrame : GameImageKind.crewLook, lookId);

/// A crew member drawn from its look ([crewSprite]); the person icon in
/// reduced mode.
class CrewSprite extends StatelessWidget {
  const CrewSprite(this.lookId, {super.key, this.size = 35, this.anchored = false, this.opacity = 1});

  final String lookId;

  /// Width and height; the 35 px cell scales by whole numbers when it is a
  /// multiple of 35.
  final double size;
  final bool anchored;
  final double opacity;

  @override
  Widget build(BuildContext context) {
    final image = GameImage(
      crewSprite(lookId, anchored: anchored),
      fallback: Icons.person,
      width: size,
      height: size,
      iconSize: size / 2,
    );
    return opacity == 1 ? image : Opacity(opacity: opacity, child: image);
  }
}
