use std::{
    fmt::{self, Display},
    path::PathBuf,
};

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

    /// stress testing mode
    #[arg(short, long, default_value = "seq", value_parser = mode)]
    pub mode: Mode,

    /// current working directory
    #[arg(short, long, default_value = ".")]
    pub workdir: PathBuf,
}

/// stress testing mode
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Sequential,
    Parallel,
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Sequential => "sequential",
                Self::Parallel => "parallel",
            }
        )
    }
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

fn mode(s: &str) -> Result<Mode, String> {
    match s.to_lowercase().as_ref() {
        "seq" | "sequential" => Ok(Mode::Sequential),
        "par" | "parallel" => Ok(Mode::Parallel),
        other => Err(format!("failed to parse mode: \"{other}\"")),
    }
}
