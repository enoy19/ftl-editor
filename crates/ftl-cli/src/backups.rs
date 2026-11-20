//! `ftled backups`: the backup history of a save file or, with `--profile`,
//! of the profile (`ae_prof.sav`).

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use ftl_app::{ProfileFile, SaveOptions};

use crate::file_or_default;

#[derive(Subcommand)]
pub enum BackupsCommand {
    /// List the backups of the save file, newest first: time, size, path.
    List {
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// The profile (ae_prof.sav) instead of the saved game.
        #[arg(long)]
        profile: bool,
        /// Backup folder [default: the editor's data folder, see `ftled where`].
        #[arg(long)]
        backup_dir: Option<PathBuf>,
    },
    /// Write a backup over the save file.
    ///
    /// The restore follows the steps of a save: the backup must decode and
    /// encode back to its own bytes, FTL must not have written the file
    /// meanwhile, and the current file is backed up first.
    Restore {
        /// A path, or a file name in the backup folder (as `list` prints it).
        backup: PathBuf,
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// The profile (ae_prof.sav) instead of the saved game.
        #[arg(long)]
        profile: bool,
        /// Backup folder [default: the editor's data folder, see `ftled where`].
        #[arg(long)]
        backup_dir: Option<PathBuf>,
    },
}

fn dir_or_default(dir: Option<PathBuf>) -> Result<PathBuf> {
    match dir {
        Some(d) => Ok(d),
        None => Ok(ftl_app::default_backup_dir()?),
    }
}

/// `--file`, else FTL's profile or saved game of this OS.
fn target(file: Option<PathBuf>, profile: bool) -> Result<PathBuf> {
    match (file, profile) {
        (Some(f), _) => Ok(f),
        (None, true) => Ok(ftl_app::default_profile_file()?),
        (None, false) => file_or_default(None),
    }
}

pub fn run(command: BackupsCommand) -> Result<()> {
    match command {
        BackupsCommand::List {
            file,
            profile,
            backup_dir,
        } => {
            let file = target(file, profile)?;
            let dir = dir_or_default(backup_dir)?;
            let list = ftl_app::list_backups(&dir, &file)?;
            if list.is_empty() {
                println!("no backups of {} in {}", file.display(), dir.display());
            }
            for b in list {
                let time = jiff::Timestamp::try_from(b.modified)?
                    .to_zoned(jiff::tz::TimeZone::system())
                    .strftime("%Y-%m-%d %H:%M:%S");
                println!("{time}  {:>8} bytes  {}", b.bytes, b.path.display());
            }
        }
        BackupsCommand::Restore {
            backup,
            file,
            profile,
            backup_dir,
        } => {
            let dir = dir_or_default(backup_dir)?;
            let backup = if backup.components().count() == 1 && !backup.exists() {
                dir.join(backup)
            } else {
                backup
            };
            let opts = SaveOptions {
                backup_dir: Some(dir),
            };
            let file = target(file, profile)?;
            let report = if profile {
                ProfileFile::open(file)?.restore(&backup, &opts)?
            } else {
                let mut f = ftl_app::SaveFile::open(file)?;
                crate::print_warnings(&f);
                f.restore(&backup, &opts)?
            };
            println!(
                "restored {} to {} ({} bytes)",
                backup.display(),
                report.path.display(),
                report.bytes
            );
            if let Some(b) = report.backup {
                println!("backup {}", b.display());
            }
        }
    }
    Ok(())
}
