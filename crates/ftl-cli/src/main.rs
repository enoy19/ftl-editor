//! ftled — read and edit FTL saved games.
//!
//! `ftled --help` and `ftled <command> --help` are the full contract.
//! Without `--file`, every command uses the `continue.sav` of this OS.
//! Exit codes: 0 success, 1 any error (the message names file, offset or field).

mod assets;
mod backups;
mod crew;
mod equipment;
mod map;
mod profile;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use ftl_app::{
    Assignment, Field, INDEXED_PATHS, LayoutSource, Patch, SaveFile, SaveOptions, SystemProcesses,
};

#[derive(Parser)]
#[command(
    name = "ftled",
    version,
    about = "Read and edit FTL: Faster Than Light saved games"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the decoded saved game. Warnings (running FTL, Steam Cloud) go to stderr.
    Show {
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Print JSON instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Print every decoded field as a tree: path, type, value, range and
    /// the edit path (`raw.<path>`, `state.<id>`), then the undecoded bytes
    /// as hex. The Expert area of the app shows the same tree.
    ///
    /// `ftled set raw.<path>=<n>` edits any number or flag within its type,
    /// `ftled set state.<id>=<n>` a state var. Fields named `unknown_*` are
    /// fields whose meaning nobody knows yet: a raw edit can break the save
    /// for FTL, although the verify step still refuses bytes that do not
    /// decode back to the edited game.
    Dump {
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Print JSON (`entries`, `undecoded_hex`) instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Print the value of one field, for example `ship.scrap` or `crew.0.skills.pilot`.
    Get {
        field: Field,
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Set fields and save: `ftled set ship.scrap=9999 crew.0.skills.pilot=30`.
    ///
    /// `augments.<ID>=<count>`, `beacons.<n>.seen=<0|1>`, the map paths of `ftled map set`, `nearby.<ship field>` (`nearby.ship.hull`),
    /// `fires.<x>.<y>` (0 puts a fire out, 100 lights one) and
    /// `breaches.<x>.<y>` (0 seals a breach, 100 adds one) at a room square
    /// of the ship grid and `rooms.<n>.oxygen` (0 to 100 percent, on a ship
    /// with the oxygen system) need the decoded ship body, which needs the ship layouts of
    /// the located ftl.dat (`ftled show` names it).
    ///
    /// `weapons.<ID>=<count>` and `drones.<ID>=<count>` put weapons and drones
    /// into the slots of the ship blueprint or take the last copies out;
    /// `weapons.<ID>.armed` and `drones.<ID>.armed` arm that many copies within
    /// the free bars of the system and the reactor (a repair or anti-personnel
    /// drone is armed in FTL); `cargo.<ID>=<count>` fills the four cargo boxes.
    /// A move from cargo to a slot is two assignments: `cargo.ION_4=0 weapons.ION_4=1`.
    ///
    /// All assignments apply or none does. Before writing, ftled backs up the
    /// old file, checks that FTL has not written it meanwhile, and verifies
    /// that the new bytes decode to the edited game. It warns on stderr when
    /// FTL runs or Steam Cloud syncs the folder, and still writes.
    Set(SetArgs),
    /// Check that the file decodes and encodes back to the same bytes.
    Verify {
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// List the editable fields: the fixed ones with their ranges, then the
    /// path patterns of the indexed ones (their range depends on the game;
    /// `ftled set --dry-run` names it).
    Fields,
    /// Print the default save file and backup folder of this OS.
    Where,
    /// The game's own data in ftl.dat.
    #[command(subcommand)]
    Assets(assets::AssetsCommand),
    /// List the player ships with their names, read from ftl.dat.
    Ships(assets::ShipsArgs),
    /// List the backups of a save file and restore one.
    Backups {
        #[command(subcommand)]
        command: backups::BackupsCommand,
    },
    /// Read and edit the profile (ae_prof.sav): ship unlocks, achievements, stats.
    Profile {
        #[command(subcommand)]
        command: profile::ProfileCommand,
    },
    /// List the crew of the player ship and edit skills and health.
    #[command(subcommand)]
    Crew(crew::CrewCommand),
    /// List reactor, systems, weapons, drones and augments and edit them.
    #[command(subcommand)]
    Equipment(equipment::EquipmentCommand),
    /// List the sector map (beacons, stores, rebel fleet) and edit it.
    #[command(subcommand)]
    Map(map::MapCommand),
    /// Run a one-click action and save, as the app's action buttons do.
    ///
    /// Actions: `max-everything` (fuel, drone parts, missiles and scrap to
    /// the editor caps, hull to the ship's maximum, every installed system
    /// to its blueprint maximum, reactor to 25), `crew-fully-trained` (every
    /// skill to the second mastery of the race), `repair-ship` (hull to the
    /// ship's maximum, every fire out, every breach sealed, every room's
    /// oxygen at 100) and `heal-crew`
    /// (every crew member to full health). An action only improves the ship.
    /// The ship's limits come from the located ftl.dat; without one the hull
    /// and the system levels go to the editor caps, and ftled says so on
    /// stderr, as for broken system bars, which have no edit path yet.
    /// Saving works as in `ftled set`.
    Action(ActionArgs),
}

/// The arguments of `ftled action`.
#[derive(Args)]
pub struct ActionArgs {
    action: ftl_app::actions::Action,
    #[command(flatten)]
    save: SaveArgs,
}

/// The arguments of `ftled set` and `ftled crew set`.
#[derive(Args)]
pub struct SetArgs {
    #[arg(required = true, value_name = "FIELD=VALUE")]
    assignments: Vec<Assignment>,
    #[command(flatten)]
    save: SaveArgs,
}

/// File and save options of `ftled set` and `ftled action`.
#[derive(Args)]
pub struct SaveArgs {
    #[arg(short, long)]
    file: Option<PathBuf>,
    /// Show the changes, write nothing.
    #[arg(long)]
    dry_run: bool,
    /// Write no backup copy.
    #[arg(long)]
    no_backup: bool,
    /// Backup folder [default: the editor's data folder, see `ftled where`].
    #[arg(long)]
    backup_dir: Option<PathBuf>,
}

/// Applies the assignments and saves through `ftl-app`, as `ftled set --help` says.
fn run_set(args: SetArgs) -> Result<()> {
    let SetArgs { assignments, save } = args;
    let mut f = SaveFile::open(file_or_default(save.file.clone())?)?;
    print_warnings(&f);
    apply_and_save(&mut f, &Patch::new(assignments), save)
}

/// Plans the action with the limits of the located ftl.dat, then saves like `set`.
fn run_action(args: ActionArgs) -> Result<()> {
    let ActionArgs { action, save } = args;
    let mut f = SaveFile::open(file_or_default(save.file.clone())?)?;
    print_warnings(&f);
    let data = equipment::game_data();
    let plan = ftl_app::actions::plan(action, f.game(), data.as_ref());
    for note in &plan.notes {
        eprintln!("note[{}]: {note}", note.code());
    }
    if plan.patch.assignments.is_empty() {
        println!("{action}: nothing to change");
        return Ok(());
    }
    apply_and_save(&mut f, &plan.patch, save)
}

fn apply_and_save(f: &mut SaveFile, patch: &Patch, save: SaveArgs) -> Result<()> {
    let SaveArgs {
        file: _,
        dry_run,
        no_backup,
        backup_dir,
    } = save;
    let changes = f.apply(patch)?;
    for c in &changes {
        println!("{}: {} -> {}", c.field, c.old, c.new);
    }
    if dry_run {
        f.verify()?;
        println!("dry run: verified, nothing written");
        return Ok(());
    }
    let backup_dir = match (no_backup, backup_dir) {
        (true, _) => None,
        (false, Some(d)) => Some(d),
        (false, None) => Some(ftl_app::default_backup_dir()?),
    };
    let report = f.save(&SaveOptions { backup_dir })?;
    println!("wrote {} ({} bytes)", report.path.display(), report.bytes);
    match report.backup {
        Some(b) => println!("backup {}", b.display()),
        None => println!("no backup"),
    }
    Ok(())
}

/// `ftled dump`: one line per entry, indented by depth, or JSON.
fn print_dump(g: &ftl_app::SavedGame, json: bool) -> Result<()> {
    let entries = ftl_app::raw::entries(g);
    let hex = ftl_app::raw::hex_lines(&g.undecoded);
    if json {
        let dump = serde_json::json!({ "entries": entries, "undecoded_hex": hex });
        println!("{}", serde_json::to_string_pretty(&dump)?);
        return Ok(());
    }
    for e in &entries {
        let name = e.path.rsplit('.').next().unwrap_or(&e.path);
        let mut line = format!(
            "{:indent$}{name}: {} {}",
            "",
            e.kind.name(),
            e.value,
            indent = 2 * e.depth
        );
        if let (Some(field), Some((min, max))) = (&e.field, e.range) {
            line.push_str(&format!("  [{field} {min}..={max}]"));
        }
        if e.unknown {
            line.push_str("  (unknown)");
        }
        println!("{}", line.trim_end());
    }
    if !hex.is_empty() {
        println!("undecoded bytes ({}):", g.undecoded.len());
        for l in hex {
            println!("  {l}");
        }
    }
    Ok(())
}

/// The ship a `--nearby` flag picks.
fn ship_of(nearby: bool) -> ftl_app::ShipId {
    if nearby {
        ftl_app::ShipId::Nearby
    } else {
        ftl_app::ShipId::Player
    }
}

fn file_or_default(file: Option<PathBuf>) -> Result<PathBuf> {
    match file {
        Some(f) => Ok(f),
        None => Ok(ftl_app::default_save_file()?),
    }
}

/// Prints the warnings for an opened file to stderr, one line each with its code.
fn print_warnings(f: &SaveFile) {
    for w in ftl_app::warnings(f.path(), &SystemProcesses) {
        eprintln!("warning[{}]: {w}", w.code());
    }
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Show { file, json } => {
            let f = SaveFile::open(file_or_default(file)?)?;
            print_warnings(&f);
            let g = f.game();
            if json {
                println!("{}", serde_json::to_string_pretty(g)?);
            } else {
                let s = &g.player_ship;
                println!("file:       {}", f.path().display());
                println!(
                    "format:     {} (dlc {}, {:?})",
                    g.file_format, g.dlc_enabled, g.difficulty
                );
                println!("ship:       {} ({})", s.name, s.blueprint_id);
                println!(
                    "crew:       {}",
                    s.crew
                        .iter()
                        .map(|c| format!("{} ({})", c.name, c.race))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!("sector:     {}", g.sector_number);
                for field in Field::ALL {
                    println!("{:<20}{}", format!("{field}:"), field.get(g));
                }
                match f.layout_source() {
                    LayoutSource::FtlDat(p) => println!("layouts:    {}", p.display()),
                    LayoutSource::Given => println!("layouts:    given"),
                    LayoutSource::None { reason } => println!(
                        "layouts:    none, everything from the player ship rooms on stays undecoded ({})",
                        reason.replace('\n', " ")
                    ),
                }
                println!("undecoded:  {} bytes", g.undecoded.len());
            }
        }
        Command::Dump { file, json } => {
            let f = SaveFile::open(file_or_default(file)?)?;
            print_warnings(&f);
            print_dump(f.game(), json)?;
        }
        Command::Get { field, file } => {
            let f = SaveFile::open(file_or_default(file)?)?;
            println!("{}", field.get(f.game())?);
        }
        Command::Set(args) => run_set(args)?,
        Command::Verify { file } => {
            let path = file_or_default(file)?;
            let f = SaveFile::open(&path)?;
            anyhow::ensure!(
                !f.is_dirty(),
                "{}: encoding the decoded game gives other bytes",
                path.display()
            );
            f.verify().context("re-decode")?;
            println!("ok: {} round-trips byte-identical", path.display());
        }
        Command::Fields => {
            for field in Field::ALL {
                let (min, max) = field.range();
                println!("{:<20}{min}..={max}", field.path());
            }
            for pattern in INDEXED_PATHS {
                println!("{pattern}");
            }
        }
        Command::Where => {
            println!("save file:  {}", ftl_app::default_save_file()?.display());
            println!("profile:    {}", ftl_app::default_profile_file()?.display());
            println!("backups:    {}", ftl_app::default_backup_dir()?.display());
        }
        Command::Assets(c) => assets::run_assets(c)?,
        Command::Ships(a) => assets::run_ships(a)?,
        Command::Backups { command } => backups::run(command)?,
        Command::Profile { command } => profile::run(command)?,
        Command::Crew(c) => crew::run(c)?,
        Command::Equipment(c) => equipment::run(c)?,
        Command::Map(c) => map::run(c)?,
        Command::Action(a) => run_action(a)?,
    }
    Ok(())
}
