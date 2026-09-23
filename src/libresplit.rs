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
		let icon = lss.game_icon;
        let attempt_count = lss.attempt_count;
		let finished_count = lss.finished_count;

        // Constructs splits vector.
        let mut splits: Vec<Split> = Vec::new();
        for lss_split in lss.segments {
            let split = Split {
                title: lss_split.name,
                time: lss_split.split_time,
                best_time: "0.000000".to_string(),
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
		//todo: implement
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Split {
    pub title: String,
    pub time: String,
    pub best_time: String,
    pub best_segment: String,
}
