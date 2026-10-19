//! Independently synthetic browser fixtures, projected by the real weight core.
//! Regenerate: nix develop -c cargo run -q -p raring-core --example weight_fixtures > app/src/lib/sample/weight.ts
use chrono::{Duration, NaiveDate};
use raring_core::{store::Store, view::TrendRange, weight};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn main() {
    let store = Store::open(":memory:").unwrap();
    let end = NaiveDate::from_ymd_opt(2025, 10, 16).unwrap();
    for i in 0..620 {
        if matches!(i % 31, 5..=9) || i % 7 == 2 {
            continue;
        }
        let date = end - Duration::days(619 - i);
        let kg = 72.4 + 0.6 * (i as f64 / 19.).sin() + 0.2 * (i as f64 / 2.).cos();
        store
            .put(
                "weight",
                &date.to_string(),
                "synthetic",
                &json!({
                    "weight": kg * 1000., "bmi": 23.8, "bodyFat": 21.0,
                    "timestampGMT": date.and_hms_opt(7, 0, 0).unwrap().and_utc().timestamp_millis()
                }),
            )
            .unwrap();
    }
    let mut all = BTreeMap::<String, BTreeMap<String, Value>>::new();
    for (unit, system) in [("kg", "metric"), ("lb", "statute_us")] {
        store
            .put(
                "profile_settings",
                "me",
                "synthetic",
                &json!({"measurementSystem": system}),
            )
            .unwrap();
        let mut cases = BTreeMap::new();
        for range in [
            TrendRange::Days7,
            TrendRange::Days14,
            TrendRange::Days30,
            TrendRange::Days90,
            TrendRange::Year,
            TrendRange::All,
        ] {
            let key = serde_json::to_value(range)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            cases.insert(
                key,
                serde_json::to_value(weight::view(&store, range, end, end).unwrap()).unwrap(),
            );
        }
        all.insert(unit.to_string(), cases);
    }
    println!("// Generated synthetic projections; regenerate with crates/core/examples/weight_fixtures.rs.");
    println!("import type {{WeightView}} from '../generated/WeightView';");
    println!(
        "export const weightFixtures = {} satisfies Record<string,Record<string,WeightView>>;",
        serde_json::to_string(&all).unwrap()
    );
}
