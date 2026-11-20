import 'package:flutter_test/flutter_test.dart';
import 'package:ftl_editor/ui/save_location.dart';

void main() {
  const def = '/home/u/FasterThanLight/continue.sav';

  group('startFile', () {
    test('takes the last file while it exists', () {
      expect(
        SaveLocation.startFile(lastFile: '/x/a.sav', defaultFile: def, fileExists: (_) => true),
        '/x/a.sav',
      );
    });
    test('falls back to the default when the last file is gone', () {
      expect(
        SaveLocation.startFile(lastFile: '/x/a.sav', defaultFile: def, fileExists: (_) => false),
        def,
      );
    });
    test('falls back to the default without a last file', () {
      expect(SaveLocation.startFile(lastFile: null, defaultFile: def), def);
    });
  });

  group('dialogDirectory', () {
    test('opens in the folder of the current path', () {
      expect(
        SaveLocation.dialogDirectory(currentPath: '/x/a.sav', defaultFile: def, dirExists: (_) => true),
        '/x',
      );
    });
    test('opens in the FTL folder when the current folder is gone', () {
      expect(
        SaveLocation.dialogDirectory(
          currentPath: '/gone/a.sav',
          defaultFile: def,
          dirExists: (d) => d != '/gone',
        ),
        '/home/u/FasterThanLight',
      );
    });
    test('lets the OS choose when no folder exists', () {
      expect(
        SaveLocation.dialogDirectory(currentPath: '', defaultFile: def, dirExists: (_) => false),
        isNull,
      );
    });
  });
}
