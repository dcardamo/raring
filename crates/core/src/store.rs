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
            "SELECT item.value FROM raw r,
             json_each(CASE WHEN r.endpoint='activities' AND json_type(r.json)='array' THEN r.json ELSE '[]' END) item
             WHERE CAST(json_extract(item.value, '$.activityId') AS TEXT)=?1
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
        self.db.lock().unwrap().execute_batch("DELETE FROM raw; DELETE FROM activities; DELETE FROM notifications; DELETE FROM cursors; DELETE FROM settings;")
    }
}
