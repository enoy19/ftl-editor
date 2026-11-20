//! Ship layouts: `data/<layout>.txt` (rooms, doors, offsets; `DatParser.readLayout`)
//! plus the hull image rectangle and floor offset from `data/<layout>.xml`.

use crate::Error;
use crate::xml::{self, ChassisXml};

/// All positions are in FTL's units: room and door coordinates in squares
/// of 35 pixels, offsets in squares, image and floor offsets in pixels.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShipLayout {
    pub offset_x: i32,
    pub offset_y: i32,
    pub horizontal: i32,
    pub vertical: i32,
    /// Shield ellipse `[w, h, x, y]`.
    pub shield_ellipse: Option<[i32; 4]>,
    pub rooms: Vec<ShipLayoutRoom>,
    pub doors: Vec<ShipLayoutDoor>,
    /// Hull image rectangle `[x, y, w, h]` from the `.xml` file.
    pub image: Option<[i32; 4]>,
    /// Floor image offset `[x, y]` from the `.xml` file.
    pub floor_offset: Option<[i32; 2]>,
    /// The weapon mounts of the `.xml` file in file order: weapon slot `n`
    /// sits on mount `n` (FTL's `Ship::GetWeaponMount`); artillery mounts
    /// follow the slots.
    pub weapon_mounts: Vec<WeaponMount>,
}

/// Where FTL draws a weapon on the hull (`<mount>` of `data/<layout>.xml`,
/// read by `Ship::OnInit`, FTL 1.6.13): `x`, `y` in pixels from the top left
/// of the hull image, `rotate` turns the art a quarter clockwise,
/// `mirror` flips it, `slide` is the way it moves out when powered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponMount {
    pub x: i32,
    pub y: i32,
    pub rotate: bool,
    pub mirror: bool,
    pub slide: Slide,
}

/// The direction a weapon slides out when powered (`Globals::GetDirection`:
/// up 2, down 0, left 3, right 1, any other text 4). An unpowered weapon
/// sits half its length back into the hull.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slide {
    Up,
    Down,
    Left,
    Right,
    No,
}

impl Slide {
    /// `Globals::GetDirection`; a mount without the attribute does not slide.
    pub fn parse(text: Option<&str>) -> Slide {
        match text {
            Some("up") => Slide::Up,
            Some("down") => Slide::Down,
            Some("left") => Slide::Left,
            Some("right") => Slide::Right,
            _ => Slide::No,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipLayoutRoom {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// A door on the wall at square `(x, y)` between two rooms; `-1` is space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipLayoutDoor {
    pub x: i32,
    pub y: i32,
    pub room_a: i32,
    pub room_b: i32,
    pub vertical: bool,
}

/// Parses the `.txt` layout: a keyword line followed by one number per line.
/// Lines are trimmed (some files end lines with `\r\n`) and empty lines are
/// skipped everywhere; the Java reader skips them only before a keyword.
/// Unknown keywords are skipped, like in the Java reader.
pub(crate) fn parse_txt(inner_path: &str, bytes: &[u8]) -> Result<ShipLayout, Error> {
    let text = String::from_utf8_lossy(bytes);
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let mut layout = ShipLayout::default();
    while let Some(keyword) = lines.next() {
        let mut num = |what: &str| -> Result<i32, Error> {
            let line = lines.next().unwrap_or("");
            line.parse().map_err(|_| Error::Parse {
                inner_path: inner_path.to_owned(),
                detail: format!("{keyword}: {what} is {line:?}, not a number"),
            })
        };
        match keyword {
            "X_OFFSET" => layout.offset_x = num("value")?,
            "Y_OFFSET" => layout.offset_y = num("value")?,
            "HORIZONTAL" => layout.horizontal = num("value")?,
            "VERTICAL" => layout.vertical = num("value")?,
            "ELLIPSE" => layout.shield_ellipse = Some([num("w")?, num("h")?, num("x")?, num("y")?]),
            "ROOM" => {
                let (id, x, y, w, h) = (num("id")?, num("x")?, num("y")?, num("w")?, num("h")?);
                layout.rooms.push(ShipLayoutRoom {
                    id: id as u32,
                    x,
                    y,
                    w: w as u32,
                    h: h as u32,
                });
            }
            "DOOR" => {
                let (x, y, room_a, room_b, vertical) = (
                    num("x")?,
                    num("y")?,
                    num("room a")?,
                    num("room b")?,
                    num("vertical")?,
                );
                layout.doors.push(ShipLayoutDoor {
                    x,
                    y,
                    room_a,
                    room_b,
                    vertical: vertical != 0,
                });
            }
            _ => {}
        }
    }
    Ok(layout)
}

pub(crate) fn add_chassis(
    layout: &mut ShipLayout,
    inner_path: &str,
    bytes: &[u8],
) -> Result<(), Error> {
    let chassis: ChassisXml = xml::parse(inner_path, bytes)?;
    layout.image = chassis.img.map(|r| [r.x, r.y, r.w, r.h]);
    layout.floor_offset = chassis.offsets.and_then(|o| o.floor).map(|p| [p.x, p.y]);
    layout.weapon_mounts = chassis
        .weapon_mounts
        .map(|m| m.mounts)
        .unwrap_or_default()
        .into_iter()
        .map(|m| WeaponMount {
            x: m.x,
            y: m.y,
            rotate: m.rotate == "true",
            mirror: m.mirror == "true",
            slide: Slide::parse(m.slide.as_deref()),
        })
        .collect();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bad_number_names_the_keyword_and_the_line() {
        let err = parse_txt("data/x.txt", b"ROOM\n0\nzwei\n")
            .unwrap_err()
            .to_string();
        assert_eq!(err, "data/x.txt: ROOM: x is \"zwei\", not a number");
    }

    #[test]
    fn mounts_keep_file_order_and_ftls_reading_of_their_attributes() {
        let mut l = ShipLayout::default();
        add_chassis(
            &mut l,
            "data/x.xml",
            br#"<img x="-59" y="-103" w="652" h="418"/>
<weaponMounts>
  <mount x="540" y="155" rotate="true" mirror="true" gib="6" slide="up"/>
  <!-- a comment between mounts -->
  <mount x="52" y="136" rotate="false" mirror="false" slide="left"/>
  <mount x="1" y="2" rotate="TRUE" mirror="yes"/>
</weaponMounts>"#,
        )
        .unwrap();
        assert_eq!(
            l.weapon_mounts,
            [
                WeaponMount {
                    x: 540,
                    y: 155,
                    rotate: true,
                    mirror: true,
                    slide: Slide::Up
                },
                WeaponMount {
                    x: 52,
                    y: 136,
                    rotate: false,
                    mirror: false,
                    slide: Slide::Left
                },
                WeaponMount {
                    x: 1,
                    y: 2,
                    rotate: false,
                    mirror: false,
                    slide: Slide::No
                },
            ]
        );
    }

    #[test]
    fn crlf_and_blank_lines_are_tolerated() {
        let l = parse_txt(
            "x",
            b"X_OFFSET\r\n\r\n3\r\nUNKNOWN\r\nDOOR\r\n1\r\n2\r\n-1\r\n0\r\n0\r\n",
        )
        .unwrap();
        assert_eq!(l.offset_x, 3);
        assert_eq!(l.doors[0].room_a, -1);
        assert!(!l.doors[0].vertical);
    }
}
