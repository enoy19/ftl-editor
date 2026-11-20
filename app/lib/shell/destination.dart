import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';

/// Where the top bar can go: five area tabs, then the items of the Expert
/// menu (decision ftl-editor-uses-top-tabs-and-portrait-crew-cards).
enum Destination {
  ship(Icons.rocket_launch_outlined, Icons.rocket_launch),
  crew(Icons.groups_outlined, Icons.groups),
  equipment(Icons.handyman_outlined, Icons.handyman),
  map(Icons.map_outlined, Icons.map),
  profile(Icons.emoji_events_outlined, Icons.emoji_events),
  dump(Icons.data_object, Icons.data_object),
  stateVars(Icons.tune, Icons.tune),
  unknownFields(Icons.help_outline, Icons.help);

  const Destination(this.icon, this.selectedIcon);

  final IconData icon;
  final IconData selectedIcon;

  bool get expert => index >= Destination.dump.index;

  static List<Destination> get areas => values.where((d) => !d.expert).toList();
  static List<Destination> get expertItems => values.where((d) => d.expert).toList();

  String label(AppLocalizations l) => switch (this) {
    ship => l.areaShip,
    crew => l.areaCrew,
    equipment => l.areaEquipment,
    map => l.areaMap,
    profile => l.areaProfile,
    dump => l.expertDump,
    stateVars => l.expertStateVars,
    unknownFields => l.expertUnknown,
  };

  /// The one sentence of a page that a later step builds.
  String later(AppLocalizations l) => switch (this) {
    ship || crew || equipment || profile || dump || stateVars || unknownFields => '',
    map => l.laterMap,
  };
}

const expertIcon = Icons.science_outlined;
