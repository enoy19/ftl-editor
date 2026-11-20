import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import 'components/tokens.dart';

/// The one Material theme of the app, for light and dark, with the Material
/// [AppTokens] (the fallback look while no ftl.dat is open).
ThemeData ftlTheme(Brightness brightness) {
  final base = ThemeData(colorSchemeSeed: Colors.deepOrange, brightness: brightness, useMaterial3: true);
  return base.copyWith(extensions: [AppTokens.material(base)]);
}

/// The generated `AppLocalizations.localizationsDelegates` carry the
/// Material and Cupertino localizations of flutter_localizations, but the
/// widgets of material_ui look up their own localization types. Without
/// material_ui's delegates every AppBar, Chip and TextField throws
/// "No MaterialLocalizations found" under `de`.
const ftlLocalizationsDelegates = <LocalizationsDelegate<dynamic>>[
  AppLocalizations.delegate,
  ...GlobalMaterialLocalizations.delegates,
];
