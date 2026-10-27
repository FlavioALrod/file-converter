/// Reexporta o utilitario compartilhado de `fs_utils` com o nome que o
/// motor de conversao ja usava desde a Fase 2 (`unique_output_path`), para
/// nao precisar tocar em image_engine.rs/document_engine.rs/media_engine.rs
/// so por causa desta promocao. A implementacao de verdade mora em
/// src-tauri/src/fs_utils.rs (Fase 5), reaproveitada tambem pelo
/// gerenciador de arquivos -- ver manager/storage.rs.
pub use crate::fs_utils::unique_path as unique_output_path;
