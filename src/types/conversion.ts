/**
 * Categoria de arquivo usada pela INTERFACE para agrupar o seletor de
 * formato de forma contextual (ver FormatPicker). E mais fina do que a
 * categoria que o backend usa para decidir qual motor Rust chamar
 * (imagem/documento/midia, ver src-tauri/src/conversion/mod.rs) porque
 * audio e video, apesar de usarem o mesmo motor (FFmpeg) la atras, tem
 * listas de formato e contexto bem diferentes aqui na frente.
 */
export type FileCategory = "image" | "document" | "audio" | "video";

export interface FormatOption {
  /** Extensao em minusculas, sem ponto. Ex.: "png". */
  extension: string;
  /** Nome amigavel mostrado na interface. Ex.: "PNG". */
  label: string;
  category: FileCategory;
}

export type ConversionStatus =
  | "idle"
  | "converting"
  | "done"
  | "error";

export interface QueuedFile {
  id: string;
  /** Caminho absoluto no disco, obtido via evento de arraste do Tauri. */
  sourcePath: string;
  /** Apenas o nome do arquivo, para exibicao. */
  fileName: string;
  sourceExtension: string;
  category: FileCategory;
  status: ConversionStatus;
  /** 0-100. So preenchido quando o motor reporta progresso real (midia). */
  progressPercent?: number;
  /** Preenchido quando status === "done". */
  outputPath?: string;
  /** Preenchido quando status === "error" (inclui cancelamento pelo usuario). */
  errorMessage?: string;
}

/** Espelha ConversionOutcome em src-tauri/src/conversion/mod.rs (serde). */
export interface ConversionOutcome {
  outputPath: string;
  durationMs: number;
}

/** Espelha ProgressPayload em src-tauri/src/conversion/media_engine.rs. */
export interface ProgressPayload {
  jobId: string;
  percent: number;
}
