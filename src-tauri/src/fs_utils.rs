use std::path::{Path, PathBuf};

/// Utilitarios de sistema de arquivos sem dependencia de nenhum dominio do
/// app (nem conversao, nem gerenciador de arquivos) -- por isso moraram
/// aqui, na raiz do crate, em vez de dentro de `conversion/`. Promovido na
/// Fase 5 para o motor de conversao e o gerenciador de arquivos poderem
/// reaproveitar a mesma logica de "nao sobrescrever sem querer" sem um
/// modulo depender do outro.

/// Evita sobrescrever um arquivo existente sem avisar: se o caminho pedido
/// ja existe, acrescenta " (1)", " (2)" etc. ate achar um nome livre -- o
/// mesmo padrao que gerenciadores de arquivos usam para downloads duplicados.
pub fn unique_path(requested: &Path) -> PathBuf {
    if !requested.exists() {
        return requested.to_path_buf();
    }

    let parent = requested.parent().unwrap_or_else(|| Path::new(""));
    let stem = requested
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("item");
    let extension = requested.extension().and_then(|e| e.to_str());

    for n in 1..1000 {
        let candidate_name = match extension {
            Some(ext) => format!("{stem} ({n}).{ext}"),
            None => format!("{stem} ({n})"),
        };
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    // Caso extremamente improvavel (999 colisoes): devolve o caminho
    // original em vez de travar o fluxo; quem chamar isso ainda vai
    // reportar um erro visivel se a escrita realmente falhar.
    requested.to_path_buf()
}
