use spex::xml::XmlDocument;

pub struct LiveSplitFile {
    pub game_name: String,
    pub category_name: String,
    pub platform: String,
    pub attempt_count: u32,
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

                    // Get split time.
                    let elm_split_times = elm_segment.opt("SplitTimes").opt("SplitTime").element();
                    let split_time = match elm_split_times {
                        Some(elm_split_time) => {
                            let elm_real_time = elm_split_time.opt("RealTime").element();
                            match elm_real_time {
                                Some(real_time) => {
                                    real_time.text().unwrap_or("0.000000").to_string()
                                }
                                None => "0.000000".to_string(), // default if element is missing.
                            }
                        }
                        None => "0.000000".to_string(),
                    };

                    // Get best segment .
                    let elm_best_segments = elm_segment.opt("BestSegmentTime").element();
                    let best_segment = match elm_best_segments {
                        Some(elm_best_segment) => {
                            let elm_real_time = elm_best_segment.opt("RealTime").element();
                            match elm_real_time {
                                Some(real_time) => {
                                    real_time.text().unwrap_or("0.000000").to_string()
                                }
                                None => "0.000000".to_string(), // default if element is missing.
                            }
                        }
                        None => "0.000000".to_string(),
                    };

                    let segment = Segment { name, split_time, best_segment };
                    segments.push(segment);
                }
            }
            None => {
                let placeholder = Segment {
                    name: "No Splits Provided".to_string(),
                    split_time: "0.000000".to_string(),
                    best_segment: "0.000000".to_string(),
                };
                segments.push(placeholder);
            }
        }

        LiveSplitFile {
            game_name,
            category_name,
            platform,
            attempt_count,
            segments,
        }
    }
}

pub struct Segment {
    pub name: String,
    pub split_time: String,
    pub best_segment: String,
}
