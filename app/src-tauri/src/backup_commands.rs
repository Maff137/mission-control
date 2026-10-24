use crate::backup::{Error, Result, Store, fail};
use std::path::PathBuf;
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::Mutex;
pub type Storage = Mutex<Store>;

async fn choose(app: tauri::AppHandle, save: bool) -> Result<Option<PathBuf>> {
    tauri::async_runtime::spawn_blocking(move || {
        let dialog = app
            .dialog()
            .file()
            .add_filter("Mission Control backup", &["mcbackup"]);
        let selected = if save {
            dialog
                .set_file_name("mission-control.mcbackup")
                .blocking_save_file()
        } else {
            dialog.blocking_pick_file()
        };
        selected
            .map(|p| {
                p.into_path()
                    .map_err(|_| fail("invalid_path", "Select a local file."))
            })
            .transpose()
    })
    .await
    .map_err(|e| fail("dialog_failed", e.to_string()))?
}
// Keep SQLx's borrowed Executor futures on a blocking worker.
// The WebView and main thread remain available during file and database operations.
#[tauri::command]
pub async fn export_backup(app: tauri::AppHandle) -> Result<bool> {
    tauri::async_runtime::spawn_blocking(move || {
        tauri::async_runtime::block_on(async {
            let state = app.state::<Storage>();
            let store = state.lock().await;
            store.ready()?;
            let path = choose(app.clone(), true).await?;
            store.export(path.as_deref()).await
        })
    })
    .await
    .map_err(|e| fail("operation_failed", e.to_string()))?
}
#[tauri::command]
pub async fn stage_restore(app: tauri::AppHandle) -> Result<bool> {
    tauri::async_runtime::spawn_blocking(move || {
        tauri::async_runtime::block_on(async {
            let state = app.state::<Storage>();
            let mut store = state.lock().await;
            store.ready()?;
            let path = choose(app.clone(), false).await?;
            store.stage(path.as_deref()).await
        })
    })
    .await
    .map_err(|e| fail("operation_failed", e.to_string()))?
}
#[tauri::command]
pub async fn cancel_restore(state: State<'_, Storage>) -> Result<()> {
    state.lock().await.cancel()
}
#[tauri::command]
pub async fn confirm_restore(app: tauri::AppHandle) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        tauri::async_runtime::block_on(async {
            app.state::<Storage>().lock().await.confirm().await
        })
    })
    .await
    .map_err(|e| fail("operation_failed", e.to_string()))?
}
#[tauri::command]
pub async fn backup_status(state: State<'_, Storage>) -> Result<String> {
    Ok(state.lock().await.status.clone())
}
#[derive(serde::Serialize)]
pub struct Record {
    id: String,
    value: String,
    created_at: String,
}
#[tauri::command]
pub async fn read_records(app: tauri::AppHandle, state: State<'_, Storage>) -> Result<Vec<Record>> {
    let store = state.lock().await;
    store.ready()?;
    let instances = app.state::<tauri_plugin_sql::DbInstances>();
    let pools = instances.0.read().await;
    let Some(tauri_plugin_sql::DbPool::Sqlite(pool)) = pools.get("sqlite:mission-control.db")
    else {
        return Err(fail(
            "database_unavailable",
            "Restart the application; keep all database files.",
        ));
    };
    let rows: Vec<(String, String, String)> = sqlx::query_as("SELECT id, value, created_at FROM persistence_test_records ORDER BY created_at ASC, id ASC").fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|(id, value, created_at)| Record {
            id,
            value,
            created_at,
        })
        .collect())
}
#[tauri::command]
pub async fn insert_record(
    app: tauri::AppHandle,
    state: State<'_, Storage>,
    id: String,
    value: String,
) -> std::result::Result<(), Error> {
    let store = state.lock().await;
    store.ready()?;
    if value.trim().is_empty() || uuid::Uuid::parse_str(&id).is_err() {
        return Err(fail(
            "invalid_record",
            "Enter a non-empty value and valid record ID.",
        ));
    }
    let instances = app.state::<tauri_plugin_sql::DbInstances>();
    let pools = instances.0.read().await;
    let Some(tauri_plugin_sql::DbPool::Sqlite(pool)) = pools.get("sqlite:mission-control.db")
    else {
        return Err(fail(
            "database_unavailable",
            "Restart the application; keep all database files.",
        ));
    };
    sqlx::query("INSERT INTO persistence_test_records (id, value) VALUES (?, ?)")
        .bind(id)
        .bind(value)
        .execute(pool)
        .await?;
    Ok(())
}
