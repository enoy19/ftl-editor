import 'package:material_ui/material_ui.dart';

import 'l10n/app_localizations.dart';
import 'src/rust/frb_generated.dart';
import 'ui/app_setup.dart';
import 'ui/editor_page.dart';
import 'ui/gallery/gallery.dart';

/// Debug switch: `flutter run -d macos --dart-define=FTL_UI_GALLERY=true`
/// opens the gallery of UI variants instead of the editor.
const _gallery = bool.fromEnvironment('FTL_UI_GALLERY');

Future<void> main() async {
  // The gallery shows mock data; only its FTL look reads ftl.dat through the Rust core.
  await RustLib.init();
  if (_gallery) {
    runApp(const GalleryApp());
    return;
  }
  runApp(const FtlEditorApp());
}

class FtlEditorApp extends StatelessWidget {
  const FtlEditorApp({super.key, this.home = const EditorPage()});

  /// The start page; tests pass an [EditorPage] with their own file chooser.
  final Widget home;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      onGenerateTitle: (context) => AppLocalizations.of(context)!.appTitle,
      localizationsDelegates: ftlLocalizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      theme: ftlTheme(Brightness.light),
      darkTheme: ftlTheme(Brightness.dark),
      home: home,
    );
  }
}
