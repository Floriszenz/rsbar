mod errors;
mod ffi;
mod utils;

use std::{borrow::BorrowMut, ffi::CStr, path::PathBuf, time::SystemTime};

use clap::Parser;

use crate::{
    errors::{ProgramError, ProgramResult},
    ffi::{ZbarConfig, ZbarSymbolType},
    utils::cli_args::Args,
};

static mut NOT_FOUND: i8 = 0;
static mut EXIT_CODE: i8 = 0;
static mut NUM_IMAGES: i8 = 0;
static mut NUM_SYMBOLS: i8 = 0;
static mut XMLLVL: i8 = 0;
static mut POLYGON: i8 = 0;
static mut ONESHOT: i8 = 0;
static mut BINARY: i8 = 0;
static mut SEQ: i8 = 0;

static mut XML_BUF: *mut libc::c_char = std::ptr::null_mut();
static mut XML_BUF_LEN: libc::c_uint = 0;

const XML_HEAD: &str = "<barcodes xmlns='http://zbar.sourceforge.net/2008/barcode'>\n";
const XML_FOOT: &str = "</barcodes>\n";

const WARNING_NOT_FOUND_HEAD: &str = "\n\
    WARNING: barcode data was not detected in some image(s)\n\
    Things to check:\n\
    \t- is the barcode type supported? Currently supported symbologies are:\n";

const WARNING_NOT_FOUND_TAIL: &str = "\t- is the barcode large enough in the image?\n\
    \t- is the barcode mostly in focus?\n\
    \t- is there sufficient contrast/illumination?\n\
    \t- If the symbol is split in several barcodes, are they combined in one image?\n\
    \t- Did you enable the barcode type?\n\
    \t\tsome EAN/UPC codes are disabled by default. To enable all, use:\n\
    \t\t$ zbarimg -S*.enable <files>\n\
    \t\tPlease also notice that some variants take precedence over others.\n\
    \t\tDue to that, if you want, for example, ISBN-10, you should do:\n\
    \t\t$ zbarimg -Sisbn10.enable <files>\n\
    \n";

const fn zbar_fourcc(
    a: libc::c_char,
    b: libc::c_char,
    c: libc::c_char,
    d: libc::c_char,
) -> libc::c_ulong {
    a as libc::c_ulong
        | ((b as libc::c_ulong) << 8)
        | ((c as libc::c_ulong) << 16)
        | ((d as libc::c_ulong) << 24)
}

unsafe fn zbar_processor_parse_config(
    processor: *mut libc::c_void,
    config_string: *const libc::c_char,
) -> libc::c_int {
    let mut sym: ZbarSymbolType = ZbarSymbolType::ZbarNone;
    let mut cfg: ZbarConfig = ZbarConfig::Enable;
    let mut val: libc::c_int = 0;

    let parse_res = ffi::zbar_parse_config(
        config_string,
        sym.borrow_mut(),
        cfg.borrow_mut(),
        val.borrow_mut(),
    );

    if parse_res != 0 {
        return parse_res;
    }

    ffi::zbar_processor_set_config(processor, sym, cfg, val)
}

unsafe fn parse_config(processor: *mut libc::c_void, config_string: *const libc::c_char) -> i8 {
    if zbar_processor_parse_config(processor, config_string) != 0 {
        return 1;
    }

    if CStr::from_ptr(config_string) == CStr::from_ptr(b"binary\0".as_ptr().cast()) {
        BINARY = 1;
    }

    0
}

unsafe fn scan_image(filename: &PathBuf, processor: *mut libc::c_void) -> libc::c_int {
    if EXIT_CODE == 3 {
        return -1;
    }

    let mut found: libc::c_int = 0;
    let image = image::open(filename).unwrap();

    let zimage = ffi::zbar_image_create();
    assert!(!zimage.is_null());
    ffi::zbar_image_set_format(
        zimage,
        zbar_fourcc('Y' as i8, '8' as i8, '0' as i8, '0' as i8),
    );

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

            if POLYGON == 1 {
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
                println!("<index num='{SEQ}'>");
            }

            ffi::zbar_symbol_xml(sym, &mut XML_BUF, &mut XML_BUF_LEN);

            print!("{}", CStr::from_ptr(XML_BUF).to_str().unwrap());
        }

        found += 1;
        NUM_SYMBOLS += 1;

        if BINARY == 0 {
            if ONESHOT == 1 {
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

    NUM_IMAGES += 1;

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
        NOT_FOUND += 1;
    }

    0
}

pub fn run() -> ProgramResult<()> {
    let start_time = SystemTime::now();
    let args = Args::parse();

    args.check_images()?;

    unsafe {
        // Parse program arguments
        ONESHOT = args.oneshot.into();
        POLYGON = args.polygon.into();

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

        // If XML enabled, print head of XML output
        if XMLLVL > 0 {
            print!("{XML_HEAD}");
        }

        if BINARY == 1 {
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

        // Apply other arguments to processor instance
        ffi::zbar_processor_set_visible(processor, args.display.into());

        for setting in args.config {
            if parse_config(processor, setting.as_ptr().cast()) != 0 {
                return Err(ProgramError::InvalidConfig(setting));
            }
        }

        SEQ = 0;

        for image_path in args.images {
            if scan_image(&image_path, processor) != 0 {
                return Err(ProgramError::ImageScanFailed(
                    image_path.display().to_string(),
                ));
            }

            SEQ += 1;
        }

        /* ignore quit during last image */
        if EXIT_CODE == 3 {
            EXIT_CODE = 0;
        }

        if XMLLVL > 0 {
            print!("{XML_FOOT}");
        }

        if !XML_BUF.is_null() {
            libc::free(XML_BUF.cast());
        }

        if NUM_IMAGES > 0 && !args.quiet && XMLLVL <= 0 {
            eprint!("scanned {NUM_SYMBOLS} barcode symbols from {NUM_IMAGES} images");
            eprintln!(
                " in {:.2} seconds",
                start_time.elapsed().unwrap().as_secs_f32()
            );

            if NOT_FOUND == 1 {
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

        if NUM_IMAGES > 0 && NOT_FOUND == 1 && EXIT_CODE == 0 {
            EXIT_CODE = 4;
        }

        // Clean up
        ffi::zbar_processor_destroy(processor);
    }

    Ok(())
}
