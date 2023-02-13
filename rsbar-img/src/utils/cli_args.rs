use std::path::PathBuf;

use clap::Parser;

use crate::{
    errors::{ProgramError, ProgramResult},
    utils::{self, XmlPrinter},
};

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

    /// Minimal output, only print decoded symbol data
    #[arg(short, long)]
    pub quiet: bool,

    /// Increase debug output level
    /// (mutually exclusive with the --quiet option)
    #[arg(short, long, action = clap::ArgAction::Count, overrides_with = "quiet")]
    pub verbose: u8, // TODO: Maybe refactor to an enum

    /// Output decoded symbol data without converting charsets
    /// (mutually exclusive with the --[no]xml options)
    #[arg(long)]
    pub raw: bool,

    /// Enable XML output format
    #[arg(long, overrides_with_all = ["_no_xml", "raw"])]
    pub xml: bool, // TODO: Maybe use an optional path for output - would not require hacky output to stderr

    /// Disable XML output format (default)
    #[arg(long = "noxml")]
    _no_xml: bool,
}

impl Args {
    pub fn check_images(&self) -> ProgramResult<()> {
        if self.images.is_empty() {
            return Err(ProgramError::NoImagePassed);
        }

        Ok(())
    }

    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    pub fn parse_configs(&self, processor: *mut libc::c_void) -> ProgramResult<()> {
        self.config
            .iter()
            .try_for_each(|setting| utils::parse_config(processor, setting))
    }

    pub fn scan_images(
        &self,
        processor: *mut libc::c_void,
        xml_printer: &Option<XmlPrinter>,
    ) -> ProgramResult<u8> {
        self.images
            .iter()
            .enumerate()
            .map(|(idx, image_path)| {
                utils::scan_image(image_path, idx, processor, self, xml_printer)
            })
            .collect::<Result<Vec<u8>, _>>()
            .map(|symbol_counts| symbol_counts.iter().sum())
    }
}
