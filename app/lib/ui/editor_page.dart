import 'dart:io';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/foundation.dart' show mapEquals;
import 'package:material_ui/material_ui.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../l10n/app_localizations.dart';
import '../look/ftl_look.dart';
import '../look/ftl_theme.dart';
import '../shell/app_shell.dart';
import '../shell/status_bar.dart';
import '../src/rust/api/actions.dart';
import '../src/rust/api/assets.dart';
import '../src/rust/api/crew.dart';
import '../src/rust/api/editor.dart';
import '../src/rust/api/equipment.dart';
import '../src/rust/api/expert.dart';
import '../src/rust/api/floorplan.dart';
import '../src/rust/api/identity.dart';
import '../src/rust/api/map.dart';
import '../src/rust/api/status.dart';
import 'backup_history.dart';
import 'change_sheet.dart';
import 'components/components.dart';
import 'command_palette.dart';
import 'edit_history.dart';
import 'crew_page.dart';
import 'equipment_page.dart';
import 'error_text.dart';
import 'expert_page.dart';
import 'field_image.dart';
import 'floorplan/floorplan_view.dart';
import 'game_image.dart';
import 'map_page.dart';
import 'profile_page.dart';
import 'quick_actions.dart';
import 'resource_bar.dart';
import 'save_dialogs.dart';
import 'save_location.dart';
import 'sector_tree.dart';
import 'ship_choice.dart';
import 'ship_card.dart';

export 'ship_card.dart' show ShipCard;
import 'ship_page.dart';

/// Asks the person for a saved game; answers its path or `null` on cancel.
typedef PickFile = Future<String?> Function(BuildContext context, String? initialDirectory);

/// The native file dialog of the OS.
Future<String?> nativePickFile(BuildContext context, String? initialDirectory) async {
  final l = AppLocalizations.of(context)!;
  final file = await openFile(
    acceptedTypeGroups: [
      XTypeGroup(label: l.ftlSaves, extensions: const ['sav']),
      XTypeGroup(label: l.allFiles),
    ],
    initialDirectory: initialDirectory,
    confirmButtonText: l.open,
  );
  return file?.path;
}

/// Asks the person for `ftl.dat`, its folder or `FTL.app`; answers the path
/// or `null` on cancel.
typedef PickFtlDat = Future<String?> Function(BuildContext context);

/// The native file dialog of the OS. It accepts `FTL.app` as one item,
/// because a macOS dialog does not open the bundle.
Future<String?> nativePickFtlDat(BuildContext context) async {
  final l = AppLocalizations.of(context)!;
  final file = await openFile(
    acceptedTypeGroups: [
      XTypeGroup(label: l.ftlDatFiles, extensions: const ['dat']),
      XTypeGroup(
        label: l.ftlDatApp,
        extensions: const ['app'],
        uniformTypeIdentifiers: const ['com.apple.application-bundle'],
      ),
    ],
    confirmButtonText: l.open,
  );
  return file?.path;
}

/// Answers the warnings of the core for a save file (running FTL, Steam
/// Cloud); tests inject their own.
typedef WarningsOf = Future<List<FileWarning>> Function(String path);

/// The warnings of the Rust core for the file at [path].
Future<List<FileWarning>> coreWarnings(String path) => saveWarnings(path: path);

/// Opens a saved game, shows it in the [AppShell] and saves edits.
///
/// One action chooses a file: the button opens the file dialog and loads the
/// choice. There is no reload button: when the window gets the focus back
/// (after quitting FTL) and nothing is edited, the page reads the file again.
///
/// Game images come from the local `ftl.dat`. At start the page opens the
/// remembered `ftl.dat`; a found one waits for a confirm, and without one the
/// person picks it. Until then the page runs in reduced mode (Material icons).
/// With `ftl.dat` open the page wears the original FTL look ([FtlTheme]); a
/// look that cannot be read leaves the Material look and the status bar says
/// why.
///
/// Every rule (ranges, verify, backup, changed-on-disk check) lives in the
/// Rust core. This page only collects input and shows what Rust answers.
///
/// The page reads the warnings of the open file (running FTL, Steam Cloud)
/// at open, when the window gets the focus back and before a save. While FTL
/// runs, a save asks first; Cancel is the default.
///
/// Save opens a sheet with every change and an undo per row before it
/// writes. A click on the backup item of the status bar lists the backups
/// of the open file; a restore writes through the same checks as a save.
class EditorPage extends StatefulWidget {
  const EditorPage({
    super.key,
    this.backupDir,
    this.pickFile = nativePickFile,
    this.defaultFile,
    this.assetsConfigDir,
    this.locateFtlDat,
    this.pickFtlDat = nativePickFtlDat,
    this.defaultProfileFile,
    this.warningsOf = coreWarnings,
    this.loadLook = FtlLook.load,
  });

  /// Reads the FTL look from the open `ftl.dat`; tests replace it.
  final Future<FtlLook> Function() loadLook;

  /// Warnings of a save file; tests replace the process list and the folder.
  final WarningsOf warningsOf;

  /// Backup folder for saves, restores and the history; `null` uses the
  /// editor's data folder.
  final String? backupDir;

  /// File chooser; tests replace the native dialog.
  final PickFile pickFile;

  /// FTL's save file; `null` asks the core for the file of this OS.
  final String? defaultFile;

  /// Folder of the editor's config file with the confirmed `ftl.dat`; `null`
  /// is the OS config folder. Tests pass their own.
  final String? assetsConfigDir;

  /// Locates `ftl.dat`; `null` asks the core. Tests replace the search.
  final Future<FtlDatStatus> Function()? locateFtlDat;

  /// Chooser for `ftl.dat`; tests replace the native dialog.
  final PickFtlDat pickFtlDat;

  /// FTL's profile for the Profile page; `null` asks the core for the file
  /// of this OS.
  final String? defaultProfileFile;

  @override
  State<EditorPage> createState() => _EditorPageState();
}

class _EditorPageState extends State<EditorPage> {
  /// Marks the page below the [Theme] of the look ([build]).
  final _pageKey = GlobalKey();

  /// The context for dialogs, popovers and snack bars: the page below the
  /// [Theme] of the look. The state's own [context] sits above that Theme,
  /// so a dialog opened from it fell back to the Material look.
  BuildContext get _ui => _pageKey.currentContext ?? context;

  /// New values by path, only those that differ from the file. The resource
  /// bar accepts only values inside the core's range.
  final Map<String, int> _edits = {};

  /// New crew names and races by path (`crew.<n>.name`, `crew.<n>.race`),
  /// only those that differ from the file; saved with [_edits].
  final Map<String, String> _texts = {};

  /// The steps of Undo and Redo over [_edits] and [_texts]; a newly read
  /// file (open, save, restore) clears them.
  final EditHistory<_Staged> _history = EditHistory();

  /// The races of the race picker, named in the language and the game data
  /// of [_racesKey].
  List<RaceChoice> _races = const [];
  String? _racesKey;

  /// The staged edits of the profile; Save on the Profile page writes them.
  final ProfileEdits _profileEdits = ProfileEdits();
  late final AppLifecycleListener _lifecycle;

  String? _defaultFile;
  String? _path;
  SaveSession? _session;
  SaveSummary? _summary;

  /// The crew of the open file; its fields join the edits of the Ship page.
  List<CrewMember>? _crew;

  /// Name, type, difficulty and sector of the player ship and of the enemy
  /// ship (`null` without one), for the ship card.
  ShipIdentity? _identity;
  ShipIdentity? _nearbyIdentity;

  /// The hangar ships as types of the player ship, named in the language
  /// and the game data of [_typesKey].
  List<ShipTypeChoice> _types = const [];
  String? _typesKey;

  /// The floorplan of the open file from the open `ftl.dat`; `null` in
  /// reduced mode. Its fires and breaches join the edits.
  Floorplan? _floorplan;

  /// Whether the Ship, Crew and Equipment pages show the enemy ship at the
  /// beacon instead of the player ship ([ShipSwitch]).
  bool _nearby = false;

  /// The crew, floorplan and equipment of the enemy ship, read like those
  /// of the player ship; `null` without one. Their fields (`nearby.<path>`)
  /// join the edits as well, so the change sheet lists the edits of both ships.
  List<CrewMember>? _nearbyCrew;

  /// `crew.add` of the player ship and of the enemy ship; `null` without
  /// their rooms or ship.
  FieldValue? _crewAdd;
  FieldValue? _nearbyCrewAdd;

  /// The new crew members `crew.add` stages, by ship and count, as the core
  /// builds them without any other edit: their fields start at these values.
  final Map<String, List<CrewMember>> _addedCrew = {};

  /// The looks of the character creator by race.
  final Map<String, List<LookChoice>> _looks = {};
  Floorplan? _nearbyFloorplan;
  Equipment? _nearbyEquipment;

  /// The equipment of the open file in [_equipmentLanguage]; `null` until
  /// [_equipmentIn] reads it. Its fields join the edits too.
  Equipment? _equipment;
  String? _equipmentLanguage;
  List<AugmentChoice> _augmentChoices = const [];
  List<ItemChoice> _itemChoices = const [];

  /// The sector map of the open file in [_mapLanguage] (store item names);
  /// `null` until [_mapIn] reads it, or when the world is not decoded. Its
  /// fields join the edits.
  SectorMap? _map;
  String? _mapLanguage;

  /// The sector tree of the file, read with [_map]; [_treeBlock] the
  /// core's code when there is none.
  SectorTreeView? _tree;
  String? _treeBlock;

  /// Augments added since the file was read, by `augments.<ID>`.
  final Map<String, FieldValue> _addedAugments = {};

  /// The fields of weapons and drones the file lacked and the person added
  /// since (`itemFields` of the core), by path.
  final Map<String, FieldValue> _addedItems = {};

  /// The raw view of the open file for the Expert area; `null` until
  /// [_expertDump] reads it.
  ExpertDump? _expert;

  /// The fields edited in the Expert area since the file was read
  /// (`raw.<path>`, `state.<id>`), by path; they join [_fields].
  final Map<String, FieldValue> _rawFields = {};

  /// The warnings of the open file; `null` until they are read.
  SaveGuard? _guard;

  /// The last backup a save of this session wrote.
  LastBackup? _lastBackup;

  /// The last error of open or save, rendered with the current locale.
  Object? _error;
  bool _busy = false;

  /// What the core located for `ftl.dat`; `null` until it answered.
  FtlDatStatus? _dat;

  /// The `ftl.dat` whose game data is open; `null` is the reduced mode.
  String? _datOpen;

  /// The theme of the FTL look of the open `ftl.dat`; `null` is the Material look.
  ThemeData? _lookTheme;

  /// Why the FTL look could not be read from the open `ftl.dat`.
  Object? _lookError;

  /// The last error of locate, remember or open of `ftl.dat`.
  Object? _datError;
  bool _datBusy = false;

  @override
  void initState() {
    super.initState();
    _lifecycle = AppLifecycleListener(onResume: _resume);
    _profileEdits.addListener(_profileEdited);
    _start();
    _startGameData();
  }

  @override
  void dispose() {
    _lifecycle.dispose();
    _profileEdits.dispose();
    super.dispose();
  }

  /// Save follows the staged profile edits.
  void _profileEdited() => setState(() {});

  /// Opens the last opened file, or FTL's default file.
  Future<void> _start() async {
    try {
      _defaultFile = widget.defaultFile ?? defaultSaveFile();
    } catch (e) {
      setState(() => _error = e);
    }
    final prefs = await SharedPreferences.getInstance();
    final start = SaveLocation.startFile(
      lastFile: prefs.getString(SaveLocation.lastFileKey),
      defaultFile: _defaultFile ?? '',
    );
    if (start.isNotEmpty) await _open(start);
  }

  /// Opens the remembered `ftl.dat`; a found or missing one waits for the person.
  Future<void> _startGameData() async {
    try {
      final status = await (widget.locateFtlDat?.call() ?? ftlDatStatus(configDir: widget.assetsConfigDir));
      setState(() => _dat = status);
      final path = status.path;
      if (status.source == DatSource.remembered && path != null) await _openGameData(path);
    } catch (e) {
      setState(() => _datError = e);
    }
  }

  Future<void> _openGameData(String path) async {
    setState(() {
      _datBusy = true;
      _datError = null;
    });
    try {
      await openGameData(path: path);
      // The look switches together with the game images, in one frame.
      ThemeData? lookTheme;
      Object? lookError;
      try {
        lookTheme = ftlLookTheme(await widget.loadLook());
      } catch (e) {
        lookError = e;
      }
      // The race titles of the crew come from the game data that just opened.
      final session = _session;
      setState(() {
        _datOpen = path;
        _lookTheme = lookTheme;
        _lookError = lookError;
        if (session != null) _readShips(session);
        // Names and blueprint caps of the equipment come from it too.
        _equipment = null;
        _map = null;
      });
    } catch (e) {
      setState(() => _datError = e);
    } finally {
      setState(() => _datBusy = false);
    }
  }

  /// Remembers the confirmed or picked `ftl.dat` and opens it.
  Future<void> _useFtlDat(String picked) async {
    try {
      final dat = await rememberFtlDat(configDir: widget.assetsConfigDir, picked: picked);
      setState(() => _dat = FtlDatStatus(path: dat, source: DatSource.remembered, tried: const []));
      await _openGameData(dat);
    } catch (e) {
      setState(() => _datError = e);
    }
  }

  Future<void> _pickFtlDat() async {
    final picked = await widget.pickFtlDat(context);
    if (picked != null) await _useFtlDat(picked);
  }

  Future<void> _choose() async {
    final dir = SaveLocation.dialogDirectory(currentPath: _path ?? '', defaultFile: _defaultFile);
    final path = await widget.pickFile(context, dir);
    if (path != null) await _open(path);
  }

  /// Reads the open file again from disk; asks first when edits would be lost.
  Future<void> _reload() async {
    final path = _path;
    if (path == null || _busy) return;
    if (_hasEdits && !await confirmDiscardEdits(_ui)) return;
    if (mounted) await _open(path);
  }

  /// The window has the focus again: FTL may have started or quit.
  void _resume() {
    final path = _path;
    if (path == null || _busy) return;
    if (_hasEdits) {
      _readWarnings(path);
    } else {
      _open(path);
    }
  }

  /// Reads the warnings of [path] and shows them; answers them for a save.
  Future<SaveGuard?> _readWarnings(String path) async {
    try {
      final guard = SaveGuard.of(await widget.warningsOf(path));
      if (mounted && _path == path) setState(() => _guard = guard);
      return guard;
    } catch (e) {
      // The core answers a failed check as a warning; this is a bridge fault.
      if (mounted) setState(() => _guard = null);
      return null;
    }
  }

  Future<void> _open(String path) async {
    setState(() {
      _busy = true;
      _path = path;
      _error = null;
    });
    try {
      final session = await openSave(path: path);
      _show(session, session.summary());
      await _readWarnings(path);
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(SaveLocation.lastFileKey, path);
    } catch (e) {
      setState(() {
        _session = null;
        _summary = null;
        _crew = null;
        _identity = null;
        _nearbyIdentity = null;
        _typesKey = null;
        _floorplan = null;
        _equipment = null;
        _map = null;
        _nearbyCrew = null;
        _nearbyFloorplan = null;
        _nearbyEquipment = null;
        _expert = null;
        _guard = null;
        _error = e;
      });
    } finally {
      setState(() => _busy = false);
    }
  }

  void _show(SaveSession session, SaveSummary summary) {
    setState(() {
      _session = session;
      _summary = summary;
      _readShips(session);
      // The enemy ship stays on screen while the file still has one.
      _nearby = _nearby && summary.nearbyBlueprintId != null;
      _equipment = null;
      _map = null;
      _expert = null;
      _rawFields.clear();
      _addedAugments.clear();
      _addedItems.clear();
      _edits.clear();
      _texts.clear();
      _history.clear();
    });
  }

  /// Reads the crew and the floorplan of both ships of [session].
  void _readShips(SaveSession session) {
    _identity = identityOf(session: session, nearby: false);
    _nearbyIdentity = session.summary().nearbyBlueprintId == null ? null : identityOf(session: session, nearby: true);
    _typesKey = null;
    _crew = crewOf(session: session, nearby: false);
    _crewAdd = crewAddField(session: session, nearby: false);
    _nearbyCrewAdd = crewAddField(session: session, nearby: true);
    _addedCrew.clear();
    _floorplan = _floorplanOf(session, nearby: false);
    _nearbyCrew = crewOf(session: session, nearby: true);
    _nearbyFloorplan = _floorplanOf(session, nearby: true);
  }

  /// The floorplan of a ship of [session] once the app has opened `ftl.dat`.
  /// The core may hold game data already (opening a save locates `ftl.dat`
  /// for the layouts), but the reduced mode shows the Ship page without a
  /// floorplan.
  Floorplan? _floorplanOf(SaveSession session, {required bool nearby}) =>
      _datOpen == null ? null : floorplanOf(session: session, nearby: nearby);

  /// The equipment of the open file with names in [language], read once per
  /// file, game data and language, for the ship shown; the enemy ship's is
  /// read with it.
  Equipment? _equipmentIn(String language) {
    final session = _session;
    if (session == null) return null;
    if (_equipment == null || _equipmentLanguage != language) {
      _equipment = equipmentOf(session: session, nearby: false, language: language);
      _nearbyEquipment = _summary?.nearbyBlueprintId == null
          ? null
          : equipmentOf(session: session, nearby: true, language: language);
      _augmentChoices = augmentChoices(language: language);
      _itemChoices = itemChoices(language: language);
      _equipmentLanguage = language;
    }
    return _nearby ? _nearbyEquipment : _equipment;
  }

  /// The sector map of the open file with names in [language], read once
  /// per file, game data and language; `null` without a decoded world.
  SectorMap? _mapIn(String language) {
    final session = _session;
    if (session == null) return null;
    if (_mapLanguage != language || _map == null) {
      try {
        _map = sectorMapOf(session: session, language: language);
      } catch (_) {
        _map = null;
      }
      try {
        _tree = sectorTreeOf(session: session, language: language);
        _treeBlock = null;
      } on EditorError catch (e) {
        _tree = null;
        _treeBlock = e.code;
      } catch (_) {
        _tree = null;
        _treeBlock = null;
      }
      _mapLanguage = language;
    }
    return _map;
  }

  /// The sector map as the pending edits leave it, or why the core refused
  /// them; `(null, null)` without a map or without edits.
  (SectorMap?, String?) _mapPreview(String language, AppLocalizations l) {
    final session = _session;
    if (session == null || _map == null || !_hasEdits) return (null, null);
    try {
      final edits = [for (final e in _edits.entries) FieldEdit(path: e.key, value: e.value)];
      final texts = [for (final e in _texts.entries) TextEdit(path: e.key, text: e.value)];
      return (sectorMapPreview(session: session, edits: edits, texts: texts, language: language), null);
    } catch (e) {
      return (null, l.error(errorText(e, l)));
    }
  }

  /// The sector tree as a staged jump leaves it; `null` without one.
  SectorTreeView? _treePreview(String language) {
    final session = _session;
    if (session == null || _tree == null || !_edits.containsKey(sectorJumpPath)) return null;
    try {
      final edits = [for (final e in _edits.entries) FieldEdit(path: e.key, value: e.value)];
      final texts = [for (final e in _texts.entries) TextEdit(path: e.key, text: e.value)];
      return sectorTreePreview(session: session, edits: edits, texts: texts, language: language);
    } catch (_) {
      return null;
    }
  }

  /// Stages a sector jump. A jump replaces the beacons of the sector, so
  /// it drops the pending edits of the old map (Undo brings them back).
  void _editJump(String path, int value) {
    final inFile = _tree?.jumpField?.value;
    _change(() {
      _edits.removeWhere(
        (p, _) => p == path || p == 'fleet.offset' || p == 'beacons.current' || p.startsWith('beacons.'),
      );
      _texts.removeWhere((p, _) => p.startsWith('beacons.'));
      if (value != inFile) _edits[path] = value;
    });
  }

  /// The sector map with only the stores the pending edits add, as they
  /// start: the fields of a new store and the items a swap replaces. The
  /// file's map without such an edit.
  SectorMap? get _mapBase {
    final session = _session;
    final map = _map;
    if (session == null || map == null) return map;
    final added = [
      for (final e in _edits.entries)
        if (_isStoreFlag(e.key) && e.value == 1) FieldEdit(path: e.key, value: 1),
    ];
    if (added.isEmpty) return map;
    final key = '${_mapLanguage ?? ''}|${added.map((e) => e.path).join(',')}';
    if (_mapBaseKey != key) {
      try {
        _mapBaseCache = sectorMapPreview(
          session: session,
          edits: added,
          texts: const [],
          language: _mapLanguage ?? 'en',
        );
      } catch (_) {
        _mapBaseCache = map;
      }
      _mapBaseKey = key;
    }
    return _mapBaseCache;
  }

  SectorMap? _mapBaseCache;
  String? _mapBaseKey;

  static bool _isStoreFlag(String path) => RegExp(r'^beacons\.\d+\.store$').hasMatch(path);

  /// The fields of the stores the pending edits add ([_mapBase]).
  List<FieldValue> get _addedStoreFields {
    final base = _mapBase;
    if (identical(base, _map) || base == null) return const [];
    final inFile = {for (final f in mapFields(_map)) f.path};
    return [
      for (final f in mapFields(base))
        if (!inFile.contains(f.path)) f,
    ];
  }

  /// The blueprints a store slot of [kind] can take, in the app language.
  List<StoreChoice> _storeChoices(String kind) => switch (kind) {
    'weapon' => [
      for (final c in _itemChoices)
        if (!c.drone) (id: c.id, name: c.name),
    ],
    'drone' => [
      for (final c in _itemChoices)
        if (c.drone) (id: c.id, name: c.name),
    ],
    'augment' => [for (final c in _augmentChoices) (id: c.id, name: c.name)],
    'crew' => [for (final r in _races) (id: r.id, name: r.name ?? r.id)],
    _ => const [],
  };

  /// Adds one augment of [id] to the ship shown: its field joins the edits,
  /// the core keeps the limit of three.
  void _addAugment(String id) {
    final session = _session;
    if (session == null) return;
    final l = AppLocalizations.of(context)!;
    final path = '${_nearby ? nearbyPrefix : ''}augments.$id';
    final FieldValue field;
    try {
      field =
          _fields.where((f) => f.path == path).firstOrNull ?? augmentField(session: session, nearby: _nearby, id: id);
    } catch (e) {
      _snack(Icons.error, l.error(errorText(e, l)));
      return;
    }
    setState(() => _addedAugments.putIfAbsent(field.path, () => field));
    _edit(field.path, (_edits[field.path] ?? field.value) + 1);
  }

  /// Adds one weapon or drone of [id] to the slots or with [toCargo] to the
  /// cargo: its fields join the edits, the core keeps the slots, the cargo
  /// boxes and the blueprints.
  void _addItem(String id, {required bool toCargo}) {
    final session = _session;
    if (session == null) return;
    final fields = itemFields(session: session, nearby: _nearby, id: id);
    final target = toCargo ? fields?.cargo : fields?.slots;
    if (fields == null || target == null) return;
    setState(() {
      for (final f in itemFieldValues(fields)) {
        if (!_fields.any((g) => g.path == f.path)) _addedItems[f.path] = f;
      }
    });
    final inFile = _fields.firstWhere((f) => f.path == target.path).value;
    _edit(target.path, (_edits[target.path] ?? inFile) + 1);
  }

  /// The equipment of the ship shown as the pending edits leave it, or why
  /// the core refused them; `(null, null)` without a file.
  (Equipment?, String?) _preview(String language, AppLocalizations l) {
    final session = _session;
    if (session == null || (_nearby ? _nearbyEquipment : _equipment) == null) return (null, null);
    try {
      final edits = [for (final e in _edits.entries) FieldEdit(path: e.key, value: e.value)];
      return (equipmentPreview(session: session, nearby: _nearby, edits: edits, language: language), null);
    } catch (e) {
      return (null, l.error(errorText(e, l)));
    }
  }

  /// The floorplan and the equipment of the ship shown as the pending edits
  /// leave them, for the rooms of the Ship page; `(null, null)` without a
  /// floorplan, without edits or when the core refused them (the Equipment
  /// page names why).
  (Floorplan?, Equipment?) _shipPreview(String language) {
    final session = _session;
    final plan = _nearby ? _nearbyFloorplan : _floorplan;
    if (session == null || plan == null || _edits.isEmpty) return (null, null);
    try {
      final edits = [for (final e in _edits.entries) FieldEdit(path: e.key, value: e.value)];
      return (
        floorplanPreview(session: session, nearby: _nearby, edits: edits),
        equipmentPreview(session: session, nearby: _nearby, edits: edits, language: language),
      );
    } catch (_) {
      return (null, null);
    }
  }

  /// Every editable field of the open file: the resources, the crew, the
  /// equipment and the floorplan of both ships, the sector map, and the
  /// augments, weapons and drones added since.
  List<FieldValue> get _fields => [
    ...?_summary?.fields,
    ...crewFields(_crewOf(nearby: false)),
    ?_crewAdd,
    ...equipmentFields(_equipment),
    ...floorplanFields(_floorplan),
    ...?_summary?.nearbyFields,
    ...crewFields(_crewOf(nearby: true)),
    ?_nearbyCrewAdd,
    ...equipmentFields(_nearbyEquipment),
    ...floorplanFields(_nearbyFloorplan),
    ...mapFields(_map),
    ..._addedStoreFields,
    ..._addedAugments.values,
    ..._addedItems.values,
    ..._rawFields.values,
  ];

  /// The raw view of the open file, read once per open or save.
  ExpertDump? get _expertDump {
    final session = _session;
    if (session == null) return null;
    return _expert ??= expertOf(session: session);
  }

  /// Stages an edit of the Expert area; its field joins [_fields], so the
  /// change sheet names its raw path.
  void _editRaw(FieldValue field, int value) {
    _rawFields.putIfAbsent(field.path, () => field);
    _edit(field.path, value);
  }

  /// Stages [value] for [path]; the latest edit goes last, because the core
  /// applies the edits in order and a weapon or drone count depends on the
  /// slots and bars the earlier ones leave.
  void _edit(String path, int value) {
    // A field the file lacks (the level of a system a staged install adds)
    // has no file value: every value of it is an edit.
    final inFile = _fields.where((f) => f.path == path).firstOrNull?.value;
    _change(() {
      // Undoing a staged install also drops the level edit of that system.
      if (path.endsWith('.installed') && value == inFile) {
        _edits.remove('${path.substring(0, path.length - '.installed'.length)}.level');
      }
      for (final nearby in [false, true]) {
        final add = nearby ? _nearbyCrewAdd : _crewAdd;
        final added = _added(nearby: nearby);
        if (add?.path == path && value < added.length) _dropCrewFrom(nearby, added.first.index + value);
      }
      _edits.remove(path);
      if (value != inFile) _edits[path] = value;
    });
  }

  void _editText(String path, String text) {
    for (final i in [_identity, _nearbyIdentity].nonNulls) {
      final inFile = path == i.namePath
          ? i.name
          : path == i.typePath
          ? i.blueprintId
          : null;
      if (inFile != null) return _change(() => text == inFile ? _texts.remove(path) : _texts[path] = text);
    }
    final item = _mapBase?.beacons
        .expand((b) => b.store?.items ?? const <StoreItemView>[])
        .where((i) => i.itemPath == path)
        .firstOrNull;
    if (item != null) {
      return _change(() => text == item.id ? _texts.remove(path) : _texts[path] = text);
    }
    final c = [
      ..._crewOf(nearby: false),
      ..._crewOf(nearby: true),
    ].where((c) => c.namePath == path || c.racePath == path).firstOrNull;
    if (c == null) return;
    final inFile = path == c.namePath ? c.name : c.race;
    _change(() {
      text == inFile ? _texts.remove(path) : _texts[path] = text;
      // A new race gives new tints; a look staged for the old race goes.
      if (path == c.racePath) _edits.removeWhere((p, _) => _isLookOf(c, p));
    });
  }

  /// Stages a difficulty of the ship card; it is no field of [_fields], so
  /// the palette and the resource rows leave it out.
  void _editDifficulty(String path, int value) {
    final inFile = _identity?.difficulty.value;
    _change(() {
      _edits.remove(path);
      if (value != inFile) _edits[path] = value;
    });
  }

  /// The hangar ships as types of the player ship in [language].
  List<ShipTypeChoice> _shipTypes(String language) {
    final session = _session;
    if (session == null) return const [];
    final key = '$language|$_datOpen';
    if (_typesKey != key) {
      _types = shipTypeChoices(session: session, language: language);
      _typesKey = key;
    }
    return _types;
  }

  /// The edits of the ship card of the ship shown.
  ShipCardEdits _cardEdits(String language) => ShipCardEdits(
    identity: _nearby ? _nearbyIdentity : _identity,
    texts: _texts,
    edits: _edits,
    onText: _editText,
    onEdit: _editDifficulty,
    typeChoices: () => _shipTypes(language),
  );

  /// The staged edits as one value, for [_history].
  _Staged get _staged => (edits: Map.of(_edits), texts: Map.of(_texts));

  void _restoreStaged(_Staged s) {
    _edits
      ..clear()
      ..addAll(s.edits);
    _texts
      ..clear()
      ..addAll(s.texts);
  }

  /// Runs [change] of the staged edits; a change that alters them is one
  /// step of Undo.
  void _change(VoidCallback change) {
    if (_grouped) return setState(change);
    final before = _staged;
    setState(change);
    if (!mapEquals(before.edits, _edits) || !mapEquals(before.texts, _texts)) _history.record(before);
  }

  /// Within [_group]: the changes join one step of Undo.
  bool _grouped = false;

  /// Runs [changes] (a replace in a slot: removal, then addition) as one
  /// step of Undo.
  void _group(VoidCallback changes) {
    if (_grouped) return changes();
    final before = _staged;
    _grouped = true;
    try {
      changes();
    } finally {
      _grouped = false;
    }
    if (!mapEquals(before.edits, _edits) || !mapEquals(before.texts, _texts)) _history.record(before);
  }

  void _undo() {
    if (_history.undo(_staged) case final s?) setState(() => _restoreStaged(s));
  }

  void _redo() {
    if (_history.redo(_staged) case final s?) setState(() => _restoreStaged(s));
  }

  /// The crew texts for the pages, with the races in [language].
  CrewTexts _crewTexts(String language) {
    final key = '$language|$_datOpen';
    if (_racesKey != key) {
      _races = raceChoices(language: language);
      _racesKey = key;
    }
    final added = _added(nearby: _nearby);
    return CrewTexts(
      values: _texts,
      races: _races,
      onChanged: _editText,
      previews: _crewPreviews(),
      looksOf: (race) => _looks[race] ??= lookChoices(race: race),
      onLook: _editLook,
      newFrom: added.firstOrNull?.index,
      onRemoveNew: _removeNew,
    );
  }

  /// The crew of a ship: the file's, then the new members `crew.add` stages.
  List<CrewMember> _crewOf({required bool nearby}) => [...?(nearby ? _nearbyCrew : _crew), ..._added(nearby: nearby)];

  /// The new crew members of the staged `crew.add` of a ship, as the core
  /// builds them (`crewPreview` with that one edit): the last ones of its list.
  List<CrewMember> _added({required bool nearby}) {
    final session = _session;
    final add = nearby ? _nearbyCrewAdd : _crewAdd;
    final count = add == null ? 0 : _edits[add.path] ?? 0;
    if (session == null || add == null || count <= 0) return const [];
    return _addedCrew.putIfAbsent('$nearby|$count', () {
      try {
        final all = crewPreview(
          session: session,
          nearby: nearby,
          edits: [FieldEdit(path: add.path, value: count)],
          texts: const [],
        );
        return all.sublist(all.length - count);
      } catch (_) {
        return const [];
      }
    });
  }

  /// The path prefix of crew member [c]: `crew.<n>.`, with `nearby.` on the enemy ship.
  static String _crewBase(CrewMember c) => c.namePath.substring(0, c.namePath.length - 'name'.length);

  /// Whether [path] sets the look (sex, a tint) of crew member [c].
  static bool _isLookOf(CrewMember c, String path) {
    final base = _crewBase(c);
    return path == '${base}male' || path.startsWith('${base}tints.');
  }

  /// Stages [look] for [member]: its sex and a tint per layer. With a
  /// pending race every tint is staged, because the race gives new ones;
  /// the core applies the look after the race.
  void _editLook(CrewMember member, LookChoice look) {
    final base = _crewBase(member);
    final pendingRace = _texts[member.racePath];
    final newRace = pendingRace != null && pendingRace != member.race;
    _change(() {
      _edits.removeWhere((p, _) => _isLookOf(member, p));
      if ((look.male ? 1 : 0) != member.sex.value) _edits['${base}male'] = look.male ? 1 : 0;
      for (final (k, tint) in look.tints.indexed) {
        final inFile = k < member.tints.length ? member.tints[k].value : null;
        if (newRace || tint != inFile) _edits['${base}tints.$k'] = tint;
      }
    });
  }

  static final _crewPathPattern = RegExp(r'^((?:nearby\.)?crew\.)(\d+)\.(.+)$');

  /// Drops the new crew member [index] of the ship shown: the edits of the
  /// later new members move down by one, and `crew.add` drops by one.
  void _removeNew(int index) {
    final add = _nearby ? _nearbyCrewAdd : _crewAdd;
    final added = _added(nearby: _nearby);
    if (add == null || added.isEmpty) return;
    final prefix = _nearby ? 'nearby.crew.' : 'crew.';
    Map<String, V> shifted<V>(Map<String, V> map) {
      final out = <String, V>{};
      for (final e in map.entries) {
        final m = _crewPathPattern.firstMatch(e.key);
        final i = m == null || m.group(1) != prefix ? null : int.parse(m.group(2)!);
        if (i == null || i < index) {
          out[e.key] = e.value;
        } else if (i > index) {
          out['$prefix${i - 1}.${m!.group(3)}'] = e.value;
        }
      }
      return out;
    }

    _change(() {
      final edits = shifted(_edits);
      final texts = shifted(_texts);
      _edits
        ..clear()
        ..addAll(edits);
      _texts
        ..clear()
        ..addAll(texts);
      final count = (_edits[add.path] ?? 0) - 1;
      count <= 0 ? _edits.remove(add.path) : _edits[add.path] = count;
    });
  }

  /// Drops every edit and text of the crew members of a ship from index
  /// [from] on: the new members a lower `crew.add` no longer stages.
  void _dropCrewFrom(bool nearby, int from) {
    final prefix = nearby ? 'nearby.crew.' : 'crew.';
    bool gone(String path) {
      final m = _crewPathPattern.firstMatch(path);
      return m != null && m.group(1) == prefix && int.parse(m.group(2)!) >= from;
    }

    _edits.removeWhere((p, _) => gone(p));
    _texts.removeWhere((p, _) => gone(p));
  }

  /// The crew members of the ship shown with a pending race as the core
  /// would leave them, with every pending edit applied; empty when no race
  /// is pending or an edit is invalid (Save names that error).
  Map<int, CrewMember> _crewPreviews() {
    final session = _session;
    final crew = _nearby ? _nearbyCrew : _crew;
    if (session == null || crew == null) return const {};
    final prefix = _nearby ? 'nearby.crew.' : 'crew.';
    final pending = _texts.keys.any((p) => p.startsWith(prefix)) || _edits.keys.any((p) => p.startsWith(prefix));
    if (!pending) return const {};
    try {
      final preview = crewPreview(
        session: session,
        nearby: _nearby,
        edits: [for (final e in _edits.entries) FieldEdit(path: e.key, value: e.value)],
        texts: [for (final e in _texts.entries) TextEdit(path: e.key, text: e.value)],
      );
      return {for (final c in preview) c.index: c};
    } catch (_) {
      return const {};
    }
  }

  bool get _hasEdits => _edits.isNotEmpty || _texts.isNotEmpty;

  /// Icon and label of a field of either ship; a field of the enemy ship
  /// says so ("Enemy ship: Hull").
  (IconData, String) _look(String path, AppLocalizations l) {
    if (mapLook(path, l, _map) case final look?) return look;
    final nearby = isNearbyPath(path);
    final plan = nearby ? _nearbyFloorplan : _floorplan;
    final equipment = nearby ? _nearbyEquipment : _equipment;
    final crew = _crewOf(nearby: nearby);
    final look =
        expertLook(path, l) ??
        floorplanLook(path, l, plan, equipment) ??
        itemLook(path, l, equipment, _itemChoices) ??
        fieldLook(path, l, crew: crew, equipment: equipment, choices: _augmentChoices);
    return shipNamed(look, path, l);
  }

  /// The game image of a field of either ship ([fieldImage]) for the change
  /// sheet, the value popover and the palette.
  FieldImage? _image(String path) {
    final nearby = isNearbyPath(path);
    return fieldImage(
      path,
      crew: (nearby ? _nearbyCrew : _crew) ?? const [],
      equipment: nearby ? _nearbyEquipment : _equipment,
      choices: _itemChoices,
    );
  }

  /// Runs a one-click action: the core plans its edits on the file, they
  /// join the pending edits (a planned value replaces a pending one),
  /// and the change sheet shows them before anything is written.
  Future<void> _runAction(String id) async {
    final session = _session;
    if (session == null || _busy) return;
    final l = AppLocalizations.of(context)!;
    final PlannedAction plan;
    try {
      plan = planAction(session: session, action: id);
    } catch (e) {
      _snack(Icons.error, l.error(errorText(e, l)));
      return;
    }
    final inFile = {for (final f in _fields) f.path: f.value};
    _change(() {
      for (final e in plan.edits) {
        final old = inFile[e.path];
        if (old == null) continue;
        e.value == old ? _edits.remove(e.path) : _edits[e.path] = e.value;
      }
    });
    final notes = [for (final n in plan.notes) actionNoteText(n.code, n.count, l, detail: n.message)];
    final label = quickActions.firstWhere((a) => a.id == id, orElse: () => QuickAction(id, Icons.bolt)).label(l);
    if (plan.edits.isEmpty) {
      _snack(Icons.info_outline, [l.actionNothing(label), ...notes].join('\n'));
      return;
    }
    await _save(notes: notes);
  }

  /// Every field and action of the open file for the command palette.
  List<PaletteEntry> _paletteEntries(AppLocalizations l) => [
    if (_session != null)
      ...paletteFields(
        _fields.where((f) => !isQuietSquare(f) && !isCrewListPath(f.path)).toList(),
        _look,
        l,
        image: _image,
      ),
    if (_session != null) ...paletteActions(l),
  ];

  /// Enter in the palette: an action runs, a field opens its editor over
  /// the page the shell switched to.
  Future<void> _pick(PaletteEntry entry) async {
    if (entry.kind == PaletteKind.action) return _runAction(entry.id);
    final field = _fields.where((f) => f.path == entry.id).firstOrNull;
    if (field == null || !mounted) return;
    // The page shows the ship of the field.
    setState(() => _nearby = isNearbyPath(field.path));
    final l = AppLocalizations.of(context)!;
    final width = MediaQuery.sizeOf(context).width;
    final v = await showResourcePopover(
      _ui,
      anchor: Rect.fromLTWH(width / 2 - 140, 96, 280, 0),
      field: field,
      current: _edits[field.path] ?? field.value,
      look: _look(field.path, l),
      image: _image(field.path),
    );
    if (v != null) _edit(field.path, v);
  }

  Future<void> _save({List<String> notes = const []}) async {
    final l = AppLocalizations.of(context)!;
    final session = _session;
    final path = _path;
    if (session == null || path == null) return;

    final guard = await _readWarnings(path);
    if (!mounted) return;
    if (guard?.ftlRunning == true && !await confirmFtlRunning(_ui)) return;
    final summary = _summary;
    if (summary == null || !mounted) return;
    final write = await showChangeSheet(
      _ui,
      changes: [
        ...fieldChangeRows(
          pendingChanges(_fields, _edits).where((c) => !isCrewListPath(c.path)).toList(),
          l,
          look: _look,
          image: _image,
        ),
        ...crewListChangeRows(_crewOf(nearby: false), _crewAdd, _edits, l),
        ...identityChangeRows(
          player: _identity,
          nearby: _nearbyIdentity,
          texts: _texts,
          edits: _edits,
          choices: _shipTypes(Localizations.localeOf(context).languageCode),
          l: l,
        ),
        ...crewTextChangeRows(_crewOf(nearby: false), _crewTexts(Localizations.localeOf(context).languageCode), l),
        ...storeItemChangeRows(_mapBase, _texts, _storeChoices, l),
        ...sectorJumpChangeRows(_tree, _edits, l),
        for (final r in [
          ...crewListChangeRows(_crewOf(nearby: true), _nearbyCrewAdd, _edits, l),
          ...crewTextChangeRows(_crewOf(nearby: true), _crewTexts(Localizations.localeOf(context).languageCode), l),
        ])
          ChangeRow(
            id: r.id,
            icon: r.icon,
            image: r.image,
            title: shipNamed((r.icon, r.title), r.id, l).$2,
            subtitle: r.subtitle,
            change: r.change,
          ),
      ],
      onUndo: (p) => _change(() {
        for (final nearby in [false, true]) {
          final added = _added(nearby: nearby);
          if (p == (nearby ? _nearbyCrewAdd : _crewAdd)?.path && added.isNotEmpty) {
            _dropCrewFrom(nearby, added.first.index);
          }
        }
        _edits.remove(p);
        _texts.remove(p);
      }),
      steamCloud: guard?.steamCloud == true,
      notes: notes,
    );
    if (!write || !_hasEdits) return;

    setState(() => _busy = true);
    try {
      session.apply(
        edits: [for (final e in _edits.entries) FieldEdit(path: e.key, value: e.value)],
        texts: [for (final e in _texts.entries) TextEdit(path: e.key, text: e.value)],
      );
      final result = await session.save(backupDir: widget.backupDir);
      _show(session, session.summary());
      if (result.backup case final backup?) {
        setState(() => _lastBackup = LastBackup(path: backup, time: DateTime.now()));
      }
      _snack(Icons.check_circle, l.saved(result.backup ?? '-'));
    } catch (e) {
      // apply is all-or-nothing and save writes nothing on failure, so the
      // file on disk is unchanged. Reading it again drops the in-memory edit.
      _snack(Icons.error, l.error(errorText(e, l)));
      await _open(path);
    } finally {
      setState(() => _busy = false);
    }
  }

  Future<void> _showBackups() async {
    final path = _path;
    if (path == null) return;
    await showBackupHistory(
      _ui,
      fileName: File(path).uri.pathSegments.last,
      load: () => listBackups(savePath: path, backupDir: widget.backupDir),
      restore: _restore,
    );
  }

  /// Writes [backup] over the open file; answers whether it did.
  Future<bool> _restore(BackupInfo backup) async {
    final l = AppLocalizations.of(context)!;
    final session = _session;
    final path = _path;
    if (session == null || path == null) return false;
    final guard = await _readWarnings(path);
    if (!mounted) return false;
    if (guard?.ftlRunning == true && !await confirmFtlRunning(_ui)) return false;

    setState(() => _busy = true);
    try {
      final result = await restoreBackup(session: session, backup: backup.path, backupDir: widget.backupDir);
      _show(session, session.summary());
      if (result.backup case final b?) {
        setState(() => _lastBackup = LastBackup(path: b, time: DateTime.now()));
      }
      _snack(Icons.settings_backup_restore, l.restored(result.backup ?? '-'));
      return true;
    } catch (e) {
      _snack(Icons.error, l.error(errorText(e, l)));
      await _open(path);
      return false;
    } finally {
      setState(() => _busy = false);
    }
  }

  void _snack(IconData icon, String text) => showAppSnack(_ui, icon, text);

  /// The page below a [GameDataScope], so every [GameImage] knows whether
  /// `ftl.dat` is open, and below the theme of the look. The [Theme] stays in
  /// the tree in both looks, so a switch keeps the state of the pages.
  @override
  Widget build(BuildContext context) => Theme(
    data: _lookTheme ?? Theme.of(context),
    child: GameDataScope(
      datPath: _datOpen,
      child: Builder(key: _pageKey, builder: _page),
    ),
  );

  /// The [ShipSwitch] while the file has an enemy ship, else `null`. The
  /// Ship and Crew pages scroll it with their content.
  Widget? get _shipSwitch => _summary?.nearbyBlueprintId == null
      ? null
      : ShipSwitch(nearby: _nearby, onChanged: (v) => setState(() => _nearby = v));

  /// [page] keyed by the ship shown, so it does not carry hover or scroll
  /// of one ship over to the other.
  Widget _perShip(Widget page) => KeyedSubtree(key: ValueKey('ship-shown-$_nearby'), child: page);

  /// [page] below the [ShipSwitch] while the file has an enemy ship: the
  /// Equipment page, whose two panels scroll on their own.
  Widget _withShipSwitch(Widget page) => ShipSwitchFrame(
    hasNearby: _summary?.nearbyBlueprintId != null,
    nearby: _nearby,
    onChanged: (v) => setState(() => _nearby = v),
    child: page,
  );

  Widget _page(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final equipment = _equipmentIn(Localizations.localeOf(context).languageCode);
    // Read here like the equipment: its fields join the edits of every page.
    final map = _mapIn(Localizations.localeOf(context).languageCode);
    final floorplan = _nearby ? _nearbyFloorplan : _floorplan;
    final crew = _session == null ? null : _crewOf(nearby: _nearby);
    return AppShell(
      busy: _busy,
      onSave: _session == null || _busy || !_hasEdits ? null : _save,
      onSaveProfile: _profileEdits.save,
      onBackups: _session == null || _busy ? null : _showBackups,
      onChooseFile: _busy ? null : _choose,
      onChooseGameData: _datBusy ? null : _pickFtlDat,
      onUndo: _session == null || _busy || !_history.canUndo ? null : _undo,
      onRedo: _session == null || _busy || !_history.canRedo ? null : _redo,
      onUndoProfile: _profileEdits.canUndo ? _profileEdits.undoStep : null,
      onRedoProfile: _profileEdits.canRedo ? _profileEdits.redoStep : null,
      actions: QuickActionBar(onRun: _session == null || _busy ? null : _runAction),
      palette: PaletteSource(entries: _paletteEntries, onPick: _pick),
      status: ShellStatus.withGuard(
        path: _path,
        guard: _guard,
        lastBackup: _lastBackup,
        gameData: _datOpen,
        lookError: _lookError == null ? null : l.lookUnreadable('$_lookError'),
      ),
      banner: _datOpen == null && (_dat != null || _datError != null)
          ? GameDataCard(
              status: _dat,
              error: _datError == null ? null : errorText(_datError!, l),
              busy: _datBusy,
              onConfirm: _useFtlDat,
              onChoose: _pickFtlDat,
            )
          : null,
      map: Builder(
        builder: (context) {
          final language = Localizations.localeOf(context).languageCode;
          // The swap chooser offers the blueprints of the equipment and the races.
          _equipmentIn(language);
          _crewTexts(language);
          final (preview, previewError) = _mapPreview(language, l);
          return MapPage(
            map: map,
            preview: preview,
            previewError: previewError,
            edits: _edits,
            onChanged: (path, value) => path == sectorJumpPath ? _editJump(path, value) : _edit(path, value),
            tree: _tree,
            treePreview: _treePreview(language),
            treeBlock: _treeBlock,
            texts: _texts,
            onText: _editText,
            storeChoices: _storeChoices,
            previewOf: () => _mapPreview(language, l).$1,
          );
        },
      ),
      profile: ProfileView(
        edits: _profileEdits,
        backupDir: widget.backupDir,
        pickFile: widget.pickFile,
        defaultFile: widget.defaultProfileFile,
        warningsOf: widget.warningsOf,
      ),
      ship: _perShip(
        Builder(
          builder: (context) {
            final (preview, previewEquipment) = _shipPreview(Localizations.localeOf(context).languageCode);
            return ShipPage(
              path: _path,
              busy: _busy,
              onChoose: _choose,
              onReload: _path == null ? null : _reload,
              error: _error == null ? null : errorText(_error!, l),
              summary: _summary,
              edits: _edits,
              onChanged: _edit,
              floorplan: floorplan,
              preview: preview,
              previewEquipment: previewEquipment,
              crew: crew ?? const [],
              equipment: floorplan == null ? null : equipment,
              texts: _crewTexts(Localizations.localeOf(context).languageCode),
              nearby: _nearby,
              shipSwitch: _shipSwitch,
              card: _cardEdits(Localizations.localeOf(context).languageCode),
              itemChoices: _itemChoices,
              onAddItem: _addItem,
              group: _group,
            );
          },
        ),
      ),
      crew: _perShip(
        CrewPage(
          crew: crew,
          edits: _edits,
          onChanged: _edit,
          texts: _crewTexts(Localizations.localeOf(context).languageCode),
          add: _nearby ? _nearbyCrewAdd : _crewAdd,
          shipSwitch: _shipSwitch,
        ),
      ),
      // The equipment is read here, not in the Builder: its fields join the
      // edits of every page (an action and the change sheet need them) while
      // the shell builds only the page on screen. The preview is for this page.
      expert: (item) => ExpertPage(
        key: ValueKey('expert-${item.name}'),
        view: ExpertView.of(item),
        dump: _expertDump,
        edits: _edits,
        onChanged: _editRaw,
      ),
      equipment: _withShipSwitch(
        Builder(
          builder: (context) {
            final language = Localizations.localeOf(context).languageCode;
            final (preview, previewError) = _preview(language, l);
            return EquipmentPage(
              equipment: equipment,
              preview: preview,
              previewError: previewError,
              edits: _edits,
              onChanged: _edit,
              addedAugments: _addedAugments.values.toList(),
              choices: _augmentChoices,
              onAddAugment: _addAugment,
              itemChoices: _itemChoices,
              onAddItem: _addItem,
              group: _group,
            );
          },
        ),
      ),
    );
  }
}

/// Asks for `ftl.dat` while the page runs in reduced mode: the found path
/// with a confirm button, or the way to pick one.
class GameDataCard extends StatelessWidget {
  const GameDataCard({
    super.key,
    required this.status,
    required this.error,
    required this.busy,
    required this.onConfirm,
    required this.onChoose,
  });

  final FtlDatStatus? status;

  /// The localized error of the last attempt, if any.
  final String? error;
  final bool busy;
  final ValueChanged<String> onConfirm;
  final VoidCallback onChoose;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final found = status?.source == DatSource.found ? status?.path : null;
    final error = this.error;
    return Panel(
      icon: found != null ? Icons.image_search : Icons.image_not_supported,
      title: found != null ? l.ftlDatFound : l.ftlDatNotFound,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          AppText(found != null ? l.ftlDatFoundHint : l.ftlDatNotFoundHint, wrap: true),
          if (found != null) ...[
            const SizedBox(height: Space.xs),
            AppText(found, role: TextRole.caption, wrap: true, selectable: true),
          ],
          if (error != null) ...[
            const SizedBox(height: Space.sm),
            AppText(error, color: AppTokens.of(context).warn, wrap: true),
          ],
          const SizedBox(height: Space.md),
          Wrap(
            spacing: Space.sm,
            runSpacing: Space.sm,
            children: [
              if (found != null)
                AppButton(
                  kind: ButtonKind.primary,
                  icon: Icons.check,
                  label: l.ftlDatConfirm,
                  onPressed: busy ? null : () => onConfirm(found),
                ),
              AppButton(icon: Icons.file_open, label: l.ftlDatChoose, onPressed: busy ? null : onChoose),
            ],
          ),
        ],
      ),
    );
  }
}

/// The staged edits of the saved game: numbers and texts by path.
typedef _Staged = ({Map<String, int> edits, Map<String, String> texts});
