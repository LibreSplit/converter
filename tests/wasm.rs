#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn wasm_convert_returns_output() {
    let input = include_str!("fixtures/sa2_fallen-hero.lss");
    let output = converter::convert(input.to_owned(), converter::ComparisonMethod::GameTime);

    assert!(!output.is_empty());
}

#[wasm_bindgen_test]
fn wasm_history_returns_zip_blob() {
    let input = "<Run><Offset>00:00:00</Offset><AttemptHistory /><Segments /></Run>";
    let output = converter::convert_history(input.to_owned()).expect("history conversion should succeed");

    assert_eq!(output.type_(), "application/zip");
	assert!(output.size() >= 22.0);
}
