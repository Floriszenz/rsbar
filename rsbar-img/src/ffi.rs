#![allow(dead_code)]

#[link(name = "zbar", kind = "static")]
extern "C" {
    pub fn zbar_set_verbosity(level: libc::c_int);

    pub fn zbar_processor_create(threaded: libc::c_int) -> *mut libc::c_void;

    pub fn zbar_processor_request_dbus(
        proc: *mut libc::c_void,
        req_dbus_enabled: libc::c_int,
    ) -> libc::c_int;

    pub fn zbar_processor_init(
        proc: *const libc::c_void,
        dev: *const libc::c_char,
        enable_display: libc::c_int,
    ) -> libc::c_int;

    pub fn _zbar_error_spew(container: *const libc::c_void, verbosity: libc::c_int) -> libc::c_int;

    pub fn zbar_processor_set_visible(proc: *mut libc::c_void, visible: libc::c_int)
        -> libc::c_int;

    pub fn zbar_parse_config(
        cfgstr: *const libc::c_char,
        sym: *mut ZbarSymbolType,
        cfg: *mut ZbarConfig,
        val: *mut libc::c_int,
    ) -> libc::c_int;

    pub fn zbar_processor_set_config(
        proc: *mut libc::c_void,
        sym: ZbarSymbolType,
        cfg: ZbarConfig,
        val: libc::c_int,
    ) -> libc::c_int;

    pub fn zbar_image_create() -> *mut libc::c_void;

    pub fn zbar_image_set_format(img: *mut libc::c_void, fmt: libc::c_ulong);

    pub fn zbar_image_set_size(img: *mut libc::c_void, w: libc::c_uint, h: libc::c_uint);

    pub fn zbar_image_set_data(
        img: *mut libc::c_void,
        data: *const libc::c_void,
        len: libc::c_ulong,
        cleanup: unsafe extern "C" fn(*mut libc::c_void),
    );

    pub fn zbar_image_free_data(img: *mut libc::c_void);

    pub fn zbar_process_image(proc: *mut libc::c_void, img: *mut libc::c_void) -> libc::c_int;

    pub fn zbar_image_first_symbol(img: *const libc::c_void) -> *const libc::c_void;

    pub fn zbar_symbol_next(sym: *const libc::c_void) -> *const libc::c_void;

    pub fn zbar_symbol_get_type(sym: *const libc::c_void) -> ZbarSymbolType;

    pub fn zbar_get_symbol_name(sym: ZbarSymbolType) -> *const libc::c_char;

    pub fn zbar_symbol_get_loc_size(sym: *const libc::c_void) -> libc::c_uint;

    pub fn zbar_symbol_get_loc_x(sym: *const libc::c_void, idx: libc::c_uint) -> libc::c_int;

    pub fn zbar_symbol_get_loc_y(sym: *const libc::c_void, idx: libc::c_uint) -> libc::c_int;

    pub fn zbar_symbol_get_data(sym: *const libc::c_void) -> *const libc::c_char;

    pub fn zbar_symbol_xml(
        sym: *const libc::c_void,
        buf: *mut *mut libc::c_char,
        len: *mut libc::c_uint,
    ) -> *const libc::c_char;

    pub fn zbar_image_destroy(img: *mut libc::c_void);

    pub fn zbar_processor_is_visible(proc: *mut libc::c_void) -> libc::c_int;

    pub fn zbar_processor_user_wait(proc: *mut libc::c_void, timeout: libc::c_int) -> libc::c_int;

    pub fn zbar_processor_destroy(proc: *mut libc::c_void);
}

#[repr(C)]
#[derive(PartialEq)]
pub enum ZbarSymbolType {
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
pub enum ZbarConfig {
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
