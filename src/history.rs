use std::{
    collections::BTreeMap,
    io::{Cursor, Write},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use typed_path::Utf8UnixPath;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::ComparisonMethod;
use crate::libresplit::Time;
use crate::livesplit::{Attempt as LiveSplitAttempt, HistoryTime, LiveSplitFile, LiveSplitHistory};

#[derive(Serialize)]
struct Attempt {
    start_time: String,
    end_time: String,
    final_time: Time,
    reason: &'static str,
    splits: Vec<AttemptSplit>,
}

#[derive(Serialize)]
struct AttemptSplit {
    title: String,
    time: Option<Time>,
    segment: Option<Time>,
}

const LIVESPLIT_TIMESTAMP_FORMAT: &str = "%m/%d/%Y %H:%M:%S";
const LIBRESPLIT_TIMESTAMP_FORMAT: &str = "%Y-%m-%d_%H-%M-%S";
const LIBRESPLIT_DATE_FORMAT: &str = "%Y-%m-%d";

fn parse_livesplit_timestamp(text: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(text.trim(), LIVESPLIT_TIMESTAMP_FORMAT)
        .ok()
        .map(|date_time| date_time.and_utc())
}

pub fn convert(lss: &LiveSplitHistory, dir_name: &str) -> Result<Vec<u8>, String> {
    let dir = Utf8UnixPath::new(dir_name);
    if dir.file_name() != Some(dir_name) || !dir.is_valid() {
        return Err("Invalid directory name".to_owned());
    }

    let mut histories: BTreeMap<String, Vec<Attempt>> = BTreeMap::new();

    for attempt in &lss.attempts {
        push_attempt(&mut histories, lss, attempt, attempt.time.has_time());
    }

    let mut files = Vec::with_capacity(histories.len());
    for (date, attempts) in histories {
        let data = serde_json::to_vec_pretty(&attempts)
            .map_err(|error| format!("Unable to serialize attempt history: {error}"))?;
        files.push((format!("{dir_name}/{date}.json"), data));
    }

    make_zip(&files)
}

fn push_attempt(
    histories: &mut BTreeMap<String, Vec<Attempt>>,
    lss: &LiveSplitHistory,
    attempt: &LiveSplitAttempt,
    finished: bool,
) {
    let started_utc = attempt
        .started
        .as_deref()
        .and_then(parse_livesplit_timestamp);
    let ended_utc = attempt.ended.as_deref().and_then(parse_livesplit_timestamp);

    let (splits, accumulated) = convert_splits(lss, attempt.id, finished);
    let duration_real_time =
        started_utc
            .as_ref()
            .zip(ended_utc.as_ref())
            .and_then(|(started, ended)| {
                let seconds = ended.signed_duration_since(started).num_seconds();
                if seconds < 0 {
                    return None;
                }

                i128::from(seconds)
                    .checked_mul(1_000_000_000)?
                    .checked_sub(attempt.pause_time.unwrap_or(0))?
                    .checked_add(lss.offset)
            });

    let date = started_utc
        .as_ref()
        .or(ended_utc.as_ref())
        .map(|date_time| date_time.format(LIBRESPLIT_DATE_FORMAT).to_string())
        .unwrap_or_else(|| "undated".to_owned());

    let derived_real_time = match (duration_real_time, accumulated.real_time) {
        (Some(duration), Some(split)) => Some(duration.max(split)),
        (duration, split) => duration.or(split),
    };

    let final_time = HistoryTime {
        real_time: attempt.time.real_time.or(derived_real_time),
        game_time: attempt.time.game_time.or(accumulated.game_time),
    };

    histories.entry(date).or_default().push(Attempt {
        start_time: started_utc
            .map(|date_time| date_time.format(LIBRESPLIT_TIMESTAMP_FORMAT).to_string())
            .unwrap_or_default(),
        end_time: ended_utc
            .map(|date_time| date_time.format(LIBRESPLIT_TIMESTAMP_FORMAT).to_string())
            .unwrap_or_default(),
        final_time: convert_time_format(final_time),
        reason: if finished { "FINISHED" } else { "RESET" },
        splits,
    });
}

fn convert_splits(
    lss: &LiveSplitHistory,
    attempt_id: i32,
    finished: bool,
) -> (Vec<AttemptSplit>, HistoryTime) {
    let recorded_count = lss
        .segments
        .iter()
        .rposition(|segment| segment.history.contains_key(&attempt_id))
        .map_or(0, |index| index + 1);
    let reached_count = if recorded_count > 0 {
        recorded_count
    } else if finished {
        lss.segments.len()
    } else {
        0
    };

    let mut total = HistoryTime {
        real_time: Some(0),
        game_time: Some(0),
    };
    let mut seen = [false, false];
    let mut last_recorded = [true, true];
    let mut result = Vec::with_capacity(reached_count);

    for (index, segment) in lss.segments.iter().take(reached_count).enumerate() {
        let history = match segment.history.get(&attempt_id) {
            Some(history) => *history,
            None => {
                // unknown time
                total = HistoryTime::default();
                HistoryTime::default()
            }
        };

        let mut split_time = HistoryTime::default();
        let mut segment_time = HistoryTime::default();

        for method in [ComparisonMethod::RealTime, ComparisonMethod::GameTime] {
            let method_index = method as usize;
            let value = time_for_method(&history, method);
            let running_total = time_for_method_mut(&mut total, method);

            if let Some(value) = value {
                seen[method_index] = true;
                *running_total = running_total.and_then(|total| total.checked_add(value));
                *time_for_method_mut(&mut split_time, method) = *running_total;

                // keep running total for skipped segments
                if index == 0 || last_recorded[method_index] {
                    *time_for_method_mut(&mut segment_time, method) = Some(value);
                }
            }

            last_recorded[method_index] = value.is_some();
        }

        result.push(AttemptSplit {
            title: segment.name.clone(),
            time: split_time
                .has_time()
                .then(|| convert_time_format(split_time)),
            segment: segment_time
                .has_time()
                .then(|| convert_time_format(segment_time)),
        });
    }

    (
        result,
        HistoryTime {
            real_time: seen[ComparisonMethod::RealTime as usize]
                .then_some(total.real_time)
                .flatten(),
            game_time: seen[ComparisonMethod::GameTime as usize]
                .then_some(total.game_time)
                .flatten(),
        },
    )
}

fn time_for_method(time: &HistoryTime, method: ComparisonMethod) -> Option<i128> {
    match method {
        ComparisonMethod::RealTime => time.real_time,
        ComparisonMethod::GameTime => time.game_time,
    }
}

fn time_for_method_mut(time: &mut HistoryTime, method: ComparisonMethod) -> &mut Option<i128> {
    match method {
        ComparisonMethod::RealTime => &mut time.real_time,
        ComparisonMethod::GameTime => &mut time.game_time,
    }
}

fn convert_time_format(time: HistoryTime) -> Time {
    Time {
        real_time: time
            .real_time
            .map(LiveSplitFile::format_time)
            .unwrap_or_else(|| "-".to_owned()),
        game_time: time
            .game_time
            .map(LiveSplitFile::format_time)
            .unwrap_or_else(|| "-".to_owned()),
    }
}

fn make_zip(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, data) in files {
        archive
            .start_file(name, options)
            .map_err(|error| format!("Unable to add history file to ZIP: {error}"))?;
        archive
            .write_all(data)
            .map_err(|error| format!("Unable to write history file to ZIP: {error}"))?;
    }

    archive
        .finish()
        .map(Cursor::into_inner)
        .map_err(|error| format!("Unable to finish attempt-history ZIP: {error}"))
}
