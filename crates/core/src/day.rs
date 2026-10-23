//! Your day (docs/spec.md, Rules: Your day): Garmin's recorded sleep, Body
//! Battery, stress, heart rate, activities and naps for the local calendar day
//! on one axis. Read from local data only; nothing is drawn after the upload.
use crate::{naps_journal, screens, store::Store, view::*};
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone};

/// Elapsed minutes from this local midnight to the next: 1380 or 1500 on a
/// clock-change day.
pub fn day_minutes(day: NaiveDate) -> f64 {
    let at = |d: NaiveDate| Local.from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap()).earliest();
    match (at(day), day.succ_opt().and_then(at)) {
        (Some(a), Some(b)) => (b - a).num_seconds() as f64 / 60.,
        _ => 1440.,
    }
}

fn minute_of(local: &str, midnight: NaiveDateTime) -> Option<f64> {
    let text = local.get(..19)?.replace(' ', "T");
    let t = NaiveDateTime::parse_from_str(&text, "%Y-%m-%dT%H:%M:%S").ok()?;
    Some((t - midnight).num_seconds() as f64 / 60.)
}

/// Garmin's level after a nap minus before it: the last measured slot at or
/// before the start and the first at or after the end, within the day.
pub fn nap_change(slots: &[Option<f64>], start_min: f64, end_min: f64) -> Option<f64> {
    if !(start_min >= 0. && end_min > start_min) {
        return None;
    }
    let before = slots.iter().enumerate().take((start_min / 3.).floor() as usize + 1).rev().find_map(|(_, v)| *v)?;
    let after = slots.iter().skip((end_min / 3.).ceil() as usize).find_map(|v| *v)?;
    Some(after - before)
}

/// Slots past `limit_min` are blanked: nothing is drawn after the upload or now.
fn until(mut slots: Vec<Option<f64>>, step: f64, limit_min: f64) -> Vec<Option<f64>> {
    slots.iter_mut().enumerate().filter(|(i, _)| *i as f64 * step > limit_min).for_each(|(_, v)| *v = None);
    while slots.last().is_some_and(Option::is_none) {
        slots.pop();
    }
    slots
}

pub fn timeline(store: &Store, view: &TodayView, today: NaiveDate, now: DateTime<FixedOffset>, day_minutes: f64) -> DayTimeline {
    let date = today.to_string();
    let midnight = today.and_hms_opt(0, 0, 0).unwrap();
    let now_min = (now.naive_local() - midnight).num_seconds() as f64 / 60.;
    let upload = view
        .watch_last_upload
        .as_deref()
        .and_then(|u| DateTime::parse_from_rfc3339(u).ok())
        .map(|u| (u.with_timezone(now.offset()).naive_local() - midnight).num_seconds() as f64 / 60.);
    let upload_min = upload.filter(|m| *m >= 0.).map(|m| m.min(1440.));
    // An upload before midnight draws nothing; without an upload time the
    // phone's clock is the only limit.
    let limit = upload.map_or(now_min, |m| m.max(0.)).min(now_min);
    // Garmin's intraday slots count elapsed minutes from midnight; on a
    // clock-change day they don't line up with the wall-clock axis.
    let clock_change = (day_minutes - 1440.).abs() >= 1.;
    let clip = |a: f64, b: f64| (b > 0. && a < limit).then_some((a.max(0.), b.min(limit)));
    // Garmin files a night under the morning it ends.
    let night = screens::sleep_row(store, &date).unwrap_or(serde_json::Value::Null);
    let start = screens::local_time(screens::sleep(&night), &["sleepStartTimestampLocal"]).and_then(|s| minute_of(&s, midnight));
    let sleep = start
        .map(|start| {
            screens::stages(&night)
                .iter()
                .filter_map(|s| {
                    let (a, b) = clip(start + s.start_min, start + s.start_min + s.minutes)?;
                    Some(StageSpan { stage: s.stage, start_min: a, end_min: b })
                })
                .collect()
        })
        .unwrap_or_default();
    let stress = screens::stress(store, &date);
    let raw = store.get("stress", &date);
    let body_battery = raw.as_ref().map(screens::battery_timeline).unwrap_or_default();
    let measured = raw.as_ref().map(|v| screens::battery_slots(v, true)).unwrap_or_default();
    let hr = screens::heart_rate_day(store, &date);
    let mut events: Vec<DayEvent> = store
        .activities_between(&date, &date)
        .iter()
        .filter_map(|(id, start)| {
            let card = screens::activity(store, id)?;
            let start_min = minute_of(start, midnight);
            Some(DayEvent {
                kind: DayEventKind::Activity,
                id: Some(id.clone()),
                name: card.name,
                icon: card.icon,
                end_min: start_min.map(|s| (s + card.duration_s / 60.).min(1440.)),
                start_min,
                duration_min: Some(card.duration_s / 60.),
                body_battery_change: None,
                feedback: None,
            })
        })
        .collect();
    let (naps, _) = naps_journal::naps(store, &date);
    for nap in naps {
        let start_min = nap.start_local.as_deref().and_then(|s| minute_of(s, midnight)).filter(|m| (0. ..1440.).contains(m));
        let end_min = nap.end_local.as_deref().and_then(|s| minute_of(s, midnight)).filter(|m| *m > 0.);
        // A nap that began before midnight belongs to the day it started.
        if start_min.is_none() && end_min.is_none_or(|e| e > 1440.) {
            continue;
        }
        events.push(DayEvent {
            kind: DayEventKind::Nap,
            id: None,
            name: "Nap".into(),
            icon: Icon::Moon,
            body_battery_change: start_min.zip(end_min).filter(|(_, e)| *e <= limit).and_then(|(s, e)| nap_change(&measured, s, e)),
            start_min,
            end_min: end_min.map(|e| e.min(1440.)),
            duration_min: nap.duration_min,
            feedback: nap.feedback,
        });
    }
    events.sort_by(|a, b| a.start_min.unwrap_or(f64::MAX).total_cmp(&b.start_min.unwrap_or(f64::MAX)));
    let body_battery = if clock_change { vec![] } else { until(body_battery, 3., limit) };
    let body_battery_now = body_battery.iter().rev().find_map(|v| *v).or_else(|| {
        let summary = store.get("summary", &date)?;
        summary["bodyBatteryMostRecentValue"].as_f64()
    });
    DayTimeline {
        date,
        now_min,
        upload_min,
        sleep,
        asleep_min: screens::asleep_min(&night),
        body_battery,
        body_battery_now,
        stress_average: stress.average,
        stress: if clock_change { vec![] } else { until(stress.timeline, 3., limit) },
        heart_rate: if clock_change || (hr.duration_min - 1440.).abs() >= 1. { vec![] } else { until(hr.timeline, 2., limit) },
        clock_change,
        resting_hr: hr.resting,
        events,
    }
}
