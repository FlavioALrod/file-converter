import { useEffect, useState } from "react";
import { Button } from "@/components/ui/Button";
import { listAllFolders } from "@/lib/manager-api";
import type { Folder } from "@/types/manager";

interface MoveDialogProps {
  open: boolean;
  /** Pasta que o item esta hoje -- nao pode ser escolhida como destino (nao faz sentido). */
  currentFolderId: number | null;
  /** Se estiver movendo uma pasta, o id dela: nao pode ser movida para dentro de si mesma. */
  excludeFolderId?: number | null;
  onConfirm: (destinationId: number | null) => void;
  onCancel: () => void;
}

interface FolderOption {
  id: number | null;
  label: string;
}

/** Monta a lista achatada com indentacao visual a partir da relacao parent_id. */
function buildOptions(folders: Folder[], excludeId: number | null | undefined): FolderOption[] {
  const byParent = new Map<number | null, Folder[]>();
  for (const folder of folders) {
    const list = byParent.get(folder.parentId) ?? [];
    list.push(folder);
    byParent.set(folder.parentId, list);
  }

  const options: FolderOption[] = [{ id: null, label: "Raiz" }];

  function walk(parentId: number | null, depth: number) {
    const children = byParent.get(parentId) ?? [];
    for (const child of children) {
      if (child.id === excludeId) continue; // nao pode mover para dentro de si mesma
      options.push({ id: child.id, label: `${"  ".repeat(depth)}${depth > 0 ? "\u2514 " : ""}${child.name}` });
      walk(child.id, depth + 1);
    }
  }
  walk(null, 0);

  return options;
}

export function MoveDialog({ open, currentFolderId, excludeFolderId, onConfirm, onCancel }: MoveDialogProps) {
  const [options, setOptions] = useState<FolderOption[]>([{ id: null, label: "Raiz" }]);
  const [selected, setSelected] = useState<number | null>(null);

  useEffect(() => {
    if (!open) return;
    listAllFolders()
      .then((folders) => setOptions(buildOptions(folders, excludeFolderId)))
      .catch((err) => console.error("Falha ao listar pastas:", err));
  }, [open, excludeFolderId]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-ink/40 p-4" onClick={onCancel}>
      <div
        role="dialog"
        aria-modal="true"
        className="w-full max-w-sm rounded-lg border-2 border-primary-muted/40 bg-surface p-5 shadow-hard"
        onClick={(event) => event.stopPropagation()}
      >
        <h2 className="font-display text-base font-semibold text-ink">Mover para...</h2>
        <select
          autoFocus
          value={selected ?? "root"}
          onChange={(event) => setSelected(event.target.value === "root" ? null : Number(event.target.value))}
          className="mt-3 w-full rounded-md border-2 border-primary-muted/40 bg-surface px-3 py-2 font-mono text-sm text-ink outline-none focus:border-primary"
        >
          {options.map((option) => (
            <option key={option.id ?? "root"} value={option.id ?? "root"}>
              {option.label}
            </option>
          ))}
        </select>
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" onClick={onCancel}>
            Cancelar
          </Button>
          <Button
            variant="accent"
            disabled={selected === currentFolderId}
            onClick={() => onConfirm(selected)}
          >
            Mover
          </Button>
        </div>
      </div>
    </div>
  );
}
