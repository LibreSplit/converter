#![cfg(not(target_arch = "wasm32"))]

use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
};

use serde_json::Value;
use zip::ZipArchive;

use super::convert_history_inner;

const LIVE_SPLIT_HISTORY: &str = r#"
<Run version="1.7.0">
  <GameName>Example Game</GameName>
  <CategoryName>Any%</CategoryName>
  <Offset>-00:00:05</Offset>
  <AttemptCount>4</AttemptCount>
  <AttemptHistory>
    <Attempt id="1" started="09/23/2026 10:00:00" ended="09/23/2026 10:00:35">
      <RealTime>00:00:30</RealTime>
      <GameTime>00:00:25</GameTime>
      <PauseTime>00:00:05</PauseTime>
    </Attempt>
    <Attempt id="2" started="09/23/2026 11:00:00" ended="09/23/2026 11:00:45">
      <PauseTime>00:00:05</PauseTime>
    </Attempt>
    <Attempt id="3" started="09/24/2026 12:00:00" ended="09/24/2026 12:00:10" />
  </AttemptHistory>
  <Segments>
    <Segment>
      <Name>Opening</Name>
      <SegmentHistory>
        <Time id="1"><RealTime>00:00:10</RealTime><GameTime>00:00:08</GameTime></Time>
        <Time id="2"><RealTime>00:00:12</RealTime><GameTime>00:00:10</GameTime></Time>
      </SegmentHistory>
    </Segment>
    <Segment>
      <Name>Skipped or second</Name>
      <SegmentHistory>
        <Time id="1" />
        <Time id="2"><RealTime>00:00:08</RealTime><GameTime>00:00:07</GameTime></Time>
      </SegmentHistory>
    </Segment>
    <Segment>
      <Name>Finish</Name>
      <SegmentHistory>
        <Time id="1"><RealTime>00:00:20</RealTime><GameTime>00:00:17</GameTime></Time>
      </SegmentHistory>
    </Segment>
  </Segments>
</Run>
"#;

#[test]
fn converts_livesplit_history_to_dated_json_files() {
    let archive = convert_history_inner(LIVE_SPLIT_HISTORY, "Example Game - Any%")
        .expect("history conversion should succeed");
    let files = stored_zip_files(&archive);

    assert_eq!(
        files.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "Example Game - Any%/2026-09-23.json",
            "Example Game - Any%/2026-09-24.json"
        ]
    );

    let first_date: Value =
        serde_json::from_slice(&files["Example Game - Any%/2026-09-23.json"]).unwrap();
    let attempts = first_date.as_array().unwrap();
    assert_eq!(attempts.len(), 2);

    let finished = &attempts[0];
    assert_eq!(finished["start_time"], "2026-09-23_10-00-00");
    assert_eq!(finished["end_time"], "2026-09-23_10-00-35");
    assert_eq!(finished["reason"], "FINISHED");
    assert_eq!(finished["final_time"]["real_time"], "00:00:30.000000");
    assert_eq!(finished["final_time"]["game_time"], "00:00:25.000000");
    assert_eq!(
        finished["splits"][0]["time"]["real_time"],
        "00:00:10.000000"
    );
    assert_eq!(
        finished["splits"][0]["segment"]["game_time"],
        "00:00:08.000000"
    );
    assert!(finished["splits"][1]["time"].is_null());
    assert!(finished["splits"][1]["segment"].is_null());
    assert_eq!(
        finished["splits"][2]["time"]["real_time"],
        "00:00:30.000000"
    );
    assert_eq!(
        finished["splits"][2]["time"]["game_time"],
        "00:00:25.000000"
    );
    assert!(finished["splits"][2]["segment"].is_null());

    let reset = &attempts[1];
    assert_eq!(reset["reason"], "RESET");
    assert_eq!(reset["final_time"]["real_time"], "00:00:35.000000");
    assert_eq!(reset["final_time"]["game_time"], "00:00:17.000000");
    assert_eq!(reset["splits"].as_array().unwrap().len(), 2);
    assert_eq!(reset["splits"][1]["time"]["real_time"], "00:00:20.000000");
    assert_eq!(
        reset["splits"][1]["segment"]["real_time"],
        "00:00:08.000000"
    );

    let second_date: Value =
        serde_json::from_slice(&files["Example Game - Any%/2026-09-24.json"]).unwrap();
    let before_first_split = &second_date[0];
    assert_eq!(before_first_split["reason"], "RESET");
    assert_eq!(
        before_first_split["final_time"]["real_time"],
        "00:00:05.000000"
    );
    assert_eq!(before_first_split["final_time"]["game_time"], "-");
    assert_eq!(before_first_split["splits"].as_array().unwrap().len(), 0);
}

#[test]
fn converts_legacy_livesplit_runs() {
    let input = r#"
        <Run version="1.4.0">
          <Offset>00:00:00</Offset>
          <RunHistory>
            <Run id="1">00:00:03.5000000</Run>
          </RunHistory>
          <Segments>
            <Segment>
              <Name>Only Split</Name>
              <SegmentHistory><Time id="1">00:00:03.5000000</Time></SegmentHistory>
            </Segment>
          </Segments>
        </Run>
    "#;

    let archive = convert_history_inner(input, "Legacy Splits").unwrap();
    let files = stored_zip_files(&archive);
    let history: Value = serde_json::from_slice(&files["Legacy Splits/undated.json"]).unwrap();

    assert_eq!(history[0]["reason"], "FINISHED");
    assert_eq!(history[0]["final_time"]["real_time"], "00:00:03.500000");
    assert_eq!(history[0]["final_time"]["game_time"], "-");
    assert_eq!(
        history[0]["splits"][0]["time"]["real_time"],
        "00:00:03.500000"
    );
    assert_eq!(
        history[0]["splits"][0]["segment"]["real_time"],
        "00:00:03.500000"
    );
}

#[test]
fn missing_segment_history_does_not_produce_partial_times() {
    // users can delete individual history entries leaving unknown gaps
    for (attempt_time, expected_real_time, expected_game_time) in [
        (
            "<RealTime>00:01:00</RealTime><GameTime>00:00:45</GameTime>",
            "00:01:00.000000",
            "00:00:45.000000",
        ),
        ("", "-", "-"),
    ] {
        let input = format!(
            r#"
			<Run version="1.7.0">
				<AttemptHistory><Attempt id="1">{attempt_time}</Attempt></AttemptHistory>
				<Segments>
					<Segment>
						<Name>Opening</Name>
						<SegmentHistory>
							<Time id="1"><RealTime>00:00:10</RealTime><GameTime>00:00:08</GameTime></Time>
						</SegmentHistory>
					</Segment>
					<Segment>
						<Name>Deleted</Name>
						<SegmentHistory />
					</Segment>
					<Segment>
						<Name>Finish</Name>
						<SegmentHistory>
							<Time id="1"><RealTime>00:00:30</RealTime><GameTime>00:00:22</GameTime></Time>
						</SegmentHistory>
					</Segment>
				</Segments>
			</Run>
			"#
        );

        let archive = convert_history_inner(&input, "Edited Splits").unwrap();
        let files = stored_zip_files(&archive);
        let history: Value = serde_json::from_slice(&files["Edited Splits/undated.json"]).unwrap();
        let attempt = &history[0];
        let splits = attempt["splits"].as_array().unwrap();

        assert_eq!(splits.len(), 3);
        assert_eq!(splits[0]["time"]["real_time"], "00:00:10.000000");
        assert_eq!(splits[0]["time"]["game_time"], "00:00:08.000000");
        assert!(splits[1]["time"].is_null());
        assert!(splits[2]["time"].is_null());
        assert_eq!(attempt["final_time"]["real_time"], expected_real_time);
        assert_eq!(attempt["final_time"]["game_time"], expected_game_time);
    }
}

#[test]
fn an_empty_history_is_an_empty_zip() {
    let archive =
        convert_history_inner("<Run><Offset>00:00:00</Offset><Segments /></Run>", "Empty").unwrap();
    assert!(stored_zip_files(&archive).is_empty());
}

#[test]
fn invalid_xml_returns_an_error() {
    assert!(convert_history_inner("<Run", "Invalid").is_err());
}

#[test]
fn history_directory_names_are_preserved() {
    for directory_name in [
        "sa2_fallen-hero",
        "ゲーム - Any%",
        "splits.lss",
        r"splits\other",
        "C:splits",
        "Any%: ?*<>|\"",
    ] {
        let archive = convert_history_inner(LIVE_SPLIT_HISTORY, directory_name).unwrap();
        let files = stored_zip_files(&archive);
        assert_eq!(
            files.keys().cloned().collect::<Vec<_>>(),
            [
                format!("{directory_name}/2026-09-23.json"),
                format!("{directory_name}/2026-09-24.json"),
            ]
        );
    }
}

#[test]
fn history_directory_must_be_a_name_not_a_path() {
    for directory_name in [
        "",
        ".",
        "..",
        "/splits",
        "../splits",
        "splits/other",
        "splits\0",
    ] {
        assert!(
            convert_history_inner(LIVE_SPLIT_HISTORY, directory_name).is_err(),
            "{directory_name:?}"
        );
    }
}

fn stored_zip_files(archive: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut archive = ZipArchive::new(Cursor::new(archive)).expect("output should be a valid ZIP");
    let mut files = BTreeMap::new();

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .expect("ZIP entry should be readable");
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .expect("ZIP entry contents should be readable");
        files.insert(file.name().to_owned(), data);
    }

    files
}
