//! Private, platform-neutral status snapshot for native surfaces.
use crate::{
    rules, screens,
    store::Store,
    view::{DayPhase, Effort, Tone},
};
use chrono::{DateTime, Local, Utc};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    pub value: Option<f64>,
    pub tone: Tone,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSnapshot {
    pub version: u8,
    /// Stable verified account binding, checked by the core before native presentation.
    pub account_id: Option<String>,
    pub generated_at: DateTime<Utc>,
    pub as_of: Option<String>,
    pub date: String,
    pub enabled: bool,
    pub phase: DayPhase,
    pub focus_line: String,
    pub focus_reason: Option<String>,
    pub focus_target: &'static str,
    pub readiness: Reading,
    pub sleep: Reading,
    pub body_battery: Reading,
    pub stress: Reading,
    pub heart_rate: Reading,
}

pub fn snapshot(store: &Store, now: DateTime<Local>) -> StatusSnapshot {
    let date = screens::current_date(store, now.naive_local()).to_string();
    let today = screens::today_at(store, &date, now.fixed_offset());
    let phase = today.focus.phase;
    let night = matches!(
        phase,
        DayPhase::Evening | DayPhase::WindDown | DayPhase::LateNight
    );
    let waiting = !today.focus.effort || today.call.is_none() || today.readiness.is_none();
    let focus_line = if night || !today.focus.effort {
        today.headline.clone()
    } else if waiting {
        "Waiting for readiness".into()
    } else {
        match today.call {
            Some(Effort::Rest) => "Rest",
            Some(Effort::Easy) => "Easy",
            Some(Effort::Steady) => "Steady",
            Some(Effort::Push) => "Push",
            None => "Waiting for readiness",
        }
        .into()
    };
    let focus_reason = if night {
        today
            .sleep_plan
            .as_ref()
            .and_then(|plan| plan.reason.clone())
    } else if waiting {
        None
    } else {
        rules::limiting_factor(&today.factors)
            .map(|factor| format!("{} is your biggest drag today.", factor.name))
    };
    let focus_target = if night {
        "sleep"
    } else if waiting {
        "today"
    } else {
        "readiness"
    };
    // Overnight handover may keep yesterday's readiness on Today, but it must
    // not present yesterday's daytime readings as today's latest values.
    let calendar_date = now.date_naive().to_string();
    let current = (calendar_date != date)
        .then(|| screens::today_at(store, &calendar_date, now.fixed_offset()));
    let live = current.as_ref().unwrap_or(&today);
    let battery = live.body_battery.now.filter(|v| (0. ..=100.).contains(v));
    // The stress card's average is a different measurement; never substitute it.
    let stress = live.stress.timeline.iter().rev().flatten().next().copied();
    let heart_rate = live.heart_rate_day.timeline.iter().rev().flatten().next().copied();
    StatusSnapshot {
        version: 1,
        account_id: crate::account::identity(store),
        generated_at: now.with_timezone(&Utc),
        as_of: today.watch_last_upload,
        date,
        enabled: store.settings().notify_status,
        phase,
        focus_line,
        focus_reason,
        focus_target,
        readiness: today.readiness.map_or(
            Reading {
                value: None,
                tone: Tone::Neutral,
            },
            |g| Reading {
                value: Some(g.value),
                tone: g.tone,
            },
        ),
        sleep: today.sleep.map_or(
            Reading {
                value: None,
                tone: Tone::Neutral,
            },
            |g| Reading {
                value: Some(g.value),
                tone: g.tone,
            },
        ),
        body_battery: Reading {
            value: battery,
            tone: match battery {
                Some(v) if v >= 70. => Tone::Good,
                Some(v) if v >= 30. => Tone::Fair,
                Some(_) => Tone::Poor,
                None => Tone::Neutral,
            },
        },
        stress: Reading {
            value: stress,
            tone: Tone::Neutral,
        },
        // Reuse the all-day parser; resting/overnight HR cannot substitute for a sample.
        heart_rate: Reading {
            value: heart_rate,
            tone: Tone::Neutral,
        },
    }
}

// Tauri and JNI open separate core handles to the same database. Keep a
// snapshot read/build/replace from crossing another handle's account erasure.
pub(crate) static WRITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

/// Same-directory temporary file and rename: native readers see either complete JSON or none.
pub fn erase(path: &Path) -> io::Result<()> {
    let remove = |path: &Path| match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    };
    remove(path)?;
    // A crash between creating and renaming may leave private JSON behind.
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let prefix = format!(
        "{}.",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) && name.ends_with(".tmp") {
            remove(&entry.path())?;
        }
    }
    Ok(())
}

pub fn write(path: &Path, snapshot: &StatusSnapshot) -> io::Result<()> {
    let data = serde_json::to_vec(snapshot).map_err(io::Error::other)?;
    let suffix = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let temp = path.with_file_name(format!(
        "{}.{}.{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id(),
        suffix
    ));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp)?;
        file.write_all(&data)?;
        file.sync_all()?;
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
