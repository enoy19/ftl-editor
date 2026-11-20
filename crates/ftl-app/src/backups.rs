//! The backup history of a save file or a profile: list the copies
//! [`SaveFile::save`] and [`ProfileFile::save`] wrote and restore one.
//!
//! A backup of `<stem>.sav` is a file `<stem>_<time>[_<n>].sav` in the
//! backup folder (see `write_backup`). A restore writes the bytes of a backup
//! through the same steps as a save: the backup must decode and encode back
//! to its own bytes, the file must not have changed on disk since it was
//! opened, and the current file is backed up first.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use ftl_core::{DecodeError, ProfileCodec, SaveCodec};
use ftl_sav::{ProfCodec, SavCodec};

use crate::{AppError, ProfileFile, SaveFile, SaveOptions, SaveReport, io_err, write_checked};

/// One backup of a save file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupEntry {
    pub path: PathBuf,
    /// When the backup was written (the file's modification time).
    pub modified: SystemTime,
    pub bytes: u64,
}

/// The backups of `save_path` in `backup_dir`, newest first. A missing
/// folder has no backups.
pub fn list_backups(backup_dir: &Path, save_path: &Path) -> Result<Vec<BackupEntry>, AppError> {
    let entries = match fs::read_dir(backup_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(io_err(backup_dir)(e)),
    };
    let prefix = format!("{}_", crate::backup_stem(save_path));
    let mut out = Vec::new();
    for entry in entries {
        let entry = entry.map_err(io_err(backup_dir))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(&prefix) || path.extension().is_none_or(|e| e != "sav") {
            continue;
        }
        let meta = entry.metadata().map_err(io_err(&path))?;
        if !meta.is_file() {
            continue;
        }
        out.push(BackupEntry {
            modified: meta.modified().map_err(io_err(&path))?,
            bytes: meta.len(),
            path,
        });
    }
    // Newest first; the name breaks a tie (a later `_<n>` is newer).
    out.sort_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| b.path.cmp(&a.path))
    });
    Ok(out)
}

impl SaveFile {
    /// Writes the bytes of `backup` over the file, as a save does.
    /// The backup must decode with the layouts of this file and encode back
    /// to its own bytes; a restore never writes bytes the codec cannot read.
    pub fn restore(&mut self, backup: &Path, opts: &SaveOptions) -> Result<SaveReport, AppError> {
        let layouts = self.layouts.clone();
        let (report, game) = restore_checked(
            &self.path,
            &mut self.on_disk,
            backup,
            |b| SavCodec.decode(b, layouts.as_ref()),
            |g| SavCodec.encode(g),
            opts,
        )?;
        self.game = game;
        Ok(report)
    }
}

impl ProfileFile {
    /// Writes the bytes of `backup` over the profile, as
    /// [`SaveFile::restore`] does for a save.
    pub fn restore(&mut self, backup: &Path, opts: &SaveOptions) -> Result<SaveReport, AppError> {
        let (report, profile) = restore_checked(
            &self.path,
            &mut self.on_disk,
            backup,
            |b| ProfCodec.decode(b),
            |p| ProfCodec.encode(p),
            opts,
        )?;
        self.profile = profile;
        Ok(report)
    }
}

/// The steps of every restore: read the backup, decode it, check that it
/// encodes back to its own bytes, then write it as a save writes
/// ([`write_checked`]). Answers the decoded model of the restored file.
fn restore_checked<M>(
    path: &Path,
    on_disk: &mut Vec<u8>,
    backup: &Path,
    decode: impl Fn(&[u8]) -> Result<M, DecodeError>,
    encode: impl Fn(&M) -> Vec<u8>,
    opts: &SaveOptions,
) -> Result<(SaveReport, M), AppError> {
    let bytes = fs::read(backup).map_err(io_err(backup))?;
    let model = decode(&bytes).map_err(|source| AppError::Decode {
        path: backup.to_owned(),
        source,
    })?;
    if encode(&model) != bytes {
        return Err(AppError::Verify {
            detail: format!("{} does not encode back to its own bytes", backup.display()),
        });
    }
    let report = write_checked(path, on_disk, bytes, opts)?;
    Ok((report, model))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Assignment, Field, Patch};
    use std::fs::File;
    use std::time::Duration;

    const FIXTURE: &[u8] =
        include_bytes!("../../ftl-sav/tests/fixtures/continue-1.6.13-format11.sav");

    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("continue.sav");
        fs::write(&path, FIXTURE).unwrap();
        let backups = dir.path().join("backups");
        (dir, path, backups)
    }

    fn put(dir: &Path, name: &str, bytes: &[u8], age_s: u64) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, bytes).unwrap();
        let t = SystemTime::now() - Duration::from_secs(age_s);
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(t)
            .unwrap();
        path
    }

    fn scrap(f: &mut SaveFile, v: i32) {
        f.apply(&Patch::new(vec![Assignment::new(Field::SCRAP, v)]))
            .unwrap();
    }

    #[test]
    fn backups_list_newest_first_with_size_and_only_of_this_file() {
        let (_dir, path, backups) = setup();
        let old = put(&backups, "continue_2026-11-01_100000.sav", b"old", 300);
        let new = put(&backups, "continue_2026-11-01_100500.sav", b"newest", 10);
        let mid = put(&backups, "continue_2026-11-01_100200_2.sav", b"mi", 100);
        put(&backups, "ae_prof_2026-11-01_100600.sav", b"profile", 1);
        put(&backups, "continue_notes.txt", b"x", 1);
        let list = list_backups(&backups, &path).unwrap();
        let paths: Vec<_> = list.iter().map(|b| b.path.clone()).collect();
        assert_eq!(paths, [new, mid, old]);
        assert_eq!(list.iter().map(|b| b.bytes).collect::<Vec<_>>(), [6, 2, 3]);
        assert!(list[0].modified > list[1].modified);
    }

    #[test]
    fn a_missing_backup_folder_has_no_backups() {
        let (dir, path, _) = setup();
        assert_eq!(list_backups(&dir.path().join("none"), &path).unwrap(), []);
    }

    #[test]
    fn a_save_lists_its_backup() {
        let (_dir, path, backups) = setup();
        let mut f = SaveFile::open(&path).unwrap();
        scrap(&mut f, 9999);
        let report = f
            .save(&SaveOptions {
                backup_dir: Some(backups.clone()),
            })
            .unwrap();
        let list = list_backups(&backups, &path).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(Some(list[0].path.clone()), report.backup);
    }

    #[test]
    fn restore_writes_the_backup_bytes_and_backs_up_the_current_file() {
        let (_dir, path, backups) = setup();
        let mut f = SaveFile::open(&path).unwrap();
        scrap(&mut f, 9999);
        let first = f
            .save(&SaveOptions {
                backup_dir: Some(backups.clone()),
            })
            .unwrap();
        let edited = fs::read(&path).unwrap();
        let first = first.backup.unwrap();

        let report = f
            .restore(
                &first,
                &SaveOptions {
                    backup_dir: Some(backups.clone()),
                },
            )
            .unwrap();
        assert_eq!(fs::read(&path).unwrap(), FIXTURE);
        let current = report.backup.unwrap();
        assert_ne!(current, first);
        assert_eq!(fs::read(&current).unwrap(), edited);
        assert_eq!(
            f.game().player_ship.resources.scrap,
            SaveFile::open(&path)
                .unwrap()
                .game()
                .player_ship
                .resources
                .scrap
        );
        assert!(!f.is_dirty());
        assert_eq!(list_backups(&backups, &path).unwrap().len(), 2);
    }

    #[test]
    fn restore_refuses_when_the_file_changed_since_open() {
        let (_dir, path, backups) = setup();
        let backup = put(&backups, "continue_2026-11-01_100000.sav", FIXTURE, 60);
        let mut f = SaveFile::open(&path).unwrap();
        let mut other = FIXTURE.to_vec();
        other.push(0);
        fs::write(&path, &other).unwrap();
        let err = f
            .restore(
                &backup,
                &SaveOptions {
                    backup_dir: Some(backups.clone()),
                },
            )
            .unwrap_err();
        assert!(matches!(err, AppError::ChangedOnDisk { .. }), "{err}");
        assert_eq!(fs::read(&path).unwrap(), other);
        assert_eq!(
            list_backups(&backups, &path).unwrap().len(),
            1,
            "no backup of the refused restore"
        );
    }

    const PROFILE: &[u8] =
        include_bytes!("../../ftl-sav/tests/fixtures/profile/ae_prof-1.6.13.sav");

    #[test]
    fn a_profile_restores_its_backup_through_the_same_steps() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ae_prof.sav");
        fs::write(&path, PROFILE).unwrap();
        let backups = dir.path().join("backups");
        let opts = SaveOptions {
            backup_dir: Some(backups.clone()),
        };
        // A save backup lies in the same folder and is not one of the profile.
        put(&backups, "continue_2026-11-01_100000.sav", FIXTURE, 60);
        let mut f = ProfileFile::open(&path).unwrap();
        f.unlock_all_ships();
        let first = f.save(&opts).unwrap().backup.unwrap();
        let unlocked = fs::read(&path).unwrap();
        let list = list_backups(&backups, &path).unwrap();
        assert_eq!(list.iter().map(|b| &b.path).collect::<Vec<_>>(), [&first]);

        let report = f.restore(&first, &opts).unwrap();
        assert_eq!(fs::read(&path).unwrap(), PROFILE);
        assert_eq!(fs::read(report.backup.unwrap()).unwrap(), unlocked);
        assert_eq!(f.profile(), ProfileFile::open(&path).unwrap().profile());
        assert!(!f.is_dirty());

        // A save is no profile: the restore refuses it and writes nothing.
        let save = put(&backups, "ae_prof_2026-11-01_090000.sav", FIXTURE, 600);
        let err = f.restore(&save, &opts).unwrap_err();
        assert!(matches!(err, AppError::Decode { .. }), "{err}");
        assert_eq!(fs::read(&path).unwrap(), PROFILE);
    }

    #[test]
    fn restore_refuses_a_backup_that_does_not_decode() {
        let (_dir, path, backups) = setup();
        let broken = put(&backups, "continue_2026-11-01_100000.sav", b"no save", 60);
        let mut f = SaveFile::open(&path).unwrap();
        let err = f
            .restore(
                &broken,
                &SaveOptions {
                    backup_dir: Some(backups.clone()),
                },
            )
            .unwrap_err();
        assert!(
            matches!(err, AppError::Decode { ref path, .. } if *path == broken),
            "{err}"
        );
        assert_eq!(fs::read(&path).unwrap(), FIXTURE);
    }
}
