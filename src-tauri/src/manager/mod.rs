//! Gerenciador de arquivos e agendamento de conversoes (Fase 5).
//!
//! Organizado em camadas, cada uma com uma responsabilidade so:
//! - `db`: conexao com o SQLite e schema.
//! - `folders` / `files` / `jobs`: operacoes de banco para cada tabela --
//!   nao sabem nada sobre o disco.
//! - `storage`: seguranca de caminho e operacoes de disco -- nao sabe nada
//!   sobre o banco.
//! - `scheduler`: o loop de fundo que executa agendamentos devidos,
//!   reaproveitando o motor de conversao existente (`conversion::convert`).
//!
//! Os comandos Tauri que orquestram banco + disco juntos (criar pasta,
//! importar, renomear, mover, excluir) ficam em `commands/files.rs` e
//! `commands/jobs.rs`, nao aqui -- este modulo so expoe as operacoes de
//! baixo nivel.

pub mod db;
pub mod error;
pub mod files;
pub mod folders;
pub mod jobs;
pub mod scheduler;
pub mod storage;

use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
