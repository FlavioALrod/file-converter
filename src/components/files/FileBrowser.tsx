import { useCallback, useEffect, useState } from "react";
import {
  CalendarClock,
  ChevronRight,
  Copy,
  File as FileIcon,
  FolderInput,
  FolderPlus,
  Folder as FolderIcon,
  Pencil,
  Star,
  Trash2,
  Upload,
} from "lucide-react";
import { Button } from "@/components/ui/Button";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import { PromptDialog } from "@/components/ui/PromptDialog";
import { MoveDialog } from "@/components/files/MoveDialog";
import { ScheduleDialog } from "@/components/files/ScheduleDialog";
import {
  createFolder,
  deleteFile,
  deleteFolder,
  duplicateFile,
  duplicateFolder,
  importFiles,
  listFolder,
  moveFile,
  moveFolder,
  renameFile,
  renameFolder,
  setFileFavorite,
  setFolderFavorite,
} from "@/lib/manager-api";
import { pickFiles } from "@/lib/tauri-api";
import type { Folder, FolderListing, ManagedFile } from "@/types/manager";

type RenameTarget = { kind: "folder" | "file"; id: number; currentName: string };
type MoveTarget = { kind: "folder" | "file"; id: number };
type DeleteTarget = { kind: "folder" | "file"; id: number; name: string };

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function FileBrowser({ onJobScheduled }: { onJobScheduled: () => void }) {
  const [currentFolderId, setCurrentFolderId] = useState<number | null>(null);
  const [listing, setListing] = useState<FolderListing | null>(null);
  const [importMessage, setImportMessage] = useState<string | null>(null);

  const [isNewFolderOpen, setIsNewFolderOpen] = useState(false);
  const [renameTarget, setRenameTarget] = useState<RenameTarget | null>(null);
  const [moveTarget, setMoveTarget] = useState<MoveTarget | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<DeleteTarget | null>(null);
  const [scheduleTarget, setScheduleTarget] = useState<ManagedFile | null>(null);

  const refresh = useCallback(() => {
    listFolder(currentFolderId)
      .then(setListing)
      .catch((err) => console.error("Falha ao listar pasta:", err));
  }, [currentFolderId]);

  useEffect(refresh, [refresh]);

  async function handleCreateFolder(name: string) {
    try {
      await createFolder(currentFolderId, name);
      setIsNewFolderOpen(false);
      refresh();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : String(err));
    }
  }

  async function handleImport() {
    const sourcePaths = await pickFiles();
    if (sourcePaths.length === 0) return;
    try {
      const result = await importFiles(currentFolderId, sourcePaths);
      if (result.failed.length > 0) {
        setImportMessage(
          `${result.imported.length} importado(s), ${result.failed.length} com erro: ${result.failed
            .map((f) => f.fileName)
            .join(", ")}`,
        );
      } else {
        setImportMessage(null);
      }
      refresh();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : String(err));
    }
  }

  async function handleRename(newName: string) {
    if (!renameTarget) return;
    try {
      if (renameTarget.kind === "folder") {
        await renameFolder(renameTarget.id, newName);
      } else {
        await renameFile(renameTarget.id, newName);
      }
      setRenameTarget(null);
      refresh();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : String(err));
    }
  }

  async function handleMove(destinationId: number | null) {
    if (!moveTarget) return;
    try {
      if (moveTarget.kind === "folder") {
        await moveFolder(moveTarget.id, destinationId);
      } else {
        await moveFile(moveTarget.id, destinationId);
      }
      setMoveTarget(null);
      refresh();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : String(err));
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    try {
      if (deleteTarget.kind === "folder") {
        await deleteFolder(deleteTarget.id);
      } else {
        await deleteFile(deleteTarget.id);
      }
      setDeleteTarget(null);
      refresh();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : String(err));
    }
  }

  /**
   * Botao de estrela (Fase 6). Recebe o item inteiro (nao so o id) porque
   * precisa saber o estado ATUAL para mandar o valor OPOSTO ao backend --
   * o backend grava o valor que mandamos, nao faz um "toggle" por conta
   * propria (ver o comentario em manager-api.ts sobre essa escolha).
   */
  async function handleToggleFolderFavorite(folder: Folder) {
    try {
      await setFolderFavorite(folder.id, !folder.isFavorite);
      refresh();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : String(err));
    }
  }

  async function handleToggleFileFavorite(file: ManagedFile) {
    try {
      await setFileFavorite(file.id, !file.isFavorite);
      refresh();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : String(err));
    }
  }

  /**
   * Botao de duplicar (Fase 6). Sem dialogo de confirmacao: diferente de
   * excluir, duplicar nao e destrutivo -- na pior das hipoteses, sobra uma
   * copia extra que da pra excluir depois. O proprio backend ja gera um
   * nome alternativo ("<nome> (copia)") sozinho, entao nao ha nada para o
   * usuario decidir aqui, so esperar o resultado.
   */
  async function handleDuplicateFolder(id: number) {
    try {
      await duplicateFolder(id);
      refresh();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : String(err));
    }
  }

  async function handleDuplicateFile(id: number) {
    try {
      await duplicateFile(id);
      refresh();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : String(err));
    }
  }

  if (!listing) {
    return <p className="py-8 text-center font-body text-sm text-ink-muted">Carregando...</p>;
  }

  const isEmpty = listing.folders.length === 0 && listing.files.length === 0;

  // Favoritos no topo, independentemente de serem pasta ou arquivo (pedido
  // explicito da especificacao): o backend ja devolve cada lista (pastas,
  // arquivos) com os favoritos primeiro DENTRO do proprio tipo -- ver
  // `ORDER BY is_favorite DESC` em folders.rs/files.rs -- mas pastas e
  // arquivos chegam em arrays separados. Separar aqui em "favoritos" x
  // "demais" e juntar pasta+arquivo dentro de cada grupo e o que garante
  // que um ARQUIVO favorito apareca acima de uma PASTA nao favorita, nao
  // so dentro do proprio tipo.
  const favoriteFolders = listing.folders.filter((f) => f.isFavorite);
  const favoriteFiles = listing.files.filter((f) => f.isFavorite);
  const regularFolders = listing.folders.filter((f) => !f.isFavorite);
  const regularFiles = listing.files.filter((f) => !f.isFavorite);
  const hasFavorites = favoriteFolders.length > 0 || favoriteFiles.length > 0;

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex items-center justify-between gap-3">
        <nav className="flex min-w-0 flex-wrap items-center gap-1 font-body text-sm text-ink-muted">
          <button onClick={() => setCurrentFolderId(null)} className="hover:text-ink hover:underline">
            Raiz
          </button>
          {listing.breadcrumb.map((item) => (
            <span key={item.id} className="flex items-center gap-1">
              <ChevronRight size={14} />
              <button onClick={() => setCurrentFolderId(item.id)} className="hover:text-ink hover:underline">
                {item.name}
              </button>
            </span>
          ))}
        </nav>
        <div className="flex flex-shrink-0 gap-2">
          <Button variant="outline" icon={<FolderPlus size={16} />} onClick={() => setIsNewFolderOpen(true)}>
            Nova pasta
          </Button>
          <Button variant="outline" icon={<Upload size={16} />} onClick={handleImport}>
            Importar
          </Button>
        </div>
      </div>

      {importMessage && <p className="font-body text-xs text-danger">{importMessage}</p>}

      <div className="flex-1 overflow-y-auto">
        {isEmpty ? (
          <p className="py-8 text-center font-body text-sm text-ink-muted">
            Esta pasta esta vazia. Crie uma subpasta ou importe arquivos para comecar.
          </p>
        ) : (
          <div className="flex flex-col gap-4">
            {hasFavorites && (
              <div className="flex flex-col gap-1.5">
                <p className="flex items-center gap-1.5 font-display text-xs font-semibold uppercase tracking-wide text-ink-muted">
                  <Star size={13} fill="currentColor" className="text-accent" />
                  Favoritos
                </p>
                <ul className="flex flex-col gap-1.5">
                  {favoriteFolders.map((folder) => (
                    <FolderRow
                      key={`folder-${folder.id}`}
                      folder={folder}
                      onOpen={() => setCurrentFolderId(folder.id)}
                      onToggleFavorite={() => handleToggleFolderFavorite(folder)}
                      onDuplicate={() => handleDuplicateFolder(folder.id)}
                      onRename={() => setRenameTarget({ kind: "folder", id: folder.id, currentName: folder.name })}
                      onMove={() => setMoveTarget({ kind: "folder", id: folder.id })}
                      onDelete={() => setDeleteTarget({ kind: "folder", id: folder.id, name: folder.name })}
                    />
                  ))}
                  {favoriteFiles.map((file) => (
                    <FileRow
                      key={`file-${file.id}`}
                      file={file}
                      onSchedule={() => setScheduleTarget(file)}
                      onToggleFavorite={() => handleToggleFileFavorite(file)}
                      onDuplicate={() => handleDuplicateFile(file.id)}
                      onRename={() =>
                        setRenameTarget({ kind: "file", id: file.id, currentName: file.currentFilename })
                      }
                      onMove={() => setMoveTarget({ kind: "file", id: file.id })}
                      onDelete={() => setDeleteTarget({ kind: "file", id: file.id, name: file.currentFilename })}
                    />
                  ))}
                </ul>
              </div>
            )}

            {(regularFolders.length > 0 || regularFiles.length > 0) && (
              <ul className="flex flex-col gap-1.5">
                {regularFolders.map((folder) => (
                  <FolderRow
                    key={`folder-${folder.id}`}
                    folder={folder}
                    onOpen={() => setCurrentFolderId(folder.id)}
                    onToggleFavorite={() => handleToggleFolderFavorite(folder)}
                    onDuplicate={() => handleDuplicateFolder(folder.id)}
                    onRename={() => setRenameTarget({ kind: "folder", id: folder.id, currentName: folder.name })}
                    onMove={() => setMoveTarget({ kind: "folder", id: folder.id })}
                    onDelete={() => setDeleteTarget({ kind: "folder", id: folder.id, name: folder.name })}
                  />
                ))}
                {regularFiles.map((file) => (
                  <FileRow
                    key={`file-${file.id}`}
                    file={file}
                    onSchedule={() => setScheduleTarget(file)}
                    onToggleFavorite={() => handleToggleFileFavorite(file)}
                    onDuplicate={() => handleDuplicateFile(file.id)}
                    onRename={() =>
                      setRenameTarget({ kind: "file", id: file.id, currentName: file.currentFilename })
                    }
                    onMove={() => setMoveTarget({ kind: "file", id: file.id })}
                    onDelete={() => setDeleteTarget({ kind: "file", id: file.id, name: file.currentFilename })}
                  />
                ))}
              </ul>
            )}
          </div>
        )}
      </div>

      <PromptDialog
        open={isNewFolderOpen}
        title="Nova pasta"
        label="Nome da pasta"
        confirmLabel="Criar"
        onConfirm={handleCreateFolder}
        onCancel={() => setIsNewFolderOpen(false)}
      />
      <PromptDialog
        open={renameTarget !== null}
        title={renameTarget?.kind === "folder" ? "Renomear pasta" : "Renomear arquivo"}
        label="Novo nome"
        initialValue={renameTarget?.currentName ?? ""}
        confirmLabel="Renomear"
        onConfirm={handleRename}
        onCancel={() => setRenameTarget(null)}
      />
      <MoveDialog
        open={moveTarget !== null}
        currentFolderId={currentFolderId}
        excludeFolderId={moveTarget?.kind === "folder" ? moveTarget.id : undefined}
        onConfirm={handleMove}
        onCancel={() => setMoveTarget(null)}
      />
      <ConfirmDialog
        open={deleteTarget !== null}
        title={deleteTarget?.kind === "folder" ? "Excluir pasta?" : "Excluir arquivo?"}
        description={
          deleteTarget?.kind === "folder"
            ? `"${deleteTarget?.name}" e tudo dentro dela serao removidos do armazenamento gerenciado pelo app. O arquivo original de onde qualquer item foi importado nao e afetado.`
            : `"${deleteTarget?.name}" sera removido do armazenamento gerenciado pelo app. O arquivo original de onde ele foi importado nao e afetado.`
        }
        confirmLabel="Excluir"
        onConfirm={handleDelete}
        onCancel={() => setDeleteTarget(null)}
      />
      <ScheduleDialog
        file={scheduleTarget}
        onScheduled={() => {
          setScheduleTarget(null);
          onJobScheduled();
        }}
        onCancel={() => setScheduleTarget(null)}
      />
    </div>
  );
}

/**
 * Botao de favoritar, compartilhado entre pasta e arquivo. Estrela
 * preenchida (`fill="currentColor"`) quando favorito, so contorno quando
 * nao -- o padrao visual mais reconhecido para "favorito" em qualquer
 * gerenciador de arquivos, o que ajuda a bater com o icone de referencia
 * enviado. Fica como primeiro elemento da linha de proposito: e o que
 * precisa ser mais facil de notar/bater o olho, conforme pedido
 * ("facil a visualizacao e encontrar o arquivo mais usado").
 */
function FavoriteButton({ isFavorite, onToggle }: { isFavorite: boolean; onToggle: () => void }) {
  return (
    <button
      onClick={onToggle}
      aria-label={isFavorite ? "Remover dos favoritos" : "Adicionar aos favoritos"}
      aria-pressed={isFavorite}
      className={`flex-shrink-0 p-1 ${isFavorite ? "text-accent" : "text-ink-muted/40 hover:text-ink-muted"}`}
    >
      <Star size={16} fill={isFavorite ? "currentColor" : "none"} />
    </button>
  );
}

interface FolderRowProps {
  folder: Folder;
  onOpen: () => void;
  onToggleFavorite: () => void;
  onDuplicate: () => void;
  onRename: () => void;
  onMove: () => void;
  onDelete: () => void;
}

function FolderRow({ folder, onOpen, onToggleFavorite, onDuplicate, onRename, onMove, onDelete }: FolderRowProps) {
  return (
    <li className="flex items-center gap-3 rounded-md border border-primary-muted/20 bg-surface px-3 py-2">
      <FavoriteButton isFavorite={folder.isFavorite} onToggle={onToggleFavorite} />
      <FolderIcon size={18} className="flex-shrink-0 text-primary-muted" />
      <button onClick={onOpen} className="min-w-0 flex-1 truncate text-left font-body text-sm text-ink hover:underline">
        {folder.name}
      </button>
      <RowActions onDuplicate={onDuplicate} onRename={onRename} onMove={onMove} onDelete={onDelete} />
    </li>
  );
}

interface FileRowProps {
  file: ManagedFile;
  onSchedule: () => void;
  onToggleFavorite: () => void;
  onDuplicate: () => void;
  onRename: () => void;
  onMove: () => void;
  onDelete: () => void;
}

function FileRow({ file, onSchedule, onToggleFavorite, onDuplicate, onRename, onMove, onDelete }: FileRowProps) {
  return (
    <li className="flex items-center gap-3 rounded-md border border-primary-muted/20 bg-surface px-3 py-2">
      <FavoriteButton isFavorite={file.isFavorite} onToggle={onToggleFavorite} />
      <FileIcon size={18} className="flex-shrink-0 text-ink-muted" />
      <div className="min-w-0 flex-1">
        <p className="truncate font-body text-sm text-ink">{file.currentFilename}</p>
        <p className="font-mono text-xs text-ink-muted">{formatSize(file.sizeBytes)}</p>
      </div>
      <button
        onClick={onSchedule}
        className="flex items-center gap-1 rounded-md border-2 border-primary-muted/40 px-2 py-1 font-body text-xs text-ink-muted hover:border-accent hover:text-accent"
      >
        <CalendarClock size={14} />
        Agendar
      </button>
      <RowActions onDuplicate={onDuplicate} onRename={onRename} onMove={onMove} onDelete={onDelete} />
    </li>
  );
}

/**
 * Acoes "estruturais" (mudam o item em si ou criam uma copia dele) --
 * diferente do botao de favoritar, que e so uma preferencia de exibicao.
 * `onDuplicate` foi adicionado aqui, junto de renomear/mover/excluir, em
 * vez de ficar ao lado da estrela: duplicar e da mesma natureza dessas
 * tres (uma operacao sobre o item), nao uma preferencia como favoritar.
 */
function RowActions({
  onDuplicate,
  onRename,
  onMove,
  onDelete,
}: {
  onDuplicate: () => void;
  onRename: () => void;
  onMove: () => void;
  onDelete: () => void;
}) {
  return (
    <div className="flex flex-shrink-0 gap-1 text-ink-muted">
      <button onClick={onDuplicate} aria-label="Duplicar" className="p-1 hover:text-ink">
        <Copy size={15} />
      </button>
      <button onClick={onRename} aria-label="Renomear" className="p-1 hover:text-ink">
        <Pencil size={15} />
      </button>
      <button onClick={onMove} aria-label="Mover" className="p-1 hover:text-ink">
        <FolderInput size={15} />
      </button>
      <button onClick={onDelete} aria-label="Excluir" className="p-1 hover:text-danger">
        <Trash2 size={15} />
      </button>
    </div>
  );
}
