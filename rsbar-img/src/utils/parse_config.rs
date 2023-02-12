use std::borrow::BorrowMut;

use crate::{
    errors::{ProgramError, ProgramResult},
    ffi::{self, ZbarConfig, ZbarSymbolType},
    BINARY,
};

fn zbar_processor_parse_config(
    processor: *mut libc::c_void,
    config_string: &str,
) -> ProgramResult<()> {
    let mut sym: ZbarSymbolType = ZbarSymbolType::ZbarNone;
    let mut cfg: ZbarConfig = ZbarConfig::Enable;
    let mut val: libc::c_int = 0;

    unsafe {
        if ffi::zbar_parse_config(
            config_string.as_ptr().cast(),
            sym.borrow_mut(),
            cfg.borrow_mut(),
            val.borrow_mut(),
        ) != 0
        {
            return Err(ProgramError::ConfigParseFailed(String::from(config_string)));
        }

        if ffi::zbar_processor_set_config(processor, sym, cfg, val) != 0 {
            return Err(ProgramError::ConfigSetFailed(String::from(config_string)));
        }
    }

    Ok(())
}

pub fn parse_config(processor: *mut libc::c_void, config_string: &str) -> ProgramResult<()> {
    zbar_processor_parse_config(processor, config_string)?;

    if config_string == "binary" {
        unsafe {
            BINARY = 1;
        }
    }

    Ok(())
}
