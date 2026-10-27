export interface Folder {
  id: number;
  parentId: number | null;
  name: string;
  /** Favorito (Fase 6): persistido no banco, sobrevive a fechar o app. */
  isFavorite: boolean;
}

export interface BreadcrumbItem {
  id: number;
  name: string;
}

export interface ManagedFile {
  id: number;
  folderId: number | null;
  originalFilename: string;
  currentFilename: string;
  extension: string;
  sizeBytes: number;
  importedAtUnixMs: number;
  /** Favorito (Fase 6): persistido no banco, sobrevive a fechar o app. */
  isFavorite: boolean;
}

export interface FolderListing {
  breadcrumb: BreadcrumbItem[];
  folders: Folder[];
  files: ManagedFile[];
}

export interface ImportFailure {
  fileName: string;
  error: string;
}

export interface ImportResult {
  imported: ManagedFile[];
  failed: ImportFailure[];
}

/** Espelha manager::jobs::JobStatus (Rust) -- mesmo pequeno conjunto de estados. */
export type JobStatus = "pending" | "running" | "completed" | "failed" | "cancelled";

export interface JobView {
  id: number;
  fileId: number;
  fileName: string;
  targetExtension: string;
  scheduledAtUnixMs: number;
  status: JobStatus;
  createdAtUnixMs: number;
  completedAtUnixMs: number | null;
  outputPath: string | null;
  errorMessage: string | null;
}
