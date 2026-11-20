//! The achievements of the game for the profile: id, name and description
//! in the app language, kind and ship, from `ftl.dat` (`GameData`).

pub use ftl_assets::AchievementKind;
use ftl_assets::GameData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AchievementEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: AchievementKind,
    /// The player ship of a ship, quest or victory achievement.
    pub ship: Option<String>,
}

/// Every achievement of the game in the order of `ftl.dat`, texts in
/// `language` (`de`) else English; empty without game data.
pub fn catalog(data: Option<&GameData>, language: &str) -> Vec<AchievementEntry> {
    let Some(data) = data else {
        return Vec::new();
    };
    data.achievements()
        .iter()
        .map(|a| {
            let (name, description) = data
                .achievement_text_in(&a.id, language)
                .unwrap_or((&a.name, &a.description));
            AchievementEntry {
                id: a.id.clone(),
                name: name.to_owned(),
                description: description.to_owned(),
                kind: a.kind,
                ship: a.ship.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_game_data_the_catalog_is_empty() {
        assert!(catalog(None, "en").is_empty());
    }
}
