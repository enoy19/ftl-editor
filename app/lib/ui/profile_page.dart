import 'dart:io';

import 'package:flutter/foundation.dart' show mapEquals;
import 'package:material_ui/material_ui.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../l10n/app_localizations.dart';
import '../look/ftl_look.dart';
import '../look/ftl_theme.dart';
import '../src/rust/api/assets.dart';
import '../shell/status_bar.dart';
import '../src/rust/api/profile.dart';
import '../src/rust/api/status.dart';
import 'backup_history.dart';
import 'change_sheet.dart';
import 'components/components.dart';
import 'edit_history.dart';
import 'editor_page.dart';
import 'error_text.dart';
import 'game_image.dart';
import 'save_dialogs.dart';
import 'save_location.dart';
import 'ship_page.dart';

/// The staged edits of the profile, path to value (`ships.<ID>.<a|b|c>`,
/// `achievements.<ID>`; see `ProfileEdit`). The editor page owns them, so
/// they outlive a visit of the Profile tab like the edits of the saved game,
/// and asks [save] for the Save button while the Profile page is on screen.
class ProfileEdits extends ChangeNotifier {
  final Map<String, int> _values = {};
  VoidCallback? _save;

  /// The profile these edits belong to; another file drops them.
  String? path;

  Map<String, int> get values => Map.unmodifiable(_values);
  bool get isEmpty => _values.isEmpty;
  List<ProfileEdit> get list => [for (final e in _values.entries) ProfileEdit(path: e.key, value: e.value)];

  /// Save of the Profile page; `null` while nothing is staged. It calls the
  /// page on screen: the shell rebuilds the page on each visit.
  VoidCallback? get save => _values.isEmpty ? null : _runSave;

  void _runSave() => _save?.call();

  /// The steps of Undo and Redo; [clear] (a save, another file) drops them.
  final EditHistory<Map<String, int>> _history = EditHistory();

  /// Inside [group]: the changes record no steps of their own.
  bool _grouping = false;

  bool get canUndo => _history.canUndo;
  bool get canRedo => _history.canRedo;

  void _set(Map<String, int> values) {
    _values
      ..clear()
      ..addAll(values);
    notifyListeners();
  }

  void replace(Map<String, int> values) {
    if (mapEquals(values, _values)) return;
    if (!_grouping) _history.record(Map.of(_values));
    _set(values);
  }

  /// Runs [body] as one step of Undo, such as "Unlock all ships".
  void group(VoidCallback body) {
    final before = Map.of(_values);
    _grouping = true;
    try {
      body();
    } finally {
      _grouping = false;
    }
    if (!mapEquals(before, _values)) _history.record(before);
  }

  void undo(String path) {
    if (!_values.containsKey(path)) return;
    replace(Map.of(_values)..remove(path));
  }

  /// Takes back the last change; `false` without one.
  bool undoStep() {
    final before = _history.undo(Map.of(_values));
    if (before == null) return false;
    _set(before);
    return true;
  }

  /// Makes the last undone change again; `false` without one.
  bool redoStep() {
    final after = _history.redo(Map.of(_values));
    if (after == null) return false;
    _set(after);
    return true;
  }

  void clear() {
    final had = _history.canUndo || _history.canRedo;
    _history.clear();
    if (_values.isEmpty) {
      if (had) notifyListeners();
      return;
    }
    _values.clear();
    notifyListeners();
  }
}

/// The value of a profile edit path in [summary]: 0 or 1 for a layout, 0 or
/// the difficulty (1 Easy, 2 Normal, 3 Hard) for an achievement.
int profileValue(ProfileSummary summary, String path) {
  final parts = path.split('.');
  if (parts.first == 'ships' && parts.length == 3) {
    final ship = summary.ships.where((s) => s.id == parts[1]).firstOrNull;
    if (ship == null) return 0;
    final unlocked = switch (parts[2]) {
      'a' => ship.layoutA,
      'b' => ship.layoutB,
      _ => ship.layoutC,
    };
    return unlocked ? 1 : 0;
  }
  final id = parts.skip(1).join('.');
  final a = summary.achievements.where((a) => a.id == id).firstOrNull;
  return a == null ? 0 : difficultyValue(a.difficulty);
}

int difficultyValue(String d) => switch (d) {
  'easy' => 1,
  'normal' => 2,
  _ => 3,
};

String difficultyOf(int value) => switch (value) {
  1 => 'easy',
  2 => 'normal',
  _ => 'hard',
};

/// The Profile page: it opens the player profile (`ae_prof.sav`) on its own,
/// apart from the saved game of the Ship page. A click on a layout lock, an
/// achievement or "Unlock all ships" stages an edit in [edits]; the page
/// shows the staged profile with the pending look at once, and Save opens
/// the change sheet and writes the file with a backup. Its backup history
/// restores an earlier profile.
///
/// The page opens the last profile that opened without error, else FTL's
/// default file. The shell rebuilds a page on each visit, so the profile is
/// read fresh every time and the staged edits apply to it again. Every rule
/// (which layouts, which achievements, verify, backup) lives in the Rust core.
class ProfileView extends StatefulWidget {
  const ProfileView({
    super.key,
    this.edits,
    this.backupDir,
    this.pickFile = nativePickFile,
    this.defaultFile,
    this.warningsOf = coreWarnings,
  });

  /// The staged edits; `null` keeps them in the page alone.
  final ProfileEdits? edits;

  /// Backup folder for saves, restores and the history; `null` uses the
  /// editor's data folder.
  final String? backupDir;

  /// File chooser; tests replace the native dialog.
  final PickFile pickFile;

  /// FTL's profile; `null` asks the core for the file of this OS.
  final String? defaultFile;

  /// Warnings of the profile (running FTL, Steam Cloud); tests replace them.
  final WarningsOf warningsOf;

  /// The preference of the last profile that opened without error.
  static const lastFileKey = 'lastProfileFile';

  @override
  State<ProfileView> createState() => _ProfileViewState();
}

class _ProfileViewState extends State<ProfileView> {
  late final ProfileEdits _edits = widget.edits ?? ProfileEdits();
  String? _defaultFile;
  String? _path;
  ProfileSession? _session;

  /// The file as it is on disk.
  ProfileSummary? _file;

  /// The file with the staged edits; `null` while nothing is staged.
  ProfilePreview? _preview;
  List<AchievementInfo> _catalog = const [];
  Object? _error;
  bool _busy = false;
  String? _datPath;
  String? _language;

  @override
  void initState() {
    super.initState();
    // The shell reads it when the editor page rebuilds for a staged edit.
    _edits._save = _save;
    _edits.addListener(_refresh);
    _start();
  }

  @override
  void dispose() {
    _edits.removeListener(_refresh);
    if (_edits._save == _save) _edits._save = null;
    if (widget.edits == null) _edits.dispose();
    super.dispose();
  }

  /// Ship and achievement names come from `ftl.dat` in the app language:
  /// read them again when either changes.
  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final datPath = GameDataScope.maybeOf(context)?.datPath;
    final language = Localizations.localeOf(context).languageCode;
    if (datPath == _datPath && language == _language) return;
    _datPath = datPath;
    _language = language;
    _catalog = achievementCatalog(language: language);
    _refresh();
  }

  String get _lang => _language ?? 'en';

  /// Reads the file and the staged profile again. Edits that no longer
  /// apply (the file changed) are dropped with the error.
  void _refresh() {
    final session = _session;
    if (session == null || !mounted) return;
    ProfilePreview? preview;
    Object? error;
    if (!_edits.isEmpty) {
      try {
        preview = session.preview(edits: _edits.list, language: _lang);
      } catch (e) {
        error = e;
      }
    }
    setState(() {
      _file = session.summary(language: _lang);
      _preview = preview;
      if (error != null) _error = error;
    });
    // Not during a build: the editor page listens to the edits.
    if (error != null) WidgetsBinding.instance.addPostFrameCallback((_) => _edits.clear());
  }

  Future<void> _start() async {
    try {
      _defaultFile = widget.defaultFile ?? defaultProfileFile();
    } catch (e) {
      setState(() => _error = e);
    }
    final prefs = await SharedPreferences.getInstance();
    final start =
        _edits.path ??
        SaveLocation.startFile(lastFile: prefs.getString(ProfileView.lastFileKey), defaultFile: _defaultFile ?? '');
    if (start.isNotEmpty) await _open(start);
  }

  Future<void> _choose() async {
    final dir = SaveLocation.dialogDirectory(currentPath: _path ?? '', defaultFile: _defaultFile);
    final path = await widget.pickFile(context, dir);
    if (path != null) await _open(path);
  }

  Future<void> _open(String path) async {
    if (!mounted) return;
    setState(() {
      _busy = true;
      _path = path;
      _error = null;
    });
    try {
      final session = await openProfile(path: path);
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(ProfileView.lastFileKey, path);
      if (!mounted) return;
      _session = session;
      if (_edits.path != path) {
        _edits.path = path;
        _edits.clear();
      }
      _refresh();
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _session = null;
        _file = null;
        _preview = null;
        _error = e;
      });
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  /// Stages [path] = [value]. An edit that the profile already shows
  /// without it is dropped; it goes last, after the edits it depends on
  /// (a Type-B edit adds or removes achievements).
  void _stage(String path, int value) {
    final session = _session;
    if (session == null) return;
    final next = Map.of(_edits.values)..remove(path);
    try {
      final without = session.preview(
        edits: [for (final e in next.entries) ProfileEdit(path: e.key, value: e.value)],
        language: _lang,
      );
      if (profileValue(without.summary, path) != value) next[path] = value;
      _edits.replace(next);
    } catch (e) {
      final l = AppLocalizations.of(context)!;
      _snack(Icons.error, l.error(errorText(e, l)));
    }
  }

  /// Stages the edits that unlock every layout still locked.
  void _unlockAll() {
    final l = AppLocalizations.of(context)!;
    final session = _session;
    if (session == null) return;
    final List<ProfileEdit> edits;
    try {
      edits = session.unlockAllEdits(edits: _edits.list);
    } catch (e) {
      _snack(Icons.error, l.error(errorText(e, l)));
      return;
    }
    if (edits.isEmpty) {
      _snack(Icons.check_circle, l.profileAllUnlocked);
      return;
    }
    _edits.group(() {
      for (final e in edits) {
        _stage(e.path, e.value);
      }
    });
  }

  /// Asks before a write while FTL runs; answers the Steam Cloud state
  /// for the change sheet, or `null` to stop.
  Future<SaveGuard?> _guard(String path) async {
    SaveGuard guard;
    try {
      guard = SaveGuard.of(await widget.warningsOf(path));
    } catch (_) {
      // The core answers a failed check as a warning; this is a bridge fault.
      guard = const SaveGuard();
    }
    if (!mounted) return null;
    if (guard.ftlRunning == true && !await confirmFtlRunning(context)) return null;
    return guard;
  }

  /// Shows the staged edits in the change sheet and writes them on Write.
  /// A failed save reads the file again and drops the edits.
  Future<void> _save() async {
    if (!mounted) return;
    final l = AppLocalizations.of(context)!;
    final session = _session;
    final path = _path;
    final preview = _preview;
    if (session == null || path == null || preview == null || _busy) return;
    final guard = await _guard(path);
    if (guard == null || !mounted) return;
    final write = await showChangeSheet(
      context,
      changes: profileChangeRows(preview.changes, l, ships: preview.summary.ships, catalog: _catalog),
      onUndo: _edits.undo,
      steamCloud: guard.steamCloud == true,
    );
    if (!write || _edits.isEmpty || !mounted) return;
    setState(() => _busy = true);
    try {
      final result = await session.save(edits: _edits.list, backupDir: widget.backupDir);
      if (!mounted) return;
      _edits.clear();
      _refresh();
      _snack(Icons.check_circle, l.saved(result.backup ?? '-'));
    } catch (e) {
      if (!mounted) return;
      _snack(Icons.error, l.error(errorText(e, l)));
      _edits.clear();
      await _open(path);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _showBackups() async {
    final path = _path;
    if (path == null) return;
    await showBackupHistory(
      context,
      fileName: File(path).uri.pathSegments.last,
      load: () => listBackups(savePath: path, backupDir: widget.backupDir),
      restore: _restore,
    );
  }

  /// Writes [backup] over the profile; answers whether it did. The staged
  /// edits are dropped: they were made on the profile it replaces.
  Future<bool> _restore(BackupInfo backup) async {
    final l = AppLocalizations.of(context)!;
    final session = _session;
    final path = _path;
    if (session == null || path == null) return false;
    if (await _guard(path) == null || !mounted) return false;
    setState(() => _busy = true);
    try {
      final result = await session.restore(backup: backup.path, backupDir: widget.backupDir);
      if (!mounted) return true;
      _edits.clear();
      _refresh();
      _snack(Icons.settings_backup_restore, l.restored(result.backup ?? '-'));
      return true;
    } catch (e) {
      if (!mounted) return false;
      _snack(Icons.error, l.error(errorText(e, l)));
      await _open(path);
      return false;
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _snack(IconData icon, String text) => showAppSnack(context, icon, text);

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final idle = _session != null && !_busy;
    return ProfileBody(
      path: _path,
      busy: _busy,
      onChoose: _choose,
      error: _error == null ? null : errorText(_error!, l),
      summary: _preview?.summary ?? _file,
      file: _file,
      catalog: _catalog,
      onUnlockAll: idle ? _unlockAll : null,
      onBackups: idle ? _showBackups : null,
      onLayout: idle ? (ship, layout, unlock) => _stage('ships.$ship.$layout', unlock ? 1 : 0) : null,
      onAchievement: idle ? (id, value) => _stage('achievements.$id', value) : null,
    );
  }
}

/// Shows a dialog that draws game images: the dialog route lies above the
/// page, outside its [GameDataScope], so the scope comes along.
Future<T?> _showGameDialog<T>(BuildContext context, Widget dialog) {
  final scope = GameDataScope.maybeOf(context);
  return showDialog<T>(
    context: context,
    builder: (_) => scope == null ? dialog : GameDataScope(datPath: scope.datPath, load: scope.load, child: dialog),
  );
}

/// The name of achievement [id] in [catalog], else the id.
String achievementName(String id, List<AchievementInfo> catalog) =>
    catalog.where((a) => a.id == id).firstOrNull?.name ?? id;

/// The rows of the change sheet for staged profile edits: one per edit with
/// a visible effect. A layout row names the ship; a Type-B row the
/// achievements it adds or removes. The id is the edit path, for undo.
List<ChangeRow> profileChangeRows(
  List<ProfileEditChange> changes,
  AppLocalizations l, {
  required List<ProfileShip> ships,
  required List<AchievementInfo> catalog,
}) {
  String names(List<String> ids) => ids.map((id) => achievementName(id, catalog)).join(', ');
  return [
    for (final c in changes)
      if (c.old != c.value || c.addedAchievements.isNotEmpty || c.removedAchievements.isNotEmpty)
        if (c.path.split('.') case ['ships', final id, final layout])
          ChangeRow(
            id: c.path,
            icon: c.value == 1 ? Icons.lock_open : Icons.lock,
            title: (c.value == 1 ? l.profileUnlockLayout : l.profileLockLayout)(
              ships.where((s) => s.id == id).firstOrNull?.name ?? id,
              layout.toUpperCase(),
            ),
            subtitle: c.addedAchievements.isNotEmpty
                ? l.profileUnlockAdds(names(c.addedAchievements))
                : c.removedAchievements.isNotEmpty
                ? l.profileLockRemoves(names(c.removedAchievements))
                : null,
          )
        else
          _achievementRow(c, l, catalog),
  ];
}

ChangeRow _achievementRow(ProfileEditChange c, AppLocalizations l, List<AchievementInfo> catalog) {
  final id = c.path.substring('achievements.'.length);
  final info = catalog.where((a) => a.id == id).firstOrNull;
  final name = info?.name ?? id;
  final title = c.value == 0
      ? l.profileAchievementRemove(name)
      : c.old == 0
      ? l.profileAchievementAdd(name, difficultyText(l, difficultyOf(c.value)))
      : l.profileAchievementSet(name, difficultyText(l, difficultyOf(c.value)));
  return ChangeRow(
    id: c.path,
    icon: c.value == 0 ? Icons.remove_circle_outline : Icons.emoji_events,
    title: title,
    subtitle: info?.description,
  );
}

/// The body of the Profile page. Pure: [ProfileView] owns the file and the
/// edits. [summary] is the profile as shown (with the staged edits), [file]
/// the profile on disk; where they differ, the page shows the pending look.
class ProfileBody extends StatelessWidget {
  const ProfileBody({
    super.key,
    required this.path,
    required this.busy,
    required this.onChoose,
    required this.error,
    required this.summary,
    required this.onUnlockAll,
    this.file,
    this.catalog = const [],
    this.onBackups,
    this.onLayout,
    this.onAchievement,
  });

  final String? path;
  final bool busy;
  final VoidCallback onChoose;

  /// The localized text of the last error, or `null`.
  final String? error;
  final ProfileSummary? summary;

  /// The profile on disk; `null` shows nothing as pending.
  final ProfileSummary? file;

  /// Every achievement of the game with its name; empty in reduced mode.
  final List<AchievementInfo> catalog;

  /// `null` disables "Unlock all ships".
  final VoidCallback? onUnlockAll;

  /// Opens the backup history of the profile; `null` disables it.
  final VoidCallback? onBackups;

  /// A click on a layout lock: the ship, the layout (`a`, `b`, `c`) and
  /// whether to unlock it. `null` makes the locks read-only.
  final void Function(String ship, String layout, bool unlock)? onLayout;

  /// An achievement set to 0 (remove), 1 Easy, 2 Normal or 3 Hard. `null`
  /// makes the achievements read-only.
  final void Function(String id, int value)? onAchievement;

  Future<void> _addAchievement(BuildContext context, ProfileSummary summary) async {
    final present = {for (final a in summary.achievements) a.id};
    final id = await _showGameDialog<String>(
      context,
      _AchievementPicker(
        choices: [
          for (final a in catalog)
            if (!present.contains(a.id)) a,
        ],
      ),
    );
    if (id != null) onAchievement?.call(id, 1);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final accent = AppTokens.of(context).accent;
    final summary = this.summary;
    final error = this.error;
    final file = this.file ?? summary;
    final onAchievement = this.onAchievement;
    // Achievements the staged edits remove stay in the grid, marked.
    final removed = [
      if (summary != null && file != null)
        for (final a in file.achievements)
          if (!summary.achievements.any((s) => s.id == a.id)) a,
    ];
    return ListView(
      padding: const EdgeInsets.all(Space.xl),
      children: [
        _ProfileFileCard(path: path, busy: busy, onChoose: onChoose, onBackups: onBackups),
        if (error != null) ...[const SizedBox(height: Space.lg), ErrorCard(text: error)],
        if (summary != null && file != null) ...[
          const SizedBox(height: Space.xl),
          TitleRow(
            icon: Icons.rocket_launch,
            iconColor: accent,
            title: l.profileShips,
            trailing: AppButton(
              key: const ValueKey('profile-unlock-all'),
              kind: ButtonKind.primary,
              icon: Icons.lock_open,
              label: l.profileUnlockAll,
              onPressed: onUnlockAll,
            ),
          ),
          const SizedBox(height: Space.md),
          Wrap(
            spacing: Space.md,
            runSpacing: Space.md,
            children: [
              for (final s in summary.ships)
                ProfileShipTile(
                  ship: s,
                  inFile: file.ships.where((f) => f.id == s.id).firstOrNull ?? s,
                  catalog: catalog,
                  onLayout: onLayout,
                ),
            ],
          ),
          const SizedBox(height: Space.sm),
          HintRow(text: l.profileLayoutBRule),
          const SizedBox(height: Space.xs),
          HintRow(text: l.profileCloseFtl),
          const SizedBox(height: Space.xl),
          TitleRow(
            icon: Icons.emoji_events,
            iconColor: accent,
            title: l.profileAchievements(summary.achievements.length),
            trailing: AppButton(
              key: const ValueKey('profile-add-achievement'),
              kind: ButtonKind.tonal,
              icon: Icons.add,
              label: l.profileAddAchievement,
              onPressed: onAchievement == null || catalog.isEmpty ? null : () => _addAchievement(context, summary),
            ),
          ),
          const SizedBox(height: Space.md),
          Wrap(
            spacing: Space.sm,
            runSpacing: Space.sm,
            children: [
              for (final a in summary.achievements)
                _AchievementTile(
                  achievement: a,
                  info: catalog.where((c) => c.id == a.id).firstOrNull,
                  pending: !file.achievements.any((f) => f.id == a.id && f.difficulty == a.difficulty),
                  onEdit: onAchievement,
                ),
              for (final a in removed)
                _AchievementTile(
                  achievement: a,
                  info: catalog.where((c) => c.id == a.id).firstOrNull,
                  pending: true,
                  removed: true,
                  onEdit: onAchievement,
                ),
            ],
          ),
          const SizedBox(height: Space.xl),
          Wrap(
            spacing: Space.lg,
            runSpacing: Space.lg,
            children: [
              _StatsCard(stats: summary.stats),
              _ScoresCard(scores: summary.topScores),
            ],
          ),
        ],
      ],
    );
  }
}

class _ProfileFileCard extends StatelessWidget {
  const _ProfileFileCard({required this.path, required this.busy, required this.onChoose, required this.onBackups});

  final String? path;
  final bool busy;
  final VoidCallback onChoose;
  final VoidCallback? onBackups;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final path = this.path;
    final file = path == null ? null : File(path);
    return FilePanel(
      icon: Icons.account_circle_outlined,
      name: file == null ? l.profileNoFile : file.uri.pathSegments.last,
      folder: file?.parent.path,
      actions: [
        AppButton(
          key: const ValueKey('profile-backups'),
          icon: Icons.history,
          label: l.profileBackups,
          onPressed: busy ? null : onBackups,
        ),
        AppButton(
          key: const ValueKey('profile-choose'),
          kind: ButtonKind.tonal,
          icon: Icons.file_open,
          label: l.profileChoose,
          onPressed: busy ? null : onChoose,
        ),
      ],
    );
  }
}

/// One player ship: its hull and the locks of layouts A, B and C. A click
/// on a lock stages its unlock or lock; a lock that differs from [inFile]
/// has the pending look.
class ProfileShipTile extends StatelessWidget {
  const ProfileShipTile({super.key, required this.ship, ProfileShip? inFile, this.catalog = const [], this.onLayout})
    : inFile = inFile ?? ship;

  final ProfileShip ship;
  final ProfileShip inFile;
  final List<AchievementInfo> catalog;
  final void Function(String ship, String layout, bool unlock)? onLayout;

  @override
  Widget build(BuildContext context) {
    final name = ship.name ?? ship.id;
    final onLayout = this.onLayout;
    Widget badge(String layout, bool unlocked, bool inFile, bool exists) => _LayoutBadge(
      key: ValueKey('profile-layout-${ship.id}-$layout'),
      layout: layout.toUpperCase(),
      unlocked: unlocked,
      pending: unlocked != inFile,
      exists: exists,
      typeB: layout == 'b' ? ship.achievements.map((id) => achievementName(id, catalog)).join(', ') : null,
      onTap: !exists || onLayout == null ? null : () => onLayout(ship.id, layout, !unlocked),
    );
    final body = Column(
      children: [
        GameImage(
          GameImageRef.of(GameImageKind.ship, ship.id),
          fallback: Icons.rocket_launch,
          width: 196,
          height: 110,
          iconSize: 48,
        ),
        const SizedBox(height: Space.sm),
        AppText(name, role: TextRole.label),
        const SizedBox(height: Space.sm),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            badge('a', ship.layoutA, inFile.layoutA, true),
            badge('b', ship.layoutB, inFile.layoutB, true),
            badge('c', ship.layoutC, inFile.layoutC, ship.hasLayoutC),
          ],
        ),
      ],
    );
    return SizedBox(
      key: ValueKey('profile-ship-${ship.id}'),
      width: 220,
      child: Panel(tile: true, child: body),
    );
  }
}

class _LayoutBadge extends StatelessWidget {
  const _LayoutBadge({
    super.key,
    required this.layout,
    required this.unlocked,
    required this.pending,
    required this.exists,
    this.typeB,
    this.onTap,
  });

  final String layout;
  final bool unlocked;
  final bool pending;
  final bool exists;

  /// For Type-B: the names of the ship's three achievements.
  final String? typeB;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final scheme = Theme.of(context).colorScheme;
    final tokens = AppTokens.of(context);
    final state = unlocked ? l.profileLayoutUnlocked(layout) : l.profileLayoutLocked(layout);
    final (icon, color) = !exists
        ? (Icons.block, scheme.outlineVariant)
        : (
            unlocked ? Icons.lock_open : Icons.lock,
            pending ? tokens.accent : (unlocked ? scheme.primary : scheme.outline),
          );
    final typeB = this.typeB;
    final onTap = this.onTap;
    final tip = !exists
        ? l.profileLayoutNone
        : [
            onTap == null ? state : l.profileLayoutClick(state),
            if (typeB != null) '${l.profileLayoutBRule}\n$typeB',
          ].join('\n');
    // A staged change is framed in the accent color: in the Material look
    // the accent is the color of an unlocked layout too.
    final badge = DecoratedBox(
      key: pending ? const ValueKey('pending') : null,
      decoration: pending
          ? ShapeDecoration(
              shape: StadiumBorder(side: BorderSide(color: tokens.accent, width: 2)),
            )
          : const BoxDecoration(),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: Space.sm, vertical: Space.xs),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 20, color: color),
            const SizedBox(width: Space.xs),
            AppText(layout, role: TextRole.label, color: color),
          ],
        ),
      ),
    );
    return Tooltip(
      message: tip,
      child: onTap == null
          ? badge
          : MouseRegion(
              cursor: SystemMouseCursors.click,
              child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: onTap, child: badge),
            ),
    );
  }
}

String difficultyText(AppLocalizations l, String d) => switch (d) {
  'easy' => l.difficultyEasy,
  'hard' => l.difficultyHard,
  _ => l.difficultyNormal,
};

/// One achievement: its image and difficulty; name and description in the
/// tooltip. A click opens the editor of the achievement.
class _AchievementTile extends StatelessWidget {
  const _AchievementTile({
    required this.achievement,
    required this.info,
    required this.pending,
    this.removed = false,
    this.onEdit,
  });

  final ProfileAchievement achievement;
  final AchievementInfo? info;

  /// Added or changed by a staged edit: drawn in the accent color.
  final bool pending;

  /// Removed by a staged edit: faded, with "Removed" as caption.
  final bool removed;
  final void Function(String id, int value)? onEdit;

  Future<void> _edit(BuildContext context) async {
    final value = await _showGameDialog<int>(
      context,
      _AchievementDialog(id: achievement.id, info: info, value: removed ? 0 : difficultyValue(achievement.difficulty)),
    );
    if (value != null) onEdit?.call(achievement.id, value);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final scheme = Theme.of(context).colorScheme;
    final tokens = AppTokens.of(context);
    // Only the word: the one-bar signal icon read as a stray comma.
    final color = pending
        ? tokens.accent
        : switch (achievement.difficulty) {
            'easy' => scheme.tertiary,
            'hard' => scheme.error,
            _ => scheme.primary,
          };
    final look = FtlTheme.of(context);
    final info = this.info;
    final body = Column(
      children: [
        Opacity(
          opacity: removed ? 0.35 : 1,
          child: GameImage(
            GameImageRef.of(GameImageKind.achievement, achievement.id),
            fallback: Icons.emoji_events,
            width: 64,
            height: 64,
            iconSize: 40,
          ),
        ),
        const SizedBox(height: Space.xs),
        AppText(
          removed ? l.profileAchievementRemoved : difficultyText(l, achievement.difficulty),
          role: TextRole.caption,
          color: color,
        ),
      ],
    );
    final onEdit = this.onEdit;
    final tile = SizedBox(
      key: ValueKey('profile-achievement-${achievement.id}'),
      width: 96,
      // A window frame needs 22 px of the 96; the small FTL box keeps the image whole.
      child: look != null
          ? FtlBox(
              outline: pending ? FtlColors.hover : FtlColors.line,
              padding: const EdgeInsets.all(Space.sm),
              child: body,
            )
          : Card(
              margin: EdgeInsets.zero,
              shape: pending
                  ? RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(Space.md),
                      side: BorderSide(color: tokens.accent, width: 2),
                    )
                  : null,
              child: Padding(padding: const EdgeInsets.all(Space.sm), child: body),
            ),
    );
    return Tooltip(
      message: info == null ? achievement.id : '${info.name}\n${info.description}',
      child: onEdit == null
          ? tile
          : MouseRegion(
              cursor: SystemMouseCursors.click,
              child: GestureDetector(onTap: () => _edit(context), child: tile),
            ),
    );
  }
}

/// The editor of one achievement: image, name and description, the three
/// difficulties to choose from and Remove. Answers 0 (remove), 1 Easy,
/// 2 Normal or 3 Hard; [value] 0 is an achievement the profile lacks.
class _AchievementDialog extends StatelessWidget {
  const _AchievementDialog({required this.id, required this.info, required this.value});

  final String id;
  final AchievementInfo? info;
  final int value;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final info = this.info;
    return AppDialog(
      icon: Icons.emoji_events,
      title: info?.name ?? id,
      content: SizedBox(
        width: 420,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                GameImage(
                  GameImageRef.of(GameImageKind.achievement, id),
                  fallback: Icons.emoji_events,
                  width: 64,
                  height: 64,
                  iconSize: 40,
                ),
                const SizedBox(width: Space.md),
                Expanded(child: AppText(info?.description ?? id, wrap: true)),
              ],
            ),
            const SizedBox(height: Space.lg),
            AppText(l.profileAchievementDifficulty, role: TextRole.label),
            const SizedBox(height: Space.sm),
            Wrap(
              spacing: Space.sm,
              children: [
                for (final v in const [1, 2, 3])
                  Tag(
                    key: ValueKey('achievement-difficulty-${difficultyOf(v)}'),
                    label: difficultyText(l, difficultyOf(v)),
                    marked: v == value,
                    onTap: () => Navigator.of(context).pop(v),
                  ),
              ],
            ),
          ],
        ),
      ),
      actions: [
        if (value != 0)
          AppButton(
            key: const ValueKey('achievement-remove'),
            kind: ButtonKind.text,
            icon: Icons.remove_circle_outline,
            label: l.profileAchievementRemoveButton,
            onPressed: () => Navigator.of(context).pop(0),
          ),
        AppButton(kind: ButtonKind.text, label: l.cancel, onPressed: () => Navigator.of(context).pop()),
      ],
    );
  }
}

/// The picker of Add achievement: every achievement the profile lacks, with
/// its image, name and description and a filter field. Answers the chosen id.
class _AchievementPicker extends StatefulWidget {
  const _AchievementPicker({required this.choices});

  final List<AchievementInfo> choices;

  @override
  State<_AchievementPicker> createState() => _AchievementPickerState();
}

class _AchievementPickerState extends State<_AchievementPicker> {
  String _filter = '';

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final words = _filter.toLowerCase();
    final shown = [
      for (final c in widget.choices)
        if (words.isEmpty || c.name.toLowerCase().contains(words) || c.description.toLowerCase().contains(words)) c,
    ];
    return AppDialog(
      icon: Icons.emoji_events,
      title: l.profileAddAchievement,
      content: SizedBox(
        width: 480,
        height: 420,
        child: Column(
          children: [
            AppTextField(
              key: const ValueKey('achievement-filter'),
              autofocus: true,
              search: true,
              hint: l.equipFilter,
              onChanged: (v) => setState(() => _filter = v),
            ),
            const SizedBox(height: Space.sm),
            Expanded(
              child: ListView(
                children: [
                  for (final c in shown)
                    ListRow(
                      key: ValueKey('achievement-choice-${c.id}'),
                      leading: GameImage(
                        GameImageRef.of(GameImageKind.achievement, c.id),
                        fallback: Icons.emoji_events,
                        width: 32,
                        height: 32,
                        iconSize: 24,
                      ),
                      title: c.name,
                      subtitle: c.description,
                      onTap: () => Navigator.of(context).pop(c.id),
                    ),
                ],
              ),
            ),
          ],
        ),
      ),
      actions: [AppButton(kind: ButtonKind.text, label: l.cancel, onPressed: () => Navigator.of(context).pop())],
    );
  }
}

class _StatsCard extends StatelessWidget {
  const _StatsCard({required this.stats});

  final ProfileStats stats;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final s = stats;
    final rows = <(IconData, String, int?, int)>[
      (Icons.gps_fixed, l.statShipsDefeated, s.mostShipsDefeated, s.totalShipsDefeated),
      (Icons.radar, l.statBeaconsExplored, s.mostBeaconsExplored, s.totalBeaconsExplored),
      (Icons.recycling, l.statScrapCollected, s.mostScrapCollected, s.totalScrapCollected),
      (Icons.person_add, l.statCrewHired, s.mostCrewHired, s.totalCrewHired),
      (Icons.sports_esports, l.statGamesPlayed, null, s.totalGamesPlayed),
      (Icons.emoji_events, l.statVictories, null, s.totalVictories),
    ];
    Widget right(Widget child) => Align(alignment: Alignment.centerRight, child: child);
    const cell = EdgeInsets.symmetric(vertical: Space.xs);
    return _TitledCard(
      icon: Icons.query_stats,
      title: l.profileStats,
      children: [
        Table(
          columnWidths: const {
            0: FixedColumnWidth(32),
            1: FlexColumnWidth(),
            2: FixedColumnWidth(72),
            3: FixedColumnWidth(72),
          },
          defaultVerticalAlignment: TableCellVerticalAlignment.middle,
          children: [
            TableRow(
              children: [
                const SizedBox(),
                const SizedBox(),
                right(AppText(l.statBest, role: TextRole.caption)),
                right(AppText(l.statTotal, role: TextRole.caption)),
              ],
            ),
            for (final (icon, label, best, total) in rows)
              TableRow(
                children: [
                  Padding(padding: cell, child: Icon(icon, size: 18)),
                  Padding(padding: cell, child: AppText(label)),
                  Padding(
                    padding: cell,
                    child: right(AppText(best == null ? '' : '$best', role: TextRole.number)),
                  ),
                  Padding(
                    padding: cell,
                    child: right(AppText('$total', role: TextRole.number)),
                  ),
                ],
              ),
          ],
        ),
      ],
    );
  }
}

class _ScoresCard extends StatelessWidget {
  const _ScoresCard({required this.scores});

  final List<ProfileScore> scores;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final tokens = AppTokens.of(context);
    return _TitledCard(
      icon: Icons.leaderboard,
      title: l.profileTopScores,
      children: [
        if (scores.isEmpty) AppText(l.profileNoScores, color: tokens.muted),
        for (final s in scores)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: Space.xs),
            child: Row(
              children: [
                GameImage(
                  GameImageRef.of(GameImageKind.ship, s.shipId),
                  fallback: Icons.rocket_launch,
                  width: 40,
                  height: 24,
                  iconSize: 18,
                ),
                const SizedBox(width: Space.sm),
                SizedBox(
                  width: 56,
                  child: Align(
                    alignment: Alignment.centerRight,
                    child: AppText('${s.value}', role: TextRole.number),
                  ),
                ),
                const SizedBox(width: Space.md),
                Expanded(child: AppText('${s.shipName} · ${l.sector(s.sector)} · ${difficultyText(l, s.difficulty)}')),
                if (s.victory)
                  Tooltip(
                    message: l.profileVictory,
                    child: Icon(Icons.military_tech, color: tokens.accent),
                  ),
              ],
            ),
          ),
      ],
    );
  }
}

/// A card 420 wide with an icon and a title above [children]: the stats and
/// the top scores of the profile.
class _TitledCard extends StatelessWidget {
  const _TitledCard({required this.icon, required this.title, required this.children});

  final IconData icon;
  final String title;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) => SizedBox(
    width: 420,
    child: Panel(
      icon: icon,
      title: title,
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: children),
    ),
  );
}
