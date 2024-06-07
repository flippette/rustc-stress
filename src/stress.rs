use std::{
    env,
    ffi::{OsStr, OsString},
    fs::DirEntry,
    ops::Sub,
    path::{Path, PathBuf},
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

    for project in projects.as_ref() {
        let project = Project::new(project.path())?;

        info!("{padding}cleaning {:?}", project.name());
        project.clean()?;

        info!("{padding}building {:?}", project.name());
        let build_start = Instant::now();

        let build_output = project.build(cores.as_ref())?;
        if !build_output.status.success() {
            error!("!!! building {:?} failed !!!", project.name());
            error!("------ stdout ------");
            String::from_utf8(build_output.stdout)?
                .lines()
                .for_each(|line| error!("{line}"));
            error!("------ stderr ------");
            String::from_utf8(build_output.stderr)?
                .lines()
                .for_each(|line| error!("{line}"));
            bail!("failed to build {:?}", project.name());
        }

        info!(
            "{padding}built {:?} in {:.2}s",
            project.name(),
            Instant::now().sub(build_start).as_secs_f32()
        );
    }

    Ok(())
}

/// wrapper for cargo operations
struct Project {
    name: OsString,
    path: PathBuf,
}

impl Project {
    fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().canonicalize()?;
        let name = path.file_name().unwrap().to_os_string();

        Ok(Self { name, path })
    }

    fn name(&self) -> &OsStr {
        &self.name
    }

    fn clean(&self) -> Result<Output> {
        Ok(Command::new("cargo")
            .arg("clean")
            .current_dir(&self.path)
            .output()?)
    }

    fn build(&self, cores: impl AsRef<[usize]>) -> Result<Output> {
        with_affinity(cores_to_threads(cores), || -> Result<_> {
            Ok(Command::new("cargo")
                .arg("build")
                .env("RUSTFLAGS", "")
                .env("PATH", env::var("PATH")?)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .current_dir(&self.path)
                .output()?)
        })?
    }
}

fn cores_to_threads(cores: impl AsRef<[usize]>) -> Vec<usize> {
    cores
        .as_ref()
        .iter()
        .flat_map(|&core| [core, core + num_cpus::get_physical()])
        .collect::<Vec<_>>()
}

fn with_affinity<T>(
    threads: impl AsRef<[usize]>,
    f: impl FnOnce() -> T,
) -> Result<T> {
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
        return affinity::set_thread_affinity(threads.as_ref())
            .map_err(|err| eyre!("failed to set affinity: {err:?}"));
        #[cfg(windows)]
        return affinity::set_process_affinity(threads.as_ref())
            .map_err(|err| eyre!("failed to set affinity: {err:?}"));
    }

    let cur_aff = get_affinity()?;
    set_affinity(threads.as_ref())?;
    let res = f();
    set_affinity(cur_aff)?;
    Ok(res)
}
