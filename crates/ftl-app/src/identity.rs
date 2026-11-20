//! The ship types the player ship can take (`ship.type`), named from
//! `ftl.dat`: the hangar ships in hangar order, each with the reason it is
//! refused ([`ftl_core::identity::TypeBlock`]), or none.

use ftl_assets::GameData;
pub use ftl_core::identity::{
    DifficultyBlock, MAX_SHIP_NAME, TypeBlock, difficulty_block, type_block,
};
use ftl_core::{SavedGame, ShipId, ShipLayouts};

/// One hangar ship as a type of the player ship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipTypeChoice {
    /// The ship blueprint id, the value of `ship.type`.
    pub id: String,
    /// The ship name of the blueprint in the language, else its id.
    pub name: String,
    /// The layout of the hangar: `A`, `B` (`_2`) or `C` (`_3`).
    pub layout: char,
    /// Why the player ship cannot take this type; `None` when it can (also
    /// for its own type).
    pub block: Option<TypeBlock>,
}

/// The hangar ships of `data` as types of the player ship of `game`;
/// empty without game data.
pub fn ship_type_choices(
    game: &SavedGame,
    data: Option<&GameData>,
    layouts: &dyn ShipLayouts,
    language: &str,
) -> Vec<ShipTypeChoice> {
    let Some(data) = data else {
        return Vec::new();
    };
    data.player_ships()
        .into_iter()
        .map(|b| ShipTypeChoice {
            id: b.id.clone(),
            name: data
                .display_name_in(&b.id, language)
                .filter(|n| !n.is_empty())
                .unwrap_or(&b.id)
                .to_owned(),
            layout: layout_letter(&b.id),
            block: type_block(game, ShipId::Player, &b.id, layouts),
        })
        .collect()
}

/// `A`, `B` or `C` from the id suffix of a hangar ship.
fn layout_letter(id: &str) -> char {
    if id.ends_with("_2") {
        'B'
    } else if id.ends_with("_3") {
        'C'
    } else {
        'A'
    }
}
