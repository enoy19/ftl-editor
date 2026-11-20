//! Codec for FTL profiles (`ae_prof.sav`, format 9, and legacy `prof.sav`,
//! format 4), the adapter for the ftl-core [`ProfileCodec`] port.
//!
//! The same two layers as the saved-game codec: [`wire`] mirrors the bytes
//! with binrw, [`map`] converts to the domain. The reference for the layout
//! is `ProfileParser.java` of the Java editor.

pub mod map;
pub mod wire;

use std::io::Cursor;

use binrw::{BinRead, BinWrite};
use ftl_core::profile::Profile;
use ftl_core::{DecodeError, ProfileCodec};

/// The [`ProfileCodec`] for formats 4 and 9.
#[derive(Debug, Default, Clone, Copy)]
pub struct ProfCodec;

impl ProfileCodec for ProfCodec {
    fn decode(&self, bytes: &[u8]) -> Result<Profile, DecodeError> {
        let wire = wire::WireProfile::read(&mut Cursor::new(bytes)).map_err(|e| DecodeError {
            offset: crate::error_offset(&e),
            field: "profile",
            problem: e.to_string(),
        })?;
        Ok(map::to_domain(wire))
    }

    fn encode(&self, profile: &Profile) -> Vec<u8> {
        let wire = map::to_wire(profile);
        let mut out = Cursor::new(Vec::with_capacity(1024));
        wire.write(&mut out)
            .expect("writing to a Vec cannot fail and every domain value has a wire form");
        out.into_inner()
    }
}
