//! Ports the application layer needs from the outside.

use crate::layout::ShipLayouts;
use crate::model::SavedGame;
use crate::profile::Profile;

/// Turns the bytes of a `continue.sav` into a [`SavedGame`] and back.
///
/// Contract: for every input that `decode` accepts and every `layouts`,
/// `encode(decode(bytes, layouts)) == bytes`. `layouts` decides how far the
/// codec decodes: a ship it does not know keeps its rooms and everything
/// after them in [`SavedGame::undecoded`].
pub trait SaveCodec {
    fn decode(&self, bytes: &[u8], layouts: &dyn ShipLayouts) -> Result<SavedGame, DecodeError>;
    fn encode(&self, game: &SavedGame) -> Vec<u8>;
}

/// Turns the bytes of an `ae_prof.sav` (or legacy `prof.sav`) into a
/// [`Profile`] and back, with the same contract as [`SaveCodec`].
pub trait ProfileCodec {
    fn decode(&self, bytes: &[u8]) -> Result<Profile, DecodeError>;
    fn encode(&self, profile: &Profile) -> Vec<u8>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{field} at offset {offset} (0x{offset:X}): {problem}")]
pub struct DecodeError {
    /// Byte offset where the failing read started.
    pub offset: usize,
    /// Name of the field the codec tried to read.
    pub field: &'static str,
    pub problem: String,
}
