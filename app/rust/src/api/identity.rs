//! The identity of the open save for the ship card: ship name, ship type,
//! difficulty and sector. A thin adapter over `ftl_core::identity` and
//! `ftl_app::identity`; edits go through [`SaveSession::apply`] with the text
//! paths `ship.name`, `nearby.ship.name`, `ship.type` and the number path
//! `difficulty`. The sector is read only.

use flutter_rust_bridge::frb;
use ftl_app::identity::{self as app_identity, difficulty_block};
use ftl_app::{Field, assets};

use super::editor::{EditorError, FieldValue, SaveSession};
use super::field::ship;

/// Name, type, difficulty and sector of one ship of the save.
pub struct ShipIdentity {
    /// `ship.name` or `nearby.ship.name`, a text path.
    pub name_path: String,
    pub name: String,
    /// `ship.type`, a text path; `nearby.ship.type` is always refused.
    pub type_path: String,
    pub blueprint_id: String,
    /// `difficulty`: 0 Easy, 1 Normal, 2 Hard (from format 7); min equals
    /// max when it stays (`difficulty_block` says why).
    pub difficulty: FieldValue,
    /// Why the difficulty stays, a stable code of
    /// `ftl_core::identity::DifficultyBlock` (`flagship`, `not_decoded`).
    pub difficulty_block: Option<String>,
    /// The one-based sector number; read only, it follows the sector tree.
    pub sector: i32,
}

/// The identity of the player ship, or with `nearby` of the ship at the
/// beacon (difficulty and sector are the game's).
#[frb(sync)]
pub fn identity_of(session: &SaveSession, nearby: bool) -> Result<ShipIdentity, EditorError> {
    let g = session.file.game();
    let on = ship(nearby);
    let name = Field::ShipName.on(on);
    let kind = Field::ShipType.on(on);
    let value = Field::Difficulty.get(g)?;
    let (min, max) = Field::Difficulty.range(g).unwrap_or((value, value));
    Ok(ShipIdentity {
        name_path: name.path(),
        name: name.value(g)?.to_string(),
        type_path: kind.path(),
        blueprint_id: kind.value(g)?.to_string(),
        difficulty: FieldValue {
            path: Field::Difficulty.path(),
            value,
            min,
            max,
        },
        difficulty_block: difficulty_block(g).map(|b| b.code().to_owned()),
        sector: Field::Sector.get(g)?,
    })
}

/// A hangar ship as a type of the player ship.
pub struct ShipTypeChoice {
    pub id: String,
    /// The ship name in the language.
    pub name: String,
    /// `A`, `B` or `C`.
    pub layout: String,
    /// Why the ship cannot take it, a stable code of
    /// `ftl_core::identity::TypeBlock` (`other_layout`, `system_room`, ...);
    /// `None` when it can.
    pub block: Option<String>,
    /// The system id a `system_room` or `start_system` block names.
    pub system: Option<String>,
}

/// The hangar ships as types of the player ship, named from the open
/// `ftl.dat`; empty without it.
#[frb(sync)]
pub fn ship_type_choices(session: &SaveSession, language: String) -> Vec<ShipTypeChoice> {
    let data = assets::shared();
    app_identity::ship_type_choices(
        session.file.game(),
        data.as_ref().map(|a| a.data().as_ref()),
        session.file.layouts(),
        &language,
    )
    .into_iter()
    .map(|c| ShipTypeChoice {
        id: c.id,
        name: c.name,
        layout: c.layout.to_string(),
        block: c.block.map(|b| b.code().to_owned()),
        system: c.block.and_then(|b| b.system()).map(|k| k.id().to_owned()),
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::editor::{FieldEdit, TextEdit, open_fixture};

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ftl-sav/tests/fixtures/continue-1.6.13-format11.sav"
    );

    #[test]
    fn the_identity_of_the_fixture_reaches_the_app_and_its_edits_apply() {
        let mut session = open_fixture(FIXTURE);
        let i = identity_of(&session, false).unwrap();
        assert_eq!(i.name_path, "ship.name");
        assert_eq!(i.name, "The Kestrel");
        assert_eq!(i.blueprint_id, "PLAYER_SHIP_HARD");
        assert_eq!(
            (i.difficulty.value, i.difficulty.min, i.difficulty.max),
            (0, 0, 2)
        );
        assert_eq!(i.difficulty_block, None);
        assert_eq!(i.sector, 1);
        let n = identity_of(&session, true).unwrap();
        assert_eq!(n.name_path, "nearby.ship.name");
        assert_eq!(n.blueprint_id, "JELLY_BUTTON_P_DLC");

        session
            .apply(
                vec![FieldEdit {
                    path: "difficulty".into(),
                    value: 2,
                }],
                vec![TextEdit {
                    path: "ship.name".into(),
                    text: "Red Dwarf".into(),
                }],
            )
            .unwrap();
        let i = identity_of(&session, false).unwrap();
        assert_eq!((i.name.as_str(), i.difficulty.value), ("Red Dwarf", 2));
        assert_eq!(session.summary().ship_name, "Red Dwarf");
    }
}
