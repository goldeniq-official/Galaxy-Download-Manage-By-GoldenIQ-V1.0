// SPDX-License-Identifier: RPL-1.5
//! SQLite store (§08 data, D-01, D-02, N-04). WAL + synchronous=NORMAL. Migrations run in one transaction
//! each, after a backup, and are tracked in `user_version`; a database from a newer GDM opens read-only.
//! `quick_check` at open; a corrupt database is quarantined with its -wal/-shm and the newest backup that
//! passes `quick_check` is restored. Backups: `VACUUM INTO` a temporary file, check it, rename; keep 7.
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const KEEP_BACKUPS: usize = 7;
pub const BACKUP_EVERY: Duration = Duration::from_secs(24 * 3600);

const MIGRATIONS: &[&str] = &[
    // 1: G5 core schema
    r#"
    CREATE TABLE dirs(id INTEGER PRIMARY KEY, path TEXT NOT NULL UNIQUE);
    CREATE TABLE jobs(
        id TEXT PRIMARY KEY, url TEXT NOT NULL, final_url TEXT, referrer TEXT, name TEXT NOT NULL,
        dir_id INTEGER NOT NULL REFERENCES dirs(id), state TEXT NOT NULL, size INTEGER, done INTEGER NOT NULL DEFAULT 0,
        validator TEXT, sha256_expected TEXT, sha256 TEXT, epoch INTEGER NOT NULL DEFAULT 0,
        unverified INTEGER NOT NULL DEFAULT 0, protection TEXT, error_code TEXT, file_id TEXT,
        allow_private TEXT, created_at INTEGER NOT NULL, finished_at INTEGER, flags INTEGER NOT NULL DEFAULT 0);
    CREATE TABLE segments(
        job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE, start INTEGER NOT NULL, end_excl INTEGER NOT NULL,
        committed INTEGER NOT NULL, PRIMARY KEY(job_id, start),
        CHECK(start < end_excl AND start <= committed AND committed <= end_excl));
    CREATE TABLE names(dir_id INTEGER NOT NULL, name_key TEXT NOT NULL, job_id TEXT NOT NULL, UNIQUE(dir_id, name_key));
    CREATE TABLE history(id INTEGER PRIMARY KEY, job_id TEXT NOT NULL, at INTEGER NOT NULL, event TEXT NOT NULL, detail TEXT);
    CREATE INDEX history_job ON history(job_id);
    "#,
];
pub const SCHEMA: i64 = MIGRATIONS.len() as i64;

#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    pub id: String,
    pub url: String,
    pub final_url: Option<String>,
    pub referrer: Option<String>,
    pub name: String,
    pub dir: PathBuf,
    pub dir_id: i64,
    pub state: String,
    pub size: Option<u64>,
    pub done: u64,
    pub validator: Option<String>,
    pub sha256_expected: Option<String>,
    pub sha256: Option<String>,
    pub epoch: u64,
    pub unverified: bool,
    pub protection: Option<String>,
    pub error_code: Option<String>,
    pub file_id: Option<String>,
    pub allow_private: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct OpenReport {
    pub restored_from: Option<PathBuf>,
    pub quarantined: Option<PathBuf>,
    pub read_only: bool,
    pub migrated_from: Option<i64>,
    pub backup_made: Option<PathBuf>,
}

pub struct Store {
    pub conn: Connection,
    pub path: PathBuf,
    pub read_only: bool,
}

pub fn now() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64) }

fn err(e: impl std::fmt::Display) -> String { e.to_string() }

fn quick_ok(c: &Connection) -> bool {
    c.query_row("PRAGMA quick_check", [], |r| r.get::<_, String>(0)).is_ok_and(|s| s == "ok")
}

fn backup_dir(db: &Path) -> PathBuf { db.with_file_name("backups") }

/// newest first
pub fn backups(db: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(backup_dir(db)).into_iter().flatten().flatten().map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("gdm-") && n.ends_with(".db")))
        .collect();
    v.sort();
    v.reverse();
    v
}

fn sidecars(db: &Path) -> [PathBuf; 3] {
    let s = db.to_string_lossy();
    [db.to_path_buf(), PathBuf::from(format!("{s}-wal")), PathBuf::from(format!("{s}-shm"))]
}

impl Store {
    pub fn open(db: &Path) -> Result<(Store, OpenReport), String> {
        let mut rep = OpenReport::default();
        if let Some(d) = db.parent() { std::fs::create_dir_all(d).map_err(err)?; }
        let conn = match Self::try_open(db) {
            Some(c) => c,
            None => {
                // D-01: quarantine db + wal + shm together, then restore the newest backup that checks out
                let tag = format!("corrupt-{}", now());
                for p in sidecars(db) {
                    if p.exists() { std::fs::rename(&p, format!("{}.{tag}", p.display())).map_err(err)?; }
                }
                rep.quarantined = Some(PathBuf::from(format!("{}.{tag}", db.display())));
                for b in backups(db) {
                    let ok = Connection::open_with_flags(&b, OpenFlags::SQLITE_OPEN_READ_ONLY).is_ok_and(|c| quick_ok(&c));
                    if ok && std::fs::copy(&b, db).is_ok() { rep.restored_from = Some(b); break; }
                }
                Self::try_open(db).ok_or("database unusable after restore")?
            }
        };
        let mut st = Store { conn, path: db.to_path_buf(), read_only: false };
        let v: i64 = st.conn.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(err)?;
        if v > SCHEMA {
            // D-02: written by a newer GDM: never migrate down, never write
            st.read_only = true;
            rep.read_only = true;
            st.conn = Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(err)?;
            return Ok((st, rep));
        }
        if v > 0 && v < SCHEMA { rep.backup_made = Some(st.backup()?); }
        for (i, m) in MIGRATIONS.iter().enumerate().skip(v as usize) {
            let tx = st.conn.transaction().map_err(err)?;
            tx.execute_batch(m).map_err(err)?;
            tx.pragma_update(None, "user_version", (i + 1) as i64).map_err(err)?;
            tx.commit().map_err(err)?;
        }
        if v != SCHEMA { rep.migrated_from = Some(v); }
        if rep.backup_made.is_none() { rep.backup_made = st.backup_if_due()?; }
        Ok((st, rep))
    }

    fn try_open(db: &Path) -> Option<Connection> {
        let c = Connection::open(db).ok()?;
        c.busy_timeout(Duration::from_secs(10)).ok()?;
        c.pragma_update(None, "journal_mode", "WAL").ok()?;
        c.pragma_update(None, "synchronous", "NORMAL").ok()?;
        c.pragma_update(None, "foreign_keys", "ON").ok()?;
        quick_ok(&c).then_some(c)
    }

    /// VACUUM INTO a temporary file, quick_check it, then publish under its final name; keep 7
    pub fn backup(&self) -> Result<PathBuf, String> {
        let d = backup_dir(&self.path);
        std::fs::create_dir_all(&d).map_err(err)?;
        // fixed-width nanoseconds: the name sorts by time (two backups in one second must not swap order)
        let mut t = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos());
        while d.join(format!("gdm-{t:024}.db")).exists() { t += 1; }
        let name = format!("gdm-{t:024}.db");
        let fin = d.join(&name);
        let tmp = d.join(format!("{name}.tmp"));
        let _ = std::fs::remove_file(&tmp);
        self.conn.execute("VACUUM INTO ?1", params![tmp.to_string_lossy()]).map_err(err)?;
        let ok = Connection::open_with_flags(&tmp, OpenFlags::SQLITE_OPEN_READ_ONLY).is_ok_and(|c| quick_ok(&c));
        if !ok { let _ = std::fs::remove_file(&tmp); return Err("backup failed quick_check".into()); }
        std::fs::rename(&tmp, &fin).map_err(err)?;
        for old in backups(&self.path).into_iter().skip(KEEP_BACKUPS) { let _ = std::fs::remove_file(old); }
        Ok(fin)
    }

    /// daily: a backup when the newest one is older than 24 h (or there is none)
    pub fn backup_if_due(&self) -> Result<Option<PathBuf>, String> {
        if self.read_only { return Ok(None); }
        let due = backups(&self.path).first().and_then(|b| std::fs::metadata(b).ok()?.modified().ok())
            .is_none_or(|t| t.elapsed().unwrap_or_default() >= BACKUP_EVERY);
        if due { self.backup().map(Some) } else { Ok(None) }
    }

    pub fn dir_id(&self, dir: &Path) -> Result<i64, String> {
        let p = dir.to_string_lossy().to_string();
        self.conn.execute("INSERT OR IGNORE INTO dirs(path) VALUES (?1)", params![p]).map_err(err)?;
        self.conn.query_row("SELECT id FROM dirs WHERE path = ?1", params![p], |r| r.get(0)).map_err(err)
    }

    pub fn dirs(&self) -> Result<Vec<PathBuf>, String> {
        let mut q = self.conn.prepare("SELECT path FROM dirs").map_err(err)?;
        let v = q.query_map([], |r| r.get::<_, String>(0)).map_err(err)?.flatten().map(PathBuf::from).collect();
        Ok(v)
    }

    /// N-04: reserve a unique name (case-insensitive like Windows) and insert the job in ONE transaction.
    /// `taken(name)` reports a file already on disk (final name or its .part).
    pub fn insert_job(&mut self, j: &Job, taken: &dyn Fn(&str) -> bool) -> Result<String, String> {
        let tx = self.conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(err)?;
        let mut chosen = None;
        for n in 0..10_000u32 {
            let cand = if n == 0 { j.name.clone() } else { crate::name::with_counter(&j.name, n) };
            if taken(&cand) { continue; }
            if tx.execute("INSERT OR IGNORE INTO names(dir_id, name_key, job_id) VALUES (?1, ?2, ?3)", params![j.dir_id, name_key(&cand), j.id]).map_err(err)? == 1 {
                chosen = Some(cand);
                break;
            }
        }
        let name = chosen.ok_or("no free name")?;
        tx.execute(
            "INSERT INTO jobs(id, url, final_url, referrer, name, dir_id, state, size, done, validator, sha256_expected, epoch, unverified, allow_private, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![j.id, j.url, j.final_url, j.referrer, name, j.dir_id, j.state, j.size.map(|s| s as i64), j.done as i64, j.validator, j.sha256_expected, j.epoch as i64, j.unverified, j.allow_private, now()],
        ).map_err(err)?;
        tx.execute("INSERT INTO history(job_id, at, event) VALUES (?1, ?2, 'added')", params![j.id, now()]).map_err(err)?;
        tx.commit().map_err(err)?;
        Ok(name)
    }

    pub fn job(&self, id: &str) -> Result<Option<Job>, String> {
        self.conn.query_row(
            "SELECT j.id, j.url, j.final_url, j.referrer, j.name, d.path, j.dir_id, j.state, j.size, j.done, j.validator, j.sha256_expected, j.sha256,
                    j.epoch, j.unverified, j.protection, j.error_code, j.file_id, j.allow_private
             FROM jobs j JOIN dirs d ON d.id = j.dir_id WHERE j.id = ?1",
            params![id],
            |r| Ok(Job {
                id: r.get(0)?, url: r.get(1)?, final_url: r.get(2)?, referrer: r.get(3)?, name: r.get(4)?,
                dir: PathBuf::from(r.get::<_, String>(5)?), dir_id: r.get(6)?, state: r.get(7)?,
                size: r.get::<_, Option<i64>>(8)?.map(|v| v as u64), done: r.get::<_, i64>(9)? as u64, validator: r.get(10)?,
                sha256_expected: r.get(11)?, sha256: r.get(12)?, epoch: r.get::<_, i64>(13)? as u64, unverified: r.get(14)?,
                protection: r.get(15)?, error_code: r.get(16)?, file_id: r.get(17)?, allow_private: r.get(18)?,
            }),
        ).optional().map_err(err)
    }

    pub fn job_ids(&self) -> Result<Vec<String>, String> {
        let mut q = self.conn.prepare("SELECT id FROM jobs ORDER BY created_at, id").map_err(err)?;
        let v = q.query_map([], |r| r.get(0)).map_err(err)?.flatten().collect();
        Ok(v)
    }

    pub fn segments(&self, id: &str) -> Result<Vec<[u64; 3]>, String> {
        let mut q = self.conn.prepare("SELECT start, end_excl, committed FROM segments WHERE job_id = ?1 ORDER BY start").map_err(err)?;
        let v = q.query_map(params![id], |r| Ok([r.get::<_, i64>(0)? as u64, r.get::<_, i64>(1)? as u64, r.get::<_, i64>(2)? as u64]))
            .map_err(err)?.flatten().collect();
        Ok(v)
    }

    /// E-04: called only after FlushFileBuffers succeeded for every offset in `segs`
    pub fn save_checkpoint(&mut self, id: &str, final_url: &str, validator: Option<&str>, total: Option<u64>, epoch: u64, segs: &[[u64; 3]]) -> Result<(), String> {
        let tx = self.conn.transaction().map_err(err)?;
        tx.execute("DELETE FROM segments WHERE job_id = ?1", params![id]).map_err(err)?;
        {
            let mut ins = tx.prepare_cached("INSERT INTO segments(job_id, start, end_excl, committed) VALUES (?1, ?2, ?3, ?4)").map_err(err)?;
            for s in segs { ins.execute(params![id, s[0] as i64, s[1] as i64, s[2] as i64]).map_err(err)?; }
        }
        let done: u64 = segs.iter().map(|s| s[2] - s[0]).sum();
        tx.execute(
            "UPDATE jobs SET final_url = ?2, validator = ?3, size = ?4, epoch = ?5, done = ?6, unverified = ?7 WHERE id = ?1",
            params![id, final_url, validator, total.map(|t| t as i64), epoch as i64, done as i64, validator.is_none()],
        ).map_err(err)?;
        tx.commit().map_err(err)
    }

    pub fn set_state(&self, id: &str, state: &str, error: Option<&str>) -> Result<(), String> {
        self.conn.execute("UPDATE jobs SET state = ?2, error_code = ?3 WHERE id = ?1", params![id, state, error]).map_err(err)?;
        self.conn.execute("INSERT INTO history(job_id, at, event, detail) VALUES (?1, ?2, ?3, ?4)", params![id, now(), state, error]).map_err(err)?;
        Ok(())
    }

    pub fn set_file_id(&self, id: &str, fid: &str) -> Result<(), String> {
        self.conn.execute("UPDATE jobs SET file_id = ?2 WHERE id = ?1", params![id, fid]).map(|_| ()).map_err(err)
    }

    pub fn finish(&self, id: &str, state: &str, name: &str, sha256: &str, protection: &str) -> Result<(), String> {
        self.conn.execute(
            "UPDATE jobs SET state = ?2, name = ?3, sha256 = ?4, protection = ?5, finished_at = ?6, error_code = NULL WHERE id = ?1",
            params![id, state, name, sha256, protection, now()],
        ).map_err(err)?;
        self.conn.execute("INSERT INTO history(job_id, at, event, detail) VALUES (?1, ?2, ?3, ?4)", params![id, now(), state, sha256]).map_err(err)?;
        Ok(())
    }

    /// a renamed final file also owns its name in the directory
    pub fn claim_name(&self, dir_id: i64, name: &str, job: &str) -> Result<(), String> {
        self.conn.execute("INSERT OR REPLACE INTO names(dir_id, name_key, job_id) VALUES (?1, ?2, ?3)", params![dir_id, name_key(name), job]).map(|_| ()).map_err(err)
    }

    pub fn history(&self, id: &str) -> Result<Vec<(String, Option<String>)>, String> {
        let mut q = self.conn.prepare("SELECT event, detail FROM history WHERE job_id = ?1 ORDER BY id").map_err(err)?;
        let v = q.query_map(params![id], |r| Ok((r.get(0)?, r.get(1)?))).map_err(err)?.flatten().collect();
        Ok(v)
    }
}

/// Windows compares names case-insensitively: NFC + Unicode lower case (approximates the NTFS upcase table;
/// a collision the key misses still fails safely at create time with CREATE_NEW)
pub fn name_key(n: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    n.nfc().collect::<String>().to_lowercase()
}
