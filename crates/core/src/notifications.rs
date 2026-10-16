use crate::{screens, store::Store, view::Settings};
use chrono::{DateTime, Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    let view = screens::today(store, &day);
    if view.sleep.is_some() && view.readiness.is_some() {
        add(
            "morning",
            "Your day at a glance",
            view.headline.clone(),
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
        add(
            "vitals",
            "Vitals outside range",
            "Possible illness or strain. Take it easy.".into(),
            day.clone(),
        );
    }
    if let Some(v) = view.fitness_age.fitness_age {
        if let Some(previous) = store
            .history("fitness_age", &day, 2)
            .get(1)
            .and_then(|(_, v)| v["fitnessAge"].as_f64())
        {
            if (v - previous).abs() > 0.001 {
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
            .history("max_metrics", &day, 2)
            .get(1)
            .and_then(|(_, v)| screens::vo2(v))
        {
            if (v - previous).abs() > 0.001 {
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
        let current = statuses[0].1["trainingStatus"].as_str();
        let before = statuses[1].1["trainingStatus"].as_str();
        if current.is_some() && current != before {
            add(
                "training-status",
                "Training status changed",
                format!("Now {}.", current.unwrap_or_default()),
                day.clone(),
            );
        }
    }
    if let Some(bed) = screens::sleep_view(store, &day).tonight_bedtime {
        if let Some((h, m)) = bed
            .split_once(':')
            .and_then(|(h, m)| Some((h.parse::<i64>().ok()?, m.parse::<i64>().ok()?)))
        {
            let delta =
                (h * 60 + m - now.hour() as i64 * 60 - now.minute() as i64).rem_euclid(1440);
            if delta <= 30 {
                add(
                    "bedtime",
                    "Time to wind down",
                    format!("Aim to be asleep by {bed}."),
                    day,
                );
            }
        }
    }
    result
}
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
