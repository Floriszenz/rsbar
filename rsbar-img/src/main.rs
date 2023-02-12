use std::{borrow::BorrowMut, ffi::CStr, path::PathBuf, time::SystemTime};

use clap::Parser;

use rsbar_img::{
    errors::{ProgramError, ProgramResult},
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

#[link(name = "zbar", kind = "static")]
extern "C" {
    static stdout: *mut libc::FILE;

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

    fn zbar_parse_config(
        cfgstr: *const libc::c_char,
        sym: *mut ZbarSymbolType,
        cfg: *mut ZbarConfig,
        val: *mut libc::c_int,
    ) -> libc::c_int;

    fn zbar_processor_set_config(
        proc: *mut libc::c_void,
        sym: ZbarSymbolType,
        cfg: ZbarConfig,
        val: libc::c_int,
    ) -> libc::c_int;

    fn zbar_image_create() -> *mut libc::c_void;

    fn zbar_image_set_format(img: *mut libc::c_void, fmt: libc::c_ulong);

    fn zbar_image_set_size(img: *mut libc::c_void, w: libc::c_uint, h: libc::c_uint);

    fn zbar_image_set_data(
        img: *mut libc::c_void,
        data: *mut libc::c_void,
        len: libc::c_ulong,
        cleanup: unsafe extern "C" fn(*mut libc::c_void),
    );

    fn zbar_image_free_data(img: *mut libc::c_void);

    fn zbar_process_image(proc: *mut libc::c_void, img: *mut libc::c_void) -> libc::c_int;

    fn zbar_image_first_symbol(img: *const libc::c_void) -> *const libc::c_void;

    fn zbar_symbol_next(sym: *const libc::c_void) -> *const libc::c_void;

    fn zbar_symbol_get_type(sym: *const libc::c_void) -> ZbarSymbolType;

    fn zbar_symbol_get_data_length(sym: *const libc::c_void) -> libc::size_t;

    fn zbar_get_symbol_name(sym: ZbarSymbolType) -> *const libc::c_char;

    fn zbar_symbol_get_loc_size(sym: *const libc::c_void) -> libc::c_uint;

    fn zbar_symbol_get_loc_x(sym: *const libc::c_void, idx: libc::c_uint) -> libc::c_int;

    fn zbar_symbol_get_loc_y(sym: *const libc::c_void, idx: libc::c_uint) -> libc::c_int;

    fn zbar_symbol_get_data(sym: *const libc::c_void) -> *const libc::c_char;

    fn zbar_symbol_xml(
        sym: *const libc::c_void,
        buf: *mut *mut libc::c_char,
        len: *mut libc::c_uint,
    ) -> *const libc::c_char;

    fn zbar_image_destroy(img: *mut libc::c_void);

    fn zbar_processor_is_visible(proc: *mut libc::c_void) -> libc::c_int;

    fn zbar_processor_user_wait(proc: *mut libc::c_void, timeout: libc::c_int) -> libc::c_int;

    fn zbar_processor_destroy(proc: *mut libc::c_void);
}

#[repr(C)]
#[derive(PartialEq)]
enum ZbarSymbolType {
    /**< no symbol decoded */
    ZbarNone = 0,
    /**< intermediate status */
    ZbarPartial = 1,
    /**< GS1 2-digit add-on */
    ZbarEan2 = 2,
    /**< GS1 5-digit add-on */
    ZbarEan5 = 5,
    /**< EAN-8 */
    ZbarEan8 = 8,
    /**< UPC-E */
    ZbarUpce = 9,
    /**< ISBN-10 (from EAN-13). @since 0.4 */
    ZbarIsbn10 = 10,
    /**< UPC-A */
    ZbarUpca = 12,
    /**< EAN-13 */
    ZbarEan13 = 13,
    /**< ISBN-13 (from EAN-13). @since 0.4 */
    ZbarIsbn13 = 14,
    /**< EAN/UPC composite */
    ZbarComposite = 15,
    /**< Interleaved 2 of 5. @since 0.4 */
    ZbarI25 = 25,
    /**< GS1 DataBar (RSS). @since 0.11 */
    ZbarDatabar = 34,
    /**< GS1 DataBar Expanded. @since 0.11 */
    ZbarDatabarExp = 35,
    /**< Codabar. @since 0.11 */
    ZbarCodabar = 38,
    /**< Code 39. @since 0.4 */
    ZbarCode39 = 39,
    /**< PDF417. @since 0.6 */
    ZbarPdf417 = 57,
    /**< QR Code. @since 0.10 */
    ZbarQrcode = 64,
    /**< SQ Code. @since 0.20.1 */
    ZbarSqcode = 80,
    /**< Code 93. @since 0.11 */
    ZbarCode93 = 93,
    /**< Code 128 */
    ZbarCode128 = 128,

    /*
     * Please see _zbar_get_symbol_hash() if adding
     * anything after 128
     */
    /** mask for base symbol type.
     * @deprecated in 0.11, remove this from existing code
     */
    ZbarSymbol = 0x00ff,
    /** 2-digit add-on flag.
     * @deprecated in 0.11, a ::ZBAR_EAN2 component is used for
     * 2-digit GS1 add-ons
     */
    ZbarAddon2 = 0x0200,
    /** 5-digit add-on flag.
     * @deprecated in 0.11, a ::ZBAR_EAN5 component is used for
     * 5-digit GS1 add-ons
     */
    ZbarAddon5 = 0x0500,
    /** add-on flag mask.
     * @deprecated in 0.11, GS1 add-ons are represented using composite
     * symbols of type ::ZBAR_COMPOSITE; add-on components use ::ZBAR_EAN2
     * or ::ZBAR_EAN5
     */
    ZbarAddon = 0x0700,
}

#[repr(C)]
enum ZbarConfig {
    /**< enable symbology/feature */
    Enable = 0,
    /**< enable check digit when optional */
    AddCheck,
    /**< return check digit when present */
    EmitCheck,
    /**< enable full ASCII character set */
    Ascii,
    /**< don't convert binary data to text */
    Binary,
    /**< number of boolean decoder configs */
    Num,
    /**< minimum data length for valid decode */
    MinLen = 0x20,
    /**< maximum data length for valid decode */
    MaxLen,
    /**< required video consistency frames */
    Uncertainty = 0x40,
    /**< enable scanner to collect position data */
    Position = 0x80,
    /**< if fails to decode, test inverted */
    TestInverted,

    /**< image scanner vertical scan density */
    XDensity = 0x100,
    /**< image scanner horizontal scan density */
    YDensity,
}

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

    let parse_res = zbar_parse_config(
        config_string,
        sym.borrow_mut(),
        cfg.borrow_mut(),
        val.borrow_mut(),
    );

    if parse_res != 0 {
        return parse_res;
    }

    zbar_processor_set_config(processor, sym, cfg, val)
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

    let zimage = zbar_image_create();
    assert!(!zimage.is_null());
    zbar_image_set_format(
        zimage,
        zbar_fourcc('Y' as i8, '8' as i8, '0' as i8, '0' as i8),
    );

    let width = image.width();
    let height = image.height();
    zbar_image_set_size(zimage, width, height);

    // extract grayscale image pixels
    // FIXME color!! ...preserve most color w/422P
    // (but only if it's a color image)
    let bloblen = (width * height) as usize;
    let blob = libc::malloc(bloblen);
    zbar_image_set_data(zimage, blob, bloblen as u64, zbar_image_free_data);

    let bytes = image.into_luma8().into_raw();

    libc::memcpy(blob, bytes.as_ptr().cast(), bloblen);

    if XMLLVL == 1 {
        XMLLVL += 1;
        println!("<source href='{}'>", filename.display());
    }

    zbar_process_image(processor, zimage);

    // output result data
    let mut sym = zbar_image_first_symbol(zimage);

    while !sym.is_null() {
        let typ = zbar_symbol_get_type(sym);
        let len = zbar_symbol_get_data_length(sym);

        if typ == ZbarSymbolType::ZbarPartial {
            continue;
        } else if XMLLVL <= 0 {
            if XMLLVL == 0 {
                print!(
                    "{}:",
                    CStr::from_ptr(zbar_get_symbol_name(typ)).to_str().unwrap()
                );
            }

            if POLYGON == 1 {
                if zbar_symbol_get_loc_size(sym) > 0 {
                    print!(
                        "{},{}",
                        zbar_symbol_get_loc_x(sym, 0),
                        zbar_symbol_get_loc_y(sym, 0)
                    );
                }

                for p in 1..zbar_symbol_get_loc_size(sym) {
                    print!(
                        " {},{}",
                        zbar_symbol_get_loc_x(sym, p),
                        zbar_symbol_get_loc_y(sym, p)
                    );
                }

                print!(":");
            }

            if len > 0 && libc::fwrite(zbar_symbol_get_data(sym).cast(), len, 1, stdout) != 1 {
                EXIT_CODE = 1;
                return -1;
            }
        } else {
            if XMLLVL < 3 {
                XMLLVL += 1;
                println!("<index num='{SEQ}'>");
            }

            zbar_symbol_xml(sym, &mut XML_BUF, &mut XML_BUF_LEN);

            if libc::fwrite(XML_BUF.cast(), XML_BUF_LEN as libc::size_t, 1, stdout) != 1 {
                EXIT_CODE = 1;
                return -1;
            }
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

        sym = zbar_symbol_next(sym);
    }

    if XMLLVL > 2 {
        XMLLVL -= 1;
        println!("</index>");
    }

    libc::fflush(stdout);

    zbar_image_destroy(zimage);

    NUM_IMAGES += 1;

    if zbar_processor_is_visible(processor) == 1 {
        let rc = zbar_processor_user_wait(processor, -1);

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

fn main() -> ProgramResult<()> {
    let start_time = SystemTime::now();
    let args = Args::parse();

    args.check_images()?;

    unsafe {
        // Parse program arguments
        ONESHOT = args.oneshot.into();
        POLYGON = args.polygon.into();

        zbar_set_verbosity(args.verbose.into());

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
        let processor = zbar_processor_create(0);

        assert!(!processor.is_null());

        if cfg!(feature = "dbus") {
            zbar_processor_request_dbus(processor, (!args.nodbus).into());
        }

        if zbar_processor_init(processor, std::ptr::null(), 0) != 0 {
            _zbar_error_spew(processor, 0);
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
        zbar_processor_set_visible(processor, args.display.into());

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
            libc::fflush(stdout);
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
        zbar_processor_destroy(processor);
    }

    Ok(())
}
