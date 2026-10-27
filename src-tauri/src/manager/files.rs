use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use super::error::StorageError;
use super::now_unix_ms;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedFile {
    pub id: i64,
    pub folder_id: Option<i64>,
    pub original_filename: String,
    pub current_filename: String,
    pub extension: String,
    pub size_bytes: i64,
    pub imported_at_unix_ms: i64,
    /// Favorito (Fase 6), persistido -- ver o comentario equivalente em
    /// folders.rs::Folder::is_favorite para a logica de ordenacao.
    pub is_favorite: bool,
}

const SELECT_COLUMNS: &str =
    "id, folder_id, original_filename, current_filename, extension, size_bytes, imported_at_unix_ms, is_favorite";

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<ManagedFile> {
    Ok(ManagedFile {
        id: row.get(0)?,
        folder_id: row.get(1)?,
        original_filename: row.get(2)?,
        current_filename: row.get(3)?,
        extension: row.get(4)?,
        size_bytes: row.get(5)?,
        imported_at_unix_ms: row.get(6)?,
        is_favorite: row.get(7)?,
    })
}

// Nao ha um `exists_in_folder` aqui (diferente de `folders::exists_in_parent`,
// que existe e e usado de verdade): para arquivos, a protecao contra nome
// duplicado ja acontece no disco -- `import_file_into` usa `unique_path`
// (gera "nome (1).ext" automaticamente) e `rename_on_disk` confere
// `new_path.exists()` antes de renomear (ver storage.rs). Uma checagem
// equivalente aqui no banco seria uma segunda fonte de verdade sobre a
// mesma coisa, nao uma protecao a mais -- por isso nao foi adicionada.

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    folder_id: Option<i64>,
    original_filename: &str,
    current_filename: &str,
    extension: &str,
    size_bytes: u64,
) -> Result<i64, StorageError> {
    conn.execute(
        "INSERT INTO managed_files
            (folder_id, original_filename, current_filename, extension, size_bytes, imported_at_unix_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            folder_id,
            original_filename,
            current_filename,
            extension,
            size_bytes as i64,
            now_unix_ms(),
        ),
    )?;
    Ok(conn.last_insert_rowid())
}

/// `ORDER BY is_favorite DESC` primeiro, mesmo raciocinio de
/// `folders::list_children`: favoritos (1) antes dos demais (0).
pub fn list_in_folder(conn: &Connection, folder_id: Option<i64>) -> Result<Vec<ManagedFile>, StorageError> {
    let sql = format!(
        "SELECT {SELECT_COLUMNS} FROM managed_files WHERE folder_id IS ?1
         ORDER BY is_favorite DESC, current_filename COLLATE NOCASE"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map([folder_id], map_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: i64) -> Result<ManagedFile, StorageError> {
    let sql = format!("SELECT {SELECT_COLUMNS} FROM managed_files WHERE id = ?1");
    conn.query_row(&sql, [id], map_row)
        .optional()?
        .ok_or(StorageError::FileNotFound)
}

pub fn rename(conn: &Connection, id: i64, new_current_filename: &str, new_extension: &str) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE managed_files SET current_filename = ?1, extension = ?2 WHERE id = ?3",
        (new_current_filename, new_extension, id),
    )?;
    Ok(())
}

pub fn move_to(conn: &Connection, id: i64, new_folder_id: Option<i64>) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE managed_files SET folder_id = ?1 WHERE id = ?2",
        (new_folder_id, id),
    )?;
    Ok(())
}

/// Marca ou desmarca um arquivo como favorito -- so banco, nao mexe em
/// nada no disco (mesmo raciocinio de `folders::set_favorite`).
pub fn set_favorite(conn: &Connection, id: i64, is_favorite: bool) -> Result<(), StorageError> {
    let affected = conn.execute(
        "UPDATE managed_files SET is_favorite = ?1 WHERE id = ?2",
        (is_favorite, id),
    )?;
    if affected == 0 {
        return Err(StorageError::FileNotFound);
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), StorageError> {
    let affected = conn.execute("DELETE FROM managed_files WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(StorageError::FileNotFound);
    }
    Ok(())
}
