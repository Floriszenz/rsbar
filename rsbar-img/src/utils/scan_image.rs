use std::{
    ffi::CStr,
    path::{Path, PathBuf},
};

use crate::{
    errors::{ProgramError, ProgramResult},
    ffi::{self, ZbarSymbolType},
};

use super::{cli_args::Args, XmlPrinter};

const fn zbar_fourcc(code: [u8; 4]) -> u64 {
    u32::from_le_bytes(code) as u64
}

pub fn scan_image(
    filename: &PathBuf,
    idx: usize,
    processor: *mut libc::c_void,
    args: &Args,
) -> ProgramResult<u8> {
    let zimage = zbar_image_new(filename)?;

    if args.xml {
        XmlPrinter::print_source_head(filename);
    }

    process_image(processor, zimage, filename)?;

    unsafe {
        // output result data
        let symbol_count = output_result(zimage, args, idx);

        ffi::zbar_image_destroy(zimage);

        if ffi::zbar_processor_is_visible(processor) == 1 {
            let rc = ffi::zbar_processor_user_wait(processor, -1);

            if rc < 0 || rc == b'q'.into() || rc == b'Q'.into() {
                // TODO: Handle user quit
            }
        }

        if args.xml {
            XmlPrinter::print_source_foot();
        }

        Ok(symbol_count)
    }
}

fn zbar_image_new(filename: &PathBuf) -> ProgramResult<*mut libc::c_void> {
    let image = image::open(filename)?;

    unsafe {
        let zimage = ffi::zbar_image_create();
        assert!(!zimage.is_null());

        ffi::zbar_image_set_format(zimage, zbar_fourcc([b'Y', b'8', b'0', b'0']));

        let width = image.width();
        let height = image.height();
        ffi::zbar_image_set_size(zimage, width, height);

        let bloblen = (width * height) as usize;
        let blob = libc::malloc(bloblen);
        ffi::zbar_image_set_data(zimage, blob, bloblen as u64, ffi::zbar_image_free_data);

        let bytes = image.into_luma8().into_raw();

        libc::memcpy(blob, bytes.as_ptr().cast(), bloblen);

        Ok(zimage)
    }
}

fn process_image(
    processor: *mut libc::c_void,
    zimage: *mut libc::c_void,
    filename: &Path,
) -> ProgramResult<()> {
    unsafe {
        let processing_result = ffi::zbar_process_image(processor, zimage);

        if processing_result == -1 {
            return Err(ProgramError::ImageProcessFailed(
                filename.display().to_string(),
            ));
        }
    }

    Ok(())
}

fn output_result(zimage: *mut libc::c_void, args: &Args, idx: usize) -> u8 {
    let mut symbol_count: u8 = 0;

    unsafe {
        let mut symbol = ffi::zbar_image_first_symbol(zimage);

        while !symbol.is_null() {
            let symbol_type = ffi::zbar_symbol_get_type(symbol);

            if symbol_type == ZbarSymbolType::ZbarPartial {
                continue;
            } else if !args.xml {
                if !args.raw {
                    print!(
                        "{}:",
                        CStr::from_ptr(ffi::zbar_get_symbol_name(symbol_type))
                            .to_str()
                            .unwrap()
                    );
                }

                if args.polygon {
                    if ffi::zbar_symbol_get_loc_size(symbol) > 0 {
                        print!(
                            "{},{}",
                            ffi::zbar_symbol_get_loc_x(symbol, 0),
                            ffi::zbar_symbol_get_loc_y(symbol, 0)
                        );
                    }

                    for p in 1..ffi::zbar_symbol_get_loc_size(symbol) {
                        print!(
                            " {},{}",
                            ffi::zbar_symbol_get_loc_x(symbol, p),
                            ffi::zbar_symbol_get_loc_y(symbol, p)
                        );
                    }

                    print!(":");
                }

                println!(
                    "{}",
                    CStr::from_ptr(ffi::zbar_symbol_get_data(symbol))
                        .to_str()
                        .unwrap()
                );
            } else if args.xml {
                XmlPrinter::print_index_head(idx as u8);
                let symbol_xml = ffi::zbar_symbol_xml(symbol, &mut std::ptr::null_mut(), &mut 0);

                if let Ok(symbol_xml) = CStr::from_ptr(symbol_xml).to_str() {
                    XmlPrinter::print_symbol(symbol_xml);
                }

                XmlPrinter::print_index_foot();
            }

            symbol_count += 1;

            if args.oneshot {
                break;
            }

            symbol = ffi::zbar_symbol_next(symbol);
        }
    }

    symbol_count
}
