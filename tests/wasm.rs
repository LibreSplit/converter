#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn wasm_convert_returns_output() {
	let input = include_str!("fixtures/sa2_fallen-hero.lss");
	let output = converter::convert(input.to_owned(), converter::ComparisonMethod::GameTime);

	assert!(!output.is_empty());
}
