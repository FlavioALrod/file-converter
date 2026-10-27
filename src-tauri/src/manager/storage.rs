use std::path::{Component, Path, PathBuf};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::fs_utils::unique_path;

use super::error::StorageError;

/// Raiz do armazenamento gerenciado: uma subpasta dedicada dentro do
/// diretorio de dados do aplicativo (nunca a raiz de `app_data_dir()`
/// direto, para nao se misturar com o banco de dados ou qualquer outro
/// arquivo interno do app).
pub fn managed_root(app: &AppHandle) -> Result<PathBuf, StorageError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| StorageError::AppDataDirUnavailable(e.to_string()))?;
    Ok(data_dir.join("arquivos"))
}

/// Valida um nome de pasta/arquivo fornecido pelo usuario (nunca um
/// caminho completo). Rejeita por COMPONENTE de caminho, nao por busca de
/// substring: "a..b" e um nome de arquivo valido (".." so aparece como
/// texto), mas "../etc", "/etc" ou "sub/nome" nao sao -- essa distincao e
/// a forma correta e recomendada de checar isso em Rust (usando
/// `Path::components()` em vez de comparar strings), confirmada na
/// pesquisa desta fase.
pub fn sanitize_name(name: &str) -> Result<String, StorageError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(StorageError::InvalidName(name.to_string()));
    }

    let as_path = Path::new(trimmed);
    let mut components = as_path.components();
    let first = components.next();
    let has_more_than_one = components.next().is_some();

    if has_more_than_one || !matches!(first, Some(Component::Normal(_))) {
        return Err(StorageError::InvalidName(name.to_string()));
    }

    Ok(trimmed.to_string())
}

/// Resolve o caminho em disco de uma pasta a partir do seu id, subindo a
/// cadeia de pastas-pai no banco e juntando cada nome (ja validado no
/// momento da criacao) a partir da raiz gerenciada. `None` representa a
/// raiz do armazenamento gerenciado.
pub fn folder_disk_path(
    conn: &Connection,
    root: &Path,
    folder_id: Option<i64>,
) -> Result<PathBuf, StorageError> {
    let mut names = Vec::new();
    let mut current = folder_id;

    while let Some(id) = current {
        let (name, parent_id): (String, Option<i64>) = conn
            .query_row(
                "SELECT name, parent_id FROM folders WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| StorageError::FolderNotFound)?;
        names.push(name);
        current = parent_id;
    }

    let mut path = root.to_path_buf();
    for name in names.into_iter().rev() {
        path.push(name);
    }

    // Checagem defensiva (alem da validacao de cada nome individual na
    // criacao): o caminho final precisa continuar dentro da raiz. Como
    // cada segmento ja passou por `sanitize_name`, isto nunca deveria
    // falhar de verdade -- e uma rede de seguranca, nao o mecanismo
    // principal de protecao.
    if !path.starts_with(root) {
        return Err(StorageError::PathEscape);
    }

    Ok(path)
}

/// Copia um arquivo de uma localizacao qualquer do computador para dentro
/// do armazenamento gerenciado. Usa `std::fs::copy`, que e uma copia feita
/// pelo sistema operacional (streaming/otimizada pelo kernel), nao uma
/// leitura do arquivo inteiro para a memoria -- importante para arquivos de
/// video grandes, por exemplo. Nunca move nem apaga o arquivo original.
pub fn import_file_into(
    dest_dir: &Path,
    source: &Path,
    desired_name: &str,
) -> Result<(PathBuf, u64), StorageError> {
    std::fs::create_dir_all(dest_dir).map_err(|e| StorageError::Io {
        path: dest_dir.to_path_buf(),
        source: e,
    })?;

    let dest = unique_path(&dest_dir.join(desired_name));
    std::fs::copy(source, &dest).map_err(|e| StorageError::Io {
        path: source.to_path_buf(),
        source: e,
    })?;

    let size = std::fs::metadata(&dest)
        .map_err(|e| StorageError::Io {
            path: dest.clone(),
            source: e,
        })?
        .len();

    Ok((dest, size))
}

pub fn create_dir(path: &Path) -> Result<(), StorageError> {
    std::fs::create_dir_all(path).map_err(|e| StorageError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Renomeia um arquivo ou pasta no disco, mantendo-o na mesma pasta-pai.
pub fn rename_on_disk(old_path: &Path, new_name: &str) -> Result<PathBuf, StorageError> {
    let sanitized = sanitize_name(new_name)?;
    let parent = old_path.parent().unwrap_or_else(|| Path::new("."));
    let new_path = parent.join(&sanitized);

    if new_path.exists() {
        return Err(StorageError::NameAlreadyExists(sanitized));
    }

    std::fs::rename(old_path, &new_path).map_err(|e| StorageError::Io {
        path: old_path.to_path_buf(),
        source: e,
    })?;

    Ok(new_path)
}

/// Move um arquivo ou pasta para dentro de outro diretorio, mantendo o
/// mesmo nome.
pub fn move_on_disk(old_path: &Path, new_parent_dir: &Path) -> Result<PathBuf, StorageError> {
    let file_name = old_path
        .file_name()
        .ok_or_else(|| StorageError::PathEscape)?;
    let new_path = new_parent_dir.join(file_name);

    if new_path.exists() {
        return Err(StorageError::NameAlreadyExists(
            file_name.to_string_lossy().into_owned(),
        ));
    }

    std::fs::create_dir_all(new_parent_dir).map_err(|e| StorageError::Io {
        path: new_parent_dir.to_path_buf(),
        source: e,
    })?;
    std::fs::rename(old_path, &new_path).map_err(|e| StorageError::Io {
        path: old_path.to_path_buf(),
        source: e,
    })?;

    Ok(new_path)
}

/// Apaga a copia gerenciada (arquivo ou pasta, recursivamente) -- NUNCA o
/// arquivo original de onde ela foi importada, que o app nunca referencia
/// de novo depois da copia. Usa `symlink_metadata` (em vez de `metadata`)
/// para decidir entre arquivo/pasta sem seguir um link simbolico, caso
/// algum tenha sido colocado manualmente dentro da pasta gerenciada por
/// fora do app.
pub fn delete_on_disk(path: &Path) -> Result<(), StorageError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| StorageError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let result = if metadata.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };

    result.map_err(|e| StorageError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}
