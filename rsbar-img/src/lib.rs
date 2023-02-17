mod errors;
mod ffi;
mod utils;

use std::time::SystemTime;

use clap::Parser;

use crate::{
    errors::{ProgramError, ProgramResult},
    utils::{cli_args::Args, LogVerbosity, XmlPrinter},
};

pub fn run() -> ProgramResult<()> {
    let start_time = SystemTime::now();
    let args = Args::parse();

    set_global_verbosity(args.verbosity);

    check_images(&args)?;

    let processor = initialize_processor(&args)?;

    if args.xml {
        XmlPrinter::print_head();
    }

    let detected_symbol_count = scan_images(&args, processor)?;

    if args.xml {
        XmlPrinter::print_foot();
    }

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

fn set_global_verbosity(verbosity: LogVerbosity) {
    unsafe {
        ffi::zbar_set_verbosity(verbosity);
    }
}

fn check_images(args: &Args) -> ProgramResult<()> {
    if args.images.is_empty() {
        return Err(ProgramError::NoImagePassed);
    }

    Ok(())
}

fn initialize_processor(args: &Args) -> ProgramResult<*mut libc::c_void> {
    let Args { nodbus, .. } = args;

    unsafe {
        let processor = ffi::zbar_processor_create(0);

        assert!(!processor.is_null());

        if cfg!(feature = "dbus") {
            ffi::zbar_processor_request_dbus(processor, (!nodbus).into());
        }

        if ffi::zbar_processor_init(processor, std::ptr::null(), 0) != 0 {
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
    args.images
        .iter()
        .enumerate()
        .map(|(idx, image_path)| utils::scan_image(image_path, idx, processor, args))
        .collect::<Result<Vec<u8>, _>>()
        .map(|symbol_counts| symbol_counts.iter().sum())
}

fn drop_processor(processor: *mut libc::c_void) {
    unsafe {
        ffi::zbar_processor_destroy(processor);
    }
}

fn print_no_symbol_detected_warning(detected_symbol_count: u8) {
    if detected_symbol_count == 0 {
        eprintln!(
            "\n\
            WARNING: barcode data was not detected in some image(s)\n\
            Things to check:\n  \
                - is the barcode type supported? Currently supported symbologies are:"
        );

        #[cfg(feature = "ean")]
        eprintln!("\t- EAN/UPC (EAN-13, EAN-8, EAN-2, EAN-5, UPC-A, UPC-E, ISBN-10, ISBN-13)");

        #[cfg(feature = "databar")]
        eprintln!("\t- DataBar, DataBar Expanded");

        #[cfg(feature = "code128")]
        eprintln!("\t- Code 128");

        #[cfg(feature = "code93")]
        eprintln!("\t- Code 93");

        #[cfg(feature = "code39")]
        eprintln!("\t- Code 39");

        #[cfg(feature = "codabar")]
        eprintln!("\t- Codabar");

        #[cfg(feature = "i25")]
        eprintln!("\t- Interleaved 2 of 5");

        #[cfg(feature = "qrcode")]
        eprintln!("\t- QR code");

        #[cfg(feature = "sqcode")]
        eprintln!("\t- SQ code");

        #[cfg(feature = "pdf417")]
        eprintln!("\t- PDF 417");

        eprintln!(
            "  - is the barcode large enough in the image?\n  \
            - is the barcode mostly in focus?\n  \
            - is there sufficient contrast/illumination?\n  \
            - If the symbol is split in several barcodes, are they combined in one image?\n  \
            - Did you enable the barcode type?\n    \
                some EAN/UPC codes are disabled by default. To enable all, use:\n    \
                $ zbarimg -S*.enable <files>\n    \
                Please also notice that some variants take precedence over others.\n    \
                Due to that, if you want, for example, ISBN-10, you should do:\n    \
                $ zbarimg -Sisbn10.enable <files>\n"
        );
    }
}

fn print_scan_result(args: Args, detected_symbol_count: u8, elapsed_time: f32) {
    if !args.verbosity.is_quiet() && !args.xml {
        println!(
            "scanned {detected_symbol_count} barcode symbols from {} images in {elapsed_time:.2} seconds",
            args.image_count()
        );

        print_no_symbol_detected_warning(detected_symbol_count);
    }
}
