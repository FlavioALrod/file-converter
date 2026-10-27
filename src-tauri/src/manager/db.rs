use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use super::error::StorageError;

/// Conexao compartilhada com o SQLite, gerenciada como estado do Tauri
/// (`.manage(...)` em lib.rs) e protegida por um Mutex -- `rusqlite::Connection`
/// nao e seguro para acesso concorrente sem isso. Para um app desktop de um
/// usuario so, serializar o acesso ao banco e simples e suficiente; nao ha
/// motivo para um pool de conexoes aqui.
pub struct Database(pub Mutex<Connection>);

/// Abre (ou cria) o banco dentro do diretorio de dados do aplicativo --
/// `app.path().app_data_dir()`, a API confirmada do Tauri 2 para isso -- e
/// garante que o schema existe.
pub fn init(app: &AppHandle) -> Result<Database, StorageError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| StorageError::AppDataDirUnavailable(e.to_string()))?;
    std::fs::create_dir_all(&data_dir).map_err(|e| StorageError::Io {
        path: data_dir.clone(),
        source: e,
    })?;

    let db_path = data_dir.join("gerenciador.sqlite3");
    let conn = Connection::open(db_path)?;

    // WAL: melhor resiliencia e concorrencia de leitura do que o modo padrao
    // (rollback journal) -- util porque o agendador em segundo plano pode
    // ler o banco enquanto um comando do frontend escreve.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    // Sem isto, o SQLite ignora `ON DELETE CASCADE` por padrao em cada nova
    // conexao -- apagar uma pasta nao apagaria os arquivos/agendamentos
    // dentro dela no banco, so o registro da propria pasta.
    conn.pragma_update(None, "foreign_keys", "ON")?;

    create_schema(&conn)?;
    run_migrations(&conn)?;

    Ok(Database(Mutex::new(conn)))
}

fn create_schema(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            parent_id INTEGER REFERENCES folders(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            created_at_unix_ms INTEGER NOT NULL,
            is_favorite INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS managed_files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_id INTEGER REFERENCES folders(id) ON DELETE CASCADE,
            original_filename TEXT NOT NULL,
            current_filename TEXT NOT NULL,
            extension TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            imported_at_unix_ms INTEGER NOT NULL,
            is_favorite INTEGER NOT NULL DEFAULT 0
        );

        -- `status`: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
        -- (modelo de estado pequeno e explicito, como pedido na especificacao).
        -- `scheduled_at_unix_ms` e um timestamp Unix em milissegundos, calculado
        -- no frontend a partir da data/hora local escolhida pelo usuario -- ver
        -- o comentario sobre isso no Cargo.toml e em src/lib/manager-api.ts.
        CREATE TABLE IF NOT EXISTS scheduled_jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL REFERENCES managed_files(id) ON DELETE CASCADE,
            target_extension TEXT NOT NULL,
            scheduled_at_unix_ms INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at_unix_ms INTEGER NOT NULL,
            completed_at_unix_ms INTEGER,
            output_path TEXT,
            error_message TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_managed_files_folder ON managed_files(folder_id);
        CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders(parent_id);
        CREATE INDEX IF NOT EXISTS idx_jobs_status_due ON scheduled_jobs(status, scheduled_at_unix_ms);
        ",
    )?;
    Ok(())
}

/// Migracao leve para a coluna `is_favorite` (recurso de favoritos).
///
/// `CREATE TABLE IF NOT EXISTS` acima so cria a tabela quando ela ainda nao
/// existe -- quem ja rodou o app antes desta funcionalidade tem um banco
/// com `folders`/`managed_files` sem essa coluna, e so adicionar o campo no
/// `CREATE TABLE` nao alcancaria esse banco ja existente. Por isso, aqui
/// conferimos se a coluna ja existe (`PRAGMA table_info`) e so rodamos o
/// `ALTER TABLE` quando falta -- o SQLite nao permite `ADD COLUMN`
/// repetido, entao a checagem antes e o que torna isto seguro de rodar
/// toda vez que o app abre, nao so na primeira.
fn run_migrations(conn: &Connection) -> Result<(), StorageError> {
    for table in ["folders", "managed_files"] {
        if !column_exists(conn, table, "is_favorite")? {
            // DEFAULT 0 (nao favorito) para linhas ja existentes -- a
            // especificacao pede para a migracao nao afetar o que ja esta
            // la, e um item antigo comecar "nao favoritado" e exatamente
            // o comportamento neutro esperado.
            conn.execute(
                &format!("ALTER TABLE {table} ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0"),
                [],
            )?;
        }
    }
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, StorageError> {
    // `table` so recebe literais fixos ("folders"/"managed_files") escritos
    // no proprio codigo acima, nunca entrada do usuario -- seguro interpolar
    // direto no SQL do PRAGMA (que, por limitacao do proprio SQLite, nao
    // aceita nome de tabela como parametro ligado `?`).
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let found = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|name| name.ok())
        .any(|name| name == column);
    Ok(found)
}
