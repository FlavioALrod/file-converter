use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};
use tokio::sync::Notify;

use crate::conversion::{self, registry::ConversionRegistry};

use super::db::Database;
use super::error::StorageError;
use super::{files, jobs, storage};

/// Handle compartilhado do agendador, registrado como estado do Tauri. Os
/// comandos de criar/cancelar/reagendar chamam `wake()` depois de mudar o
/// banco, para o loop em segundo plano recalcular o proximo horario na
/// hora, em vez de so descobrir a mudanca no proximo despertar ja
/// programado (que pode ser horas depois).
pub struct SchedulerHandle {
    notify: Arc<Notify>,
}

impl SchedulerHandle {
    pub fn wake(&self) {
        self.notify.notify_one();
    }
}

pub fn current_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Nenhum agendamento pendente: ainda assim dormimos (em vez de bloquear
/// para sempre), so para o loop acordar de tempos em tempos mesmo se um
/// `notify` for perdido por algum motivo -- mas o valor e alto porque o
/// caminho normal de acordar cedo e sempre o `Notify`, nao este teto.
const IDLE_SLEEP: Duration = Duration::from_secs(3600);

/// Inicia o loop do agendador como uma tarefa assincrona de fundo.
///
/// Sem polling: a cada volta, consulta so o PROXIMO horario pendente
/// (`MIN(scheduled_at_unix_ms)`, uma consulta indexada e barata) e dorme
/// exatamente ate la, com `tokio::select!` correndo em paralelo contra o
/// `Notify` para acordar antes se necessario. E o padrao confirmado na
/// pesquisa desta fase, usado por varios schedulers Rust reais.
pub fn spawn(app: AppHandle) -> SchedulerHandle {
    let notify = Arc::new(Notify::new());
    let handle = SchedulerHandle {
        notify: notify.clone(),
    };

    tauri::async_runtime::spawn(async move {
        loop {
            let wait = next_wait_duration(&app).unwrap_or_else(|e| {
                eprintln!("agendador: erro ao consultar o proximo horario, tentando de novo em 1 min: {e}");
                Duration::from_secs(60)
            });

            tokio::select! {
                _ = tokio::time::sleep(wait) => {}
                _ = notify.notified() => {}
            }

            if let Err(e) = run_due_jobs(&app).await {
                eprintln!("agendador: erro ao processar agendamentos devidos: {e}");
            }
        }
    });

    handle
}

fn next_wait_duration(app: &AppHandle) -> Result<Duration, StorageError> {
    let db = app.state::<Database>();
    let next_ms = {
        let conn = db.0.lock().unwrap();
        jobs::next_pending_time(&conn)?
    };

    Ok(match next_ms {
        Some(scheduled_ms) => {
            let remaining = scheduled_ms - current_unix_ms();
            Duration::from_millis(remaining.max(0) as u64)
        }
        None => IDLE_SLEEP,
    })
}

async fn run_due_jobs(app: &AppHandle) -> Result<(), StorageError> {
    let due_ids = {
        let db = app.state::<Database>();
        let conn = db.0.lock().unwrap();
        jobs::find_due(&conn, current_unix_ms())?
    };

    for job_db_id in due_ids {
        execute_job(app, job_db_id).await;
    }

    Ok(())
}

/// Executa um unico agendamento devido, do "reivindicar" (marcar como
/// `running` de forma atomica, que impede dupla execucao) ate marcar o
/// resultado final. Erros de cada etapa ficam contidos aqui -- uma falha
/// num job nao deve derrubar o loop do agendador nem impedir os outros.
async fn execute_job(app: &AppHandle, job_db_id: i64) {
    let claimed = {
        let db = app.state::<Database>();
        let conn = db.0.lock().unwrap();
        match jobs::try_mark_running(&conn, job_db_id) {
            Ok(claimed) => claimed,
            Err(e) => {
                eprintln!("agendador: nao foi possivel reivindicar o job {job_db_id}: {e}");
                false
            }
        }
    };
    if !claimed {
        // Ja reivindicado por outra volta do loop, ou cancelado entre a
        // consulta de "devidos" e aqui -- nao e um erro, so nao ha nada a
        // fazer.
        return;
    }

    let result = run_conversion_for_job(app, job_db_id).await;

    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();
    let outcome = match result {
        Ok(output_path) => jobs::mark_completed(&conn, job_db_id, &output_path),
        Err(message) => jobs::mark_failed(&conn, job_db_id, &message),
    };
    if let Err(e) = outcome {
        eprintln!("agendador: nao foi possivel gravar o resultado do job {job_db_id}: {e}");
    }
}

/// Resolve o arquivo de origem no disco, revalida que ele ainda existe
/// (pode ter sido apagado ou movido por fora do app desde que o
/// agendamento foi criado -- a especificacao pede essa revalidacao
/// explicitamente), e entao reaproveita `conversion::convert`, o MESMO
/// motor que a Aba 1 usa, em vez de duplicar a logica de conversao aqui.
async fn run_conversion_for_job(app: &AppHandle, job_db_id: i64) -> Result<String, String> {
    let (source_path, target_extension) = {
        let db = app.state::<Database>();
        let conn = db.0.lock().unwrap();
        let job = jobs::get(&conn, job_db_id).map_err(|e| e.to_string())?;
        let file = files::get(&conn, job.file_id).map_err(|e| e.to_string())?;
        let root = storage::managed_root(app).map_err(|e| e.to_string())?;
        let folder_path =
            storage::folder_disk_path(&conn, &root, file.folder_id).map_err(|e| e.to_string())?;
        (folder_path.join(&file.current_filename), job.target_extension)
    };

    if !source_path.exists() {
        return Err(
            "o arquivo de origem deste agendamento nao existe mais em disco (foi apagado ou movido)"
                .to_string(),
        );
    }

    let output_path = crate::fs_utils::unique_path(&source_path.with_extension(&target_extension));
    let job_id_str = format!("scheduled-{job_db_id}");
    let app_for_blocking = app.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let registry = app_for_blocking.state::<ConversionRegistry>();
        conversion::convert(
            &source_path.to_string_lossy(),
            &output_path.to_string_lossy(),
            &target_extension,
            &job_id_str,
            registry.inner(),
            &app_for_blocking,
        )
    })
    .await
    .map_err(|e| format!("a tarefa de conversao foi interrompida: {e}"))?
    .map(|outcome| outcome.output_path)
    .map_err(|e| e.to_string())
}
