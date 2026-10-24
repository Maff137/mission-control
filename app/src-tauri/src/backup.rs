//! B4: snapshots and restart-only restoration. No frontend-supplied paths.
use serde::Serialize;
use sqlx::{Connection, Executor, SqliteConnection, sqlite::SqliteConnectOptions};
use std::{
    borrow::Cow,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

// One-shot filesystem failures exist only in the Rust test executable.
#[cfg(test)]
thread_local! {
    pub static TEST_FAULT: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}
#[cfg(test)]
fn test_fault(operation: &str) -> Result<()> {
    TEST_FAULT.with(|fault| {
        if fault.get() == Some(operation) {
            fault.set(None);
            Err(std::io::Error::from(std::io::ErrorKind::StorageFull).into())
        } else {
            Ok(())
        }
    })
}

pub const MAX_SIZE: u64 = 64 * 1024 * 1024;
const APP_ID: i64 = 0x4d434250; // MCBP
const FORMAT: i64 = 1;
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Serialize)]
pub struct Error {
    pub code: &'static str,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        fail(
            "file_operation",
            format!("{e}. Check permissions and free disk space; keep recovery files."),
        )
    }
}
impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        fail(
            "database_validation",
            format!(
                "{e}. Check storage permissions and free disk space; keep database and recovery files. When restoring, select a compatible, intact Mission Control backup."
            ),
        )
    }
}
pub fn fail(code: &'static str, message: impl Into<String>) -> Error {
    Error {
        code,
        message: message.into(),
    }
}

pub fn migrator() -> sqlx::migrate::Migrator {
    sqlx::migrate::Migrator {
        migrations: Cow::Owned(
            super::migrations()
                .into_iter()
                .map(|m| {
                    sqlx::migrate::Migration::new(
                        m.version,
                        m.description.into(),
                        m.kind.into(),
                        m.sql.into(),
                        false,
                    )
                })
                .collect(),
        ),
        ..sqlx::migrate::Migrator::DEFAULT
    }
}
pub async fn open(path: &Path, readonly: bool, immutable: bool) -> Result<SqliteConnection> {
    Ok(SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(path)
            .read_only(readonly)
            .immutable(immutable)
            .create_if_missing(false)
            .busy_timeout(Duration::from_secs(3))
            .pragma("trusted_schema", "OFF")
            .pragma("temp_store", "MEMORY")
            .pragma("cache_size", "-2048")
            .statement_cache_capacity(0),
    )
    .await?)
}
type Schema = Vec<(String, String, String, Option<String>)>;
async fn schema(db: &mut SqliteConnection) -> Result<Schema> {
    Ok(sqlx::query_as(
        "SELECT type, name, tbl_name, sql FROM sqlite_schema ORDER BY type, name LIMIT 16",
    )
    .fetch_all(db)
    .await?)
}
pub async fn validate(path: &Path, backup: bool) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() < 512 || metadata.len() > MAX_SIZE {
        return Err(fail(
            "invalid_size",
            "Select a regular SQLite backup between 512 bytes and 64 MiB.",
        ));
    }
    let mut header = [0; 100];
    File::open(path)?.read_exact(&mut header)?;
    if &header[..16] != b"SQLite format 3\0" {
        return Err(fail(
            "invalid_format",
            "This is not a SQLite Mission Control backup.",
        ));
    }
    if backup
        && (i64::from(u32::from_be_bytes(header[68..72].try_into().unwrap())) != APP_ID
            || i64::from(u32::from_be_bytes(header[60..64].try_into().unwrap())) != FORMAT)
    {
        return Err(fail(
            "unsupported_format",
            "Expected Mission Control backup format 1 (.mcbackup).",
        ));
    }
    let mut db = open(path, true, true).await?;
    {
        let mut handle = db.lock_handle().await?;
        // SQLx owns this connection and the guard excludes all concurrent SQLite calls.
        // Limits bound allocations/compilation before reading any untrusted schema/data.
        unsafe {
            let raw = handle.as_raw_handle().as_ptr();
            for (category, limit) in [
                (libsqlite3_sys::SQLITE_LIMIT_LENGTH, 1_048_576),
                (libsqlite3_sys::SQLITE_LIMIT_SQL_LENGTH, 16_384),
                (libsqlite3_sys::SQLITE_LIMIT_COLUMN, 64),
                (libsqlite3_sys::SQLITE_LIMIT_EXPR_DEPTH, 32),
                (libsqlite3_sys::SQLITE_LIMIT_COMPOUND_SELECT, 8),
                (libsqlite3_sys::SQLITE_LIMIT_ATTACHED, 0),
                (libsqlite3_sys::SQLITE_LIMIT_TRIGGER_DEPTH, 0),
                (libsqlite3_sys::SQLITE_LIMIT_VARIABLE_NUMBER, 64),
                (libsqlite3_sys::SQLITE_LIMIT_WORKER_THREADS, 0),
            ] {
                libsqlite3_sys::sqlite3_limit(raw, category, limit);
            }
            libsqlite3_sys::sqlite3_db_config(
                raw,
                libsqlite3_sys::SQLITE_DBCONFIG_DEFENSIVE,
                1,
                std::ptr::null_mut::<i32>(),
            );
        }
    }
    let started = Instant::now();
    db.lock_handle()
        .await?
        .set_progress_handler(1000, move || started.elapsed() < Duration::from_secs(10));
    let mut expected = SqliteConnection::connect("sqlite::memory:").await?;
    migrator()
        .run(&mut expected)
        .await
        .map_err(|e| fail("internal_schema", e.to_string()))?;
    // Reject all unknown schema objects BEFORE querying user tables or integrity checks.
    if schema(&mut db).await? != schema(&mut expected).await? {
        return Err(fail(
            "incompatible_schema",
            "The backup schema differs from this application. Unknown tables, triggers, views and constraints are rejected.",
        ));
    }
    let history: Vec<(i64, String, bool, Vec<u8>)> = sqlx::query_as("SELECT version, description, success, checksum FROM _sqlx_migrations ORDER BY version LIMIT 100").fetch_all(&mut db).await?;
    let expected_history: Vec<_> = migrator()
        .iter()
        .map(|m| {
            (
                m.version,
                m.description.to_string(),
                true,
                m.checksum.to_vec(),
            )
        })
        .collect();
    if history != expected_history {
        return Err(fail(
            "incompatible_migrations",
            "Migration versions, descriptions or checksums are incompatible. Future versions cannot be restored.",
        ));
    }
    let invalid: (i64,) = sqlx::query_as(
        "SELECT count(*) FROM persistence_test_records WHERE typeof(id) != 'text' OR typeof(value) != 'text' OR typeof(created_at) != 'text' OR typeof(note) != 'text' OR length(CAST(id AS BLOB)) + length(CAST(value AS BLOB)) + length(CAST(created_at AS BLOB)) + length(CAST(note AS BLOB)) > 1000000"
    ).fetch_one(&mut db).await?;
    if invalid.0 != 0 {
        return Err(fail(
            "invalid_records",
            "Backup records must contain text fields with at most 1,000,000 combined UTF-8 bytes per record.",
        ));
    }
    let integrity: Vec<(String,)> = sqlx::query_as("PRAGMA integrity_check(1)")
        .fetch_all(&mut db)
        .await?;
    if integrity != vec![("ok".into(),)]
        || !sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&mut db)
            .await?
            .is_empty()
    {
        return Err(fail(
            "corrupt_database",
            "SQLite integrity or foreign-key validation failed.",
        ));
    }
    db.close().await?;
    expected.close().await?;
    Ok(())
}
fn sync_dir(path: &Path) -> Result<()> {
    #[cfg(test)]
    test_fault("directory_sync")?;
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
fn durable_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap();
    let temporary = parent.join(format!(".marker-{}", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        #[cfg(test)]
        test_fault("marker_write")?;
        file.write_all(bytes)?;
        #[cfg(test)]
        test_fault("marker_sync")?;
        file.sync_all()?;
        // Atomic no-clobber publication. A partial marker is never visible.
        #[cfg(test)]
        test_fault("marker_publish")?;
        fs::hard_link(&temporary, path)?;
        sync_dir(parent)
    })();
    let _ = fs::remove_file(temporary);
    result
}
fn private_file(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
pub fn copy_bounded(source: &Path, target: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(source)?;
    if !meta.is_file() || meta.len() > MAX_SIZE {
        return Err(fail(
            "invalid_size",
            "Select a regular backup no larger than 64 MiB.",
        ));
    }
    let mut input = File::open(source)?.take(MAX_SIZE + 1);
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(target)?;
    #[cfg(test)]
    test_fault("copy")?;
    let count = std::io::copy(&mut input, &mut output)?;
    if count > MAX_SIZE {
        return Err(fail("invalid_size", "Backup grew beyond the 64 MiB limit."));
    }
    private_file(target)?;
    #[cfg(test)]
    test_fault("copy_sync")?;
    output.sync_all()?;
    sync_dir(target.parent().unwrap())
}
pub async fn snapshot(source: &Path, target: &Path, backup: bool) -> Result<()> {
    let mut db = open(source, true, false).await?;
    sqlx::query("VACUUM INTO ?")
        .bind(
            target
                .to_str()
                .ok_or_else(|| fail("invalid_path", "The selected path must be valid Unicode."))?,
        )
        .execute(&mut db)
        .await?;
    db.close().await?;
    if backup {
        let mut output = open(target, false, false).await?;
        output
            .execute(
                format!("PRAGMA application_id={APP_ID}; PRAGMA user_version={FORMAT};").as_str(),
            )
            .await?;
        output.close().await?;
    }
    private_file(target)?;
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(target)?
        .sync_all()?;
    sync_dir(target.parent().unwrap())?;
    validate(target, backup).await
}

pub struct Store {
    pub dir: PathBuf,
    pub draft: Option<PathBuf>,
    pub pending: bool,
    pub status: String,
    _lock: File,
}
impl Store {
    pub fn acquire(dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&dir)?;
        let lock_path = dir.join("backup.lock");
        if lock_path.is_symlink() {
            return Err(fail(
                "unsafe_storage",
                "Application lock must not be a symbolic link.",
            ));
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;
        lock.try_lock().map_err(|_| {
            fail(
                "application_busy",
                "Another Mission Control instance is using this storage. Quit it first.",
            )
        })?;
        Ok(Self {
            dir: dir.canonicalize()?,
            draft: None,
            pending: false,
            status: String::new(),
            _lock: lock,
        })
    }
    pub fn database(&self) -> PathBuf {
        self.dir.join("mission-control.db")
    }
    pub fn ready(&self) -> Result<()> {
        if self.pending {
            Err(fail(
                "restart_required",
                "Restore is confirmed. Quit and reopen the application before writing data.",
            ))
        } else {
            Ok(())
        }
    }
    pub fn workspace(&self) -> Result<PathBuf> {
        let path = self.dir.join(format!("restore-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
        }
        sync_dir(&self.dir)?;
        Ok(path)
    }
    pub async fn stage(&mut self, source: Option<&Path>) -> Result<bool> {
        self.ready()?;
        self.cancel()?;
        let Some(source) = source else {
            return Ok(false);
        };
        let canonical = source.canonicalize()?;
        if canonical.starts_with(self.dir.parent().unwrap()) {
            return Err(fail(
                "protected_source",
                "Select an exported .mcbackup outside application storage. Do not import a live database or internal recovery files.",
            ));
        }
        if source.extension().and_then(|v| v.to_str()) != Some("mcbackup") {
            return Err(fail(
                "invalid_extension",
                "Select an exported .mcbackup file.",
            ));
        }
        for suffix in ["-wal", "-shm", "-journal"] {
            let mut sidecar = source.as_os_str().to_os_string();
            sidecar.push(suffix);
            if Path::new(&sidecar).exists() {
                return Err(fail(
                    "active_backup",
                    "The selected file has SQLite sidecars. Close its writer and export a standalone Mission Control backup.",
                ));
            }
        }
        let work = self.workspace()?;
        let result = async {
            copy_bounded(source, &work.join("incoming.db"))?;
            validate(&work.join("incoming.db"), true).await
        }
        .await;
        if let Err(error) = result {
            let _ = fs::remove_dir_all(&work);
            return Err(error);
        }
        self.draft = Some(work);
        Ok(true)
    }
    pub fn cancel(&mut self) -> Result<()> {
        self.ready()?;
        if let Some(path) = self.draft.take() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }
    pub async fn confirm(&mut self) -> Result<()> {
        self.ready()?;
        let work = self
            .draft
            .as_ref()
            .ok_or_else(|| fail("no_selection", "Select and validate a backup first."))?;
        validate(&work.join("incoming.db"), true).await?;
        let marker = self.dir.join("restore.pending");
        let publication = durable_write(
            &marker,
            work.file_name().unwrap().to_str().unwrap().as_bytes(),
        );
        self.pending = marker.exists();
        publication?;
        self.draft = None;
        Ok(())
    }
    pub async fn export(&self, destination: Option<&Path>) -> Result<bool> {
        self.ready()?;
        let Some(destination) = destination else {
            return Ok(false);
        };
        let parent = destination
            .parent()
            .ok_or_else(|| fail("invalid_destination", "Choose a backup folder."))?
            .canonicalize()?;
        // All Mission Control application-managed siblings (including dev/prod) are protected.
        if parent.starts_with(self.dir.parent().unwrap()) {
            return Err(fail(
                "protected_destination",
                "Choose a backup folder outside application storage, such as Documents.",
            ));
        }
        if destination.extension().and_then(|x| x.to_str()) != Some("mcbackup") {
            return Err(fail("invalid_extension", "Use the .mcbackup extension."));
        }
        if destination.exists() || destination.is_symlink() {
            return Err(fail(
                "destination_exists",
                "Choose a new filename. Existing files are never overwritten.",
            ));
        }
        self.cleanup_export()?;
        let work = parent.join(format!(".mission-control-export-{}", uuid::Uuid::new_v4()));
        durable_write(
            &self.dir.join("export.pending"),
            work.to_str()
                .ok_or_else(|| fail("invalid_path", "Choose a Unicode backup path."))?
                .as_bytes(),
        )?;
        fs::create_dir(&work)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&work, fs::Permissions::from_mode(0o700))?;
        }
        let temp = work.join("snapshot.mcbackup");
        let result = async {
            snapshot(&self.database(), &temp, true).await?;
            #[cfg(test)]
            test_fault("export_publish")?;
            fs::hard_link(&temp, destination)?;
            sync_dir(&parent)?;
            Ok(true)
        }
        .await;
        let cleanup = self.cleanup_export();
        match result {
            Err(error) => Err(error),
            Ok(value) => {
                cleanup?;
                Ok(value)
            }
        }
    }
    fn cleanup_export(&self) -> Result<()> {
        let marker = self.dir.join("export.pending");
        if !marker.exists() {
            return Ok(());
        }
        if fs::metadata(&marker)?.len() > 8192 {
            return Err(fail(
                "invalid_export_marker",
                "Export cleanup marker is invalid; keep application files.",
            ));
        }
        let work = PathBuf::from(fs::read_to_string(&marker)?);
        let name = work
            .file_name()
            .and_then(|v| v.to_str())
            .and_then(|v| v.strip_prefix(".mission-control-export-"));
        if name.and_then(|v| uuid::Uuid::parse_str(v).ok()).is_none()
            || !work.is_absolute()
            || work.is_symlink()
        {
            return Err(fail(
                "invalid_export_marker",
                "Export cleanup path is invalid; keep application files.",
            ));
        }
        if work.exists() {
            // Never recursively delete: only the single snapshot created by this service.
            for suffix in ["", "-wal", "-shm", "-journal"] {
                let snapshot = work.join(format!("snapshot.mcbackup{suffix}"));
                if snapshot.exists() || snapshot.is_symlink() {
                    fs::remove_file(snapshot)?;
                }
            }
            fs::remove_dir(&work)?;
            sync_dir(work.parent().unwrap())?;
        }
        fs::remove_file(marker)?;
        sync_dir(&self.dir)
    }
    fn cleanup_drafts(&self, pending: Option<&Path>) -> Result<()> {
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if !entry.file_type()?.is_dir() || Some(path.as_path()) == pending {
                continue;
            }
            let name = entry.file_name();
            let id = name.to_str().and_then(|v| v.strip_prefix("restore-"));
            if id.and_then(|v| uuid::Uuid::parse_str(v).ok()).is_none() {
                continue;
            }
            let children: Vec<_> = fs::read_dir(&path)?.collect::<std::io::Result<_>>()?;
            if children.iter().all(|child| {
                child.file_name() == "incoming.db" && child.file_type().is_ok_and(|t| t.is_file())
            }) {
                for child in children {
                    fs::remove_file(child.path())?;
                }
                fs::remove_dir(path)?;
            }
        }
        sync_dir(&self.dir)
    }
    fn transaction(&self) -> Result<Option<PathBuf>> {
        let pointer = self.dir.join("restore.pending");
        if !pointer.exists() {
            return Ok(None);
        }
        if fs::metadata(&pointer)?.len() > 128 {
            return Err(fail(
                "invalid_recovery",
                "Invalid restore marker. Keep all recovery files.",
            ));
        }
        let name = fs::read_to_string(pointer)?;
        let id = name
            .strip_prefix("restore-")
            .ok_or_else(|| fail("invalid_recovery", "Invalid restore marker."))?;
        uuid::Uuid::parse_str(id)
            .map_err(|_| fail("invalid_recovery", "Invalid restore identifier."))?;
        let work = self.dir.join(name);
        if work.is_symlink() {
            return Err(fail(
                "invalid_recovery",
                "Recovery directory must not be a symbolic link.",
            ));
        }
        Ok(Some(work))
    }
    fn move_live(&self, work: &Path, prefix: &str) -> Result<()> {
        for suffix in ["", "-wal", "-shm", "-journal"] {
            let path = self.dir.join(format!("mission-control.db{suffix}"));
            if path.exists() {
                fs::rename(path, work.join(format!("{prefix}{suffix}")))?;
                #[cfg(feature = "native-acceptance")]
                if prefix == "original-live.db" && suffix.is_empty() {
                    crate::native_acceptance::checkpoint("original-displaced");
                }
            }
        }
        sync_dir(work)?;
        sync_dir(&self.dir)
    }
    async fn rollback(&mut self, work: &Path) -> Result<()> {
        validate(&work.join("original.db"), false).await?;
        let temp = work.join(format!("rollback-{}.db", uuid::Uuid::new_v4()));
        copy_bounded(&work.join("original.db"), &temp)?;
        self.move_live(work, &format!("failed-{}", uuid::Uuid::new_v4()))?;
        fs::rename(temp, self.database())?;
        sync_dir(&self.dir)?;
        validate(&self.database(), false).await?;
        fs::remove_file(self.dir.join("restore.pending"))?;
        sync_dir(&self.dir)?;
        self.status =
            "Interrupted restore recovered the original data. Recovery files were retained.".into();
        Ok(())
    }
    /// Called only by the first plugin, before SQL plugin setup or migrations.
    pub async fn startup(&mut self) -> Result<()> {
        self.cleanup_export()?;
        let transaction = self.transaction()?;
        self.cleanup_drafts(transaction.as_deref())?;
        let Some(work) = transaction else {
            return Ok(());
        };
        if work.join("installing").exists() {
            return self.rollback(&work).await;
        }
        if let Err(error) = validate(&work.join("incoming.db"), true).await {
            fs::rename(
                self.dir.join("restore.pending"),
                work.join("restore.rejected"),
            )?;
            sync_dir(&self.dir)?;
            self.status = format!(
                "Restore rejected before replacement: {error}. Original data was not replaced."
            );
            return Ok(());
        }
        // VACUUM includes committed WAL data and recovers a hot journal using SQLite.
        // Use a writable connection for hot-journal recovery before the read-only snapshot.
        let mut db = open(&self.database(), false, false).await?;
        db.execute("SELECT count(*) FROM sqlite_schema").await?;
        db.close().await?;
        let original = work.join("original.db");
        if original.exists() {
            fs::rename(
                &original,
                work.join(format!("previous-original-{}.db", uuid::Uuid::new_v4())),
            )?;
        }
        #[cfg(feature = "native-acceptance")]
        crate::native_acceptance::checkpoint("before-snapshot");
        snapshot(&self.database(), &original, false).await?;
        #[cfg(feature = "native-acceptance")]
        crate::native_acceptance::checkpoint("after-snapshot");
        let candidate = work.join("candidate.db");
        if candidate.exists() {
            fs::remove_file(&candidate)?;
        }
        copy_bounded(&work.join("incoming.db"), &candidate)?;
        durable_write(&work.join("installing"), b"1")?;
        let result = async {
            self.move_live(&work, "original-live.db")?;
            #[cfg(test)]
            test_fault("install_rename")?;
            #[cfg(feature = "native-acceptance")]
            crate::native_acceptance::checkpoint("before-install");
            fs::rename(candidate, self.database())?;
            #[cfg(feature = "native-acceptance")]
            crate::native_acceptance::checkpoint("after-install");
            sync_dir(&self.dir)?;
            validate(&self.database(), true).await
        }
        .await;
        if let Err(error) = result {
            self.rollback(&work).await?;
            self.status = format!("Restore failed; original data recovered: {error}");
            return Ok(());
        }
        self.pending = true;
        self.status = "Backup restored successfully. Original data remains in application-managed recovery storage.".into();
        Ok(())
    }
    /// Called after SQL plugin preload/migrations succeed. Until then, interruption rolls back.
    pub fn finish_startup(&mut self) -> Result<()> {
        if self.pending {
            fs::remove_file(self.dir.join("restore.pending"))?;
            sync_dir(&self.dir)?;
            self.pending = false;
        }
        Ok(())
    }
}
