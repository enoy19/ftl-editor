import 'package:material_ui/material_ui.dart';

import '../../l10n/app_localizations.dart';
import '../../look/ftl_tabs.dart';
import '../../look/ftl_theme.dart';

/// The choice between the player ship and the enemy ship at the beacon on
/// the Ship, Crew and Equipment pages. Two chips in the FTL look (the faces
/// of the area tabs with all corners cut; the open one is the ship shown), a
/// segmented button in the Material look. It is part of the page, never
/// pinned above it. The keys are `ship-player` and `ship-nearby`.
class ShipSwitch extends StatelessWidget {
  const ShipSwitch({super.key, required this.nearby, required this.onChanged});

  /// Whether the enemy ship is shown.
  final bool nearby;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final look = FtlTheme.of(context);
    if (look != null) {
      return Align(
        alignment: Alignment.centerLeft,
        child: FtlChipRow(
          key: const ValueKey('ship-switch'),
          look: look,
          chips: [
            FtlTab(l.shipPlayer, key: const ValueKey('ship-player')),
            FtlTab(l.shipEnemy, key: const ValueKey('ship-nearby')),
          ],
          open: nearby ? 1 : 0,
          onOpen: (i) => onChanged(i == 1),
        ),
      );
    }
    return Align(
      alignment: Alignment.centerLeft,
      child: SegmentedButton<bool>(
        key: const ValueKey('ship-switch'),
        showSelectedIcon: false,
        segments: [
          ButtonSegment(
            value: false,
            icon: const Icon(Icons.rocket_launch, key: ValueKey('ship-player')),
            label: Text(l.shipPlayer),
          ),
          ButtonSegment(
            value: true,
            icon: const Icon(Icons.gps_fixed, key: ValueKey('ship-nearby')),
            label: Text(l.shipEnemy),
          ),
        ],
        selected: {nearby},
        onSelectionChanged: (s) => onChanged(s.first),
      ),
    );
  }
}
