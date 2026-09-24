use converter::{ComparisonMethod, convert};

#[test]
fn converts_a_valid_livesplit_file() {
	let input = include_str!("fixtures/sa2_fallen-hero.lss");
	let output = convert(input.to_owned(), ComparisonMethod::GameTime);

	assert!(!output.contains("\"error\""));
	assert!(!output.is_empty());
}

#[test]
fn returns_an_error_for_invalid_xml() {
	let output = convert("<some invalid xml".to_owned(), ComparisonMethod::GameTime);

	assert!(output.contains("\"error\""));
}
