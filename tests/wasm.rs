#![cfg(target_arch = "wasm32")]

use std::io::{Cursor, Read};

use js_sys::Uint8Array;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use zip::ZipArchive;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn wasm_convert_returns_output() {
    let input = include_str!("fixtures/sa2_fallen-hero.lss");
    let output = converter::convert(input.to_owned(), converter::ComparisonMethod::GameTime);

    assert!(!output.is_empty());
}

#[wasm_bindgen_test]
async fn wasm_history_returns_zip_blob() {
    let input = r#"
		<Run version="1.7.0">
			<Offset>00:00:00</Offset>
			<AttemptHistory>
				<Attempt id="1" started="01/01/2025 10:00:00" ended="01/01/2025 10:00:03">
					<RealTime>00:00:03</RealTime>
				</Attempt>
			</AttemptHistory>
		<Segments />
	</Run>"#;

    let output =
        converter::convert_history(input.to_owned(), r"Example Game: Any%\practice".to_owned())
            .expect("history conversion should succeed");

    assert_eq!(output.type_(), "application/zip");
    let buffer = JsFuture::from(output.array_buffer()).await.unwrap();
    let bytes = Uint8Array::new(&buffer).to_vec();
    let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("Blob should contain a ZIP");
    assert_eq!(archive.len(), 1);
    let mut file = archive.by_index(0).unwrap();
    assert_eq!(file.name(), r"Example Game: Any%\practice/2026-09-23.json");
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();
    let attempts: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(attempts[0]["final_time"]["real_time"], "00:00:03.000000");
}

#[wasm_bindgen_test]
fn wasm_history_rejects_invalid_directory() {
    assert!(converter::convert_history("<Run />".to_owned(), "../splits".to_owned()).is_err());
}
