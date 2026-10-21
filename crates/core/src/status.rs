//! Private, platform-neutral status snapshot for native surfaces.
use crate::{
    screens,
    store::Store,
    view::{DayPhase, Effort, LeadKind, Tone},
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
    pub phase: DayPhase,
    pub focus_line: String,
    pub focus_target: &'static str,
    pub readiness: Reading,
    pub sleep: Reading,
    pub body_battery: Reading,
    pub stress: Reading,
}

pub fn snapshot(store: &Store, now: DateTime<Local>) -> StatusSnapshot {
    let date = screens::current_date(store, now.naive_local()).to_string();
    // The live Today's own lead decides, so a surface never disagrees with it.
    let mut today = screens::today_at_calendar(store, &date, now.fixed_offset(), now.date_naive());
    // Rest, illness, a pause, limits and "Not today" get a short private
    // line; never the limits text or a session's name. Read from the view
    // before the lead changes it.
    let local = crate::local::load(store);
    let (state, dismissed) = crate::lead::hold_back(&local, &mut today.clone(), now.date_naive());
    crate::lead::apply(store, &mut today, now.date_naive(), now.fixed_offset());
    let phase = today.focus.phase;
    let held = || {
        crate::lead::circumstance_line(&state)
            .or(dismissed.then_some("No training suggestion today"))
    };
    let effort = today
        .call
        .filter(|_| today.focus.effort && today.readiness.is_some());
    let lead = today.lead.as_ref();
    let (focus_line, focus_target): (String, &'static str) = match lead.map(|l| l.kind) {
        Some(LeadKind::Sleep) => (lead.map(|l| l.headline.clone()).unwrap_or_default(), "sleep"),
        Some(LeadKind::Waiting) | None => ("Waiting for last night's sleep".into(), "today"),
        // The day can finish: no effort call, and never the activity's name.
        Some(LeadKind::Done) => ("Activity recorded today".into(), "today"),
        Some(
            LeadKind::Ill | LeadKind::Paused | LeadKind::Rest | LeadKind::Limits | LeadKind::Review | LeadKind::Quiet,
        ) => (held().unwrap_or("Training suggestions limited").into(), "today"),
        // Wherever Today shows Garmin's effort call.
        Some(_) => match (held(), effort) {
            (Some(line), _) => (line.into(), "today"),
            (None, Some(effort)) => (
                match effort {
                    Effort::Rest => "Rest",
                    Effort::Easy => "Easy",
                    Effort::Steady => "Steady",
                    Effort::Push => "Push",
                }
                .into(),
                "readiness",
            ),
            (None, None) => ("Waiting for readiness".into(), "today"),
        },
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
    StatusSnapshot {
        version: 1,
        account_id: crate::account::identity(store),
        generated_at: now.with_timezone(&Utc),
        as_of: today.watch_last_upload,
        date,
        phase,
        focus_line,
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
