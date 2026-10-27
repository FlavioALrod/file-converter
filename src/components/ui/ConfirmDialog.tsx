import { useEffect } from "react";
import { Button } from "@/components/ui/Button";

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  description: string;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

/**
 * Modal de confirmacao proprio do app, no lugar do `window.confirm()`
 * nativo do sistema operacional (auditoria Fase 4). Duas razoes:
 *
 * 1. Design: um dialogo nativo do SO quebra completamente a identidade
 *    visual do app (paleta, tipografia, bordas) -- a propria especificacao
 *    pede para toda funcionalidade seguir o design system existente.
 * 2. Previsibilidade: `window.confirm` depende do WebView do sistema
 *    operacional, que varia entre plataformas e nao e o tipo de dependencia
 *    externa que vale a pena ter para algo tao central quanto "o usuario
 *    tem certeza que quer limpar a tela".
 *
 * Generico de proposito -- outras acoes destrutivas do app (ex.: excluir
 * uma pasta na Aba 2, no futuro) podem reaproveitar este mesmo componente.
 */
export function ConfirmDialog({
  open,
  title,
  description,
  confirmLabel = "Confirmar",
  cancelLabel = "Cancelar",
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  useEffect(() => {
    if (!open) return;
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onCancel();
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onCancel]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-ink/40 p-4"
      onClick={onCancel}
    >
      <div
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="confirm-dialog-title"
        className="w-full max-w-sm rounded-lg border-2 border-primary-muted/40 bg-surface p-5 shadow-hard"
        onClick={(event) => event.stopPropagation()}
      >
        <h2 id="confirm-dialog-title" className="font-display text-base font-semibold text-ink">
          {title}
        </h2>
        <p className="mt-2 font-body text-sm text-ink-muted">{description}</p>
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" onClick={onCancel}>
            {cancelLabel}
          </Button>
          <Button variant="accent" onClick={onConfirm}>
            {confirmLabel}
          </Button>
        </div>
      </div>
    </div>
  );
}
