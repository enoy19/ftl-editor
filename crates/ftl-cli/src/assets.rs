//! `ftled assets ...` and `ftled ships`: the game's own data from `ftl.dat`.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};
use ftl_app::assets::{self, AssetsConfig, Source};

#[derive(Subcommand)]
pub enum AssetsCommand {
    /// Print the path of the game's ftl.dat.
    ///
    /// Tries the path remembered in the editor's config file first, then the
    /// Steam, GOG and Humble folders of this OS (only macOS Steam is tested).
    /// stdout carries the path only; stderr says where it came from.
    Locate {
        /// Use this file instead of searching.
        #[arg(long)]
        dat: Option<PathBuf>,
        /// Store the path in the editor's config file, so the next search starts there.
        #[arg(long)]
        remember: bool,
    },
    /// List the inner paths of ftl.dat, one per line, in package order.
    Ls {
        /// Only paths that start with this prefix, for example `fonts/` or `img/customizeUI/`.
        prefix: Option<String>,
        /// Path of ftl.dat [default: `ftled assets locate`].
        #[arg(long)]
        dat: Option<PathBuf>,
    },
    /// Write the bytes of one entry of ftl.dat to a file (never into the repository).
    Cat {
        /// Inner path, for example `fonts/JustinFont8.font`.
        inner_path: String,
        /// File to write.
        #[arg(short, long)]
        out: PathBuf,
        /// Path of ftl.dat [default: `ftled assets locate`].
        #[arg(long)]
        dat: Option<PathBuf>,
    },
}

fn dat_or_located(dat: Option<PathBuf>) -> Result<PathBuf> {
    Ok(match dat {
        Some(d) => d,
        None => assets::locate()?.path,
    })
}

#[derive(Args)]
pub struct ShipsArgs {
    /// Path of ftl.dat [default: `ftled assets locate`].
    #[arg(long)]
    dat: Option<PathBuf>,
}

pub fn run_assets(command: AssetsCommand) -> Result<()> {
    match command {
        AssetsCommand::Locate { dat, remember } => {
            let config = AssetsConfig::open_default()?;
            let (path, how) = match dat {
                Some(d) => (d, "given with --dat"),
                None => {
                    let located = config.locate()?;
                    let how = match located.source {
                        Source::Remembered => "remembered",
                        Source::Found => "found, not remembered yet",
                    };
                    (located.path, how)
                }
            };
            if remember {
                config.remember(&path)?;
            }
            println!("{}", path.display());
            match remember {
                true => eprintln!("remembered in {}", config.file().display()),
                false => eprintln!("{how}"),
            }
        }
        AssetsCommand::Ls { prefix, dat } => {
            let prefix = prefix.unwrap_or_default();
            for p in assets::entry_paths(dat_or_located(dat)?)? {
                if p.starts_with(&prefix) {
                    println!("{p}");
                }
            }
        }
        AssetsCommand::Cat {
            inner_path,
            out,
            dat,
        } => {
            let bytes = assets::read_entry(dat_or_located(dat)?, &inner_path)?;
            std::fs::write(&out, &bytes)?;
            eprintln!("{} bytes to {}", bytes.len(), out.display());
        }
    }
    Ok(())
}

/// One line per player ship in hangar order: blueprint id, name, class.
pub fn run_ships(args: ShipsArgs) -> Result<()> {
    let dat = dat_or_located(args.dat)?;
    let data = assets::load(&dat)?;
    for ship in data.player_ships() {
        println!("{:<24}{:<24}{}", ship.id, ship.name, ship.class_name);
    }
    Ok(())
}
