use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::error::ConversionError;

use super::paths::unique_output_path;
use super::process_utils::{check_binary_available, verify_output_file};
use super::registry::ConversionRegistry;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "aac", "ogg"];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "webm", "avi"];

const FFMPEG_INSTALL_HINT: &str =
    "instale com 'sudo dnf install ffmpeg-free' (pacotes oficiais do Fedora) -- para mais codecs de audio/video com licenciamento restrito, habilite o repositorio RPM Fusion e instale o pacote 'ffmpeg' completo.";

/// Payload do evento `conversion-progress`, emitido para o frontend enquanto
/// o FFmpeg roda. Espelha ProgressPayload em src/types/conversion.ts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressPayload {
    pub job_id: String,
    pub percent: f64,
}

/// Converte audio/video usando o FFmpeg.
///
/// Decisoes de design:
/// - Nenhuma opcao de codec/qualidade e exposta: o FFmpeg escolhe um
///   codec padrao sensato a partir da extensao de saida (ex.: .mp3 vira
///   libmp3lame automaticamente), o que atende ao pedido da especificacao
///   de nao expor complexidade tecnica desnecessaria na interface padrao.
/// - Progresso real vem da propria saida do FFmpeg: a linha "Duration:"
///   no inicio da execucao da o tempo total, e as linhas "time=" seguintes
///   dao o tempo ja processado. Dividir um pelo outro da a porcentagem --
///   sem precisar do `ffprobe` como dependencia separada so para isso.
/// - `stdin(Stdio::null())` evita que o FFmpeg fique esperando uma
///   confirmacao interativa de "sobrescrever arquivo?" (o que travaria o
///   processo); como `unique_output_path` ja garante que o destino nao
///   existe de antemao, essa pergunta nunca deveria aparecer de qualquer
///   forma.
pub fn convert(
    source: &Path,
    output_requested: &Path,
    job_id: &str,
    registry: &ConversionRegistry,
    app: &AppHandle,
) -> Result<PathBuf, ConversionError> {
    check_binary_available("ffmpeg", "-version", FFMPEG_INSTALL_HINT)?;

    let source_extension = extension_of(source);
    let target_extension = extension_of(output_requested);
    validate_pair(&source_extension, &target_extension)?;

    let output = unique_output_path(output_requested);

    let mut command = Command::new("ffmpeg");
    command
        .arg("-hide_banner")
        .arg("-nostdin")
        .arg("-i")
        .arg(source)
        .arg(&output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|e| ConversionError::ProcessSpawn {
        binary: "ffmpeg".to_string(),
        source: e,
    })?;

    let stderr = child
        .stderr
        .take()
        .expect("stderr foi configurado como piped() ao montar o comando");

    let shared_child = Arc::new(Mutex::new(child));
    registry.register(job_id, shared_child.clone());

    let mut total_seconds: Option<f64> = None;
    for line in BufReader::new(stderr).lines().map_while(Result::ok) {
        if total_seconds.is_none() {
            total_seconds = parse_duration_line(&line);
        }
        if let (Some(total), Some(elapsed)) = (total_seconds, parse_time_progress_line(&line)) {
            if total > 0.0 {
                let percent = (elapsed / total * 100.0).clamp(0.0, 100.0);
                let _ = app.emit(
                    "conversion-progress",
                    ProgressPayload {
                        job_id: job_id.to_string(),
                        percent,
                    },
                );
            }
        }
    }

    let status = shared_child
        .lock()
        .unwrap()
        .wait()
        .map_err(|e| ConversionError::ProcessSpawn {
            binary: "ffmpeg".to_string(),
            source: e,
        })?;

    let was_cancelled = registry.finish(job_id);
    if was_cancelled {
        return Err(ConversionError::Cancelled);
    }
    if !status.success() {
        return Err(ConversionError::ProcessFailed {
            binary: "ffmpeg".to_string(),
            code: status.code(),
        });
    }
    // Confirma existencia E tamanho (auditoria Fase 4) -- um `output.exists()`
    // sozinho deixaria passar um arquivo de 0 bytes como "sucesso".
    verify_output_file(&output, "ffmpeg")?;

    // Sinaliza 100% explicitamente: a ultima linha "time=" as vezes fica um
    // pouco abaixo da duracao total por causa de arredondamento.
    let _ = app.emit(
        "conversion-progress",
        ProgressPayload {
            job_id: job_id.to_string(),
            percent: 100.0,
        },
    );

    Ok(output)
}

fn extension_of(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default()
}

/// `pub(crate)`: mesma razao do document_engine -- reaproveitado pelo
/// dispatcher central `conversion::is_pair_supported` (Fase 5).
pub(crate) fn validate_pair(source_extension: &str, target_extension: &str) -> Result<(), ConversionError> {
    let source_is_audio = AUDIO_EXTENSIONS.contains(&source_extension);
    let source_is_video = VIDEO_EXTENSIONS.contains(&source_extension);
    let target_is_audio = AUDIO_EXTENSIONS.contains(&target_extension);
    let target_is_video = VIDEO_EXTENSIONS.contains(&target_extension);

    // Audio -> audio e video -> video (transcodificacao normal), e tambem
    // video -> audio (extrair so a trilha sonora). Audio -> video fica de
    // fora: nao ha nenhuma imagem de origem para compor o video.
    let allowed = (source_is_audio && target_is_audio)
        || (source_is_video && target_is_video)
        || (source_is_video && target_is_audio);

    if allowed {
        Ok(())
    } else {
        Err(ConversionError::NotYetSupported {
            from: source_extension.to_string(),
            to: target_extension.to_string(),
        })
    }
}

/// Exemplo de linha real do FFmpeg:
/// "  Duration: 00:03:25.04, start: 0.000000, bitrate: 320 kb/s"
fn parse_duration_line(line: &str) -> Option<f64> {
    let start = line.find("Duration: ")? + "Duration: ".len();
    let rest = &line[start..];
    let end = rest.find(',')?;
    parse_timestamp(rest[..end].trim())
}

/// Exemplo de linha real do FFmpeg:
/// "frame=  120 fps= 30 q=28.0 size=    512kB time=00:00:04.00 bitrate=..."
fn parse_time_progress_line(line: &str) -> Option<f64> {
    let start = line.find("time=")? + "time=".len();
    let rest = &line[start..];
    let end = rest.find(' ').unwrap_or(rest.len());
    parse_timestamp(rest[..end].trim())
}

fn parse_timestamp(value: &str) -> Option<f64> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let hours: f64 = parts[0].parse().ok()?;
    let minutes: f64 = parts[1].parse().ok()?;
    let seconds: f64 = parts[2].parse().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}
