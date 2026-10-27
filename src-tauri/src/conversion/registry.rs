use std::collections::{HashMap, HashSet};
use std::process::Child;
use std::sync::{Arc, Mutex};

/// Mantem o processo externo (LibreOffice, FFmpeg) de cada conversao em
/// andamento, indexado pelo `job_id` que o frontend gera por arquivo na
/// fila. E assim que o comando `cancel_conversion` encontra o processo
/// certo para encerrar sem afetar outras conversoes simultaneas.
///
/// Registrado como estado gerenciado do Tauri (`.manage(...)` em lib.rs) e
/// injetado nos comandos via `tauri::State<ConversionRegistry>`. O motor de
/// imagens nao usa este registro -- suas operacoes sao rapidas o bastante
/// para nao justificar a complexidade de cancelamento.
#[derive(Default)]
pub struct ConversionRegistry {
    children: Mutex<HashMap<String, Arc<Mutex<Child>>>>,
    cancelled: Mutex<HashSet<String>>,
}

impl ConversionRegistry {
    pub fn register(&self, job_id: &str, child: Arc<Mutex<Child>>) {
        self.children.lock().unwrap().insert(job_id.to_string(), child);
    }

    /// Chamado pelo motor de conversao quando o processo termina (com
    /// sucesso, erro ou por ter sido cancelado). Remove o registro e
    /// devolve `true` se este job havia sido marcado para cancelamento --
    /// e o sinal que o motor usa para devolver `ConversionError::Cancelled`
    /// em vez de tratar a saida do processo como uma falha de verdade.
    pub fn finish(&self, job_id: &str) -> bool {
        self.children.lock().unwrap().remove(job_id);
        self.cancelled.lock().unwrap().remove(job_id)
    }

    /// Chamado pelo comando `cancel_conversion`. Devolve `true` se um
    /// processo em andamento foi encontrado e o pedido de encerramento foi
    /// enviado (o encerramento em si e assincrono: o motor so vai perceber
    /// e devolver o erro correspondente quando o loop de leitura/espera dele
    /// notar que o processo saiu).
    pub fn cancel(&self, job_id: &str) -> bool {
        let found = match self.children.lock().unwrap().get(job_id) {
            Some(child) => {
                let _ = child.lock().unwrap().kill();
                true
            }
            None => false,
        };
        if found {
            self.cancelled.lock().unwrap().insert(job_id.to_string());
        }
        found
    }
}
