use tauri::State;

use crate::conversion::registry::ConversionRegistry;

/// Pede o cancelamento de uma conversao em andamento pelo `job_id`.
///
/// Devolve `true` se um processo em andamento foi encontrado e recebeu o
/// sinal de encerramento. O motor correspondente (document_engine ou
/// media_engine) e quem efetivamente percebe isso e devolve
/// `ConversionError::Cancelled` para a chamada de `convert_file` ainda
/// pendente -- este comando so inicia o pedido, nao espera o resultado.
///
/// Conversoes de imagem nao sao registradas aqui (ver registry.rs) porque
/// sao rapidas demais para precisar de cancelamento.
#[tauri::command]
pub fn cancel_conversion(registry: State<'_, ConversionRegistry>, job_id: String) -> bool {
    registry.cancel(&job_id)
}
