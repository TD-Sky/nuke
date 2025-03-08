use std::path::{Path, PathBuf};
use std::time::SystemTime;

use nu_protocol::{Spanned, engine::Closure};

use crate::utils::path::PathExt;

#[derive(Debug)]
pub struct Task {
    pub(crate) name: Spanned<String>,
    pub(crate) deps: Vec<Spanned<String>>,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) target: Option<PathBuf>,
    pub(crate) run: Option<Spanned<Closure>>,
}

impl Task {
    pub fn name(&self) -> &str {
        &self.name.item
    }

    pub fn name_span(&self) -> Spanned<&str> {
        self.name.as_deref()
    }

    pub fn cached_at(&self) -> Option<SystemTime> {
        let out_mtime = self.target.as_ref()?.timestamp()?;

        for dep in &self.files {
            let dep_mtime = Path::new(dep).timestamp()?;
            if dep_mtime > out_mtime {
                return None;
            }
        }
        Some(out_mtime)
    }

    pub fn run(&self) -> Option<&Spanned<Closure>> {
        self.run.as_ref()
    }

    pub fn deps(&self) -> &[Spanned<String>] {
        &self.deps
    }
}
