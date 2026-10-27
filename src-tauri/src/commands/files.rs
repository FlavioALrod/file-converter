use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::fs_utils::unique_path;
use crate::manager::db::Database;
use crate::manager::error::StorageError;
use crate::manager::{files, folders, storage};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderListing {
    pub breadcrumb: Vec<folders::BreadcrumbItem>,
    pub folders: Vec<folders::Folder>,
    pub files: Vec<files::ManagedFile>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFailure {
    pub file_name: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported: Vec<files::ManagedFile>,
    pub failed: Vec<ImportFailure>,
}

#[tauri::command]
pub fn list_folder(app: AppHandle, folder_id: Option<i64>) -> Result<FolderListing, String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();
    Ok(FolderListing {
        breadcrumb: folders::breadcrumb(&conn, folder_id).map_err(|e| e.to_string())?,
        folders: folders::list_children(&conn, folder_id).map_err(|e| e.to_string())?,
        files: files::list_in_folder(&conn, folder_id).map_err(|e| e.to_string())?,
    })
}

/// Lista todas as pastas de uma vez, para o seletor de destino da acao
/// "Mover" na interface (ver MoveDialog.tsx).
#[tauri::command]
pub fn list_all_folders(app: AppHandle) -> Result<Vec<folders::Folder>, String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();
    folders::list_all(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_folder(app: AppHandle, parent_id: Option<i64>, name: String) -> Result<folders::Folder, String> {
    let sanitized = storage::sanitize_name(&name).map_err(|e| e.to_string())?;

    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();

    let root = storage::managed_root(&app).map_err(|e| e.to_string())?;
    let parent_disk_path =
        storage::folder_disk_path(&conn, &root, parent_id).map_err(|e| e.to_string())?;
    let new_disk_path = parent_disk_path.join(&sanitized);

    // Disco primeiro, banco depois: se o disco falhar (permissao negada,
    // disco cheio), nada e gravado no banco. Se o disco der certo mas o
    // banco falhar, desfazemos a pasta recem-criada em vez de deixar um
    // diretorio orfao sem registro.
    storage::create_dir(&new_disk_path).map_err(|e| e.to_string())?;

    match folders::create(&conn, parent_id, &sanitized) {
        // Busca o registro de volta do banco em vez de remontar a struct a
        // mao: `get` e a unica fonte de verdade sobre o formato de um
        // Folder (inclui is_favorite, e qualquer campo futuro) sem
        // precisar lembrar de atualizar esta construcao tambem -- foi
        // exatamente o oposto disso que causou o erro de build anterior.
        Ok(id) => folders::get(&conn, id).map_err(|e| e.to_string()),
        Err(db_err) => {
            let _ = std::fs::remove_dir(&new_disk_path);
            Err(db_err.to_string())
        }
    }
}

#[tauri::command]
pub fn import_files(app: AppHandle, folder_id: Option<i64>, source_paths: Vec<String>) -> Result<ImportResult, String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();

    let root = storage::managed_root(&app).map_err(|e| e.to_string())?;
    let dest_dir = storage::folder_disk_path(&conn, &root, folder_id).map_err(|e| e.to_string())?;

    let mut imported = Vec::new();
    let mut failed = Vec::new();

    for source_str in source_paths {
        let source = std::path::Path::new(&source_str);
        let file_name = source
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| source_str.clone());
        let extension = source
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let result = (|| -> Result<files::ManagedFile, String> {
            let (dest_path, size) =
                storage::import_file_into(&dest_dir, source, &file_name).map_err(|e| e.to_string())?;
            let current_filename = dest_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| file_name.clone());

            match files::create(&conn, folder_id, &file_name, &current_filename, &extension, size) {
                // Mesma correcao de create_folder acima: busca o registro
                // de volta do banco em vez de remontar a struct a mao. Como
                // bonus, corrige tambem uma pequena inconsistencia que essa
                // reconstrucao manual tinha: `imported_at_unix_ms` era
                // calculado de novo aqui (um `now_unix_ms()` chamado uma
                // segunda vez, podendo divergir por alguns milissegundos do
                // valor que `files::create` ja tinha gravado no banco).
                Ok(id) => files::get(&conn, id).map_err(|e| e.to_string()),
                Err(db_err) => {
                    let _ = std::fs::remove_file(&dest_path);
                    Err(db_err.to_string())
                }
            }
        })();

        match result {
            Ok(managed_file) => imported.push(managed_file),
            Err(error) => failed.push(ImportFailure { file_name, error }),
        }
    }

    Ok(ImportResult { imported, failed })
}

#[tauri::command]
pub fn rename_folder(app: AppHandle, id: i64, new_name: String) -> Result<(), String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();

    let root = storage::managed_root(&app).map_err(|e| e.to_string())?;
    let old_disk_path = storage::folder_disk_path(&conn, &root, Some(id)).map_err(|e| e.to_string())?;

    let new_disk_path = storage::rename_on_disk(&old_disk_path, &new_name).map_err(|e| e.to_string())?;

    if let Err(db_err) = folders::rename(&conn, id, &new_disk_path.file_name().unwrap().to_string_lossy()) {
        // Desfaz a renomeacao no disco para nao deixar banco e disco
        // dessincronizados (o banco e a fonte de verdade para o nome
        // exibido na interface).
        let _ = std::fs::rename(&new_disk_path, &old_disk_path);
        return Err(db_err.to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn rename_file(app: AppHandle, id: i64, new_name: String) -> Result<(), String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();

    let file = files::get(&conn, id).map_err(|e| e.to_string())?;
    let root = storage::managed_root(&app).map_err(|e| e.to_string())?;
    let folder_path =
        storage::folder_disk_path(&conn, &root, file.folder_id).map_err(|e| e.to_string())?;
    let old_disk_path = folder_path.join(&file.current_filename);

    let new_disk_path = storage::rename_on_disk(&old_disk_path, &new_name).map_err(|e| e.to_string())?;
    let new_filename = new_disk_path.file_name().unwrap().to_string_lossy().into_owned();
    let new_extension = new_disk_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if let Err(db_err) = files::rename(&conn, id, &new_filename, &new_extension) {
        let _ = std::fs::rename(&new_disk_path, &old_disk_path);
        return Err(db_err.to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn move_folder(app: AppHandle, id: i64, new_parent_id: Option<i64>) -> Result<(), String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();

    let root = storage::managed_root(&app).map_err(|e| e.to_string())?;
    let old_disk_path = storage::folder_disk_path(&conn, &root, Some(id)).map_err(|e| e.to_string())?;
    let new_parent_disk_path =
        storage::folder_disk_path(&conn, &root, new_parent_id).map_err(|e| e.to_string())?;

    let new_disk_path =
        storage::move_on_disk(&old_disk_path, &new_parent_disk_path).map_err(|e| e.to_string())?;

    if let Err(db_err) = folders::move_to(&conn, id, new_parent_id) {
        let _ = std::fs::rename(&new_disk_path, &old_disk_path);
        return Err(db_err.to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn move_file(app: AppHandle, id: i64, new_folder_id: Option<i64>) -> Result<(), String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();

    let file = files::get(&conn, id).map_err(|e| e.to_string())?;
    let root = storage::managed_root(&app).map_err(|e| e.to_string())?;
    let old_folder_path =
        storage::folder_disk_path(&conn, &root, file.folder_id).map_err(|e| e.to_string())?;
    let old_disk_path = old_folder_path.join(&file.current_filename);
    let new_folder_path =
        storage::folder_disk_path(&conn, &root, new_folder_id).map_err(|e| e.to_string())?;

    let new_disk_path =
        storage::move_on_disk(&old_disk_path, &new_folder_path).map_err(|e| e.to_string())?;

    if let Err(db_err) = files::move_to(&conn, id, new_folder_id) {
        let _ = std::fs::rename(&new_disk_path, &old_disk_path);
        return Err(db_err.to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn delete_folder(app: AppHandle, id: i64) -> Result<(), String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();

    let root = storage::managed_root(&app).map_err(|e| e.to_string())?;
    let disk_path = storage::folder_disk_path(&conn, &root, Some(id)).map_err(|e| e.to_string())?;

    // Disco primeiro: se a pasta nao puder ser apagada (permissao, etc.),
    // o registro no banco continua existindo e o usuario ve o erro em vez
    // de a pasta "sumir" da interface mas continuar ocupando espaco.
    storage::delete_on_disk(&disk_path).map_err(|e| e.to_string())?;
    folders::delete(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_file(app: AppHandle, id: i64) -> Result<(), String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();

    let file = files::get(&conn, id).map_err(|e| e.to_string())?;
    let root = storage::managed_root(&app).map_err(|e| e.to_string())?;
    let folder_path =
        storage::folder_disk_path(&conn, &root, file.folder_id).map_err(|e| e.to_string())?;
    let disk_path = folder_path.join(&file.current_filename);

    storage::delete_on_disk(&disk_path).map_err(|e| e.to_string())?;
    files::delete(&conn, id).map_err(|e| e.to_string())
}

// === Favoritos (Fase 6) =====================================================
//
// Botao de estrela em cada linha de pasta/arquivo (ver FileBrowser.tsx).
// So mexe no banco -- nunca no disco -- porque favoritar e uma preferencia
// de organizacao pessoal, nao uma mudanca no arquivo em si. A lista volta
// ordenada com os favoritos primeiro (ver folders::list_children e
// files::list_in_folder), e o frontend agrupa os favoritos de pastas e
// arquivos juntos no topo, ja que os dois tipos chegam em listas separadas.

#[tauri::command]
pub fn set_folder_favorite(app: AppHandle, id: i64, is_favorite: bool) -> Result<(), String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();
    folders::set_favorite(&conn, id, is_favorite).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_file_favorite(app: AppHandle, id: i64, is_favorite: bool) -> Result<(), String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();
    files::set_favorite(&conn, id, is_favorite).map_err(|e| e.to_string())
}

// === Duplicar (Fase 6) ======================================================
//
// Botao de copiar em cada linha. A copia sempre nasce do lado da original
// (mesma pasta-pai), nunca pergunta destino -- por isso "copiar uma pasta
// para dentro dela mesma" nao tem como acontecer neste fluxo por
// construcao: o destino e sempre irmao da origem, nunca descendente dela.
// Mesmo assim, `duplicate_folder_into` abaixo confere isso explicitamente
// antes de comecar, como defesa para o caso de esta funcao auxiliar ser
// reaproveitada no futuro por um fluxo que aceite um destino escolhido
// pelo usuario (onde a garantia por construcao deixaria de valer).

#[tauri::command]
pub fn duplicate_file(app: AppHandle, id: i64) -> Result<files::ManagedFile, String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();

    let source = files::get(&conn, id).map_err(|e| e.to_string())?;
    let root = storage::managed_root(&app).map_err(|e| e.to_string())?;
    let folder_disk_path =
        storage::folder_disk_path(&conn, &root, source.folder_id).map_err(|e| e.to_string())?;
    let source_disk_path = folder_disk_path.join(&source.current_filename);

    let dest_disk_path = unique_path(&folder_disk_path.join(copy_candidate_name(&source.current_filename)));

    // `std::fs::copy` e uma copia feita pelo sistema operacional (nao le o
    // arquivo inteiro para a memoria do processo) -- mesma razao de
    // `storage::import_file_into` usar a mesma funcao. Erros de permissao
    // ou espaco em disco chegam aqui como `io::Error` comum, com a
    // mensagem do proprio sistema operacional incluida.
    std::fs::copy(&source_disk_path, &dest_disk_path)
        .map_err(|e| format!("nao foi possivel copiar '{}': {e}", source.current_filename))?;

    let dest_filename = dest_disk_path
        .file_name()
        .expect("dest_disk_path construido com join() sempre tem nome de arquivo")
        .to_string_lossy()
        .into_owned();
    let size_bytes = std::fs::metadata(&dest_disk_path)
        .map(|m| m.len())
        .unwrap_or(source.size_bytes as u64);

    match files::create(
        &conn,
        source.folder_id,
        &source.original_filename,
        &dest_filename,
        &source.extension,
        size_bytes,
    ) {
        Ok(new_id) => files::get(&conn, new_id).map_err(|e| e.to_string()),
        Err(db_err) => {
            // Copia no disco deu certo mas o registro no banco falhou:
            // desfaz a copia fisica para nao deixar um arquivo "fantasma"
            // sem registro nenhum, visivel so por fora do app.
            let _ = std::fs::remove_file(&dest_disk_path);
            Err(db_err.to_string())
        }
    }
}

#[tauri::command]
pub fn duplicate_folder(app: AppHandle, id: i64) -> Result<folders::Folder, String> {
    let db = app.state::<Database>();
    let conn = db.0.lock().unwrap();

    let source = folders::get(&conn, id).map_err(|e| e.to_string())?;
    let root = storage::managed_root(&app).map_err(|e| e.to_string())?;
    let parent_disk_path =
        storage::folder_disk_path(&conn, &root, source.parent_id).map_err(|e| e.to_string())?;

    let dest_disk_path = unique_path(&parent_disk_path.join(copy_candidate_name(&source.name)));
    let dest_name = dest_disk_path
        .file_name()
        .expect("dest_disk_path construido com join() sempre tem nome de arquivo")
        .to_string_lossy()
        .into_owned();

    storage::create_dir(&dest_disk_path).map_err(|e| e.to_string())?;

    let new_folder_id = match folders::create(&conn, source.parent_id, &dest_name) {
        Ok(new_id) => new_id,
        Err(db_err) => {
            let _ = std::fs::remove_dir(&dest_disk_path);
            return Err(db_err.to_string());
        }
    };

    if let Err(e) = duplicate_folder_into(&conn, &root, id, new_folder_id, &dest_disk_path) {
        // Copia parcial: desfaz tudo (disco e banco) em vez de deixar uma
        // pasta pela metade. `folders::delete` apaga em cascata qualquer
        // subpasta/arquivo que ja tenha sido gravado no banco antes da
        // falha, entao basta apagar a pasta-raiz da copia.
        let _ = std::fs::remove_dir_all(&dest_disk_path);
        let _ = folders::delete(&conn, new_folder_id);
        return Err(e.to_string());
    }

    folders::get(&conn, new_folder_id).map_err(|e| e.to_string())
}

/// Copia recursivamente o CONTEUDO de uma pasta (subpastas e arquivos) para
/// dentro de um destino ja criado no disco e no banco. Conduzido pelo
/// BANCO, nao por uma varredura direta do disco: a estrutura ja confiavel
/// no banco e o que determina o que e copiado, para nunca duplicar algo
/// que o app nao esteja rastreando (por exemplo, um arquivo que o usuario
/// tenha colocado na pasta gerenciada por fora do app, sem passar por
/// "Importar"). Disco e banco andam juntos em cada passo: so grava o
/// registro depois que a copia fisica correspondente deu certo.
fn duplicate_folder_into(
    conn: &Connection,
    root: &Path,
    source_folder_id: i64,
    dest_folder_id: i64,
    dest_disk_path: &Path,
) -> Result<(), StorageError> {
    let source_disk_path = storage::folder_disk_path(conn, root, Some(source_folder_id))?;

    // Defesa explicita (alem da garantia por construcao explicada no
    // comentario do modulo acima): nunca copiar uma pasta para dentro dela
    // mesma, o que causaria uma recursao sem fim.
    if dest_disk_path.starts_with(&source_disk_path) {
        return Err(StorageError::CopyIntoSelf);
    }

    for child_folder in folders::list_children(conn, Some(source_folder_id))? {
        let child_dest_disk_path = dest_disk_path.join(&child_folder.name);
        storage::create_dir(&child_dest_disk_path)?;
        let new_child_id = folders::create(conn, Some(dest_folder_id), &child_folder.name)?;
        duplicate_folder_into(conn, root, child_folder.id, new_child_id, &child_dest_disk_path)?;
    }

    for file in files::list_in_folder(conn, Some(source_folder_id))? {
        let source_file_path = source_disk_path.join(&file.current_filename);
        let dest_file_path = dest_disk_path.join(&file.current_filename);
        std::fs::copy(&source_file_path, &dest_file_path).map_err(|e| StorageError::Io {
            path: source_file_path.clone(),
            source: e,
        })?;
        files::create(
            conn,
            Some(dest_folder_id),
            &file.original_filename,
            &file.current_filename,
            &file.extension,
            file.size_bytes as u64,
        )?;
    }

    Ok(())
}

/// Monta o nome candidato "<nome> (copia)" ou "<nome> (copia).<ext>" a
/// partir do nome original. O resultado ainda passa por `unique_path`
/// (reaproveitado do motor de conversao via fs_utils, nao reimplementado
/// aqui) para o caso de ja existir uma copia com esse nome -- a mesma
/// desambiguacao usada em todo o resto do app, nao uma segunda forma de
/// gerar nomes alternativos.
fn copy_candidate_name(original_name: &str) -> String {
    let path = Path::new(original_name);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| original_name.to_string());
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => format!("{stem} (copia).{ext}"),
        None => format!("{stem} (copia)"),
    }
}
