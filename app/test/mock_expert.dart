// A small raw view as `expertOf` answers it, for widget tests and shots.
// No Rust core.

import 'package:ftl_editor/src/rust/api/editor.dart';
import 'package:ftl_editor/src/rust/api/expert.dart';

const _min = -2147483648;
const _max = 2147483647;

ExpertEntry _entry(String path, int depth, String kind, String value, {String? edit, bool unknown = false}) =>
    ExpertEntry(
      path: path,
      depth: depth,
      kind: kind,
      value: value,
      unknown: unknown,
      field: edit == null
          ? null
          : FieldValue(
              path: edit,
              value: kind == 'bool' ? (value == 'true' ? 1 : 0) : int.parse(value),
              min: kind == 'bool' ? 0 : _min,
              max: kind == 'bool' ? 1 : _max,
            ),
    );

final mockExpert = ExpertDump(
  entries: [
    _entry('file_format', 0, 'i32', '11', edit: 'raw.file_format'),
    _entry('dlc_enabled', 0, 'bool', 'true', edit: 'raw.dlc_enabled'),
    _entry('difficulty', 0, 'enum', 'normal'),
    _entry('unknown_beta', 0, 'i32', '0', edit: 'raw.unknown_beta', unknown: true),
    _entry('state_vars', 0, 'list', '2'),
    _entry('state_vars.0', 1, 'record', ''),
    _entry('state_vars.0.id', 2, 'text', 'nebula'),
    _entry('state_vars.0.value', 2, 'i32', '11', edit: 'state.nebula'),
    _entry('state_vars.1', 1, 'record', ''),
    _entry('state_vars.1.id', 2, 'text', 'fired_shot'),
    _entry('state_vars.1.value', 2, 'i32', '1907', edit: 'state.fired_shot'),
    _entry('player_ship', 0, 'record', ''),
    _entry('player_ship.name', 1, 'text', 'Kestrel'),
    _entry('player_ship.resources', 1, 'record', ''),
    _entry('player_ship.resources.hull', 2, 'i32', '30', edit: 'raw.player_ship.resources.hull'),
    _entry('player_ship.resources.scrap', 2, 'i32', '412', edit: 'raw.player_ship.resources.scrap'),
    _entry('player_ship.crew', 1, 'list', '1'),
    _entry('player_ship.crew.0', 2, 'record', ''),
    _entry('player_ship.crew.0.name', 3, 'text', 'Ann'),
    _entry(
      'player_ship.crew.0.unknown_lambda',
      3,
      'i32',
      '0',
      edit: 'raw.player_ship.crew.0.unknown_lambda',
      unknown: true,
    ),
    _entry('situation', 0, 'none', ''),
    _entry('undecoded', 0, 'text', '20 bytes'),
  ],
  undecodedHex: const [
    '00000000  46 54 4c 00 01 00 00 00 02 00 00 00 03 00 00 00  FTL.............',
    '00000010  ff ff ff ff                                      ....',
  ],
  undecodedBytes: 20,
);
