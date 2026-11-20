//! `ftled map ...`: the sector map, its beacons, stores and the rebel fleet.

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use ftl_app::map::{sector_tree_view, sector_view};
use ftl_app::{SaveFile, assets};

use crate::{SetArgs, file_or_default, print_warnings, run_set};

#[derive(Subcommand)]
pub enum MapCommand {
    /// Print the sector, the rebel fleet and one line per beacon: id,
    /// position on the sector map (from the layout seed), visited, seen,
    /// fleet, enemy and quest; then the store of a beacon with its slots
    /// (`beacons.<n>.store.<shelf>.<slot>`, count 0 sold out or 1) and
    /// supplies. Item names come from the located ftl.dat.
    List {
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Print the sector tree FTL rebuilds from the tree seed and the sector
    /// lists of ftl.dat: one line per sector with its index (the value of
    /// `sector.jump`), level, kind, name, the sectors it leads to, and
    /// whether it is visited, current or blocked for a jump.
    Tree {
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Edit the map and save; the same command as `ftled set`.
    ///
    /// Paths: `fleet.offset` (-959, a new sector, to 60; sets the fleet
    /// flag of every beacon), `beacons.<n>.visited=0` (the beacon's event
    /// comes back; never the current beacon), `beacons.<n>.seen`,
    /// `beacons.<n>.store.<shelf>.<slot>` (0 or 1) and
    /// `beacons.<n>.store.<fuel|missiles|drone_parts>` (0 to 99),
    /// `sector.jump=<index of ftled map tree>` (an arrival in that sector).
    /// Example: `ftled map set fleet.offset=-959 beacons.3.store.0.1=1`.
    Set(SetArgs),
}

pub fn run(command: MapCommand) -> Result<()> {
    match command {
        MapCommand::Tree { file } => {
            let f = SaveFile::open(file_or_default(file)?)?;
            print_warnings(&f);
            let data = assets::locate()
                .ok()
                .and_then(|l| assets::load(l.path).ok());
            let v = sector_tree_view(f.game(), f.layouts(), data.as_ref(), "en")
                .map_err(|b| anyhow::anyhow!("{} ({})", b.message, b.code))?;
            match (&v.jump, v.jump_block) {
                (Some(b), _) => println!("sector.jump  {} ({}..={})", b.value, b.min, b.max),
                (None, block) => println!("sector.jump fixed: {}", block.unwrap_or("unknown")),
            }
            for s in &v.sectors {
                println!(
                    "{:>2}  sector {}  {:<8} {:<28} next {:?}{}{}{}",
                    s.index,
                    s.level + 1,
                    s.kind.id(),
                    s.name,
                    s.next,
                    if s.visited { "  visited" } else { "" },
                    if s.current { "  [ship]" } else { "" },
                    s.jump_block.map_or(String::new(), |b| format!("  ({b})")),
                );
            }
        }
        MapCommand::List { file } => {
            let f = SaveFile::open(file_or_default(file)?)?;
            print_warnings(&f);
            let data = assets::locate()
                .ok()
                .and_then(|l| assets::load(l.path).ok());
            let Some(v) = sector_view(f.game(), data.as_ref(), "en") else {
                anyhow::bail!("the sector map is not decoded");
            };
            println!("sector {}", v.sector);
            let fleet = &v.fleet;
            match (&fleet.field, fleet.block) {
                (Some(b), _) => println!(
                    "fleet.offset  {} ({}..={}), fudge {}, radius {}",
                    b.value, b.min, b.max, fleet.fudge, fleet.radius
                ),
                (None, block) => println!(
                    "fleet offset {} fudge {} (fixed: {})",
                    fleet.offset,
                    fleet.fudge,
                    block.unwrap_or("unknown")
                ),
            }
            if let Some(reason) = v.layout_block {
                println!("positions unknown: {reason}");
            }
            for b in &v.beacons {
                let at = b
                    .point
                    .map_or_else(|| "-".to_owned(), |p| format!("{},{}", p.x, p.y));
                let visited = b.visited.as_ref().map_or(1, |f| f.value);
                println!(
                    "beacons.{:<2} {at:<8} visited {visited} seen {} fleet {:?}{}{}{}",
                    b.id,
                    b.seen.value,
                    b.fleet,
                    if b.current { "  [ship]" } else { "" },
                    b.enemy
                        .as_ref()
                        .map_or(String::new(), |e| format!("  enemy {e}")),
                    b.quest
                        .as_ref()
                        .map_or(String::new(), |q| format!("  quest {q}")),
                );
                if let Some(store) = &b.store {
                    for (n, shelf) in store.shelves.iter().enumerate() {
                        for (slot, item) in shelf.items.iter().enumerate() {
                            let Some(item) = item else { continue };
                            println!(
                                "  beacons.{}.store.{n}.{slot}  {:?} {} {}  count {}",
                                b.id,
                                shelf.kind,
                                item.id,
                                item.name.as_deref().unwrap_or(""),
                                item.stock.value
                            );
                        }
                    }
                    for (supply, field) in &store.supplies {
                        println!(
                            "  beacons.{}.store.{supply}  {}/{}",
                            b.id, field.value, field.max
                        );
                    }
                }
            }
        }
        MapCommand::Set(args) => run_set(args)?,
    }
    Ok(())
}
