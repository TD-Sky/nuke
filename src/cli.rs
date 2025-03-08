use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    /// Number of parallel jobs [default: the number of cores]
    #[arg(long, short, value_name = "N")]
    pub jobs: Option<usize>,

    /// Task to run
    pub name: Option<String>,
}

impl Cli {
    pub fn nuke_schedule(&self) -> NukeSchedule<'_> {
        NukeSchedule {
            jobs: self.jobs,
            name: self.name.as_deref(),
        }
    }
}

#[derive(Debug)]
pub struct NukeSchedule<'s> {
    jobs: Option<usize>,
    name: Option<&'s str>,
}

impl std::fmt::Display for NukeSchedule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("nuke schedule")?;
        if let Some(n) = self.jobs {
            write!(f, " --jobs={n}")?;
        }
        if let Some(name) = self.name {
            write!(f, " {name}")?;
        }
        Ok(())
    }
}
