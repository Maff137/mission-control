use crate::backup::{Result, fail};
use crate::backup_commands::Storage;
use serde::Serialize;
use tauri::{Manager, State};

#[derive(Serialize)]
pub struct StudyArea {
    id: String,
    name: String,
    color: String,
    created_at: String,
}

#[derive(Serialize)]
pub struct StudySession {
    id: String,
    area_id: String,
    area_name: String,
    area_color: String,
    duration_minutes: i64,
    study_date: String,
    notes: String,
    created_at: String,
}

pub(crate) fn valid_study_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-'
        || !bytes.iter().enumerate().all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return false;
    }
    let year: u32 = value[0..4].parse().unwrap_or(0);
    let month: u32 = value[5..7].parse().unwrap_or(0);
    let day: u32 = value[8..10].parse().unwrap_or(0);
    if year == 0 {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

fn valid_id(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok()
}

#[tauri::command]
pub async fn list_study_areas(
    app: tauri::AppHandle,
    state: State<'_, Storage>,
) -> Result<Vec<StudyArea>> {
    let store = state.lock().await;
    store.ready()?;
    let instances = app.state::<tauri_plugin_sql::DbInstances>();
    let pools = instances.0.read().await;
    let Some(tauri_plugin_sql::DbPool::Sqlite(pool)) = pools.get("sqlite:mission-control.db") else {
        return Err(fail("database_unavailable", "Restart the application; keep all database files."));
    };
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT id, name, color, created_at FROM study_areas ORDER BY created_at ASC, id ASC"
    ).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(id, name, color, created_at)| StudyArea {
        id, name, color, created_at
    }).collect())
}

#[tauri::command]
pub async fn create_study_area(
    app: tauri::AppHandle,
    state: State<'_, Storage>,
    id: String,
    name: String,
    color: String,
) -> Result<()> {
    let name = name.trim();
    if !valid_id(&id) || name.is_empty() || name.chars().count() > 80
        || color.len() != 7 || !color.starts_with('#')
        || !color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
    {
        return Err(fail("invalid_study_area", "Enter a valid name (1–80 characters), color (#RRGGBB), and area ID."));
    }
    let store = state.lock().await;
    store.ready()?;
    let instances = app.state::<tauri_plugin_sql::DbInstances>();
    let pools = instances.0.read().await;
    let Some(tauri_plugin_sql::DbPool::Sqlite(pool)) = pools.get("sqlite:mission-control.db") else {
        return Err(fail("database_unavailable", "Restart the application; keep all database files."));
    };
    sqlx::query("INSERT INTO study_areas (id, name, color) VALUES (?, ?, ?)")
        .bind(id).bind(name).bind(color).execute(pool).await?;
    Ok(())
}

#[tauri::command]
pub async fn list_study_sessions(
    app: tauri::AppHandle,
    state: State<'_, Storage>,
) -> Result<Vec<StudySession>> {
    let store = state.lock().await;
    store.ready()?;
    let instances = app.state::<tauri_plugin_sql::DbInstances>();
    let pools = instances.0.read().await;
    let Some(tauri_plugin_sql::DbPool::Sqlite(pool)) = pools.get("sqlite:mission-control.db") else {
        return Err(fail("database_unavailable", "Restart the application; keep all database files."));
    };
    let rows: Vec<(String, String, String, String, i64, String, String, String)> = sqlx::query_as(
        "SELECT s.id, s.area_id, a.name, a.color, s.duration_minutes, s.study_date, s.notes, s.created_at
         FROM study_sessions s JOIN study_areas a ON a.id = s.area_id
         ORDER BY s.study_date DESC, s.created_at DESC, s.id DESC"
    ).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(id, area_id, area_name, area_color, duration_minutes, study_date, notes, created_at)| StudySession {
        id, area_id, area_name, area_color, duration_minutes, study_date, notes, created_at
    }).collect())
}

#[tauri::command]
pub async fn create_study_session(
    app: tauri::AppHandle,
    state: State<'_, Storage>,
    id: String,
    area_id: String,
    duration_minutes: i64,
    study_date: String,
    notes: String,
) -> Result<()> {
    if !valid_id(&id) || !valid_id(&area_id) || duration_minutes <= 0
        || !valid_study_date(&study_date) || notes.chars().count() > 10000
    {
        return Err(fail("invalid_study_session", "Enter a valid area, positive duration, calendar date, and notes of up to 10,000 characters."));
    }
    let store = state.lock().await;
    store.ready()?;
    let instances = app.state::<tauri_plugin_sql::DbInstances>();
    let pools = instances.0.read().await;
    let Some(tauri_plugin_sql::DbPool::Sqlite(pool)) = pools.get("sqlite:mission-control.db") else {
        return Err(fail("database_unavailable", "Restart the application; keep all database files."));
    };
    let exists: (i64,) = sqlx::query_as("SELECT count(*) FROM study_areas WHERE id = ?")
        .bind(&area_id).fetch_one(pool).await?;
    if exists.0 != 1 {
        return Err(fail("unknown_study_area", "Choose an existing study area."));
    }
    sqlx::query("INSERT INTO study_sessions (id, area_id, duration_minutes, study_date, notes) VALUES (?, ?, ?, ?, ?)")
        .bind(id).bind(area_id).bind(duration_minutes).bind(study_date).bind(notes)
        .execute(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn calendar_dates_are_validated_without_timezone_conversion() {
        for date in ["2026-11-12", "2024-02-29", "2000-02-29", "0001-01-01"] {
            assert!(super::valid_study_date(date), "{date}");
        }
        for date in ["2026-02-29", "1900-02-29", "2026-04-31", "0000-01-01", "2026-13-01", "2026-00-10", "2026-11-00", "2026-1-2", "2026-11-12T00:00:00Z", "٢٠٢٦-١١-١٢"] {
            assert!(!super::valid_study_date(date), "{date}");
        }
    }
}
