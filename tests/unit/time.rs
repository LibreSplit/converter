use super::LiveSplitFile;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn parses_livesplit_durations() {
	assert_eq!(LiveSplitFile::parse_time("00:01:02.123456789"), Some(62123456789));
	assert_eq!(LiveSplitFile::parse_time("1.02:03:04.123456789"), Some(93784123456789));
	assert_eq!(LiveSplitFile::parse_time(" -00:00:05.25"), Some(-5250000000));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn formats_libresplit_durations() {
	assert_eq!(LiveSplitFile::format_time(0), "00:00:00.000000");
	assert_eq!(LiveSplitFile::format_time(93784123456789), "26:03:04.123456");
	assert_eq!(LiveSplitFile::format_time(-5250000000), "-00:00:05.250000");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn conversion_for_sub_microsecond_times_are_correct() {
	assert_eq!(LiveSplitFile::convert_time("00:00:00.000001999"), "00:00:00.000001");
	// this should truncate the 999 and -0 should be just 0.
	assert_eq!(LiveSplitFile::convert_time("-00:00:00.000000999"), "00:00:00.000000");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn invalid_livesplit_durations_convert_to_missing_times() {
	for source in ["", "-", "00:60:00", "00:00:60", "00:00:00.1234567890"] {
		assert_eq!(LiveSplitFile::parse_time(source), None, "{source:?}");
		assert_eq!(LiveSplitFile::convert_time(source), "-", "{source:?}");
	}
}
