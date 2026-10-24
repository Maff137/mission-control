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
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:mission-control.db", migrations())
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running Mission Control");
}

#[cfg(test)]
mod migration_tests;
