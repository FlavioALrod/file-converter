import { useEffect, useState } from "react";
import { Button } from "@/components/ui/Button";

interface PromptDialogProps {
  open: boolean;
  title: string;
  label: string;
  initialValue?: string;
  confirmLabel?: string;
  onConfirm: (value: string) => void;
  onCancel: () => void;
}

/**
 * Modal de texto generico -- mesma casca visual do ConfirmDialog (Fase 4),
 * so que com um campo de entrada em vez de so confirmar/cancelar. Usado
 * para nomear uma pasta nova e para renomear pastas/arquivos (ver
 * FileBrowser.tsx), para nao duplicar o mesmo modal duas vezes.
 */
export function PromptDialog({
  open,
  title,
  label,
  initialValue = "",
  confirmLabel = "Confirmar",
  onConfirm,
  onCancel,
}: PromptDialogProps) {
  const [value, setValue] = useState(initialValue);

  useEffect(() => {
    if (open) setValue(initialValue);
  }, [open, initialValue]);

  useEffect(() => {
    if (!open) return;
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onCancel();
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [open, onCancel]);

  if (!open) return null;

  function submit() {
    const trimmed = value.trim();
    if (trimmed.length === 0) return;
    onConfirm(trimmed);
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-ink/40 p-4" onClick={onCancel}>
      <div
        role="dialog"
        aria-modal="true"
        className="w-full max-w-sm rounded-lg border-2 border-primary-muted/40 bg-surface p-5 shadow-hard"
        onClick={(event) => event.stopPropagation()}
      >
        <h2 className="font-display text-base font-semibold text-ink">{title}</h2>
        <label className="mt-3 block font-body text-xs font-medium text-ink-muted">{label}</label>
        <input
          autoFocus
          value={value}
          onChange={(event) => setValue(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") submit();
          }}
          className="mt-1 w-full rounded-md border-2 border-primary-muted/40 bg-surface px-3 py-2 font-body text-sm text-ink outline-none focus:border-primary"
        />
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" onClick={onCancel}>
            Cancelar
          </Button>
          <Button variant="accent" disabled={value.trim().length === 0} onClick={submit}>
            {confirmLabel}
          </Button>
        </div>
      </div>
    </div>
  );
}
