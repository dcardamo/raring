//! An event groups this week's sessions of its sport; Garmin's race, whose
//! sport is unverified, never does.
use chrono::{Local, TimeZone};
use raring_core::{store::Store, view::*, Raring};
use serde_json::json;

fn at(d: u32, h: u32) -> chrono::DateTime<Local> {
    Local.with_ymd_and_hms(2025, 3, d, h, 0, 0).unwrap()
}

#[test]
fn an_event_groups_its_sport_and_counts_recorded_minutes() {
    let dir = std::env::temp_dir().join(format!("raring-eventweek-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store = Store::open(dir.join("raring.db")).unwrap();
    store.put("activity", "5", "now", &json!({"activityId": 5, "activityName": "Easy Run", "activityTypeDTO": {"typeKey": "running"},
        "summaryDTO": {"startTimeLocal": "2025-03-11 07:00:00", "duration": 2400}})).unwrap();
    store.put("activity", "6", "now", &json!({"activityId": 6, "activityName": "Ride", "activityTypeDTO": {"typeKey": "cycling"},
        "summaryDTO": {"startTimeLocal": "2025-03-12 07:00:00", "duration": 3600}})).unwrap();
    store.set_cursor("listed:lo", "2025-03-01").unwrap();
    store.set_cursor("listed:hi", "2025-03-13").unwrap();
    let core = Raring::open(dir.join("raring.db")).unwrap();
    let now = at(13, 9);
    let add = |date: &str, sport| core.local_action(&LocalAction::AddIntention { date: date.into(), time: None, sport, type_key: None, name: None,
        duration_min: Some(30), allocation: Allocation::default(), kind: IntentionKind::Workout, garmin_id: None }, now).unwrap();
    let board = || core.plan_board("2025-03-13", now).unwrap();
    add("2025-03-14", Sport::Running);
    assert!(board().event_week.is_none(), "no event, no grouping");
    // Garmin's race has no verified sport.
    core.local_action(&LocalAction::AddEvent { name: "Spring 10K".into(), date: "2025-04-06".into(), sport: None }, now).unwrap();
    assert!(board().event_week.is_none(), "an event without a sport only counts down");
    core.local_action(&LocalAction::AddEvent { name: "Hill 10K".into(), date: "2025-04-20".into(), sport: Some(Sport::Running) }, now).unwrap();
    add("2025-03-15", Sport::Cycling);
    let w = board().event_week.unwrap();
    assert_eq!((w.event.name.as_str(), w.sport, w.items.len(), w.recorded_min), ("Hill 10K", Sport::Running, 1, 40.), "running sessions and minutes only");
    let run = w.items[0].clone();
    core.local_action(&LocalAction::SetIntentionStatus { id: run, status: IntentionStatus::LetGo }, now).unwrap();
    assert!(board().event_week.is_none(), "a let-go session no longer serves it");
    assert!(core.plan_board("2025-03-03", now).unwrap().event_week.is_none(), "only the current week");
    drop(core);
    let _ = std::fs::remove_dir_all(dir);
}
