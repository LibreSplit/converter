use super::LibreSplitFile;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use spex::parsing::XmlReader;
use std::io::Cursor;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

fn fixture_icon() -> String {
	let file = XmlReader::parse_auto(Cursor::new(include_str!("../../tests/fixtures/sa2_fallen-hero.lss"))).unwrap();
	file.root().req("GameIcon").text().unwrap().to_owned()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn empty_icons_remain_empty() {
    for source in ["", "<![CDATA[]]>", " \n<![CDATA[ \t\n]]> "] {
        assert_eq!(LibreSplitFile::convert_icon(source).unwrap(), "");
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn embedded_fixture_icons_preserve_the_original_image_bytes() {
    let icon = fixture_icon();
	let serialized = STANDARD.decode(&icon).unwrap();

	let expected = &serialized[161..serialized.len() - 1];
	assert!(expected.starts_with(b"\x89PNG\r\n\x1a\n"));
	let wrapped = format!(" \n<![CDATA[\n{icon}\n]]> \n");
	let spaced = icon
		.as_bytes()
		.chunks(73)
		.map(|chunk| std::str::from_utf8(chunk).unwrap())
		.collect::<Vec<_>>()
		.join("\n\t");

	for source in [&icon, &wrapped, &spaced] {
		let uri = LibreSplitFile::convert_icon(source).unwrap();
		let encoded_image = uri.strip_prefix("data:image/png;base64,").unwrap();
		assert_eq!(STANDARD.decode(encoded_image).unwrap(), expected);
	}
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn relative_paths_are_preserved_exactly() {
    for source in [
        "icons/boss.png",
        "../icons/boss.png",
        "./icons/../boss.png",
        r"icons\boss.png",
        r"./foo\icon.png",
        r"C:icons\boss.png",
        r"\icons\boss.png",
        " icon.png ",
        " ",
        "\ticon.png\n",
    ] {
        assert_eq!(LibreSplitFile::convert_icon(source).unwrap(), source);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn unix_paths_escape_filename_characters_without_changing_them() {
    for (source, expected) in [
        ("/tmp/icon.png ", "file:///tmp/icon.png%20"),
        ("/tmp/ icon.png", "file:///tmp/%20icon.png"),
        (r"/tmp/foo\icon.png", "file:///tmp/foo%5Cicon.png"),
        ("/tmp/icon#1?.png", "file:///tmp/icon%231%3F.png"),
        ("/tmp/icon%20.png", "file:///tmp/icon%2520.png"),
        ("/tmp/été.png", "file:///tmp/%C3%A9t%C3%A9.png"),
        ("/tmp/icon\n.png", "file:///tmp/icon%0A.png"),
        ("//tmp/icon.png", "file:////tmp/icon.png"),
    ] {
        assert_eq!(LibreSplitFile::convert_icon(source).unwrap(), expected, "{source:?}");
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn windows_absolute_paths_keep_the_drive_or_share() {
    for (source, expected) in [
        (r"C:\Icons\My Icon.png", "file:///C:/Icons/My%20Icon.png"),
        ("D:/Icons/icon#1.png", "file:///D:/Icons/icon%231.png"),
        (r"\\server\share\icon.png", "file://server/share/icon.png"),
        (
            r"\\server\My Share\icon.png",
            "file://server/My%20Share/icon.png",
        ),
        (r"\\?\C:\Icons\icon.png", "file:///C:/Icons/icon.png"),
        (
            r"\\?\UNC\server\share\icon.png",
            "file://server/share/icon.png",
        ),
    ] {
        assert_eq!(LibreSplitFile::convert_icon(source).unwrap(), expected, "{source:?}");
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn existing_uris_are_preserved_exactly() {
    for source in [
        "https://example.com/My%20Icon.png?version=1#icon",
        "HTTP://example.com/icon.png",
        "file:///tmp/icon%20.png",
        "file://server/share/icon.png",
        "data:image/png;base64,iVBORw0KGgo=",
    ] {
        assert_eq!(LibreSplitFile::convert_icon(source).unwrap(), source);
    }
}
