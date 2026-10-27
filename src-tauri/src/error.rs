use std::path::PathBuf;

/// Erro de dominio para qualquer operacao de conversao.
///
/// Fica tipado aqui dentro do backend; so vira `String` na borda do comando
/// Tauri (ver commands/convert_file.rs), que e o formato que a Promise
/// rejeitada do lado do JavaScript consegue receber.
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("nao foi possivel ler o arquivo de origem '{path}': {source}")]
    ReadSource {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("nao foi possivel escrever o arquivo de saida '{path}': {source}")]
    WriteOutput {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("falha ao decodificar ou codificar a imagem: {0}")]
    Image(#[from] image::ImageError),

    #[error("falha ao interpretar o SVG de origem: {0}")]
    SvgParse(String),

    #[error("falha ao renderizar o SVG para uma imagem de pixels")]
    SvgRender,

    #[error("falha ao vetorizar a imagem para SVG: {0}")]
    Vectorize(String),

    #[error("conversao de '.{from}' para '.{to}' ainda nao e suportada nesta fase do projeto")]
    NotYetSupported { from: String, to: String },

    // --- Fase 2: motores baseados em processo externo (LibreOffice, FFmpeg) ---
    #[error(
        "o programa externo '{binary}' nao foi encontrado no sistema. {hint}"
    )]
    MissingDependency { binary: String, hint: String },

    #[error("nao foi possivel iniciar o processo '{binary}': {source}")]
    ProcessSpawn {
        binary: String,
        #[source]
        source: std::io::Error,
    },

    #[error("o processo '{binary}' encerrou com erro (codigo {code:?})")]
    ProcessFailed { binary: String, code: Option<i32> },

    #[error("o processo '{binary}' nao respondeu dentro do tempo limite e foi encerrado")]
    ProcessTimeout { binary: String },

    #[error("conversao cancelada pelo usuario")]
    Cancelled,

    #[error(
        "o arquivo de saida esperado nao foi criado -- a combinacao de formatos pode nao ser suportada por '{binary}'"
    )]
    OutputNotCreated { binary: String },

    // --- Fase 4: endurecidos durante a auditoria ---------------------------
    #[error(
        "'{binary}' terminou sem erro, mas o arquivo de saida ficou vazio (0 bytes) -- a conversao provavelmente falhou silenciosamente"
    )]
    EmptyOutput { binary: String },

    #[error(
        "a imagem de origem e grande demais para renderizar com seguranca ({width}x{height}px, limite {limit}x{limit}px)"
    )]
    ImageTooLarge { width: u32, height: u32, limit: u32 },

    #[error(
        "a vetorizacao produziu um arquivo que nao e um SVG valido (nao foi possivel reabri-lo como vetor): {0}"
    )]
    InvalidVectorOutput(String),
}
