// The two ships of a saved game: the player ship and the ship at the
// current beacon. The core addresses every field of the nearby ship as
// `nearby.<path>` with the same `<path>` as on the player ship
// (`ftl_core::Field::Nearby`), so the pages map a path to its icon and label
// through [localPath] and the editor names the ship in the label.

import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import 'components/components.dart';

/// What every edit path of the nearby ship starts with.
const nearbyPrefix = 'nearby.';

/// Whether [path] is a field of the nearby ship.
bool isNearbyPath(String path) => path.startsWith(nearbyPrefix);

/// [path] without its ship: `nearby.systems.shields.level` is
/// `systems.shields.level`.
String localPath(String path) => isNearbyPath(path) ? path.substring(nearbyPrefix.length) : path;

/// The ship part of [path]: `nearby.` or nothing.
String shipPrefix(String path) => isNearbyPath(path) ? nearbyPrefix : '';

/// [look] of [path] with the ship named for a field of the enemy ship
/// ("Enemy ship: Hull"), as the change sheet and the palette list it.
(IconData, String) shipNamed((IconData, String) look, String path, AppLocalizations l) =>
    isNearbyPath(path) ? (look.$1, l.shipEnemyField(look.$2)) : look;

/// [child] below the [ShipSwitch] when [hasNearby], else [child] alone: the
/// Equipment page, whose two panels scroll on their own, shows one ship at a
/// time (the Ship and Crew pages scroll the switch with their content). The
/// key of the page names the ship, so a page does not carry the hover or
/// the scroll of one ship over to the other.
class ShipSwitchFrame extends StatelessWidget {
  const ShipSwitchFrame({
    super.key,
    required this.hasNearby,
    required this.nearby,
    required this.onChanged,
    required this.child,
  });

  final bool hasNearby;
  final bool nearby;
  final ValueChanged<bool> onChanged;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    if (!hasNearby) return child;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(Space.xl, Space.lg, Space.xl, 0),
          child: ShipSwitch(nearby: nearby, onChanged: onChanged),
        ),
        Expanded(
          child: KeyedSubtree(key: ValueKey('ship-shown-$nearby'), child: child),
        ),
      ],
    );
  }
}
