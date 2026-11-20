//! The bytes of a saved game, field by field. All ints are little-endian i32.
//!
//! The codec reads in passes, because the later passes need facts
//! from `ftl.dat` that depend on the ship blueprint id read in the first:
//! 1. [`WireSave`]: the file header, the state vars and the player ship up to
//!    its crew list ([`WireShipHead`]). The rest lands in [`WireSave::rest`].
//! 2. [`WireShipSystems`]: reactor, systems and extended system info. The
//!    number of slots per system comes from the ship blueprint.
//! 3. [`WireShipBody`]: rooms to augments (`readShip` in SavedGameParser.java),
//!    read only when the layout of the ship is known.
//! 4. [`WireWorld`]: cargo to distant quests, read only after a body.
//! 5. [`WireSituation`]: the encounter to the end of the file, read only
//!    after a world and only when the layout port knows the nearby ship and
//!    every drone kind with a pod ([`WireSituation::read`]).
//!
//! Everything after the last pass is written back unchanged. One module per
//! area of the file: [`crew`], [`systems`], [`body`], [`world`],
//! [`encounter`], [`projectile`], [`extended`].

pub mod body;
pub mod crew;
pub mod encounter;
pub mod extended;
pub mod projectile;
pub mod systems;
pub mod world;

use std::io::{Read, Seek, SeekFrom, Write};

use binrw::{BinRead, BinResult, BinWrite, Endian, binrw, helpers::until_eof};

pub use body::*;
pub use crew::*;
pub use encounter::*;
pub use extended::*;
pub use projectile::*;
pub use systems::*;
pub use world::*;

pub const KNOWN_FORMATS: [i32; 5] = [2, 7, 8, 9, 11];

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireSave {
    #[br(assert(KNOWN_FORMATS.contains(&format), "unsupported file format {}; known: 2, 7, 8, 9, 11", format))]
    pub format: i32,
    #[br(if(format == 11))]
    #[bw(if(*format == 11))]
    pub random_native: Option<Bool32>,
    #[br(if(format >= 7))]
    #[bw(if(*format >= 7))]
    pub dlc_enabled: Option<Bool32>,
    pub difficulty: i32,
    pub ships_defeated: i32,
    pub beacons_explored: i32,
    pub scrap_collected: i32,
    pub crew_hired: i32,
    #[br(args(format >= 11))]
    #[bw(args(*format >= 11))]
    pub player_ship_name: FtlString,
    #[br(args(format >= 11))]
    #[bw(args(*format >= 11))]
    pub player_ship_blueprint_id: FtlString,
    pub sector_number: i32,
    pub victory: i32,
    #[br(temp)]
    #[bw(calc = state_vars.len() as u32)]
    state_var_count: u32,
    #[br(count = state_var_count, args { inner: (format >= 11,) })]
    #[bw(args(*format >= 11))]
    pub state_vars: Vec<WireStateVar>,
    #[br(args(format))]
    #[bw(args(*format))]
    pub player_ship: WireShipHead,
    #[br(parse_with = until_eof)]
    pub rest: Vec<u8>,
}

#[binrw]
#[brw(little, import(utf8: bool))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireStateVar {
    #[brw(args(utf8))]
    pub id: FtlString,
    pub value: i32,
}

/// The player ship from its blueprint id to its crew list.
#[binrw]
#[brw(little, import(format: i32))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireShipHead {
    #[brw(args(format >= 11))]
    pub blueprint_id: FtlString,
    #[brw(args(format >= 11))]
    pub name: FtlString,
    #[brw(args(format >= 11))]
    pub gfx_base_name: FtlString,
    #[br(temp)]
    #[bw(calc = starting_crew.len() as u32)]
    starting_crew_count: u32,
    #[br(count = starting_crew_count, args { inner: (format >= 11,) })]
    #[bw(args(format >= 11))]
    pub starting_crew: Vec<WireStartingCrew>,
    #[brw(if(format >= 7))]
    pub jump: Option<WireJumpState>,
    pub hull: i32,
    pub fuel: i32,
    pub drone_parts: i32,
    pub missiles: i32,
    pub scrap: i32,
    #[br(temp)]
    #[bw(calc = crew.len() as u32)]
    crew_count: u32,
    #[br(count = crew_count, args { inner: (format,) })]
    #[bw(args(format))]
    pub crew: Vec<WireCrewMember>,
}

#[binrw]
#[brw(little, import(utf8: bool))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireStartingCrew {
    #[brw(args(utf8))]
    pub race: FtlString,
    #[brw(args(utf8))]
    pub name: FtlString,
}

/// Format 7 and later.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireJumpState {
    pub hostile: Bool32,
    pub jump_charge_ticks: i32,
    pub jumping: Bool32,
    pub jump_anim_ticks: i32,
}

/// A bool stored as an i32 that must be 0 or 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bool32(pub bool);

impl BinRead for Bool32 {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(r: &mut R, endian: Endian, _: ()) -> BinResult<Self> {
        let pos = r.stream_position()?;
        match i32::read_options(r, endian, ())? {
            0 => Ok(Bool32(false)),
            1 => Ok(Bool32(true)),
            v => Err(binrw::Error::AssertFail {
                pos,
                message: format!("expected a bool (0 or 1), found {v}"),
            }),
        }
    }
}

impl BinWrite for Bool32 {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(&self, w: &mut W, endian: Endian, _: ()) -> BinResult<()> {
        i32::from(self.0).write_options(w, endian, ())
    }
}

/// A string: an i32 byte length, then the bytes.
///
/// Format 11 (FTL 1.6.1+) stores UTF-8. Older formats store one byte per
/// character; this crate maps those bytes to and from Latin-1, which is
/// lossless for every byte value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FtlString(pub String);

impl BinRead for FtlString {
    type Args<'a> = (bool,);

    fn read_options<R: Read + Seek>(
        r: &mut R,
        endian: Endian,
        (utf8,): (bool,),
    ) -> BinResult<Self> {
        let pos = r.stream_position()?;
        let len = i32::read_options(r, endian, ())?;
        let start = r.stream_position()?;
        let end = r.seek(SeekFrom::End(0))?;
        r.seek(SeekFrom::Start(start))?;
        if len < 0 || start + len as u64 > end {
            return Err(binrw::Error::AssertFail {
                pos,
                message: format!(
                    "string length {len} does not fit the {} bytes left in the file",
                    end - start
                ),
            });
        }
        let mut buf = vec![0u8; len as usize];
        r.read_exact(&mut buf)?;
        if utf8 {
            String::from_utf8(buf)
                .map(FtlString)
                .map_err(|e| binrw::Error::AssertFail {
                    pos,
                    message: format!("string is not valid UTF-8: {e}"),
                })
        } else {
            Ok(FtlString(buf.into_iter().map(char::from).collect()))
        }
    }
}

impl BinWrite for FtlString {
    type Args<'a> = (bool,);

    fn write_options<W: Write + Seek>(
        &self,
        w: &mut W,
        endian: Endian,
        (utf8,): (bool,),
    ) -> BinResult<()> {
        let bytes: Vec<u8> = if utf8 {
            self.0.as_bytes().to_vec()
        } else {
            // A char above U+00FF has no single-byte form; FTL before 1.6.1
            // cannot show it either.
            self.0
                .chars()
                .map(|c| u8::try_from(c).unwrap_or(b'?'))
                .collect()
        };
        (bytes.len() as i32).write_options(w, endian, ())?;
        bytes.write_options(w, endian, ())
    }
}
