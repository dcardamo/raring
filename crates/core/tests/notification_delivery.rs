//! Delivery records only what a platform actually showed. All values are
//! synthetic.
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime};
use raring_core::{notifications, store::Store, Notification};
use serde_json::json;

const DATE: &str = "2025-03-12";

fn day(offset: i64) -> String {
    (NaiveDate::parse_from_str(DATE, "%Y-%m-%d").unwrap()
        + chrono::Duration::days(offset))
    .to_string()
}
fn at(date: &str, time: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(&format!("{date}T{time}:00"), "%Y-%m-%dT%H:%M:%S").unwrap()
}
fn local(t: NaiveDateTime) -> DateTime<Local> {
    t.and_local_timezone(Local).earliest().unwrap()
}
/// A night that ends on `date` with tonight's plan due at 21:45.
fn store_with_tonight() -> (Store, String) {
    let store = Store::open(":memory:").unwrap();
    let night = day(1);
    store
        .put(
            "sleep",
            DATE,
            "now",
            &json!({"dailySleepDTO": {
                "calendarDate": DATE,
                "sleepTimeSeconds": 25000,
                "sleepStartTimestampLocal": format!("{DATE}T23:00:00"),
                "sleepEndTimestampLocal": format!("{DATE}T07:00:00"),
                "sleepNeed": {"actual": 450, "baseline": 450},
                "nextSleepNeed": {
                    "calendarDate": night, "actual": 450, "baseline": 450,
                    "recommendedBedtimeStartMins": 1350, "recommendedBedtimeEndMins": 1410,
                },
            }}),
        )
        .unwrap();
    let mut settings = store.settings();
    settings.notify_bedtime = true;
    settings.bedtime_lead_min = 45;
    store.set_settings(&settings).unwrap();
    (store, night)
}

fn item(kind: &str, tag: &str) -> Notification {
    Notification {
        kind: kind.into(),
        title: format!("{kind} title"),
        body: format!("{kind} body"),
        tag: tag.into(),
    }
}

#[test]
fn a_failed_delivery_keeps_the_notification_unmarked_and_due() {
    let (store, night) = store_with_tonight();
    let due: Vec<_> = notifications::notifications(&store, local(at(DATE, "22:00")))
        .into_iter()
        .filter(|n| n.kind == "bedtime")
        .collect();
    assert_eq!(due.len(), 1);

    let delivery =
        notifications::deliver(&store, &due, local(at(DATE, "22:00")), |_| Err("blocked".into()))
            .unwrap();
    assert!(delivery.shown.is_empty());
    assert_eq!(delivery.failed[0].kind, "bedtime");
    assert_eq!(delivery.failed[0].tag, night);
    assert_eq!(delivery.failed[0].error, "blocked");
    assert!(!store.notified("bedtime", &night), "a refused reminder is not sent");
    assert!(
        notifications::notifications(&store, local(at(DATE, "22:05")))
            .iter()
            .any(|n| n.kind == "bedtime"),
        "the next check offers it again"
    );

    let delivery =
        notifications::deliver(&store, &due, local(at(DATE, "22:06")), |_| Ok(())).unwrap();
    assert_eq!(delivery.shown.len(), 1);
    assert!(delivery.failed.is_empty());
    assert!(store.notified("bedtime", &night));
    assert!(
        !notifications::notifications(&store, local(at(DATE, "22:07")))
            .iter()
            .any(|n| n.kind == "bedtime"),
        "a shown reminder is not offered twice"
    );
}

#[test]
fn each_notification_is_marked_on_its_own_delivery() {
    let store = Store::open(":memory:").unwrap();
    let due = [item("morning", "2025-03-12"), item("weekly", "2025-03-12")];
    let delivery = notifications::deliver(
        &store,
        &due,
        local(at(DATE, "09:00")),
        |n| match n.kind.as_str() {
            "weekly" => Err("no notification service".into()),
            _ => Ok(()),
        },
    )
    .unwrap();
    assert_eq!(
        delivery.shown.iter().map(|n| n.kind.as_str()).collect::<Vec<_>>(),
        ["morning"]
    );
    assert_eq!(
        delivery.failed.iter().map(|n| n.kind.as_str()).collect::<Vec<_>>(),
        ["weekly"]
    );
    assert!(store.notified("morning", "2025-03-12"));
    assert!(!store.notified("weekly", "2025-03-12"));
}
