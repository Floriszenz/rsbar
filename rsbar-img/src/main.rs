use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The image file(s) to scan
    image: Vec<PathBuf>,

    /// Enable display of following images to the screen
    #[arg(short, long, overrides_with = "_no_display")]
    display: bool,

    /// Disable display of following images (default)
    #[arg(short = 'D', long = "nodisplay")]
    _no_display: bool,

    /// Exit after scanning one bar code
    #[arg(short = '1', long)]
    oneshot: bool,

    /// Output points delimiting code zone with decoded symbol data
    #[arg(long)]
    polygon: bool,

    /// Minimal output, only print decoded symbol data
    #[arg(short, long)]
    quiet: bool,

    /// Output decoded symbol data without converting charsets
    #[arg(long)]
    raw: bool,

    /// Increase debug output level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8, // TODO: Maybe refactor to an enum

    /// Enable XML output format
    #[arg(long, overrides_with = "_no_xml")]
    xml: bool,

    /// Disable XML output format (default)
    #[arg(long = "noxml")]
    _no_xml: bool,
    // TODO: -S<CONFIG>[=<VALUE>], --set <CONFIG>[=<VALUE>]\n set decoder/scanner <CONFIG> to <VALUE> (or 1)
    // TODO: --nodbus (depending on feature) disable dbus message
}

fn main() {
    let args = Args::parse();

    println!("{args:?}");
}
