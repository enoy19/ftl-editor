//! Warnings for an opened saved game: conditions that do not stop a save, but
//! can undo it or overwrite it.
//!
//! - A running FTL writes `continue.sav` when it quits and overwrites an edit.
//! - Steam Cloud keeps its own copy of the save folder when
//!   `steam_autocloud.vdf` lies in it, and can restore the old file.

use std::fmt;
use std::path::{Path, PathBuf};

/// File names of the FTL game executable on macOS, Windows and Linux.
/// macOS: `FTL.app/Contents/MacOS/FTL` (checked against the Steam bundle).
pub const FTL_PROCESS_NAMES: &[&str] = &["FTL", "FTLGame.exe", "FTL.amd64", "FTL.x86"];

/// The marker Steam writes into a folder it synchronises with Steam Cloud.
pub const STEAM_CLOUD_MARKER: &str = "steam_autocloud.vdf";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    /// An FTL game process runs. It writes the save file when it quits.
    FtlRunning { pid: u32 },
    /// Steam Cloud synchronises the folder of the save file.
    SteamCloud { marker: PathBuf },
    /// The process list could not be read, so a running FTL is unknown.
    ProcessListUnavailable { detail: String },
}

impl Warning {
    /// Stable code for a UI that shows its own text, like [`crate::AppError::code`].
    pub fn code(&self) -> &'static str {
        match self {
            Warning::FtlRunning { .. } => "ftl_running",
            Warning::SteamCloud { .. } => "steam_cloud",
            Warning::ProcessListUnavailable { .. } => "process_list_unavailable",
        }
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Warning::FtlRunning { pid } => write!(
                f,
                "FTL is running (pid {pid}); it writes continue.sav when it quits and overwrites this edit. Quit FTL first."
            ),
            Warning::SteamCloud { marker } => write!(
                f,
                "Steam Cloud synchronises this folder ({}); Steam can restore the old file. Turn off Steam Cloud for FTL to keep the edit.",
                marker.display()
            ),
            Warning::ProcessListUnavailable { detail } => write!(
                f,
                "cannot read the process list ({detail}); a running FTL is unknown. Quit FTL before you save."
            ),
        }
    }
}

/// One running process: its id and the file name of its executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

/// Port: the processes that run on this machine now.
pub trait ProcessList {
    /// `Err` carries the reason the list could not be read.
    fn processes(&self) -> Result<Vec<ProcessInfo>, String>;
}

/// The real process list of this OS, read through `sysinfo`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemProcesses;

impl ProcessList for SystemProcesses {
    fn processes(&self) -> Result<Vec<ProcessInfo>, String> {
        use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
        let mut sys = System::new().map_err(|e| e.to_string())?;
        // Names only: the executable path needs more rights on some systems.
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::Never),
        );
        Ok(sys
            .processes()
            .iter()
            .map(|(pid, p)| ProcessInfo {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().into_owned(),
            })
            .collect())
    }
}

/// The warnings for the save file at `save_path`, in a stable order: running
/// FTL processes by pid, then Steam Cloud.
pub fn warnings(save_path: &Path, processes: &dyn ProcessList) -> Vec<Warning> {
    let mut out = match processes.processes() {
        Ok(list) => {
            let mut pids: Vec<u32> = list
                .into_iter()
                .filter(|p| FTL_PROCESS_NAMES.contains(&p.name.as_str()))
                .map(|p| p.pid)
                .collect();
            pids.sort_unstable();
            pids.into_iter()
                .map(|pid| Warning::FtlRunning { pid })
                .collect()
        }
        Err(detail) => vec![Warning::ProcessListUnavailable { detail }],
    };
    let folder = save_path.parent().unwrap_or(Path::new("."));
    let marker = folder.join(STEAM_CLOUD_MARKER);
    if marker.is_file() {
        out.push(Warning::SteamCloud { marker });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct Fake(Vec<(u32, &'static str)>);

    impl ProcessList for Fake {
        fn processes(&self) -> Result<Vec<ProcessInfo>, String> {
            Ok(self
                .0
                .iter()
                .map(|&(pid, name)| ProcessInfo {
                    pid,
                    name: name.into(),
                })
                .collect())
        }
    }

    struct Broken;

    impl ProcessList for Broken {
        fn processes(&self) -> Result<Vec<ProcessInfo>, String> {
            Err("unsupported".into())
        }
    }

    fn save_in(dir: &Path) -> PathBuf {
        let path = dir.join("continue.sav");
        fs::write(&path, b"").unwrap();
        path
    }

    #[test]
    fn no_warning_without_ftl_and_without_steam_cloud() {
        let dir = tempfile::tempdir().unwrap();
        let path = save_in(dir.path());
        // Names that only contain "FTL" are no FTL game process.
        let procs = Fake(vec![
            (1, "launchd"),
            (41640, "FTL Save Editor"),
            (7, "ftled"),
        ]);
        assert_eq!(warnings(&path, &procs), vec![]);
    }

    #[test]
    fn a_running_ftl_is_a_warning_with_its_pid() {
        let dir = tempfile::tempdir().unwrap();
        let path = save_in(dir.path());
        for name in FTL_PROCESS_NAMES {
            let procs = Fake(vec![(1, "launchd"), (4242, name)]);
            let w = warnings(&path, &procs);
            assert_eq!(w, vec![Warning::FtlRunning { pid: 4242 }], "{name}");
            assert_eq!(w[0].code(), "ftl_running");
            assert!(w[0].to_string().contains("pid 4242"));
        }
    }

    #[test]
    fn steam_autocloud_vdf_in_the_save_folder_is_a_warning() {
        let dir = tempfile::tempdir().unwrap();
        let path = save_in(dir.path());
        let marker = dir.path().join(STEAM_CLOUD_MARKER);
        fs::write(&marker, b"").unwrap();
        let w = warnings(&path, &Fake(vec![]));
        assert_eq!(
            w,
            vec![Warning::SteamCloud {
                marker: marker.clone()
            }]
        );
        assert_eq!(w[0].code(), "steam_cloud");
    }

    #[test]
    fn both_warnings_come_in_a_stable_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = save_in(dir.path());
        let marker = dir.path().join(STEAM_CLOUD_MARKER);
        fs::write(&marker, b"").unwrap();
        let procs = Fake(vec![(900, "FTL"), (12, "FTLGame.exe")]);
        assert_eq!(
            warnings(&path, &procs),
            vec![
                Warning::FtlRunning { pid: 12 },
                Warning::FtlRunning { pid: 900 },
                Warning::SteamCloud { marker },
            ]
        );
    }

    #[test]
    fn an_unreadable_process_list_is_a_warning_not_silence() {
        let dir = tempfile::tempdir().unwrap();
        let path = save_in(dir.path());
        let w = warnings(&path, &Broken);
        assert_eq!(
            w,
            vec![Warning::ProcessListUnavailable {
                detail: "unsupported".into()
            }]
        );
        assert_eq!(w[0].code(), "process_list_unavailable");
    }

    #[test]
    fn the_real_process_list_holds_this_test() {
        let me = std::process::id();
        let list = SystemProcesses.processes().unwrap();
        assert!(
            list.iter().any(|p| p.pid == me),
            "pid {me} not in {} processes",
            list.len()
        );
    }
}
