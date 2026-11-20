//! The state around the open file: what can undo or overwrite an edit (a
//! running FTL, Steam Cloud) and its backup history. A thin adapter over
//! `ftl_app::warnings`, `ftl_app::list_backups` and `SaveFile::restore`;
//! every rule lives there.

use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use ftl_app::{SaveOptions, SystemProcesses};

use super::editor::{EditorError, SaveResult, SaveSession};

/// One warning for a save file, as `ftl_app::Warning` answers it.
pub struct FileWarning {
    /// `ftl_running`, `steam_cloud` or `process_list_unavailable`
    /// (see `Warning::code`).
    pub code: String,
    /// The English message of the core, as detail for a tooltip.
    pub message: String,
}

/// The warnings for the save file at `path`, read now: running FTL
/// processes by pid, then Steam Cloud. An unreadable process list is the
/// warning `process_list_unavailable`, not an empty list.
pub fn save_warnings(path: String) -> Vec<FileWarning> {
    ftl_app::warnings(path.as_ref(), &SystemProcesses)
        .into_iter()
        .map(|w| FileWarning {
            code: w.code().to_owned(),
            message: w.to_string(),
        })
        .collect()
}

/// One backup of a save file.
pub struct BackupInfo {
    pub path: String,
    /// When the backup was written, in milliseconds since the Unix epoch.
    pub modified_ms: i64,
    pub bytes: u32,
}

fn backup_dir_or_default(backup_dir: Option<String>) -> Result<PathBuf, EditorError> {
    Ok(match backup_dir {
        Some(d) => d.into(),
        None => ftl_app::default_backup_dir()?,
    })
}

/// The backups of the save file at `save_path`, newest first.
///
/// `backup_dir` defaults to the editor's data folder; tests pass their own.
pub fn list_backups(
    save_path: String,
    backup_dir: Option<String>,
) -> Result<Vec<BackupInfo>, EditorError> {
    let dir = backup_dir_or_default(backup_dir)?;
    Ok(ftl_app::list_backups(&dir, save_path.as_ref())?
        .into_iter()
        .map(|b| BackupInfo {
            path: b.path.display().to_string(),
            modified_ms: b
                .modified
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_millis() as i64),
            bytes: b.bytes as u32,
        })
        .collect())
}

/// Writes `backup` over the open file, as a save does: it verifies the
/// backup, refuses when the file changed on disk, and backs up the current
/// file into `backup_dir` first. In-memory edits of the session are gone.
pub fn restore_backup(
    session: &mut SaveSession,
    backup: String,
    backup_dir: Option<String>,
) -> Result<SaveResult, EditorError> {
    let dir = backup_dir_or_default(backup_dir)?;
    let report = session.file.restore(
        backup.as_ref(),
        &SaveOptions {
            backup_dir: Some(dir),
        },
    )?;
    Ok(SaveResult {
        path: report.path.display().to_string(),
        backup: report.backup.map(|b| b.display().to_string()),
        bytes: report.bytes as u32,
    })
}
