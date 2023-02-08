use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The image file(s) to scan
    images: Vec<PathBuf>,

    /// Set decoder/scanner <CONFIG> to <VALUE> (or 1)
    #[arg(short = 'S', long = "set", value_name = "CONFIG[=<VALUE>]")]
    config: Vec<String>,

    /// Enable display of following images to the screen
    #[arg(short, long, overrides_with = "_no_display")]
    display: bool,

    /// Disable display of following images (default)
    #[arg(short = 'D', long = "nodisplay")]
    _no_display: bool,

    /// Disable dbus message
    #[arg(long, hide = cfg!(not(feature = "dbus")))]
    nodbus: bool,

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
}

#[link(name = "zbar", kind = "static")]
extern "C" {
    fn zbar_set_verbosity(level: libc::c_int);

    fn zbar_processor_create(threaded: libc::c_int) -> *mut libc::c_void;

    fn zbar_processor_request_dbus(
        proc: *mut libc::c_void,
        req_dbus_enabled: libc::c_int,
    ) -> libc::c_int;

    fn zbar_processor_init(
        proc: *const libc::c_void,
        dev: *const libc::c_char,
        enable_display: libc::c_int,
    ) -> libc::c_int;

    fn _zbar_error_spew(container: *const libc::c_void, verbosity: libc::c_int) -> libc::c_int;

    fn zbar_processor_set_visible(proc: *mut libc::c_void, visible: libc::c_int) -> libc::c_int;

    fn zbar_processor_destroy(proc: *mut libc::c_void);
}

fn main() {
    let args = Args::parse();

    if args.images.is_empty() {
        panic!("ERROR: specify image file(s) to scan");
    }

    unsafe {
        zbar_set_verbosity(args.verbose.into());

        let processor = zbar_processor_create(0);

        assert!(!processor.is_null());

        if cfg!(feature = "dbus") {
            zbar_processor_request_dbus(processor, (!args.nodbus).into());
        }

        if zbar_processor_init(processor, std::ptr::null(), 0) != 0 {
            _zbar_error_spew(processor, 0);
            return;
        }

        zbar_processor_set_visible(processor, args.display.into());

        zbar_processor_destroy(processor);
    }
}
