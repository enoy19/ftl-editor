import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../src/rust/api/status.dart';
import 'components/components.dart';

/// Local time of a backup as `yyyy-MM-dd HH:mm:ss`.
String backupTime(BackupInfo b) {
  final t = DateTime.fromMillisecondsSinceEpoch(b.modifiedMs.toInt());
  String two(int n) => n.toString().padLeft(2, '0');
  return '${t.year}-${two(t.month)}-${two(t.day)} ${two(t.hour)}:${two(t.minute)}:${two(t.second)}';
}

String _size(int bytes) => '${(bytes / 1024).toStringAsFixed(1)} KB';

/// Lists the backups of the open file, newest first, each with a restore
/// button. A restore asks first; [restore] answers whether it wrote the
/// file, and then the history closes.
Future<void> showBackupHistory(
  BuildContext context, {
  required String fileName,
  required Future<List<BackupInfo>> Function() load,
  required Future<bool> Function(BackupInfo backup) restore,
}) => showDialog<void>(
  context: context,
  builder: (_) => BackupHistory(fileName: fileName, load: load, restore: restore),
);

class BackupHistory extends StatefulWidget {
  const BackupHistory({super.key, required this.fileName, required this.load, required this.restore});

  final String fileName;
  final Future<List<BackupInfo>> Function() load;
  final Future<bool> Function(BackupInfo backup) restore;

  @override
  State<BackupHistory> createState() => _BackupHistoryState();
}

class _BackupHistoryState extends State<BackupHistory> {
  late final Future<List<BackupInfo>> _backups = widget.load();

  Future<void> _restore(BackupInfo b) async {
    final l = AppLocalizations.of(context)!;
    final yes = await showDialog<bool>(
      context: context,
      builder: (context) => AppDialog(
        key: const ValueKey('restore-confirm'),
        icon: Icons.settings_backup_restore,
        title: l.restoreTitle,
        content: AppText(l.restoreBody(backupTime(b)), wrap: true),
        actions: [
          AppButton(
            kind: ButtonKind.text,
            label: l.cancel,
            autofocus: true,
            onPressed: () => Navigator.pop(context, false),
          ),
          AppButton(kind: ButtonKind.primary, label: l.restore, onPressed: () => Navigator.pop(context, true)),
        ],
      ),
    );
    if (yes != true || !mounted) return;
    final navigator = Navigator.of(context);
    if (await widget.restore(b)) navigator.pop();
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final content = SizedBox(
      width: 520,
      child: FutureBuilder(
        future: _backups,
        builder: (context, snap) {
          if (snap.hasError) return AppText('${snap.error}', color: AppTokens.of(context).warn, wrap: true);
          final list = snap.data;
          if (list == null) return const Center(child: CircularProgressIndicator());
          if (list.isEmpty) return AppText(l.backupsEmpty, wrap: true);
          return ListView(
            shrinkWrap: true,
            children: [
              for (final b in list)
                ListRow(
                  key: ValueKey('backup-${b.path}'),
                  leading: const Icon(Icons.save_outlined),
                  title: backupTime(b),
                  subtitle: '${_size(b.bytes)} · ${Uri.file(b.path).pathSegments.last}',
                  trailing: AppButton(
                    kind: ButtonKind.text,
                    icon: Icons.settings_backup_restore,
                    label: l.restore,
                    onPressed: () => _restore(b),
                  ),
                ),
            ],
          );
        },
      ),
    );
    return AppDialog(
      key: const ValueKey('backup-history'),
      icon: Icons.history,
      title: l.backupsTitle(widget.fileName),
      content: content,
      actions: [AppButton(kind: ButtonKind.text, label: l.close, onPressed: () => Navigator.pop(context))],
    );
  }
}
