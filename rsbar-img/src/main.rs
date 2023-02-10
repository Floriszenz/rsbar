#![allow(dead_code)]
use std::{borrow::BorrowMut, ffi::CStr, path::PathBuf};

use clap::Parser;

static mut NOT_FOUND: i8 = 0;
static mut EXIT_CODE: i8 = 0;
static mut NUM_IMAGES: i8 = 0;
static mut NUM_SYMBOLS: i8 = 0;
static mut XMLLVL: i8 = 0;
static mut POLYGON: i8 = 0;
static mut ONESHOT: i8 = 0;
static mut BINARY: i8 = 0;

const XML_HEAD: &str = "<barcodes xmlns='http://zbar.sourceforge.net/2008/barcode'>\n";
const XML_FOOT: &str = "</barcodes>\n";

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

    fn zbar_processor_destroy(proc: *mut libc::c_void);
}

#[repr(C)]
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

fn main() {
    let args = Args::parse();

    if args.images.is_empty() {
        panic!("ERROR: specify image file(s) to scan");
    }

    unsafe {
        // Parse program arguments
        ONESHOT = args.oneshot.into();

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
            return;
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
                return;
            }
        }

        // Clean up
        zbar_processor_destroy(processor);
    }
}
