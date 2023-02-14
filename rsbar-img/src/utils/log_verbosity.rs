use clap::ValueEnum;

// TODO: remove repr(C) & number values as soon as there are not needed anymore
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogVerbosity {
    /// The most minimal output
    Quiet = 0,
    /// Relatively little output
    Minimal = 1,
    /// Standard output. This is the default if verbosity level is not set
    Normal = 2,
    /// Relatively verbose, but not exhaustive
    Detailed = 4,
    /// The most verbose and informative verbosity
    Diagnostic = 8,
}

impl LogVerbosity {
    pub fn is_quiet(&self) -> bool {
        self.eq(&LogVerbosity::Quiet)
    }
}
