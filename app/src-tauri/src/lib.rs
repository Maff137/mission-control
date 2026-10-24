mod backup;
mod backup_commands;
use tauri::Manager;

fn migrations() -> Vec<tauri_plugin_sql::Migration> {
    vec![
        tauri_plugin_sql::Migration {
            version: 1,
            description: "create_persistence_test_records",
            sql: include_str!("../migrations/0001_persistence_test_records.sql"),
            kind: tauri_plugin_sql::MigrationKind::Up,
        },
        tauri_plugin_sql::Migration {
            version: 2,
            description: "add_test_record_note",
            sql: include_str!("../migrations/0002_add_test_record_note.sql"),
            kind: tauri_plugin_sql::MigrationKind::Up,
        },
    ]
}

#[cfg(all(feature = "native-acceptance", not(debug_assertions)))]
compile_error!("native-acceptance instrumentation requires a debug build");

#[cfg(feature = "native-acceptance")]
mod native_acceptance;

fn startup_message(error: &dyn std::fmt::Display) -> String {
    format!(
        "Mission Control could not open its native storage.\n\nYour data has not been reset. Keep the database and recovery files. Check storage permissions and available disk space, then reopen the application. If the problem persists, retain these details for support:\n\n{error}"
    )
}

fn initialize(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let directory = app.path().app_config_dir()?;
    #[cfg(feature = "native-acceptance")]
    native_acceptance::verify(app.config().identifier.as_str(), &directory)?;
    let mut store = backup::Store::acquire(directory)?;
    tauri::async_runtime::block_on(store.startup())?;
    app.manage(backup_commands::Storage::new(store));
    #[cfg(feature = "native-acceptance")]
    native_acceptance::checkpoint("before-sql");
    app.handle().plugin(
        tauri_plugin_sql::Builder::default()
            .add_migrations("sqlite:mission-control.db", migrations())
            .build(),
    )?;
    #[cfg(feature = "native-acceptance")]
    native_acceptance::checkpoint("before-ack");
    tauri::async_runtime::block_on(async {
        app.state::<backup_commands::Storage>()
            .lock()
            .await
            .finish_startup()
    })?;
    #[cfg(feature = "native-acceptance")]
    native_acceptance::checkpoint("after-ack");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut context = tauri::generate_context!();
    // Do not create a WebView or expose commands until native initialization succeeds.
    for window in &mut context.config_mut().app.windows {
        window.create = false;
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            backup_commands::export_backup,
            backup_commands::stage_restore,
            backup_commands::cancel_restore,
            backup_commands::confirm_restore,
            backup_commands::backup_status,
            backup_commands::read_records,
            backup_commands::insert_record,
        ])
        .setup(|app| {
            if let Err(error) = initialize(app) {
                let message = startup_message(&error);
                eprintln!("{message}");
                // A separate local page never loads the normal application's scripts.
                let details = serde_json::to_string(&message)?;
                tauri::WebviewWindowBuilder::new(
                    app,
                    "startup-error",
                    tauri::WebviewUrl::App("startup-error.html".into()),
                )
                .title("Mission Control — Startup error")
                .inner_size(640.0, 520.0)
                .initialization_script(format!(
                    "window.addEventListener('DOMContentLoaded', () => {{ document.getElementById('details').textContent = {details}; }});"
                ))
                .build()?;
                return Ok(());
            }
            for config in &app.config().app.windows {
                tauri::WebviewWindowBuilder::from_config(app.handle(), config)?.build()?;
            }
            Ok(())
        })
        .run(context)
        .expect("error while running Mission Control");
}

#[cfg(test)]
mod startup_tests {
    #[test]
    fn database_errors_include_write_recovery_guidance() {
        let error = crate::backup::Error::from(sqlx::Error::Io(std::io::Error::from(
            std::io::ErrorKind::StorageFull,
        )));
        assert!(error.message.contains("free disk space"));
        assert!(error.message.contains("keep database and recovery files"));
        assert!(error.message.contains("When restoring"));
    }

    #[test]
    fn startup_error_explains_preservation_and_recovery() {
        let message = super::startup_message(&"invalid_recovery: Invalid restore marker.");
        for required in [
            "not been reset",
            "recovery files",
            "permissions",
            "disk space",
            "reopen",
            "invalid_recovery",
        ] {
            assert!(message.contains(required));
        }
    }
}

#[cfg(test)]
mod migration_tests;

#[cfg(test)]
mod backup_tests;
