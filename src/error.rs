use std::io;

pub enum Error {
    Makefile { source: io::Error },
    Plugin { source: which::Error },
    Command { source: io::Error },
    Nuke,
}

impl Error {
    pub fn makefile(e: io::Error) -> Self {
        Self::Makefile { source: e }
    }

    pub fn plugin(e: which::Error) -> Self {
        Self::Plugin { source: e }
    }

    pub fn command(e: io::Error) -> Self {
        Self::Command { source: e }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Makefile { source } => source.source(),
            Error::Plugin { source } => source.source(),
            Error::Command { source } => source.source(),
            Error::Nuke => None,
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Makefile { source } => {
                write!(f, "nuke: `make.nu`: {source}")
            }
            Self::Plugin { source } => {
                write!(f, "nuke: `nu_plugin_nuke`: {source}")
            }
            Self::Command { source } => write!(f, "nuke: failed at calling `nu`: {source}"),
            Self::Nuke => Ok(()),
        }
    }
}
