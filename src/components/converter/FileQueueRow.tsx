import { AlertTriangle, CheckCircle2, XCircle } from "lucide-react";
import { ProgressBar } from "@/components/ui/ProgressBar";
import type { QueuedFile } from "@/types/conversion";

interface FileQueueRowProps {
  file: QueuedFile;
  /** So usado (e exibido) no modo nao-compacto -- um item concluido ja tem
   * seu caminho final mostrado, nao precisa mais do par origem/destino. */
  targetExtension?: string;
  onRemove: (id: string) => void;
  onCancel: (id: string) => void;
  /**
   * Fase 3: usado na secao "Concluidos" para um item ja terminado ocupar
   * menos espaco (sem area reservada pra barra de progresso ou botao de
   * cancelar) -- e o que mantem a lista organizada depois de varias
   * conversoes, em vez de criar um componente quase identico so pra isso.
   */
  compact?: boolean;
}

export function FileQueueRow({ file, targetExtension, onRemove, onCancel, compact = false }: FileQueueRowProps) {
  const isConverting = file.status === "converting";

  if (compact) {
    return (
      <li className="flex items-center gap-3 rounded-md border border-primary-muted/20 bg-surface px-3 py-1.5">
        <CheckCircle2 size={15} className="flex-shrink-0 text-primary" />
        <p className="min-w-0 flex-1 truncate font-body text-xs text-ink-muted">{file.fileName}</p>
        {file.outputPath && (
          <p className="hidden max-w-[40%] truncate font-mono text-xs text-primary sm:block">
            {file.outputPath}
          </p>
        )}
        <button
          onClick={() => onRemove(file.id)}
          className="flex-shrink-0 text-ink-muted hover:text-danger"
          aria-label={`Remover ${file.fileName} da lista`}
        >
          <XCircle size={15} />
        </button>
      </li>
    );
  }

  return (
    <li className="flex flex-col gap-2 rounded-md border-2 border-primary-muted/30 bg-surface px-3 py-2.5">
      <div className="flex items-center gap-3">
        <div className="min-w-0 flex-1">
          <p className="truncate font-body text-sm text-ink">{file.fileName}</p>
          <p className="font-mono text-xs text-ink-muted">
            .{file.sourceExtension} &rarr; .{targetExtension}
          </p>
        </div>
        <StatusBadge file={file} />
        {isConverting ? (
          <button
            onClick={() => onCancel(file.id)}
            className="font-body text-xs font-medium text-ink-muted underline-offset-2 hover:text-danger hover:underline"
          >
            Cancelar
          </button>
        ) : (
          <button
            onClick={() => onRemove(file.id)}
            className="text-ink-muted hover:text-danger"
            aria-label={`Remover ${file.fileName}`}
          >
            <XCircle size={18} />
          </button>
        )}
      </div>
      <ProgressBar active={isConverting} percent={file.progressPercent} />
      {file.status === "error" && <p className="font-body text-xs text-danger">{file.errorMessage}</p>}
    </li>
  );
}

function StatusBadge({ file }: { file: QueuedFile }) {
  switch (file.status) {
    // "done" normalmente so aparece no modo compacto (ver acima), que tem
    // seu proprio icone -- mantido aqui tambem por seguranca, caso este
    // componente um dia seja chamado para um arquivo concluido fora da
    // secao "Concluidos".
    case "done":
      return <CheckCircle2 size={18} className="flex-shrink-0 text-primary" />;
    case "error":
      return <AlertTriangle size={18} className="flex-shrink-0 text-danger" />;
    case "converting":
      return typeof file.progressPercent === "number" ? (
        <span className="flex-shrink-0 font-mono text-xs text-accent">
          {file.progressPercent.toFixed(0)}%
        </span>
      ) : (
        <span className="flex-shrink-0 font-body text-xs text-accent">Convertendo</span>
      );
    default:
      return <span className="flex-shrink-0 font-body text-xs text-ink-muted">Pronto</span>;
  }
}
