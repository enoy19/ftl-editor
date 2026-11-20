// The Expert area over a mock raw view (mock_expert.dart): the folded tree,
// the search, staging a raw edit through the value popover with its
// ChangeMark, the state var and unknown views with the undecoded hex, the
// change sheet label of a raw path, both looks and German.

import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/l10n/app_localizations.dart';
import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/ui/expert_page.dart';
import 'package:ftl_editor/ui/gallery/gallery.dart';
import 'package:material_ui/material_ui.dart';

import 'fake_look.dart';
import 'mock_expert.dart';

Finder row(String path) => find.byKey(ValueKey('expert-row-$path'));

Future<void> pumpExpert(
  WidgetTester tester, {
  ExpertView view = ExpertView.dump,
  Map<String, int> edits = const {},
  void Function(FieldValue, int)? onChanged,
  bool ftl = false,
  Locale? locale,
}) async {
  tester.view.physicalSize = const Size(1280, 800);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  Widget page = Scaffold(
    body: ExpertPage(view: view, dump: mockExpert, edits: edits, onChanged: onChanged ?? (_, _) {}),
  );
  if (ftl) page = inFtlLook(await loadFakeLook(tester), page);
  await tester.pumpWidget(variantApp(page, locale: locale));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('the tree shows the top level and opens a record by a click', (tester) async {
    await pumpExpert(tester);
    expect(find.byKey(const ValueKey('expert-warning')), findsOneWidget);
    expect(find.text('Raw edits can break the save'), findsOneWidget);
    expect(row('file_format'), findsOneWidget);
    expect(row('player_ship'), findsOneWidget);
    expect(row('player_ship.resources'), findsNothing, reason: 'records start closed');
    await tester.tap(row('player_ship'));
    await tester.pumpAndSettle();
    await tester.tap(row('player_ship.resources'));
    await tester.pumpAndSettle();
    expect(row('player_ship.resources.scrap'), findsOneWidget);
    expect(find.descendant(of: row('player_ship.resources.scrap'), matching: find.text('412')), findsOneWidget);
    await tester.tap(row('player_ship'));
    await tester.pumpAndSettle();
    expect(row('player_ship.resources.scrap'), findsNothing, reason: 'closing a record hides its children');
  });

  testWidgets('a search lists the matches flat with their whole path', (tester) async {
    await pumpExpert(tester);
    await tester.enterText(find.byKey(const ValueKey('expert-search')), 'ship scrap');
    await tester.pumpAndSettle();
    expect(find.text('player_ship.resources.scrap'), findsOneWidget);
    expect(row('file_format'), findsNothing);
    expect(find.text('1 of 22 fields'), findsOneWidget);
    await tester.enterText(find.byKey(const ValueKey('expert-search')), 'zzz');
    await tester.pumpAndSettle();
    expect(find.text('No field matches.'), findsOneWidget);
  });

  testWidgets('a click on a number opens its popover and stages the edit with its field', (tester) async {
    FieldValue? field;
    int? value;
    await pumpExpert(
      tester,
      onChanged: (f, v) {
        field = f;
        value = v;
      },
    );
    await tester.tap(row('file_format'));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('resource-popover')), findsOneWidget);
    expect(find.text('Raw: file_format'), findsOneWidget, reason: 'the popover names the raw path');
    await tester.enterText(find.byType(TextField).last, '9');
    await tester.tap(find.text('Apply'));
    await tester.pumpAndSettle();
    expect((field?.path, value), ('raw.file_format', 9));
  });

  testWidgets('a staged edit shows its value with a ChangeMark; a bool shows true or false', (tester) async {
    await pumpExpert(tester, edits: {'raw.file_format': 9, 'raw.dlc_enabled': 0, 'raw.unknown_beta': 0});
    expect(find.descendant(of: row('file_format'), matching: find.text('9')), findsOneWidget);
    expect(find.byKey(const ValueKey('expert-mark-file_format')), findsOneWidget);
    expect(find.descendant(of: row('dlc_enabled'), matching: find.text('false')), findsOneWidget);
    expect(find.byKey(const ValueKey('expert-mark-dlc_enabled')), findsOneWidget);
    expect(find.byKey(const ValueKey('expert-mark-unknown_beta')), findsNothing, reason: 'the file value');
  });

  testWidgets('the state var view lists the values with their edit by name', (tester) async {
    await pumpExpert(tester, view: ExpertView.stateVars);
    expect(row('state_vars.0.value'), findsOneWidget);
    expect(row('state_vars.1.value'), findsOneWidget);
    expect(find.text('nebula'), findsOneWidget, reason: 'a state var goes by its name');
    expect(row('file_format'), findsNothing);
    await tester.enterText(find.byKey(const ValueKey('expert-search')), 'nebula');
    await tester.pumpAndSettle();
    expect(row('state_vars.0.value'), findsOneWidget, reason: 'the search finds the name in the edit path');
    expect(row('state_vars.1.value'), findsNothing);
    await tester.tap(row('state_vars.0.value'));
    await tester.pumpAndSettle();
    expect(find.text('State var: nebula'), findsOneWidget);
  });

  testWidgets('the unknown view lists the unknown fields and the undecoded bytes as hex', (tester) async {
    await pumpExpert(tester, view: ExpertView.unknownFields);
    expect(row('unknown_beta'), findsOneWidget);
    expect(row('player_ship.crew.0.unknown_lambda'), findsOneWidget);
    expect(row('file_format'), findsNothing);
    expect(find.text('unknown'), findsNothing, reason: 'every row of this view is one');
    expect(find.byKey(const ValueKey('expert-undecoded')), findsOneWidget);
    expect(find.text('Undecoded bytes (20)'), findsOneWidget);
    expect(find.textContaining('46 54 4c 00'), findsOneWidget);
  });

  test('expertLook names raw paths and state vars, and nothing else', () async {
    final l = await AppLocalizations.delegate.load(const Locale('en'));
    expect(expertLook('raw.player_ship.resources.scrap', l)?.$2, 'Raw: player_ship.resources.scrap');
    expect(expertLook('state.nebula', l)?.$2, 'State var: nebula');
    expect(expertLook('ship.scrap', l), isNull);
    final de = await AppLocalizations.delegate.load(const Locale('de'));
    expect(expertLook('state.nebula', de)?.$2, 'Zustandsvariable: nebula');
  });

  testWidgets('the Expert area draws in the FTL look and speaks German', (tester) async {
    await pumpExpert(tester, ftl: true, locale: const Locale('de'), edits: {'raw.file_format': 9});
    expect(tester.takeException(), isNull);
    expect(findLabel('ROHE ÄNDERUNGEN KÖNNEN DEN SPIELSTAND BESCHÄDIGEN'), findsOneWidget);
    expect(find.byKey(const ValueKey('expert-mark-file_format')), findsOneWidget);
    await pumpExpert(tester, ftl: true, locale: const Locale('de'), view: ExpertView.unknownFields);
    expect(tester.takeException(), isNull);
    expect(findLabel('UNBEKANNTE FELDER'), findsNothing, reason: 'no shell here');
    expect(find.byKey(const ValueKey('expert-undecoded')), findsOneWidget);
  });
}
