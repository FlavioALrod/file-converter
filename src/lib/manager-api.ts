import { invoke } from "@tauri-apps/api/core";
import type {
  Folder,
  FolderListing,
  ImportResult,
  JobView,
  ManagedFile,
} from "@/types/manager";

export async function listFolder(folderId: number | null): Promise<FolderListing> {
  return await invoke<FolderListing>("list_folder", { folderId });
}

export async function createFolder(parentId: number | null, name: string): Promise<Folder> {
  return await invoke<Folder>("create_folder", { parentId, name });
}

export async function importFiles(folderId: number | null, sourcePaths: string[]): Promise<ImportResult> {
  return await invoke<ImportResult>("import_files", { folderId, sourcePaths });
}

export async function renameFolder(id: number, newName: string): Promise<void> {
  await invoke("rename_folder", { id, newName });
}

export async function renameFile(id: number, newName: string): Promise<void> {
  await invoke("rename_file", { id, newName });
}

export async function moveFolder(id: number, newParentId: number | null): Promise<void> {
  await invoke("move_folder", { id, newParentId });
}

export async function moveFile(id: number, newFolderId: number | null): Promise<void> {
  await invoke("move_file", { id, newFolderId });
}

export async function deleteFolder(id: number): Promise<void> {
  await invoke("delete_folder", { id });
}

export async function deleteFile(id: number): Promise<void> {
  await invoke("delete_file", { id });
}

export async function listAllFolders(): Promise<Folder[]> {
  return await invoke<Folder[]>("list_all_folders");
}

/**
 * Favoritar/desfavoritar (Fase 6). Recebe o valor final desejado (nao um
 * "toggle" no backend) de proposito: o frontend ja sabe o estado atual do
 * item, entao mandar o valor explicito evita qualquer ambiguidade sobre o
 * que "alternar" significa se dois cliques chegarem fora de ordem.
 */
export async function setFolderFavorite(id: number, isFavorite: boolean): Promise<void> {
  await invoke("set_folder_favorite", { id, isFavorite });
}

export async function setFileFavorite(id: number, isFavorite: boolean): Promise<void> {
  await invoke("set_file_favorite", { id, isFavorite });
}

/**
 * Duplicar (Fase 6). A copia nasce do lado da original (mesma pasta-pai),
 * com um nome alternativo gerado automaticamente -- nunca pede destino,
 * entao nunca ha escolha de usuario que precise ser validada aqui.
 */
export async function duplicateFolder(id: number): Promise<Folder> {
  return await invoke<Folder>("duplicate_folder", { id });
}

export async function duplicateFile(id: number): Promise<ManagedFile> {
  return await invoke<ManagedFile>("duplicate_file", { id });
}

export async function createJob(
  fileId: number,
  targetExtension: string,
  scheduledAtUnixMs: number,
): Promise<JobView> {
  return await invoke<JobView>("create_job", { fileId, targetExtension, scheduledAtUnixMs });
}

export async function listJobs(): Promise<JobView[]> {
  return await invoke<JobView[]>("list_jobs");
}

export async function cancelJob(id: number): Promise<void> {
  await invoke("cancel_job", { id });
}

/**
 * Remove uma entrada do historico de agendamentos (Fase 7, botao de
 * lixeira). So apaga a notificacao/registro -- nunca o arquivo original
 * nem o arquivo ja convertido, que continuam onde estavam.
 */
export async function deleteJob(id: number): Promise<void> {
  await invoke("delete_job", { id });
}

export async function retryJob(id: number, newScheduledAtUnixMs: number): Promise<JobView> {
  return await invoke<JobView>("retry_job", { id, newScheduledAtUnixMs });
}

/**
 * Combina os valores de um <input type="date"> e um <input type="time">
 * (strings "YYYY-MM-DD" e "HH:MM") num timestamp Unix em milissegundos.
 *
 * Isto acontece aqui, no frontend, de proposito: o `Date` do JavaScript ja
 * interpreta esses valores no fuso horario local do sistema operacional
 * corretamente, nativamente -- nao ha necessidade de nenhuma biblioteca de
 * data/hora no backend Rust so para refazer essa conversao (ver o
 * comentario correspondente no Cargo.toml). O backend so compara numeros.
 */
export function localDateTimeToUnixMs(dateValue: string, timeValue: string): number | null {
  if (!dateValue || !timeValue) return null;
  const parsed = new Date(`${dateValue}T${timeValue}`);
  if (Number.isNaN(parsed.getTime())) return null;
  return parsed.getTime();
}

export function formatUnixMs(unixMs: number): string {
  return new Date(unixMs).toLocaleString();
}
