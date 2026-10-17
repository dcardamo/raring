//! Read-only Garmin gear snapshots. Distances and thresholds arrive in metres;
//! neither the activity list nor an inferred lifespan is a substitute.
use crate::{
    store::Store,
    view::{GearItem, GearView},
};
use serde_json::Value;
use std::collections::HashSet;

fn metres(v: &Value) -> Option<f64> {
    v.as_f64().filter(|n| n.is_finite() && *n >= 0.)
}

pub(crate) fn valid_list(v: &Value) -> bool {
    let Some(items) = v.as_array() else {
        return false;
    };
    let mut ids = HashSet::new();
    items.iter().all(|item| {
        item["uuid"]
            .as_str()
            .is_some_and(|id| !id.is_empty() && ids.insert(id))
            && item["gearType"].as_str().is_some()
            && item["status"].as_str().is_some()
            && ["distanceUsedMeters", "maxUsageDistanceMeters"]
                .iter()
                .all(|field| {
                    item.get(*field)
                        .is_none_or(|value| value.is_null() || metres(value).is_some())
                })
    })
}

pub(crate) fn view(store: &Store) -> Option<GearView> {
    let list = store.get("gear", "me")?;
    if !valid_list(&list) {
        return None;
    }
    let fetched_at = store.fetched_at("gear", "me")?;
    // Without Garmin's account units, retain the endpoint's actual metre unit.
    let system = store.get("profile_settings", "me").unwrap_or(Value::Null);
    let (unit, divisor) = match system["measurementSystem"].as_str() {
        Some("metric") => ("km", 1000.),
        Some(s) if s.starts_with("statute") => ("mi", 1609.344),
        _ => ("m", 1.),
    };
    let items: Vec<_> = list
        .as_array()?
        .iter()
        .filter_map(|item| {
            let kind = match item["gearType"].as_str()? {
                "SHOES" => "Shoes",
                "BIKE" => "Bike",
                _ => return None,
            };
            let name = item["name"]
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(kind);
            Some(GearItem {
                id: item["uuid"].as_str()?.to_owned(),
                name: name.to_owned(),
                kind: kind.to_owned(),
                status: match item["status"].as_str()? {
                    "ACTIVE" => "Active",
                    "RETIRED" => "Retired",
                    _ => "Status unavailable",
                }
                .into(),
                distance: metres(&item["distanceUsedMeters"]).map(|v| v / divisor),
                threshold: metres(&item["maxUsageDistanceMeters"])
                    .filter(|v| *v > 0.)
                    .map(|v| v / divisor),
            })
        })
        .collect();
    (!items.is_empty()).then_some(GearView {
        items,
        unit: unit.into(),
        fetched_at,
    })
}
