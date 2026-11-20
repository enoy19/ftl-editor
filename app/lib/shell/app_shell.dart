import 'package:flutter/services.dart';
import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../look/ftl_theme.dart';
import '../ui/command_palette.dart';
import '../ui/components/components.dart';
import '../ui/edit_history.dart';
import 'destination.dart';
import 'ftl_shell.dart';
import 'status_bar.dart';

/// The window of the editor: one top bar with the area tabs on the left and
/// Expert and Save on the right, the page of the chosen area, and the status
/// bar (decision ftl-editor-uses-top-tabs-and-portrait-crew-cards).
///
/// The shell holds no file state. Its owner passes the Ship, Crew, Equipment,
/// Map and Profile pages, the status and the save actions: Save writes the saved
/// game ([onSave]) and, on the Profile page, the profile ([onSaveProfile]);
/// `null` disables it.
/// [onBackups] opens the backup history from the status bar, [onChooseFile]
/// the saved-game chooser from its file name and [onChooseGameData] the
/// `ftl.dat` chooser from its game data item. [actions]
/// (the one-click actions) sit left of Expert. With [palette], Cmd+K
/// (Ctrl+K off macOS) opens the command palette over the areas and the
/// entries of the owner.
class AppShell extends StatefulWidget {
  const AppShell({
    super.key,
    required this.ship,
    required this.crew,
    required this.equipment,
    this.map,
    required this.profile,
    required this.status,
    this.onSave,
    this.onSaveProfile,
    this.onBackups,
    this.onChooseFile,
    this.onChooseGameData,
    this.onUndo,
    this.onRedo,
    this.onUndoProfile,
    this.onRedoProfile,
    this.busy = false,
    this.banner,
    this.actions,
    this.palette,
    this.expert,
  });

  /// The body of the Ship page.
  final Widget ship;

  /// The body of the Crew page.
  final Widget crew;

  /// The body of the Equipment page.
  final Widget equipment;

  /// The body of the Map page; `null` shows what a later step brings.
  final Widget? map;

  /// The body of the Profile page. It opens its own file; the status bar
  /// keeps showing the saved game.
  final Widget profile;
  final ShellStatus status;
  final VoidCallback? onSave;

  /// Save on the Profile page: the profile owns its own file and edits.
  final VoidCallback? onSaveProfile;
  final VoidCallback? onBackups;

  /// The saved-game chooser of the open button, from the file name of the status bar.
  final VoidCallback? onChooseFile;

  /// The `ftl.dat` chooser, from the game data item of the status bar.
  final VoidCallback? onChooseGameData;

  /// Undo and Redo of the staged edits of the saved game and of the profile.
  final VoidCallback? onUndo;
  final VoidCallback? onRedo;
  final VoidCallback? onUndoProfile;
  final VoidCallback? onRedoProfile;

  /// Shows a progress line under the top bar.
  final bool busy;

  /// A card above every page, such as the question for `ftl.dat`.
  final Widget? banner;

  /// The action buttons of the top bar, left of Expert.
  final Widget? actions;

  /// The fields and actions of the command palette; `null` turns it off.
  final PaletteSource? palette;

  /// The body of an item of the Expert menu (raw dump, state vars, unknown
  /// fields); `null` leaves those pages empty.
  final Widget Function(Destination item)? expert;

  @override
  State<AppShell> createState() => _AppShellState();
}

class _AppShellState extends State<AppShell> with SingleTickerProviderStateMixin {
  late final TabController _tabs = TabController(length: Destination.areas.length, vsync: this);
  Destination _dest = Destination.ship;

  @override
  void dispose() {
    _tabs.dispose();
    super.dispose();
  }

  void _go(Destination d) => setState(() {
    _dest = d;
    if (!d.expert) _tabs.index = Destination.areas.indexOf(d);
  });

  /// Opens the palette; Enter switches to the area of the entry, then
  /// hands a field or action to the owner.
  Future<void> _openPalette() async {
    final palette = widget.palette;
    if (palette == null) return;
    final l = AppLocalizations.of(context)!;
    final chosen = await showCommandPalette(context, [...paletteAreas(l), ...palette.entries(l)]);
    if (chosen == null || !mounted) return;
    if (chosen.area case final area?) _go(area);
    // After the frame that builds the new page, so the owner can open a
    // popover over it.
    if (chosen.kind != PaletteKind.area) WidgetsBinding.instance.addPostFrameCallback((_) => palette.onPick(chosen));
  }

  Widget _withPalette(Widget shell) => CallbackShortcuts(
    bindings: {paletteActivator(): _openPalette},
    child: Focus(autofocus: true, onKeyEvent: _undoKeys, child: shell),
  );

  /// Cmd+Z and Shift+Cmd+Z press Undo and Redo. A text field keeps them for
  /// its own text: the event goes on to the text shortcuts of the app.
  KeyEventResult _undoKeys(FocusNode node, KeyEvent event) {
    final keyboard = HardwareKeyboard.instance;
    final redo = redoActivator().accepts(event, keyboard);
    if (!redo && !undoActivator().accepts(event, keyboard)) return KeyEventResult.ignored;
    final focus = FocusManager.instance.primaryFocus?.context;
    if (focus != null && focus.findAncestorWidgetOfExactType<EditableText>() != null) return KeyEventResult.ignored;
    final action = redo ? _onRedo : _onUndo;
    if (action == null) return KeyEventResult.ignored;
    action();
    return KeyEventResult.handled;
  }

  /// Undo and Redo of the page on screen.
  VoidCallback? get _onUndo => _dest == Destination.profile ? widget.onUndoProfile : widget.onUndo;
  VoidCallback? get _onRedo => _dest == Destination.profile ? widget.onRedoProfile : widget.onRedo;

  /// Undo and Redo, a gap, then the one-click actions of the owner.
  Widget get _actions => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      UndoRedoBar(onUndo: _onUndo, onRedo: _onRedo),
      if (widget.actions case final actions?) ...[const SizedBox(width: Space.md), actions],
    ],
  );

  /// The page of the chosen area.
  Widget get _page => KeyedSubtree(
    key: ValueKey(_dest),
    child: switch (_dest) {
      Destination.ship => widget.ship,
      Destination.crew => widget.crew,
      Destination.equipment => widget.equipment,
      Destination.map => widget.map ?? _Later(destination: _dest),
      Destination.profile => widget.profile,
      final d when d.expert && widget.expert != null => widget.expert!(d),
      _ => _Later(destination: _dest),
    },
  );

  /// The save action of the page on screen.
  VoidCallback? get _onSave => _dest == Destination.profile ? widget.onSaveProfile : widget.onSave;

  @override
  Widget build(BuildContext context) {
    final statusBar = StatusBar(
      status: widget.status,
      onBackups: widget.onBackups,
      onChooseFile: widget.onChooseFile,
      onChooseGameData: widget.onChooseGameData,
    );
    if (FtlTheme.of(context) case final look?) {
      return _withPalette(
        FtlShell(
          look: look,
          dest: _dest,
          onDestination: _go,
          onSave: _onSave,
          busy: widget.busy,
          banner: widget.banner,
          page: _page,
          statusBar: statusBar,
          actions: _actions,
        ),
      );
    }
    final l = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    final scheme = theme.colorScheme;
    return _withPalette(
      Scaffold(
        appBar: AppBar(
          titleSpacing: 16,
          title: Row(
            children: [
              // The name stays in the window title and the tooltip: as text it
              // took the width the area tabs need at 1280 px, as in the FTL look.
              Tooltip(
                message: l.appTitle,
                child: Icon(Icons.rocket_launch, color: scheme.primary),
              ),
              const SizedBox(width: Space.xl),
              Flexible(
                child: TabBar(
                  key: const ValueKey('area-tabs'),
                  controller: _tabs,
                  isScrollable: true,
                  tabAlignment: TabAlignment.start,
                  dividerHeight: 0,
                  // An Expert item has no tab; no tab looks selected then.
                  indicator: _dest.expert ? const BoxDecoration() : null,
                  labelColor: _dest.expert ? scheme.onSurfaceVariant : null,
                  onTap: (i) => setState(() => _dest = Destination.areas[i]),
                  tabs: [
                    for (final d in Destination.areas)
                      Tab(
                        key: ValueKey('area-${d.name}'),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(_dest == d ? d.selectedIcon : d.icon, size: 20),
                            const SizedBox(width: Space.sm),
                            _TabLabel(d.label(l)),
                          ],
                        ),
                      ),
                  ],
                ),
              ),
            ],
          ),
          actions: [
            _actions,
            PopupMenuButton<Destination>(
              key: const ValueKey('expert-menu'),
              tooltip: l.areaExpert,
              onSelected: (d) => setState(() => _dest = d),
              itemBuilder: (_) => [
                for (final d in Destination.expertItems)
                  PopupMenuItem(
                    value: d,
                    child: ListTile(dense: true, leading: Icon(d.icon), title: AppText(d.label(l))),
                  ),
              ],
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: Space.md),
                child: Row(
                  children: [
                    Icon(expertIcon, color: _dest.expert ? scheme.primary : null),
                    const SizedBox(width: Space.xs),
                    // The chosen Expert item names the menu, since no tab marks it.
                    AppText(_dest.expert ? _dest.label(l) : l.areaExpert),
                    const Icon(Icons.arrow_drop_down),
                  ],
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.only(left: Space.sm, right: Space.lg),
              child: AppButton(
                key: const ValueKey('save-button'),
                kind: ButtonKind.primary,
                icon: Icons.save,
                label: l.save,
                onPressed: _onSave,
              ),
            ),
          ],
          bottom: PreferredSize(
            preferredSize: const Size.fromHeight(4),
            child: widget.busy
                ? const LinearProgressIndicator()
                : Divider(height: 4, thickness: 1, color: scheme.outlineVariant),
          ),
        ),
        body: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            if (widget.banner case final banner?)
              Padding(padding: const EdgeInsets.fromLTRB(Space.xl, Space.lg, Space.xl, 0), child: banner),
            Expanded(child: _page),
          ],
        ),
        // As the bottom bar of the Scaffold, a SnackBar opens above the status
        // bar instead of over it, so the backup item stays clickable.
        bottomNavigationBar: statusBar,
      ),
    );
  }
}

/// The Undo and Redo buttons of the top bar (keys `undo-button`,
/// `redo-button`); `null` disables one.
class UndoRedoBar extends StatelessWidget {
  const UndoRedoBar({super.key, required this.onUndo, required this.onRedo});

  final VoidCallback? onUndo;
  final VoidCallback? onRedo;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        AppButton(
          key: const ValueKey('undo-button'),
          kind: ButtonKind.text,
          icon: Icons.undo,
          tooltip: l.undo,
          onPressed: onUndo,
        ),
        const SizedBox(width: Space.xs),
        AppButton(
          key: const ValueKey('redo-button'),
          kind: ButtonKind.text,
          icon: Icons.redo,
          tooltip: l.redo,
          onPressed: onRedo,
        ),
      ],
    );
  }
}

/// A page that a later step builds: its icon and one sentence.
class _Later extends StatelessWidget {
  const _Later({required this.destination});

  final Destination destination;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final scheme = Theme.of(context).colorScheme;
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(destination.icon, size: 72, color: scheme.outlineVariant),
          const SizedBox(height: Space.md),
          AppText(destination.later(l), color: AppTokens.of(context).muted),
        ],
      ),
    );
  }
}

/// A tab label in the color the TabBar gives its selected and unselected tabs.
class _TabLabel extends StatelessWidget {
  const _TabLabel(this.text);

  final String text;

  @override
  Widget build(BuildContext context) => AppText(text, color: DefaultTextStyle.of(context).style.color);
}
