use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use crate::livesplit::LiveSplitFile;

#[derive(Serialize, Deserialize, Debug)]
pub struct LibreSplitFile {
    pub name: String,
	pub category: String,
	pub icon: String,
    pub attempt_count: u32,
	pub finished_count: u32,
    pub splits: Vec<Split>,
    pub width: u32,
    pub height: u32,
}

impl LibreSplitFile {
    pub fn from_livesplit(lss: LiveSplitFile) -> Self {
        let name = lss.game_name;
		let category = lss.category_name;
		let icon = Self::convert_icon(&lss.game_icon).unwrap_or("".to_string());
        let attempt_count = lss.attempt_count;
		let finished_count = lss.finished_count;

        // Constructs splits vector.
        let mut splits: Vec<Split> = Vec::new();
        for lss_split in lss.segments {
            let split = Split {
                title: lss_split.name,
				icon: Self::convert_icon(lss_split.icon.as_str()).unwrap_or("".to_string()),
                time: lss_split.split_time,
                best_time: lss_split.best_time,
                best_segment: lss_split.best_segment,
            };
            splits.push(split);
        }

        // Get size.
        // The window of LibreSplit will not shrink beyond this size.
        let width = 60;
        let height = 80;

        LibreSplitFile {
            name,
			category,
			icon,
            attempt_count,
			finished_count,
            splits,
            width,
            height,
        }
    }

    pub fn get(&self) -> String {
        let rtn = to_string_pretty(&self).unwrap_or("".to_string());
        format!("{}", rtn)
    }

	pub fn convert_icon(source: &str) -> Result<String, String> {
		if source.is_empty() {
			return Ok(String::new());
		}

		if source.contains('\0') {
			return Err("Icon source contains a NULL byte".to_owned());
		}

		use base64::{Engine as _, engine::general_purpose::STANDARD};
		use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
		use typed_path::{Utf8WindowsPath, Utf8WindowsPrefix};
		use xml::reader::{EventReader, XmlEvent};

		let wrapped = source.trim().starts_with("<![CDATA[");
		let icon = if wrapped {
			source.trim().strip_prefix("<![CDATA[").and_then(|text| text.strip_suffix("]]>")).ok_or("Unterminated icon CDATA")?.trim()
		} else {
			source
		};

		let windows_path = Utf8WindowsPath::new(source);
		let windows_components = windows_path.components();
		let windows_prefix = if source.starts_with('/') {
			None
		} else {
			windows_components.prefix_kind()
		};

		if !wrapped && windows_prefix.is_none() {
			if let Some((scheme, _)) = source.split_once(':') {
				if scheme.starts_with(|c: char| c.is_ascii_alphabetic()) && scheme.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'-' | b'.')) {
					return Ok(source.to_owned());
				}
			}
		}

		let encoded: Vec<u8> = icon.bytes().filter(|c| !c.is_ascii_whitespace()).collect();
		if wrapped || encoded.starts_with(b"AAEAAAD/////") {
			if encoded.is_empty() {
				return Ok(String::new());
			}

			let decoded = STANDARD.decode(&encoded).map_err(|error| format!("Invalid base64 in embedded icon: {error}"))?;
			const STREAM_HEADER: &[u8] = b"\x00\x01\x00\x00\x00\xff\xff\xff\xff\x01\x00\x00\x00\x00\x00\x00\x00";
			if !decoded.starts_with(STREAM_HEADER) {
				return Err("Invalid .NET stream header in embedded icon".to_owned());
			}

			let image = decoded.windows(10).enumerate().find_map(|(offset, record)| {
				if record[0] != 15 || record[9] != 2 {
					return None;
				}

				let length = i32::from_le_bytes(record[5..9].try_into().ok()?);
				let start = offset + 10;
				let end = start.checked_add(usize::try_from(length).ok()?)?;
				let bitmap_type = b"System.Drawing.Bitmap";
				if decoded.get(end..) != Some(&[11][..]) || !decoded[..offset].windows(bitmap_type.len()).any(|text| text == bitmap_type) {
					return None;
				}

				decoded.get(start..end)
			})
			.ok_or("Invalid .NET bitmap byte array in embedded icon")?;

			let mime = infer::get(image)
				.filter(|format| format.matcher_type() == infer::MatcherType::Image)
				.map(|format| format.mime_type())
				.or_else(|| {
					for event in EventReader::new(image) {
						if let XmlEvent::StartElement { name, .. } = event.ok()? {
							return (name.local_name == "svg" && name.namespace.as_deref().is_none_or(|namespace| {
								namespace == "http://www.w3.org/2000/svg"
							}))
							.then_some("image/svg+xml");
						}
					}
					None
				})
				.ok_or("Unsupported image format in embedded icon")?;

			let mut uri = format!("data:{mime};base64,");
			STANDARD.encode_string(image, &mut uri);
			return Ok(uri);
		}

		let (scheme, path) = if let Some(prefix) = windows_prefix {
			if !windows_path.is_absolute() {
				return Ok(source.to_owned());
			}

			let unix_path = windows_path.with_unix_encoding();
			match prefix {
				Utf8WindowsPrefix::Disk(drive) | Utf8WindowsPrefix::VerbatimDisk(drive) => {
					("file://", format!("/{drive}:{unix_path}"))
				}
				Utf8WindowsPrefix::UNC(server, share) | Utf8WindowsPrefix::VerbatimUNC(server, share) => {
					("file://", format!("{server}/{share}{unix_path}"))
				}
				_ => return Err("Unsupported Windows icon path prefix".to_owned()),
			}
		} else if source.starts_with('/') {
			("file://", source.to_owned())
		} else {
			return Ok(source.to_owned());
		};

		const PATH_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
			.remove(b'-')
			.remove(b'.')
			.remove(b'_')
			.remove(b'~')
			.remove(b'/')
			.remove(b':');

		Ok(format!("{scheme}{}", utf8_percent_encode(&path, PATH_ENCODE_SET)))
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Split {
    pub title: String,
	pub icon: String,
    pub time: Time,
    pub best_time: Time,
    pub best_segment: Time,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Time {
	pub real_time: String,
	pub game_time: String,
}

#[cfg(test)]
#[path = "../tests/unit/icon.rs"]
mod tests;
