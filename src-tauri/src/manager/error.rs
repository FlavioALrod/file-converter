use std::path::PathBuf;

/// Erro de dominio do gerenciador de arquivos e agendamento -- separado de
/// `ConversionError` (em src-tauri/src/error.rs) de proposito: sao dominios
/// diferentes (organizar arquivos/pastas e agendar jobs, versus executar
/// uma conversao). Juntar os dois num enum so misturaria responsabilidades
/// que o resto do projeto mantem separadas.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("nao foi possivel localizar o diretorio de dados do aplicativo: {0}")]
    AppDataDirUnavailable(String),

    #[error("erro no banco de dados: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("erro de sistema de arquivos em '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(
        "nome invalido '{0}': nao pode ser vazio, nem conter '/', '\\', nem ser '.' ou '..'"
    )]
    InvalidName(String),

    #[error("pasta nao encontrada")]
    FolderNotFound,

    #[error("arquivo gerenciado nao encontrado")]
    FileNotFound,

    #[error("agendamento nao encontrado")]
    JobNotFound,

    #[error("ja existe um item chamado '{0}' nesta pasta")]
    NameAlreadyExists(String),

    #[error(
        "o caminho resultante sairia da area de armazenamento do aplicativo -- operacao bloqueada por seguranca"
    )]
    PathEscape,

    #[error("nao e possivel duplicar uma pasta para dentro dela mesma")]
    CopyIntoSelf,
}
