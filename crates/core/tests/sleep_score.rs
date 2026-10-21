//! The Sleep Score's word on Today and Sleep is Garmin's when sent
//! (docs/spec.md, Today and Sleep). All values are synthetic.
use raring_core::{screens, store::Store, view::*};
use serde_json::json;

const DATE: &str = "2025-03-12";

#[test]
fn the_score_word_is_garmins_when_sent_and_the_bands_otherwise() {
    for (score, qualifier, word, tone) in [
        (92., Some("EXCELLENT"), "Excellent", Tone::Good),
        (87., Some("GOOD"), "Good", Tone::Good),
        (92., None, "Good", Tone::Good),
        (64., None, "Fair", Tone::Fair),
        (55., Some("POOR"), "Poor", Tone::Poor),
    ] {
        let store = Store::open(":memory:").unwrap();
        let mut overall = json!({"value": score});
        if let Some(q) = qualifier {
            overall["qualifierKey"] = json!(q);
        }
        store.put("sleep", DATE, "synthetic", &json!({"dailySleepDTO": {"sleepTimeSeconds": 27000, "sleepScores": {"overall": overall}}})).unwrap();
        for gauge in [screens::today(&store, DATE).sleep, screens::sleep_view(&store, DATE).score] {
            let g = gauge.expect("a scored night");
            assert_eq!((g.value, g.label.as_str(), g.tone), (score, word, tone), "{score} {qualifier:?}");
        }
    }
}
