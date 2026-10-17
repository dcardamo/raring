use chrono::{DateTime, FixedOffset, Utc};
use raring_core::{screens, store::Store, view::PlannedState};
use serde_json::json;

#[test]
fn live_calendar_advances_at_midnight_while_sleep_day_stays_yesterday() {
    let store = Store::open(":memory:").unwrap();
    let now: DateTime<FixedOffset> = "2026-10-21T00:15:00-04:00".parse().unwrap();
    let utc = now.with_timezone(&Utc);
    store.put("sleep", "2026-10-20", &utc.to_rfc3339(), &json!({"dailySleepDTO":{"sleepTimeSeconds":27000,"sleepEndTimestampLocal":"2026-10-20T06:30:00"}})).unwrap();
    assert_eq!(screens::current_date(&store, now.naive_local()).to_string(), "2026-10-20");
    store.put("scheduled_workouts", "current", &utc.to_rfc3339(), &json!({
        "startDate":"2026-10-20", "endDate":"2026-11-03",
        "entries":[{"calendarDate":"2026-10-20","workoutName":"Yesterday Run"},
                   {"calendarDate":"2026-10-21","workoutName":"Today Ride"}]
    })).unwrap();
    store.put("upcoming_events", "current", &utc.to_rfc3339(), &json!([
        {"eventType":"RACE","eventName":"Town 5K","calendarDate":"2026-10-21","startTimeGMT":"2026-10-20T22:00:00Z"}
    ])).unwrap();

    let display = "2026-10-20";
    let live_today = screens::today_at_calendar(&store, display, now, now.date_naive());
    assert_eq!(live_today.date, display);
    assert_eq!(live_today.planned.session.unwrap().name, "Today Ride");
    let live_training = screens::training_at_calendar(&store, display, now.date_naive(), utc);
    assert_eq!(live_training.planned.session.unwrap().name, "Today Ride");
    assert_eq!(live_training.race.unwrap().days_until, 0);

    // Explicit historical views retain their requested calendar date.
    assert_eq!(screens::today_at(&store, display, now).planned.session.unwrap().name, "Yesterday Run");
    let historical = screens::training_at_calendar(&store, display, display.parse().unwrap(), utc);
    assert_eq!(historical.race.unwrap().days_until, 1);
    assert_eq!(historical.planned.session.unwrap().name, "Yesterday Run");

    store.put("scheduled_workouts", "current", &utc.to_rfc3339(), &json!({"startDate":"2026-10-20","endDate":"2026-11-03","entries":[{"calendarDate":"2026-10-20","workoutName":"Yesterday Run"}]})).unwrap();
    let no_next = screens::training_at_calendar(&store, display, now.date_naive(), utc);
    assert_eq!(no_next.planned.state, PlannedState::Empty);
    assert!(no_next.planned.session.is_none());
    assert_eq!(no_next.planned.coverage_start.as_deref(), Some("2026-10-20"));
    assert_eq!(no_next.planned.coverage_end.as_deref(), Some("2026-11-03"));
}
