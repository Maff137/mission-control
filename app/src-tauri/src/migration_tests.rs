use sqlx::{
    Connection, Executor, SqliteConnection,
    migrate::{MigrateError, Migration, Migrator},
};
use std::{
    borrow::Cow,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

// Each test owns a unique directory; no application data paths are used.
struct TestDatabase(PathBuf);
impl TestDatabase {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "mission-control-migrations-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    async fn open(&self) -> SqliteConnection {
        SqliteConnection::connect(&format!(
            "sqlite:{}?mode=rwc",
            self.0.join("test.db").display()
        ))
        .await
        .unwrap()
    }
}
impl Drop for TestDatabase {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn migrator(count: usize) -> Migrator {
    // Match the plugin's MigrationList conversion, including kind and transaction policy.
    Migrator {
        migrations: Cow::Owned(
            super::migrations()
                .into_iter()
                .take(count)
                .map(|m| {
                    Migration::new(
                        m.version,
                        m.description.into(),
                        m.kind.into(),
                        m.sql.into(),
                        false,
                    )
                })
                .collect(),
        ),
        ..Migrator::DEFAULT
    }
}
type Record = (String, String, String);
async fn records(db: &mut SqliteConnection) -> Vec<Record> {
    sqlx::query_as("SELECT id, value, created_at FROM persistence_test_records ORDER BY id")
        .fetch_all(db)
        .await
        .unwrap()
}
async fn seed(db: &mut SqliteConnection) -> Vec<Record> {
    for (id, value) in [
        ("first", "identifiable B2 record"),
        ("second", "Unicode: estudo 日本語"),
        ("third", "quote'; --"),
    ] {
        sqlx::query("INSERT INTO persistence_test_records (id, value) VALUES (?, ?)")
            .bind(id)
            .bind(value)
            .execute(&mut *db)
            .await
            .unwrap();
    }
    let rows = records(db).await;
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|r| r.2.ends_with('Z')));
    rows
}
async fn history(db: &mut SqliteConnection) -> Vec<(i64, String, bool, Vec<u8>, i64)> {
    sqlx::query_as("SELECT version, installed_on, success, checksum, execution_time FROM _sqlx_migrations ORDER BY version").fetch_all(db).await.unwrap()
}
async fn integrity(db: &mut SqliteConnection) {
    let result: Vec<(String,)> = sqlx::query_as("PRAGMA integrity_check")
        .fetch_all(&mut *db)
        .await
        .unwrap();
    assert_eq!(result, vec![("ok".into(),)]);
    assert!(
        sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&mut *db)
            .await
            .unwrap()
            .is_empty()
    );
    for sql in [
        "INSERT INTO persistence_test_records (id, value) VALUES ('blank', '   ')",
        "INSERT INTO persistence_test_records (id, value) VALUES (NULL, 'value')",
        "INSERT INTO persistence_test_records (id, value) VALUES ('null', NULL)",
    ] {
        assert!(db.execute(sql).await.is_err());
    }
}
async fn final_schema(db: &mut SqliteConnection) {
    let columns: Vec<(String, String, i64, Option<String>)> = sqlx::query_as("SELECT name, type, \"notnull\", dflt_value FROM pragma_table_info('persistence_test_records') ORDER BY cid").fetch_all(&mut *db).await.unwrap();
    assert_eq!(columns.len(), 4);
    assert_eq!(
        columns[3],
        ("note".into(), "TEXT".into(), 1, Some("''".into()))
    );
    assert!(sqlx::query("INSERT INTO persistence_test_records (id, value, note) VALUES ('null-note', 'value', NULL)").execute(&mut *db).await.is_err());
    integrity(db).await;
}

#[test]
fn existing_database_upgrade_and_repeated_initialization() {
    tauri::async_runtime::block_on(async {
        let file = TestDatabase::new();
        let mut db = file.open().await;
        migrator(1).run(&mut db).await.unwrap();
        let original = seed(&mut db).await;
        db.close().await.unwrap();
        let mut db = file.open().await;
        migrator(2).run(&mut db).await.unwrap();
        assert_eq!(records(&mut db).await, original);
        let notes: Vec<(String,)> = sqlx::query_as("SELECT note FROM persistence_test_records")
            .fetch_all(&mut db)
            .await
            .unwrap();
        assert_eq!(notes, vec![(String::new(),); 3]);
        final_schema(&mut db).await;
        db.execute("INSERT INTO persistence_test_records (id, value) VALUES ('new', 'B3 record')")
            .await
            .unwrap();
        db.execute("UPDATE persistence_test_records SET note = 'retained note' WHERE id = 'new'")
            .await
            .unwrap();
        let upgraded = records(&mut db).await;
        assert_eq!(upgraded.len(), 4);
        let applied = history(&mut db).await;
        assert_eq!(
            applied.iter().map(|r| (r.0, r.2)).collect::<Vec<_>>(),
            vec![(1, true), (2, true)]
        );
        db.close().await.unwrap();
        let mut db = file.open().await;
        migrator(2).run(&mut db).await.unwrap();
        assert_eq!(records(&mut db).await, upgraded);
        assert_eq!(history(&mut db).await, applied);
        let note: (String,) =
            sqlx::query_as("SELECT note FROM persistence_test_records WHERE id = 'new'")
                .fetch_one(&mut db)
                .await
                .unwrap();
        assert_eq!(note.0, "retained note");
        final_schema(&mut db).await;
        db.close().await.unwrap();
    });
}

#[test]
fn fresh_installation() {
    tauri::async_runtime::block_on(async {
        let file = TestDatabase::new();
        let mut db = file.open().await;
        migrator(2).run(&mut db).await.unwrap();
        assert!(records(&mut db).await.is_empty());
        assert_eq!(
            history(&mut db)
                .await
                .iter()
                .map(|r| (r.0, r.2))
                .collect::<Vec<_>>(),
            vec![(1, true), (2, true)]
        );
        final_schema(&mut db).await;
        db.close().await.unwrap();
    });
}

#[test]
fn failed_upgrade_rolls_back_and_recovers_after_reopen() {
    tauri::async_runtime::block_on(async {
        let file = TestDatabase::new();
        let mut db = file.open().await;
        migrator(1).run(&mut db).await.unwrap();
        let original = seed(&mut db).await;
        let applied = history(&mut db).await;
        db.close().await.unwrap();
        let mut db = file.open().await;
        let mut failing = migrator(2);
        failing.migrations.to_mut()[1] = Migration::new(2, "failing".into(), tauri_plugin_sql::MigrationKind::Up.into(), "ALTER TABLE persistence_test_records ADD COLUMN partial_note TEXT; CREATE TABLE partial (id TEXT); UPDATE persistence_test_records SET value = 'changed'; INSERT INTO missing_table VALUES (1);".into(), false);
        assert!(matches!(
            failing.run(&mut db).await,
            Err(MigrateError::ExecuteMigration(_, 2))
        ));
        db.close().await.unwrap();
        let mut db = file.open().await;
        assert_eq!(records(&mut db).await, original);
        assert_eq!(history(&mut db).await, applied);
        let partial: (i64,) =
            sqlx::query_as("SELECT count(*) FROM sqlite_master WHERE name = 'partial'")
                .fetch_one(&mut db)
                .await
                .unwrap();
        assert_eq!(partial.0, 0);
        let columns: (i64,) = sqlx::query_as("SELECT count(*) FROM pragma_table_info('persistence_test_records') WHERE name IN ('partial_note', 'note')").fetch_one(&mut db).await.unwrap();
        assert_eq!(columns.0, 0);
        integrity(&mut db).await;
        migrator(2).run(&mut db).await.unwrap();
        assert_eq!(records(&mut db).await, original);
        final_schema(&mut db).await;
        assert_eq!(history(&mut db).await.len(), 2);
        db.close().await.unwrap();
    });
}

#[test]
fn future_version_and_checksum_mismatch_fail_without_changes() {
    tauri::async_runtime::block_on(async {
        let file = TestDatabase::new();
        let mut db = file.open().await;
        migrator(2).run(&mut db).await.unwrap();
        let original = seed(&mut db).await;
        let applied = history(&mut db).await;
        assert!(matches!(
            migrator(1).run(&mut db).await,
            Err(MigrateError::VersionMissing(2))
        ));
        let mut changed = migrator(2);
        changed.migrations.to_mut()[0] = Migration::new(
            1,
            "changed".into(),
            tauri_plugin_sql::MigrationKind::Up.into(),
            "CREATE TABLE overwritten (id TEXT);".into(),
            false,
        );
        assert!(matches!(
            changed.run(&mut db).await,
            Err(MigrateError::VersionMismatch(1))
        ));
        db.close().await.unwrap();
        let mut db = file.open().await;
        assert_eq!(records(&mut db).await, original);
        assert_eq!(history(&mut db).await, applied);
        final_schema(&mut db).await;
        migrator(2).run(&mut db).await.unwrap();
        db.close().await.unwrap();
    });
}
