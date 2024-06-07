mod cli;
mod logging;
mod stress;

use std::{
    fs::{self, DirEntry},
    ops::Sub,
    path::Path,
    time::Instant,
};

use clap::Parser;
use cli::{Args, Mode};
use eyre::{ensure, eyre, Result};
use stress::stress;
use tracing::info;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    logging::init(&args.log_path)?;

    let projects = projects(args.projects)?;
    let cores = if args.cores.is_empty() {
        (0..num_cpus::get_physical()).collect()
    } else {
        args.cores
    };

    ensure!(!projects.is_empty(), "no projects found");

    info!(
        "found projects: {:?}",
        projects.iter().map(DirEntry::file_name).collect::<Vec<_>>()
    );
    info!("stressing cores {cores:?} in {} mode", args.mode);
    info!("{}", "=".repeat(20));

    for run in 0.. {
        info!("run #{run} started");
        let run_start = Instant::now();

        match args.mode {
            Mode::Sequential => {
                for core in &cores {
                    info!("  stressing core {core}");
                    let core_start = Instant::now();

                    stress([*core], &projects, 4)?;

                    info!(
                        "  core {core} took {:.2}s",
                        Instant::now().sub(core_start).as_secs_f32()
                    );
                }
            }
            Mode::Parallel => {
                stress(&cores, &projects, 2)?;
            }
        }

        info!(
            "run #{run} finished in {:.2}s",
            Instant::now().sub(run_start).as_secs_f32()
        );
    }

    Ok(())
}

fn projects(path: impl AsRef<Path>) -> Result<Vec<DirEntry>> {
    fn has_cargo_toml(entry: &DirEntry) -> bool {
        fs::read_dir(entry.path()).is_ok_and(|entries| {
            entries.filter_map(Result::ok).any(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name == "Cargo.toml")
            })
        })
    }

    fs::read_dir(path.as_ref())
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(has_cargo_toml)
                .collect()
        })
        .map_err(|err| eyre!("io error: {err:?}"))
}
