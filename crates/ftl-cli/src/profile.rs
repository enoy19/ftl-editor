//! `ftled profile`: read and edit the FTL profile (`ae_prof.sav`).
//!
//! Without `--file`, every command uses the `ae_prof.sav` in the FTL save
//! folder of this OS. A save follows the same rules as `ftled set`: backup,
//! changed-on-disk check, re-decode before the write.

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use ftl_app::profile::{Layout, PLAYER_SHIPS, PROFILE_PATHS};
use ftl_app::{Profile, ProfileAssignment, ProfileFile, ProfilePatch, SaveOptions};

#[derive(Subcommand)]
pub enum ProfileCommand {
    /// Print ship unlocks, achievements, scores and stats of the profile.
    Show {
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Print JSON instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Unlock Type-A, B and C of every ship and save.
    ///
    /// Type-B has no flag in the file: FTL unlocks it from two of the ship's
    /// three achievements, so missing ship achievements are added on Easy.
    /// Lanius and Crystal have no Type-C.
    UnlockAll {
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Show the changes, write nothing.
        #[arg(long)]
        dry_run: bool,
        /// Backup folder [default: the editor's data folder, see `ftled where`].
        #[arg(long)]
        backup_dir: Option<PathBuf>,
    },
    /// Set profile fields and save, all or none: `ships.<SHIP_ID>.<a|b|c>=0|1`,
    /// `achievements.<ID>=0..3` (0 removes, 1 Easy, 2 Normal, 3 Hard).
    ///
    /// `ships.<ID>.b=1` adds missing ship achievements on Easy until two of
    /// three are present; `=0` removes them until one is left.
    Set {
        /// Assignments such as `ships.PLAYER_SHIP_HARD.c=1`.
        #[arg(required = true)]
        edits: Vec<ProfileAssignment>,
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Show the changes, write nothing.
        #[arg(long)]
        dry_run: bool,
        /// Backup folder [default: the editor's data folder, see `ftled where`].
        #[arg(long)]
        backup_dir: Option<PathBuf>,
    },
    /// List the editable profile paths.
    Fields,
}

fn write(f: &mut ProfileFile, dry_run: bool, backup_dir: Option<PathBuf>) -> Result<()> {
    if dry_run {
        f.verify()?;
        println!("dry run: verified, nothing written");
        return Ok(());
    }
    let backup_dir = Some(match backup_dir {
        Some(d) => d,
        None => ftl_app::default_backup_dir()?,
    });
    let report = f.save(&SaveOptions { backup_dir })?;
    println!("wrote {} ({} bytes)", report.path.display(), report.bytes);
    if let Some(b) = report.backup {
        println!("backup {}", b.display());
    }
    Ok(())
}

fn file_or_default(file: Option<PathBuf>) -> Result<PathBuf> {
    match file {
        Some(f) => Ok(f),
        None => Ok(ftl_app::default_profile_file()?),
    }
}

pub fn run(command: ProfileCommand) -> Result<()> {
    match command {
        ProfileCommand::Show { file, json } => {
            let f = ProfileFile::open(file_or_default(file)?)?;
            if json {
                println!("{}", serde_json::to_string_pretty(f.profile())?);
            } else {
                println!("file:         {}", f.path().display());
                print_profile(f.profile());
            }
        }
        ProfileCommand::UnlockAll {
            file,
            dry_run,
            backup_dir,
        } => {
            let mut f = ProfileFile::open(file_or_default(file)?)?;
            let changes = f.unlock_all_ships();
            for c in &changes {
                let layout = match c.layout {
                    Layout::A => "A",
                    Layout::B => "B",
                    Layout::C => "C",
                };
                if c.added_achievements.is_empty() {
                    println!("{}: unlock Type-{layout}", c.ship_id);
                } else {
                    println!(
                        "{}: unlock Type-{layout} (adds {} on Easy)",
                        c.ship_id,
                        c.added_achievements.join(", ")
                    );
                }
            }
            if changes.is_empty() {
                println!("every ship layout is already unlocked; nothing to write");
                return Ok(());
            }
            write(&mut f, dry_run, backup_dir)?;
        }
        ProfileCommand::Set {
            edits,
            file,
            dry_run,
            backup_dir,
        } => {
            let mut f = ProfileFile::open(file_or_default(file)?)?;
            for c in f.apply(&ProfilePatch::new(edits))? {
                let mut line = format!("{}: {} -> {}", c.field, c.old, c.new);
                if !c.added_achievements.is_empty() {
                    line += &format!(" (adds {} on Easy)", c.added_achievements.join(", "));
                }
                if !c.removed_achievements.is_empty() {
                    line += &format!(" (removes {})", c.removed_achievements.join(", "));
                }
                println!("{line}");
            }
            write(&mut f, dry_run, backup_dir)?;
        }
        ProfileCommand::Fields => {
            for p in PROFILE_PATHS {
                println!("{p}");
            }
        }
    }
    Ok(())
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "-" }
}

fn print_profile(p: &Profile) {
    println!("format:       {}", p.file_format);
    if let Some(level) = p.newbie_tip_level {
        println!("newbie tips:  {level:?}");
    }
    println!();
    println!("{:<24}{:<4}{:<4}{:<4}", "ship", "A", "B", "C");
    for ship in PLAYER_SHIPS {
        if let Some(s) = p.ship_unlocks(ship.id) {
            let c = if ship.has_layout_c {
                yes(s.layout_c)
            } else {
                "n/a"
            };
            println!(
                "{:<24}{:<4}{:<4}{:<4}",
                ship.id,
                yes(s.layout_a),
                yes(s.layout_b),
                c
            );
        }
    }
    println!();
    println!("achievements: {}", p.achievements.len());
    for a in &p.achievements {
        match a.layouts {
            Some(l) => {
                let l = l.map(|d| d.map_or("-".to_owned(), |d| format!("{d:?}")));
                println!(
                    "  {:<32}{:?} (A {}, B {}, C {})",
                    a.id, a.difficulty, l[0], l[1], l[2]
                );
            }
            None => println!("  {:<32}{:?}", a.id, a.difficulty),
        }
    }
    println!();
    println!("top scores:");
    for s in &p.top_scores {
        println!(
            "  {:>6}  {} ({}), sector {}, {:?}{}",
            s.value,
            s.ship_name,
            s.ship_id,
            s.sector,
            s.difficulty,
            if s.victory { ", victory" } else { "" }
        );
    }
    println!("ship best:");
    for s in &p.ship_best {
        println!("  {:>6}  {} ({})", s.value, s.ship_name, s.ship_id);
    }
    let st = &p.stats;
    println!();
    println!("stats:                      most    total");
    println!(
        "  ships defeated        {:>8} {:>8}",
        st.most_ships_defeated, st.total_ships_defeated
    );
    println!(
        "  beacons explored      {:>8} {:>8}",
        st.most_beacons_explored, st.total_beacons_explored
    );
    println!(
        "  scrap collected       {:>8} {:>8}",
        st.most_scrap_collected, st.total_scrap_collected
    );
    println!(
        "  crew hired            {:>8} {:>8}",
        st.most_crew_hired, st.total_crew_hired
    );
    println!(
        "  games played                   {:>8}",
        st.total_games_played
    );
    println!("  victories                      {:>8}", st.total_victories);
    let c = &p.crew_records;
    println!("crew records:");
    for (label, r) in [
        ("repairs", &c.most_repairs),
        ("combat kills", &c.most_combat_kills),
        ("piloted evasions", &c.most_piloted_evasions),
        ("jumps survived", &c.most_jumps_survived),
        ("skill masteries", &c.most_skill_masteries),
    ] {
        println!("  {label:<20}{:>6}  {} ({})", r.value, r.name, r.race);
    }
}
