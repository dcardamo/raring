use crate::{
    screens,
    store::Store,
    view::{Settings, SleepView, TodayView, Tone, VitalKind, VitalRangeSource},
};
use chrono::{DateTime, Datelike, Local, NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub kind: String,
    pub title: String,
    pub body: String,
    pub tag: String,
}
fn enabled(s: &Settings, kind: &str) -> bool {
    match kind {
        "morning" => s.notify_morning_readiness,
        "weekly" => s.notify_weekly_checkin,
        "vo2max" | "fitness-age" => s.notify_metric_changes,
        "vitals" => s.notify_vitals,
        "training-status" => s.notify_training_status,
        "bedtime" => s.notify_bedtime,
        _ => false,
    }
}
fn duration(min: f64) -> String {
    let min = min.round() as i64;
    format!("{}h {:02}m", min / 60, min % 60)
}
/// Collapsed notifications show only the first line, so it carries the night.
fn morning_body(view: &TodayView, sleep: &SleepView) -> String {
    let mut night = vec![];
    if let Some(asleep) = sleep.asleep_min {
        night.push(format!("Slept {}", duration(asleep)));
    }
    if let Some(score) = &sleep.score {
        night.push(format!(
            "sleep score {:.0} ({})",
            score.value,
            score.label.to_lowercase()
        ));
    }
    if let (Some(asleep), Some(need)) = (sleep.asleep_min, sleep.need_min) {
        let gap = (asleep - need).round();
        night.push(match gap {
            g if g >= 0. => format!("{g:.0} min over need"),
            g => format!("{:.0} min short of need", -g),
        });
    }
    let mut recovery = vec![];
    if let Some(hrv) = view.hrv.last_night {
        recovery.push(match &view.hrv.status {
            Some(status) => format!("HRV {hrv:.0} ms ({})", status.to_lowercase()),
            None => format!("HRV {hrv:.0} ms"),
        });
    }
    if let Some(reading) = view
        .vitals
        .readings
        .iter()
        .find(|r| r.kind == VitalKind::RestingHr && r.value.is_some())
    {
        let name = if reading.status.is_some() || reading.range_source == VitalRangeSource::Garmin {
            "Sleep HR"
        } else {
            "resting HR"
        };
        if let Some(hr) = reading.value {
            recovery.push(format!("{name} {hr:.0}"));
        }
    }
    if let Some(battery) = view.body_battery.now {
        recovery.push(format!("Body Battery {battery:.0}"));
    }
    [night.join(" · "), recovery.join(" · ")]
        .into_iter()
        .chain(view.call_note.clone())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
pub fn notifications(store: &Store, now: DateTime<Local>) -> Vec<Notification> {
    let day = now.date_naive().to_string();
    let mut result = vec![];
    let settings = store.settings();
    let mut add = |kind: &str, title: &str, body: String, tag: String| {
        if enabled(&settings, kind) && !store.notified(kind, &tag) {
            result.push(Notification {
                kind: kind.into(),
                title: title.into(),
                body,
                tag,
            });
        }
    };
    let mut view = screens::today(store, &day);
    // Rest, illness, a pause or limits leave a factual update without a training call.
    let local = crate::local::load(store);
    let (state, _) = crate::lead::hold_back(&local, &mut view, now.date_naive());
    let held = crate::lead::circumstance_line(&state);
    if let (Some(_), Some(readiness)) = (&view.sleep, &view.readiness) {
        add(
            "morning",
            &match view.call {
                Some(_) => format!(
                    "{} · readiness {:.0} ({})",
                    view.headline,
                    readiness.value,
                    readiness.label.to_lowercase()
                ),
                None => format!("Readiness {:.0}", readiness.value),
            },
            {
                let body = morning_body(&view, &screens::sleep_view(store, &day));
                match held {
                    Some(line) if body.is_empty() => format!("{line} · no training call"),
                    Some(line) => format!("{body}\n{line} · no training call"),
                    None => body,
                }
            },
            day.clone(),
        );
    }
    if now.weekday() == chrono::Weekday::Mon
        && now.hour() >= 8
        && store.get("summary", &day).is_some()
    {
        add(
            "weekly",
            "Your weekly check-in",
            "See last week's wins and what to focus on.".into(),
            day.clone(),
        );
    }
    if view.vitals.off_count >= 2 {
        let mut outliers = view.vitals.readings.iter().filter(|r| r.tone == Tone::Poor);
        let garmin = outliers.clone().any(|r| {
            r.status.is_some() || r.range_source == VitalRangeSource::Garmin
        });
        let raring = outliers.any(|r| r.range_source == VitalRangeSource::Raring);
        let source = match (garmin, raring) {
            (true, true) => "Garmin's baselines or Raring's usual ranges",
            (true, false) => "Garmin's baselines",
            (false, true) => "Raring's usual ranges",
            (false, false) => "personal ranges",
        };
        add(
            "vitals",
            "Vitals outside range",
            format!("Several readings are outside {source}. Take it easy and check how you feel."),
            day.clone(),
        );
    }
    if let Some(v) = view.fitness_age.fitness_age {
        if let Some(previous) = store
            .history("fitness_age", &day, 90)
            .into_iter()
            .find_map(|(d, v)| (d != day).then(|| v["fitnessAge"].as_f64()).flatten())
        {
            if v.round() != previous.round() {
                add(
                    "fitness-age",
                    "Fitness Age changed",
                    format!("Now {v:.0} years."),
                    format!("{day}:{v:.2}"),
                );
            }
        }
    }
    if let Some(v) = store
        .get("max_metrics", &day)
        .and_then(|v| screens::vo2(&v))
    {
        if let Some(previous) = store
            .history("max_metrics", &day, 90)
            .into_iter()
            .find_map(|(d, v)| (d != day).then(|| screens::vo2(&v)).flatten())
        {
            if v.round() != previous.round() {
                add(
                    "vo2max",
                    "VO2max changed",
                    format!("Now {v:.1}."),
                    format!("{day}:{v:.2}"),
                );
            }
        }
    }
    let statuses = store.history("training", &day, 2);
    if statuses.len() == 2 {
        let current = screens::status(&statuses[0].1);
        let before = screens::status(&statuses[1].1);
        if current.is_some() && current != before {
            add(
                "training-status",
                "Training status changed",
                format!("Now {}.", current.unwrap_or_default()),
                day.clone(),
            );
        }
    }
    if let Some(reminder) = bedtime(store, now.naive_local()) {
        if reminder.at <= now.naive_local() {
            add("bedtime", &reminder.title, reminder.body, reminder.night);
        }
    }
    result
}
/// Tonight's wind-down reminder from the shared sleep plan, in local
/// wall-clock time. The platform maps `at` to its own clock, so timezone and
/// daylight-saving changes only need a reschedule.
#[derive(Debug, Clone, PartialEq)]
pub struct BedtimeReminder {
    pub at: NaiveDateTime,
    /// The reminder is pointless once the asleep-by time has passed.
    pub until: NaiveDateTime,
    /// One reminder per sleep episode, keyed by the morning it ends.
    pub night: String,
    pub title: String,
    pub body: String,
}
/// The pending reminder for the current plan, or None when reminders are off,
/// the plan is stale, its night was already reminded or its time has passed.
pub fn bedtime(store: &Store, now: NaiveDateTime) -> Option<BedtimeReminder> {
    if !store.settings().notify_bedtime {
        return None;
    }
    let date = screens::current_date(store, now).to_string();
    let plan = screens::sleep_plan(store, &date)?;
    let parse = |s: &str| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok();
    let at = parse(&plan.wind_down_at)?;
    let until = parse(&plan.bedtime_at)?;
    if plan.stale || now >= until || store.notified("bedtime", &plan.night) {
        return None;
    }
    let need = duration(plan.need_min);
    let body = match &plan.bedtime_start {
        Some(start) => format!(
            "In bed by {} · Garmin's bedtime window: {start}–{} · need {need}",
            plan.in_bed, plan.bedtime
        ),
        None => format!(
            "In bed by {} · aim to be asleep by {} (Raring's estimate) · need {need}",
            plan.in_bed, plan.bedtime
        ),
    };
    Some(BedtimeReminder {
        at,
        until,
        night: plan.night,
        title: "Time to wind down".into(),
        body,
    })
}
/// Record notifications a platform has already shown. Call `deliver` instead
/// when the platform can refuse one.
pub fn mark_sent(
    store: &Store,
    items: &[Notification],
    now: DateTime<Local>,
) -> rusqlite::Result<()> {
    for n in items {
        store.mark_notified(&n.kind, &n.tag, &now.to_rfc3339())?;
    }
    Ok(())
}

/// What a platform's delivery attempt did with the due notifications.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Delivery {
    /// Recorded as sent because the platform accepted them.
    pub shown: Vec<Notification>,
    /// Left unmarked, so the next check offers them again.
    pub failed: Vec<Failed>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Failed {
    pub kind: String,
    pub tag: String,
    pub error: String,
}

/// Show each due notification and record only those the platform accepted.
/// A refused or failed delivery stays unmarked: the next check offers it again
/// rather than losing it.
pub fn deliver(
    store: &Store,
    items: &[Notification],
    now: DateTime<Local>,
    mut show: impl FnMut(&Notification) -> Result<(), String>,
) -> rusqlite::Result<Delivery> {
    let mut delivery = Delivery::default();
    for item in items {
        match show(item) {
            Ok(()) => {
                store.mark_notified(&item.kind, &item.tag, &now.to_rfc3339())?;
                delivery.shown.push(item.clone());
            }
            Err(error) => delivery.failed.push(Failed {
                kind: item.kind.clone(),
                tag: item.tag.clone(),
                error,
            }),
        }
    }
    Ok(delivery)
}
