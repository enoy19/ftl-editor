<p align="center"><img src="brand/logo.svg" alt="FTL Save Editor logo" width="160"></p>

# FTL Save Editor

**Jump your saved game to where you want it: ship, crew and profile, edited safely.**

FTL Save Editor edits the saved games (`continue.sav`) and the player profile
(`ae_prof.sav`) of FTL: Faster Than Light, single player. It comes as a
desktop app (macOS, Windows, Linux) and as the command line tool `ftled`.
Both use the same Rust core.

> Not affiliated with or endorsed by Subset Games. FTL: Faster Than Light is a
> trademark of Subset Games. This project ships no game files; it reads
> `ftl.dat` from your own installation at runtime.

## Features

- **Ship**: scrap, fuel, missiles, drone parts, hull; rename the ship.
- **Crew**: skills, health, race, name; add and remove crew members.
- **Equipment**: reactor, system and subsystem levels, weapons, drones,
  augments and cargo, with the limits of the ship's blueprint.
- **Map**: the sector map with beacons, stores and the rebel fleet.
- **Profile**: unlock ships and their Type-B/Type-C layouts, add or remove
  achievements, show scores and stats.
- **Expert**: every decoded field as a tree, including raw fields and the
  bytes the codec does not decode yet.
- **Game look**: the app draws game sprites, fonts and the ship floorplan from
  your local `ftl.dat`; without it, the app uses a plain look.
- **Safe saves**: every save backs up the old file (with a restore list),
  refuses when FTL wrote the file meanwhile, warns while FTL runs or Steam
  Cloud may overwrite the file, and verifies that the new bytes decode to the
  edited game. A change list with undo comes before every save.

Tested with FTL 1.6.13 (saves of format 11, profiles of format 9). The codecs
also read save formats 2, 7, 8 and 9 and profile format 4. Saves of
Hyperspace or Multiverse are not supported yet
([docs/modding-landscape.md](docs/modding-landscape.md)).

## Download

The latest release, built by GitHub Actions:

| OS | App | Command line tool `ftled` |
|---|---|---|
| macOS (Apple silicon and Intel) | [FTL-Save-Editor-macos-universal.dmg](https://github.com/enoy19/ftl-editor/releases/latest/download/FTL-Save-Editor-macos-universal.dmg) | [ftled-macos-universal.tar.gz](https://github.com/enoy19/ftl-editor/releases/latest/download/ftled-macos-universal.tar.gz) |
| Windows x64 | [FTL-Save-Editor-windows-x64.zip](https://github.com/enoy19/ftl-editor/releases/latest/download/FTL-Save-Editor-windows-x64.zip) | [ftled-windows-x64.zip](https://github.com/enoy19/ftl-editor/releases/latest/download/ftled-windows-x64.zip) |
| Linux x64 | [FTL-Save-Editor-linux-x64.tar.gz](https://github.com/enoy19/ftl-editor/releases/latest/download/FTL-Save-Editor-linux-x64.tar.gz) | [ftled-linux-x64.tar.gz](https://github.com/enoy19/ftl-editor/releases/latest/download/ftled-linux-x64.tar.gz) |

Checksums: [SHA256SUMS.txt](https://github.com/enoy19/ftl-editor/releases/latest/download/SHA256SUMS.txt). Older versions are on the
[releases page](https://github.com/enoy19/ftl-editor/releases).

- macOS: open the DMG and drag the app to Applications. The app is signed
  ad-hoc and not notarized: on the first start, right-click the app and
  choose **Open**.
- Windows: unpack the zip and start `ftl_editor.exe`. It is not signed, so
  SmartScreen may ask once.
- Linux: unpack the archive and start `ftl_editor`; it needs GTK 3.

FTL keeps its saves in `~/Library/Application Support/FasterThanLight`
(macOS), `Documents\My Games\FasterThanLight` (Windows) and
`~/.local/share/FasterThanLight` (Linux).

Close FTL before you save: FTL writes `continue.sav` when it quits and would
overwrite the edit.

## Command line

`cargo install ftled` installs the command line tool from crates.io; the
release archives above hold the same binary. From a checkout:

```sh
cargo run -p ftled -- where                       # default save file and backup folder
cargo run -p ftled -- show
cargo run -p ftled -- set ship.scrap=9999 --dry-run
cargo run -p ftled -- set ship.scrap=9999 ship.fuel=99
cargo run -p ftled -- profile --help
cargo run -p ftled -- --help                      # every command
```

## Build from source

Prerequisites:

- Rust (the app's bridge pins its toolchain in `app/rust/rust-toolchain.toml`;
  rustup installs it on first build)
- Flutter 3.x with Dart 3.14 or newer, with desktop support for your OS
- For the macOS app: Xcode
- [bun](https://bun.sh) for the scripts under `scripts/`
- `flutter_rust_bridge_codegen` in the version of `flutter_rust_bridge` in
  `Cargo.lock` (`bun scripts/codegen.ts --install` builds it with cargo)
- Optional: an FTL installation, for game images and fonts in the app

The bridge bindings and the localizations are generated, not in git:

```sh
bun scripts/codegen.ts --install          # once after a clone, again after a change in app/rust/src/api
cargo test                                # core, codecs, app
cd app && flutter run -d macos            # or -d windows, -d linux
bun scripts/build-dmg.ts --verify         # macOS: dist/FTL-Save-Editor-<version>-macos-arm64.dmg
```

Architecture, contracts, the full gate and contributor rules are in
[AGENTS.md](AGENTS.md).

## Checking a save against the Java editor

[FTL Profile Editor](https://github.com/Vhati/ftl-profile-editor) by Vhati is
the reference for the file format. `reference/ftl-profile-editor` pins its
public upstream as a submodule; `reference/patches/` adds a Java 21 build and
two small round-trip probes. With FTL installed, Java 21 or later and Maven:

```sh
bun scripts/oracle.ts crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav
bun scripts/oracle.ts --profile crates/ftl-sav/tests/fixtures/profile/ae_prof-1.6.13.sav
```

The script checks out the submodule, applies the patches, builds the Java
editor, reads the file with it, writes it back and prints `IDENTICAL` when
the bytes match (exit code 0). `--resources <dir>` names the folder that holds
`ftl.dat` when the script does not find it; `--help` lists everything.

## Screenshots

Screenshots show game art from `ftl.dat`, so they are not part of this
repository. The shot tests render them from your own installation, for example:

```sh
cd app && flutter test test/expert_shots_test.dart --dart-define=FTL_UI_SHOTS=/tmp/shots
```

## Credits

Used in the code or as format reference:

- [FTL Profile Editor](https://github.com/Vhati/ftl-profile-editor) by Vhati
  (Java, started by blerf) — the source of the file format knowledge and the
  round-trip oracle during development. GPL-2.0 text, no version named.
- [FTL: Hyperspace](https://github.com/FTL-Hyperspace/FTL-Hyperspace) by the
  Hyperspace authors — field names from `FTLGameStripped.h` and the profile
  field order from `SaveFile.cpp`. GPL-3.0-or-later (ZHL parts MIT).
- [ftldat-rs](https://github.com/itb-community/ftldat-rs) by itb-community —
  reads `ftl.dat` (crate `ftldat`). GPL-3.0.
- [ftl_font](https://github.com/D-side/ftl_font) by D-side — an independent
  description of the `.font` format, used as a cross-check. MIT.

Read for context (no code or data taken):

- [Slipstream Mod Manager](https://github.com/Vhati/Slipstream-Mod-Manager) by Vhati — GPL-2.0
- [ftlman](https://github.com/afishhh/ftlman) by afishhh — GPL-2.0
- [silpkg](https://github.com/afishhh/silpkg) by afishhh — GPL-2.0
- [Superluminal 2](https://github.com/kartoFlane/superluminal2) by kartoFlane — GPL-2.0
- [FTLAV](https://github.com/Niels-NTG/FTLAV) by Niels-NTG — MIT
- [FTL: Multiverse](https://github.com/FTL-Multiverse-Team/FTL-Multiverse-Releases) — no license stated

Libraries: binrw, flutter_rust_bridge, clap, directories, jiff, sysinfo,
quick-xml, image, serde, anyhow, thiserror and Flutter (see `Cargo.lock` and
`app/pubspec.lock`). Tools: Ghidra for the notes on the game binary, resvg for
the icon render.

## License

GPL-3.0-or-later, see [LICENSE](LICENSE). Copyright 2026 enoy19.

The Java editor ships only the GPLv2 text and names no version, so GPLv2 §9
allows any FSF version; `ftldat` is GPL-3.0, so the combined work is
GPL-3.0-or-later. Game assets are read at runtime from the local FTL
installation and are not part of this repository.
