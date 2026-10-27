use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use super::error::StorageError;
use super::now_unix_ms;

/// Modelo de estado pequeno e explicito, como pedido na especificacao --
/// cada agendamento esta em exatamente um destes estados, sem variaveis de
/// status duplicadas ou paralelas em outro lugar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobStatus {
    /// Unica fonte de verdade para o texto gravado na coluna `status` do
    /// SQLite. Toda consulta abaixo monta seu SQL via `format!()` chamando
    /// isto, em vez de repetir a string literal -- evita exatamente o tipo
    /// de duplicacao que a auditoria encontrou (o metodo existia, mas cada
    /// consulta tinha o proprio literal escrito a mao, podendo divergir).
    fn as_db_str(self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        }
    }

    fn from_db_str(value: &str) -> Self {
        match value {
            "running" => JobStatus::Running,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            "cancelled" => JobStatus::Cancelled,
            _ => JobStatus::Pending,
        }
    }
}

/// Visao de um agendamento ja com o nome do arquivo de origem (join com
/// `managed_files`), para a interface nao precisar de uma consulta extra
/// por item da lista.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobView {
    pub id: i64,
    pub file_id: i64,
    pub file_name: String,
    pub target_extension: String,
    pub scheduled_at_unix_ms: i64,
    pub status: JobStatus,
    pub created_at_unix_ms: i64,
    pub completed_at_unix_ms: Option<i64>,
    pub output_path: Option<String>,
    pub error_message: Option<String>,
}

const SELECT_JOIN: &str = "
    SELECT j.id, j.file_id, f.current_filename, j.target_extension, j.scheduled_at_unix_ms,
           j.status, j.created_at_unix_ms, j.completed_at_unix_ms, j.output_path, j.error_message
    FROM scheduled_jobs j
    JOIN managed_files f ON f.id = j.file_id
";

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<JobView> {
    let status_str: String = row.get(5)?;
    Ok(JobView {
        id: row.get(0)?,
        file_id: row.get(1)?,
        file_name: row.get(2)?,
        target_extension: row.get(3)?,
        scheduled_at_unix_ms: row.get(4)?,
        status: JobStatus::from_db_str(&status_str),
        created_at_unix_ms: row.get(6)?,
        completed_at_unix_ms: row.get(7)?,
        output_path: row.get(8)?,
        error_message: row.get(9)?,
    })
}

pub fn create(
    conn: &Connection,
    file_id: i64,
    target_extension: &str,
    scheduled_at_unix_ms: i64,
) -> Result<i64, StorageError> {
    let sql = format!(
        "INSERT INTO scheduled_jobs
            (file_id, target_extension, scheduled_at_unix_ms, status, created_at_unix_ms)
         VALUES (?1, ?2, ?3, '{}', ?4)",
        JobStatus::Pending.as_db_str()
    );
    conn.execute(
        &sql,
        (file_id, target_extension, scheduled_at_unix_ms, now_unix_ms()),
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_all(conn: &Connection) -> Result<Vec<JobView>, StorageError> {
    let sql = format!("{SELECT_JOIN} ORDER BY j.scheduled_at_unix_ms ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: i64) -> Result<JobView, StorageError> {
    let sql = format!("{SELECT_JOIN} WHERE j.id = ?1");
    conn.query_row(&sql, [id], map_row)
        .optional()?
        .ok_or(StorageError::JobNotFound)
}

/// Agendamentos pendentes cujo horario ja chegou (inclui atrasados: um job
/// que ficou com o app fechado passado o horario marcado tambem conta como
/// "devido" -- politica documentada na especificacao e no ROADMAP).
pub fn find_due(conn: &Connection, now_ms: i64) -> Result<Vec<i64>, StorageError> {
    let sql = format!(
        "SELECT id FROM scheduled_jobs WHERE status = '{}' AND scheduled_at_unix_ms <= ?1
         ORDER BY scheduled_at_unix_ms ASC",
        JobStatus::Pending.as_db_str()
    );
    let mut stmt = conn.prepare(&sql)?;
    let ids = stmt
        .query_map([now_ms], |row| row.get(0))?
        .collect::<Result<Vec<i64>, _>>()?;
    Ok(ids)
}

/// Horario do proximo agendamento pendente (o mais proximo no futuro, ou
/// ja atrasado) -- usado pelo agendador para calcular quanto tempo dormir.
pub fn next_pending_time(conn: &Connection) -> Result<Option<i64>, StorageError> {
    let sql = format!(
        "SELECT MIN(scheduled_at_unix_ms) FROM scheduled_jobs WHERE status = '{}'",
        JobStatus::Pending.as_db_str()
    );
    conn.query_row(&sql, [], |row| row.get(0))
        .map_err(StorageError::from)
}

/// Transicao atomica pending -> running. So marca de verdade (devolve
/// `true`) se o job ainda estava pendente -- e isso que impede a mesma
/// tarefa de ser executada duas vezes caso o agendador seja acordado mais
/// de uma vez perto do mesmo horario.
pub fn try_mark_running(conn: &Connection, id: i64) -> Result<bool, StorageError> {
    let sql = format!(
        "UPDATE scheduled_jobs SET status = '{}' WHERE id = ?1 AND status = '{}'",
        JobStatus::Running.as_db_str(),
        JobStatus::Pending.as_db_str()
    );
    let affected = conn.execute(&sql, [id])?;
    Ok(affected > 0)
}

pub fn mark_completed(conn: &Connection, id: i64, output_path: &str) -> Result<(), StorageError> {
    let sql = format!(
        "UPDATE scheduled_jobs SET status = '{}', completed_at_unix_ms = ?1, output_path = ?2
         WHERE id = ?3",
        JobStatus::Completed.as_db_str()
    );
    conn.execute(&sql, (now_unix_ms(), output_path, id))?;
    Ok(())
}

pub fn mark_failed(conn: &Connection, id: i64, error_message: &str) -> Result<(), StorageError> {
    let sql = format!(
        "UPDATE scheduled_jobs SET status = '{}', completed_at_unix_ms = ?1, error_message = ?2
         WHERE id = ?3",
        JobStatus::Failed.as_db_str()
    );
    conn.execute(&sql, (now_unix_ms(), error_message, id))?;
    Ok(())
}

/// Cancela um agendamento. So pendentes podem ser cancelados -- um job que
/// ja esta rodando segue ate o fim (a conversao em si tem seu proprio
/// cancelamento, reaproveitado do motor existente, nao duplicado aqui).
pub fn cancel(conn: &Connection, id: i64) -> Result<(), StorageError> {
    let sql = format!(
        "UPDATE scheduled_jobs SET status = '{}' WHERE id = ?1 AND status = '{}'",
        JobStatus::Cancelled.as_db_str(),
        JobStatus::Pending.as_db_str()
    );
    let affected = conn.execute(&sql, [id])?;
    if affected == 0 {
        return Err(StorageError::JobNotFound);
    }
    Ok(())
}

/// Remove um agendamento do historico (Fase 7, botao de lixeira). Diferente
/// de `cancel` (que so muda o status e mantem o registro), isto apaga a
/// linha de verdade -- e o que da ao usuario controle total sobre quais
/// notificacoes de agendamento quer continuar vendo na lista, qualquer que
/// seja o status. Nao mexe em nenhum arquivo em disco: so remove a
/// ENTRADA DO HISTORICO, nunca o arquivo original nem o arquivo ja
/// convertido (se o job tiver concluido com sucesso, o resultado continua
/// onde foi salvo).
///
/// Sem protecao especial contra apagar um job "running" aqui no backend:
/// a interface ja evita oferecer o botao nesse caso (ver JobsPanel.tsx),
/// e mesmo que isto fosse chamado enquanto um job esta rodando, nao haveria
/// nada inseguro -- o agendador so tentaria gravar o resultado numa linha
/// que nao existe mais, o que no SQLite so afeta zero linhas, sem erro.
pub fn delete(conn: &Connection, id: i64) -> Result<(), StorageError> {
    let affected = conn.execute("DELETE FROM scheduled_jobs WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(StorageError::JobNotFound);
    }
    Ok(())
}

/// Reagenda um job com falha ou cancelado de volta para pendente, com um
/// novo horario.
pub fn retry(conn: &Connection, id: i64, new_scheduled_at_unix_ms: i64) -> Result<(), StorageError> {
    let sql = format!(
        "UPDATE scheduled_jobs
         SET status = '{}', scheduled_at_unix_ms = ?1, completed_at_unix_ms = NULL, error_message = NULL
         WHERE id = ?2 AND status IN ('{}', '{}')",
        JobStatus::Pending.as_db_str(),
        JobStatus::Failed.as_db_str(),
        JobStatus::Cancelled.as_db_str()
    );
    let affected = conn.execute(&sql, (new_scheduled_at_unix_ms, id))?;
    if affected == 0 {
        return Err(StorageError::JobNotFound);
    }
    Ok(())
}
