# FTL modding landscape and reverse-engineering sources

Research of 2026-11-19 (GitHub API and source reading on that day). What other
FTL tools know that matters for ftl-editor, and what the editor does with it.
Inferences are marked as such; everything else was read in the linked source.

## Tools

| Name | Purpose | Platforms (macOS) | License | Activity | Link |
|---|---|---|---|---|---|
| FTL: Hyperspace | Binary mod: hooks into FTL, new races/systems/augments, Lua API, seeds, bug fixes | Windows 1.6.9, Linux 1.6.13, macOS 1.6.12/1.6.13 (x86_64 only, Rosetta on Apple Silicon) | GPL-3.0-or-later, ZHL parts MIT | push 2026-11-04, v1.23.2 (2026-09-25), 157 stars | https://github.com/FTL-Hyperspace/FTL-Hyperspace |
| FTL: Multiverse | Overhaul mod on Hyperspace | as Hyperspace; installer Windows only | none stated | v5.5.1 (2025-05-28) | https://github.com/FTL-Multiverse-Team/FTL-Multiverse-Releases |
| Slipstream Mod Manager | Patches mods into ftl.dat (`.xml.append`, `<mod:find…>`) | Java, all three; FTL up to 1.6.3 per README | GPL-2.0 text, no version in files | push 2025-08, 97 stars | https://github.com/Vhati/Slipstream-Mod-Manager |
| ftlman | Rust alternative to Slipstream, Lua patch API, installs Hyperspace | Linux, Windows, macOS arm64/x86_64 | GPL-2.0 (only/or-later unchecked) | push 2026-11-16, v0.7.4 | https://github.com/afishhh/ftlman |
| FTL Profile Editor (Vhati) | Save and profile editor, this repo's format oracle | Java | GPL-2.0 text | 2021 | https://github.com/Vhati/ftl-profile-editor |
| Superluminal 2 | Ship editor | Java | GPL-2.0 | archived 2018, forks active | https://github.com/kartoFlane/superluminal2 |
| ftl_font | Tools for the 1.6 `.font` format | Ruby | MIT | 2024 | https://github.com/D-side/ftl_font |
| ftldat-rs | dat/PKG reader (in use here) | Rust | GPL-3.0 | 2023 | https://github.com/itb-community/ftldat-rs |
| silpkg | PKG read/write (base of ftlman) | Rust nightly | GPL-2.0 | 2025 | https://github.com/afishhh/silpkg |
| FTLAV | Reads live game values | Java | MIT | 2026-07 | https://github.com/Niels-NTG/FTLAV |

## Reverse-engineering knowledge

- **Struct layouts for macOS 1.6.13**: Hyperspace
  `libzhlgen/test/functions/mac/FTLGameStripped.h` (Room, Ship, Ship::DoorState,
  HackingSystem, CloakingSystem, ShipManager, CrewMember, ShipGraph …).
  Function addresses: `libzhlgen/test/functions/mac/1.6.13/*.zhl`
  (for example `HackingSystem::SaveState`). Inference: disassembling each
  `SaveState`/`LoadState` of the local binary next to these offsets names the
  fields Java leaves unnamed. Done 2026-11-19: `docs/unknown-fields.md` lists
  each field with function, address, offset and confidence. The header's
  offsets need a shift (8-byte `std::string`, 48-byte `std::map`, a second
  vtable pointer for a class with a primary base); match by neighbours.
- **Profile field order**: a commented rebuild of `ScoreKeeper::Save` in
  Hyperspace `src/game/UserInterface/SaveFile.cpp`.
- **Bitmap fonts**: `freetype::font_data::init_bitmap` in
  `mac/1.6.13/freetype_font_data.zhl`; an independent format description in
  ftl_font `lib/ftl_font/binary/{font,character,tex}.rb` (sections padded to
  64 bytes). Hyperspace ships its own `.font` files (`Mod Files/fonts/ja/`) —
  test inputs for our decoder (load in tests, never commit).
- **Floors and interiors**: `Ship` holds `floorImageName`, `shipFloor`,
  `floorPrimitive`, `wallsPrimitive`, `doorsPrimitive`, `doorState`; `Room`
  holds `floorPrimitive`, `blackoutPrimitive`, `computerPrimitive` and more.
  The drawing order lives in `Ship::OnRenderFloor` / `Room::OnRenderFloor`
  (addresses in Ship.zhl and Room.zhl; code not read yet).

## Modded saves (Hyperspace, Multiverse)

- Hyperspace writes its own files: `<prefix>_continue.sav`, `<prefix>_prof.sav`,
  `<prefix>_prof_backup.sav`, `<prefix>_version.sav` (one int, today 4); the
  prefix is `hs` or `hs_<mod>` (Multiverse: `hs_mv_*` per a forum report).
  Source: `src/overrides/FileHelper.cpp`, `SaveFile.cpp`.
- Hyperspace inserts fields in the middle of the stream, for example in
  `StarMap::SaveGame` (`src/game/Gameplay/World/Seeds.cpp:222`) and
  `CrewMember::SaveState` (`src/game/Ship/CustomCrew.cpp:2930`), and in about
  20 more hook files. A Hyperspace save is no vanilla save with a tail; this
  codec misreads it from the first crew member on.
- Slipstream and ftlman patch ftl.dat in place, so ftl-assets already reads
  their blueprints. Hyperspace's own XML (`data/hyperspace.xml`) is unknown to
  ftl-assets.

## What ftl-editor does with it

Roadmap: name the remaining unknown fields from the Hyperspace headers, check
the font decoder against ftl_font, recognize Hyperspace saves, and edit them.
ftldat-rs stays (silpkg needs nightly Rust and is GPL-2.0).
