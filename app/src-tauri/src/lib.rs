#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![tauri_plugin_sql::Migration {
        version: 1,
        description: "create_persistence_test_records",
        sql: include_str!("../migrations/0001_persistence_test_records.sql"),
        kind: tauri_plugin_sql::MigrationKind::Up,
    }];
    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:mission-control.db", migrations)
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running Mission Control");
}

#[cfg(test)]
mod tests {
    use sqlx::{
        Connection, Executor, SqliteConnection,
        migrate::{Migration, MigrationType, Migrator},
    };
    use std::borrow::Cow;

    #[test]
    fn migration_preserves_records_after_reopen_and_repeated_initialization() {
        tauri::async_runtime::block_on(async {
            let path =
                std::env::temp_dir().join(format!("mission-control-{}.db", std::process::id()));
            let url = format!("sqlite:{}?mode=rwc", path.display());
            let migration = Migration::new(
                1,
                Cow::Borrowed("create_persistence_test_records"),
                MigrationType::Simple,
                Cow::Borrowed(include_str!(
                    "../migrations/0001_persistence_test_records.sql"
                )),
                false,
            );
            let migrator = Migrator {
                migrations: Cow::Owned(vec![migration]),
                ..Migrator::DEFAULT
            };
            let mut db = SqliteConnection::connect(&url).await.unwrap();
            migrator.run(&mut db).await.unwrap();
            sqlx::query("INSERT INTO persistence_test_records (id, value) VALUES (?, ?)")
                .bind("stable-id")
                .bind("durable record")
                .execute(&mut db)
                .await
                .unwrap();
            assert!(
                sqlx::query(
                    "INSERT INTO persistence_test_records (id, value) VALUES ('invalid', '   ')"
                )
                .execute(&mut db)
                .await
                .is_err()
            );
            assert!(db.execute("SELECT * FROM missing_table").await.is_err());
            db.close().await.unwrap();
            let mut reopened = SqliteConnection::connect(&url).await.unwrap();
            migrator.run(&mut reopened).await.unwrap();
            let rows: Vec<(String, String, String)> =
                sqlx::query_as("SELECT id, value, created_at FROM persistence_test_records")
                    .fetch_all(&mut reopened)
                    .await
                    .unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].0, "stable-id");
            assert_eq!(rows[0].1, "durable record");
            assert!(rows[0].2.ends_with('Z'));
            reopened.close().await.unwrap();
            std::fs::remove_file(path).unwrap();
        });
    }
    #[test]
    fn failed_migration_rolls_back_without_resetting_existing_data() {
        tauri::async_runtime::block_on(async {
            let mut db = SqliteConnection::connect("sqlite::memory:").await.unwrap();
            db.execute("CREATE TABLE existing (value TEXT); INSERT INTO existing VALUES ('keep');")
                .await
                .unwrap();
            let migration = Migration::new(
                1,
                Cow::Borrowed("invalid"),
                MigrationType::Simple,
                Cow::Borrowed("CREATE TABLE partial (id TEXT); INSERT INTO missing VALUES (1);"),
                false,
            );
            let migrator = Migrator {
                migrations: Cow::Owned(vec![migration]),
                ..Migrator::DEFAULT
            };
            assert!(migrator.run(&mut db).await.is_err());
            let count: (i64,) =
                sqlx::query_as("SELECT count(*) FROM sqlite_master WHERE name = 'partial'")
                    .fetch_one(&mut db)
                    .await
                    .unwrap();
            assert_eq!(count.0, 0);
            let value: (String,) = sqlx::query_as("SELECT value FROM existing")
                .fetch_one(&mut db)
                .await
                .unwrap();
            assert_eq!(value.0, "keep");
        });
    }
}
