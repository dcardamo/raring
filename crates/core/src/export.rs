//! Offline archive creation. The caller decides how to hand the completed file to the user.
use crate::store::Store;
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};
use serde_json::{json, Map, Value};
use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::Path,
};
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error("export account binding does not match the active cached profile")]
    AccountMismatch,
}

// Never copy credentials from a future Garmin response or a new settings key.
// The denylist is applied at every depth, before any value reaches either JSON or CSV.
pub(crate) fn credential_key(key: &str) -> bool {
    let key: String = key
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    [
        "token",
        "password",
        "passwd",
        "secret",
        "credential",
        "authorization",
        "authentication",
        "apikey",
        "accesskey",
        "refreshkey",
        "bearer",
        "cookie",
        "sessionkey",
        "sessionid",
        "accountsession",
        "accountpending",
        "clientsecret",
        "privatekey",
        "ticket",
    ]
    .iter()
    .any(|word| key.contains(word))
}

fn safe(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|key, _| !credential_key(key));
            for child in map.values_mut() {
                safe(child);
            }
        }
        Value::Array(items) => {
            for child in items {
                safe(child);
            }
        }
        _ => {}
    }
}

// Spreadsheet applications execute cells beginning with formula sigils or control prefixes.
// Apply only to text cells; typed numeric values keep their sign and remain numbers.
fn csv_text(value: &str) -> String {
    if matches!(
        value.chars().next(),
        Some('=' | '+' | '-' | '@' | '\t' | '\r')
    ) {
        format!("'{value}")
    } else {
        value.to_owned()
    }
}

fn options() -> FileOptions {
    FileOptions::default().compression_method(CompressionMethod::Deflated)
}

fn entry(zip: &mut ZipWriter<File>, name: &str, body: &[u8]) -> Result<(), ExportError> {
    zip.start_file(name, options())?;
    zip.write_all(body)?;
    Ok(())
}

/// Produce an archive from a consistent SQLite snapshot. No network or token store is accessed.
/// The caller supplies a private temporary path and publishes the file only after success.
pub(crate) fn write(store: &Store, path: &Path, version: &str) -> Result<(), ExportError> {
    store.with_export_db(|db| {
        db.execute_batch("BEGIN DEFERRED")?;
        let result = write_snapshot(db, path, version);
        db.execute_batch("ROLLBACK")?;
        result
    })?
}

/// Bind the snapshot to the expected account before any archive bytes are created.
/// Both queries and the subsequent export use this same SQLite read transaction.
pub(crate) fn write_for_account(
    store: &Store,
    path: &Path,
    version: &str,
    expected: &str,
) -> Result<(), ExportError> {
    write_bound_snapshot(store, path, version, expected, || {}, || {})
}

fn write_bound_snapshot(
    store: &Store,
    path: &Path,
    version: &str,
    expected: &str,
    before_first_read: impl FnOnce(),
    after_identity_read: impl FnOnce(),
) -> Result<(), ExportError> {
    store.with_export_db(|db| {
        db.execute_batch("BEGIN DEFERRED")?;
        let result = (|| {
            before_first_read();
            let session: Option<String> = db.query_row(
                "SELECT value FROM settings WHERE key='account_session'", [], |row| row.get(0),
            ).optional()?;
            // This first SELECT pins the WAL snapshot, including profile and export data.
            after_identity_read();
            let active = session.as_deref()
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
                .is_some_and(|s| s["active"] == true && s["identity"].as_str() == Some(expected));
            let profile: Option<String> = db.query_row(
                "SELECT json FROM raw WHERE endpoint='profile' AND key='me'", [], |row| row.get(0),
            ).optional()?;
            let profile_matches = profile.as_deref()
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
                .and_then(|p| crate::account::profile_identity(&p))
                .as_deref() == Some(expected);
            if !crate::account::valid_identity(expected) || !active || !profile_matches {
                return Err(ExportError::AccountMismatch);
            }
            write_snapshot(db, path, version)
        })();
        db.execute_batch("ROLLBACK")?;
        result
    })?
}

/// The active account both markers agree on, read inside the export's own
/// snapshot. None when the account is suspended, unbound or unverifiable.
fn verified_identity(db: &Connection) -> Option<String> {
    let session: Option<String> = db
        .query_row("SELECT value FROM settings WHERE key='account_session'", [], |r| r.get(0))
        .optional()
        .ok()
        .flatten();
    let identity = session
        .as_deref()
        .and_then(|s| serde_json::from_str::<Value>(s).ok())
        .filter(|s| s["active"] == true)
        .and_then(|s| s["identity"].as_str().map(str::to_owned))
        .filter(|id| crate::account::valid_identity(id));
    let profile: Option<String> = db
        .query_row("SELECT json FROM raw WHERE endpoint='profile' AND key='me'", [], |r| r.get(0))
        .optional()
        .ok()
        .flatten();
    let cached = profile
        .as_deref()
        .and_then(|s| serde_json::from_str::<Value>(s).ok())
        .and_then(|p| crate::account::profile_identity(&p));
    match (identity, cached) {
        (Some(bound), Some(profile)) if bound == profile => Some(bound),
        _ => None,
    }
}

fn write_snapshot(db: &Connection, path: &Path, version: &str) -> Result<(), ExportError> {
    let mut file = OpenOptions::new();
    file.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        file.mode(0o600);
    }
    let mut zip = ZipWriter::new(file.open(path)?);
    let mut settings = Map::new();
    let mut stmt = db.prepare("SELECT key,value FROM settings ORDER BY key")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (key, raw) = row?;
        if credential_key(&key) {
            continue;
        }
        let mut value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
        safe(&mut value);
        settings.insert(key, value);
    }
    entry(
        &mut zip,
        "settings.json",
        serde_json::to_string_pretty(&settings)?.as_bytes(),
    )?;

    // Names are generated, never taken from untrusted endpoint or key strings.
    // The index preserves both original identifiers and fetch timestamps.
    let mut index = csv::Writer::from_writer(Vec::new());
    index.write_record(["endpoint", "key", "fetched_at", "file"])?;
    let mut count = 0usize;
    let mut stmt =
        db.prepare("SELECT endpoint,key,fetched_at,json FROM raw ORDER BY endpoint,key")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let endpoint: String = row.get(0)?;
        let key: String = row.get(1)?;
        let fetched_at: String = row.get(2)?;
        let raw: String = row.get(3)?;
        if credential_key(&endpoint) || credential_key(&key) {
            continue;
        }
        let mut value: Value = serde_json::from_str(&raw)?;
        safe(&mut value);
        let name = format!("raw/{count:08}.json");
        entry(&mut zip, &name, value.to_string().as_bytes())?;
        index.write_record([
            csv_text(&endpoint),
            csv_text(&key),
            csv_text(&fetched_at),
            csv_text(&name),
        ])?;
        count += 1;
    }
    entry(
        &mut zip,
        "raw/index.csv",
        &index.into_inner().map_err(|e| e.into_error())?,
    )?;

    // These are the per-day rows in the raw cache (including historical range backfill).
    // Scalars are columns; JSON is also kept to preserve nested values and arrays.
    let mut days = 0usize;
    let mut daily_index = csv::Writer::from_writer(Vec::new());
    daily_index.write_record(["endpoint", "file"])?;
    let mut stmt = db.prepare(
        "SELECT DISTINCT endpoint FROM raw WHERE key GLOB '????-??-??' ORDER BY endpoint",
    )?;
    let endpoints = stmt.query_map([], |row| row.get::<_, String>(0))?;
    for (number, endpoint) in endpoints.enumerate() {
        let endpoint = endpoint?;
        if credential_key(&endpoint) {
            continue;
        }
        let mut columns = BTreeSet::new();
        let mut fields =
            db.prepare("SELECT json FROM raw WHERE endpoint=?1 AND key GLOB '????-??-??'")?;
        let values = fields.query_map([&endpoint], |row| row.get::<_, String>(0))?;
        for raw in values {
            let mut value: Value = serde_json::from_str(&raw?)?;
            safe(&mut value);
            if let Value::Object(map) = value {
                columns.extend(
                    map.into_iter()
                        .filter(|(_, v)| !v.is_object() && !v.is_array())
                        .map(|(k, _)| k),
                );
            }
        }
        let mut csv = csv::Writer::from_writer(Vec::new());
        csv.write_record(
            ["date", "fetched_at"]
                .into_iter()
                .chain(columns.iter().map(String::as_str))
                .chain(["json"])
                .map(csv_text),
        )?;
        let mut daily = db.prepare("SELECT key,fetched_at,json FROM raw WHERE endpoint=?1 AND key GLOB '????-??-??' ORDER BY key")?;
        let mut rows = daily.query([&endpoint])?;
        while let Some(row) = rows.next()? {
            let key: String = row.get(0)?;
            let at: String = row.get(1)?;
            let raw: String = row.get(2)?;
            let mut value: Value = serde_json::from_str(&raw)?;
            safe(&mut value);
            let mut record = vec![csv_text(&key), csv_text(&at)];
            for column in &columns {
                record.push(
                    value
                        .get(column)
                        .filter(|v| !v.is_null())
                        .map(|v| match v {
                            Value::String(s) => csv_text(s),
                            _ => v.to_string(),
                        })
                        .unwrap_or_default(),
                );
            }
            record.push(match &value {
                Value::Number(_) => value.to_string(),
                _ => csv_text(&value.to_string()),
            });
            csv.write_record(&record)?;
            days += 1;
        }
        let name = format!("daily/{number:04}.csv");
        entry(
            &mut zip,
            &name,
            &csv.into_inner().map_err(|e| e.into_error())?,
        )?;
        // Mapping is kept in a separate small CSV, rather than using API paths as zip paths.
        daily_index.write_record([csv_text(&endpoint), csv_text(&name)])?;
    }
    entry(
        &mut zip,
        "daily/index.csv",
        &daily_index.into_inner().map_err(|e| e.into_error())?,
    )?;
    entry(&mut zip, "manifest.json", serde_json::to_string_pretty(&json!({
        "exported_at": Utc::now().to_rfc3339(), "app_version": version,
        "raw_responses": count, "daily_rows": days,
        // The account is a numeric identifier, never a credential; an import
        // refuses a file that names another account.
        "archive": {"format": "raring-export", "version": crate::import::FORMAT_VERSION, "account": verified_identity(db)},
        "format": "raw/index.csv maps Garmin response keys to raw/*.json; daily/index.csv maps daily CSV tables to endpoint names. Settings are local Raring settings only. Credential fields are excluded."
    }))?.as_bytes())?;
    zip.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Read, sync::{mpsc, Arc}, thread};

    fn bound_fixture() -> (Arc<Store>, std::path::PathBuf, std::path::PathBuf) {
        let directory = std::env::temp_dir().join(format!(
            "raring-bound-export-{}-{}", std::process::id(), Utc::now().timestamp_nanos_opt().unwrap()
        ));
        std::fs::create_dir(&directory).unwrap();
        let store = Arc::new(Store::open(directory.join("data.db")).unwrap());
        switch(&store, 11, "A");
        (store, directory.join("archive.zip"), directory)
    }

    fn switch(store: &Store, id: u64, marker: &str) {
        let _boundary = crate::status::WRITE_LOCK.lock().unwrap();
        let profile = json!({"profileId":id,"displayName":marker});
        assert!(store.bind_account(&id.to_string(), &profile, "now").unwrap());
        assert!(store.activate_account().unwrap());
        store.put("summary", "2026-01-02", "now", &json!({"marker":marker,"accessToken":"secret"})).unwrap();
        store.set_setting("plan", &format!("\"{marker}\"")).unwrap();
    }

    fn boundary_available() {
        // Other tests may briefly own the process-wide barrier. Wait for an
        // actual acquisition rather than race them with try_lock(). The export
        // is paused on a channel throughout this assertion.
        let (acquired, ready) = mpsc::channel();
        let probe = thread::spawn(move || {
            let _boundary = crate::status::WRITE_LOCK.lock().unwrap();
            let _ = acquired.send(());
        });
        assert!(ready.recv_timeout(std::time::Duration::from_secs(5)).is_ok(), "ZIP construction must not hold the local session barrier");
        probe.join().unwrap();
    }

    fn assert_archive_marker(path: &Path, marker: &str) {
        let mut archive = zip::ZipArchive::new(File::open(path).unwrap()).unwrap();
        let mut settings = String::new();
        archive.by_name("settings.json").unwrap().read_to_string(&mut settings).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&settings).unwrap()["plan"], marker);
        let mut raw = String::new();
        archive.by_name("raw/00000001.json").unwrap().read_to_string(&mut raw).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&raw).unwrap(), json!({"marker":marker}));
        assert!(!settings.contains("account_session"));
        assert!(!settings.contains("account_pending"));
    }

    #[test]
    fn bound_export_rejects_aba_before_first_snapshot_read_without_creating_bytes() {
        let (store, path, directory) = bound_fixture();
        let (entered_tx, paused) = mpsc::channel();
        let (resume, proceed) = mpsc::channel();
        let (identity_read, read_paused) = mpsc::channel();
        let (finish, finish_rx) = mpsc::channel();
        let worker = {
            let store = Arc::clone(&store);
            let path = path.clone();
            thread::spawn(move || write_bound_snapshot(&store, &path, "test", "11", || {
                entered_tx.send(()).unwrap();
                proceed.recv().unwrap();
            }, || {
                identity_read.send(()).unwrap();
                finish_rx.recv().unwrap();
            }))
        };
        paused.recv().unwrap();
        boundary_available();
        switch(&store, 22, "B");
        resume.send(()).unwrap();
        read_paused.recv().unwrap();
        switch(&store, 11, "A-again");
        finish.send(()).unwrap();
        assert!(matches!(worker.join().unwrap(), Err(ExportError::AccountMismatch)));
        assert!(!path.exists()); // Neither publication nor the native share path can be reached.
        write_for_account(&store, &path, "test", "11").unwrap();
        assert_archive_marker(&path, "A-again");
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn bound_export_rejects_inactive_or_mismatched_cached_profile() {
        let (store, path, directory) = bound_fixture();
        store.put("profile", "me", "now", &json!({"profileId":22,"displayName":"B"})).unwrap();
        assert!(matches!(write_for_account(&store, &path, "test", "11"), Err(ExportError::AccountMismatch)));
        assert!(!path.exists());
        switch(&store, 11, "A-restored");
        store.suspend_account().unwrap();
        assert!(matches!(write_for_account(&store, &path, "test", "11"), Err(ExportError::AccountMismatch)));
        assert!(!path.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn bound_export_after_first_identity_read_keeps_a_only_across_aba() {
        let (store, path, directory) = bound_fixture();
        let (entered, paused) = mpsc::channel();
        let (resume, proceed) = mpsc::channel();
        let worker = {
            let store = Arc::clone(&store);
            let path = path.clone();
            thread::spawn(move || write_bound_snapshot(&store, &path, "test", "11", || {}, || {
                entered.send(()).unwrap();
                proceed.recv().unwrap();
            }))
        };
        paused.recv().unwrap();
        boundary_available();
        switch(&store, 22, "B");
        switch(&store, 11, "A-again");
        resume.send(()).unwrap();
        worker.join().unwrap().unwrap();
        assert_archive_marker(&path, "A");
        std::fs::remove_dir_all(directory).unwrap();
    }

    fn check_keys(value: &Value) {
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    assert!(!credential_key(key), "credential field in exported JSON");
                    check_keys(value);
                }
            }
            Value::Array(items) => {
                for item in items {
                    check_keys(item);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn csv_escapes_text_formulas_but_keeps_numbers_and_raw_json() {
        let store = Store::open(":memory:").unwrap();
        let original = json!({
            "name": "=1+2", "negative": -0.3, "whole": -5,
            "normal": "Easy day", "plus": "+SUM(1,2)", "minus": "-text",
            "at": "@task", "tab": "\t=1+2", "cr": "\r=1+2"
        });
        store
            .put("summary", "2025-01-02", "now", &original)
            .unwrap();
        store
            .put("listed", "=formula", "now", &json!({"name":"sample"}))
            .unwrap();
        let path = std::env::temp_dir().join(format!(
            "raring-csv-test-{}-{}.zip",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap()
        ));
        write(&store, &path, "1.2.3").unwrap();
        let mut archive = zip::ZipArchive::new(File::open(&path).unwrap()).unwrap();
        let mut index = String::new();
        archive
            .by_name("raw/index.csv")
            .unwrap()
            .read_to_string(&mut index)
            .unwrap();
        assert!(index.contains("'=formula"));
        let mut csv_body = String::new();
        archive
            .by_name("daily/0000.csv")
            .unwrap()
            .read_to_string(&mut csv_body)
            .unwrap();
        let mut csv = csv::Reader::from_reader(csv_body.as_bytes());
        let headers = csv.headers().unwrap().clone();
        let row = csv.records().next().unwrap().unwrap();
        let cell = |key| {
            row.get(headers.iter().position(|header| header == key).unwrap())
                .unwrap()
        };
        assert_eq!(cell("name"), "'=1+2");
        assert_eq!(cell("negative"), "-0.3");
        assert_eq!(cell("whole"), "-5");
        assert_eq!(cell("normal"), "Easy day");
        for key in ["plus", "minus", "at", "tab", "cr"] {
            assert!(cell(key).starts_with('\''), "{key} was not escaped");
        }
        let mut raw = String::new();
        archive
            .by_name("raw/00000001.json")
            .unwrap()
            .read_to_string(&mut raw)
            .unwrap();
        assert_eq!(serde_json::from_str::<Value>(&raw).unwrap(), original);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn preserves_real_plan_schema_and_prospectively_captured_snapshot() {
        use chrono::TimeZone;
        let store = Store::open(":memory:").unwrap();
        let date = chrono::NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
        let now = chrono::Local
            .from_local_datetime(&date.and_hms_opt(12, 0, 0).unwrap())
            .single()
            .unwrap();
        crate::demo::seed(&store, date).unwrap();
        crate::plan::set(
            &store,
            date,
            &crate::view::PlanUpdate {
                targets: crate::view::PlanTargets::preset(crate::view::PlanPreset::Strength),
                paused: false,
            },
        )
        .unwrap();
        crate::plan::capture_sleep_plan(&store, now).unwrap();
        let snapshot = "sleep_plan:2025-01-03";
        let expected_plan: Value = serde_json::from_str(&store.setting("plan").unwrap()).unwrap();
        let expected_snapshot: Value =
            serde_json::from_str(&store.setting(snapshot).unwrap()).unwrap();
        let path = std::env::temp_dir().join(format!(
            "raring-plan-export-{}-{}.zip",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap()
        ));
        write(&store, &path, "1.2.3").unwrap();
        let mut archive = zip::ZipArchive::new(File::open(&path).unwrap()).unwrap();
        let mut body = String::new();
        archive
            .by_name("settings.json")
            .unwrap()
            .read_to_string(&mut body)
            .unwrap();
        let settings: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(settings["plan"], expected_plan);
        assert_eq!(settings[snapshot], expected_snapshot);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn excludes_credentials_at_every_depth_and_exports_every_safe_setting() {
        let store = Store::open(":memory:").unwrap();
        store.set_setting("settings", r#"{"notifyBedtime":true,"weeklyTargets":{"sessions":3},"refreshToken":"do-not-export"}"#).unwrap();
        store.set_setting("future-plan", "4").unwrap();
        store.set_setting("plan", r#"{"weeks":{"2025-01-06":{"targets":{"strengthSessions":3},"paused":true}},"refreshToken":"do-not-export"}"#).unwrap();
        store.set_setting("sleep_plan:2025-01-03", r#"{"plan":{"night":"2025-01-03","bedtimeAt":"2025-01-02T22:30:00","accessToken":"do-not-export"},"capturedAt":"2025-01-02T18:00:00"}"#).unwrap();
        store.set_setting("access_token", "do-not-export").unwrap();
        store.set_setting("account_session", r#"{"identity":"101","active":true}"#).unwrap();
        store.set_setting("account_pending", "101").unwrap();
        store.put("summary", "2025-01-02", "now", &json!({"value":1,"oauthToken":"do-not-export","deep":[{"client_secret":"do-not-export","ok":2}]})).unwrap();
        let path = std::env::temp_dir().join(format!(
            "raring-export-test-{}-{}.zip",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap()
        ));
        write(&store, &path, "1.2.3").unwrap();
        let mut archive = zip::ZipArchive::new(File::open(&path).unwrap()).unwrap();
        let mut settings_json = String::new();
        archive
            .by_name("settings.json")
            .unwrap()
            .read_to_string(&mut settings_json)
            .unwrap();
        let settings: Value = serde_json::from_str(&settings_json).unwrap();
        assert_eq!(
            settings["plan"]["weeks"]["2025-01-06"]["targets"]["strengthSessions"],
            3
        );
        assert_eq!(settings["plan"]["weeks"]["2025-01-06"]["paused"], true);
        assert_eq!(
            settings["sleep_plan:2025-01-03"]["plan"]["night"],
            "2025-01-03"
        );
        assert_eq!(
            settings["sleep_plan:2025-01-03"]["plan"]["bedtimeAt"],
            "2025-01-02T22:30:00"
        );
        assert_eq!(
            settings["sleep_plan:2025-01-03"]["capturedAt"],
            "2025-01-02T18:00:00"
        );
        assert!(settings["plan"].get("refreshToken").is_none());
        assert!(settings["sleep_plan:2025-01-03"]["plan"]
            .get("accessToken")
            .is_none());
        let mut contents = String::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let mut body = String::new();
            file.read_to_string(&mut body).unwrap();
            if file.name().ends_with(".json") {
                check_keys(&serde_json::from_str::<Value>(&body).unwrap());
            } else if file.name().starts_with("daily/") && file.name().ends_with(".csv") {
                let mut csv = csv::Reader::from_reader(body.as_bytes());
                for column in csv.headers().unwrap() {
                    assert!(!credential_key(column), "credential column in daily CSV");
                }
            }
            contents.push_str(&body);
        }
        std::fs::remove_file(path).unwrap();
        assert!(!contents.contains("do-not-export"));
        assert!(!contents.contains("access_token"));
        assert!(!contents.contains("account_session"));
        assert!(!contents.contains("account_pending"));
        assert!(!contents.contains("oauthToken"));
        assert!(!contents.contains("client_secret"));
        assert!(contents.contains("future-plan"));
        assert!(contents.contains("weeklyTargets"));
        assert!(contents.contains("daily/0000.csv"));
        assert!(contents.contains("1.2.3"));
    }
}
