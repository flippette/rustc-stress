use std::path::PathBuf;

use clap::Parser;

///
/// stress testing with rustc
///
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// physical cores to stress test
    #[arg(short, long, value_parser = core)]
    pub cores: Vec<usize>,

    /// log file to write results to
    #[arg(short, long, default_value = "stress.log")]
    pub log_path: PathBuf,

    /// current working directory
    #[arg(short, long, default_value = ".")]
    pub workdir: PathBuf,
}

/// parse a valid physical core index
fn core(s: &str) -> Result<usize, String> {
    match s.parse() {
        Ok(n) if n < num_cpus::get_physical() => Ok(n),
        Ok(n) => Err(format!(
            "zero-based core index {n} is greater than physical core count"
        )),
        Err(err) => Err(format!("failed to parse core index: {err:?}")),
    }
}
