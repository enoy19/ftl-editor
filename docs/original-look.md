# The original FTL look: what it is made of

Everything below was read from `ftl.dat` of FTL 1.6.13 (macOS Steam, 2,837
entries) with `ftled assets ls` and `ftled assets cat`. Nothing extracted is
in the repository (game assets are read at runtime only);
the app reads the same entries at runtime. Colors and insets are numbers
measured from the pixels.

## Fonts (`fonts/*.font`)

The Latin fonts are `JustinFont8`, `JustinFont10`, `JustinFont11Bold`,
`JustinFont12Bold`, `HL1`, `HL2` and `num_font`. `c&c.font`, `c&cnew.font`,
`fonts/ja/*` and `fonts/zh-Hans/black.ttf` serve other scripts. The binary
`FTL.app/Contents/MacOS/FTL` names them in this order: `num_font`, `HL1`,
`HL2`, `JustinFont11Bold`, `JustinFont12Bold`, `JustinFont10`, `JustinFont8`,
`c&cnew`, `c&c`, then the Japanese fonts. It also names `.ttf` twins of the
Latin fonts that the package does not hold.

### Format (decoded)

A web search found no description of the format (FTL modding pages and
Slipstream discuss the package, not the fonts). The layout below comes from
the bytes; `crates/ftl-assets/src/font.rs` decodes it. Big-endian:

| Offset | Field |
|---|---|
| 0 | `FONT` |
| 4 | u8 version 1, u8 line height, u8 baseline, u8 0 |
| 8 | u32 24: offset of the first glyph record |
| 12 | u16 glyph count, u16 16: record size |
| 16 | u32 offset of the TEX block |
| 20 | u32 length of the TEX block (32 + width * height) |
| 24 | glyph records, 16 bytes each |
| TEX | `TEX\n`, u16 0x0240, u16 0, u16 width, u16 height, u16 1, u16 0, u32 32 (header), u32 width * height, then one coverage byte per pixel, row by row |

A glyph record: u32 code point, u16 x, u16 y, u8 width, u8 height, i8 top
(rows above the baseline), u8 0, i8 left, u8 0, i8 right, u8 pad (HL1 fills
it with 0x9a, HL2 with 0x66). The pen moves `left + width + right`. Every
coverage byte is 0 or 255: the fonts are 1-bit bitmaps.

Header bytes as seen (`xxd -l 32`):

```text
JustinFont8  464f4e54 01 13 0c 00 00000018 0184 0010 00001880 00010020
             first record: 00000020 0001 0010 01 00 fd 00 03 00 01 00   (' ')
             second:       00000021 0003 0005 02 08 08 00 00 00 01 00   ('!')
HL1          464f4e54 01 0b 08 00 00000018 012c 0010 00001300 00008020
             first record: 00000020 0001 000b 01 01 fe 00 04 00 00 9a
num_font     464f4e54 01 18 14 00 00000018 0145 0010 00001480 00020020
TEX block    54455830a 0240 0000 0100 0100 0001 0000 00000020 00010000   (JustinFont8, 256x256)
```

| Font | Glyphs | Atlas | Line height | Baseline | Tallest glyph | Character |
|---|---|---|---|---|---|---|
| JustinFont8 | 388 | 256x256 | 19 | 12 | 12 | small regular sans, body text |
| JustinFont10 | 238 | 256x128 | 19 | 12 | 13 | regular sans |
| JustinFont11Bold | 381 | 256x256 | 19 | 12 | 14 | bold sans; lacks `{` and some signs |
| JustinFont12Bold | 238 | 256x256 | 19 | 12 | 15 | bold sans, names and titles |
| HL1 | 300 | 128x256 | 11 | 8 | 8 | tiny caps-height pixel font, labels |
| HL2 | 252 | 256x128 | 12 | 9 | 9 | outline letters: the coverage is the outline only |
| num_font | 325 | 256x512 | 24 | 20 | 20 | large rounded pixel font, numbers |

Each font holds Latin with accents, `ß`, `ẞ` and Cyrillic.

Which text uses which font: the `.font` files carry no use, and the game
binary was not disassembled. What the pixels show: HL2 is drawn like the
labels baked into the button images (`img/customizeUI/button_accept_*`:
outlined letters in ink on the button face). The first mock used HL2 for tab
and button labels; those tabs were rejected, and the tab and button
labels now use JustinFont12Bold. num_font matches the size of the number field of
`img/statusUI/top_scrap.png`, JustinFont12Bold and JustinFont10 the ship
name and the body text. Confirm or correct this against the running game.

## Frames and buttons

States follow one naming rule in every folder: `_on` is the idle face,
`_off` the disabled one, `_select2` hover or pressed; `_base` is the shape
behind a state-less image. Insets are what `ftl_assets::measure_insets`
answers: first where the straight outline starts, then widened until the
stretched strips are uniform. The second step matters: the glow of a cut
corner reaches past the straight start, and the first shot of the mock with
the straight-edge insets (19) smeared that glow into pale bands along each
edge. The test `measures_the_insets_of_the_real_frames` in
`crates/ftl-assets/tests/real.rs` prints and checks them.

| Image | Size | Insets l/t/r/b | Notes |
|---|---|---|---|
| `img/window_outline.png` | 66x72 | 31/36/32/35 (straight edge at 19) | The generic window: 7 px glow, 4 px line, corners cut by 12. Transparent middle; the uniform middle is 3x1 px. |
| `img/window_base.png` | 66x72 | 33/36/32/35 | The same with the hex pattern inside (not a clean 9-slice: the pattern stretches); `window_mask.png` is the fill shape. |
| `img/optionsUI/tab_page_{on,off,select2}.png` | 74x29 | 33/14/33/14 (`_off` 33/14/32/14) | Trapezoid tab, slant 21 px plus glow; no uniform row, so one middle row is left: stretch horizontally only. `_off` is 2 px taller (the open page). |
| `img/customizeUI/button_accept_{on,off,select2}.png` | 134x34 | 2/2/2/2 | Text baked in; the face has corners cut by 2. |
| `img/customizeUI/box_crew_details.png` | 205x122 | 3/3/3/3 | 3 px line, inner dividers: not a clean 9-slice. |
| `img/statusUI/top_scrap.png` | 135x55 | — | Icon box plus number field, drawn as is. Also `top_fuel_on` 80x40, `top_missiles_on` 80x40, `top_drones_on` 75x40, `top_hull` 385x65. |
| `img/upgradeUI/Equipment/tabButtons/*_{on,off,select2}.png` | 127 or 109 x48 | — | Closed tabs of the ship menu with icons baked in (see "How FTL builds its tab row"). |
| `img/upgradeUI/Equipment/{upgrades,crew,equipment}_main.png` | 601x478 | — | The ship-menu window of each page, its open tab part of the frame. |
| `img/scoreUI/tab_*_{on,off,select2}.png` | 193 to 387 x53 | — | Arrow tabs of the score screen, no text in the image. |
| `img/systemUI/button_default_{on,off,select2,base}.png` | 40x79 | — | System power button. |
| `img/main_menus/button_cancel_base.png` | 173x54 | — | Dark button well. |
| `img/confirm_window.png` | 140x90 | — | Window with two button wells. |

Other UI folders: `img/customizeUI` (hangar: `box_*`, `button_*`,
`miniship_*`), `img/upgradeUI` (upgrade and equipment screens),
`img/storeUI` (`store_*` buttons), `img/systemUI` (system buttons and
weapon boxes), `img/statusUI` (top bar), `img/ui_icons/icon_{scrap,fuel,missiles,drones}.png`
(32x32 white icons).

### Resource icons

The resource boxes of the Ship page draw these entries (read 2026-11-19),
each at its own size (factor 1) in a 32x32 slot, multiplied with the text
color of the box (`resourceImage` in `app/lib/ui/resource_bar.dart`):

| Resource | Entry | Size | Opaque part |
|---|---|---|---|
| Fuel | `img/ui_icons/icon_fuel.png` | 32x32 | 19x21 at 7,6 |
| Drone parts | `img/ui_icons/icon_drones.png` | 32x32 | 14x16 at 9,8 |
| Missiles | `img/ui_icons/icon_missiles.png` | 32x32 | 21x7 at 5,12 |
| Scrap | `img/ui_icons/icon_scrap.png` | 32x32 | 22x22 at 5,5 |
| Hull | `img/map/map_icon_ship.png` | 64x64 | 16x25 at 25,20, cropped to the middle 32x32 |

FTL has no hull icon: its top bar writes HULL on `top_hull_label.png` (52x28)
beside the bar of `top_hull.png`, and `ftl.dat` holds no heart, shield or
ship symbol in `img/ui_icons`, `img/statusUI` or `img/storeUI`. The hull
takes the player ship marker of the sector map instead: a white outline with
a dark fill and a cyan glow. The `top_*_on.png` boxes carry the same icons
as `ui_icons` baked into their light half; `upgradeUI/details_scrap.png` is
a 24x24 scrap gear.

## How FTL builds its tab row

Background for the tab row of the app, read from the images above:

- The ship menu has no button for the open tab. Each page is one window
  image (`upgrades_main`, `crew_main`, `equipment_main`) whose solid
  trapezoid tab with its icon grows out of a 7 px frame band. The tabs of
  the three pages sit 80 px apart (flat tops at x 31..86, 124..166,
  204..262), so the slants of neighbours meet at the frame.
- The closed tabs are buttons, `tabButtons/<page>_<target>_{on,off,select2}`,
  one image set per pair of pages. Each points at the open tab (`<` right of
  it, `>` left of it); its outer side follows the slant of the window tab.
  The icon is baked in, there is no text. Face rows 7..44, glow to row 47,
  which touches the frame band.
- The options and score screens draw their labels on the image:
  `optionsUI/tab_page_*` and `scoreUI/tab_*` hold no text. The open options
  tab is the taller `_off` image on the frame line
  (`box_options_configure_tab.png` shows a tab fused into a box corner).
- The leftmost tab of the score and ship-menu screens stands with a vertical
  side on the frame edge.

Three variants were rendered on 2026-11-18: A the ship-menu images with
labels instead of icons, B the options `tab_page` images, C flat FTL buttons.
Variant C was chosen, so the tab row is variant C (`app/lib/look/ftl_tabs.dart`):
closed tabs are dark wells (`#292422`) with a 2 px light outline and a light
label, the hovered one turns outline and label yellow, the open tab is a
solid light face with an ink label that runs into the window's top line.
The first tab starts on the straight part of that line, after the cut
corner.

## FTL's upgrade and equipment screens

Read for the Equipment page; variant C of the gallery was chosen (rows with a segment per level) on 2026-11-18:

- `img/upgradeUI/upgrade_system_bar_{on,off,select2,max_on,max_select2,none}.png`
  (66x150) and `upgrade_subsystem_bar_*` (66x113, pilot, sensors, doors,
  battery) are one bar per system. A dark part (rows 2..120, subsystem
  2..83) holds the level cells, a 2 px line at row 121 (84), a light part
  below with a gear at x 7..20 for "can be upgraded"; `max_*` has no gear,
  `none` (not installed) is grey with a dark lower part. The level cells are
  drawn by the game, not images.
- `Equipment/equipment_reactor_{on,off,select2}.png` (272x113) is the
  reactor box: dark part for the reactor bars, line at row 84, the gear
  section from x 204.
- `Equipment/box_{weapons,drones,crew,base}_*.png` (109x76, crew 100x67) are
  the slots of the equipment screen: a round type icon at the top left, the
  art area at rows 10..56, a dark name strip at 58..74. `_on` is light,
  `_off` dark, `_selected` light with a yellow line, `box_overlay_red` marks
  a slot that cannot take the item.
- `Equipment/box_augment_{on,off,selected,off_selected}.png` (235x40) hold
  one augment name each; FTL has three augment slots.
- The green of a powered cell is not in an image; the variants take it from
  the green system icons (`img/icons/s_*_green1.png`, #47F443 to #99FD96).
- Weapon strips stand upright (ion_4 27x54); the variants turn the first
  frame a quarter. Drone sprites are 64x64.

### System icons

`img/icons/` holds four images per system, read on 2026-11-18:

- `s_<id>.png` (48x48): a white disc with the symbol cut out and a white
  glow. FTL loads it, but the HUD and the upgrade screen do not tint it:
  each state has its own image (below).
- `s_<id>_overlay.png` (32x32): the symbol in white with a 1 px black
  contour, no disc. FTL draws it on the room floor. A tint that keeps the
  alpha but replaces every color (`srcIn`) paints the contour over and
  leaves a flat silhouette, which the first variants did; a multiply keeps
  the contour black. `_overlay2` is the symbol outline only.
  `ShipSystem::OnRender` (FTL 1.6.13, read 2026-11-19) draws `_overlay` with
  `GL_RenderPrimitiveWithColor`, a multiply: `COLOR_GRAY` 125,125,125 by
  default, `COLOR_BLUE_ION` 133,231,237 while ion damage locks bars (it
  blinks: alpha level of the animation over 0.5), `COLOR_RED_PLAIN` 255,0,0
  at health 0, `COLOR_ORANGE` 255,153,76 with some bars damaged (the
  color-blind setting swaps in `COLOR_CB_RED` and `COLOR_CB_ORANGE`). The
  same function draws `_overlay2` purple (`COLOR_PURPLE_HACKING`) while
  hacked and yellow when a virtual of the system answers true. The static
  colors are set in a static initializer from 0..255 floats, then scaled by
  1/255; the Java editor's `SystemRoomSprite` tints agree (gray 0.49).
- `s_<id>_<color><n>.png` (64x64, disc of about 26 px inside a glow to the
  edge): the colored disc with the symbol cut out, the icon of the HUD.
  Colors `green`, `grey`, `red`, `orange` with `1` and `2`, `blue1`; `1` has
  a light sheen in its upper right, `2` is flat and lighter.

How FTL composes the icon, read from the macOS binary of FTL 1.6.13
(x86_64, symbols present) on 2026-11-19 with `objdump -d`:

- `ShipSystem::ShipSystem` loads `icons/s_<id>_<color><n>.png` for the static
  `colorNames` = green, grey, orange, red, blue and n = 1, 2 (blue only 1),
  each as its own image primitive; also `s_<id>.png`, `_overlay.png` (the
  room floor) and `_overlay2.png`. There is no tint and no outline layer: the
  outline of the HUD icon is the edge of the disc in the state image itself.
- `ShipSystem::RenderSystemSymbol(bool, int forceColor)` draws exactly one of
  these images. With `forceColor` 0 or 1 it takes that color. Otherwise:
  blue (4) while ioned, red (3) at health 0, orange (2) with some bars
  damaged, grey (1) with no power, else green (0). Frame `2` replaces `1` when
  a virtual of the system answers true, and then FTL draws it three times
  (a brighter glow); which virtual that is was not read.
- `UpgradeBox::OnRender` (the upgrade screen) calls it with `forceColor` 0:
  every installed system is `green1` there. `SystemBox::OnRender` (the HUD)
  passes -1.

The Equipment page draws the 64x64 state images unchanged at their own size
(the glow reaches past the 34 px row), as FTL does: `green1` installed,
`orange1` with some damaged bars, `red1` with all bars damaged, `grey1` not
installed (FTL shows no uninstalled system on the upgrade screen; grey is its
"off" color). Under the pointer the row takes frame `2` of the same color,
because FTL's orange means damage, not hover.

### Room floor and oxygen

FTL draws every room floor itself, tinted by the oxygen of the room. Read
from the macOS binary of FTL 1.6.13 with Ghidra on 2026-11-19:

- `Room::OnRenderFloor` (0x100068890) draws the floor primitive with
  `GL_RenderPrimitiveWithColor` and the color
  `(230 + i, 226 - 2i, 219 - 2i) / 255`, `i = (int)((100 - oxygen) * 0.25)`
  (constants 100.0 at 0x1002b4aa0, 0.25 at 0x1002b4b40); the alpha is the
  floor alpha `Ship::OnRenderFloor` passes (1 unless the ship fades). Full
  oxygen is `#E6E2DB`, the tile color of the Java editor; 0 % is `#FFB0A9`.
- `Room::OnRenderO2Low` (0x100068900) draws a second primitive while the
  oxygen is at most 5.0 (0x1002b41f0): `Room::Room` builds it from
  `effects/low_o2_stripes_1x2.png` (room narrower than 36 px), else `_2x1`
  (lower than 36 px), else `_2x2`, at the top left of the room, alpha 0.5.
- `Ship::OnLoop` copies `ShipManager::GetOxygenLevels` into each room
  (+0xb8); without the oxygen system that list is empty and every room
  reads 0, so such a ship shows red, striped rooms.

The app's `oxygenTint` and `lowOxygenStripes`
(`app/lib/ui/floorplan/floorplan_view.dart`) follow this; the stripes are
clipped to their room.

### Sector map background

`StarMap::StarMap` (FTL 1.6.13 macOS, Ghidra, 2026-11-19) loads
`map/zone_1.png` to `map/zone_3.png` (752 x 534 each, galaxy photos);
`StarMap::OnRender` draws zone `sector % 3 + 1` (zero-based sector) inside
the stencil mask of the map window with `GL_SetColorTint(0.75, 0.75, 0.75,
1)` (constants at 0x1002b5810 and 0x1002b78c0). The Map page draws the same
image over the whole map area (`BoxFit.cover`) under a modulate filter of
`#BFBFBF`; the beacons, joins and the fleet zone lie above it.

## Colors (measured)

| Use | Color | Where measured |
|---|---|---|
| Outline, button face `_on`, light text | `#EBF5E5` | every outline; `button_accept_on` face |
| Button face `_off` | `#A4ABA0` | `button_accept_off` |
| Hover, `_select2` | `#FFE65E` | `button_accept_select2`, `tab_page_select2` |
| Ink of labels on a face | `#193133` | letters of `button_accept_*` |
| Glow around frames | `#79F2FD`, alpha 0x45 next to the line falling to 0x03 | `window_outline`, `top_scrap` |
| Glow under tabs | `#5DB0B7` | `tab_page_*` |
| Window fill | `#7A6463` (mean of the hex pattern, alpha 0xFC) | middle of `window_base` |
| Inner box fill | `#292422`, alpha 0xF6 | `box_crew_details` |
| Dark well | `#090909`, alpha 0xDD | `button_cancel_base` |
| Warning | `#FD5446` | `img/statusUI/top_hull_red_label.png` |
| Pressed button | `#D1C255` | not measured: FTL has no pressed face; the hover face a fifth towards the ink |

The accent is yellow (`#FFE65E`), not orange. A scan of all 364 images of
`img/{customizeUI,upgradeUI,storeUI,systemUI,statusUI}` for opaque orange
pixels (r > 200, 90 < g < 190, b < 90) found them only in the hangar ship
sprites (`miniship_*`): the orange of FTL is the paint of the ships, not of
the UI. Red appears only in warning variants (`*_red.png`).

## How the app uses it

- `ftl_assets::BitmapFont` decodes a `.font`; `atlas_png` answers a white PNG
  whose alpha is the coverage, so the app tints it.
- `ftl_assets::measure_insets` measures a frame; `ftl_app::assets::GameAssets`
  answers `font` and `insets` from its cache; the bridge exposes `game_font`
  and `game_image_insets` next to `game_image`.
- `app/lib/look/` draws text glyph by glyph without smoothing (`FtlText`),
  9-slices a frame image (`FtlFrame`), paints FTL text buttons from the
  measured faces (`FtlButton`) and the area tabs (`FtlTabStrip`).
  `LookMockScreen` is one mock screen in the gallery (`look-ftl`); the shots
  `look-ftl` and `look-ftl-de` come from `test/gallery_shots_test.dart`.
- `app/lib/look/ftl_theme.dart` makes it the look of the real app:
  `FtlTheme` (a `ThemeExtension`), `ftlLookTheme`, and `FtlWindow` (the
  9-slice of `window_outline` with a fill: `FtlColors.window` for the page,
  dialogs and the change sheet, `FtlColors.box` for cards), `FtlBox` (a
  small code-drawn box for tiles and tags), `FtlDialog` and `FtlTag`. The
  shell (`app/lib/shell/ftl_shell.dart`) is tab row C with Expert and Save
  over one window. The resource bar draws each value with its icon (table
  "Resource icons") into one `FtlBox` of `ftlResourceSize`; the `top_*` boxes
  stay in the mock. The shots `ftl-ship`, `ftl-crew`, `ftl-profile`
  and `ftl-changes` (with `-de` twins) show the real shell over mock data.

Limits of the mock: the window fill is the mean color, not the hex pattern
of `window_base`; the font-to-use mapping above is read from the look, not
from the game code.
