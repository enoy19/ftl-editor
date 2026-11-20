// A ship summary as the core would answer it for a mid-run save. Widget tests
// and shots use it instead of the Rust core.

import 'package:ftl_editor/src/rust/api/editor.dart';

const mockSummary = SaveSummary(
  path: '/Users/player/Library/Application Support/FasterThanLight/continue.sav',
  fileFormat: 11,
  dlcEnabled: true,
  difficulty: 'normal',
  shipName: 'The Kestrel',
  shipBlueprintId: 'PLAYER_SHIP_HARD',
  sectorNumber: 3,
  fields: [
    FieldValue(path: 'ship.hull', value: 22, min: 0, max: 30),
    FieldValue(path: 'ship.fuel', value: 14, min: 0, max: 999),
    FieldValue(path: 'ship.drone_parts', value: 3, min: 0, max: 999),
    FieldValue(path: 'ship.missiles', value: 8, min: 0, max: 999),
    FieldValue(path: 'ship.scrap', value: 214, min: 0, max: 99999),
  ],
  nearbyFields: [],
  undecodedBytes: 18234,
  dirty: false,
);
