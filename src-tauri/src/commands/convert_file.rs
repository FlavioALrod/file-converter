use tauri::{AppHandle, Manager};

use crate::conversion::{self, registry::ConversionRegistry, ConversionOutcome};

/// Comando unico chamado pelo frontend via `invoke("convert_file", {...})`
/// para qualquer categoria de arquivo (imagem, documento, audio ou video) --
/// ver src/lib/tauri-api.ts. `job_id` e gerado pelo frontend por arquivo na
/// fila e serve tanto para progresso quanto para cancelamento.
///
/// Roda em uma thread da pool de bloqueio do Tauri, entao nem a decodificacao
/// de imagem nem um `soffice`/`ffmpeg` demorado prendem a interface -- o
/// React continua respondendo normalmente durante a conversao.
///
/// Recebe `AppHandle` (em vez de `tauri::State<ConversionRegistry>`
/// diretamente) porque `AppHandle` e `Clone + 'static`, o que permite mover
/// uma copia para dentro do `spawn_blocking` sem truques de lifetime; o
/// registro e obtido de dentro da propria closure via `app.state::<...>()`.
#[tauri::command]
pub async fn convert_file(
    app: AppHandle,
    job_id: String,
    source_path: String,
    output_path: String,
    target_extension: String,
) -> Result<ConversionOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let registry = app.state::<ConversionRegistry>();
        conversion::convert(
            &source_path,
            &output_path,
            &target_extension,
            &job_id,
            registry.inner(),
            &app,
        )
    })
    .await
    .map_err(|join_err| format!("a tarefa de conversao foi interrompida: {join_err}"))?
    .map_err(|conversion_err| conversion_err.to_string())
}
