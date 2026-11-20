import 'dart:io';

/// Which saved game the editor proposes and where the file dialog opens.
///
/// Pure rules over paths, so a unit test covers them without a dialog.
abstract final class SaveLocation {
  /// SharedPreferences key of the last file that opened without error.
  static const lastFileKey = 'lastSaveFile';

  /// The last opened file while it still exists, else FTL's default file.
  static String startFile({
    required String? lastFile,
    required String defaultFile,
    bool Function(String path)? fileExists,
  }) {
    final exists = fileExists ?? (p) => File(p).existsSync();
    return lastFile != null && lastFile.isNotEmpty && exists(lastFile) ? lastFile : defaultFile;
  }

  /// The folder of the path in the field, else the folder of FTL's default
  /// file, whichever exists first. `null` lets the OS choose.
  static String? dialogDirectory({
    required String currentPath,
    required String? defaultFile,
    bool Function(String path)? dirExists,
  }) {
    final exists = dirExists ?? (p) => Directory(p).existsSync();
    for (final file in [currentPath.trim(), defaultFile]) {
      if (file == null || file.isEmpty) continue;
      final dir = File(file).parent.path;
      if (exists(dir)) return dir;
    }
    return null;
  }
}
