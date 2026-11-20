// The gallery of UI variants, behind the debug switch FTL_UI_GALLERY (see
// main.dart). The maintainer picks one variant per view; the others are then deleted.
// Navigation went to variant C (now lib/shell), crew card B to the Crew page
// (lib/ui/crew_page.dart), Equipment variant C to the Equipment page
// (lib/ui/equipment_page.dart); the original look stays here.

import 'package:material_ui/material_ui.dart';

import '../components/components.dart';
import '../../l10n/app_localizations.dart';
import '../../look/look_mock.dart';
import '../app_setup.dart';

enum GalleryVariant {
  /// The original FTL look from the local ftl.dat (dark only).
  look('look-ftl');

  const GalleryVariant(this.id);

  /// Short name, also the file name of its screenshot.
  final String id;

  String label(AppLocalizations l) => switch (this) {
    look => l.galleryLook,
  };

  IconData get icon => switch (this) {
    look => Icons.rocket_outlined,
  };

  Widget build() => switch (this) {
    look => const LookMockLoader(),
  };
}

/// A MaterialApp with the app's theme and localizations around [home].
Widget variantApp(Widget home, {Brightness brightness = Brightness.light, Locale? locale}) => MaterialApp(
  debugShowCheckedModeBanner: false,
  onGenerateTitle: (context) => AppLocalizations.of(context)!.galleryTitle,
  localizationsDelegates: ftlLocalizationsDelegates,
  supportedLocales: AppLocalizations.supportedLocales,
  locale: locale,
  theme: ftlTheme(brightness),
  home: home,
);

class GalleryApp extends StatefulWidget {
  const GalleryApp({super.key});

  @override
  State<GalleryApp> createState() => _GalleryAppState();
}

class _GalleryAppState extends State<GalleryApp> {
  Brightness _brightness = Brightness.light;
  Locale _locale = const Locale('en');

  @override
  Widget build(BuildContext context) => variantApp(
    GalleryPage(
      dark: _brightness == Brightness.dark,
      onToggleDark: () =>
          setState(() => _brightness = _brightness == Brightness.dark ? Brightness.light : Brightness.dark),
      onToggleLocale: () => setState(() => _locale = Locale(_locale.languageCode == 'en' ? 'de' : 'en')),
    ),
    brightness: _brightness,
    locale: _locale,
  );
}

/// Chips choose the variant; the variant fills the rest of the window.
class GalleryPage extends StatefulWidget {
  const GalleryPage({super.key, this.dark = false, this.onToggleDark, this.onToggleLocale});

  final bool dark;
  final VoidCallback? onToggleDark;
  final VoidCallback? onToggleLocale;

  @override
  State<GalleryPage> createState() => _GalleryPageState();
}

class _GalleryPageState extends State<GalleryPage> {
  GalleryVariant _variant = GalleryVariant.look;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final scheme = Theme.of(context).colorScheme;
    return Scaffold(
      appBar: AppBar(
        title: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.palette_outlined),
            const SizedBox(width: Space.md),
            AppText(l.galleryTitle, role: TextRole.title),
          ],
        ),
        actions: [
          IconButton(
            tooltip: l.galleryDark,
            icon: Icon(widget.dark ? Icons.light_mode : Icons.dark_mode),
            onPressed: widget.onToggleDark,
          ),
          IconButton(tooltip: l.galleryLanguage, icon: const Icon(Icons.translate), onPressed: widget.onToggleLocale),
          const SizedBox(width: Space.sm),
        ],
      ),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(Space.lg, 0, Space.lg, Space.md),
            child: Wrap(
              spacing: Space.sm,
              runSpacing: Space.sm,
              children: [
                for (final v in GalleryVariant.values)
                  ChoiceChip(
                    key: ValueKey('gallery-${v.id}'),
                    avatar: Icon(v.icon, size: 18),
                    label: AppText(v.label(l)),
                    selected: v == _variant,
                    onSelected: (_) => setState(() => _variant = v),
                  ),
              ],
            ),
          ),
          Expanded(
            child: Container(
              margin: const EdgeInsets.fromLTRB(Space.lg, 0, Space.lg, Space.lg),
              decoration: BoxDecoration(
                border: Border.all(color: scheme.outlineVariant),
                borderRadius: BorderRadius.circular(12),
              ),
              clipBehavior: Clip.antiAlias,
              child: KeyedSubtree(key: ValueKey(_variant), child: _variant.build()),
            ),
          ),
        ],
      ),
    );
  }
}
