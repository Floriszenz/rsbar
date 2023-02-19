use std::path::PathBuf;

use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The image file(s) to scan
    pub images: Vec<PathBuf>,

    /// Set decoder/scanner <CONFIG> to <VALUE> (or 1)
    #[arg(short = 'S', long = "set", value_name = "CONFIG[=<VALUE>]")]
    pub config: Vec<String>,

    /// Enable display of following images to the screen
    #[arg(short, long, overrides_with = "_no_display")]
    pub display: bool,

    /// Disable display of following images (default)
    #[arg(short = 'D', long = "nodisplay")]
    _no_display: bool,

    /// Disable dbus message
    #[arg(long, hide = cfg!(not(feature = "dbus")))]
    pub nodbus: bool,

    /// Exit after scanning one bar code
    #[arg(short = '1', long)]
    pub oneshot: bool,

    /// Output points delimiting code zone with decoded symbol data
    #[arg(long)]
    pub polygon: bool,

    /// Set debug output level
    #[command(flatten)]
    pub verbosity: Verbosity,

    /// Output decoded symbol data without converting charsets
    /// (mutually exclusive with the --[no]xml options)
    #[arg(long)]
    pub raw: bool,

    /// Enable XML output format
    #[arg(long, overrides_with_all = ["_no_xml", "raw"])]
    pub xml: bool,

    /// Disable XML output format (default)
    #[arg(long = "noxml")]
    _no_xml: bool,
}

impl Args {
    pub fn image_count(&self) -> usize {
        self.images.len()
    }
}
