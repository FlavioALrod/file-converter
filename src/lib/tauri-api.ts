import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { save as saveDialog, open as openDialog } from "@tauri-apps/plugin-dialog";
import type { ConversionOutcome, ProgressPayload } from "@/types/conversion";

/**
 * Escuta o evento nativo de arrastar-e-soltar da janela do Tauri.
 *
 * Importante: isso NAO e o evento HTML5 `ondrop` do navegador. O Tauri
 * intercepta o drag-and-drop no nivel do sistema operacional e entrega os
 * caminhos absolutos dos arquivos (`event.payload.paths`), que e o que o
 * backend Rust precisa para abrir o arquivo em disco. Por isso a zona de
 * soltar em ConverterTab.tsx so usa CSS/estado do React para o destaque
 * visual, e depende deste listener para saber o que foi solto.
 *
 * Requer `app.windows[].dragDropEnabled` (padrao: true) em tauri.conf.json.
 */
export function onFilesDropped(
  onDrop: (paths: string[]) => void,
  onHoverChange?: (hovering: boolean) => void,
): () => void {
  let unlisten: (() => void) | undefined;
  let cancelled = false;

  getCurrentWebview()
    .onDragDropEvent((event) => {
      switch (event.payload.type) {
        case "enter":
        case "over":
          onHoverChange?.(true);
          break;
        case "drop":
          onHoverChange?.(false);
          onDrop(event.payload.paths);
          break;
        case "leave":
          onHoverChange?.(false);
          break;
      }
    })
    .then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    })
    .catch((err) => {
      console.error("Falha ao registrar o listener de arraste/soltar:", err);
    });

  return () => {
    cancelled = true;
    unlisten?.();
  };
}

/** Abre o seletor de arquivos nativo como alternativa ao arrastar-e-soltar. */
export async function pickFiles(): Promise<string[]> {
  const result = await openDialog({ multiple: true, directory: false });
  if (!result) return [];
  return Array.isArray(result) ? result : [result];
}

/** Pede ao usuario onde salvar o arquivo convertido. */
export async function pickSaveLocation(suggestedName: string): Promise<string | null> {
  return await saveDialog({ defaultPath: suggestedName });
}

/**
 * Chama o comando Rust `convert_file`, usado para qualquer categoria de
 * arquivo (imagem, documento, audio, video) -- quem decide o motor Rust
 * certo e o backend, ver src-tauri/src/conversion/mod.rs.
 *
 * `jobId` identifica esta conversao especifica (normalmente o `id` do
 * QueuedFile) para associar eventos de progresso e permitir cancelamento.
 *
 * Lanca (throw) uma string de erro legivel caso a conversao falhe --
 * `#[tauri::command]` no backend converte `Result::Err` em uma rejeicao de
 * Promise no lado do JavaScript.
 */
export async function convertFile(
  jobId: string,
  sourcePath: string,
  outputPath: string,
  targetExtension: string,
): Promise<ConversionOutcome> {
  return await invoke<ConversionOutcome>("convert_file", {
    jobId,
    sourcePath,
    outputPath,
    targetExtension,
  });
}

/**
 * Pede para cancelar a conversao identificada por `jobId`. Devolve `true`
 * se um processo em andamento foi encontrado -- o cancelamento em si e
 * assincrono: a Promise de `convertFile` ainda pendente para esse arquivo
 * vai rejeitar logo em seguida, com a mensagem de cancelamento.
 */
export async function cancelConversion(jobId: string): Promise<boolean> {
  return await invoke<boolean>("cancel_conversion", { jobId });
}

/**
 * Assina o evento `conversion-progress`, emitido pelo motor de midia
 * (FFmpeg) enquanto uma conversao de audio/video esta em andamento. Imagem
 * e documento nao emitem este evento -- a interface trata a ausencia de
 * progresso como "indeterminado" (ver ProgressBar).
 */
export function onConversionProgress(onProgress: (payload: ProgressPayload) => void): () => void {
  let unlisten: (() => void) | undefined;
  let cancelled = false;

  listen<ProgressPayload>("conversion-progress", (event) => onProgress(event.payload))
    .then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    })
    .catch((err) => {
      console.error("Falha ao assinar o evento de progresso:", err);
    });

  return () => {
    cancelled = true;
    unlisten?.();
  };
}
