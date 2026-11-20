//! The one-click actions with the limits of the open game data: the
//! actions themselves live in `ftl-core` ([`Action::plan`]); this module
//! reads the limits of the ship blueprint from `ftl.dat`, so `ftled action`
//! and the app plan the same patch.

use ftl_assets::GameData;
pub use ftl_core::{Action, ActionNote, ActionPlan, ShipLimits, UnknownAction};
use ftl_core::{SavedGame, SystemKind};

/// The limits of the player ship from its blueprint; empty without game
/// data or for a ship `ftl.dat` does not know.
pub fn limits(game: &SavedGame, data: Option<&GameData>) -> ShipLimits {
    let id = &game.player_ship.blueprint_id;
    let Some(data) = data else {
        return ShipLimits::default();
    };
    ShipLimits {
        max_hull: data
            .ship(id)
            .map(|b| b.hull as i32)
            .filter(|&hull| hull > 0),
        system_levels: SystemKind::ALL
            .into_iter()
            .filter_map(|kind| {
                data.max_system_level(id, kind.id())
                    .map(|level| (kind, level as i32))
            })
            .collect(),
    }
}

/// The patch and notes of `action` on `game` with the limits of `data`.
pub fn plan(action: Action, game: &SavedGame, data: Option<&GameData>) -> ActionPlan {
    action.plan(game, &limits(game, data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ftl_core::{KnownLayouts, SaveCodec};
    use ftl_sav::SavCodec;

    const FIXTURE: &[u8] =
        include_bytes!("../../ftl-sav/tests/fixtures/continue-1.6.13-format11.sav");

    #[test]
    fn without_game_data_the_limits_are_empty_and_the_plan_names_the_caps() {
        let game = SavCodec.decode(FIXTURE, &KnownLayouts::new()).unwrap();
        assert_eq!(limits(&game, None), ShipLimits::default());
        let p = plan(Action::RepairShip, &game, None);
        assert!(p.notes.contains(&ActionNote::HullAtEditorCap { cap: 999 }));
        assert!(p.notes.contains(&ActionNote::RoomsNotDecoded));
    }
}
