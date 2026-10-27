use tauri::{AppHandle, Manager};

use crate::conversion;
use crate::manager::db::Database;
use crate::manager::jobs::{self, JobView};
use crate::manager::scheduler::SchedulerHandle;
use crate::manager::files;

#[tauri::command]
pub fn create_job(
    app: AppHandle,
    file_id: i64,
    target_extension: String,
    scheduled_at_unix_ms: i64,
) -> Result<JobView, String> {
    // Validacao antes de salvar, como pede a especificacao: data valida
    // (um timestamp positivo e minimamente sensato) e combinacao de
    // formato suportada -- reaproveitando a mesma validacao dos motores de
    // conversao, nao uma copia das regras.
    if scheduled_at_unix_ms <= 0 {
        return Err("data e hora de agendamento invalidas".to_string());
    }

    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();

    let file = files::get(&conn, file_id).map_err(|e| e.to_string())?;
    if !conversion::is_pair_supported(&file.extension, &target_extension) {
        return Err(format!(
            "conversao de '.{}' para '.{}' nao e suportada",
            file.extension, target_extension
        ));
    }

    let job_id = jobs::create(&conn, file_id, &target_extension, scheduled_at_unix_ms)
        .map_err(|e| e.to_string())?;
    let created = jobs::get(&conn, job_id).map_err(|e| e.to_string())?;

    drop(conn);
    app.state::<SchedulerHandle>().wake();

    Ok(created)
}

#[tauri::command]
pub fn list_jobs(app: AppHandle) -> Result<Vec<JobView>, String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();
    jobs::list_all(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel_job(app: AppHandle, id: i64) -> Result<(), String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();
    jobs::cancel(&conn, id).map_err(|e| e.to_string())?;
    drop(conn);
    app.state::<SchedulerHandle>().wake();
    Ok(())
}

/// Remove uma entrada do historico de agendamentos (botao de lixeira, Fase
/// 7). Acorda o agendador pela mesma razao de create/cancel/retry acima:
/// se o job apagado era o proximo da fila, o agendador precisa recalcular
/// o horario para o qual deve dormir, em vez de so descobrir isso no
/// despertar ja programado (que agora nao tem mais sentido).
#[tauri::command]
pub fn delete_job(app: AppHandle, id: i64) -> Result<(), String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();
    jobs::delete(&conn, id).map_err(|e| e.to_string())?;
    drop(conn);
    app.state::<SchedulerHandle>().wake();
    Ok(())
}

#[tauri::command]
pub fn retry_job(app: AppHandle, id: i64, new_scheduled_at_unix_ms: i64) -> Result<JobView, String> {
    if new_scheduled_at_unix_ms <= 0 {
        return Err("data e hora de agendamento invalidas".to_string());
    }

    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();
    jobs::retry(&conn, id, new_scheduled_at_unix_ms).map_err(|e| e.to_string())?;
    let updated = jobs::get(&conn, id).map_err(|e| e.to_string())?;

    drop(conn);
    app.state::<SchedulerHandle>().wake();

    Ok(updated)
}

