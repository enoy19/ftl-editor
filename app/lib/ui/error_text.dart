import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';

import '../l10n/app_localizations.dart';
import '../src/rust/api/editor.dart';

/// The localized text for an error from the Rust core.
///
/// The core answers an [EditorError] with a stable code (`AppError::code`
/// and, for game data, `AssetsError::code`); the text comes from the ARB files. The English message of the core stays as detail. Any other
/// error shows its message without the Rust backtrace.
String errorText(Object e, AppLocalizations l) {
  if (e is EditorError) {
    final path = e.path ?? '-';
    return switch (e.code) {
      'no_save' => l.errorNoSave(path),
      'no_profile' => l.errorNoProfile(path),
      'changed_on_disk' => l.errorChangedOnDisk(path),
      'decode' => l.errorDecode(path, e.message),
      'verify' => l.errorVerify(e.message),
      'edit' => l.errorEdit(e.message),
      'io' => l.errorIo(path, e.message),
      'no_home' => l.errorNoHome(e.message),
      'ftl_dat_not_found' => l.errorFtlDatNotFound(e.message),
      'config' => l.errorConfig(path, e.message),
      'unknown_image_id' => l.errorUnknownImageId(e.message),
      'ftl_dat_not_open' => l.errorFtlDatNotOpen,
      'entry_missing' => l.errorEntryMissing(e.message),
      'ftl_dat_unreadable' => l.errorFtlDatUnreadable(path, e.message),
      _ => e.message,
    };
  }
  final text = e is AnyhowException ? e.message : '$e';
  final cut = text.indexOf('\n\nStack backtrace:');
  return (cut < 0 ? text : text.substring(0, cut)).trim();
}
