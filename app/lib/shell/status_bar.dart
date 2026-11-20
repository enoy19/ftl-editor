import 'dart:io';

import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../look/ftl_look.dart';
import '../look/ftl_theme.dart';
import '../src/rust/api/status.dart';
import '../ui/components/components.dart';

/// A backup the editor wrote in this session.
class LastBackup {
  const LastBackup({required this.path, required this.time});

  final String path;
  final DateTime time;
}

/// What the warnings of the core (`ftl_app::warnings`) say about the open
/// file: a running FTL and Steam Cloud.
class SaveGuard {
  const SaveGuard({this.ftlRunning, this.steamCloud, this.ftlUnknownReason});

  /// Reads the warning codes: `ftl_running`, `steam_cloud` and
  /// `process_list_unavailable`. Without a readable process list a running
  /// FTL stays unknown (`null`) and [ftlUnknownReason] says why.
  factory SaveGuard.of(List<FileWarning> warnings) {
    bool has(String code) => warnings.any((w) => w.code == code);
    final unavailable = warnings.where((w) => w.code == 'process_list_unavailable').firstOrNull;
    return SaveGuard(
      ftlRunning: has('ftl_running') ? true : (unavailable == null ? false : null),
      steamCloud: has('steam_cloud'),
      ftlUnknownReason: unavailable?.message,
    );
  }

  /// `null`: not checked, or the process list could not be read.
  final bool? ftlRunning;
  final bool? steamCloud;

  /// The message of the core when the process list could not be read.
  final String? ftlUnknownReason;
}

/// What the status bar shows. `null` in [ftlRunning] or [steamCloud] means
/// the editor has not checked it (no file open); [ftlUnknownReason] says why
/// a check could not tell whether FTL runs.
class ShellStatus {
  const ShellStatus({
    this.path,
    this.ftlRunning,
    this.steamCloud,
    this.ftlUnknownReason,
    this.lastBackup,
    this.gameData,
    this.lookError,
  });

  ShellStatus.withGuard({this.path, required SaveGuard? guard, this.lastBackup, this.gameData, this.lookError})
    : ftlRunning = guard?.ftlRunning,
      steamCloud = guard?.steamCloud,
      ftlUnknownReason = guard?.ftlUnknownReason;

  final String? path;

  /// The open `ftl.dat`; `null` is the reduced mode (icons, no game images).
  final String? gameData;

  /// Why the FTL look could not be read from [gameData]; the app then keeps
  /// the Material look.
  final String? lookError;
  final bool? ftlRunning;
  final bool? steamCloud;
  final String? ftlUnknownReason;
  final LastBackup? lastBackup;
}

/// File, running FTL, Steam Cloud and last backup, at the bottom of the window.
///
/// A click on the file name calls [onChooseFile] (the saved-game chooser of
/// the open button), a click on the game data item [onChooseGameData] (the
/// `ftl.dat` chooser), a click on the backup item [onBackups] (the backup
/// history). A clickable item shows the click cursor and turns the accent
/// color under the pointer: yellow (`select2`) in the FTL look, the primary
/// color with an ink highlight in the Material look.
///
/// Every item is an icon and AppText of the body role, the file name too: in
/// the FTL look the bar is bitmap text on the dark ground under a line in the
/// outline color.
class StatusBar extends StatelessWidget {
  const StatusBar({super.key, required this.status, this.onBackups, this.onChooseFile, this.onChooseGameData});

  final ShellStatus status;
  final VoidCallback? onBackups;
  final VoidCallback? onChooseFile;
  final VoidCallback? onChooseGameData;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final scheme = Theme.of(context).colorScheme;
    final look = FtlTheme.of(context);

    /// [hint] joins the tooltip as its last line when the item is clickable.
    Widget item(
      Key key,
      IconData icon,
      String text,
      String? tooltip, {
      Color? color,
      VoidCallback? onTap,
      Key? buttonKey,
      String? hint,
    }) => StatusItem(
      key: key,
      buttonKey: buttonKey,
      icon: icon,
      text: text,
      tooltip: onTap == null || hint == null ? tooltip : [?tooltip, hint].join('\n'),
      color: color,
      onTap: onTap,
    );

    final path = status.path;
    final file = path == null ? null : File(path);
    final backup = status.lastBackup;
    final warn = scheme.error;
    return Container(
      key: const ValueKey('status-bar'),
      height: 30,
      padding: const EdgeInsets.symmetric(horizontal: Space.xs),
      decoration: BoxDecoration(
        color: look == null ? scheme.surfaceContainer : FtlColors.ground,
        border: Border(top: BorderSide(color: look == null ? scheme.outlineVariant : FtlColors.off)),
      ),
      child: Row(
        children: [
          // The left items share the room and cut their text, so a long
          // language does not push the bar over its edge.
          Expanded(
            child: Row(
              children: [
                Flexible(
                  child: item(
                    const ValueKey('status-file'),
                    Icons.description_outlined,
                    file == null ? l.noFileOpen : file.uri.pathSegments.last,
                    file?.parent.path,
                    onTap: onChooseFile,
                    hint: l.statusChooseFile,
                  ),
                ),
                Flexible(
                  child: switch (status.ftlRunning) {
                    true => item(
                      const ValueKey('status-ftl'),
                      Icons.play_circle,
                      l.statusFtlRunning,
                      l.closeFtlHint,
                      color: warn,
                    ),
                    false => item(const ValueKey('status-ftl'), Icons.stop_circle_outlined, l.statusFtlClosed, null),
                    null => item(
                      const ValueKey('status-ftl'),
                      Icons.help_outline,
                      l.statusFtlUnknown,
                      switch (status.ftlUnknownReason) {
                        final reason? => l.statusProcessListUnavailable(reason),
                        null => l.statusNotChecked,
                      },
                    ),
                  },
                ),
                Flexible(
                  child: switch (status.steamCloud) {
                    true => item(
                      const ValueKey('status-steam'),
                      Icons.cloud_sync,
                      l.statusSteamCloud,
                      l.statusSteamCloudHint,
                      color: warn,
                    ),
                    false => item(
                      const ValueKey('status-steam'),
                      Icons.cloud_off_outlined,
                      l.statusSteamCloudOff,
                      null,
                    ),
                    null => item(
                      const ValueKey('status-steam'),
                      Icons.cloud_outlined,
                      l.statusSteamCloudUnknown,
                      l.statusNotChecked,
                    ),
                  },
                ),
                Flexible(
                  child: status.gameData == null
                      ? item(
                          const ValueKey('status-game-data'),
                          Icons.image_not_supported_outlined,
                          l.statusReducedMode,
                          l.statusReducedModeHint,
                          onTap: onChooseGameData,
                          hint: l.statusChooseGameData,
                        )
                      : item(
                          const ValueKey('status-game-data'),
                          Icons.image_outlined,
                          l.statusGameData,
                          [status.gameData, status.lookError].nonNulls.join('\n'),
                          onTap: onChooseGameData,
                          hint: l.statusChooseGameData,
                        ),
                ),
              ],
            ),
          ),
          backup == null
              ? item(
                  const ValueKey('status-backup'),
                  Icons.history,
                  l.statusNoBackup,
                  onBackups == null ? null : l.backupsShow,
                  onTap: onBackups,
                  buttonKey: const ValueKey('status-backups-button'),
                )
              : item(
                  const ValueKey('status-backup'),
                  Icons.history,
                  l.statusLastBackup(_hhmm(backup.time)),
                  backup.path,
                  onTap: onBackups,
                  buttonKey: const ValueKey('status-backups-button'),
                ),
        ],
      ),
    );
  }
}

/// One item of the [StatusBar]: icon and text, with a tooltip. With [onTap]
/// it is a button: the click cursor, and under the pointer icon and text in
/// the hover color of the look (FTL `select2` yellow, else the primary color).
class StatusItem extends StatefulWidget {
  const StatusItem({
    super.key,
    required this.icon,
    required this.text,
    this.tooltip,
    this.color,
    this.onTap,
    this.buttonKey,
  });

  final IconData icon;
  final String text;
  final String? tooltip;
  final Color? color;
  final VoidCallback? onTap;

  /// The key of the button an [onTap] makes.
  final Key? buttonKey;

  @override
  State<StatusItem> createState() => _StatusItemState();
}

class _StatusItemState extends State<StatusItem> {
  bool _hover = false;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final look = FtlTheme.of(context);
    final onTap = widget.onTap;
    final hovered = onTap != null && _hover;
    final color = hovered ? (look == null ? scheme.primary : FtlColors.hover) : widget.color;
    Widget row = Padding(
      padding: const EdgeInsets.symmetric(horizontal: Space.sm),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(widget.icon, size: 16, color: color ?? scheme.onSurfaceVariant),
          const SizedBox(width: Space.xs),
          Flexible(child: AppText(widget.text, color: color)),
        ],
      ),
    );
    if (onTap != null) {
      row = InkWell(
        key: widget.buttonKey,
        onTap: onTap,
        // InkWell defaults to WidgetStateMouseCursor.adaptiveClickable, which
        // stays the arrow outside the web: the item must say it is a button.
        mouseCursor: SystemMouseCursors.click,
        onHover: (h) => setState(() => _hover = h),
        // The FTL look marks the hover by the yellow text alone, as its buttons do.
        hoverColor: look == null ? null : Colors.transparent,
        splashColor: look == null ? null : Colors.transparent,
        highlightColor: look == null ? null : Colors.transparent,
        child: row,
      );
    }
    final tooltip = widget.tooltip;
    return tooltip == null ? row : Tooltip(message: tooltip, child: row);
  }
}

String _hhmm(DateTime t) => '${t.hour.toString().padLeft(2, '0')}:${t.minute.toString().padLeft(2, '0')}';
