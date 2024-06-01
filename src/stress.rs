use std::{
    env,
    fs::DirEntry,
    ops::Sub,
    process::{Command, Output, Stdio},
    time::Instant,
};

use eyre::{bail, eyre, Result};
use tracing::{error, info};

pub fn stress(
    cores: impl AsRef<[usize]>,
    projects: impl AsRef<[DirEntry]>,
    log_padding: usize,
) -> Result<()> {
    let padding = " ".repeat(log_padding);
    let cwd = env::current_dir()?;

    for project in projects.as_ref() {
        let name = project.file_name();

        env::set_current_dir(project.path())?;
        info!("{padding}cleaning {name:?}");
        cargo_clean()?;

        info!("{padding}building {name:?}");
        let build_start = Instant::now();

        let build_output = cargo_build(cores.as_ref())?;
        if !build_output.status.success() {
            error!("!!! building {name:?} failed !!!");
            error!("------ stdout ------");
            String::from_utf8(build_output.stdout)?
                .lines()
                .for_each(|line| error!("{line}"));
            error!("------ stderr ------");
            String::from_utf8(build_output.stderr)?
                .lines()
                .for_each(|line| error!("{line}"));
            bail!("failed to build {name:?}");
        }

        info!(
            "{padding}built {name:?} in {:.2}s",
            Instant::now().sub(build_start).as_secs_f32()
        );

        env::set_current_dir(&cwd)?;
    }

    Ok(())
}

fn cargo_clean() -> Result<Output> {
    let output = Command::new("cargo").arg("clean").output()?;
    Ok(output)
}

fn cargo_build(cores: impl AsRef<[usize]>) -> Result<Output> {
    let cur_affinity = get_affinity()?;
    set_affinity(cores_to_threads(cores))?;
    let build_output = Command::new("cargo")
        .arg("build")
        .env("RUSTFLAGS", "")
        .env("PATH", env::var("PATH")?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    set_affinity(cur_affinity)?;
    Ok(build_output)
}

fn cores_to_threads(cores: impl AsRef<[usize]>) -> Vec<usize> {
    cores
        .as_ref()
        .iter()
        .flat_map(|&core| [core, core + num_cpus::get_physical()])
        .collect::<Vec<_>>()
}

fn get_affinity() -> Result<Vec<usize>> {
    #[cfg(unix)]
    return affinity::get_thread_affinity()
        .map_err(|err| eyre!("failed to get affinity: {err:?}"));
    #[cfg(windows)]
    return affinity::get_process_affinity()
        .map_err(|err| eyre!("failed to get affinity: {err:?}"));
}

fn set_affinity(threads: impl AsRef<[usize]>) -> Result<()> {
    #[cfg(unix)]
    affinity::set_thread_affinity(threads.as_ref())
        .map_err(|err| eyre!("failed to set affinity: {err:?}"))?;
    #[cfg(windows)]
    affinity::set_process_affinity(threads.as_ref())
        .map_err(|err| eyre!("failed to set affinity: {err:?}"))?;
    Ok(())
}
