//! `ftled crew ...`: the crew of the player ship or the nearby ship.

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use ftl_app::{SaveFile, assets, crew};

use crate::{SetArgs, file_or_default, print_warnings, run_set};

#[derive(Subcommand)]
pub enum CrewCommand {
    /// Print one block per crew member: index, name, race, health and the six
    /// skills as `points/max` with the mastery level (0, 1 or 2) in brackets.
    ///
    /// The race title comes from the located ftl.dat; without one the race id
    /// stands alone. Enemy boarding drones in the crew list are left out.
    /// `--nearby` lists the crew of the ship at the current beacon, whose
    /// paths start with `nearby.`.
    List {
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// The nearby ship instead of the player ship.
        #[arg(long)]
        nearby: bool,
    },
    /// Edit crew fields and save; the same command as `ftled set`.
    ///
    /// Paths: `crew.<n>.skills.<pilot|engine|shield|weapon|repair|combat>`
    /// (0 to two mastery intervals of the race; one interval is mastery 1)
    /// and `crew.<n>.health` (1 to the full health of the race); text:
    /// `crew.<n>.name` (1 to 32 characters) and `crew.<n>.race` (human, engi,
    /// mantis, slug, rock, energy, crystal, anaerobic; keeps the health share
    /// and each mastery level). `crew list` prints `<n>` and every range.
    /// Example: `ftled crew set crew.0.skills.pilot=30 "crew.2.name=Ann Lee"`.
    Set(SetArgs),
}

pub fn run(command: CrewCommand) -> Result<()> {
    match command {
        CrewCommand::List { file, nearby } => {
            let f = SaveFile::open(file_or_default(file)?)?;
            print_warnings(&f);
            // Race titles are a convenience; a missing ftl.dat only drops them.
            let data = assets::locate()
                .ok()
                .and_then(|l| assets::load(l.path).ok());
            let on = crate::ship_of(nearby);
            for c in crew::roster(f.game(), on, data.as_ref()) {
                let race = match &c.race_name {
                    Some(name) => format!("{name} ({})", c.race),
                    None => c.race.clone(),
                };
                println!("{}crew.{}  {}  {race}", on.prefix(), c.index, c.name);
                println!("  health  {}/{}", c.health.value, c.health.max);
                for s in &c.skills {
                    println!(
                        "  {:<7} {}/{} [{}]",
                        s.kind.id(),
                        s.points.value,
                        s.points.max,
                        s.mastery
                    );
                }
            }
        }
        CrewCommand::Set(args) => run_set(args)?,
    }
    Ok(())
}
