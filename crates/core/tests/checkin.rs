//! The weekly check-in: score colours only for Garmin's scores, a win only
//! for a change the cards show, and a focus that names Garmin's metric.
use raring_core::{screens, store::Store, view::*};
use serde_json::json;

const MONDAY: &str = "2025-01-13";
fn day(i: u32) -> String {
    // 1..=7 is the week just ended (6–12 Jan); 8..=14 the week before.
    (chrono::NaiveDate::from_ymd_opt(2025, 1, 13).unwrap() - chrono::Duration::days(i.into())).to_string()
}
fn week(store: &Store, source: &str, this: serde_json::Value, before: serde_json::Value) {
    for i in 1..=14 {
        store.put(source, &day(i), "now", if i <= 7 { &this } else { &before }).unwrap();
    }
}
fn card(view: &CheckinView, area: CheckinArea) -> &CheckinCard {
    view.cards.iter().find(|c| c.area == area).unwrap()
}

#[test]
fn only_readiness_and_sleep_carry_score_colours() {
    let store = Store::open(":memory:").unwrap();
    week(&store, "readiness", json!([{"inputContext": "AFTER_WAKEUP_RESET", "score": 30}]), json!([{"inputContext": "AFTER_WAKEUP_RESET", "score": 30}]));
    week(&store, "training", json!({"latestTrainingStatusData": {"1": {"acuteTrainingLoadDTO": {"dailyTrainingLoadAcute": 596}}}}), json!({}));
    week(&store, "fitness_age", json!({"fitnessAge": 39.}), json!({}));
    let view = screens::checkin(&store, MONDAY);
    assert_eq!(card(&view, CheckinArea::Readiness).tone, Tone::Poor);
    for area in [CheckinArea::Training, CheckinArea::Health] {
        let c = card(&view, area);
        assert!(c.value.is_some());
        assert_eq!(c.tone, Tone::Neutral, "{area:?} has no Garmin rating to colour");
    }
}

#[test]
fn a_win_needs_a_change_the_cards_show() {
    let store = Store::open(":memory:").unwrap();
    week(&store, "fitness_age", json!({"fitnessAge": 39.}), json!({"fitnessAge": 39.4}));
    week(&store, "readiness", json!([{"inputContext": "AFTER_WAKEUP_RESET", "score": 70}]), json!([{"inputContext": "AFTER_WAKEUP_RESET", "score": 69.5}]));
    assert_eq!(screens::checkin(&store, MONDAY).win, None, "changes under 1 read as steady");
    week(&store, "fitness_age", json!({"fitnessAge": 39.}), json!({"fitnessAge": 40.5}));
    assert_eq!(screens::checkin(&store, MONDAY).win.as_deref(), Some("Fitness Age down 1.5 years on the week before"));
}

#[test]
fn the_focus_names_garmins_metric() {
    let store = Store::open(":memory:").unwrap();
    store.put("load_focus", &day(1), "now", &json!({"lowAerobic": {"value": 100, "optimalMin": 300, "optimalMax": 600}})).unwrap();
    assert_eq!(screens::checkin(&store, MONDAY).focus.as_deref(), Some("Garmin's load focus: low aerobic was furthest below its range"));
}
