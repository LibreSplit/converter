use std::{
    ffi::{CStr, CString},
    io::Cursor,
    os::raw::c_char,
    ptr::null_mut,
};

use spex::parsing::XmlReader;

mod libresplit;
mod livesplit;

// Shared logic for both interfaces.
fn convert_inner(file: &str) -> Result<String, String> {
    let cursor = Cursor::new(file);
    let xml = XmlReader::parse_auto(cursor).map_err(|e| e.to_string())?;
    let livesplit_data = livesplit::LiveSplitFile::new(xml);
    Ok(libresplit::LibreSplitFile::from_livesplit(livesplit_data).get())
}

// Build the library for WASM targets.
// Used on the LibreSplit website, for converting splits.
// Accepts a LiveSplit XML file as a string and returns LibreSplit JSON.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn convert(file: String) -> String {
    convert_inner(&file).unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e))
}

// C FFI entrypoints used by the split editor.

// 'converter_convert' takes a null-terminated UTF-8 string and returns an owned C string.
#[unsafe(no_mangle)]
pub extern "C" fn converter_convert(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return null_mut();
    }

    let input = unsafe {
        match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return null_mut(),
        }
    };

    match convert_inner(input) {
        Ok(output) => CString::new(output).unwrap().into_raw(),
        Err(error) => CString::new(format!("{{\"error\":\"{}\"}}", error))
            .unwrap()
            .into_raw(),
    }
}

// The caller must free the returned pointer with 'converter_free_string'.
#[unsafe(no_mangle)]
pub extern "C" fn converter_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
