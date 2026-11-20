//! Codec for FTL saved games (`continue.sav`).
//!
//! Two layers:
//! - [`wire`] mirrors the bytes of the file with `binrw`, one struct field per
//!   field on disk, so reading and writing come from one declaration.
//! - [`map`] converts the wire structs to the domain model of `ftl-core` and
//!   back: string encodings, the difficulty flag, format-dependent fields.
//!
//! The reference for the layout is `SavedGameParser.java` of Vhati's FTL
//! Profile Editor (<https://github.com/Vhati/ftl-profile-editor>).

pub mod map;
pub mod profile;
pub mod wire;

pub use profile::ProfCodec;

use std::io::Cursor;

use binrw::{BinRead, BinWrite};
use ftl_core::{DecodeError, SaveCodec, SavedGame, ShipLayouts, ShipShape, SystemKind};

/// The [`SaveCodec`] for every file format this crate knows.
#[derive(Debug, Default, Clone, Copy)]
pub struct SavCodec;

impl SaveCodec for SavCodec {
    fn decode(&self, bytes: &[u8], layouts: &dyn ShipLayouts) -> Result<SavedGame, DecodeError> {
        let mut head = wire::WireSave::read(&mut Cursor::new(bytes)).map_err(|e| DecodeError {
            offset: error_offset(&e),
            field: "saved game",
            problem: e.to_string(),
        })?;
        let format = head.format;
        let shape = layouts.ship(&head.player_ship.blueprint_id.0);
        let kinds = system_kinds(format, shape.as_ref());

        // Passes 2 and 3 read the bytes after the head; offsets in errors
        // count from the start of the file.
        let tail = std::mem::take(&mut head.rest);
        let base = (bytes.len() - tail.len()) as u64;
        let at = |field: &'static str| {
            move |e: binrw::Error| DecodeError {
                offset: base as usize + error_offset(&e),
                field,
                problem: e.to_string(),
            }
        };
        let mut r = Cursor::new(&tail[..]);
        let systems = wire::WireShipSystems::read_args(&mut r, (format, kinds.clone()))
            .map_err(at("player ship systems"))?;
        let body = match shape {
            Some(shape) => {
                let rooms = shape
                    .rooms
                    .iter()
                    .map(|r| (r.squares_h, r.squares_v))
                    .collect();
                let b = wire::WireShipBody::read_args(&mut r, (format, rooms, shape.door_count))
                    .map_err(at("player ship rooms to augments"))?;
                Some((b, shape))
            }
            None => None,
        };
        // The world follows the body on disk, so it needs the body.
        let world = match body {
            Some(_) => Some(
                wire::WireWorld::read_args(&mut r, (format,))
                    .map_err(at("cargo, sector map and beacons"))?,
            ),
            None => None,
        };
        // The situation follows the world. It needs the layout of the
        // nearby ship and the drone kinds; without them it stays undecoded.
        let situation = match (&body, &world) {
            (Some((b, _)), Some(_)) => {
                let mark = r.position();
                let ship = |id: &str| {
                    let shape = layouts.ship(id)?;
                    Some((system_kinds(format, Some(&shape)), shape))
                };
                let drone_kind = |id: &str| layouts.drone_kind(id);
                let facts = wire::SituationFacts {
                    ship: &ship,
                    drone_kind: &drone_kind,
                    player: wire::ExtendedShape::of(&systems, &kinds, b, &drone_kind),
                };
                let s = wire::WireSituation::read(&mut r, format, &facts)
                    .map_err(at("encounter, nearby ship, projectiles and flagship"))?;
                if s.is_none() {
                    r.set_position(mark);
                }
                s
            }
            _ => None,
        };
        let rest = tail[r.position() as usize..].to_vec();
        map::to_domain(map::WireGame {
            head,
            systems,
            kinds,
            body,
            world,
            situation,
            rest,
        })
    }

    fn encode(&self, game: &SavedGame) -> Vec<u8> {
        let parts = map::to_wire(game);
        let f = game.file_format;
        let mut out = Cursor::new(Vec::with_capacity(game.undecoded.len() + 4096));
        let infallible = "writing to a Vec cannot fail and every domain value has a wire form";
        parts.head.write(&mut out).expect(infallible);
        parts.systems.write_args(&mut out, (f,)).expect(infallible);
        if let Some(b) = parts.body {
            b.write_args(&mut out, (f,)).expect(infallible);
        }
        if let Some(w) = parts.world {
            w.write_args(&mut out, (f,)).expect(infallible);
        }
        if let Some(s) = parts.situation {
            s.write(&mut out, f).expect(infallible);
        }
        let mut bytes = out.into_inner();
        bytes.extend_from_slice(&game.undecoded);
        bytes
    }
}

/// The system kind index of each system slot, in file order: one slot per
/// system of the format, more where the ship blueprint places a system in
/// several rooms. Without a known ship, one slot per system.
fn system_kinds(format: i32, shape: Option<&ShipShape>) -> Vec<usize> {
    (0..wire::system_count(format))
        .flat_map(|i| {
            let slots = shape.map_or(1, |s| s.system_slots(SystemKind::ALL[i]));
            std::iter::repeat_n(i, slots)
        })
        .collect()
}

pub(crate) fn error_offset(e: &binrw::Error) -> usize {
    use binrw::Error::*;
    let pos = match e {
        BadMagic { pos, .. } | AssertFail { pos, .. } | Custom { pos, .. } => *pos,
        NoVariantMatch { pos } | EnumErrors { pos, .. } => *pos,
        Backtrace(bt) => return error_offset(&bt.error),
        _ => 0,
    };
    pos as usize
}
