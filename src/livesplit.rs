use spex::xml::XmlDocument;

use crate::libresplit::Time;

pub struct LiveSplitFile {
    pub game_name: String,
    pub category_name: String,
	pub game_icon: String,
    pub _platform: String, // unused
    pub attempt_count: u32,
	pub finished_count: u32,
    pub segments: Vec<Segment>,
}

impl LiveSplitFile {
    pub fn new(file: XmlDocument) -> Self {
        // Read game name.
        let elm_game_name = file.root().opt("GameName").element();
        let game_name = match elm_game_name {
            Some(name) => name.text().expect("Unknown Game"),
            None => "Unknown Game",
        }
        .to_string();

        // Read category.
        let elm_category_name = file.root().opt("CategoryName").element();
        let category_name = match elm_category_name {
            Some(category) => category.text().expect("Unknown Category"),
            None => "Unknown Category",
        }
        .to_string();

		// Read game icon.
		let elm_game_icon = file.root().opt("GameIcon").element();
		let game_icon = match elm_game_icon {
			Some(icon) => icon.text().expect("Unknown Game Icon"),
			None => "Unknown Game Icon",
		}
		.to_string();

        // Read platform.
        let elm_platform = file.root().opt("Platform").element();
        let platform = match elm_platform {
            Some(plat) => plat.text().expect("Unknown Platform"),
            None => "Unknown Platform",
        }
        .to_string();

        // Read attempt count.
        let elm_attempt_count = file.root().opt("AttemptCount").element();
        let attempt_count_str = match elm_attempt_count {
            Some(count_str) => count_str.text().expect("0"),
            None => "0",
        };
        let attempt_count: u32 = attempt_count_str.trim().parse().unwrap_or(0);
		let finished_count: u32 = Self::get_finished_count(&file);

        // Read splits.
        let mut segments: Vec<Segment> = Vec::new();
        let elm_segments = file.root().opt("Segments").element();
        match elm_segments {
            Some(segments_iter) => {
                for elm_segment in segments_iter.elements().filter(|e| e.is_named("Segment")) {
                    // Get split name.
                    let elm_name = elm_segment.opt("Name").element();
                    let name = match elm_name {
                        Some(name) => name.text().unwrap_or("Unknown Split").to_string(),
                        None => "Unknown Split".to_string(),
                    };

					// Get icon.
					let elm_icon = elm_segment.opt("Icon").element();
					let icon = match elm_icon {
						Some(icon) => icon.text().unwrap_or("").to_string(),
						None => "".to_string(),
					};

                    // Get split time.
                    let elm_split_times = elm_segment.opt("SplitTimes").opt("SplitTime").element();
                    let split_real_time = match elm_split_times {
                        Some(elm_split_time) => {
                            let elm_real_time = elm_split_time.opt("RealTime").element();
                            match elm_real_time {
                                Some(real_time) => {
                                    real_time.text().unwrap_or("-").to_string()
                                }
                                None => "-".to_string(), // default if element is missing.
                            }
                        }
                        None => "-".to_string(),
                    };

					let split_game_time = match elm_split_times {
                        Some(elm_split_time) => {
                            let elm_real_time = elm_split_time.opt("GameTime").element();
                            match elm_real_time {
                                Some(real_time) => {
                                    real_time.text().unwrap_or("-").to_string()
                                }
                                None => "-".to_string(), // default if element is missing.
                            }
                        }
                        None => "-".to_string(),
                    };

					let split_time = Time {
						real_time: split_real_time,
						game_time: split_game_time,
					};

                    // Get best segment .
                    let elm_best_segments = elm_segment.opt("BestSegmentTime").element();
                    let best_segment_real = match elm_best_segments {
                        Some(elm_best_segment) => {
                            let elm_real_time = elm_best_segment.opt("RealTime").element();
                            match elm_real_time {
                                Some(real_time) => {
                                    real_time.text().unwrap_or("-").to_string()
                                }
                                None => "-".to_string(), // default if element is missing.
                            }
                        }
                        None => "-".to_string(),
                    };

					let best_segment_game = match elm_best_segments {
                        Some(elm_best_segment) => {
                            let elm_real_time = elm_best_segment.opt("GameTime").element();
                            match elm_real_time {
                                Some(real_time) => {
                                    real_time.text().unwrap_or("-").to_string()
                                }
                                None => "-".to_string(), // default if element is missing.
                            }
                        }
                        None => "-".to_string(),
                    };

					let best_segment = Time {
						real_time: best_segment_real,
						game_time: best_segment_game,
					};

					let best_time = Time {
						real_time: "".to_string(),
						game_time: "".to_string(),
					};

                    let segment = Segment { name, icon, split_time, best_time, best_segment };
                    segments.push(segment);
                }
            }
            None => {
                let placeholder = Segment {
                    name: "No Splits Provided".to_string(),
					icon: "".to_string(),
                    split_time: Time {
						real_time: "-".to_string(),
						game_time: "-".to_string(),
					},
                    best_time: Time {
						real_time: "-".to_string(),
						game_time: "-".to_string(),
					},
                    best_segment: Time {
						real_time: "-".to_string(),
						game_time: "-".to_string(),
					},
                };
                segments.push(placeholder);
            }
        }

        LiveSplitFile {
            game_name,
            category_name,
			game_icon,
            _platform: platform,
            attempt_count,
			finished_count,
            segments,
        }
    }

	fn get_finished_count(file: &XmlDocument) -> u32 {
		let Some(attempts) = file.root().opt("AttemptHistory").element() else {
			return 0;
		};

		let mut finished_attempts = 0;
		for attempt in attempts.elements().filter(|e| e.is_named("Attempt")) {
			if attempt.elements().any(|child| child.is_named("RealTime") || child.is_named("GameTime")) {
				finished_attempts += 1;
			}
		}

		return finished_attempts;
	}
}

pub struct Segment {
    pub name: String,
	pub icon: String,
    pub split_time: Time,
	pub best_time: Time,
    pub best_segment: Time,
}
