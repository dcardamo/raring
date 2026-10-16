//! SQLite raw response cache. Nothing parsed is discarded: changing parsers never needs a refetch.
use crate::view::Settings;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use std::{path::Path, sync::Mutex};

pub struct Store {
    db: Mutex<Connection>,
}
impl Store {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let db = Connection::open(path)?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version < 1 {
            db.execute_batch("BEGIN;
                CREATE TABLE IF NOT EXISTS raw (endpoint TEXT NOT NULL, key TEXT NOT NULL, fetched_at TEXT NOT NULL, json TEXT NOT NULL, PRIMARY KEY(endpoint,key));
                CREATE TABLE IF NOT EXISTS activities (id TEXT PRIMARY KEY, start_local TEXT NOT NULL);
                CREATE INDEX IF NOT EXISTS activities_start ON activities(start_local DESC);
                CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                CREATE TABLE IF NOT EXISTS notifications (kind TEXT NOT NULL, tag TEXT NOT NULL, sent_at TEXT NOT NULL, PRIMARY KEY(kind,tag));
                CREATE TABLE IF NOT EXISTS cursors (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                PRAGMA user_version=1; COMMIT;")?;
        }
        Ok(Self { db: Mutex::new(db) })
    }
    pub fn put(&self, endpoint: &str, key: &str, at: &str, value: &Value) -> rusqlite::Result<()> {
        let db = self.db.lock().unwrap();
        db.execute("INSERT INTO raw VALUES (?1,?2,?3,?4) ON CONFLICT(endpoint,key) DO UPDATE SET fetched_at=excluded.fetched_at,json=excluded.json", params![endpoint,key,at,value.to_string()])?;
        if endpoint == "activity" || endpoint == "activities" {
            let items: Vec<&Value> = if let Some(a) = value.as_array() {
                a.iter().collect()
            } else {
                vec![value]
            };
            for item in items {
                let id = item["activityId"]
                    .as_u64()
                    .map(|v| v.to_string())
                    .or_else(|| item["activityId"].as_str().map(str::to_owned));
                if let (Some(id), Some(start)) = (
                    id,
                    item["summaryDTO"]["startTimeLocal"]
                        .as_str()
                        .or_else(|| item["startTimeLocal"].as_str()),
                ) {
                    db.execute("INSERT INTO activities VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET start_local=excluded.start_local",params![id,start])?;
                }
            }
        }
        Ok(())
    }
    pub fn get(&self, endpoint: &str, key: &str) -> Option<Value> {
        let db = self.db.lock().unwrap();
        db.query_row(
            "SELECT json FROM raw WHERE endpoint=?1 AND key=?2",
            params![endpoint, key],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
    }
    pub fn history(&self, endpoint: &str, through: &str, limit: usize) -> Vec<(String, Value)> {
        let db = self.db.lock().unwrap();
        let Ok(mut stmt) = db.prepare(
            "SELECT key,json FROM raw WHERE endpoint=?1 AND key<=?2 ORDER BY key DESC LIMIT ?3",
        ) else {
            return vec![];
        };
        stmt.query_map(params![endpoint, through, limit as i64], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map(|rows| {
            rows.flatten()
                .filter_map(|(k, s)| serde_json::from_str(&s).ok().map(|v| (k, v)))
                .collect()
        })
        .unwrap_or_default()
    }
    pub fn activity_ids(&self, since: Option<&str>, limit: usize) -> Vec<String> {
        let db = self.db.lock().unwrap();
        let Ok(mut stmt) = db.prepare(
            "SELECT id FROM activities WHERE start_local>=?1 ORDER BY replace(start_local, ' ', 'T') DESC LIMIT ?2",
        ) else {
            return vec![];
        };
        stmt.query_map(params![since.unwrap_or(""), limit as i64], |r| r.get(0))
            .map(|it| it.flatten().collect())
            .unwrap_or_default()
    }
    pub fn newest_activity(&self) -> Option<String> {
        self.db
            .lock()
            .unwrap()
            .query_row(
                "SELECT start_local FROM activities ORDER BY replace(start_local, ' ', 'T') DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten()
    }
    pub fn setting(&self, key: &str) -> Option<String> {
        self.db
            .lock()
            .unwrap()
            .query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .optional()
            .ok()
            .flatten()
    }
    pub fn set_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.db.lock().unwrap().execute("INSERT INTO settings VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![key,value])?;
        Ok(())
    }
    pub fn settings(&self) -> Settings {
        self.setting("settings")
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    pub fn set_settings(&self, settings: &Settings) -> rusqlite::Result<()> {
        let json = serde_json::to_string(settings)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        self.set_setting("settings", &json)
    }
    pub fn cursor(&self, key: &str) -> Option<String> {
        self.db
            .lock()
            .unwrap()
            .query_row("SELECT value FROM cursors WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .optional()
            .ok()
            .flatten()
    }
    pub fn set_cursor(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.db.lock().unwrap().execute("INSERT INTO cursors VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,value])?;
        Ok(())
    }
    pub fn notified(&self, kind: &str, tag: &str) -> bool {
        self.db
            .lock()
            .unwrap()
            .query_row(
                "SELECT 1 FROM notifications WHERE kind=?1 AND tag=?2",
                params![kind, tag],
                |r| r.get::<_, i64>(0),
            )
            .optional()
            .ok()
            .flatten()
            .is_some()
    }
    pub fn mark_notified(&self, kind: &str, tag: &str, at: &str) -> rusqlite::Result<()> {
        self.db.lock().unwrap().execute(
            "INSERT OR IGNORE INTO notifications VALUES (?1,?2,?3)",
            params![kind, tag, at],
        )?;
        Ok(())
    }
    pub fn days(&self) -> u32 {
        self.db
            .lock()
            .unwrap()
            .query_row(
                "SELECT count(*) FROM raw WHERE endpoint='summary'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0)
    }
    pub fn forget_account(&self) -> rusqlite::Result<()> {
        self.db.lock().unwrap().execute_batch("DELETE FROM raw; DELETE FROM activities; DELETE FROM notifications; DELETE FROM cursors; DELETE FROM settings;")
    }
}
