pub mod document_engine;
pub mod image_engine;
pub mod media_engine;
pub mod paths;
pub mod process_utils;
pub mod registry;

use std::path::Path;
use std::time::Instant;

use serde::Serialize;
use tauri::AppHandle;

use crate::error::ConversionError;

use registry::ConversionRegistry;

/// Resultado devolvido ao frontend quando uma conversao termina com sucesso.
/// Nomes dos campos em camelCase de proposito: e assim que o `serde`
/// serializa para o JSON que o lado TypeScript espera (ver
/// ConversionOutcome em src/types/conversion.ts).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionOutcome {
    pub output_path: String,
    pub duration_ms: u128,
}

/// A qual motor uma extensao de arquivo pertence. Existe separada da
/// categoria mais fina usada na interface (imagem/documento/audio/video,
/// ver src/lib/formats.ts) porque aqui so importa qual modulo Rust deve
/// tratar o arquivo -- audio e video usam o mesmo motor (FFmpeg), entao
/// nao precisam ser distinguidos neste nivel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileCategory {
    Image,
    Document,
    Media,
}

const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif", "ico", "svg",
];
const DOCUMENT_EXTENSIONS: &[&str] = &["pdf", "docx", "doc", "odt", "rtf", "txt", "html", "htm"];
const MEDIA_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "flac", "aac", "ogg", "mp4", "mkv", "webm", "avi",
];

/// Verifica se uma combinacao de extensao de origem/destino e suportada,
/// SEM executar a conversao. Usado pelo agendamento (Fase 5) para validar
/// antes de salvar um job no banco -- reaproveitando a mesma validacao que
/// cada motor usa na hora de converter de verdade (`validate_pair` em
/// document/media, `is_pair_supported` em image), em vez de duplicar as
/// regras de formato numa terceira lista.
pub fn is_pair_supported(source_extension: &str, target_extension: &str) -> bool {
    let source_extension = source_extension.to_lowercase();
    let target_extension = target_extension.to_lowercase();
    let Some(category) = category_of_extension(&target_extension) else {
        return false;
    };
    match category {
        FileCategory::Image => image_engine::is_pair_supported(&source_extension, &target_extension),
        FileCategory::Document => {
            document_engine::validate_pair(&source_extension, &target_extension).is_ok()
        }
        FileCategory::Media => media_engine::validate_pair(&source_extension, &target_extension).is_ok(),
    }
}

fn category_of_extension(extension: &str) -> Option<FileCategory> {
    if IMAGE_EXTENSIONS.contains(&extension) {
        Some(FileCategory::Image)
    } else if DOCUMENT_EXTENSIONS.contains(&extension) {
        Some(FileCategory::Document)
    } else if MEDIA_EXTENSIONS.contains(&extension) {
        Some(FileCategory::Media)
    } else {
        None
    }
}

/// Ponto de entrada unico usado pelo comando Tauri `convert_file`. Decide,
/// a partir da extensao de destino, qual motor deve processar o arquivo, e
/// devolve um `ConversionOutcome` no mesmo formato para qualquer categoria
/// -- e a camada que existe justamente para o frontend (e o comando Tauri)
/// nao precisarem conhecer tres fluxos diferentes.
pub fn convert(
    source_path: &str,
    output_path_requested: &str,
    target_extension: &str,
    job_id: &str,
    registry: &ConversionRegistry,
    app: &AppHandle,
) -> Result<ConversionOutcome, ConversionError> {
    let target_extension = target_extension.to_lowercase();
    let category = category_of_extension(&target_extension).ok_or_else(|| {
        let source_extension = Path::new(source_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        ConversionError::NotYetSupported {
            from: source_extension,
            to: target_extension.clone(),
        }
    })?;

    // O motor de imagens e anterior a Fase 2 e ja resolve seu proprio tempo
    // de execucao e caminho final internamente (ver image_engine::convert).
    // Mantido como esta, sem forcar a mesma forma dos motores novos, para
    // nao mexer em codigo que ja funciona so por uniformidade.
    if category == FileCategory::Image {
        return image_engine::convert(source_path, output_path_requested, &target_extension);
    }

    let started = Instant::now();
    let source = Path::new(source_path);
    let output_requested = Path::new(output_path_requested);

    let final_output_path = match category {
        FileCategory::Document => {
            document_engine::convert(source, output_requested, job_id, registry)?
        }
        FileCategory::Media => media_engine::convert(source, output_requested, job_id, registry, app)?,
        FileCategory::Image => unreachable!("tratado acima com retorno antecipado"),
    };

    Ok(ConversionOutcome {
        output_path: final_output_path.to_string_lossy().into_owned(),
        duration_ms: started.elapsed().as_millis(),
    })
}
