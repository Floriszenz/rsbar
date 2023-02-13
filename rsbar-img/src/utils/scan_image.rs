use std::{ffi::CStr, path::PathBuf};

use crate::{
    errors::{ProgramError, ProgramResult},
    ffi::{self, ZbarSymbolType},
    EXIT_CODE, NOT_FOUND, NUM_SYMBOLS, USE_BINARY_OUTPUT, XMLLVL,
};

use super::cli_args::Args;

const fn zbar_fourcc(a: u8, b: u8, c: u8, d: u8) -> u64 {
    a as u64 | ((b as u64) << 8) | ((c as u64) << 16) | ((d as u64) << 24)
}

pub fn scan_image(
    filename: &PathBuf,
    idx: usize,
    processor: *mut libc::c_void,
    args: &Args,
) -> ProgramResult<()> {
    let mut found_symbol: bool = false;
    let image = image::open(filename)?;

    unsafe {
        let zimage = ffi::zbar_image_create();
        assert!(!zimage.is_null());
        ffi::zbar_image_set_format(zimage, zbar_fourcc(b'Y', b'8', b'0', b'0'));

        let width = image.width();
        let height = image.height();
        ffi::zbar_image_set_size(zimage, width, height);

        let bloblen = (width * height) as usize;
        let blob = libc::malloc(bloblen);
        ffi::zbar_image_set_data(zimage, blob, bloblen as u64, ffi::zbar_image_free_data);

        let bytes = image.into_luma8().into_raw();

        libc::memcpy(blob, bytes.as_ptr().cast(), bloblen);

        if XMLLVL == 1 {
            XMLLVL += 1;
            println!("<source href='{}'>", filename.display());
        }

        let processing_result = ffi::zbar_process_image(processor, zimage);

        if processing_result == -1 {
            return Err(ProgramError::ImageProcessFailed(
                filename.display().to_string(),
            ));
        }

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

            found_symbol = true;
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

        if !found_symbol {
            NOT_FOUND = true;
        }
    }

    Ok(())
}
