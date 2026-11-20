// The real shell around the real Ship, Crew, Equipment, Map and Profile pages over mock data,
// for widget tests and shots. No Rust core.

import 'package:ftl_editor/shell/app_shell.dart';
import 'package:ftl_editor/shell/status_bar.dart';
import 'package:ftl_editor/src/rust/api/crew.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/equipment.dart';
import 'package:ftl_editor/src/rust/api/expert.dart';
import 'package:ftl_editor/src/rust/api/floorplan.dart';
import 'package:ftl_editor/src/rust/api/map.dart';
import 'package:ftl_editor/src/rust/api/profile.dart';
import 'package:ftl_editor/ui/command_palette.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/crew_page.dart';
import 'package:ftl_editor/ui/equipment_page.dart';
import 'package:ftl_editor/ui/map_page.dart';
import 'package:ftl_editor/ui/expert_page.dart';
import 'package:ftl_editor/ui/profile_page.dart';
import 'package:ftl_editor/ui/ship_choice.dart';
import 'package:ftl_editor/ui/ship_page.dart';
import 'package:material_ui/material_ui.dart';

import 'mock_crew.dart';
import 'mock_equipment.dart';
import 'mock_expert.dart';
import 'mock_profile.dart';
import 'mock_summary.dart';

Widget mockShell({
  Map<String, int> edits = const {},
  VoidCallback? onSave,
  VoidCallback? onSaveProfile,
  bool profileStaged = false,
  List<AchievementInfo> achievementCatalog = mockCatalog,
  VoidCallback? onBackups,
  VoidCallback? onChooseFile,
  VoidCallback? onChooseGameData,
  Widget? banner,
  String? equipmentHovered,
  Equipment? equipment,
  Equipment? equipmentPreview,
  List<AugmentChoice>? augmentChoices,
  List<ItemChoice> itemChoices = mockItemChoices,
  Widget? actions,
  VoidCallback? onUndo,
  VoidCallback? onRedo,
  VoidCallback? onUndoProfile,
  VoidCallback? onRedoProfile,
  PaletteSource? palette,
  Floorplan? floorplan,
  // The Map page shows this sector map; `null` shows no map page.
  SectorMap? map,
  SectorMap? mapPreview,
  SectorTreeView? tree,
  SectorTreeView? treePreview,
  SaveSummary? summary,
  List<CrewMember>? crew,
  CrewTexts crewTexts = mockCrewTexts,
  // `null`: a file without an enemy ship, no switch; else the ship shown.
  bool? nearby,
  ExpertDump? expertDump,
  void Function(FieldValue field, int value)? onExpertEdit,
  ShellStatus status = const ShellStatus(
    path: '/Users/player/Library/Application Support/FasterThanLight/continue.sav',
  ),
}) => AppShell(
  onSave: onSave,
  onSaveProfile: onSaveProfile,
  onBackups: onBackups,
  onChooseFile: onChooseFile,
  onChooseGameData: onChooseGameData,
  banner: banner,
  actions: actions,
  onUndo: onUndo,
  onRedo: onRedo,
  onUndoProfile: onUndoProfile,
  onRedoProfile: onRedoProfile,
  palette: palette,
  status: status,
  expert: (item) => ExpertPage(
    key: ValueKey('expert-${item.name}'),
    view: ExpertView.of(item),
    dump: expertDump ?? mockExpert,
    edits: edits,
    onChanged: onExpertEdit ?? (_, _) {},
  ),
  profile: ProfileBody(
    path: mockProfile.path,
    busy: false,
    onChoose: () {},
    error: null,
    summary: profileStaged ? mockStaged : mockProfile,
    file: mockProfile,
    catalog: achievementCatalog,
    onUnlockAll: () {},
    onLayout: (_, _, _) {},
    onAchievement: (_, _) {},
  ),
  ship: ShipPage(
    path: status.path,
    busy: false,
    onChoose: () {},
    error: null,
    summary: summary ?? mockSummary,
    edits: edits,
    onChanged: (_, _) {},
    floorplan: floorplan,
    crew: crew ?? fullCrew,
    equipment: equipment ?? mockEquipment,
    texts: crewTexts,
    nearby: nearby ?? false,
    shipSwitch: _shipSwitch(nearby),
  ),
  map: map == null
      ? null
      : MapPage(
          map: map,
          preview: mapPreview,
          edits: edits,
          onChanged: (_, _) {},
          tree: tree,
          treePreview: treePreview,
        ),
  crew: CrewPage(
    crew: crew ?? fullCrew,
    edits: edits,
    onChanged: (_, _) {},
    texts: crewTexts,
    shipSwitch: _shipSwitch(nearby),
  ),
  equipment: _switch(
    nearby,
    EquipmentPage(
      equipment: equipment ?? mockEquipment,
      edits: edits,
      onChanged: (_, _) {},
      addedAugments: const [],
      choices: augmentChoices ?? mockAugmentChoices,
      onAddAugment: (_) {},
      preview: equipmentPreview,
      itemChoices: itemChoices,
      onAddItem: (_, {required toCargo}) {},
      hovered: equipmentHovered,
    ),
  ),
);

/// The switch the Ship and Crew pages scroll with their content.
Widget? _shipSwitch(bool? nearby) => nearby == null ? null : ShipSwitch(nearby: nearby, onChanged: (_) {});

Widget _switch(bool? nearby, Widget page) =>
    ShipSwitchFrame(hasNearby: nearby != null, nearby: nearby ?? false, onChanged: (_) {}, child: page);
