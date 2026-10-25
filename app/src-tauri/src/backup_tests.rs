use super::backup::*;
use sqlx::{Connection, Executor, SqliteConnection};
use std::{
    fs,
    path::{Path, PathBuf},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("mc-b4-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(p.join("exports")).unwrap();
        Self(p)
    }
    fn store(&self, id: &str) -> Store {
        Store::acquire(self.0.join("storage").join(id)).unwrap()
    }
    fn export(&self, name: &str) -> PathBuf {
        self.0.join("exports").join(format!("{name}.mcbackup"))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
async fn database(store: &Store, rows: bool) {
    let mut db = SqliteConnection::connect_with(
        &sqlx::sqlite::SqliteConnectOptions::new()
            .filename(store.database())
            .create_if_missing(true),
    )
    .await
    .unwrap();
    migrator().run(&mut db).await.unwrap();
    if rows {
        db.execute("INSERT INTO persistence_test_records (id,value,created_at,note) VALUES ('a','first','2026-01-01T00:00:00.123Z','note 日本語'), ('b','quote''; --','2026-02-03T04:05:06.789Z','second note')").await.unwrap();
    }
    db.close().await.unwrap();
}
async fn contents(
    path: &Path,
) -> (
    Vec<(String, String, String, String)>,
    Vec<(i64, String, String, bool, Vec<u8>, i64)>,
) {
    let mut db = open(path, true, false).await.unwrap();
    let records =
        sqlx::query_as("SELECT id,value,created_at,note FROM persistence_test_records ORDER BY id")
            .fetch_all(&mut db)
            .await
            .unwrap();
    let history = sqlx::query_as("SELECT version,description,installed_on,success,checksum,execution_time FROM _sqlx_migrations ORDER BY version").fetch_all(&mut db).await.unwrap();
    db.close().await.unwrap();
    (records, history)
}
#[test]
fn snapshot_restore_exact_records_history_and_reopen() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let before = contents(&store.database()).await;
        let backup = f.export("records");
        assert!(store.export(Some(&backup)).await.unwrap());
        assert_eq!(contents(&backup).await, before);
        let mut db = open(&store.database(), false, false).await.unwrap();
        db.execute("DELETE FROM persistence_test_records")
            .await
            .unwrap();
        db.close().await.unwrap();
        assert!(store.stage(Some(&backup)).await.unwrap());
        store.confirm().await.unwrap();
        assert!(store.ready().is_err());
        let dir = store.dir.clone();
        drop(store);
        let mut store = Store::acquire(dir.clone()).unwrap();
        store.startup().await.unwrap();
        store.finish_startup().unwrap();
        assert_eq!(contents(&store.database()).await, before);
        drop(store);
        let mut reopened = Store::acquire(dir).unwrap();
        reopened.startup().await.unwrap();
        assert_eq!(contents(&reopened.database()).await, before);
    });
}
#[test]
fn empty_restore_repeated_attempts_and_cancellation() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, false).await;
        assert!(!store.export(None).await.unwrap());
        assert!(!store.stage(None).await.unwrap());
        assert!(store.confirm().await.is_err());
        let backup = f.export("empty");
        store.export(Some(&backup)).await.unwrap();
        store.stage(Some(&backup)).await.unwrap();
        let draft = store.draft.clone().unwrap();
        store.cancel().unwrap();
        assert!(!draft.exists());
        assert!(!store.dir.join("restore.pending").exists());
        for _ in 0..2 {
            store.stage(Some(&backup)).await.unwrap();
            store.confirm().await.unwrap();
            let dir = store.dir.clone();
            drop(store);
            store = Store::acquire(dir).unwrap();
            store.startup().await.unwrap();
            store.finish_startup().unwrap();
            assert!(contents(&store.database()).await.0.is_empty());
        }
    });
}
#[test]
fn invalid_backups_never_change_live_data() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let before = contents(&store.database()).await;
        for (name, bytes) in [
            ("text", b"not a database".to_vec()),
            ("truncated", b"SQLite format 3\0".repeat(40)),
            ("zero", vec![0; 4096]),
        ] {
            let path = f.export(name);
            fs::write(&path, bytes).unwrap();
            assert!(store.stage(Some(&path)).await.is_err());
            assert_eq!(contents(&store.database()).await, before);
        }
        assert!(store.stage(Some(&f.export("missing"))).await.is_err());
        let unrelated = f.export("unrelated");
        let mut db = SqliteConnection::connect_with(
            &sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&unrelated)
                .create_if_missing(true),
        )
        .await
        .unwrap();
        db.execute("CREATE TABLE unrelated (id INTEGER)")
            .await
            .unwrap();
        db.close().await.unwrap();
        assert!(store.stage(Some(&unrelated)).await.is_err());
        for (name, sql) in [
            (
                "future",
                "UPDATE _sqlx_migrations SET version=99 WHERE version=2",
            ),
            (
                "checksum",
                "UPDATE _sqlx_migrations SET checksum=X'00' WHERE version=1",
            ),
            (
                "trigger",
                "CREATE TRIGGER malicious AFTER INSERT ON persistence_test_records BEGIN DELETE FROM persistence_test_records; END",
            ),
            (
                "constraint",
                "PRAGMA writable_schema=ON; UPDATE sqlite_master SET sql=replace(sql, 'NOT NULL', '') WHERE name='persistence_test_records'; PRAGMA writable_schema=OFF;",
            ),
        ] {
            let path = f.export(name);
            store.export(Some(&path)).await.unwrap();
            let mut db = open(&path, false, false).await.unwrap();
            db.execute(sql).await.unwrap();
            db.close().await.unwrap();
            assert!(store.stage(Some(&path)).await.is_err(), "{name}");
            assert_eq!(contents(&store.database()).await, before);
        }
        let corrupt = f.export("corrupt");
        store.export(Some(&corrupt)).await.unwrap();
        let bytes = fs::read(&corrupt).unwrap();
        fs::write(&corrupt, &bytes[..bytes.len() / 2]).unwrap();
        assert!(store.stage(Some(&corrupt)).await.is_err());
        let oversized = f.export("oversized");
        let file = fs::File::create(&oversized).unwrap();
        file.set_len(MAX_SIZE + 1).unwrap();
        assert!(store.stage(Some(&oversized)).await.is_err());
        assert_eq!(contents(&store.database()).await, before);
        assert!(!store.dir.join("restore.pending").exists());
    });
}
#[test]
fn failed_exports_and_staged_tampering_are_safe() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let before = contents(&store.database()).await;
        assert!(store.export(Some(&store.database())).await.is_err());
        let existing = f.export("existing");
        fs::write(&existing, b"keep").unwrap();
        assert!(store.export(Some(&existing)).await.is_err());
        assert_eq!(fs::read(existing).unwrap(), b"keep");
        assert!(
            store
                .export(Some(&f.0.join("missing").join("file.mcbackup")))
                .await
                .is_err()
        );
        let backup = f.export("good");
        store.export(Some(&backup)).await.unwrap();
        store.stage(Some(&backup)).await.unwrap();
        fs::write(store.draft.as_ref().unwrap().join("incoming.db"), b"broken").unwrap();
        assert!(store.confirm().await.is_err());
        assert_eq!(contents(&store.database()).await, before);
        store.cancel().unwrap();
        store.stage(Some(&backup)).await.unwrap();
        store.confirm().await.unwrap();
        let dir = store.dir.clone();
        drop(store);
        let mut store = Store::acquire(dir).unwrap();
        let marker = fs::read_to_string(store.dir.join("restore.pending")).unwrap();
        fs::write(store.dir.join(marker).join("incoming.db"), b"broken").unwrap();
        store.startup().await.unwrap();
        assert!(store.status.contains("Restore rejected"));
        assert_eq!(contents(&store.database()).await, before);
    });
}
#[test]
fn wal_snapshot_and_interrupted_replacement_recover_original() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let mut active = open(&store.database(), false, false).await.unwrap();
        active.execute("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; INSERT INTO persistence_test_records(id,value,note) VALUES('wal','committed in WAL','wal note')").await.unwrap();
        let before = contents(&store.database()).await;
        let backup = f.export("wal");
        store.export(Some(&backup)).await.unwrap();
        assert_eq!(contents(&backup).await, before);
        active.close().await.unwrap();
        store.stage(Some(&backup)).await.unwrap();
        store.confirm().await.unwrap();
        let dir = store.dir.clone();
        drop(store);
        // Successful replacement interrupted before SQL plugin startup acknowledgement.
        let mut store = Store::acquire(dir.clone()).unwrap();
        store.startup().await.unwrap();
        drop(store);
        let mut store = Store::acquire(dir).unwrap();
        store.startup().await.unwrap();
        assert!(store.status.contains("recovered"));
        assert_eq!(contents(&store.database()).await, before);
        assert!(!store.dir.join("restore.pending").exists());
    });
}
#[test]
fn recovery_handles_each_interrupted_move_phase() {
    tauri::async_runtime::block_on(async {
        for phase in 0..3 {
            let f = Fixture::new();
            let mut store = f.store("dev");
            database(&store, true).await;
            let before = contents(&store.database()).await;
            let work = store.workspace().unwrap();
            snapshot(&store.database(), &work.join("original.db"), false)
                .await
                .unwrap();
            fs::write(work.join("installing"), b"1").unwrap();
            fs::write(
                store.dir.join("restore.pending"),
                work.file_name().unwrap().to_str().unwrap(),
            )
            .unwrap();
            if phase >= 1 {
                fs::rename(store.database(), work.join("original-live.db")).unwrap();
            }
            if phase >= 2 {
                fs::write(store.database(), b"partial replacement").unwrap();
                fs::write(store.dir.join("mission-control.db-wal"), b"stale").unwrap();
                fs::write(store.dir.join("mission-control.db-shm"), b"stale").unwrap();
                fs::write(store.dir.join("mission-control.db-journal"), b"stale").unwrap();
            }
            store.startup().await.unwrap();
            assert_eq!(contents(&store.database()).await, before);
            assert!(work.join("original.db").exists());
        }
    });
}
#[test]
fn storage_isolation_lock_and_interrupted_staging() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut dev = f.store("dev");
        let prod = f.store("production");
        database(&dev, true).await;
        database(&prod, false).await;
        assert!(Store::acquire(dev.dir.clone()).is_err());
        let backup = f.export("dev");
        dev.export(Some(&backup)).await.unwrap();
        dev.stage(Some(&backup)).await.unwrap();
        let dir = dev.dir.clone();
        drop(dev);
        let mut dev = Store::acquire(dir).unwrap();
        dev.startup().await.unwrap();
        assert_eq!(contents(&dev.database()).await.0.len(), 2);
        assert!(contents(&prod.database()).await.0.is_empty());
        assert!(
            dev.export(Some(&prod.dir.join("unsafe.mcbackup")))
                .await
                .is_err()
        );
    });
}

#[test]
fn interrupted_export_cleanup_preserves_published_backup() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let published = f.export("published");
        store.export(Some(&published)).await.unwrap();
        let bytes = fs::read(&published).unwrap();
        let work =
            f.0.join("exports")
                .join(format!(".mission-control-export-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&work).unwrap();
        fs::write(work.join("snapshot.mcbackup"), b"partial export").unwrap();
        fs::write(
            work.join("snapshot.mcbackup-journal"),
            b"interrupted journal",
        )
        .unwrap();
        fs::write(store.dir.join("export.pending"), work.to_str().unwrap()).unwrap();
        store.startup().await.unwrap();
        assert!(!work.exists());
        assert!(!store.dir.join("export.pending").exists());
        assert_eq!(fs::read(published).unwrap(), bytes);
    });
}

#[test]
fn invalid_recovery_snapshot_never_displaces_existing_database() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let before = contents(&store.database()).await;
        let work = store.workspace().unwrap();
        fs::write(work.join("original.db"), b"damaged recovery snapshot").unwrap();
        fs::write(work.join("installing"), b"1").unwrap();
        fs::write(
            store.dir.join("restore.pending"),
            work.file_name().unwrap().to_str().unwrap(),
        )
        .unwrap();
        assert!(store.startup().await.is_err());
        assert_eq!(contents(&store.database()).await, before);
        assert!(store.dir.join("restore.pending").exists());
    });
}

#[test]
fn sqlite_allocation_limits_reject_oversized_values() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let backup = f.export("limits");
        store.export(Some(&backup)).await.unwrap();
        let mut db = open(&backup, false, false).await.unwrap();
        sqlx::query("UPDATE persistence_test_records SET note=? WHERE id='a'")
            .bind("x".repeat(1_048_577))
            .execute(&mut db)
            .await
            .unwrap();
        db.close().await.unwrap();
        assert!(store.stage(Some(&backup)).await.is_err());
        assert_eq!(contents(&store.database()).await.0.len(), 2);
    });
}

#[test]
fn replacement_file_failure_rolls_back_and_allows_retry() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let before = contents(&store.database()).await;
        let backup = f.export("retry");
        store.export(Some(&backup)).await.unwrap();
        store.stage(Some(&backup)).await.unwrap();
        let work = store.draft.clone().unwrap();
        // Force the first original-file rename to fail, after the recovery snapshot
        // and installation marker are durable. Never inject into real app storage.
        fs::create_dir(work.join("original-live.db")).unwrap();
        store.confirm().await.unwrap();
        let dir = store.dir.clone();
        drop(store);
        let mut store = Store::acquire(dir.clone()).unwrap();
        store.startup().await.unwrap();
        assert!(store.status.contains("original data recovered"));
        assert_eq!(contents(&store.database()).await, before);
        assert!(work.join("original.db").exists());
        assert!(!dir.join("restore.pending").exists());
        store.stage(Some(&backup)).await.unwrap();
        store.confirm().await.unwrap();
        drop(store);
        let mut store = Store::acquire(dir).unwrap();
        store.startup().await.unwrap();
        store.finish_startup().unwrap();
        assert_eq!(contents(&store.database()).await, before);
    });
}

#[test]
fn live_sources_and_internal_storage_cannot_be_imported() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let before = contents(&store.database()).await;
        let external = f.export("standalone");
        store.export(Some(&external)).await.unwrap();
        let internal = store.dir.join("internal.mcbackup");
        copy_bounded(&external, &internal).unwrap();
        assert_eq!(
            store.stage(Some(&internal)).await.unwrap_err().code,
            "protected_source"
        );
        let mut writer = open(&external, false, false).await.unwrap();
        writer
            .execute(
                "PRAGMA journal_mode=WAL; UPDATE persistence_test_records SET note='active source'",
            )
            .await
            .unwrap();
        assert_eq!(
            store.stage(Some(&external)).await.unwrap_err().code,
            "active_backup"
        );
        writer.close().await.unwrap();
        assert_eq!(contents(&store.database()).await, before);
    });
}

#[path = "resilience_tests.rs"]
mod resilience_tests;

#[test]
fn b6_unicode_paths_and_no_clobber_publication_preserve_exact_data() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("unicode space 日本語");
        database(&store, true).await;
        let original = contents(&store.database()).await;
        let backup = f.export("backup space Ω");
        store.export(Some(&backup)).await.unwrap();
        assert_eq!(contents(&backup).await, original);
        let published = fs::read(&backup).unwrap();
        assert_eq!(
            store.export(Some(&backup)).await.unwrap_err().code,
            "destination_exists"
        );
        assert_eq!(fs::read(&backup).unwrap(), published);
        assert_eq!(contents(&store.database()).await, original);
        store.stage(Some(&backup)).await.unwrap();
        store.confirm().await.unwrap();
        let directory = store.dir.clone();
        drop(store);
        for _ in 0..2 {
            let mut reopened = Store::acquire(directory.clone()).unwrap();
            reopened.startup().await.unwrap();
            reopened.finish_startup().unwrap();
            assert_eq!(contents(&reopened.database()).await, original);
        }
    });
}
