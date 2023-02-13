mod errors;
mod ffi;
mod utils;

use std::{ffi::CStr, path::PathBuf, time::SystemTime};

use clap::Parser;

use crate::{
    errors::{ProgramError, ProgramResult},
    ffi::ZbarSymbolType,
    utils::cli_args::Args,
};

static mut NOT_FOUND: bool = false;
static mut EXIT_CODE: i8 = 0;
static mut NUM_SYMBOLS: i8 = 0;
static mut XMLLVL: i8 = 0;
static mut USE_BINARY_OUTPUT: bool = false; // TODO: Replace this with a `processor.get_config("binary")` method at a later point of refactoring

const XML_HEAD: &str = "<barcodes xmlns='http://zbar.sourceforge.net/2008/barcode'>\n";
const XML_FOOT: &str = "</barcodes>\n";

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

const fn zbar_fourcc(a: u8, b: u8, c: u8, d: u8) -> u64 {
    a as u64 | ((b as u64) << 8) | ((c as u64) << 16) | ((d as u64) << 24)
}

unsafe fn scan_image(
    filename: &PathBuf,
    idx: usize,
    processor: *mut libc::c_void,
    args: &Args,
) -> libc::c_int {
    if EXIT_CODE == 3 {
        return -1;
    }

    let mut found: libc::c_int = 0;
    let image = image::open(filename).unwrap();

    let zimage = ffi::zbar_image_create();
    assert!(!zimage.is_null());
    ffi::zbar_image_set_format(zimage, zbar_fourcc(b'Y', b'8', b'0', b'0'));

    let width = image.width();
    let height = image.height();
    ffi::zbar_image_set_size(zimage, width, height);

    // extract grayscale image pixels
    // FIXME color!! ...preserve most color w/422P
    // (but only if it's a color image)
    let bloblen = (width * height) as usize;
    let blob = libc::malloc(bloblen);
    ffi::zbar_image_set_data(zimage, blob, bloblen as u64, ffi::zbar_image_free_data);

    let bytes = image.into_luma8().into_raw();

    libc::memcpy(blob, bytes.as_ptr().cast(), bloblen);

    if XMLLVL == 1 {
        XMLLVL += 1;
        println!("<source href='{}'>", filename.display());
    }

    ffi::zbar_process_image(processor, zimage);

    // output result data
    let mut sym = ffi::zbar_image_first_symbol(zimage);

    while !sym.is_null() {
        let typ = ffi::zbar_symbol_get_type(sym);

        if typ == ZbarSymbolType::ZbarPartial {
            continue;
        } else if XMLLVL <= 0 {
            if XMLLVL == 0 {
                print!(
                    "{}:",
                    CStr::from_ptr(ffi::zbar_get_symbol_name(typ))
                        .to_str()
                        .unwrap()
                );
            }

            if args.polygon {
                if ffi::zbar_symbol_get_loc_size(sym) > 0 {
                    print!(
                        "{},{}",
                        ffi::zbar_symbol_get_loc_x(sym, 0),
                        ffi::zbar_symbol_get_loc_y(sym, 0)
                    );
                }

                for p in 1..ffi::zbar_symbol_get_loc_size(sym) {
                    print!(
                        " {},{}",
                        ffi::zbar_symbol_get_loc_x(sym, p),
                        ffi::zbar_symbol_get_loc_y(sym, p)
                    );
                }

                print!(":");
            }

            print!(
                "{}",
                CStr::from_ptr(ffi::zbar_symbol_get_data(sym))
                    .to_str()
                    .unwrap()
            );
        } else {
            if XMLLVL < 3 {
                XMLLVL += 1;
                println!("<index num='{idx}'>");
            }

            let symbol_xml = ffi::zbar_symbol_xml(sym, &mut std::ptr::null_mut(), &mut 0);

            print!("{}", CStr::from_ptr(symbol_xml).to_str().unwrap());
        }

        found += 1;
        NUM_SYMBOLS += 1;

        if !USE_BINARY_OUTPUT {
            if args.oneshot {
                if XMLLVL >= 0 {
                    println!();
                }

                break;
            } else {
                println!();
            }
        }

        sym = ffi::zbar_symbol_next(sym);
    }

    if XMLLVL > 2 {
        XMLLVL -= 1;
        println!("</index>");
    }

    ffi::zbar_image_destroy(zimage);

    if ffi::zbar_processor_is_visible(processor) == 1 {
        let rc = ffi::zbar_processor_user_wait(processor, -1);

        if rc < 0 || rc == b'q'.into() || rc == b'Q'.into() {
            EXIT_CODE = 3
        }
    }

    if XMLLVL > 1 {
        XMLLVL -= 1;
        println!("</source>");
    }

    if found == 0 {
        NOT_FOUND = true;
    }

    0
}

pub fn run() -> ProgramResult<()> {
    let start_time = SystemTime::now();
    let args = Args::parse();

    args.check_images()?;

    unsafe {
        // Parse program arguments
        ffi::zbar_set_verbosity(args.verbose.into());

        if args.xml && XMLLVL >= 0 {
            XMLLVL = 1;
        } else if !args.xml && XMLLVL > 0 {
            XMLLVL = 0;
        }

        if args.raw {
            // RAW mode takes precedence
            XMLLVL = -1;
        }

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

        args.parse_config(processor)?;

        // If XML enabled, print head of XML output
        if XMLLVL > 0 {
            print!("{XML_HEAD}");
        }

        if USE_BINARY_OUTPUT {
            XMLLVL = -1;
        }

        // TODO:
        // #ifdef _WIN32
        //     if (xmllvl == -1) {
        //         _setmode(_fileno(stdout), _O_BINARY);
        //     } else {
        //         _setmode(_fileno(stdout), _O_TEXT);
        //     }
        // #endif

        for (idx, image_path) in args.images.iter().enumerate() {
            if scan_image(image_path, idx, processor, &args) != 0 {
                return Err(ProgramError::ImageScanFailed(
                    image_path.display().to_string(),
                ));
            }
        }

        /* ignore quit during last image */
        if EXIT_CODE == 3 {
            EXIT_CODE = 0;
        }

        if XMLLVL > 0 {
            print!("{XML_FOOT}");
        }

        if !args.quiet && XMLLVL <= 0 {
            eprint!(
                "scanned {NUM_SYMBOLS} barcode symbols from {} images",
                args.image_count()
            );
            eprintln!(
                " in {:.2} seconds",
                start_time.elapsed().unwrap().as_secs_f32()
            );

            if NOT_FOUND {
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

        if NOT_FOUND && EXIT_CODE == 0 {
            EXIT_CODE = 4;
        }

        // Clean up
        ffi::zbar_processor_destroy(processor);
    }

    Ok(())
}
