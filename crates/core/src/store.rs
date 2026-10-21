//! SQLite raw response cache. Nothing parsed is discarded: changing parsers never needs a refetch.
use crate::view::Settings;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

pub struct Store {
    db: Mutex<Connection>,
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct AccountSession {
    identity: Option<String>,
    active: bool,
}

pub(crate) struct SettingsTx<'a>(&'a rusqlite::Transaction<'a>);

impl SettingsTx<'_> {
    pub(crate) fn get(&self, key: &str) -> rusqlite::Result<Option<String>> {
        self.0
            .query_row("SELECT value FROM settings WHERE key=?1", [key], |r| r.get(0))
            .optional()
    }
    pub(crate) fn set(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.0.execute("INSERT INTO settings VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![key, value])?;
        Ok(())
    }
    pub(crate) fn activity_exists(&self, id: &str) -> rusqlite::Result<bool> {
        self.0.query_row("SELECT EXISTS(SELECT 1 FROM activities WHERE id=?1)", [id], |r| r.get(0))
    }
}

/// Builds a JSON object without the named top-level fields.
struct Without<'a>(&'a [&'a str]);
impl<'de> serde::de::Visitor<'de> for Without<'_> {
    type Value = Value;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a JSON object")
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut object = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if self.0.contains(&key.as_str()) {
                map.next_value::<serde::de::IgnoredAny>()?;
            } else {
                object.insert(key, map.next_value()?);
            }
        }
        Ok(Value::Object(object))
    }
}

const SESSION_KEY: &str = "account_session";
const PENDING_KEY: &str = "account_pending";
impl Store {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let db = Connection::open(&path)?;
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
        if version < 2 {
            // Street maps: an area's tiles arrive together; map_areas lists the complete ones.
            db.execute_batch("BEGIN;
                CREATE TABLE IF NOT EXISTS map_tiles (z INTEGER NOT NULL, x INTEGER NOT NULL, y INTEGER NOT NULL, data BLOB NOT NULL, PRIMARY KEY(z,x,y));
                CREATE TABLE IF NOT EXISTS map_areas (z INTEGER NOT NULL, x INTEGER NOT NULL, y INTEGER NOT NULL, source TEXT NOT NULL, fetched_at TEXT NOT NULL, PRIMARY KEY(z,x,y));
                CREATE TABLE IF NOT EXISTS route_maps (activity TEXT PRIMARY KEY, json TEXT NOT NULL);
                PRAGMA user_version=2; COMMIT;")?;
        }
        Ok(Self {
            db: Mutex::new(db),
            path,
        })
    }
    pub(crate) fn with_export_db<T>(
        &self,
        f: impl FnOnce(&Connection) -> T,
    ) -> rusqlite::Result<T> {
        if self.path == Path::new(":memory:") {
            return Ok(f(&self.db.lock().unwrap()));
        }
        let db =
            Connection::open_with_flags(&self.path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        Ok(f(&db))
    }
    pub fn put(&self, endpoint: &str, key: &str, at: &str, value: &Value) -> rusqlite::Result<()> {
        let db = self.db.lock().unwrap();
        db.execute("INSERT INTO raw VALUES (?1,?2,?3,?4) ON CONFLICT(endpoint,key) DO UPDATE SET fetched_at=excluded.fetched_at,json=excluded.json", params![endpoint,key,at,value.to_string()])?;
        if matches!(endpoint, "activity" | "activities" | "listed") {
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
    /// The consent check and journal write are one SQLite statement. Another
    /// Store connection changing settings serializes with this write lock;
    /// a response arriving after opt-out cannot repopulate the cache.
    pub fn put_journal_if_enabled(&self, key: &str, at: &str, value: &Value) -> rusqlite::Result<bool> {
        let db = self.db.lock().unwrap();
        let changed = db.execute(
            "INSERT INTO raw(endpoint,key,fetched_at,json)
             SELECT 'journal', ?1, ?2, ?3
             WHERE EXISTS (SELECT 1 FROM settings WHERE key='settings'
                           AND json_extract(value,'$.journalEnabled')=1)
             ON CONFLICT(endpoint,key) DO UPDATE SET fetched_at=excluded.fetched_at,json=excluded.json",
            params![key, at, value.to_string()],
        )?;
        Ok(changed != 0)
    }
    fn json(&self, endpoint: &str, key: &str) -> Option<String> {
        let db = self.db.lock().unwrap();
        db.query_row(
            "SELECT json FROM raw WHERE endpoint=?1 AND key=?2",
            params![endpoint, key],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .ok()
        .flatten()
    }
    pub fn get(&self, endpoint: &str, key: &str) -> Option<Value> {
        self.json(endpoint, key).and_then(|s| serde_json::from_str(&s).ok())
    }
    /// `get` without the named top-level fields, which are skipped unparsed.
    /// A row that isn't an object is returned whole.
    pub fn get_without(&self, endpoint: &str, key: &str, skip: &[&str]) -> Option<Value> {
        let json = self.json(endpoint, key)?;
        let mut de = serde_json::Deserializer::from_str(&json);
        serde::Deserializer::deserialize_map(&mut de, Without(skip))
            .and_then(|v| de.end().map(|()| v))
            .ok()
            .or_else(|| serde_json::from_str(&json).ok())
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
    /// Day rows from `from` to `to`, oldest first.
    pub fn days_between(&self, endpoint: &str, from: &str, to: &str) -> Vec<(String, Value)> {
        let db = self.db.lock().unwrap();
        let Ok(mut stmt) = db.prepare(
            "SELECT key,json FROM raw WHERE endpoint=?1 AND key BETWEEN ?2 AND ?3 ORDER BY key",
        ) else {
            return vec![];
        };
        stmt.query_map(params![endpoint, from, to], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map(|rows| {
            rows.flatten()
                .filter_map(|(k, s)| serde_json::from_str(&s).ok().map(|v| (k, v)))
                .collect()
        })
        .unwrap_or_default()
    }
    /// One number per day row, the first of `paths` (SQLite JSON paths such as
    /// `$.a.b`) that holds one. SQLite extracts it, so large rows such as a
    /// night's sleep are never parsed whole.
    pub fn day_numbers(
        &self,
        endpoint: &str,
        paths: &[&str],
        from: &str,
        to: &str,
    ) -> Vec<(String, f64)> {
        if paths.is_empty() {
            return vec![];
        }
        let extract: Vec<String> = (0..paths.len())
            .map(|i| format!("json_extract(json,?{})", i + 4))
            .collect();
        let sql = format!(
            "SELECT key,{} FROM raw WHERE endpoint=?1 AND key BETWEEN ?2 AND ?3 AND json!='null' ORDER BY key",
            // COALESCE needs at least two arguments.
            if extract.len() == 1 {
                extract[0].clone()
            } else {
                format!("COALESCE({})", extract.join(","))
            }
        );
        let db = self.db.lock().unwrap();
        let Ok(mut stmt) = db.prepare(&sql) else {
            return vec![];
        };
        let mut args: Vec<&dyn rusqlite::ToSql> = vec![&endpoint, &from, &to];
        args.extend(paths.iter().map(|p| p as &dyn rusqlite::ToSql));
        stmt.query_map(args.as_slice(), |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<f64>>(1).ok().flatten()))
        })
        .map(|rows| {
            rows.flatten()
                .filter_map(|(k, v)| v.map(|v| (k, v)))
                .collect()
        })
        .unwrap_or_default()
    }
    /// The text at one SQLite JSON path in each stored day from `from` to `to`;
    /// rows without text there are left out.
    pub fn day_strings(&self, endpoint: &str, path: &str, from: &str, to: &str) -> Vec<(String, String)> {
        let db = self.db.lock().unwrap();
        let Ok(mut stmt) = db.prepare(
            "SELECT key,json_extract(json,?4) FROM raw WHERE endpoint=?1 AND key BETWEEN ?2 AND ?3 AND json!='null' ORDER BY key",
        ) else {
            return vec![];
        };
        stmt.query_map(params![endpoint, from, to, path], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1).ok().flatten()))
        })
        .map(|rows| rows.flatten().filter_map(|(k, v)| v.map(|v| (k, v))).collect())
        .unwrap_or_default()
    }
    /// The earliest stored key among these endpoints, ignoring empty answers.
    pub fn first_key(&self, endpoints: &[&str]) -> Option<String> {
        let db = self.db.lock().unwrap();
        endpoints
            .iter()
            .filter_map(|endpoint| {
                db.query_row(
                    "SELECT MIN(key) FROM raw WHERE endpoint=?1 AND json!='null'",
                    params![endpoint],
                    |r| r.get::<_, Option<String>>(0),
                )
                .ok()
                .flatten()
            })
            .min()
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
    /// Up to `limit` activities older than (`start_local`, `id`), newest
    /// first, with their start times: one page of stored history.
    pub fn activities_before(&self, start_local: &str, id: &str, limit: usize) -> Vec<(String, String)> {
        let db = self.db.lock().unwrap();
        let Ok(mut stmt) = db.prepare(
            "SELECT id,start_local FROM activities
             WHERE replace(start_local, ' ', 'T')<replace(?1, ' ', 'T')
                OR (replace(start_local, ' ', 'T')=replace(?1, ' ', 'T') AND id<?2)
             ORDER BY replace(start_local, ' ', 'T') DESC, id DESC LIMIT ?3",
        ) else {
            return vec![];
        };
        stmt.query_map(params![start_local, id, limit as i64], |r| Ok((r.get(0)?, r.get(1)?)))
            .map(|it| it.flatten().collect())
            .unwrap_or_default()
    }
    /// Activities starting on local days `from` through `to` (YYYY-MM-DD), oldest first.
    pub fn activities_between(&self, from: &str, to: &str) -> Vec<(String, String)> {
        let db = self.db.lock().unwrap();
        let Ok(mut stmt) = db.prepare(
            "SELECT id,start_local FROM activities WHERE substr(start_local,1,10) BETWEEN ?1 AND ?2 ORDER BY replace(start_local, ' ', 'T')",
        ) else {
            return vec![];
        };
        stmt.query_map(params![from, to], |r| Ok((r.get(0)?, r.get(1)?)))
            .map(|it| it.flatten().collect())
            .unwrap_or_default()
    }
    /// All strength activity IDs in [since, before), newest first. Filter by
    /// sport before returning IDs; busy weeks in other sports must not hide
    /// strength history. Read all three cached activity shapes, including
    /// recent list pages whose detail has not downloaded yet.
    pub fn strength_activity_ids(&self, since: &str, before: &str) -> Vec<String> {
        let db = self.db.lock().unwrap();
        let Ok(mut stmt) = db.prepare(
            "WITH page_items AS (
                SELECT CAST(json_extract(item.value, '$.activityId') AS TEXT) AS id,
                       json_extract(item.value, '$.activityType.typeKey') AS kind, r.key AS page
                FROM raw r, json_each(CASE WHEN r.endpoint='activities' AND json_type(r.json)='array' THEN r.json ELSE '[]' END) item
            ), pages AS (
                SELECT id, kind, row_number() OVER (PARTITION BY id ORDER BY page DESC) AS rank
                FROM page_items
            )
            SELECT DISTINCT a.id FROM activities a
            LEFT JOIN raw d ON d.endpoint='activity' AND d.key=a.id
            LEFT JOIN raw l ON l.endpoint='listed' AND l.key=a.id
            LEFT JOIN pages p ON p.id=a.id AND p.rank=1
            WHERE a.start_local>=?1 AND a.start_local<?2
              AND lower(coalesce(json_extract(d.json, '$.activityTypeDTO.typeKey'),
                                 json_extract(d.json, '$.activityType.typeKey'),
                                 p.kind, json_extract(l.json, '$.activityType.typeKey')))='strength_training'
            ORDER BY replace(a.start_local, ' ', 'T') DESC, a.id DESC",
        ) else {
            return vec![];
        };
        stmt.query_map(params![since, before], |r| r.get(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }
    /// A recent activity list item, without a page-count cap. Detail and
    /// historical `listed` rows are handled by the caller.
    pub fn listed_activity(&self, id: &str) -> Option<Value> {
        let db = self.db.lock().unwrap();
        db.query_row(
            // The primary key narrows the scan to list pages, and a text match
            // (stored JSON is compact) skips pages without the id before any is
            // parsed; json_each then checks the id exactly.
            "SELECT item.value FROM raw r,
             json_each(CASE WHEN json_type(r.json)='array' THEN r.json ELSE '[]' END) item
             WHERE r.endpoint='activities' AND instr(r.json, '\"activityId\":' || ?1) > 0
               AND CAST(json_extract(item.value, '$.activityId') AS TEXT)=?1
             ORDER BY r.key DESC LIMIT 1",
            [id],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
    }
    /// Stored exercise sets with each activity's local start, oldest first.
    pub fn exercise_sets(&self) -> Vec<(String, String, Value)> {
        let db = self.db.lock().unwrap();
        let Ok(mut stmt) = db.prepare(
            "SELECT a.id, a.start_local, r.json FROM raw r JOIN activities a ON a.id=r.key WHERE r.endpoint='exercise_sets' AND r.json!='null' ORDER BY replace(a.start_local, ' ', 'T')",
        ) else {
            return vec![];
        };
        stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map(|rows| {
            rows.flatten()
                .filter_map(|(id, start, s)| serde_json::from_str(&s).ok().map(|v| (id, start, v)))
                .collect()
        })
        .unwrap_or_default()
    }
    /// Older listed strength activities without stored sets, newest first.
    pub fn strength_without_sets(&self, limit: usize) -> Vec<String> {
        let db = self.db.lock().unwrap();
        let Ok(mut stmt) = db.prepare(
            "SELECT key FROM raw WHERE endpoint='listed' AND json_extract(json,'$.activityType.typeKey')='strength_training' AND key NOT IN (SELECT key FROM raw WHERE endpoint='exercise_sets') ORDER BY replace(json_extract(json,'$.startTimeLocal'), ' ', 'T') DESC LIMIT ?1",
        ) else {
            return vec![];
        };
        stmt.query_map([limit as i64], |r| r.get(0))
            .map(|it| it.flatten().collect())
            .unwrap_or_default()
    }
    pub fn fetched_at(&self, endpoint: &str, key: &str) -> Option<String> {
        self.db
            .lock()
            .unwrap()
            .query_row(
                "SELECT fetched_at FROM raw WHERE endpoint=?1 AND key=?2",
                params![endpoint, key],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten()
    }
    pub fn oldest_activity(&self) -> Option<String> {
        self.db
            .lock()
            .unwrap()
            .query_row(
                "SELECT start_local FROM activities ORDER BY replace(start_local, ' ', 'T') LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten()
    }
    /// The oldest day key with data among the given per-day endpoints.
    pub fn earliest_day(&self, endpoints: &[&str]) -> Option<String> {
        let db = self.db.lock().unwrap();
        endpoints
            .iter()
            .filter_map(|e| {
                db.query_row(
                    "SELECT min(key) FROM raw WHERE endpoint=?1 AND json!='null'",
                    [e],
                    |r| r.get::<_, Option<String>>(0),
                )
                .ok()
                .flatten()
            })
            .min()
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
    /// Several settings rows read and written as one immediate transaction.
    pub(crate) fn settings_tx<T, E: From<rusqlite::Error>>(
        &self,
        f: impl FnOnce(&SettingsTx) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let out = f(&SettingsTx(&tx))?;
        tx.commit()?;
        Ok(out)
    }
    /// Several settings rows read in one consistent snapshot, without the write lock.
    pub(crate) fn settings_read<T, E: From<rusqlite::Error>>(
        &self,
        f: impl FnOnce(&SettingsTx) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Deferred)?;
        let out = f(&SettingsTx(&tx))?;
        tx.commit()?;
        Ok(out)
    }
    /// Prospective evidence is immutable: a later sync must not move a night's target.
    pub fn insert_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.db.lock().unwrap().execute(
            "INSERT OR IGNORE INTO settings (key,value) VALUES (?1,?2)",
            params![key, value],
        )?;
        Ok(())
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
    /// Invalid session metadata never grants access or authorizes a data switch.
    pub fn account_identity(&self) -> Option<String> {
        self.setting(SESSION_KEY)
            .and_then(|s| serde_json::from_str::<AccountSession>(&s).ok())
            .and_then(|s| s.identity)
            .filter(|id| crate::account::valid_identity(id))
    }
    pub fn account_active(&self) -> bool {
        let db = self.db.lock().unwrap();
        let session: Option<String> = db
            .query_row(
                "SELECT value FROM settings WHERE key=?1",
                [SESSION_KEY],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        let session = session.and_then(|s| serde_json::from_str::<AccountSession>(&s).ok());
        if !session.as_ref().is_some_and(|s| {
            s.active
                && s.identity
                    .as_deref()
                    .is_some_and(crate::account::valid_identity)
        }) {
            return false;
        }
        let profile: Option<String> = db
            .query_row(
                "SELECT json FROM raw WHERE endpoint='profile' AND key='me'",
                [],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        profile
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .and_then(|v| crate::account::profile_identity(&v))
            == session.and_then(|s| s.identity)
    }
    pub fn suspend_account(&self) -> rusqlite::Result<()> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction()?;
        let current: Option<String> = tx
            .query_row(
                "SELECT value FROM settings WHERE key=?1",
                [SESSION_KEY],
                |r| r.get(0),
            )
            .optional()?;
        // Preserve malformed metadata: its unknown identity must not be silently
        // replaced with a migration fallback on the next binding attempt.
        if current.as_deref().is_some_and(|s| {
            serde_json::from_str::<AccountSession>(s).map_or(true, |session| {
                session
                    .identity
                    .as_deref()
                    .is_some_and(|id| !crate::account::valid_identity(id))
            })
        }) {
            tx.execute("DELETE FROM settings WHERE key=?1", [PENDING_KEY])?;
            tx.commit()?;
            return Ok(());
        }
        let identity = current
            .and_then(|s| serde_json::from_str::<AccountSession>(&s).ok())
            .and_then(|s| s.identity);
        let session = serde_json::json!({"identity": identity, "active": false});
        tx.execute("INSERT INTO settings (key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![SESSION_KEY, session.to_string()])?;
        tx.execute("DELETE FROM settings WHERE key=?1", [PENDING_KEY])?;
        tx.commit()?;
        Ok(())
    }
    /// Returns false when unbound history cannot safely be assigned to a different account.
    /// Profile and session transition together, with a complete erase only for a known switch.
    pub fn bind_account(
        &self,
        identity: &str,
        profile: &Value,
        at: &str,
    ) -> rusqlite::Result<bool> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let session: Option<String> = tx
            .query_row(
                "SELECT value FROM settings WHERE key=?1",
                [SESSION_KEY],
                |r| r.get(0),
            )
            .optional()?;
        let session = session
            .as_deref()
            .map(serde_json::from_str::<AccountSession>);
        if session.as_ref().is_some_and(|s| {
            s.as_ref().map_or(true, |session| {
                session
                    .identity
                    .as_deref()
                    .is_some_and(|id| !crate::account::valid_identity(id))
            })
        }) || !crate::account::valid_identity(identity)
            || crate::account::profile_identity(profile).as_deref() != Some(identity)
        {
            return Ok(false);
        }
        let bound = session.and_then(Result::ok).and_then(|s| s.identity);
        let cached: Option<String> = tx
            .query_row(
                "SELECT json FROM raw WHERE endpoint='profile' AND key='me'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        let cached_identity = cached
            .as_deref()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| crate::account::profile_identity(&v));
        // Existing metadata is authoritative; an old cached profile is only a migration fallback.
        let old = bound.or(cached_identity);
        let has_history: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM raw UNION ALL SELECT 1 FROM activities UNION ALL SELECT 1 FROM cursors UNION ALL SELECT 1 FROM notifications UNION ALL SELECT 1 FROM settings WHERE key NOT IN (?1,?2))",
            params![SESSION_KEY, PENDING_KEY], |r| r.get(0),
        )?;
        if old.is_none() && has_history {
            return Ok(false);
        }
        if old.as_deref().is_some_and(|old| old != identity) {
            tx.execute_batch("DELETE FROM raw; DELETE FROM activities; DELETE FROM notifications; DELETE FROM cursors; DELETE FROM settings;")?;
            tx.execute_batch(ERASE_MAPS)?;
        }
        tx.execute("INSERT INTO raw (endpoint,key,fetched_at,json) VALUES ('profile','me',?1,?2) ON CONFLICT(endpoint,key) DO UPDATE SET fetched_at=excluded.fetched_at,json=excluded.json", params![at, profile.to_string()])?;
        let session = serde_json::json!({"identity": identity, "active": false});
        tx.execute("INSERT INTO settings (key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![SESSION_KEY, session.to_string()])?;
        tx.execute("INSERT INTO settings (key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![PENDING_KEY, identity])?;
        tx.commit()?;
        Ok(true)
    }
    /// Only the pending verified profile may be activated.
    pub fn activate_account(&self) -> rusqlite::Result<bool> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction()?;
        let session: Option<String> = tx
            .query_row(
                "SELECT value FROM settings WHERE key=?1",
                [SESSION_KEY],
                |r| r.get(0),
            )
            .optional()?;
        let identity = session
            .and_then(|s| serde_json::from_str::<AccountSession>(&s).ok())
            .filter(|s| {
                !s.active
                    && s.identity
                        .as_deref()
                        .is_some_and(crate::account::valid_identity)
            })
            .and_then(|s| s.identity);
        let pending: Option<String> = tx
            .query_row(
                "SELECT value FROM settings WHERE key=?1",
                [PENDING_KEY],
                |r| r.get(0),
            )
            .optional()?;
        let profile: Option<String> = tx
            .query_row(
                "SELECT json FROM raw WHERE endpoint='profile' AND key='me'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        let profile_id = profile
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .and_then(|v| crate::account::profile_identity(&v));
        if identity.is_none() || identity != profile_id || identity != pending {
            return Ok(false);
        }
        let session = serde_json::json!({"identity": identity, "active": true});
        tx.execute(
            "UPDATE settings SET value=?1 WHERE key=?2",
            params![session.to_string(), SESSION_KEY],
        )?;
        tx.execute("DELETE FROM settings WHERE key=?1", [PENDING_KEY])?;
        tx.commit()?;
        Ok(true)
    }
    pub fn forget_account(&self) -> rusqlite::Result<()> {
        let db = self.db.lock().unwrap();
        db.execute_batch("DELETE FROM raw; DELETE FROM activities; DELETE FROM notifications; DELETE FROM cursors; DELETE FROM settings;")?;
        db.execute_batch(ERASE_MAPS)
    }
    pub(crate) fn map_area_stored(&self, (z, x, y): TileKey) -> bool {
        let db = self.db.lock().unwrap();
        db.query_row("SELECT EXISTS(SELECT 1 FROM map_areas WHERE z=?1 AND x=?2 AND y=?3)", params![z, x, y], |r| r.get(0))
            .unwrap_or(false)
    }
    /// A stored tile's data, empty where the source had none.
    pub(crate) fn map_tile(&self, z: u8, x: u32, y: u32) -> Option<Vec<u8>> {
        let db = self.db.lock().unwrap();
        db.query_row("SELECT data FROM map_tiles WHERE z=?1 AND x=?2 AND y=?3", params![z, x, y], |r| r.get(0))
            .optional()
            .ok()
            .flatten()
    }
    /// Saves a downloaded area in one transaction, only while street maps are
    /// on and the workout that asked is still stored, so a download that ends
    /// after opting out, signing out or switching accounts leaves nothing.
    pub(crate) fn put_map_area(
        &self,
        activity: &str,
        (z, x, y): TileKey,
        source: &str,
        at: &str,
        tiles: &[(TileKey, Vec<u8>)],
    ) -> rusqlite::Result<bool> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        if !tx.query_row(&format!("SELECT {MAPS_ALLOWED}"), [activity], |r| r.get::<_, bool>(0))? {
            return Ok(false);
        }
        {
            let mut insert = tx.prepare("INSERT INTO map_tiles VALUES (?1,?2,?3,?4) ON CONFLICT(z,x,y) DO UPDATE SET data=excluded.data")?;
            for ((z, x, y), data) in tiles {
                insert.execute(params![z, x, y, data])?;
            }
        }
        tx.execute("INSERT INTO map_areas VALUES (?1,?2,?3,?4,?5) ON CONFLICT(z,x,y) DO UPDATE SET source=excluded.source,fetched_at=excluded.fetched_at", params![z, x, y, source, at])?;
        tx.commit()?;
        Ok(true)
    }
    /// Street maps are on and the workout is still stored.
    pub(crate) fn maps_allowed(&self, activity: &str) -> bool {
        let db = self.db.lock().unwrap();
        db.query_row(&format!("SELECT {MAPS_ALLOWED}"), [activity], |r| r.get(0)).unwrap_or(false)
    }
    pub(crate) fn route_map(&self, activity: &str) -> Option<String> {
        let db = self.db.lock().unwrap();
        db.query_row("SELECT json FROM route_maps WHERE activity=?1", [activity], |r| r.get(0))
            .optional()
            .ok()
            .flatten()
    }
    /// Keeps a drawn card under the same conditions as `put_map_area`.
    pub(crate) fn put_route_map(&self, activity: &str, json: &str) -> rusqlite::Result<bool> {
        let db = self.db.lock().unwrap();
        let changed = db.execute(
            &format!("INSERT INTO route_maps (activity,json) SELECT ?1, ?2 WHERE {MAPS_ALLOWED} ON CONFLICT(activity) DO UPDATE SET json=excluded.json"),
            params![activity, json],
        )?;
        Ok(changed != 0)
    }
    pub(crate) fn erase_maps(&self) -> rusqlite::Result<()> {
        self.db.lock().unwrap().execute_batch(ERASE_MAPS)
    }
    /// Rows in map_areas, map_tiles and route_maps.
    #[cfg(test)]
    pub(crate) fn map_rows(&self) -> (i64, i64, i64) {
        let db = self.db.lock().unwrap();
        let count = |table: &str| db.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0)).unwrap();
        (count("map_areas"), count("map_tiles"), count("route_maps"))
    }
}

/// A map tile's or area's zoom, x and y.
pub(crate) type TileKey = (u8, u32, u32);
const ERASE_MAPS: &str = "DELETE FROM map_tiles; DELETE FROM map_areas; DELETE FROM route_maps;";
/// Street maps are on (the default, so a settings row without the field
/// counts) and workout ?1 is stored.
const MAPS_ALLOWED: &str = "(NOT EXISTS (SELECT 1 FROM settings WHERE key='settings' AND json_extract(value,'$.streetMaps')=0) AND EXISTS (SELECT 1 FROM activities WHERE id=?1))";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn get_without_skips_only_the_named_top_level_fields() {
        let store = Store::open(":memory:").unwrap();
        let night = json!({"dailySleepDTO": {"hrvData": 1, "sleepTimeSeconds": 60}, "hrvData": [{"value": 50}], "sleepLevels": [1]});
        store.put("sleep", "d", "now", &night).unwrap();
        assert_eq!(
            store.get_without("sleep", "d", &["hrvData"]),
            Some(json!({"dailySleepDTO": {"hrvData": 1, "sleepTimeSeconds": 60}, "sleepLevels": [1]}))
        );
        assert_eq!(store.get_without("sleep", "d", &[]), store.get("sleep", "d"));
        // Rows that aren't objects come back whole.
        store.put("sleep", "null", "now", &Value::Null).unwrap();
        assert_eq!(store.get_without("sleep", "null", &["hrvData"]), Some(Value::Null));
        store.put("sleep", "list", "now", &json!([{"hrvData": 1}])).unwrap();
        assert_eq!(store.get_without("sleep", "list", &["hrvData"]), Some(json!([{"hrvData": 1}])));
        assert_eq!(store.get_without("sleep", "absent", &["hrvData"]), None);
    }
}
