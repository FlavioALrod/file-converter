import { useState } from "react";
import { ChevronDown, ChevronUp } from "lucide-react";
import { FileQueueRow } from "@/components/converter/FileQueueRow";
import type { QueuedFile } from "@/types/conversion";

interface CompletedSectionProps {
  files: QueuedFile[];
  onRemove: (id: string) => void;
}

/**
 * Lista separada para resultados concluidos, em vez de misturar com a fila
 * ativa -- e o que evita que a tela "va enchendo" conforme o usuario
 * converte varios arquivos ao longo da sessao. Comeca expandida (nada
 * desaparece sem o usuario escolher) e o usuario pode recolher para
 * concentrar a atencao no que ainda esta pendente.
 */
export function CompletedSection({ files, onRemove }: CompletedSectionProps) {
  const [isCollapsed, setIsCollapsed] = useState(false);

  if (files.length === 0) return null;

  return (
    <div className="flex flex-col gap-2">
      <button
        onClick={() => setIsCollapsed((prev) => !prev)}
        className="flex items-center gap-1.5 self-start font-display text-xs font-semibold uppercase tracking-wide text-ink-muted hover:text-ink"
      >
        {isCollapsed ? <ChevronDown size={14} /> : <ChevronUp size={14} />}
        Concluidos ({files.length})
      </button>
      {!isCollapsed && (
        <ul className="flex flex-col gap-1.5">
          {files.map((file) => (
            <FileQueueRow key={file.id} file={file} onRemove={onRemove} onCancel={() => {}} compact />
          ))}
        </ul>
      )}
    </div>
  );
}
