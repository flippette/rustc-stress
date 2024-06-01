mod cli;
mod logging;
mod stress;

use std::{
    env,
    fs::{self, DirEntry},
    ops::Sub,
    process::{Command, Stdio},
    time::Instant,
};

use clap::Parser;
use cli::Args;
use eyre::{ensure, eyre, Result};
use logging::init_logging;
use tracing::{error, info};

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let cwd = fs::canonicalize(args.workdir)?;
    env::set_current_dir(&cwd)?;

    init_logging(&args.log_path)?;

    let projects = get_projects_in_cwd()?;
    let cores = if !args.cores.is_empty() {
        args.cores
    } else {
        (0..num_cpus::get_physical()).collect()
    };

    ensure!(!projects.is_empty(), "no projects found");

    info!(
        "found projects: {:?}",
        projects
            .iter()
            .map(|entry| entry.file_name())
            .collect::<Vec<_>>()
    );
    info!("stressing cores: {cores:?}");
    info!("{}", "=".repeat(20));

    for run in 0.. {
        info!("run #{run} started");
        let run_start = Instant::now();

        for core in &cores {
            info!("  stressing core {core}");
            set_thread_affinity([*core, *core + num_cpus::get_physical()])?;
            let core_start = Instant::now();

            for project in &projects {
                env::set_current_dir(project.path())?;
                info!("    cleaning {:?}", project.file_name());
                Command::new("cargo").arg("clean").output()?;

                info!("    building {:?}", project.file_name());
                let build_start = Instant::now();

                let build_output = Command::new("cargo")
                    .arg("build")
                    .env("RUSTFLAGS", "")
                    .env("PATH", env::var("PATH")?)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()?;
                if !build_output.status.success() {
                    error!("!!! building {:?} failed !!!", project.file_name());
                    error!("------ stdout ------");
                    String::from_utf8(build_output.stdout)?
                        .lines()
                        .for_each(|line| error!("{line}"));
                    error!("------ stderr ------");
                    String::from_utf8(build_output.stderr)?
                        .lines()
                        .for_each(|line| error!("{line}"));
                    return Err(eyre!(
                        "failed to build {:?}",
                        project.file_name()
                    ));
                }

                info!(
                    "    built {:?} in {:.2}s",
                    project.file_name(),
                    Instant::now().sub(build_start).as_secs_f32()
                );
                env::set_current_dir(&cwd)?;
            }

            info!(
                "  core {core} finished in {:.2}s",
                Instant::now().sub(core_start).as_secs_f32()
            );
        }

        info!(
            "run #{run} finished in {:.2}s",
            Instant::now().sub(run_start).as_secs_f32()
        );
    }

    Ok(())
}

fn get_projects_in_cwd() -> Result<Vec<DirEntry>> {
    fn is_dir(entry: &DirEntry) -> bool {
        entry.metadata().is_ok_and(|meta| meta.is_dir())
    }

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

    fs::read_dir(env::current_dir()?)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(is_dir)
                .filter(has_cargo_toml)
                .collect()
        })
        .map_err(|err| eyre!("io error: {err:?}"))
}

fn set_thread_affinity(cpus: impl AsRef<[usize]>) -> Result<()> {
    #[cfg(unix)]
    affinity::set_thread_affinity(cpus.as_ref())
        .map_err(|err| eyre!("failed to set thread affinity: {err:?}"))?;
    #[cfg(windows)]
    affinity::set_process_affinity(cpus.as_ref())
        .map_err(|err| eyre!("failed to set process affinity: {err:?}"))?;

    Ok(())
}
