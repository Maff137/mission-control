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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri::plugin::Builder::<tauri::Wry>::new("backup-lifecycle")
                .setup(|app, _| {
                    let mut store = backup::Store::acquire(app.path().app_config_dir()?)?;
                    tauri::async_runtime::block_on(store.startup())?;
                    app.manage(backup_commands::Storage::new(store));
                    Ok(())
                })
                .build(),
        )
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
            tauri::async_runtime::block_on(async {
                app.state::<backup_commands::Storage>()
                    .lock()
                    .await
                    .finish_startup()
            })?;
            Ok(())
        })
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:mission-control.db", migrations())
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running Mission Control");
}

#[cfg(test)]
mod migration_tests;

#[cfg(test)]
mod backup_tests;
