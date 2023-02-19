mod errors;
mod ffi;
mod utils;

use std::time::SystemTime;

use log::LevelFilter;

pub use crate::utils::cli_args::Args;
use crate::{
    errors::{ProgramError, ProgramResult},
    utils::XmlPrinter,
};

pub fn run(args: Args) -> ProgramResult<()> {
    let start_time = SystemTime::now();

    set_global_verbosity(args.verbosity.log_level_filter());

    check_images(&args)?;

    let processor = initialize_processor(&args)?;

    let detected_symbol_count = scan_images(&args, processor)?;

    print_scan_result(
        args,
        detected_symbol_count,
        start_time
            .elapsed()
            .map_or(f32::NAN, |time| time.as_secs_f32()),
    );

    drop_processor(processor);

    if detected_symbol_count == 0 {
        return Err(ProgramError::NoSymbolDetected);
    }

    Ok(())
}

fn set_global_verbosity(verbosity: LevelFilter) {
    let level = match verbosity {
        LevelFilter::Off => 0,
        LevelFilter::Error => 0,
        LevelFilter::Warn => 0,
        LevelFilter::Info => 0,
        LevelFilter::Debug => 64,
        LevelFilter::Trace => 128,
    };

    env_logger::Builder::new().filter_level(verbosity).init();

    unsafe {
        ffi::zbar_set_verbosity(level);
    }
}

fn check_images(args: &Args) -> ProgramResult<()> {
    if args.images.is_empty() {
        return Err(ProgramError::NoImagePassed);
    }

    Ok(())
}

fn initialize_processor(args: &Args) -> ProgramResult<*mut libc::c_void> {
    let Args {
        display, nodbus, ..
    } = args;

    unsafe {
        let processor = ffi::zbar_processor_create(0);

        assert!(!processor.is_null());

        if cfg!(feature = "dbus") {
            ffi::zbar_processor_request_dbus(processor, (!nodbus).into());
        }

        if ffi::zbar_processor_init(processor, std::ptr::null(), (*display).into()) != 0 {
            ffi::_zbar_error_spew(processor, 0);
            return Err(ProgramError::ProcessorInitFailed);
        }

        apply_arguments_to_processor(processor, args)?;

        Ok(processor)
    }
}

fn parse_configs(args: &Args, processor: *mut libc::c_void) -> ProgramResult<()> {
    args.config
        .iter()
        .try_for_each(|setting| utils::parse_config(processor, setting))
}

fn apply_arguments_to_processor(processor: *mut libc::c_void, args: &Args) -> ProgramResult<()> {
    unsafe {
        ffi::zbar_processor_set_visible(processor, args.display.into());
    }

    parse_configs(args, processor)?;

    Ok(())
}

fn scan_images(args: &Args, processor: *mut libc::c_void) -> ProgramResult<u8> {
    if args.xml {
        XmlPrinter::print_head();
    }

    let detected_symbol_count = args
        .images
        .iter()
        .enumerate()
        .map(|(idx, image_path)| utils::scan_image(image_path, idx, processor, args))
        .collect::<Result<Vec<u8>, _>>()
        .map(|symbol_counts| symbol_counts.iter().sum());

    if args.xml {
        XmlPrinter::print_foot();
    }

    detected_symbol_count
}

fn drop_processor(processor: *mut libc::c_void) {
    unsafe {
        ffi::zbar_processor_destroy(processor);
    }
}

fn print_no_symbol_detected_warning(detected_symbol_count: u8) {
    if log::log_enabled!(log::Level::Warn) && detected_symbol_count == 0 {
        let mut warning_str = String::from(
            "WARNING: barcode data was not detected in some image(s)\n\
            Things to check:\n  \
            - is the barcode type supported? Currently supported symbologies are:\n",
        );

        #[cfg(feature = "ean")]
        warning_str.push_str(
            "\t- EAN/UPC (EAN-13, EAN-8, EAN-2, EAN-5, UPC-A, UPC-E, ISBN-10, ISBN-13)\n",
        );

        #[cfg(feature = "databar")]
        warning_str.push_str("\t- DataBar, DataBar Expanded\n");

        #[cfg(feature = "code128")]
        warning_str.push_str("\t- Code 128\n");

        #[cfg(feature = "code93")]
        warning_str.push_str("\t- Code 93\n");

        #[cfg(feature = "code39")]
        warning_str.push_str("\t- Code 39\n");

        #[cfg(feature = "codabar")]
        warning_str.push_str("\t- Codabar\n");

        #[cfg(feature = "i25")]
        warning_str.push_str("\t- Interleaved 2 of 5\n");

        #[cfg(feature = "qrcode")]
        warning_str.push_str("\t- QR code\n");

        #[cfg(feature = "sqcode")]
        warning_str.push_str("\t- SQ code\n");

        #[cfg(feature = "pdf417")]
        warning_str.push_str("\t- PDF 417\n");

        warning_str.push_str(
            "  - is the barcode large enough in the image?\n  \
            - is the barcode mostly in focus?\n  \
            - is there sufficient contrast/illumination?\n  \
            - If the symbol is split in several barcodes, are they combined in one image?\n  \
            - Did you enable the barcode type?\n    \
                some EAN/UPC codes are disabled by default. To enable all, use:\n    \
                $ zbarimg -S*.enable <files>\n    \
                Please also notice that some variants take precedence over others.\n    \
                Due to that, if you want, for example, ISBN-10, you should do:\n    \
                $ zbarimg -Sisbn10.enable <files>\n",
        );

        log::warn!("{warning_str}");
    }
}

fn print_scan_result(args: Args, detected_symbol_count: u8, elapsed_time: f32) {
    log::info!("scanned {detected_symbol_count} barcode symbols from {} images in {elapsed_time:.2} seconds", args.image_count());

    print_no_symbol_detected_warning(detected_symbol_count);
}
