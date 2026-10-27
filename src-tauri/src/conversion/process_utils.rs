use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::error::ConversionError;

use super::registry::ConversionRegistry;

/// Confirma que um binario externo (`soffice`, `ffmpeg`...) existe e responde
/// antes de tentar usa-lo para valer. Falhar aqui, cedo e com uma mensagem
/// clara de instalacao, e muito melhor do que deixar a conversao falhar la
/// na frente de um jeito dificil de entender -- exatamente o que a
/// especificacao pede ("nao assumir que o executavel esta disponivel").
pub fn check_binary_available(binary: &str, version_flag: &str, hint: &str) -> Result<(), ConversionError> {
    match Command::new(binary).arg(version_flag).output() {
        Ok(_) => Ok(()),
        // O importante aqui e so saber se o SO conseguiu executar o
        // binario. Um codigo de saida != 0 para "--version" em alguma
        // distribuicao especifica nao deveria nos impedir de tentar a
        // conversao de verdade.
        Err(_) => Err(ConversionError::MissingDependency {
            binary: binary.to_string(),
            hint: hint.to_string(),
        }),
    }
}

/// Confirma que um arquivo de saida foi REALMENTE gerado: existe E nao esta
/// vazio. Um processo externo pode terminar com codigo de saida 0 e ainda
/// assim nao ter produzido nada util (disco cheio, combinacao de formato
/// nao suportada mas tratada como "sucesso" pelo programa, etc.) -- a
/// propria especificacao desta fase pede explicitamente para nao declarar
/// sucesso so por causa do codigo de saida. Usado pelos tres motores
/// (documento, midia, e a vetorizacao dentro de image_engine.rs).
pub fn verify_output_file(path: &std::path::Path, binary: &str) -> Result<(), ConversionError> {
    let metadata = std::fs::metadata(path).map_err(|_| ConversionError::OutputNotCreated {
        binary: binary.to_string(),
    })?;
    if metadata.len() == 0 {
        return Err(ConversionError::EmptyOutput {
            binary: binary.to_string(),
        });
    }
    Ok(())
}

/// Roda um `Command` ja configurado, registrando-o no `ConversionRegistry`
/// (para permitir cancelamento) e aplicando um tempo-limite para nao travar
/// para sempre se o processo externo pendurar.
///
/// Usado pelo motor de documentos. O motor de midia tem seu proprio fluxo
/// (ver media_engine.rs) porque precisa ler a saida do processo aos poucos
/// para reportar progresso, em vez de so esperar ele terminar.
pub fn run_registered(
    mut command: Command,
    binary: &str,
    job_id: &str,
    registry: &ConversionRegistry,
    timeout: Duration,
) -> Result<(), ConversionError> {
    let child = command.spawn().map_err(|e| ConversionError::ProcessSpawn {
        binary: binary.to_string(),
        source: e,
    })?;
    let shared_child = Arc::new(Mutex::new(child));
    registry.register(job_id, shared_child.clone());

    let started = Instant::now();
    let status = loop {
        let maybe_status = shared_child
            .lock()
            .unwrap()
            .try_wait()
            .map_err(|e| ConversionError::ProcessSpawn {
                binary: binary.to_string(),
                source: e,
            })?;

        if let Some(status) = maybe_status {
            break status;
        }

        if started.elapsed() > timeout {
            let _ = shared_child.lock().unwrap().kill();
            registry.finish(job_id);
            return Err(ConversionError::ProcessTimeout {
                binary: binary.to_string(),
            });
        }

        std::thread::sleep(Duration::from_millis(150));
    };

    let was_cancelled = registry.finish(job_id);
    if was_cancelled {
        return Err(ConversionError::Cancelled);
    }
    if !status.success() {
        return Err(ConversionError::ProcessFailed {
            binary: binary.to_string(),
            code: status.code(),
        });
    }

    Ok(())
}
