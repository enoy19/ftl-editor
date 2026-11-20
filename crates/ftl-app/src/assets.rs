//! The game's own data: where `ftl.dat` lives, the path the user confirmed,
//! and loading it.
//!
//! Reading the package is `ftl-assets`; this module adds the OS paths and the
//! editor's config file and is the one door for the CLI and the bridge. Its
//! errors are separate from [`crate::AppError`], whose codes the Flutter app
//! maps one by one.
//!
//! Flow for a UI: [`AssetsConfig::locate`] answers a path and whether it was
//! remembered or found. A found path is shown for confirmation, then
//! [`AssetsConfig::remember`] stores it. Without any path, the UI asks for a
//! file and remembers that one ([`dat_in`] accepts the game folder or
//! `FTL.app` too). [`open_shared`] then reads the game data once and keeps
//! it with a cache of the images read on demand.

use std::fs;
use std::path::{Path, PathBuf};

mod cache;
pub use cache::{
    Archive, GameAssets, IMAGE_CACHE_ENTRIES, ImageId, crew_look_id, open_shared, shared,
};

pub use ftl_assets::{
    AugmentBlueprint, BitmapFont, CREW_FRAME, CrewBlueprint, CrewCount, DroneBlueprint,
    Error as ReadError, GameData, Glyph, Insets, Rect, ShipBlueprint, ShipLayout, ShipLayoutDoor,
    ShipLayoutRoom, ShipSystem, SystemBlueprint, WeaponArt, WeaponBlueprint,
};

#[derive(Debug, thiserror::Error)]
pub enum AssetsError {
    #[error("no ftl.dat found; looked at:{}", list(tried))]
    NotFound { tried: Vec<PathBuf> },
    #[error("no home directory found to look for ftl.dat")]
    NoHome,
    #[error("{path}: {detail}")]
    Config { path: PathBuf, detail: String },
    #[error("ftl.dat has no {kind} with the id {id}")]
    UnknownId { kind: &'static str, id: String },
    #[error("no ftl.dat is open; the editor runs without game images")]
    NotOpen,
    #[error(transparent)]
    Read(#[from] ReadError),
}

fn list(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| format!("\n  {}", p.display()))
        .collect()
}

impl AssetsError {
    /// Stable code for a UI that shows its own text for an error.
    pub fn code(&self) -> &'static str {
        match self {
            AssetsError::NotFound { .. } => "ftl_dat_not_found",
            AssetsError::NoHome => "no_home",
            AssetsError::Config { .. } => "config",
            AssetsError::UnknownId { .. } => "unknown_image_id",
            AssetsError::NotOpen => "ftl_dat_not_open",
            AssetsError::Read(ReadError::Missing { .. }) => "entry_missing",
            AssetsError::Read(_) => "ftl_dat_unreadable",
        }
    }

    /// The file the error is about, if any.
    pub fn path(&self) -> Option<&Path> {
        match self {
            AssetsError::Config { path, .. }
            | AssetsError::Read(ReadError::Package { path, .. }) => Some(path),
            _ => None,
        }
    }
}

/// The `ftl.dat` a person picked: the file itself, or a folder that holds it
/// directly, as `FTL.app` (`Contents/Resources`), or below `resources` or
/// `data` (the layouts of the candidates). A macOS file dialog answers
/// `FTL.app` as one item and does not open the bundle.
pub fn dat_in(picked: &Path) -> Result<PathBuf, AssetsError> {
    if picked.is_file() {
        return Ok(picked.to_owned());
    }
    let tried: Vec<PathBuf> = [
        "",
        "Contents/Resources",
        "FTL.app/Contents/Resources",
        "resources",
        "data",
    ]
    .iter()
    .map(|sub| picked.join(sub).join("ftl.dat"))
    .collect();
    match tried.iter().find(|p| p.is_file()) {
        Some(found) => Ok(found.clone()),
        None => Err(AssetsError::NotFound { tried }),
    }
}

/// Where a located `ftl.dat` came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The path the user confirmed earlier, from the config file.
    Remembered,
    /// The first existing candidate; not confirmed yet.
    Found,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located {
    pub path: PathBuf,
    pub source: Source,
}

/// The parts of the environment the candidate list depends on.
#[derive(Debug, Clone, Default)]
struct Env {
    os: &'static str,
    home: PathBuf,
    program_files_x86: Option<PathBuf>,
    program_files: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
    wine_prefix: Option<PathBuf>,
}

impl Env {
    fn current() -> Result<Env, AssetsError> {
        let home = directories::BaseDirs::new()
            .ok_or(AssetsError::NoHome)?
            .home_dir()
            .to_owned();
        let var = |name: &str| std::env::var_os(name).map(PathBuf::from);
        Ok(Env {
            os: std::env::consts::OS,
            home,
            program_files_x86: var("ProgramFiles(x86)"),
            program_files: var("ProgramFiles"),
            xdg_data_home: var("XDG_DATA_HOME"),
            // The Java editor reads WINEPREFIX as a Java system property, which
            // is never set; the environment variable is what Wine uses.
            wine_prefix: var("WINEPREFIX"),
        })
    }
}

/// The folders of `FTLUtilities.findFtlDatsDir` in the Java editor, filtered to
/// `env.os`, each joined with `ftl.dat`. Only the first macOS Steam path is
/// tested against a real install. The Java editor also accepts the split
/// `data.dat` + `resource.dat` of FTL before 1.6, which `ftl-assets` does not read.
fn candidates_for(env: &Env) -> Vec<PathBuf> {
    const STEAM: &str = "Steam/steamapps/common/FTL Faster Than Light";
    // Older Steam installs wrote the folder as SteamApps.
    const STEAM_OLD_CASE: &str = "Steam/SteamApps/common/FTL Faster Than Light";
    const GOG: &str = "GOG.com/Faster Than Light";
    const HUMBLE: &str = "FTL";
    let home = &env.home;
    let mut dirs: Vec<PathBuf> = Vec::new();
    match env.os {
        "macos" => {
            for steam in [STEAM, STEAM_OLD_CASE] {
                dirs.push(
                    home.join("Library/Application Support")
                        .join(steam)
                        .join("FTL.app/Contents/Resources"),
                );
            }
            // GOG and Humble install the bundle into /Applications.
            dirs.push(PathBuf::from("/Applications/FTL.app/Contents/Resources"));
        }
        "windows" => {
            let x86 = env
                .program_files_x86
                .clone()
                .unwrap_or_else(|| r"C:\Program Files (x86)".into());
            let x64 = env
                .program_files
                .clone()
                .unwrap_or_else(|| r"C:\Program Files".into());
            for base in [x86, x64] {
                for sub in ["", "resources"] {
                    for store in [STEAM, GOG, HUMBLE] {
                        dirs.push(base.join(store).join(sub));
                    }
                }
            }
        }
        _ => {
            let xdg = env
                .xdg_data_home
                .clone()
                .unwrap_or_else(|| home.join(".local/share"));
            for steam_root in [xdg.join("Steam"), home.join(".steam/steam")] {
                for sub in ["data", "data/resources"] {
                    for apps in ["steamapps", "SteamApps"] {
                        dirs.push(
                            steam_root
                                .join(apps)
                                .join("common/FTL Faster Than Light")
                                .join(sub),
                        );
                    }
                }
            }
            let wine = env
                .wine_prefix
                .clone()
                .unwrap_or_else(|| home.join(".wine"));
            for sub in ["", "resources"] {
                for program_files in ["Program Files (x86)", "Program Files"] {
                    for store in [GOG, HUMBLE] {
                        dirs.push(
                            wine.join("drive_c")
                                .join(program_files)
                                .join(store)
                                .join(sub),
                        );
                    }
                }
            }
        }
    }
    dirs.into_iter().map(|d| d.join("ftl.dat")).collect()
}

/// The places this OS is searched for `ftl.dat`, in order.
pub fn candidates() -> Result<Vec<PathBuf>, AssetsError> {
    Ok(candidates_for(&Env::current()?))
}

/// The editor's config file with the confirmed `ftl.dat` path. The folder is
/// injected so tests never touch the real one.
#[derive(Debug, Clone)]
pub struct AssetsConfig {
    dir: PathBuf,
}

const CONFIG_FILE: &str = "config.json";

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct ConfigFile {
    ftl_dat: Option<PathBuf>,
}

impl AssetsConfig {
    /// The OS config folder of the editor (macOS: `~/Library/Application
    /// Support/ftl-editor`, Linux: `~/.config/ftl-editor`, Windows: `%APPDATA%`).
    pub fn open_default() -> Result<AssetsConfig, AssetsError> {
        let dirs =
            directories::ProjectDirs::from("", "", "ftl-editor").ok_or(AssetsError::NoHome)?;
        Ok(AssetsConfig::in_dir(dirs.config_dir()))
    }

    pub fn in_dir(dir: impl Into<PathBuf>) -> AssetsConfig {
        AssetsConfig { dir: dir.into() }
    }

    pub fn file(&self) -> PathBuf {
        self.dir.join(CONFIG_FILE)
    }

    fn config_err(&self, detail: impl ToString) -> AssetsError {
        AssetsError::Config {
            path: self.file(),
            detail: detail.to_string(),
        }
    }

    fn read(&self) -> Result<ConfigFile, AssetsError> {
        match fs::read(self.file()) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| self.config_err(e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ConfigFile::default()),
            Err(e) => Err(self.config_err(e)),
        }
    }

    /// The confirmed path, if one was remembered (it may no longer exist).
    pub fn remembered(&self) -> Result<Option<PathBuf>, AssetsError> {
        Ok(self.read()?.ftl_dat)
    }

    /// Stores `path` as the confirmed `ftl.dat`. The file must exist.
    pub fn remember(&self, path: &Path) -> Result<(), AssetsError> {
        if !path.is_file() {
            return Err(
                self.config_err(format!("{} is no file; nothing remembered", path.display()))
            );
        }
        let mut config = self.read()?;
        config.ftl_dat = Some(path.to_owned());
        let json = serde_json::to_vec_pretty(&config).map_err(|e| self.config_err(e))?;
        fs::create_dir_all(&self.dir).map_err(|e| self.config_err(e))?;
        let tmp = self.dir.join(format!("{CONFIG_FILE}.tmp"));
        fs::write(&tmp, json).map_err(|e| self.config_err(e))?;
        fs::rename(&tmp, self.file()).map_err(|e| self.config_err(e))
    }

    /// The remembered path when it still exists, else the first existing
    /// candidate of this OS.
    pub fn locate(&self) -> Result<Located, AssetsError> {
        self.locate_among(candidates()?)
    }

    fn locate_among(&self, candidates: Vec<PathBuf>) -> Result<Located, AssetsError> {
        let remembered = self.remembered()?;
        if let Some(path) = remembered.as_ref().filter(|p| p.is_file()) {
            return Ok(Located {
                path: path.clone(),
                source: Source::Remembered,
            });
        }
        if let Some(path) = candidates.iter().find(|p| p.is_file()) {
            return Ok(Located {
                path: path.clone(),
                source: Source::Found,
            });
        }
        let tried = remembered.into_iter().chain(candidates).collect();
        Err(AssetsError::NotFound { tried })
    }
}

/// [`AssetsConfig::locate`] with the editor's own config folder.
pub fn locate() -> Result<Located, AssetsError> {
    AssetsConfig::open_default()?.locate()
}

/// Every inner path of `ftl.dat`, in package order.
pub fn entry_paths(path: impl AsRef<Path>) -> Result<Vec<String>, AssetsError> {
    Ok(ftl_assets::entry_paths(path)?)
}

/// The bytes of one entry of `ftl.dat` (opens the package per call).
pub fn read_entry(path: impl AsRef<Path>, inner_path: &str) -> Result<Vec<u8>, AssetsError> {
    Ok(ftl_assets::read_entry(path, inner_path)?)
}

/// Reads blueprints, names, layouts and sprites from `ftl.dat`.
pub fn load(path: impl AsRef<Path>) -> Result<GameData, AssetsError> {
    Ok(GameData::open(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(os: &'static str, home: &str) -> Env {
        Env {
            os,
            home: home.into(),
            ..Env::default()
        }
    }

    #[test]
    fn candidates_follow_the_java_ftl_utilities() {
        let mac = candidates_for(&env("macos", "/Users/someone"));
        assert_eq!(
            mac,
            [
                "/Users/someone/Library/Application Support/Steam/steamapps/common/FTL Faster Than Light/FTL.app/Contents/Resources/ftl.dat",
                "/Users/someone/Library/Application Support/Steam/SteamApps/common/FTL Faster Than Light/FTL.app/Contents/Resources/ftl.dat",
                "/Applications/FTL.app/Contents/Resources/ftl.dat",
            ]
            .map(PathBuf::from)
        );
        let linux = candidates_for(&env("linux", "/home/x"));
        assert_eq!(
            linux[0],
            Path::new(
                "/home/x/.local/share/Steam/steamapps/common/FTL Faster Than Light/data/ftl.dat"
            )
        );
        assert!(linux.contains(&PathBuf::from(
            "/home/x/.steam/steam/steamapps/common/FTL Faster Than Light/data/ftl.dat"
        )));
        assert!(linux.contains(&PathBuf::from(
            "/home/x/.wine/drive_c/Program Files (x86)/GOG.com/Faster Than Light/ftl.dat"
        )));
        let win = candidates_for(&env("windows", "C:/Users/x"));
        assert_eq!(win.len(), 12);
        assert!(win[0].ends_with("Steam/steamapps/common/FTL Faster Than Light/ftl.dat"));
        assert!(
            win.iter()
                .any(|p| p.ends_with("GOG.com/Faster Than Light/ftl.dat"))
        );
        assert!(win.iter().any(|p| p.ends_with("FTL/resources/ftl.dat")));
    }

    struct Setup {
        _dir: tempfile::TempDir,
        config: AssetsConfig,
        found: PathBuf,
        other: PathBuf,
        missing: PathBuf,
    }

    fn setup() -> Setup {
        let dir = tempfile::tempdir().unwrap();
        let (found, other, missing) = (
            dir.path().join("found.dat"),
            dir.path().join("other.dat"),
            dir.path().join("missing.dat"),
        );
        fs::write(&found, b"x").unwrap();
        fs::write(&other, b"x").unwrap();
        let config = AssetsConfig::in_dir(dir.path().join("config"));
        Setup {
            _dir: dir,
            config,
            found,
            other,
            missing,
        }
    }

    #[test]
    fn without_a_remembered_path_the_first_existing_candidate_is_found() {
        let s = setup();
        assert_eq!(s.config.remembered().unwrap(), None);
        let l = s
            .config
            .locate_among(vec![s.missing.clone(), s.found.clone(), s.other.clone()])
            .unwrap();
        assert_eq!((l.path, l.source), (s.found, Source::Found));
    }

    #[test]
    fn the_remembered_path_wins_over_the_candidates() {
        let s = setup();
        s.config.remember(&s.other).unwrap();
        assert!(s.config.file().is_file());
        assert_eq!(s.config.remembered().unwrap(), Some(s.other.clone()));
        let l = s.config.locate_among(vec![s.found.clone()]).unwrap();
        assert_eq!((l.path, l.source), (s.other, Source::Remembered));
    }

    #[test]
    fn a_remembered_path_that_is_gone_falls_back_to_the_candidates() {
        let s = setup();
        s.config.remember(&s.other).unwrap();
        fs::remove_file(&s.other).unwrap();
        let l = s.config.locate_among(vec![s.found.clone()]).unwrap();
        assert_eq!((l.path, l.source), (s.found, Source::Found));
    }

    #[test]
    fn nothing_found_lists_the_remembered_path_and_every_candidate() {
        let s = setup();
        s.config.remember(&s.other).unwrap();
        fs::remove_file(&s.other).unwrap();
        let err = s.config.locate_among(vec![s.missing.clone()]).unwrap_err();
        assert_eq!(err.code(), "ftl_dat_not_found");
        let AssetsError::NotFound { tried } = &err else {
            panic!("{err}")
        };
        assert_eq!(tried, &[s.other, s.missing]);
    }

    #[test]
    fn remember_refuses_a_missing_file_and_a_broken_config_is_an_error() {
        let s = setup();
        let err = s.config.remember(&s.missing).unwrap_err();
        assert_eq!(err.code(), "config");
        assert!(!s.config.file().exists());
        fs::create_dir_all(s.config.file().parent().unwrap()).unwrap();
        fs::write(s.config.file(), b"{not json").unwrap();
        let err = s.config.remembered().unwrap_err();
        assert!(err.to_string().contains("config.json"), "{err}");
    }

    #[test]
    fn a_picked_folder_or_app_bundle_resolves_to_its_ftl_dat() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("FTL.app");
        let dat = app.join("Contents/Resources/ftl.dat");
        fs::create_dir_all(dat.parent().unwrap()).unwrap();
        fs::write(&dat, b"x").unwrap();
        assert_eq!(dat_in(&dat).unwrap(), dat);
        assert_eq!(dat_in(&app).unwrap(), dat);
        assert_eq!(dat_in(dir.path()).unwrap(), dat);
        let empty = dir.path().join("empty");
        fs::create_dir(&empty).unwrap();
        let err = dat_in(&empty).unwrap_err();
        assert_eq!(err.code(), "ftl_dat_not_found");
        let AssetsError::NotFound { tried } = &err else {
            panic!("{err}")
        };
        assert_eq!(tried[0], empty.join("ftl.dat"));
        assert_eq!(tried.len(), 5);
    }

    #[test]
    fn open_shared_reads_a_path_once_and_another_path_fails_without_dropping_it() {
        let Ok(found) = AssetsConfig::in_dir("/nonexistent").locate_among(candidates().unwrap())
        else {
            println!("skip: no ftl.dat installed at {:?}", candidates().unwrap());
            return;
        };
        let first = open_shared(&found.path).unwrap();
        assert!(std::sync::Arc::ptr_eq(
            &first,
            &open_shared(&found.path).unwrap()
        ));
        assert!(std::sync::Arc::ptr_eq(&first, &shared().unwrap()));
        let err = open_shared(Path::new("/nonexistent/ftl.dat")).unwrap_err();
        assert_eq!(
            (err.code(), err.path()),
            (
                "ftl_dat_unreadable",
                Some(Path::new("/nonexistent/ftl.dat"))
            )
        );
        assert!(
            std::sync::Arc::ptr_eq(&first, &shared().unwrap()),
            "a failed open keeps the open data"
        );
    }

    #[test]
    fn the_local_steam_install_is_found_when_there_is_one() {
        let s = setup();
        let Ok(found) = s.config.locate_among(candidates().unwrap()) else {
            println!("skip: no ftl.dat installed at {:?}", candidates().unwrap());
            return;
        };
        assert_eq!(found.source, Source::Found);
        assert!(
            load(&found.path)
                .unwrap()
                .ship("PLAYER_SHIP_HARD")
                .is_some()
        );
    }
}
