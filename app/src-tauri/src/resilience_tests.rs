//! B5 integration boundaries; reuses B4 temporary storage and exact snapshots.
use super::*;

// B5 helpers run only inside this Rust test executable, never in the packaged app.
#[test]
#[ignore = "invoked only by the B5 parent harness"]
fn b5_subprocess_worker() {
    let Ok(root) = std::env::var("MC_B5_CHILD_ROOT") else {
        return;
    };
    let root = PathBuf::from(root).canonicalize().unwrap();
    assert_eq!(
        root.parent().unwrap(),
        std::env::temp_dir().canonicalize().unwrap()
    );
    assert!(
        root.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("mc-b4-")
    );
    let phase = std::env::var("MC_B5_PHASE").unwrap();
    tauri::async_runtime::block_on(async {
        let mut store = Store::acquire(root.join("storage/dev")).unwrap();
        store.startup().await.unwrap();
        store.finish_startup().unwrap();
        let mut connection = None;
        match phase.as_str() {
            "wal" => {
                let mut db = open(&store.database(), false, false).await.unwrap();
                db.execute("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; INSERT INTO persistence_test_records(id,value,created_at,note) VALUES('committed','WAL commit','2026-01-02T03:04:05.678Z','durable note'); BEGIN IMMEDIATE; UPDATE persistence_test_records SET note='uncommitted'; INSERT INTO persistence_test_records(id,value) VALUES('uncommitted','must disappear')").await.unwrap();
                connection = Some(db);
            }
            "draft" | "confirmed" | "installed" => {
                store
                    .stage(Some(&root.join("exports/source.mcbackup")))
                    .await
                    .unwrap();
                if phase != "draft" {
                    store.confirm().await.unwrap();
                    assert!(store.ready().is_err());
                }
                if phase == "installed" {
                    drop(store);
                    store = Store::acquire(root.join("storage/dev")).unwrap();
                    store.startup().await.unwrap();
                    assert!(store.pending);
                }
            }
            "lock" => (),
            _ => panic!("unknown test phase"),
        }
        fs::write(root.join("ready"), b"ready").unwrap();
        // Keep Store and any open transaction alive until the parent kills this child.
        loop {
            std::hint::black_box((&store, &connection));
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
}

struct ChildGuard(std::process::Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn child_at_boundary(f: &Fixture, phase: &str) -> ChildGuard {
    let mut child = ChildGuard(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "backup_tests::resilience_tests::b5_subprocess_worker",
                "--ignored",
                "--nocapture",
            ])
            .env("MC_B5_CHILD_ROOT", &f.0)
            .env("MC_B5_PHASE", phase)
            .stdin(std::process::Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while !f.0.join("ready").exists() {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "child exited before {phase}"
        );
        assert!(
            std::time::Instant::now() < deadline,
            "child timed out at {phase}"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    child
}
async fn b5_integrity(path: &Path) {
    let mut db = open(path, true, false).await.unwrap();
    let check: Vec<(String,)> = sqlx::query_as("PRAGMA integrity_check")
        .fetch_all(&mut db)
        .await
        .unwrap();
    assert_eq!(check, vec![("ok".into(),)]);
    assert!(
        sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&mut db)
            .await
            .unwrap()
            .is_empty()
    );
    db.close().await.unwrap();
}

#[test]
fn b5_process_termination_releases_lock_and_recovers_each_boundary() {
    tauri::async_runtime::block_on(async {
        for phase in ["wal", "draft", "confirmed", "installed", "lock"] {
            let f = Fixture::new();
            let store = f.store("dev");
            database(&store, true).await;
            let original = contents(&store.database()).await;
            store.export(Some(&f.export("source"))).await.unwrap();
            // Make restoration observably different from rollback to the original.
            let mut db = open(&store.database(), false, false).await.unwrap();
            db.execute("UPDATE persistence_test_records SET note='live original, not backup'")
                .await
                .unwrap();
            db.close().await.unwrap();
            let mut expected = contents(&store.database()).await;
            let dir = store.dir.clone();
            drop(store);
            let mut child = child_at_boundary(&f, phase);
            let other = f.store("independent");
            database(&other, false).await;
            let isolated = contents(&other.database()).await;
            let recovery = if matches!(phase, "confirmed" | "installed") {
                Some(dir.join(fs::read_to_string(dir.join("restore.pending")).unwrap()))
            } else {
                None
            };
            assert_eq!(
                Store::acquire(dir.clone()).err().unwrap().code,
                "application_busy"
            );
            if phase == "wal" {
                expected.0.push((
                    "committed".into(),
                    "WAL commit".into(),
                    "2026-01-02T03:04:05.678Z".into(),
                    "durable note".into(),
                ));
                assert!(dir.join("mission-control.db-wal").metadata().unwrap().len() > 0);
            }
            child.0.kill().unwrap();
            assert!(!child.0.wait().unwrap().success());
            let mut recovered = Store::acquire(dir.clone()).unwrap();
            recovered.startup().await.unwrap();
            if phase == "confirmed" {
                assert!(recovered.pending);
                assert_eq!(contents(&recovered.database()).await, original);
                let work = dir.join(fs::read_to_string(dir.join("restore.pending")).unwrap());
                assert_eq!(contents(&work.join("original.db")).await, expected);
                b5_integrity(&work.join("original.db")).await;
            } else {
                assert_eq!(contents(&recovered.database()).await, expected, "{phase}");
            }
            if phase == "installed" {
                let work = recovery.as_ref().unwrap();
                assert!(recovered.status.contains("recovered"));
                assert_eq!(contents(&work.join("original.db")).await, expected);
                b5_integrity(&work.join("original.db")).await;
            }
            if phase == "draft" {
                assert!(fs::read_dir(&dir).unwrap().all(|entry| {
                    !entry
                        .unwrap()
                        .file_name()
                        .to_string_lossy()
                        .starts_with("restore-")
                }));
            }
            assert_eq!(contents(&other.database()).await, isolated);
            recovered.finish_startup().unwrap();
            b5_integrity(&recovered.database()).await;
            let stable = contents(&recovered.database()).await;
            drop(recovered);
            for _ in 0..2 {
                let mut reopened = Store::acquire(dir.clone()).unwrap();
                reopened.startup().await.unwrap();
                reopened.finish_startup().unwrap();
                assert_eq!(contents(&reopened.database()).await, stable, "{phase}");
                assert!(!dir.join("restore.pending").exists());
                b5_integrity(&reopened.database()).await;
            }
        }
    });
}

#[test]
fn b5_sqlite_full_readonly_and_busy_preserve_complete_data() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let store = f.store("dev");
        database(&store, true).await;
        let before = contents(&store.database()).await;
        let mut db = open(&store.database(), false, false).await.unwrap();
        let pages: (i64,) = sqlx::query_as("PRAGMA page_count")
            .fetch_one(&mut db)
            .await
            .unwrap();
        db.execute(format!("PRAGMA max_page_count={}", pages.0).as_str())
            .await
            .unwrap();
        let full = sqlx::query("INSERT INTO persistence_test_records(id,value) VALUES('full',?)")
            .bind("x".repeat(1_048_576))
            .execute(&mut db)
            .await
            .unwrap_err();
        assert_eq!(
            full.as_database_error().unwrap().code().as_deref(),
            Some("13")
        );
        db.execute("BEGIN IMMEDIATE; UPDATE persistence_test_records SET note='not committed'")
            .await
            .unwrap();
        let mut contender = SqliteConnection::connect_with(
            &sqlx::sqlite::SqliteConnectOptions::new()
                .filename(store.database())
                .busy_timeout(std::time::Duration::ZERO),
        )
        .await
        .unwrap();
        let busy = contender
            .execute("INSERT INTO persistence_test_records(id,value) VALUES('busy','blocked')")
            .await
            .unwrap_err();
        assert_eq!(
            busy.as_database_error().unwrap().code().as_deref(),
            Some("5")
        );
        contender.close().await.unwrap();
        db.execute("ROLLBACK").await.unwrap();
        db.close().await.unwrap();
        let mut readonly = open(&store.database(), true, false).await.unwrap();
        let error = readonly
            .execute("DELETE FROM persistence_test_records")
            .await
            .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("8")
        );
        readonly.close().await.unwrap();
        assert_eq!(contents(&store.database()).await, before);
        b5_integrity(&store.database()).await;
    });
}

#[cfg(unix)]
#[test]
fn b5_permission_denied_export_and_copy_allow_retry() {
    use std::os::unix::fs::PermissionsExt;
    struct PermissionsGuard(PathBuf);
    impl Drop for PermissionsGuard {
        fn drop(&mut self) {
            fs::set_permissions(&self.0, fs::Permissions::from_mode(0o700)).unwrap();
        }
    }
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let before = contents(&store.database()).await;
        let backup = f.export("source");
        store.export(Some(&backup)).await.unwrap();
        let denied = f.0.join("denied");
        fs::create_dir(&denied).unwrap();
        let permissions = PermissionsGuard(denied.clone());
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o500)).unwrap();
        let probe = fs::File::create(denied.join("probe")).unwrap_err();
        assert_eq!(probe.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(copy_bounded(&backup, &denied.join("copy.db")).is_err());
        assert!(
            store
                .export(Some(&denied.join("failed.mcbackup")))
                .await
                .is_err()
        );
        assert!(!denied.join("failed.mcbackup").exists());
        assert_eq!(contents(&store.database()).await, before);
        store.startup().await.unwrap();
        drop(permissions);
        store
            .export(Some(&denied.join("retry.mcbackup")))
            .await
            .unwrap();
        assert_eq!(contents(&denied.join("retry.mcbackup")).await, before);
        b5_integrity(&store.database()).await;
    });
}

#[test]
fn b5_retry_after_original_snapshot_and_completed_export_cleanup() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let backup = f.export("source");
        store.export(Some(&backup)).await.unwrap();
        let restored = contents(&backup).await;
        let mut db = open(&store.database(), false, false).await.unwrap();
        db.execute("UPDATE persistence_test_records SET note='original must survive'")
            .await
            .unwrap();
        db.close().await.unwrap();
        let original = contents(&store.database()).await;
        store.stage(Some(&backup)).await.unwrap();
        let work = store.draft.clone().unwrap();
        store.confirm().await.unwrap();
        // Interruption after the durable original snapshot, before installing marker.
        snapshot(&store.database(), &work.join("original.db"), false)
            .await
            .unwrap();
        copy_bounded(&work.join("incoming.db"), &work.join("candidate.db")).unwrap();
        let dir = store.dir.clone();
        drop(store);
        let mut store = Store::acquire(dir).unwrap();
        store.startup().await.unwrap();
        assert_eq!(contents(&store.database()).await, restored);
        assert_eq!(contents(&work.join("original.db")).await, original);
        let retained: Vec<_> = fs::read_dir(&work)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("previous-original-")
            })
            .collect();
        assert_eq!(retained.len(), 1);
        assert_eq!(contents(&retained[0]).await, original);
        store.finish_startup().unwrap();
        // A complete snapshot exists but publication has not happened.
        let export_work =
            f.0.join("exports")
                .join(format!(".mission-control-export-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&export_work).unwrap();
        snapshot(
            &store.database(),
            &export_work.join("snapshot.mcbackup"),
            true,
        )
        .await
        .unwrap();
        fs::write(
            store.dir.join("export.pending"),
            export_work.to_str().unwrap(),
        )
        .unwrap();
        store.startup().await.unwrap();
        assert!(!export_work.exists());
        assert_eq!(contents(&backup).await, restored);
        assert_eq!(contents(&store.database()).await, restored);
        b5_integrity(&store.database()).await;
    });
}

#[test]
fn b5_unsupported_backup_formats_and_invalid_markers_fail_closed() {
    tauri::async_runtime::block_on(async {
        let f = Fixture::new();
        let mut store = f.store("dev");
        database(&store, true).await;
        let before = contents(&store.database()).await;
        for (name, sql) in [
            ("future-format", "PRAGMA user_version=2"),
            ("old-format", "PRAGMA user_version=0"),
            ("wrong-app", "PRAGMA application_id=1"),
            (
                "old-schema",
                "ALTER TABLE persistence_test_records DROP COLUMN note",
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
        for marker in ["../outside", "restore-not-a-uuid", ""] {
            fs::write(store.dir.join("restore.pending"), marker).unwrap();
            assert!(store.startup().await.is_err());
            assert_eq!(contents(&store.database()).await, before);
            assert_eq!(
                fs::read_to_string(store.dir.join("restore.pending")).unwrap(),
                marker
            );
        }
        b5_integrity(&store.database()).await;
    });
}

#[test]
fn b5_injected_filesystem_failures_preserve_data_and_recover() {
    struct FaultGuard;
    impl Drop for FaultGuard {
        fn drop(&mut self) {
            TEST_FAULT.with(|fault| fault.set(None));
        }
    }
    tauri::async_runtime::block_on(async {
        for operation in [
            "copy",
            "copy_sync",
            "marker_write",
            "marker_sync",
            "marker_publish",
            "directory_sync",
            "export_publish",
            "install_rename",
        ] {
            let f = Fixture::new();
            let mut store = f.store("dev");
            database(&store, true).await;
            let backup = f.export("source");
            store.export(Some(&backup)).await.unwrap();
            let mut db = open(&store.database(), false, false).await.unwrap();
            db.execute(
                "UPDATE persistence_test_records SET note='original distinct from snapshot'",
            )
            .await
            .unwrap();
            db.close().await.unwrap();
            let original = contents(&store.database()).await;
            if !matches!(operation, "copy" | "copy_sync" | "export_publish") {
                store.stage(Some(&backup)).await.unwrap();
            }
            if operation == "install_rename" {
                store.confirm().await.unwrap();
                let dir = store.dir.clone();
                drop(store);
                store = Store::acquire(dir).unwrap();
            }
            let guard = FaultGuard;
            TEST_FAULT.with(|fault| fault.set(Some(operation)));
            match operation {
                "copy" | "copy_sync" => {
                    assert_eq!(
                        store.stage(Some(&backup)).await.unwrap_err().code,
                        "file_operation"
                    );
                    assert!(store.draft.is_none());
                    assert!(fs::read_dir(&store.dir).unwrap().all(|entry| {
                        !entry
                            .unwrap()
                            .file_name()
                            .to_string_lossy()
                            .starts_with("restore-")
                    }));
                }
                "export_publish" => {
                    let destination = f.export("unpublished");
                    assert_eq!(
                        store.export(Some(&destination)).await.unwrap_err().code,
                        "file_operation"
                    );
                    assert!(!destination.exists());
                    assert!(!store.dir.join("export.pending").exists());
                }
                "install_rename" => {
                    store.startup().await.unwrap();
                    assert!(store.status.contains("original data recovered"));
                }
                _ => {
                    assert_eq!(store.confirm().await.unwrap_err().code, "file_operation");
                    // A directory-sync failure can occur AFTER marker publication.
                    assert_eq!(store.pending, operation == "directory_sync");
                    assert_eq!(store.ready().is_err(), store.pending);
                }
            }
            TEST_FAULT
                .with(|fault| assert_eq!(fault.get(), None, "fault not reached: {operation}"));
            drop(guard);
            assert_eq!(contents(&store.database()).await, original, "{operation}");
            let dir = store.dir.clone();
            drop(store);
            let mut reopened = Store::acquire(dir.clone()).unwrap();
            reopened.startup().await.unwrap();
            reopened.finish_startup().unwrap();
            let expected = if operation == "directory_sync" {
                contents(&backup).await
            } else {
                original
            };
            assert_eq!(
                contents(&reopened.database()).await,
                expected,
                "{operation}"
            );
            assert!(!dir.join("restore.pending").exists());
            b5_integrity(&reopened.database()).await;
        }
    });
}
