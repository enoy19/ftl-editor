import 'package:material_ui/material_ui.dart';

import 'app_text.dart';
import 'tokens.dart';

/// Shows a short message: [icon] and [text] in a floating snack bar on the
/// ground of the look, in the text color of the look. One place for every
/// page: the pages once colored the text with onInverseSurface, which is the
/// dark ink of the FTL look on its dark snack ground.
void showAppSnack(BuildContext context, IconData icon, String text) {
  final tokens = AppTokens.of(context);
  ScaffoldMessenger.of(context)
    ..hideCurrentSnackBar()
    ..showSnackBar(
      SnackBar(
        behavior: SnackBarBehavior.floating,
        width: 560,
        content: Row(
          children: [
            Icon(icon, color: tokens.accent),
            const SizedBox(width: Space.md),
            Expanded(child: AppText(text, color: tokens.text, wrap: true)),
          ],
        ),
      ),
    );
}
