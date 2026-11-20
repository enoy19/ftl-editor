// The ship card edits the identity of the run: the name with Rename (a
// popover with a text field), the type and the difficulty as tags with a
// picker, the sector as a tag that names why it stays. A pending edit shows
// its new value with a mark, and the change sheet names it in words.

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/l10n/app_localizations_de.dart';
import 'package:ftl_editor/l10n/app_localizations_en.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/identity.dart';
import 'package:ftl_editor/ui/components/components.dart';
import 'package:ftl_editor/ui/ship_card.dart';
import 'package:material_ui/material_ui.dart';

import 'app_shell_test.dart' show pumpShell;
import 'fake_look.dart';
import 'mock_summary.dart';

const _identity = ShipIdentity(
  namePath: 'ship.name',
  name: 'The Kestrel',
  typePath: 'ship.type',
  blueprintId: 'PLAYER_SHIP_HARD',
  difficulty: FieldValue(path: 'difficulty', value: 1, min: 0, max: 2),
  sector: 3,
);

const _choices = [
  ShipTypeChoice(id: 'PLAYER_SHIP_HARD', name: 'The Kestrel', layout: 'A'),
  ShipTypeChoice(id: 'PLAYER_SHIP_HARD_2', name: 'Red-Tail', layout: 'B', block: 'other_layout'),
  ShipTypeChoice(id: 'PLAYER_SHIP_EASY', name: 'The Kestrel', layout: 'A'),
];

void main() {
  final AppLocalizations en = AppLocalizationsEn();
  final AppLocalizations de = AppLocalizationsDe();

  Future<(Map<String, String>, Map<String, int>)> pumpCard(
    WidgetTester tester, {
    ShipIdentity identity = _identity,
    Map<String, String> texts = const {},
    Map<String, int> edits = const {},
  }) async {
    final staged = (<String, String>{}, <String, int>{});
    await pumpShell(
      tester,
      Scaffold(
        body: ShipCard(
          summary: mockSummary,
          edits: ShipCardEdits(
            identity: identity,
            texts: texts,
            edits: edits,
            onText: (p, t) => staged.$1[p] = t,
            onEdit: (p, v) => staged.$2[p] = v,
            typeChoices: () => _choices,
          ),
        ),
      ),
    );
    return staged;
  }

  testWidgets('Rename stages the trimmed name of the ship', (tester) async {
    final (texts, _) = await pumpCard(tester);
    expect(findLabel('The Kestrel'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('ship-rename')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('rename-popover')), findsOneWidget);
    await tester.enterText(find.byKey(const ValueKey('rename-field')), '  Red Dwarf ');
    await tester.tap(find.byKey(const ValueKey('rename-apply')));
    await tester.pumpAndSettle();
    expect(texts, {'ship.name': 'Red Dwarf'});
  });

  testWidgets('a pending name, type and difficulty show their new values with marks', (tester) async {
    await pumpCard(
      tester,
      texts: {'ship.name': 'Red Dwarf', 'ship.type': 'PLAYER_SHIP_EASY'},
      edits: {'difficulty': 2},
    );
    expect(findLabel('Red Dwarf'), findsOneWidget);
    expect(find.byKey(const ValueKey('ship-name-mark')), findsOneWidget);
    expect(tester.widget<Tag>(find.byKey(const ValueKey('ship-difficulty'))).label, 'Hard');
    expect(tester.widget<Tag>(find.byKey(const ValueKey('ship-difficulty'))).marked, isTrue);
    expect(tester.widget<Tag>(find.byKey(const ValueKey('ship-type'))).marked, isTrue);
  });

  testWidgets('the difficulty picker stages a difficulty', (tester) async {
    final (_, edits) = await pumpCard(tester);
    await tester.tap(find.byKey(const ValueKey('ship-difficulty')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('difficulty-picker')), findsOneWidget);
    expect(find.byKey(const ValueKey('difficulty-choice-2')), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('difficulty-choice-2')));
    await tester.pumpAndSettle();
    expect(edits, {'difficulty': 2});
  });

  testWidgets('a difficulty the core keeps names why and opens nothing', (tester) async {
    await pumpCard(
      tester,
      identity: const ShipIdentity(
        namePath: 'ship.name',
        name: 'The Kestrel',
        typePath: 'ship.type',
        blueprintId: 'PLAYER_SHIP_HARD',
        difficulty: FieldValue(path: 'difficulty', value: 1, min: 1, max: 1),
        difficultyBlock: 'flagship',
        sector: 8,
      ),
    );
    final tag = tester.widget<Tag>(find.byKey(const ValueKey('ship-difficulty')));
    expect(tag.onTap, isNull);
    expect(tag.tooltip, en.difficultyBlockFlagship);
  });

  testWidgets('the sector tag names why it stays', (tester) async {
    await pumpCard(tester);
    final tag = tester.widget<Tag>(find.byKey(const ValueKey('ship-sector')));
    expect(tag.onTap, isNull);
    expect(tag.tooltip, en.shipSectorFixed);
    expect(tag.label, en.sector(3));
  });

  testWidgets('the type picker lists every type; a refused one names its reason and stays', (tester) async {
    final (texts, _) = await pumpCard(tester);
    await tester.tap(find.byKey(const ValueKey('ship-type')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('ship-type-picker')), findsOneWidget);
    expect(findLabel(en.typeBlockOtherLayout), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('ship-type-choice-PLAYER_SHIP_HARD_2')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('ship-type-picker')), findsOneWidget, reason: 'a refused type does nothing');
    await tester.tap(find.byKey(const ValueKey('ship-type-choice-PLAYER_SHIP_EASY')));
    await tester.pumpAndSettle();
    expect(texts, {'ship.type': 'PLAYER_SHIP_EASY'});
  });

  test('the change sheet names name, type and difficulty in words, in both languages', () {
    const nearby = ShipIdentity(
      namePath: 'nearby.ship.name',
      name: 'should not be seen',
      typePath: 'nearby.ship.type',
      blueprintId: 'JELLY_BUTTON_P_DLC',
      difficulty: FieldValue(path: 'difficulty', value: 1, min: 0, max: 2),
      sector: 3,
    );
    for (final l in [en, de]) {
      final rows = identityChangeRows(
        player: _identity,
        nearby: nearby,
        texts: {'ship.name': 'Red Dwarf', 'ship.type': 'PLAYER_SHIP_EASY', 'nearby.ship.name': 'Jelly'},
        edits: {'difficulty': 2},
        choices: _choices,
        l: l,
      );
      expect([for (final r in rows) r.id], ['ship.name', 'ship.type', 'difficulty', 'nearby.ship.name']);
      expect(rows[0].change, ('The Kestrel', 'Red Dwarf'));
      expect(rows[1].change, (l.shipTypeLayout('The Kestrel', 'A'), l.shipTypeLayout('The Kestrel', 'A')));
      expect(rows[2].change, (l.difficultyNormal, l.difficultyHard));
      expect(rows[3].title, l.shipEnemyField(l.fieldShipName));
    }
  });
}
