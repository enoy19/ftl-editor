// The shell in the original FTL look: tab row C with Expert and Save on the
// right above one FTL window that holds the page (decisions
// ftl-editor-uses-top-tabs-and-portrait-crew-cards and
// ftl-editor-tabs-are-flat-ftl-buttons). AppShell picks it when the theme
// carries an FtlTheme.

import 'package:material_ui/material_ui.dart';

import '../l10n/app_localizations.dart';
import '../look/ftl_look.dart';
import '../look/ftl_tabs.dart';
import '../look/ftl_theme.dart';
import '../ui/components/components.dart';
import 'destination.dart';

/// The window of the editor in the FTL look. [page] is the body of [dest];
/// [banner] sits above it inside the window.
class FtlShell extends StatelessWidget {
  const FtlShell({
    super.key,
    required this.look,
    required this.dest,
    required this.onDestination,
    required this.onSave,
    required this.busy,
    required this.banner,
    required this.page,
    required this.statusBar,
    this.actions,
  });

  final FtlLook look;
  final Destination dest;
  final ValueChanged<Destination> onDestination;
  final VoidCallback? onSave;
  final bool busy;
  final Widget? banner;
  final Widget page;
  final Widget statusBar;

  /// The action buttons, left of Expert.
  final Widget? actions;

  /// The face of a closed tab ends 3 px above the frame line; the buttons on
  /// the right end there too. 28 is the height of a button.
  static const _buttonTop = tabStripHeight + frameGlow - 3 - 28.0;

  Future<void> _openExpert(BuildContext context) async {
    final l = AppLocalizations.of(context)!;
    final box = context.findRenderObject()! as RenderBox;
    final overlay = Overlay.of(context).context.findRenderObject()! as RenderBox;
    final at = box.localToGlobal(Offset(0, box.size.height), ancestor: overlay);
    final chosen = await showMenu<Destination>(
      context: context,
      position: RelativeRect.fromRect(at & Size(box.size.width, 0), Offset.zero & overlay.size),
      items: [
        for (final d in Destination.expertItems)
          PopupMenuItem(
            value: d,
            child: Row(
              children: [
                Icon(d.icon, size: 18),
                const SizedBox(width: Space.sm),
                AppText(d.label(l)),
              ],
            ),
          ),
      ],
    );
    if (chosen != null) onDestination(chosen);
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context)!;
    final areas = Destination.areas;
    final banner = this.banner;
    return Scaffold(
      backgroundColor: FtlColors.ground,
      body: Padding(
        padding: const EdgeInsets.fromLTRB(Space.lg, Space.md, Space.lg, Space.xs),
        child: Stack(
          clipBehavior: Clip.none,
          children: [
            Positioned.fill(
              top: tabStripHeight.toDouble(),
              child: FtlWindow(
                key: const ValueKey('ftl-page-window'),
                look: look,
                fill: FtlColors.window,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    SizedBox(height: 2, child: busy ? const LinearProgressIndicator(minHeight: 2) : null),
                    if (banner != null)
                      Padding(padding: const EdgeInsets.fromLTRB(Space.lg, Space.md, Space.lg, 0), child: banner),
                    Expanded(child: ClipRect(child: page)),
                  ],
                ),
              ),
            ),
            // After the window, so the open tab covers the glow of its line.
            Positioned(
              left: 0,
              top: 0,
              child: FtlTabStrip(
                key: const ValueKey('area-tabs'),
                look: look,
                tabs: [for (final d in areas) FtlTab(d.label(l), key: ValueKey('area-${d.name}'))],
                open: areas.indexOf(dest),
                onOpen: (i) => onDestination(areas[i]),
              ),
            ),
            Positioned(
              right: 4,
              top: _buttonTop,
              child: Row(
                children: [
                  if (actions case final actions?) ...[actions, const SizedBox(width: Space.md)],
                  Builder(
                    builder: (context) => AppButton(
                      key: const ValueKey('expert-menu'),
                      // The chosen Expert item names the menu, since no tab marks it.
                      label: dest.expert ? dest.label(l) : l.areaExpert,
                      tooltip: l.areaExpert,
                      onPressed: () => _openExpert(context),
                    ),
                  ),
                  const SizedBox(width: Space.sm),
                  AppButton(
                    key: const ValueKey('save-button'),
                    kind: ButtonKind.primary,
                    icon: Icons.save,
                    label: l.save,
                    onPressed: onSave,
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
      bottomNavigationBar: statusBar,
    );
  }
}
