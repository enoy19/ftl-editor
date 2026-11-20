//! The player profile (`ae_prof.sav`) as the Flutter app sees it. A thin
//! adapter over `ftl_app::ProfileFile`: flat DTOs and an opaque handle. The
//! edit rules live in `ftl_core::profile`, verify and backup in `ftl-app`.
//!
//! The app stages profile edits as `ProfileEdit`s (paths of
//! `ftl_core::profile::ProfileField`) and shows them through `preview`;
//! `save` applies them and writes. Ship and achievement names come from the
//! open `ftl.dat` (`ftl_app::assets::shared`); without it a ship has no name
//! and the catalog of achievements is empty.

use flutter_rust_bridge::frb;
use ftl_app::achievements::{self, AchievementKind};
use ftl_app::profile::{PLAYER_SHIPS, unlock_all_patch};
use ftl_app::{Profile, ProfileFile, ProfilePatch, SaveOptions};

use super::editor::{EditorError, SaveResult};

/// One vanilla player ship of the profile with the unlock state of its
/// three layouts.
pub struct ProfileShip {
    /// Blueprint id of Type-A, for example `PLAYER_SHIP_HARD`. Its hull is
    /// `GameImageKind::Ship` with this id.
    pub id: String,
    /// The ship name of `ftl.dat`, `None` without open game data.
    pub name: Option<String>,
    pub layout_a: bool,
    /// Type-B: two of the ship's three achievements are present.
    pub layout_b: bool,
    pub layout_c: bool,
    /// Lanius and Crystal have no Type-C, and format 4 stores none.
    pub has_layout_c: bool,
    /// The three achievements of this ship; two unlock Type-B.
    pub achievements: Vec<String>,
}

/// One achievement of the profile. Its image is `GameImageKind::Achievement`
/// with [`ProfileAchievement::id`]; its name comes from [`achievement_catalog`].
pub struct ProfileAchievement {
    pub id: String,
    /// `easy`, `normal` or `hard`.
    pub difficulty: String,
    /// Victory achievements in format 9 only: for layouts A, B, C the highest
    /// difficulty won with, `None` for a layout without a win.
    pub victory_layouts: Option<Vec<Option<String>>>,
}

pub struct ProfileScore {
    pub ship_name: String,
    pub ship_id: String,
    pub value: i32,
    pub sector: i32,
    pub victory: bool,
    /// `easy`, `normal` or `hard`.
    pub difficulty: String,
}

pub struct ProfileStats {
    pub most_ships_defeated: i32,
    pub total_ships_defeated: i32,
    pub most_beacons_explored: i32,
    pub total_beacons_explored: i32,
    pub most_scrap_collected: i32,
    pub total_scrap_collected: i32,
    pub most_crew_hired: i32,
    pub total_crew_hired: i32,
    pub total_games_played: i32,
    pub total_victories: i32,
}

pub struct ProfileSummary {
    pub path: String,
    pub file_format: i32,
    /// The vanilla ships the file has a slot for, in hangar order.
    pub ships: Vec<ProfileShip>,
    pub achievements: Vec<ProfileAchievement>,
    pub top_scores: Vec<ProfileScore>,
    pub stats: ProfileStats,
    pub dirty: bool,
}

/// One staged profile edit: `ships.<SHIP_ID>.<a|b|c>` = 0 or 1, or
/// `achievements.<ID>` = 0 (absent), 1 Easy, 2 Normal, 3 Hard.
#[derive(Debug, Clone)]
pub struct ProfileEdit {
    pub path: String,
    pub value: i32,
}

/// What one staged edit changes. A Type-B edit names the ship achievements
/// it adds (on Easy) or removes.
pub struct ProfileEditChange {
    pub path: String,
    pub old: i32,
    /// The value after the edit (`new` is a keyword in Dart).
    pub value: i32,
    pub added_achievements: Vec<String>,
    pub removed_achievements: Vec<String>,
}

/// The profile as the staged edits would leave it, and what each edit changes.
pub struct ProfilePreview {
    pub summary: ProfileSummary,
    pub changes: Vec<ProfileEditChange>,
}

/// One achievement of the game, for names and the Add picker.
pub struct AchievementInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    /// `general`, `ship`, `quest` or `victory`.
    pub kind: String,
    /// The player ship of a ship, quest or victory achievement.
    pub ship_id: Option<String>,
}

/// An opened profile. Dart holds it as an opaque handle.
#[frb(opaque)]
pub struct ProfileSession {
    file: ProfileFile,
}

fn ship_name(id: &str, language: &str) -> Option<String> {
    let assets = ftl_app::assets::shared()?;
    assets
        .data()
        .display_name_in(id, language)
        .map(str::to_owned)
}

fn difficulty(d: impl std::fmt::Debug) -> String {
    format!("{d:?}").to_lowercase()
}

fn patch(edits: Vec<ProfileEdit>) -> Result<ProfilePatch, EditorError> {
    let assignments = edits
        .into_iter()
        .map(|e| {
            Ok(ftl_app::ProfileAssignment {
                field: e.path.parse().map_err(ftl_app::AppError::from)?,
                value: e.value,
            })
        })
        .collect::<Result<Vec<_>, EditorError>>()?;
    Ok(ProfilePatch::new(assignments))
}

fn summary_of(p: &Profile, path: String, dirty: bool, language: &str) -> ProfileSummary {
    let s = &p.stats;
    ProfileSummary {
        path,
        file_format: p.file_format,
        ships: PLAYER_SHIPS
            .iter()
            .filter_map(|ship| {
                let u = p.ship_unlocks(ship.id)?;
                Some(ProfileShip {
                    id: ship.id.to_owned(),
                    name: ship_name(ship.id, language),
                    layout_a: u.layout_a,
                    layout_b: u.layout_b,
                    layout_c: u.layout_c,
                    has_layout_c: ship.has_layout_c && p.file_format >= 9,
                    achievements: ship.achievements.map(str::to_owned).to_vec(),
                })
            })
            .collect(),
        achievements: p
            .achievements
            .iter()
            .map(|a| ProfileAchievement {
                id: a.id.clone(),
                difficulty: difficulty(a.difficulty),
                victory_layouts: a
                    .layouts
                    .map(|l| l.iter().map(|d| d.map(difficulty)).collect()),
            })
            .collect(),
        top_scores: p
            .top_scores
            .iter()
            .map(|t| ProfileScore {
                ship_name: t.ship_name.clone(),
                ship_id: t.ship_id.clone(),
                value: t.value,
                sector: t.sector,
                victory: t.victory,
                difficulty: difficulty(t.difficulty),
            })
            .collect(),
        stats: ProfileStats {
            most_ships_defeated: s.most_ships_defeated,
            total_ships_defeated: s.total_ships_defeated,
            most_beacons_explored: s.most_beacons_explored,
            total_beacons_explored: s.total_beacons_explored,
            most_scrap_collected: s.most_scrap_collected,
            total_scrap_collected: s.total_scrap_collected,
            most_crew_hired: s.most_crew_hired,
            total_crew_hired: s.total_crew_hired,
            total_games_played: s.total_games_played,
            total_victories: s.total_victories,
        },
        dirty,
    }
}

impl ProfileSession {
    /// The file as it is; ship names in `language` (`en`, `de`).
    #[frb(sync)]
    pub fn summary(&self, language: String) -> ProfileSummary {
        let path = self.file.path().display().to_string();
        summary_of(self.file.profile(), path, self.file.is_dirty(), &language)
    }

    /// The profile as `edits` would leave it, without changing the file,
    /// and one change per edit. An invalid edit answers the code `edit`.
    #[frb(sync)]
    pub fn preview(
        &self,
        edits: Vec<ProfileEdit>,
        language: String,
    ) -> Result<ProfilePreview, EditorError> {
        let (profile, changes) = self.file.preview(&patch(edits)?)?;
        let path = self.file.path().display().to_string();
        let dirty = !changes.is_empty() || self.file.is_dirty();
        Ok(ProfilePreview {
            summary: summary_of(&profile, path, dirty, &language),
            changes: changes
                .into_iter()
                .map(|c| ProfileEditChange {
                    path: c.field.path(),
                    old: c.old,
                    value: c.new,
                    added_achievements: c.added_achievements,
                    removed_achievements: c.removed_achievements,
                })
                .collect(),
        })
    }

    /// The edits that unlock every layout still locked after `edits`
    /// ("Unlock all"); empty when all is unlocked. The app stages them
    /// after its own.
    #[frb(sync)]
    pub fn unlock_all_edits(
        &self,
        edits: Vec<ProfileEdit>,
    ) -> Result<Vec<ProfileEdit>, EditorError> {
        let (profile, _) = self.file.preview(&patch(edits)?)?;
        Ok(unlock_all_patch(&profile)
            .assignments
            .into_iter()
            .map(|a| ProfileEdit {
                path: a.field.path(),
                value: a.value,
            })
            .collect())
    }

    /// Applies `edits` (all or none), verifies, backs up the old file and
    /// writes the file. After a failed write, open the file again.
    ///
    /// `backup_dir` defaults to the editor's data folder; tests pass their own.
    pub fn save(
        &mut self,
        edits: Vec<ProfileEdit>,
        backup_dir: Option<String>,
    ) -> Result<SaveResult, EditorError> {
        let patch = patch(edits)?;
        let opts = options(backup_dir)?;
        self.file.apply(&patch)?;
        Ok(result(self.file.save(&opts)?))
    }

    /// Writes `backup` over the profile, as a save does: it verifies the
    /// backup, refuses when the file changed on disk, and backs up the
    /// current file into `backup_dir` first. The history lists the backups
    /// with `status::list_backups`.
    pub fn restore(
        &mut self,
        backup: String,
        backup_dir: Option<String>,
    ) -> Result<SaveResult, EditorError> {
        let opts = options(backup_dir)?;
        Ok(result(self.file.restore(backup.as_ref(), &opts)?))
    }
}

/// Every achievement of the open `ftl.dat` with name and description in
/// `language`; empty in reduced mode.
#[frb(sync)]
pub fn achievement_catalog(language: String) -> Vec<AchievementInfo> {
    let data = ftl_app::assets::shared();
    achievements::catalog(data.as_ref().map(|a| a.data().as_ref()), &language)
        .into_iter()
        .map(|a| AchievementInfo {
            id: a.id,
            name: a.name,
            description: a.description,
            kind: match a.kind {
                AchievementKind::General => "general",
                AchievementKind::Ship => "ship",
                AchievementKind::Quest => "quest",
                AchievementKind::Victory => "victory",
            }
            .to_owned(),
            ship_id: a.ship,
        })
        .collect()
}

fn options(backup_dir: Option<String>) -> Result<SaveOptions, EditorError> {
    let dir = match backup_dir {
        Some(d) => d.into(),
        None => ftl_app::default_backup_dir()?,
    };
    Ok(SaveOptions {
        backup_dir: Some(dir),
    })
}

fn result(report: ftl_app::SaveReport) -> SaveResult {
    SaveResult {
        path: report.path.display().to_string(),
        backup: report.backup.map(|b| b.display().to_string()),
        bytes: report.bytes as u32,
    }
}

/// Opens a profile. A missing file answers the code `no_profile`.
pub fn open_profile(path: String) -> Result<ProfileSession, EditorError> {
    Ok(ProfileSession {
        file: ProfileFile::open(path)?,
    })
}

/// FTL's `ae_prof.sav` on this OS.
#[frb(sync)]
pub fn default_profile_file() -> Result<String, EditorError> {
    Ok(ftl_app::default_profile_file()?.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &[u8] =
        include_bytes!("../../../../crates/ftl-sav/tests/fixtures/profile/ae_prof-1.6.13.sav");

    fn session() -> (tempfile::TempDir, String, ProfileSession) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ae_prof.sav");
        std::fs::write(&path, FIXTURE).unwrap();
        let path = path.display().to_string();
        let s = open_profile(path.clone()).unwrap();
        (dir, path, s)
    }

    fn edit(path: &str, value: i32) -> ProfileEdit {
        ProfileEdit {
            path: path.into(),
            value,
        }
    }

    fn en() -> String {
        "en".into()
    }

    #[test]
    fn the_summary_lists_every_vanilla_ship_with_its_layouts() {
        let (_dir, path, s) = session();
        let sum = s.summary(en());
        assert_eq!(sum.path, path);
        assert_eq!(sum.file_format, 9);
        let ids: Vec<_> = sum.ships.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, PLAYER_SHIPS.map(|s| s.id));
        let lanius = sum
            .ships
            .iter()
            .find(|s| s.id == "PLAYER_SHIP_ANAEROBIC")
            .unwrap();
        assert!(!lanius.has_layout_c);
        assert_eq!(lanius.achievements.len(), 3);
        assert!(
            sum.ships.iter().any(|s| !s.layout_b),
            "the fixture has a locked Type-B"
        );
        assert!(!sum.achievements.is_empty());
        assert!(
            sum.achievements
                .iter()
                .all(|a| ["easy", "normal", "hard"].contains(&a.difficulty.as_str()))
        );
        assert!(
            sum.achievements
                .iter()
                .filter(|a| a.id.ends_with("_VICTORY"))
                .all(|a| a.victory_layouts.as_ref().is_some_and(|l| l.len() == 3))
        );
        assert!(!sum.dirty);
    }

    #[test]
    fn a_preview_shows_the_edits_and_leaves_the_file_alone() {
        let (_dir, _path, s) = session();
        let locked = s
            .summary(en())
            .ships
            .into_iter()
            .find(|s| !s.layout_b)
            .unwrap();
        let b = format!("ships.{}.b", locked.id);
        let p = s
            .preview(
                vec![edit(&b, 1), edit("achievements.ACH_SECTOR_8", 3)],
                en(),
            )
            .unwrap();
        let ship = p.summary.ships.iter().find(|x| x.id == locked.id).unwrap();
        assert!(ship.layout_b);
        assert!(p.summary.dirty);
        assert_eq!(p.changes.len(), 2);
        assert_eq!(
            (p.changes[0].path.as_str(), p.changes[0].value),
            (b.as_str(), 1)
        );
        assert!(!p.changes[0].added_achievements.is_empty());
        assert!(
            p.summary
                .achievements
                .iter()
                .any(|a| a.id == "ACH_SECTOR_8" && a.difficulty == "hard")
        );
        assert!(!s.summary(en()).dirty, "the file is unchanged");
        let Err(err) = s.preview(vec![edit("ships.NOPE.a", 1)], en()) else {
            panic!("previewed an unknown ship")
        };
        assert_eq!(err.code, "edit");
    }

    #[test]
    fn staged_edits_save_with_a_backup_and_read_back() {
        let (dir, path, mut s) = session();
        let edits = vec![
            edit("ships.PLAYER_SHIP_CIRCLE.c", 1),
            edit("achievements.ACH_SECTOR_8", 2),
        ];
        let backups = dir.path().join("backups");
        let result = s.save(edits, Some(backups.display().to_string())).unwrap();
        assert_eq!(result.path, path);
        assert_eq!(std::fs::read(result.backup.unwrap()).unwrap(), FIXTURE);
        let again = open_profile(path).unwrap().summary(en());
        assert!(
            again
                .ships
                .iter()
                .any(|s| s.id == "PLAYER_SHIP_CIRCLE" && s.layout_c)
        );
        assert!(
            again
                .achievements
                .iter()
                .any(|a| a.id == "ACH_SECTOR_8" && a.difficulty == "normal")
        );
    }

    #[test]
    fn unlock_all_edits_unlock_everything_and_then_are_empty() {
        let (dir, path, mut s) = session();
        // A staged lock is unlocked again.
        let staged = vec![edit("ships.PLAYER_SHIP_HARD.a", 0)];
        let edits = s.unlock_all_edits(staged).unwrap();
        assert!(edits.iter().any(|e| e.path == "ships.PLAYER_SHIP_HARD.a"));
        let edits = s.unlock_all_edits(vec![]).unwrap();
        assert!(!edits.is_empty());
        assert!(edits.iter().all(|e| e.value == 1));
        let backups = Some(dir.path().join("backups").display().to_string());
        s.save(edits, backups).unwrap();
        let again = open_profile(path).unwrap();
        for ship in &again.summary(en()).ships {
            assert!(ship.layout_a && ship.layout_b, "{}", ship.id);
            assert_eq!(ship.layout_c, ship.has_layout_c, "{}", ship.id);
        }
        assert!(again.unlock_all_edits(vec![]).unwrap().is_empty());
    }

    #[test]
    fn restore_writes_a_backup_back_and_the_list_names_it() {
        let (dir, path, mut s) = session();
        let backups = dir.path().join("backups").display().to_string();
        let edits = s.unlock_all_edits(vec![]).unwrap();
        let first = s
            .save(edits, Some(backups.clone()))
            .unwrap()
            .backup
            .unwrap();
        let list = super::super::status::list_backups(path.clone(), Some(backups.clone())).unwrap();
        assert_eq!(list.iter().map(|b| &b.path).collect::<Vec<_>>(), [&first]);

        let result = s.restore(first, Some(backups)).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), FIXTURE);
        assert!(
            result.backup.is_some(),
            "the unlocked file is backed up first"
        );
        assert!(
            s.summary(en()).ships.iter().any(|s| !s.layout_b),
            "the summary shows the restored file"
        );
    }

    #[test]
    fn a_missing_profile_answers_no_profile() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ae_prof.sav").display().to_string();
        let Err(err) = open_profile(path.clone()) else {
            panic!("opened a missing file")
        };
        assert_eq!(err.code, "no_profile");
        assert_eq!(err.path, Some(path));
    }
}
