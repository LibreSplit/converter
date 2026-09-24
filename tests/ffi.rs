use std::ffi::{CStr, CString};

use converter::{ComparisonMethod, converter_convert, converter_free_string};

#[test]
fn ffi_conversion_returns_a_converted_string() {
    let input = CString::new(include_str!("fixtures/sa2_fallen-hero.lss"))
        .expect("fixture must not contain NULL bytes");

    let result_ptr = converter_convert(input.as_ptr(), ComparisonMethod::GameTime);
    assert!(!result_ptr.is_null());

    let result = unsafe {
        CStr::from_ptr(result_ptr)
            .to_str()
            .expect("converter should return valid UTF8")
            .to_owned()
    };

    converter_free_string(result_ptr);

    assert!(!result.is_empty());
    assert!(!result.contains("\"error\""));
}

#[test]
fn ffi_conversion_returns_an_error_for_invalid_string() {
    let input = CString::new("<some invalid xml").expect("fixture must not contain NULL bytes");

    let result_ptr = converter_convert(input.as_ptr(), ComparisonMethod::GameTime);
    assert!(!result_ptr.is_null());

    let result = unsafe {
        CStr::from_ptr(result_ptr)
            .to_str()
            .expect("converter should return valid UTF8")
            .to_owned()
    };

    converter_free_string(result_ptr);

    assert!(!result.is_empty());
    assert!(result.contains("\"error\""));
}

#[test]
fn ffi_null_input_returns_null() {
    assert!(converter_convert(std::ptr::null(), ComparisonMethod::GameTime).is_null());
}
