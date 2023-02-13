mod errors;
mod ffi;
mod utils;

use std::time::SystemTime;

use clap::Parser;

use crate::{
    errors::{ProgramError, ProgramResult},
    utils::{cli_args::Args, XmlPrinter},
};

static mut EXIT_CODE: i8 = 0;

const WARNING_NOT_FOUND_HEAD: &str = "\n\
    WARNING: barcode data was not detected in some image(s)\n\
    Things to check:\n  \
        - is the barcode type supported? Currently supported symbologies are:\n";

const WARNING_NOT_FOUND_TAIL: &str = "  - is the barcode large enough in the image?\n  \
    - is the barcode mostly in focus?\n  \
    - is there sufficient contrast/illumination?\n  \
    - If the symbol is split in several barcodes, are they combined in one image?\n  \
    - Did you enable the barcode type?\n    \
        some EAN/UPC codes are disabled by default. To enable all, use:\n    \
        $ zbarimg -S*.enable <files>\n    \
        Please also notice that some variants take precedence over others.\n    \
        Due to that, if you want, for example, ISBN-10, you should do:\n    \
        $ zbarimg -Sisbn10.enable <files>\n\n";

pub fn run() -> ProgramResult<()> {
    let start_time = SystemTime::now();
    let args = Args::parse();

    args.check_images()?;

    unsafe {
        // Parse program arguments
        ffi::zbar_set_verbosity(args.verbose.into());

        let xml_printer = if args.xml {
            Some(XmlPrinter::new())
        } else {
            None
        };

        // Init processor
        let processor = ffi::zbar_processor_create(0);

        assert!(!processor.is_null());

        if cfg!(feature = "dbus") {
            ffi::zbar_processor_request_dbus(processor, (!args.nodbus).into());
        }

        if ffi::zbar_processor_init(processor, std::ptr::null(), 0) != 0 {
            ffi::_zbar_error_spew(processor, 0);
            return Err(ProgramError::ProcessorInitFailed);
        }

        // Apply other arguments to processor instance
        ffi::zbar_processor_set_visible(processor, args.display.into());

        args.parse_configs(processor)?;

        // If XML enabled, print head of XML output
        if let Some(xml_printer) = &xml_printer {
            xml_printer.print_head();
        }

        let detected_symbol_count = args.scan_images(processor, &xml_printer)?;

        /* ignore quit during last image */
        if EXIT_CODE == 3 {
            EXIT_CODE = 0;
        }

        if let Some(xml_printer) = &xml_printer {
            xml_printer.print_foot();
        }

        if !args.quiet && !args.xml {
            print!(
                "scanned {detected_symbol_count} barcode symbols from {} images",
                args.image_count()
            );
            println!(
                " in {:.2} seconds",
                start_time.elapsed().unwrap().as_secs_f32()
            );

            if detected_symbol_count == 0 {
                eprint!("{WARNING_NOT_FOUND_HEAD}");

                #[cfg(feature = "ean")]
                eprintln!(
                    "\t. EAN/UPC (EAN-13, EAN-8, EAN-2, EAN-5, UPC-A, UPC-E, ISBN-10, ISBN-13)"
                );

                #[cfg(feature = "databar")]
                eprintln!("\t. DataBar, DataBar Expanded");

                #[cfg(feature = "code128")]
                eprintln!("\t. Code 128");

                #[cfg(feature = "code93")]
                eprintln!("\t. Code 93");

                #[cfg(feature = "code39")]
                eprintln!("\t. Code 39");

                #[cfg(feature = "codabar")]
                eprintln!("\t. Codabar");

                #[cfg(feature = "i25")]
                eprintln!("\t. Interleaved 2 of 5");

                #[cfg(feature = "qrcode")]
                eprintln!("\t. QR code");

                #[cfg(feature = "sqcode")]
                eprintln!("\t. SQ code");

                #[cfg(feature = "pdf417")]
                eprintln!("\t. PDF 417");

                eprint!("{WARNING_NOT_FOUND_TAIL}");
            }
        }

        if detected_symbol_count == 0 && EXIT_CODE == 0 {
            EXIT_CODE = 4;
        }

        // Clean up
        ffi::zbar_processor_destroy(processor);
    }

    Ok(())
}
