use std::collections::HashMap;

use spex::xml::XmlDocument;

use crate::libresplit::Time;

pub struct LiveSplitFile {
    pub game_name: String,
    pub category_name: String,
    pub game_icon: String,
    pub _platform: String, // unused
    pub attempt_count: u32,
    pub finished_count: u32,
    pub start_delay: String,
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
            Some(icon) => icon.text().expect(""),
            None => "",
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

        // Read offset and convert to start_delay
        let offset = file
            .root()
            .opt("Offset")
            .element()
            .and_then(|offset| offset.text().ok())
            .and_then(Self::parse_time)
            .unwrap_or(0);

        let start_delay = Self::format_time(-offset);

        // Read splits.
        let mut segments: Vec<Segment> = Vec::new();
        let mut attempt_times: HashMap<i32, [Option<i128>; 2]> = HashMap::new();
        let elm_segments = file.root().opt("Segments").element();
        match elm_segments {
            Some(segments_iter) => {
                for (segment_idx, elm_segment) in segments_iter
                    .elements()
                    .filter(|e| e.is_named("Segment"))
                    .enumerate()
                {
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
                    let elm_split_times =
                        elm_segment.opt("SplitTimes").element().and_then(|times| {
                            times.elements().find(|time| {
                                // split must be the PB split
                                time.is_named("SplitTime")
                                    && time.att_opt("name") == Some("Personal Best")
                            })
                        });

                    let split_real_time = match elm_split_times {
                        Some(elm_split_time) => {
                            let elm_real_time = elm_split_time.opt("RealTime").element();
                            match elm_real_time {
                                Some(real_time) => {
                                    Self::convert_time(real_time.text().unwrap_or("-"))
                                }
                                None => "-".to_string(), // default if element is missing.
                            }
                        }
                        None => "-".to_string(),
                    };

                    let split_game_time = match elm_split_times {
                        Some(elm_split_time) => {
                            let elm_game_time = elm_split_time.opt("GameTime").element();
                            match elm_game_time {
                                Some(game_time) => {
                                    Self::convert_time(game_time.text().unwrap_or("-"))
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
                                    Self::convert_time(real_time.text().unwrap_or("-"))
                                }
                                None => "-".to_string(), // default if element is missing.
                            }
                        }
                        None => "-".to_string(),
                    };

                    let best_segment_game = match elm_best_segments {
                        Some(elm_best_segment) => {
                            let elm_game_time = elm_best_segment.opt("GameTime").element();
                            match elm_game_time {
                                Some(game_time) => {
                                    Self::convert_time(game_time.text().unwrap_or("-"))
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

                    // we need to go through each attempt and keep track of best times to convert rainbows.
                    let mut best = [None; 2];
                    let mut next_attempt_times = HashMap::new();
                    if let Some(history) = elm_segment.opt("SegmentHistory").element() {
                        for record in history.elements().filter(|e| e.is_named("Time")) {
                            let Some(id) = record
                                .att_opt("id")
                                .and_then(|id| id.trim().parse::<i32>().ok())
                                .filter(|id| *id > 0)
                            else {
                                continue;
                            };

                            let mut total = if segment_idx == 0 {
                                [Some(0); 2]
                            } else {
                                attempt_times.remove(&id).unwrap_or([None; 2])
                            };

                            for (method_idx, method) in ["RealTime", "GameTime"].iter().enumerate()
                            {
                                let Some(time) = record.opt(*method).element() else {
                                    continue;
                                };

                                let text = time.text().unwrap_or("-").trim();
                                if text.is_empty() {
                                    continue;
                                }

                                total[method_idx] = total[method_idx]
                                    .and_then(|elapsed| {
                                        elapsed.checked_add(Self::parse_time(text)?)
                                    })
                                    .filter(|time| time.unsigned_abs() / 1000 < i64::MAX as u128);
                                best[method_idx] =
                                    best[method_idx].into_iter().chain(total[method_idx]).min();
                            }

                            next_attempt_times.insert(id, total);
                        }
                    }

                    attempt_times = next_attempt_times;
                    let best_time = Time {
                        real_time: best[0]
                            .map(Self::format_time)
                            .unwrap_or_else(|| "-".to_owned()),
                        game_time: best[1]
                            .map(Self::format_time)
                            .unwrap_or_else(|| "-".to_owned()),
                    };

                    let segment = Segment {
                        name,
                        icon,
                        split_time,
                        best_time,
                        best_segment,
                    };
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
            start_delay,
            segments,
        }
    }

    fn convert_time(text: &str) -> String {
        Self::parse_time(text)
            .map(Self::format_time)
            .unwrap_or_else(|| "-".to_owned())
    }

    // parse time from livesplit format to libresplit's nanosecond long value for accurate conversion
    fn parse_time(text: &str) -> Option<i128> {
        fn number(text: &str) -> Option<i128> {
            if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }

            text.parse().ok()
        }

        let text = text.trim();
        let negative = text.starts_with('-');
        let mut parts = text.strip_prefix('-').unwrap_or(text).split(':');
        let hours = parts.next()?;
        let minutes = number(parts.next()?)?;
        let seconds = parts.next()?;
        if parts.next().is_some() || minutes >= 60 {
            return None;
        }

        let (days, hours) = match hours.split_once('.') {
            Some((days, hours)) => {
                let hours = number(hours)?;
                if hours >= 24 {
                    return None;
                }
                (number(days)?, hours)
            }
            None => (0, number(hours)?),
        };

        let (seconds, nanos) = match seconds.split_once('.') {
            Some((seconds, fraction)) => {
                if fraction.len() > 9 {
                    return None;
                }
                let nanos =
                    number(fraction)?.checked_mul(10_i128.pow(9 - fraction.len() as u32))?;
                (number(seconds)?, nanos)
            }
            None => (number(seconds)?, 0),
        };

        if seconds >= 60 {
            return None;
        }

        let total = days
            .checked_mul(24)?
            .checked_add(hours)?
            .checked_mul(60)?
            .checked_add(minutes)?
            .checked_mul(60)?
            .checked_add(seconds)?
            .checked_mul(1000000000)?
            .checked_add(nanos)?;

        if total / 1000 >= i128::from(i64::MAX) {
            return None;
        }

        Some(if negative { -total } else { total })
    }

    // format nanos to libresplit json string format
    fn format_time(nanos: i128) -> String {
        let micros = nanos / 1_000;
        let sign = if micros < 0 { "-" } else { "" };
        let micros = micros.unsigned_abs();

        let total_seconds = micros / 1_000_000;
        let hours = total_seconds / 3_600;
        let minutes = (total_seconds / 60) % 60;
        let seconds = total_seconds % 60;
        let micros = micros % 1_000_000;
        format!("{sign}{hours:02}:{minutes:02}:{seconds:02}.{micros:06}")
    }

    // an attempt with a RealTime or GameTime value is a finished attempt
    fn get_finished_count(file: &XmlDocument) -> u32 {
        let Some(attempts) = file.root().opt("AttemptHistory").element() else {
            return 0;
        };

        let mut finished_attempts = 0;
        for attempt in attempts.elements().filter(|e| e.is_named("Attempt")) {
            if attempt.elements().any(|child| {
                (child.is_named("RealTime") || child.is_named("GameTime"))
                    && child.text().ok().and_then(Self::parse_time).is_some()
            }) {
                finished_attempts += 1;
            }
        }

        finished_attempts
    }
}

pub struct Segment {
    pub name: String,
    pub icon: String,
    pub split_time: Time,
    pub best_time: Time,
    pub best_segment: Time,
}

#[cfg(test)]
#[path = "../tests/unit/time.rs"]
mod time_tests;
