//! Sleep's Last night / Tonight: recent nights against need, the closer tab and
//! tonight's self-reported wind-down.
use chrono::{Local, TimeZone};
use raring_core::{store::Store, view::*, Raring};
use serde_json::json;

fn at(d: u32, h: u32, m: u32) -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2025, 3, d, h, m, 0).unwrap()
}

#[test]
fn recent_nights_tonight_first_and_wind_down() {
    let dir = std::env::temp_dir().join(format!("raring-sleeptabs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = Store::open(dir.join("raring.db")).unwrap();
    // Fourteen nights 23:00–07:00 except a gap on the 10th; need 8 h, 7 h 30 asleep.
    for d in 1..=13u32 {
        if d == 10 {
            continue;
        }
        store.put("sleep", &format!("2025-03-{d:02}"), "now", &json!({"dailySleepDTO": {
            "sleepStartTimestampLocal": format!("2025-03-{:02}T23:00:00.0", d - 1), "sleepEndTimestampLocal": format!("2025-03-{d:02}T07:00:00.0"),
            "sleepTimeSeconds": 27000, "sleepNeed": {"actual": 480}}})).unwrap();
    }
    let core = Raring::open(dir.join("raring.db")).unwrap();
    let morning = core.sleep_live(at(13, 9, 0));
    assert_eq!(morning.recent.len(), 7);
    assert_eq!(morning.recent.first().unwrap().date, "2025-03-07");
    assert_eq!(morning.recent.last().unwrap().date, "2025-03-13");
    let gap = morning.recent.iter().find(|n| n.date == "2025-03-10").unwrap();
    assert_eq!((gap.asleep_min, gap.need_min), (None, None), "a missing night stays missing");
    let last = morning.recent.last().unwrap();
    assert_eq!((last.asleep_min, last.need_min), (Some(450.), Some(480.)));
    assert!(!morning.tonight_first, "after waking, Last night opens");
    let phase = |h, m| core.today_at("2025-03-13", at(13, h, m)).focus.phase;
    let late = core.sleep_live(at(13, 23, 30));
    assert_eq!(late.tonight_first, matches!(phase(23, 30), DayPhase::Evening | DayPhase::WindDown | DayPhase::LateNight), "Today's phase decides");
    assert!(late.tonight_first, "late evening opens Tonight ({:?})", phase(23, 30));
    assert!(!late.wound_down);
    // In the morning Tonight shows the coming night, which can't be reported
    // yet; tapping then mustn't record last night beside tonight's plan.
    let morning = core.sleep_live(at(13, 9, 0));
    assert_eq!(morning.sleep_plan.as_ref().unwrap().night, "2025-03-14");
    assert!(!morning.wind_down_open && !morning.wound_down);
    assert!(core.local_action(&LocalAction::ReportWindDown, at(13, 9, 0)).is_err());
    assert!(core.local_records().circumstances.wind_downs.is_empty());
    assert!(late.wind_down_open, "open from two hours before tonight's window");
    core.local_action(&LocalAction::ReportWindDown, at(13, 22, 40)).unwrap();
    assert_eq!(core.local_records().circumstances.wind_downs[0].date, "2025-03-13", "the evening before the plan's night");
    assert!(core.sleep_live(at(13, 23, 30)).wound_down);
    assert!(core.sleep_live(at(14, 1, 0)).wound_down, "after midnight the report still belongs to last evening");
    assert!(!core.sleep_live(at(14, 21, 0)).wound_down, "the next evening starts unreported");
    assert!(core.sleep("2025-03-13").recent.len() == 7 && !core.sleep("2025-03-13").tonight_first, "historical reads never open Tonight");
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn tonights_need_says_when_raring_read_it() {
    let dir = std::env::temp_dir().join(format!("raring-needread-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = Store::open(dir.join("raring.db")).unwrap();
    let night = |next: serde_json::Value| json!({"dailySleepDTO": {"sleepStartTimestampLocal": "2025-03-12T23:00:00.0",
        "sleepEndTimestampLocal": "2025-03-13T07:00:00.0", "sleepTimeSeconds": 27000, "sleepNeed": {"actual": 480}, "nextSleepNeed": next}});
    store.put("sleep", "2025-03-13", "2025-03-13T13:58:00+00:00", &night(json!({"actual": 490, "calendarDate": "2025-03-14"}))).unwrap();
    let core = Raring::open(dir.join("raring.db")).unwrap();
    let plan = core.sleep("2025-03-13").sleep_plan.unwrap();
    assert_eq!((plan.need_min, plan.need_read_at.as_deref()), (490., Some("2025-03-13T13:58:00+00:00")));
    store.put("sleep", "2025-03-13", "2025-03-13T13:58:00+00:00", &night(json!(null))).unwrap();
    assert_eq!(core.sleep("2025-03-13").sleep_plan.unwrap().need_read_at, None, "only tonight's need has a read time");
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}
