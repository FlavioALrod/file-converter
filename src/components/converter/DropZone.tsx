import { FileUp, UploadCloud } from "lucide-react";
import { Button } from "@/components/ui/Button";

interface DropZoneProps {
  isDragHovering: boolean;
  onPickFiles: () => void;
}

/**
 * Extraido de ConverterTab.tsx na Fase 2: o componente nao mudou de
 * comportamento, so ganhou vida propria para a aba principal nao virar um
 * arquivo so com tudo dentro (a fila de arquivos cresceu bastante com
 * progresso e cancelamento).
 */
export function DropZone({ isDragHovering, onPickFiles }: DropZoneProps) {
  return (
    <div
      className={`flex flex-col items-center justify-center gap-3 rounded-lg border-2 border-dashed
        px-6 py-8 text-center transition-colors duration-150
        ${isDragHovering ? "border-accent bg-accent/10" : "border-primary-muted/50 bg-surface-warm/30"}`}
    >
      <UploadCloud
        size={32}
        strokeWidth={1.75}
        className={isDragHovering ? "text-accent" : "text-primary-muted"}
      />
      <p className="font-display text-sm font-semibold text-ink">
        {isDragHovering ? "Solte para adicionar" : "Arraste arquivos aqui"}
      </p>
      <p className="font-body text-xs text-ink-muted">
        Imagens, documentos, audio ou video
      </p>
      <Button variant="outline" icon={<FileUp size={16} />} onClick={onPickFiles}>
        Selecionar do computador
      </Button>
    </div>
  );
}
