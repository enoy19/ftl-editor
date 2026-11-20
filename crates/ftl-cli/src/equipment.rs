//! `ftled equipment ...`: reactor, systems, weapons, drones and augments.

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use ftl_app::{SaveFile, assets, equipment};

use crate::{SetArgs, file_or_default, print_warnings, run_set};

#[derive(Subcommand)]
pub enum EquipmentCommand {
    /// Print the reactor and each system as `level/max` with its path, then
    /// the weapons and drones (armed ones marked), the cargo and the
    /// augments with their blueprint ids.
    ///
    /// Names and the level caps of the ship blueprint come from the located
    /// ftl.dat; without one the ids stand alone and the caps are the ones of
    /// unmodded FTL. A system that is not installed shows `-` and whether
    /// `systems.<id>.installed=1` can install it. `--nearby` lists the ship
    /// at the current beacon, whose paths start with `nearby.`.
    List {
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// The nearby ship instead of the player ship.
        #[arg(long)]
        nearby: bool,
    },
    /// List every augment of the game: id and name (needs ftl.dat).
    Augments,
    /// Edit equipment fields and save; the same command as `ftled set`.
    ///
    /// Paths: `reactor` (0 to 25; below the bars in use it takes power from
    /// the systems, the last of the file first), `systems.<id>.level` (1 up
    /// to its maximum; below its power it lowers the power),
    /// `systems.<id>.damage` (broken bars, 0 to the level; a broken bar holds
    /// no power), `systems.<id>.ion` (ion lock count, 0 to 5, a fresh lock
    /// timer), `systems.shields.energy` (Zoltan shield points, 0 to 5; the
    /// enemy up to its own max), `augments.<ID>` (how many, at most 3 in all),
    /// `weapons.<ID>`, `drones.<ID>` (how many in the slots), `weapons.<ID>.armed`,
    /// `drones.<ID>.armed` (how many armed) and `cargo.<ID>` (how many in the cargo, 4 in all). Example:
    /// `ftled equipment set systems.shields.level=8 augments.SCRAP_COLLECTOR=1`.
    Set(SetArgs),
}

pub fn game_data() -> Option<ftl_app::assets::GameData> {
    // Names are a convenience; a missing ftl.dat only drops them.
    assets::locate()
        .ok()
        .and_then(|l| assets::load(l.path).ok())
}

pub fn run(command: EquipmentCommand) -> Result<()> {
    match command {
        EquipmentCommand::List { file, nearby } => {
            let f = SaveFile::open(file_or_default(file)?)?;
            print_warnings(&f);
            let data = game_data();
            let on = crate::ship_of(nearby);
            let Some(l) = equipment::loadout(f.game(), on, data.as_ref(), f.layouts(), "en") else {
                anyhow::bail!("no ship is at the current beacon, or the situation is not decoded");
            };
            let p = on.prefix();
            println!(
                "{p}reactor  {}/{} (in use {})",
                l.reactor.value, l.reactor.max, l.reactor_in_use
            );
            for s in &l.systems {
                let name = s.name.as_deref().unwrap_or(s.kind.id());
                match &s.level {
                    Some(b) => println!(
                        "{}  {}/{}  {name} (power {}, min {}, damaged {}, ion {})",
                        b.field, b.value, b.max, s.power, b.min, s.damaged, s.ionized
                    ),
                    None => match s.block {
                        None => println!(
                            "{p}systems.{}.level  -  {name} (not installed; {}=1 installs it)",
                            s.kind.id(),
                            s.installed.field
                        ),
                        Some(b) => println!(
                            "{p}systems.{}.level  -  {name} (not installed; cannot install: {})",
                            s.kind.id(),
                            b.reason()
                        ),
                    },
                }
            }
            if !l.body_decoded {
                println!("weapons, drones, augments: not decoded (no ship layout)");
                return Ok(());
            }
            for (title, items, slots) in [
                ("weapons", &l.weapons, l.weapon_slots),
                ("drones", &l.drones, l.drone_slots),
            ] {
                let slots = slots.map_or(String::new(), |s| format!(" of {s}"));
                println!("{title}  {}{slots}", items.len());
                for i in items {
                    let armed = if i.armed { "  armed" } else { "" };
                    println!("  {}  {}{armed}", i.id, i.name.as_deref().unwrap_or(""));
                }
            }
            match &l.cargo {
                None if nearby => {}
                None => println!("cargo: not decoded"),
                Some(cargo) => {
                    println!("cargo  {} of {}", cargo.len(), l.cargo_slots);
                    for i in cargo {
                        println!("  {}  {}", i.id, i.name.as_deref().unwrap_or(""));
                    }
                }
            }
            let used: i32 = l.augments.iter().map(|a| a.count.value).sum();
            println!("augments  {used} of {}", l.augment_slots);
            for a in &l.augments {
                println!(
                    "  {}={}  {}",
                    a.count.field,
                    a.count.value,
                    a.name.as_deref().unwrap_or("")
                );
            }
        }
        EquipmentCommand::Augments => {
            let data = game_data();
            if data.is_none() {
                anyhow::bail!("no readable ftl.dat found; `ftled assets` names where it looked");
            }
            for c in equipment::augment_choices(data.as_ref(), "en") {
                println!("{}  {}", c.id, c.name);
            }
        }
        EquipmentCommand::Set(args) => run_set(args)?,
    }
    Ok(())
}
