use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::error::ConversionError;

use super::paths::unique_output_path;
use super::process_utils::{check_binary_available, run_registered, verify_output_file};
use super::registry::ConversionRegistry;

const INPUT_EXTENSIONS: &[&str] = &["docx", "doc", "odt", "rtf", "txt", "html", "htm", "pdf"];
const OUTPUT_EXTENSIONS: &[&str] = &["pdf", "docx", "odt", "rtf", "txt"];

const LIBREOFFICE_INSTALL_HINT: &str =
    "instale com 'sudo dnf install libreoffice' (ou os pacotes especificos, como 'libreoffice-writer', se preferir uma instalacao menor).";

/// Converte um documento usando o LibreOffice em modo headless
/// (`soffice --headless --convert-to`).
///
/// Decisoes de confiabilidade:
/// - `-env:UserInstallation` aponta para uma pasta de perfil temporaria e
///   exclusiva deste processo. Sem isso, rodar o `soffice --headless`
///   enquanto o usuario tem o LibreOffice normal aberto na area de trabalho
///   trava, porque os dois tentam usar o mesmo perfil bloqueado -- um
///   problema real e bem documentado desse modo de uso.
/// - O LibreOffice as vezes "termina com sucesso" sem gerar o arquivo de
///   saida quando a combinacao de formatos nao e realmente suportada; por
///   isso conferimos se o arquivo existe depois, em vez de confiar so no
///   codigo de saida do processo.
pub fn convert(
    source: &Path,
    output_requested: &Path,
    job_id: &str,
    registry: &ConversionRegistry,
) -> Result<PathBuf, ConversionError> {
    check_binary_available("soffice", "--version", LIBREOFFICE_INSTALL_HINT)?;

    let source_extension = extension_of(source);
    let target_extension = extension_of(output_requested);
    validate_pair(&source_extension, &target_extension)?;

    let output = unique_output_path(output_requested);
    let outdir = output.parent().unwrap_or_else(|| Path::new("."));

    let profile_dir = std::env::temp_dir().join(format!(
        "file-converter-lo-profile-{}-{}",
        std::process::id(),
        job_id
    ));

    let mut command = Command::new("soffice");
    command
        .arg(format!(
            "-env:UserInstallation=file://{}",
            profile_dir.display()
        ))
        .arg("--headless")
        .arg("--norestore")
        .arg("--convert-to")
        .arg(&target_extension)
        .arg("--outdir")
        .arg(outdir)
        .arg(source);

    // Documentos grandes (relatorios longos, PDFs pesados) podem demorar;
    // 3 minutos e generoso o bastante sem deixar um processo travado preso
    // para sempre.
    //
    // CORRECAO (auditoria Fase 4): antes, a limpeza da pasta de perfil
    // temporaria so acontecia DEPOIS desta chamada, usando `?` -- ou seja,
    // qualquer falha (processo com erro, timeout, cancelamento) pulava a
    // limpeza e deixava a pasta temporaria orfa no disco para sempre. Agora
    // guardamos o resultado, limpamos incondicionalmente, e so propagamos o
    // erro depois.
    let conversion_result =
        run_registered(command, "soffice", job_id, registry, Duration::from_secs(180));
    let _ = std::fs::remove_dir_all(&profile_dir);
    conversion_result?;

    // O LibreOffice sempre grava usando o mesmo nome-base do arquivo de
    // entrada dentro de --outdir, que e exatamente o caminho que montamos
    // acima -- mas ele pode terminar "com sucesso" sem gerar nada util se a
    // combinacao de formato nao for realmente suportada. `verify_output_file`
    // confirma que o arquivo existe E nao esta vazio, nao so que o processo
    // devolveu codigo 0 (a propria especificacao desta fase pede isso).
    verify_output_file(&output, "soffice")?;

    Ok(output)
}

fn extension_of(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default()
}

/// `pub(crate)` (nao so privado) para o dispatcher central em
/// `conversion::is_pair_supported` poder reaproveitar esta MESMA validacao
/// na hora de checar um agendamento antes de salvar (Fase 5), em vez de
/// duplicar a lista de formatos de documento em outro lugar.
pub(crate) fn validate_pair(source_extension: &str, target_extension: &str) -> Result<(), ConversionError> {
    if INPUT_EXTENSIONS.contains(&source_extension) && OUTPUT_EXTENSIONS.contains(&target_extension) {
        Ok(())
    } else {
        Err(ConversionError::NotYetSupported {
            from: source_extension.to_string(),
            to: target_extension.to_string(),
        })
    }
}
