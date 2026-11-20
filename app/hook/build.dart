import 'dart:io';

import 'package:flutter_rust_bridge_hooks/flutter_rust_bridge_hooks.dart';

void main(List<String> args) async {
  await build(args, (input, output) async {
    await FlutterRustBridgeNativeAssetsBuilder(
      cratePath: 'rust',
      extraCargoEnvironmentVariables: {'CARGO_ENCODED_RUSTFLAGS': _remapFlags(input.packageRoot)},
    ).run(input: input, output: output);
  });
}

/// `--remap-path-prefix` for the cargo home, the rustup home and the
/// repository, so the bridge library carries no path of the build machine:
/// panic locations and debug strings name `/cargo`, `/rustup` and `/src`
/// instead of the home folder. Stable Cargo has no `trim-paths` yet.
/// `CARGO_ENCODED_RUSTFLAGS` separates the flags with 0x1f.
String _remapFlags(Uri packageRoot) {
  final env = Platform.environment;
  final home = Platform.isWindows ? env['USERPROFILE'] : env['HOME'];
  final sep = Platform.pathSeparator;
  final repo = Directory.fromUri(packageRoot).parent.absolute.path.replaceAll(RegExp(r'[\\/]+$'), '');
  final cargo = env['CARGO_HOME'] ?? (home == null ? null : '$home$sep.cargo');
  final rustup = env['RUSTUP_HOME'] ?? (home == null ? null : '$home$sep.rustup');
  final prefixes = {?cargo: '/cargo', ?rustup: '/rustup', repo: '/src'};
  return [for (final MapEntry(:key, :value) in prefixes.entries) '--remap-path-prefix=$key=$value'].join('\x1f');
}
