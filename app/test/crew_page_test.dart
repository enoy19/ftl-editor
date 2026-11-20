import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/look/ftl_look.dart';
import 'package:ftl_editor/src/rust/api/crew.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/ui/change_sheet.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/crew_page.dart';
import 'package:flutter/gestures.dart' show kSecondaryButton;
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:ftl_editor/ui/ship_page.dart' show HintRow;
import 'package:material_ui/material_ui.dart';

import 'fake_look.dart';
import 'mock_crew.dart';

void main() {
  // A tap that misses its widget fails the test instead of printing a warning.
  WidgetController.hitTestWarningShouldBeFatal = true;

  group('the mastery step', () {
    test('cycles 0, 1, 2, 0 on the interval values of the race', () {
      const anaerobicShield = [0, 55, 110];
      expect([0, 20, 55, 56, 110].map((p) => nextMasteryPoints(p, anaerobicShield)), [55, 55, 110, 110, 0]);
      expect([0, 54, 55, 109, 110].map((p) => masteryOf(p, anaerobicShield)), [0, 0, 1, 1, 2]);
    });
  });

  group('the crew page', () {
    late Map<String, int> edits;
    late Map<String, String> texts;
    const races = [
      RaceChoice(id: 'human', name: null, lookId: 'human.m'),
      RaceChoice(id: 'engi', name: null, lookId: 'engi.m'),
      RaceChoice(id: 'crystal', name: 'Crystal', lookId: 'crystal.m'),
      RaceChoice(id: 'anaerobic', name: null, lookId: 'anaerobic.m'),
    ];

    Future<void> pump(WidgetTester tester, {List crew = const [], Size size = const Size(1280, 800)}) async {
      tester.view.physicalSize = size;
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      await tester.pumpWidget(
        variantApp(
          StatefulBuilder(
            builder: (context, setState) => Scaffold(
              body: CrewPage(
                crew: crew.isEmpty ? mockCrew : [...crew],
                edits: edits,
                onChanged: (path, value) => setState(() => edits[path] = value),
                texts: CrewTexts(
                  values: texts,
                  races: races,
                  onChanged: (path, text) => setState(() => texts[path] = text),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
    }

    setUp(() {
      edits = {};
      texts = {};
    });

    testWidgets('shows one portrait card per crew member with health and six skill tiles', (tester) async {
      await pump(tester);
      expect(find.byType(CrewCard), findsNWidgets(3));
      final tamsin = find.byKey(const ValueKey('crew-card-2'));
      expect(find.descendant(of: tamsin, matching: find.text('Tamsin')), findsOneWidget);
      expect(
        find.descendant(of: tamsin, matching: find.text('Lanius')),
        findsOneWidget,
        reason: 'l10n without ftl.dat',
      );
      expect(find.descendant(of: tamsin, matching: find.text('100/100')), findsOneWidget);
      // Six skill tiles with their full names; no ring.
      for (final skill in ['pilot', 'engine', 'shield', 'weapon', 'repair', 'combat']) {
        expect(find.byKey(ValueKey('skill-crew.2.skills.$skill')), findsOneWidget);
      }
      expect(find.descendant(of: tamsin, matching: find.text('Engines')), findsOneWidget);
      expect(find.descendant(of: tamsin, matching: find.text('Weapons')), findsOneWidget);
      expect(find.descendant(of: tamsin, matching: find.byType(CircularProgressIndicator)), findsNothing);
      expect(find.byTooltip('Shields: 56 of 110, level 1 of 2'), findsOneWidget);
      // Reduced mode: the fallback icon instead of the sprite.
      expect(find.descendant(of: tamsin, matching: find.byIcon(Icons.person)), findsOneWidget);
    });

    testWidgets('a click on a skill tile steps the mastery to the next interval value', (tester) async {
      await pump(tester);
      final ring = find.byKey(const ValueKey('skill-crew.2.skills.shield'));
      await tester.ensureVisible(ring);
      await tester.pumpAndSettle();
      await tester.tap(ring);
      await tester.pumpAndSettle();
      expect(edits, {'crew.2.skills.shield': 110});
      expect(find.byTooltip('Shields: 110 of 110, level 2 of 2'), findsOneWidget);
      // A raised skill carries the edit mark, as the Equipment page does.
      expect(find.byKey(const ValueKey('skill-mark-crew.2.skills.shield')), findsOneWidget);
      await tester.tap(ring);
      await tester.pumpAndSettle();
      expect(edits, {'crew.2.skills.shield': 0});
      await tester.tap(ring);
      await tester.pumpAndSettle();
      expect(edits, {'crew.2.skills.shield': 55});
    });

    testWidgets('a long press opens the popover for the exact points', (tester) async {
      await pump(tester);
      final ring = find.byKey(const ValueKey('skill-crew.0.skills.repair'));
      await tester.ensureVisible(ring);
      await tester.pumpAndSettle();
      await tester.longPress(ring);
      await tester.pumpAndSettle();
      final popover = find.byKey(const ValueKey('resource-popover'));
      expect(find.descendant(of: popover, matching: find.text('Hollis: Repair')), findsOneWidget);
      await tester.enterText(find.descendant(of: popover, matching: find.byType(TextField)), '30');
      await tester.tap(find.descendant(of: popover, matching: find.text('Apply')));
      await tester.pumpAndSettle();
      expect(edits, {'crew.0.skills.repair': 30});
    });

    testWidgets('a right click opens the same popover for the exact points', (tester) async {
      await pump(tester);
      final ring = find.byKey(const ValueKey('skill-crew.0.skills.repair'));
      await tester.ensureVisible(ring);
      await tester.pumpAndSettle();
      await tester.tap(ring, buttons: kSecondaryButton);
      await tester.pumpAndSettle();
      final popover = find.byKey(const ValueKey('resource-popover'));
      expect(find.descendant(of: popover, matching: find.text('Hollis: Repair')), findsOneWidget);
      expect(edits, isEmpty, reason: 'a right click does not step the mastery');
    });

    testWidgets('the hint sits at the bottom of the page while the cards leave room', (tester) async {
      await pump(tester, size: const Size(1280, 1400));
      final hint = find.byType(HintRow);
      final page = find.byType(CrewPage);
      expect(tester.getBottomLeft(page).dy - tester.getBottomLeft(hint).dy, lessThanOrEqualTo(Space.xl + 1));
    });

    testWidgets('Heal sets full health; Train fully sets every skill to its maximum', (tester) async {
      await pump(tester, crew: fullCrew);
      final heal = find.byKey(const ValueKey('heal-6'));
      await tester.ensureVisible(heal);
      await tester.pumpAndSettle();
      await tester.tap(heal);
      await tester.pumpAndSettle();
      expect(edits, {'crew.6.health': 70});
      final train = find.byKey(const ValueKey('train-6'));
      await tester.tap(train);
      await tester.pumpAndSettle();
      expect(edits['crew.6.skills.weapon'], 130);
      expect(edits['crew.6.skills.engine'], isNull, reason: 'already full: equal to the file value');
    });

    testWidgets('a click on the health chip opens the popover for the exact health', (tester) async {
      await pump(tester);
      await tester.tap(find.byKey(const ValueKey('health-crew.1.health')));
      await tester.pumpAndSettle();
      final popover = find.byKey(const ValueKey('resource-popover'));
      expect(find.descendant(of: popover, matching: find.text('Juno: Health')), findsOneWidget);
      expect(find.descendant(of: popover, matching: find.text('1–100')), findsOneWidget);
      await tester.enterText(find.descendant(of: popover, matching: find.byType(TextField)), '40');
      await tester.tap(find.descendant(of: popover, matching: find.text('Apply')));
      await tester.pumpAndSettle();
      expect(edits, {'crew.1.health': 40});
      expect(find.text('40/100'), findsOneWidget);
    });

    testWidgets('Rename stages the trimmed name; the card shows it marked', (tester) async {
      await pump(tester);
      await tester.tap(find.byKey(const ValueKey('rename-2')));
      await tester.pumpAndSettle();
      final popover = find.byKey(const ValueKey('rename-popover'));
      expect(find.descendant(of: popover, matching: find.text('Tamsin: Name')), findsOneWidget);
      await tester.enterText(find.byKey(const ValueKey('rename-field')), '   ');
      await tester.tap(find.byKey(const ValueKey('rename-apply')));
      await tester.pumpAndSettle();
      expect(find.text('1 to 32 characters'), findsWidgets, reason: 'an empty name stays in the popover');
      expect(texts, isEmpty);
      await tester.enterText(find.byKey(const ValueKey('rename-field')), '  Ripley ');
      await tester.tap(find.byKey(const ValueKey('rename-apply')));
      await tester.pumpAndSettle();
      expect(texts, {'crew.2.name': 'Ripley'});
      expect(
        find.descendant(of: find.byKey(const ValueKey('crew-card-2')), matching: find.text('Ripley')),
        findsOneWidget,
      );
      expect(find.byKey(const ValueKey('crew-name-mark-2')), findsOneWidget);
    });

    testWidgets('the race picker lists the races with portraits and stages the choice', (tester) async {
      await pump(tester);
      await tester.tap(find.byKey(const ValueKey('race-2')));
      await tester.pumpAndSettle();
      final picker = find.byKey(const ValueKey('race-picker'));
      for (final id in ['human', 'engi', 'crystal', 'anaerobic']) {
        expect(find.byKey(ValueKey('race-choice-$id')), findsOneWidget);
      }
      // Reduced mode: the app's own title; with ftl.dat the game's.
      expect(find.descendant(of: picker, matching: find.text('Lanius')), findsOneWidget);
      expect(find.descendant(of: picker, matching: find.text('Crystal')), findsOneWidget);
      await tester.tap(find.byKey(const ValueKey('race-choice-crystal')));
      await tester.pumpAndSettle();
      expect(texts, {'crew.2.race': 'crystal'});
      expect(find.descendant(of: find.byKey(const ValueKey('race-2')), matching: find.text('Crystal')), findsOneWidget);
    });

    testWidgets('a pending race shows the skills and health the core previews', (tester) async {
      // Hollis (Engi, weapons 130 = mastery 2) previewed as a Human:
      // weapons 116 of 116, health 100 of 100, pilot level 0.
      final engi = mockCrew[0];
      final human = mockMember(0, engi.name, 'human', 100, 100, [0, 0, 0, 116, 18, 1]);
      texts['crew.0.race'] = 'human';
      tester.view.physicalSize = const Size(1280, 800);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      await tester.pumpWidget(
        variantApp(
          Scaffold(
            body: CrewPage(
              crew: mockCrew,
              edits: const {},
              onChanged: (_, _) {},
              texts: CrewTexts(values: texts, races: races, previews: {0: human}),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byTooltip('Weapons: 116 of 116, level 2 of 2'), findsOneWidget);
      expect(find.byKey(const ValueKey('skill-mark-crew.0.skills.weapon')), findsNothing, reason: 'same level');
      // Repair 21 of 36 (level 1) refits to 18 of 32 (level 1); combat stays.
      expect(find.byTooltip('Repair: 18 of 32, level 1 of 2'), findsOneWidget);
    });

    testWidgets('eight crew fill two rows of four', (tester) async {
      await pump(tester, crew: fullCrew);
      expect(find.byType(CrewCard), findsNWidgets(8));
      double top(int i) => tester.getTopLeft(find.byKey(ValueKey('crew-card-$i'))).dy;
      expect({for (var i = 0; i < 4; i++) top(i)}, hasLength(1));
      expect({for (var i = 4; i < 8; i++) top(i)}, hasLength(1));
      expect(top(4), greaterThan(top(0)));
    });
  });

  testWidgets('the change sheet lists a crew edit with the crew name and the skill', (tester) async {
    final fields = crewFields(mockCrew);
    final edits = {'crew.2.skills.shield': 110};
    late Future<bool> answer;
    await tester.pumpWidget(
      variantApp(
        Builder(
          builder: (context) => TextButton(
            onPressed: () => answer = showChangeSheet(
              context,
              changes: fieldChangeRows(
                pendingChanges(fields, edits),
                AppLocalizations.of(context)!,
                look: (path, l) => fieldLook(path, l, crew: mockCrew),
              ),
              onUndo: edits.remove,
            ),
            child: const Text('save'),
          ),
        ),
      ),
    );
    await tester.tap(find.text('save'));
    await tester.pumpAndSettle();
    final row = find.byKey(const ValueKey('change-crew.2.skills.shield'));
    expect(find.descendant(of: row, matching: find.text('Tamsin: Shields')), findsOneWidget);
    expect(find.descendant(of: row, matching: findChange(56, 110)), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('change-sheet-write')));
    await tester.pumpAndSettle();
    expect(await answer, isTrue);
  });

  testWidgets('the change sheet lists a new name and a new race in words', (tester) async {
    final texts = CrewTexts(
      values: const {'crew.2.name': 'Ripley', 'crew.2.race': 'crystal', 'crew.1.name': 'Juno'},
      races: const [RaceChoice(id: 'crystal', name: 'Crystal', lookId: 'crystal.m')],
    );
    await tester.pumpWidget(
      variantApp(
        Builder(
          builder: (context) => TextButton(
            onPressed: () => showChangeSheet(
              context,
              changes: crewTextChangeRows(mockCrew, texts, AppLocalizations.of(context)!),
              onUndo: (_) {},
            ),
            child: const Text('save'),
          ),
        ),
      ),
    );
    await tester.tap(find.text('save'));
    await tester.pumpAndSettle();
    final name = find.byKey(const ValueKey('change-crew.2.name'));
    expect(find.descendant(of: name, matching: find.text('Tamsin: Name')), findsOneWidget);
    expect(find.descendant(of: name, matching: findText('Tamsin', 'Ripley')), findsOneWidget);
    final race = find.byKey(const ValueKey('change-crew.2.race'));
    expect(find.descendant(of: race, matching: find.text('Tamsin: Race')), findsOneWidget);
    expect(find.descendant(of: race, matching: findText('Lanius', 'Crystal')), findsOneWidget);
    expect(find.byKey(const ValueKey('change-crew.1.name')), findsNothing, reason: 'the name of the file');
  });

  group('new and removed crew and the look', () {
    late Map<String, int> edits;
    final looks = <(int, LookChoice)>[];
    final removedNew = <int>[];
    final kirby = mockMember(3, 'Kirby', 'human', 100, 100, [0, 0, 0, 0, 0, 0], tints: [0, 0]);

    Future<void> pump(
      WidgetTester tester, {
      required List<CrewMember> crew,
      FieldValue? add,
      int? newFrom,
      FtlLook? look,
    }) async {
      tester.view.physicalSize = const Size(1280, 1000);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      Widget themed(Widget child) => look == null ? child : inFtlLook(look, child);
      await tester.pumpWidget(
        variantApp(
          themed(
            StatefulBuilder(
              builder: (context, setState) => Scaffold(
                body: CrewPage(
                  crew: crew,
                  edits: edits,
                  onChanged: (path, value) => setState(() => edits[path] = value),
                  add: add,
                  texts: CrewTexts(
                    races: mockRaces,
                    onChanged: (_, _) {},
                    looksOf: (race) => [
                      for (final male in race == 'human' ? [true, false] : [true])
                        for (var t = 0; t < (race == 'engi' ? 1 : 4); t++)
                          LookChoice(
                            male: male,
                            tints: Int32List.fromList([0, t]),
                            lookId: mockLookId(race, male: male, tints: [0, t]),
                          ),
                    ],
                    onLook: (member, look) => looks.add((member.index, look)),
                    newFrom: newFrom,
                    onRemoveNew: removedNew.add,
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
    }

    setUp(() {
      edits = {};
      looks.clear();
      removedNew.clear();
    });

    testWidgets('the card with the plus at the end of the grid stages one default human per click', (tester) async {
      const add = FieldValue(path: 'crew.add', value: 0, min: 0, max: 5);
      await pump(tester, crew: mockCrew, add: add);
      await tester.tap(find.byKey(const ValueKey('crew-add')));
      await tester.pumpAndSettle();
      expect(edits['crew.add'], 1);
      // The core answers the new member; it shows as a new card with its mark.
      await pump(tester, crew: [...mockCrew, kirby], add: add, newFrom: 3);
      expect(find.byKey(const ValueKey('crew-card-3')), findsOneWidget);
      expect(find.byKey(const ValueKey('crew-state-3')), findsOneWidget);
      expect(tester.widget<ChangeMark>(find.byKey(const ValueKey('crew-look-mark-3'))).direction, EditDirection.raised);
      // Its remove button drops the staged member instead of staging a removal.
      await tester.tap(find.byKey(const ValueKey('remove-3')));
      expect(removedNew, [3]);
    });

    for (final ftl in [false, true]) {
      testWidgets(
        'a row of cards is one height: a removed, a new member and the plus card (${ftl ? 'FTL' : 'Material'})',
        (tester) async {
          edits['crew.1.remove'] = 1;
          await pump(
            tester,
            crew: [mockCrew[0], mockCrew[1], kirby],
            add: const FieldValue(path: 'crew.add', value: 0, min: 0, max: 5),
            newFrom: 3,
            look: ftl ? await loadFakeLook(tester) : null,
          );
          double height(Finder inside) =>
              tester.getSize(find.ancestor(of: inside, matching: find.byType(Panel)).first).height;
          final plain = height(find.byKey(const ValueKey('crew-card-0')));
          expect(height(find.byKey(const ValueKey('crew-card-1'))), plain, reason: 'removed');
          expect(height(find.byKey(const ValueKey('crew-card-3'))), plain, reason: 'new');
          expect(
            tester.getSize(find.byKey(const ValueKey('crew-add'))).height,
            plain,
            reason: 'the plus card fills the row',
          );
        },
      );
    }

    testWidgets('at FTL\'s crew limit the plus card is disabled', (tester) async {
      edits['crew.add'] = 5;
      await pump(
        tester,
        crew: mockCrew,
        add: const FieldValue(path: 'crew.add', value: 0, min: 0, max: 5),
      );
      await tester.tap(find.byKey(const ValueKey('crew-add')));
      expect(edits['crew.add'], 5);
    });

    testWidgets('Remove stages crew.<n>.remove, Keep drops it; the last of a side has no Remove', (tester) async {
      final last = mockMember(0, 'Solo', 'engi', 100, 100, [0, 0, 0, 0, 0, 0], lastOfSide: true);
      await pump(tester, crew: [...mockCrew.sublist(1), last]);
      await tester.tap(find.byKey(const ValueKey('remove-2')));
      await tester.pumpAndSettle();
      expect(edits['crew.2.remove'], 1);
      expect(find.byKey(const ValueKey('crew-state-2')), findsOneWidget);
      expect(
        tester.widget<ChangeMark>(find.byKey(const ValueKey('crew-look-mark-2'))).direction,
        EditDirection.lowered,
      );
      await tester.tap(find.byKey(const ValueKey('remove-2')));
      await tester.pumpAndSettle();
      expect(edits['crew.2.remove'], 0);
      // AppButton passes its key to the button it draws.
      final solo = tester.widget<IconButton>(find.byKey(const ValueKey('remove-0')));
      expect(solo.onPressed, isNull);
    });

    testWidgets('the look picker lists the looks of the character creator and stages the pick', (tester) async {
      await pump(tester, crew: [...mockCrew, kirby], newFrom: 3);
      expect(find.byKey(const ValueKey('look-0')), findsNothing, reason: 'an Engi has one look');
      await tester.tap(find.byKey(const ValueKey('look-3')));
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey('look-picker')), findsOneWidget);
      expect(find.byKey(const ValueKey('look-choice-7')), findsOneWidget, reason: 'human: 4 colors, male and female');
      await tester.tap(find.byKey(const ValueKey('look-choice-6')));
      await tester.pumpAndSettle();
      expect(looks.single.$1, 3);
      expect(looks.single.$2.male, isFalse);
      expect(looks.single.$2.tints, [0, 2]);
    });
  });

  testWidgets('the change sheet names new, removed and restyled crew in words', (tester) async {
    final kirby = mockMember(3, 'Kirby', 'human', 100, 100, [0, 0, 0, 0, 0, 0], tints: [0, 0]);
    final edits = {'crew.add': 1, 'crew.1.remove': 1, 'crew.3.male': 0, 'crew.3.tints.1': 2};
    await tester.pumpWidget(
      variantApp(
        Builder(
          builder: (context) => TextButton(
            onPressed: () => showChangeSheet(
              context,
              changes: crewListChangeRows(
                [...mockCrew, kirby],
                const FieldValue(path: 'crew.add', value: 0, min: 0, max: 5),
                edits,
                AppLocalizations.of(context)!,
              ),
              onUndo: (_) {},
            ),
            child: const Text('save'),
          ),
        ),
      ),
    );
    await tester.tap(find.text('save'));
    await tester.pumpAndSettle();
    final add = find.byKey(const ValueKey('change-crew.add'));
    expect(find.descendant(of: add, matching: find.text('New crew members')), findsOneWidget);
    final removed = find.byKey(const ValueKey('change-crew.1.remove'));
    expect(find.descendant(of: removed, matching: find.text('Juno: Remove')), findsOneWidget);
    expect(find.descendant(of: removed, matching: find.text('Removed')), findsOneWidget);
    final sex = find.byKey(const ValueKey('change-crew.3.male'));
    expect(find.descendant(of: sex, matching: find.text('Female')), findsOneWidget);
    final tint = find.byKey(const ValueKey('change-crew.3.tints.1'));
    expect(find.descendant(of: tint, matching: find.text('Kirby: Color layer 2')), findsOneWidget);
    expect(find.descendant(of: tint, matching: find.text('Color 3')), findsOneWidget);
    expect(isCrewListPath('nearby.crew.3.tints.1'), isTrue);
    expect(isCrewListPath('crew.3.skills.pilot'), isFalse);
  });
}
