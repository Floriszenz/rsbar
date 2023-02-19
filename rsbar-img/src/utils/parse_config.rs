use anyhow::{anyhow, Result};

use crate::ffi::{self, ZbarConfig, ZbarSymbolType};

pub fn zbar_processor_parse_config(
    processor: *mut libc::c_void,
    config_string: &str,
) -> Result<()> {
    let mut sym: ZbarSymbolType = ZbarSymbolType::ZbarNone;
    let mut cfg: ZbarConfig = ZbarConfig::Enable;
    let mut val: libc::c_int = 0;

    unsafe {
        if ffi::zbar_parse_config(config_string.as_ptr().cast(), &mut sym, &mut cfg, &mut val) != 0
        {
            return Err(anyhow!("Failed to parse the config `{config_string}`"));
        }

        if ffi::zbar_processor_set_config(processor, sym, cfg, val) != 0 {
            return Err(anyhow!(
                "Failed to set the config `{config_string}` for the processor"
            ));
        }
    }

    Ok(())
}
