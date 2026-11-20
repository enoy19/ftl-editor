//! The one API of the FTL editor. The CLI and the Flutter bridge call this
//! crate and nothing below it.
//!
//! Every save of a `continue.sav` ([`SaveFile`]) or a profile ([`ProfileFile`])
//! follows the same steps:
//! 1. Encode the edited model and decode the result again. The decoded model
//!    must equal the edited one, else the save stops (a raw edit that
//!    breaks the layout of the file, else a codec defect).
//! 2. Read the file again. If its bytes differ from the bytes at open time,
//!    the save stops: something else (FTL itself) wrote the file meanwhile.
//! 3. Copy the bytes from open time into the backup folder.
//! 4. Write a temporary file next to the target and rename it over the target.

pub mod achievements;
pub mod actions;
pub mod assets;
mod backups;
pub mod crew;
pub mod equipment;
pub mod floorplan;
pub mod identity;
pub mod layouts;
pub mod map;

pub use backups::{BackupEntry, list_backups};

pub use layouts::LayoutSource;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub use ftl_core::profile::{
    self, Profile, ProfileAssignment, ProfileChange, ProfileEditError, ProfileField, ProfilePatch,
    UnlockChange,
};
pub use ftl_core::{
    Assignment, Change, EditError, Field, INDEXED_PATHS, Patch, RACES, RawEntry, RawKind, Resource,
    SavedGame, ShipId, raw,
};
use std::sync::Arc;

use ftl_core::{DecodeError, ProfileCodec, SaveCodec};
pub use ftl_core::{KnownLayouts, ShipLayouts};
use ftl_sav::{ProfCodec, SavCodec};

mod guard;
pub use guard::{
    FTL_PROCESS_NAMES, ProcessInfo, ProcessList, STEAM_CLOUD_MARKER, SystemProcesses, Warning,
    warnings,
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(
        "no saved game at {path}. FTL deletes continue.sav when a run ends; start or continue a run, then quit FTL to write it."
    )]
    NoSave { path: PathBuf },
    #[error(
        "no profile at {path}. FTL writes ae_prof.sav when it quits for the first time; start FTL once and quit it."
    )]
    NoProfile { path: PathBuf },
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{path}: cannot decode: {source}")]
    Decode {
        path: PathBuf,
        #[source]
        source: DecodeError,
    },
    #[error(transparent)]
    Edit(#[from] EditError),
    /// A profile edit; the same code `edit` as a saved-game edit.
    #[error(transparent)]
    ProfileEdit(#[from] ProfileEditError),
    #[error(
        "{path} changed on disk since it was opened (FTL running?); nothing written. Close FTL, open the file again and repeat the edit."
    )]
    ChangedOnDisk { path: PathBuf },
    #[error(
        "the encoded game does not decode to the edited game; nothing written. A raw edit (raw.<path>) broke the layout of the file, or the codec has a defect: {detail}"
    )]
    Verify { detail: String },
    #[error("no home directory found to place {what}")]
    NoHome { what: &'static str },
}

impl AppError {
    /// Stable code for a UI that shows its own text for an error.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NoSave { .. } => "no_save",
            AppError::NoProfile { .. } => "no_profile",
            AppError::Io { .. } => "io",
            AppError::Decode { .. } => "decode",
            AppError::Edit(_) | AppError::ProfileEdit(_) => "edit",
            AppError::ChangedOnDisk { .. } => "changed_on_disk",
            AppError::Verify { .. } => "verify",
            AppError::NoHome { .. } => "no_home",
        }
    }

    /// The file the error is about, if any.
    pub fn path(&self) -> Option<&Path> {
        match self {
            AppError::NoSave { path }
            | AppError::NoProfile { path }
            | AppError::Io { path, .. }
            | AppError::Decode { path, .. }
            | AppError::ChangedOnDisk { path } => Some(path),
            AppError::Edit(_)
            | AppError::ProfileEdit(_)
            | AppError::Verify { .. }
            | AppError::NoHome { .. } => None,
        }
    }
}

fn io_err(path: &Path) -> impl FnOnce(io::Error) -> AppError + '_ {
    move |source| AppError::Io {
        path: path.to_owned(),
        source,
    }
}

/// An opened `continue.sav`.
#[derive(Debug, Clone)]
pub struct SaveFile {
    path: PathBuf,
    on_disk: Vec<u8>,
    game: SavedGame,
    /// The layouts the game was decoded with; verify decodes with the same.
    layouts: Arc<dyn ShipLayouts>,
    layout_source: LayoutSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveOptions {
    /// Folder for the copy of the previous file. `None` writes no backup.
    pub backup_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveReport {
    pub path: PathBuf,
    pub backup: Option<PathBuf>,
    pub bytes: usize,
}

impl SaveFile {
    /// Opens with the ship layouts of the located `ftl.dat`
    /// ([`layouts::default_layouts`]), loaded once per call. Without a
    /// readable `ftl.dat` it opens without layouts: the player ship then
    /// decodes up to its systems, and its rooms, doors, weapons, drones and
    /// augments stay in `SavedGame::undecoded`. [`SaveFile::layout_source`]
    /// says which.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let (layouts, source) = layouts::default_layouts();
        Self::open_inner(path.as_ref(), layouts, source)
    }

    /// Opens with the given layouts (tests, or a caller that holds them).
    pub fn open_with_layouts(
        path: impl AsRef<Path>,
        layouts: Arc<dyn ShipLayouts>,
    ) -> Result<Self, AppError> {
        Self::open_inner(path.as_ref(), layouts, LayoutSource::Given)
    }

    /// Where the ship layouts of this file came from.
    pub fn layout_source(&self) -> &LayoutSource {
        &self.layout_source
    }

    fn open_inner(
        path: &Path,
        layouts: Arc<dyn ShipLayouts>,
        layout_source: LayoutSource,
    ) -> Result<Self, AppError> {
        let path = path.to_owned();
        let on_disk = fs::read(&path).map_err(|source| match source.kind() {
            io::ErrorKind::NotFound => AppError::NoSave { path: path.clone() },
            _ => AppError::Io {
                path: path.clone(),
                source,
            },
        })?;
        let game = SavCodec
            .decode(&on_disk, layouts.as_ref())
            .map_err(|source| AppError::Decode {
                path: path.clone(),
                source,
            })?;
        Ok(SaveFile {
            path,
            on_disk,
            game,
            layouts,
            layout_source,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn game(&self) -> &SavedGame {
        &self.game
    }

    /// Applies a patch in memory. Nothing is written until [`SaveFile::save`].
    /// The layouts the file was opened with answer the slots of the ship and
    /// the weapon and drone blueprints.
    pub fn apply(&mut self, patch: &Patch) -> Result<Vec<Change>, AppError> {
        Ok(patch.apply_in(&mut self.game, self.layouts.as_ref())?)
    }

    /// The game as `patch` would leave it, without changing this file. A
    /// crew member staged for removal stays in its place with
    /// [`ftl_core::CrewMember::removed`] set ([`Patch::stage_in`]).
    pub fn preview(&self, patch: &Patch) -> Result<SavedGame, AppError> {
        let mut game = self.game.clone();
        patch.stage_in(&mut game, self.layouts.as_ref())?;
        Ok(game)
    }

    /// The slots of the ship and the weapon and drone blueprints this file
    /// was opened with (for the ranges of [`ftl_core::Field::range_in`]).
    pub fn layouts(&self) -> &dyn ShipLayouts {
        self.layouts.as_ref()
    }

    /// The bytes a save would write now.
    pub fn encoded(&self) -> Vec<u8> {
        SavCodec.encode(&self.game)
    }

    /// Whether the edited game encodes to other bytes than the file holds.
    pub fn is_dirty(&self) -> bool {
        self.encoded() != self.on_disk
    }

    /// Re-decodes the encoded game and compares it with the edited game.
    pub fn verify(&self) -> Result<Vec<u8>, AppError> {
        verify_round_trip(
            &self.game,
            |g| SavCodec.encode(g),
            |b| SavCodec.decode(b, self.layouts.as_ref()),
            "game",
        )
    }

    pub fn save(&mut self, opts: &SaveOptions) -> Result<SaveReport, AppError> {
        let bytes = self.verify()?;
        write_checked(&self.path, &mut self.on_disk, bytes, opts)
    }
}

/// Steps 2 to 4 of every save (see the crate docs): the changed-on-disk
/// check, the backup of the bytes from open time, the atomic write. On
/// success `on_disk` holds the written bytes.
fn write_checked(
    path: &Path,
    on_disk: &mut Vec<u8>,
    bytes: Vec<u8>,
    opts: &SaveOptions,
) -> Result<SaveReport, AppError> {
    let current = fs::read(path).map_err(io_err(path))?;
    if current != *on_disk {
        return Err(AppError::ChangedOnDisk {
            path: path.to_owned(),
        });
    }

    let backup = match &opts.backup_dir {
        Some(dir) => Some(write_backup(dir, path, on_disk)?),
        None => None,
    };

    let tmp = path.with_extension("sav.ftl-editor-tmp");
    fs::write(&tmp, &bytes).map_err(io_err(&tmp))?;
    fs::rename(&tmp, path).map_err(io_err(path))?;

    let report = SaveReport {
        path: path.to_owned(),
        backup,
        bytes: bytes.len(),
    };
    *on_disk = bytes;
    Ok(report)
}

/// Step 1 of every save: encode, decode again, compare with the edited
/// model.
fn verify_round_trip<M: PartialEq>(
    model: &M,
    encode: impl Fn(&M) -> Vec<u8>,
    decode: impl Fn(&[u8]) -> Result<M, DecodeError>,
    what: &str,
) -> Result<Vec<u8>, AppError> {
    let bytes = encode(model);
    let again = decode(&bytes).map_err(|e| AppError::Verify {
        detail: e.to_string(),
    })?;
    if again != *model {
        return Err(AppError::Verify {
            detail: format!("decoded {what} differs from the edited {what}"),
        });
    }
    Ok(bytes)
}

/// An opened `ae_prof.sav` (or legacy `prof.sav`). Saves follow the same
/// steps as [`SaveFile::save`].
#[derive(Debug, Clone)]
pub struct ProfileFile {
    path: PathBuf,
    on_disk: Vec<u8>,
    profile: Profile,
}

impl ProfileFile {
    /// A missing file is [`AppError::NoProfile`]; FTL writes the profile
    /// when it first quits and never deletes it.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let path = path.as_ref().to_owned();
        let on_disk = fs::read(&path).map_err(|source| match source.kind() {
            io::ErrorKind::NotFound => AppError::NoProfile { path: path.clone() },
            _ => AppError::Io {
                path: path.clone(),
                source,
            },
        })?;
        let profile = ProfCodec
            .decode(&on_disk)
            .map_err(|source| AppError::Decode {
                path: path.clone(),
                source,
            })?;
        Ok(ProfileFile {
            path,
            on_disk,
            profile,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// Unlocks every layout of every ship in memory. Nothing is written
    /// until [`ProfileFile::save`].
    pub fn unlock_all_ships(&mut self) -> Vec<UnlockChange> {
        ftl_core::profile::unlock_all_ships(&mut self.profile)
    }

    /// Applies all edits in memory, or none when one is invalid. Nothing is
    /// written until [`ProfileFile::save`].
    pub fn apply(&mut self, patch: &ProfilePatch) -> Result<Vec<ProfileChange>, AppError> {
        Ok(patch.apply(&mut self.profile)?)
    }

    /// The profile as `patch` would leave it and what each edit changes,
    /// without touching this file. The app shows staged edits with it.
    pub fn preview(&self, patch: &ProfilePatch) -> Result<(Profile, Vec<ProfileChange>), AppError> {
        let mut profile = self.profile.clone();
        let changes = patch.apply(&mut profile)?;
        Ok((profile, changes))
    }

    /// The bytes a save would write now.
    pub fn encoded(&self) -> Vec<u8> {
        ProfCodec.encode(&self.profile)
    }

    pub fn is_dirty(&self) -> bool {
        self.encoded() != self.on_disk
    }

    pub fn verify(&self) -> Result<Vec<u8>, AppError> {
        verify_round_trip(
            &self.profile,
            |p| ProfCodec.encode(p),
            |b| ProfCodec.decode(b),
            "profile",
        )
    }

    pub fn save(&mut self, opts: &SaveOptions) -> Result<SaveReport, AppError> {
        let bytes = self.verify()?;
        write_checked(&self.path, &mut self.on_disk, bytes, opts)
    }
}

/// The name part of every backup of `target`: `continue.sav` has backups
/// `continue_<time>.sav`.
fn backup_stem(target: &Path) -> String {
    target
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "save".into())
}

fn write_backup(dir: &Path, target: &Path, bytes: &[u8]) -> Result<PathBuf, AppError> {
    fs::create_dir_all(dir).map_err(io_err(dir))?;
    let stem = backup_stem(target);
    let stamp = jiff::Zoned::now().strftime("%Y-%m-%d_%H%M%S").to_string();
    let mut path = dir.join(format!("{stem}_{stamp}.sav"));
    let mut n = 1;
    while path.exists() {
        n += 1;
        path = dir.join(format!("{stem}_{stamp}_{n}.sav"));
    }
    fs::write(&path, bytes).map_err(io_err(&path))?;
    Ok(path)
}

/// Where FTL keeps `continue.sav` and `ae_prof.sav` on this OS.
pub fn default_save_dir() -> Result<PathBuf, AppError> {
    let no_home = || AppError::NoHome {
        what: "the FTL save folder",
    };
    let base = directories::BaseDirs::new().ok_or_else(no_home)?;
    if cfg!(target_os = "windows") {
        let user = directories::UserDirs::new().ok_or_else(no_home)?;
        let docs = user.document_dir().ok_or_else(no_home)?;
        Ok(docs.join("My Games").join("FasterThanLight"))
    } else {
        // macOS: ~/Library/Application Support, Linux: ~/.local/share.
        Ok(base.data_dir().join("FasterThanLight"))
    }
}

pub fn default_save_file() -> Result<PathBuf, AppError> {
    Ok(default_save_dir()?.join("continue.sav"))
}

pub fn default_profile_file() -> Result<PathBuf, AppError> {
    Ok(default_save_dir()?.join("ae_prof.sav"))
}

/// The editor's own backup folder, outside the FTL folder.
pub fn default_backup_dir() -> Result<PathBuf, AppError> {
    let dirs = directories::ProjectDirs::from("", "", "ftl-editor").ok_or(AppError::NoHome {
        what: "the backup folder",
    })?;
    Ok(dirs.data_dir().join("backups"))
}

/// The fake layout port of the codec tests (no `ftl.dat` adapter yet).
#[cfg(test)]
#[path = "../../ftl-sav/tests/common/layouts.rs"]
mod fixture_layouts;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture_layouts as layouts;

    const FIXTURE: &[u8] =
        include_bytes!("../../ftl-sav/tests/fixtures/continue-1.6.13-format11.sav");

    #[test]
    fn with_layouts_the_body_decodes_and_a_save_keeps_it() {
        let (dir, path) = setup();
        let mut f =
            SaveFile::open_with_layouts(&path, Arc::new(layouts::fixture_layouts())).unwrap();
        // The body holds the 17 rooms of the kestral layout; the fixture
        // ship has no augment.
        let body = f.game().player_ship.body.as_ref().unwrap();
        assert_eq!(body.rooms.len(), 17);
        assert!(body.augments.is_empty());
        let bare = SaveFile::open_with_layouts(&path, Arc::new(KnownLayouts::new())).unwrap();
        assert!(bare.game().player_ship.body.is_none());
        assert_eq!(bare.layout_source(), &LayoutSource::Given);
        f.apply(&Patch::new(vec![
            "systems.shields.level=7".parse().unwrap(),
        ]))
        .unwrap();
        f.save(&SaveOptions {
            backup_dir: Some(dir.path().join("backups")),
        })
        .unwrap();
        let again =
            SaveFile::open_with_layouts(&path, Arc::new(layouts::fixture_layouts())).unwrap();
        assert_eq!(again.game(), f.game());
    }

    /// A new fire and a new breach pass the verify of the save and open
    /// again as written.
    #[test]
    fn a_save_with_a_new_fire_and_breach_verifies() {
        let (dir, path) = setup();
        let mut f =
            SaveFile::open_with_layouts(&path, Arc::new(layouts::fixture_layouts())).unwrap();
        f.apply(&Patch::new(vec![
            "fires.9.5=100".parse().unwrap(),
            "breaches.8.6=100".parse().unwrap(),
        ]))
        .unwrap();
        f.save(&SaveOptions {
            backup_dir: Some(dir.path().join("backups")),
        })
        .unwrap();
        let again =
            SaveFile::open_with_layouts(&path, Arc::new(layouts::fixture_layouts())).unwrap();
        assert_eq!(again.game(), f.game());
        let body = again.game().player_ship.body.as_ref().unwrap();
        // Shields, room 5 of kestral at 8, 5 of the grid (offset 0, 2).
        assert_eq!(body.rooms[5].squares[1].fire_health, 100);
        assert_eq!((body.breaches[0].x, body.breaches[0].y), (8, 6));
    }

    /// A raw edit saves through the same verify: a number of the Expert
    /// area writes and reads back, and a raw edit that breaks the file (a
    /// format FTL 1.6 does not write) is refused before anything is written.
    #[test]
    fn raw_edits_save_through_the_verify() {
        let (dir, path) = setup();
        let opts = SaveOptions {
            backup_dir: Some(dir.path().join("backups")),
        };
        let mut f =
            SaveFile::open_with_layouts(&path, Arc::new(layouts::fixture_layouts())).unwrap();
        let id = f.game().state_vars[0].id.clone();
        f.apply(&Patch::new(vec![
            "raw.player_ship.crew.0.clone_dying=5".parse().unwrap(),
            format!("state.{id}=9").parse().unwrap(),
        ]))
        .unwrap();
        f.save(&opts).unwrap();
        let again =
            SaveFile::open_with_layouts(&path, Arc::new(layouts::fixture_layouts())).unwrap();
        assert_eq!(again.game(), f.game());
        assert_eq!(again.game().player_ship.crew[0].clone_dying, 5);
        assert_eq!(again.game().state_vars[0].value, 9);

        let before = fs::read(&path).unwrap();
        f.apply(&Patch::new(vec!["raw.file_format=2".parse().unwrap()]))
            .unwrap();
        assert!(matches!(f.save(&opts), Err(AppError::Verify { .. })));
        assert_eq!(fs::read(&path).unwrap(), before);
    }

    fn setup() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("continue.sav");
        fs::write(&path, FIXTURE).unwrap();
        (dir, path)
    }

    fn scrap(v: i32) -> Patch {
        Patch::new(vec![Assignment::new(Field::SCRAP, v)])
    }

    #[test]
    fn save_writes_the_edit_and_backs_up_the_old_bytes() {
        let (dir, path) = setup();
        let mut f = SaveFile::open(&path).unwrap();
        assert!(!f.is_dirty());
        f.apply(&scrap(9999)).unwrap();
        assert!(f.is_dirty());
        let report = f
            .save(&SaveOptions {
                backup_dir: Some(dir.path().join("backups")),
            })
            .unwrap();
        assert_eq!(fs::read(report.backup.unwrap()).unwrap(), FIXTURE);
        assert_eq!(
            SaveFile::open(&path)
                .unwrap()
                .game()
                .player_ship
                .resources
                .scrap,
            9999
        );
        assert!(!f.is_dirty());
    }

    #[test]
    fn save_refuses_when_the_file_changed_since_open() {
        let (_dir, path) = setup();
        let mut f = SaveFile::open(&path).unwrap();
        f.apply(&scrap(9999)).unwrap();
        let mut other = FIXTURE.to_vec();
        other.push(0);
        fs::write(&path, &other).unwrap();
        let err = f.save(&SaveOptions { backup_dir: None }).unwrap_err();
        assert!(matches!(err, AppError::ChangedOnDisk { .. }), "{err}");
        assert_eq!(fs::read(&path).unwrap(), other);
    }

    #[test]
    fn a_missing_save_names_why_ftl_deleted_it() {
        let dir = tempfile::tempdir().unwrap();
        let err = SaveFile::open(dir.path().join("continue.sav")).unwrap_err();
        assert!(matches!(err, AppError::NoSave { .. }), "{err}");
        assert!(
            err.to_string()
                .contains("FTL deletes continue.sav when a run ends")
        );
        assert_eq!(err.code(), "no_save");
        assert_eq!(err.path(), Some(dir.path().join("continue.sav").as_path()));
    }

    const PROFILE: &[u8] =
        include_bytes!("../../ftl-sav/tests/fixtures/profile/ae_prof-1.6.13.sav");

    fn setup_profile() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ae_prof.sav");
        fs::write(&path, PROFILE).unwrap();
        (dir, path)
    }

    #[test]
    fn profile_save_writes_the_unlocks_and_backs_up_the_old_bytes() {
        let (dir, path) = setup_profile();
        let mut f = ProfileFile::open(&path).unwrap();
        assert!(!f.is_dirty());
        assert!(!f.unlock_all_ships().is_empty());
        let report = f
            .save(&SaveOptions {
                backup_dir: Some(dir.path().join("backups")),
            })
            .unwrap();
        let backup = report.backup.unwrap();
        assert_eq!(fs::read(&backup).unwrap(), PROFILE);
        assert!(
            backup
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("ae_prof_")
        );
        let again = ProfileFile::open(&path).unwrap();
        assert!(
            again
                .profile()
                .ship_unlocks("PLAYER_SHIP_STEALTH")
                .unwrap()
                .layout_b
        );
        assert_eq!(again.profile(), f.profile());
        assert!(!f.is_dirty());
    }

    #[test]
    fn profile_edits_save_and_read_back() {
        let (dir, path) = setup_profile();
        let mut f = ProfileFile::open(&path).unwrap();
        let patch = ProfilePatch::new(
            [
                "ships.PLAYER_SHIP_CIRCLE.c=1",
                "achievements.ACH_SECTOR_5=3",
                "achievements.PLAYER_SHIP_ROCK_VICTORY=2",
            ]
            .iter()
            .map(|e| e.parse().unwrap())
            .collect(),
        );
        let (preview, changes) = f.preview(&patch).unwrap();
        assert_eq!(changes.len(), 3);
        assert!(!f.is_dirty(), "a preview leaves the file alone");
        f.apply(&patch).unwrap();
        assert_eq!(f.profile(), &preview);
        f.save(&SaveOptions {
            backup_dir: Some(dir.path().join("backups")),
        })
        .unwrap();
        let again = ProfileFile::open(&path).unwrap();
        assert_eq!(again.profile(), &preview);
        let p = again.profile();
        assert!(p.ship_unlocks("PLAYER_SHIP_CIRCLE").unwrap().layout_c);
        assert_eq!(
            ProfileField::Achievement("ACH_SECTOR_5".into()).get(p),
            3,
            "Hard"
        );
        let err = f
            .apply(&ProfilePatch::new(vec![
                "ships.PLAYER_SHIP_CRYSTAL.c=1".parse().unwrap(),
            ]))
            .unwrap_err();
        assert_eq!(err.code(), "edit");
    }

    #[test]
    fn a_missing_profile_answers_its_own_code() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ae_prof.sav");
        let err = ProfileFile::open(&path).unwrap_err();
        assert!(matches!(err, AppError::NoProfile { .. }), "{err}");
        assert_eq!(err.code(), "no_profile");
        assert_eq!(err.path(), Some(path.as_path()));
        assert!(
            err.to_string()
                .contains("FTL writes ae_prof.sav when it quits"),
            "{err}"
        );
    }

    #[test]
    fn profile_save_refuses_when_the_file_changed_since_open() {
        let (_dir, path) = setup_profile();
        let mut f = ProfileFile::open(&path).unwrap();
        f.unlock_all_ships();
        let mut other = PROFILE.to_vec();
        other.push(0);
        fs::write(&path, &other).unwrap();
        let err = f.save(&SaveOptions { backup_dir: None }).unwrap_err();
        assert!(matches!(err, AppError::ChangedOnDisk { .. }), "{err}");
        assert_eq!(fs::read(&path).unwrap(), other);
    }

    #[test]
    fn two_backups_in_one_second_get_distinct_names() {
        let (dir, path) = setup();
        let backups = dir.path().join("backups");
        let a = write_backup(&backups, &path, b"a").unwrap();
        let b = write_backup(&backups, &path, b"b").unwrap();
        assert_ne!(a, b);
    }
}
