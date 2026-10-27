use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use super::error::StorageError;
use super::now_unix_ms;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    /// Favorito (Fase 6): persistido no banco para sobreviver a fechar e
    /// reabrir o app, como pedido na especificacao -- nao e so um estado
    /// de interface. A ordenacao "favoritos no topo" e feita em duas
    /// camadas: aqui no SQL (favoritos primeiro DENTRO das pastas), e no
    /// frontend (que agrupa favoritos de pastas E arquivos juntos acima de
    /// tudo, ja que os dois tipos vem em listas separadas do backend --
    /// ver FileBrowser.tsx).
    pub is_favorite: bool,
}

/// Um item do "caminho de migalhas" (breadcrumb) mostrado na interface.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BreadcrumbItem {
    pub id: i64,
    pub name: String,
}

const SELECT_COLUMNS: &str = "id, parent_id, name, is_favorite";

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Folder> {
    Ok(Folder {
        id: row.get(0)?,
        parent_id: row.get(1)?,
        name: row.get(2)?,
        is_favorite: row.get(3)?,
    })
}

pub fn exists_in_parent(
    conn: &Connection,
    parent_id: Option<i64>,
    name: &str,
) -> Result<bool, StorageError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM folders WHERE parent_id IS ?1 AND name = ?2",
        (parent_id, name),
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn create(conn: &Connection, parent_id: Option<i64>, name: &str) -> Result<i64, StorageError> {
    if exists_in_parent(conn, parent_id, name)? {
        return Err(StorageError::NameAlreadyExists(name.to_string()));
    }
    conn.execute(
        "INSERT INTO folders (parent_id, name, created_at_unix_ms) VALUES (?1, ?2, ?3)",
        (parent_id, name, now_unix_ms()),
    )?;
    Ok(conn.last_insert_rowid())
}

/// `ORDER BY is_favorite DESC` primeiro: no SQLite, `is_favorite` e 0 ou 1,
/// entao `DESC` coloca os favoritos (1) antes dos demais -- sem precisar
/// de um `CASE` ou segunda consulta so para isso.
pub fn list_children(conn: &Connection, parent_id: Option<i64>) -> Result<Vec<Folder>, StorageError> {
    let sql =
        format!("SELECT {SELECT_COLUMNS} FROM folders WHERE parent_id IS ?1 ORDER BY is_favorite DESC, name COLLATE NOCASE");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([parent_id], map_row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Todas as pastas do app de uma vez (id + nome + pai), para a interface
/// montar o seletor de destino da acao "Mover" sem precisar de uma
/// navegacao recursiva so para isso -- a propria interface monta a
/// indentacao a partir de `parent_id`.
pub fn list_all(conn: &Connection) -> Result<Vec<Folder>, StorageError> {
    let sql = format!("SELECT {SELECT_COLUMNS} FROM folders ORDER BY name COLLATE NOCASE");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?.collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: i64) -> Result<Folder, StorageError> {
    let sql = format!("SELECT {SELECT_COLUMNS} FROM folders WHERE id = ?1");
    conn.query_row(&sql, [id], map_row)
        .optional()?
        .ok_or(StorageError::FolderNotFound)
}

/// Monta a cadeia de pastas da raiz ate `folder_id` (inclusive), para o
/// breadcrumb da interface. `None` devolve uma lista vazia (raiz).
pub fn breadcrumb(conn: &Connection, folder_id: Option<i64>) -> Result<Vec<BreadcrumbItem>, StorageError> {
    let mut chain = Vec::new();
    let mut current = folder_id;
    while let Some(id) = current {
        let folder = get(conn, id)?;
        chain.push(BreadcrumbItem {
            id: folder.id,
            name: folder.name,
        });
        current = folder.parent_id;
    }
    chain.reverse();
    Ok(chain)
}

pub fn rename(conn: &Connection, id: i64, new_name: &str) -> Result<(), StorageError> {
    let folder = get(conn, id)?;
    if exists_in_parent(conn, folder.parent_id, new_name)? {
        return Err(StorageError::NameAlreadyExists(new_name.to_string()));
    }
    conn.execute("UPDATE folders SET name = ?1 WHERE id = ?2", (new_name, id))?;
    Ok(())
}

pub fn move_to(conn: &Connection, id: i64, new_parent_id: Option<i64>) -> Result<(), StorageError> {
    let folder = get(conn, id)?;
    if exists_in_parent(conn, new_parent_id, &folder.name)? {
        return Err(StorageError::NameAlreadyExists(folder.name));
    }
    conn.execute(
        "UPDATE folders SET parent_id = ?1 WHERE id = ?2",
        (new_parent_id, id),
    )?;
    Ok(())
}

/// Marca ou desmarca uma pasta como favorita. Operacao so de banco (ao
/// contrario de renomear/mover/excluir, nao mexe em nada no disco) --
/// favoritar e uma preferencia de organizacao, nao muda onde o arquivo
/// fisico esta guardado.
pub fn set_favorite(conn: &Connection, id: i64, is_favorite: bool) -> Result<(), StorageError> {
    let affected = conn.execute(
        "UPDATE folders SET is_favorite = ?1 WHERE id = ?2",
        (is_favorite, id),
    )?;
    if affected == 0 {
        return Err(StorageError::FolderNotFound);
    }
    Ok(())
}

/// A exclusao em cascata dos arquivos e agendamentos dentro desta pasta
/// (e de subpastas) acontece pelo `ON DELETE CASCADE` do schema, nao aqui.
pub fn delete(conn: &Connection, id: i64) -> Result<(), StorageError> {
    let affected = conn.execute("DELETE FROM folders WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(StorageError::FolderNotFound);
    }
    Ok(())
}
