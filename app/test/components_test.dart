// One widget test per component, in the FTL look (the fake look of
// fake_look.dart) and in the Material look.

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/look/ftl_look.dart';
import 'package:ftl_editor/look/ftl_theme.dart';
import 'package:ftl_editor/ui/app_setup.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:material_ui/material_ui.dart';

import 'fake_look.dart';

/// [child] on a page of the Material look or, with [look], of the FTL look.
Widget page(Widget child, {FtlLook? look}) => MaterialApp(
  theme: look == null ? ftlTheme(Brightness.light) : ftlLookTheme(look),
  home: Scaffold(body: child),
);

void main() {
  group('bitmap text', () {
    testWidgets('the capitals stand in the middle of the line box', (tester) async {
      final look = await loadFakeLook(tester);
      final font = look.font(FtlFontId.justin12Bold);
      // The fake glyphs are 7 rows above the baseline in a 19 row line box.
      final above = font.drawBaseline - 7;
      final below = font.lineHeight - font.drawBaseline;
      expect((above - below).abs(), lessThanOrEqualTo(1), reason: 'above $above, below $below');
    });

    testWidgets('wrapped text breaks at spaces to fit the width', (tester) async {
      final look = await loadFakeLook(tester);
      final font = look.font(FtlFontId.justin10);
      await tester.pumpWidget(
        page(
          Align(
            alignment: Alignment.topLeft,
            child: SizedBox(width: 40, child: FtlText('aaaa bbbb cccc', font: font, wrap: true)),
          ),
        ),
      );
      // Each word is 4 glyphs of 6 px: two words with a space (54 px) do not fit 40 px.
      expect(tester.getSize(find.byType(FtlText)).height, 3 * font.lineHeight);
    });
  });

  group('AppText', () {
    testWidgets('in the FTL look it is bitmap text in the font of its role', (tester) async {
      final look = await loadFakeLook(tester);
      await tester.pumpWidget(
        page(
          look: look,
          const Column(
            children: [
              AppText('Shields', role: TextRole.title),
              AppText('note', role: TextRole.caption),
            ],
          ),
        ),
      );
      final title = tester.widget<FtlText>(find.byWidgetPredicate((w) => w is FtlText && w.text == 'SHIELDS'));
      expect(title.font, look.font(FtlFontId.justin12Bold), reason: 'a title is JustinFont12Bold in capitals');
      final note = tester.widget<FtlText>(find.byWidgetPredicate((w) => w is FtlText && w.text == 'note'));
      expect(note.font, look.font(FtlFontId.justin8));
      expect(note.color, FtlColors.off, reason: 'a caption is muted');
    });

    testWidgets('a sign the bitmap font lacks makes it system text of the same role', (tester) async {
      final look = await loadFakeLook(tester);
      await tester.pumpWidget(page(look: look, const AppText('3 € 4', role: TextRole.body)));
      final text = tester.widget<Text>(find.text('3 € 4'));
      expect(text.style?.fontSize, AppTokens.ftl.style(TextRole.body).fontSize);
      expect(find.byType(FtlText), findsNothing);
    });

    testWidgets('the middle dot stays bitmap text: the full stop stands in for it', (tester) async {
      final look = await loadFakeLook(tester);
      await tester.pumpWidget(page(look: look, const AppText('Sector 8 · Easy', role: TextRole.body)));
      expect(find.byWidgetPredicate((w) => w is FtlText && w.text == 'Sector 8 · Easy'), findsOneWidget);
    });

    testWidgets('in the Material look it is a Text in the style of its role', (tester) async {
      await tester.pumpWidget(page(const AppText('Shields', role: TextRole.title)));
      final context = tester.element(find.text('Shields'));
      final text = tester.widget<Text>(find.text('Shields'));
      expect(text.style?.fontSize, Theme.of(context).textTheme.titleMedium?.fontSize);
    });
  });

  group('Panel', () {
    testWidgets('frames its title and content: an FTL window or a card', (tester) async {
      final look = await loadFakeLook(tester);
      await tester.pumpWidget(
        page(
          look: look,
          const Panel(title: 'Floorplan', child: SizedBox(height: 20)),
        ),
      );
      expect(find.byType(FtlWindow), findsOneWidget);
      expect(findLabel('FLOORPLAN'), findsOneWidget);
      // The title stands clear of the frame line and its cut corner.
      final window = tester.getTopLeft(find.byType(FtlWindow));
      final title = tester.getTopLeft(findLabel('FLOORPLAN'));
      expect(title.dx - window.dx, greaterThanOrEqualTo(FtlWindow.line + Space.lg));
      expect(title.dy - window.dy, greaterThanOrEqualTo(FtlWindow.line + Space.md));

      await tester.pumpWidget(page(const Panel(title: 'Floorplan', child: SizedBox(height: 20))));
      await tester.pumpAndSettle(); // the theme change animates
      expect(find.byType(Card), findsOneWidget);
      expect(find.text('Floorplan'), findsOneWidget);
    });
  });

  group('AppDialog', () {
    for (final ftl in [true, false]) {
      testWidgets('is centered and as wide as its content (${ftl ? 'FTL' : 'Material'})', (tester) async {
        tester.view.physicalSize = const Size(1280, 800);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.reset);
        final look = ftl ? await loadFakeLook(tester) : null;
        await tester.pumpWidget(
          page(
            look: look,
            AppDialog(
              key: const ValueKey('dialog'),
              icon: Icons.extension,
              title: 'Add augment',
              content: const SizedBox(width: 300, height: 100),
              actions: [AppButton(label: 'Cancel', onPressed: () {})],
            ),
          ),
        );
        final r = tester.getRect(find.byKey(const ValueKey('dialog')).first);
        final frame = tester.getRect(
          find
              .descendant(of: find.byKey(const ValueKey('dialog')), matching: find.byType(ftl ? FtlWindow : Material))
              .first,
        );
        expect(frame.width, lessThan(500), reason: 'not stretched over the page');
        expect((frame.center.dx - 640).abs(), lessThan(1), reason: 'centered');
        expect(r.center.dx, closeTo(640, 1));
      });
    }
  });

  group('AppPopover', () {
    testWidgets('a title row, the content and the actions in a window', (tester) async {
      final look = await loadFakeLook(tester);
      var applied = false;
      await tester.pumpWidget(
        page(
          look: look,
          Center(
            child: SizedBox(
              width: 280,
              child: AppPopover(
                icon: Icons.favorite,
                title: 'Hull',
                actions: [AppButton(label: 'Apply', onPressed: () => applied = true)],
                child: const SizedBox(height: 30),
              ),
            ),
          ),
        ),
      );
      expect(find.byType(FtlWindow), findsOneWidget);
      expect(findLabel('HULL'), findsOneWidget);
      await tester.tap(findLabel('APPLY'));
      expect(applied, isTrue);
      // The title is centered on its icon.
      final icon = tester.getCenter(find.byIcon(Icons.favorite));
      final title = tester.getCenter(findLabel('HULL'));
      expect((icon.dy - title.dy).abs(), lessThanOrEqualTo(1));
    });
  });

  group('AppButton', () {
    testWidgets('an FTL button with the label in capitals, a Material button by kind', (tester) async {
      final look = await loadFakeLook(tester);
      var pressed = 0;
      await tester.pumpWidget(
        page(
          look: look,
          AppButton(label: 'Save', icon: Icons.save, onPressed: () => pressed++),
        ),
      );
      expect(find.byType(FtlButton), findsOneWidget);
      expect(find.byIcon(Icons.save), findsNothing, reason: 'an FTL face with a label carries no icon');
      await tester.tap(findLabel('SAVE'));
      expect(pressed, 1);

      await tester.pumpWidget(
        page(AppButton(label: 'Save', icon: Icons.save, kind: ButtonKind.primary, onPressed: () => pressed++)),
      );
      await tester.pumpAndSettle(); // the theme change animates
      await tester.tap(find.widgetWithIcon(FilledButton, Icons.save));
      expect(pressed, 2);
    });
  });

  group('ListRow', () {
    for (final ftl in [true, false]) {
      testWidgets('has one height with or without a second line (${ftl ? 'FTL' : 'Material'})', (tester) async {
        final look = ftl ? await loadFakeLook(tester) : null;
        await tester.pumpWidget(
          page(
            look: look,
            const Column(
              children: [
                ListRow(key: ValueKey('one'), leading: Icon(Icons.favorite), title: 'Hull'),
                ListRow(key: ValueKey('two'), leading: Icon(Icons.lock_open), title: 'Kestrel', subtitle: 'adds ACH'),
              ],
            ),
          ),
        );
        final one = tester.getSize(find.byKey(const ValueKey('one')));
        final two = tester.getSize(find.byKey(const ValueKey('two')));
        expect(one.height, two.height);
        expect(one.height, ftl ? AppTokens.ftl.rowHeight : 72);
      });
    }
  });

  group('Tag', () {
    testWidgets('an FTL box or a chip; with onTap a button', (tester) async {
      final look = await loadFakeLook(tester);
      var taps = 0;
      await tester.pumpWidget(
        page(
          look: look,
          Tag(label: 'Sector 3', icon: Icons.place, onTap: () => taps++),
        ),
      );
      expect(find.byType(FtlBox), findsOneWidget);
      await tester.tap(findLabel('Sector 3'));
      expect(taps, 1);

      await tester.pumpWidget(page(const Tag(label: 'Sector 3', icon: Icons.place)));
      await tester.pumpAndSettle(); // the theme change animates
      expect(find.byType(Chip), findsOneWidget);
      await tester.pumpWidget(page(Tag(label: 'Sector 3', onTap: () => taps++)));
      await tester.tap(find.byType(ActionChip));
      expect(taps, 2);
    });
  });

  group('showAppSnack', () {
    for (final ftl in [true, false]) {
      testWidgets('shows the message in the text color of the look (${ftl ? 'FTL' : 'Material'})', (tester) async {
        final look = ftl ? await loadFakeLook(tester) : null;
        await tester.pumpWidget(
          page(
            look: look,
            Builder(
              builder: (context) => TextButton(
                onPressed: () => showAppSnack(context, Icons.healing, 'Heal crew: nothing to change'),
                child: const Text('go'),
              ),
            ),
          ),
        );
        await tester.tap(find.text('go'));
        await tester.pumpAndSettle();
        final text = find.byWidgetPredicate((w) => w is AppText && w.text == 'Heal crew: nothing to change');
        expect(text, findsOneWidget);
        final tokens = AppTokens.of(tester.element(text));
        expect(tester.widget<AppText>(text).color, tokens.text);
        expect(tester.widget<SnackBar>(find.byType(SnackBar)).behavior, SnackBarBehavior.floating);
      });
    }
  });

  group('AppTextField', () {
    for (final ftl in [true, false]) {
      testWidgets('types and hints in the monospace font of the code role (${ftl ? 'FTL' : 'Material'})', (
        tester,
      ) async {
        final look = ftl ? await loadFakeLook(tester) : null;
        await tester.pumpWidget(
          page(
            look: look,
            const AppTextField(key: ValueKey('field'), hint: 'Filter', search: true),
          ),
        );
        // The hint is system text of the code role in both looks.
        final hint = tester.widget<AppText>(find.widgetWithText(AppText, 'Filter'));
        expect(hint.role, TextRole.code);
        expect(find.byType(FtlText), findsNothing, reason: 'no bitmap hint');
        await tester.enterText(find.byKey(const ValueKey('field')), 'BEAM_HULL');
        final typed = tester.widget<EditableText>(find.byType(EditableText));
        expect(typed.style.fontFamily, 'Menlo');
        expect(typed.style.fontFamilyFallback, contains('monospace'));
        expect(typed.controller.text, 'BEAM_HULL');
      });
    }
  });
}
