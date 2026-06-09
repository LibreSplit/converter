use std::io::Cursor;

use spex::parsing::XmlReader;

mod libresplit;
mod livesplit;

fn convert_inner(file: String) -> Result<String, String> {
    let cursor = Cursor::new(file);
    let xml = XmlReader::parse_auto(cursor).map_err(|e| e.to_string())?;
    let livesplit_data = livesplit::LiveSplitFile::new(xml);
    Ok(libresplit::LibreSplitFile::from_livesplit(livesplit_data).get())
}

// Build the library for WASM targets.
// Used on the LibreSplit website, for converting splits.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn convert(file: String) -> String {
    convert_inner(file).unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e))
}
