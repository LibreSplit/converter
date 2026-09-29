use std::{
    ffi::{CStr, CString},
    io::Cursor,
};

use converter::{
    ComparisonMethod, converter_convert, converter_convert_history, converter_free_bytes,
    converter_free_string,
};
use zip::ZipArchive;

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

#[test]
fn ffi_history_conversion_returns_a_zip_buffer() {
    let input = CString::new(
        r#"
        <Run version="1.7.0">
          <Offset>00:00:00</Offset>
          <AttemptHistory>
            <Attempt id="1" started="09/23/2026 10:00:00" ended="09/23/2026 10:00:03">
              <RealTime>00:00:03</RealTime>
            </Attempt>
          </AttemptHistory>
          <Segments />
        </Run>
        "#,
    )
    .unwrap();
    let mut len = 0;

    let result_ptr = converter_convert_history(input.as_ptr(), &mut len);
    assert!(!result_ptr.is_null());
    assert!(len > 0);

    {
        let result = unsafe { std::slice::from_raw_parts(result_ptr, len) };
        let mut archive = ZipArchive::new(Cursor::new(result)).expect("output should be a ZIP");
        assert_eq!(archive.len(), 1);
        assert_eq!(archive.by_index(0).unwrap().name(), "2026-09-23.json");
    }

    converter_free_bytes(result_ptr, len);
}

#[test]
fn ffi_history_conversion_returns_null_for_invalid_input() {
    let input = CString::new("<Run").unwrap();
    let mut len = usize::MAX;

    let result_ptr = converter_convert_history(input.as_ptr(), &mut len);

    assert!(result_ptr.is_null());
    assert_eq!(len, 0);
}

#[test]
fn ffi_history_conversion_rejects_null_pointers() {
    let input = CString::new("<Run />").unwrap();
    let mut len = usize::MAX;

    assert!(converter_convert_history(std::ptr::null(), &mut len).is_null());
    assert_eq!(len, 0);
    assert!(converter_convert_history(input.as_ptr(), std::ptr::null_mut()).is_null());

    converter_free_bytes(std::ptr::null_mut(), 0);
}
