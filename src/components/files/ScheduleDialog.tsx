import { useMemo, useState } from "react";
import { Button } from "@/components/ui/Button";
import { Callout } from "@/components/ui/Callout";
import { categoryOfExtension, targetFormatsFor } from "@/lib/formats";
import { createJob, localDateTimeToUnixMs } from "@/lib/manager-api";
import type { ManagedFile } from "@/types/manager";

interface ScheduleDialogProps {
  file: ManagedFile | null;
  onScheduled: () => void;
  onCancel: () => void;
}

/**
 * Reaproveita o MESMO registro de formatos da Aba 1 (`lib/formats.ts`) para
 * a lista de extensoes de destino -- a propria especificacao desta fase
 * pede para nao duplicar a logica de formato suportado entre as abas.
 */
export function ScheduleDialog({ file, onScheduled, onCancel }: ScheduleDialogProps) {
  const [targetExtension, setTargetExtension] = useState<string | null>(null);
  const [dateValue, setDateValue] = useState("");
  const [timeValue, setTimeValue] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  const category = file ? categoryOfExtension(file.extension) : undefined;
  const formatOptions = useMemo(() => (category ? targetFormatsFor(category) : []), [category]);
  const effectiveTarget = targetExtension ?? formatOptions[0]?.extension ?? null;

  if (!file) return null;

  async function handleConfirm() {
    if (!file || !effectiveTarget) return;
    const scheduledAtUnixMs = localDateTimeToUnixMs(dateValue, timeValue);
    if (scheduledAtUnixMs === null) {
      setError("Escolha uma data e um horario validos.");
      return;
    }

    setIsSaving(true);
    setError(null);
    try {
      await createJob(file.id, effectiveTarget, scheduledAtUnixMs);
      onScheduled();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsSaving(false);
    }
  }

  const isPastDateTime =
    dateValue && timeValue && (localDateTimeToUnixMs(dateValue, timeValue) ?? Infinity) < Date.now();

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-ink/40 p-4" onClick={onCancel}>
      <div
        role="dialog"
        aria-modal="true"
        className="w-full max-w-sm rounded-lg border-2 border-primary-muted/40 bg-surface p-5 shadow-hard"
        onClick={(event) => event.stopPropagation()}
      >
        <h2 className="font-display text-base font-semibold text-ink">Agendar conversao</h2>
        <p className="mt-1 truncate font-body text-sm text-ink-muted">{file.currentFilename}</p>

        <p className="mb-1.5 mt-4 font-display text-xs font-semibold uppercase tracking-wide text-ink-muted">
          Converter para
        </p>
        <div className="flex flex-wrap gap-2">
          {formatOptions.map((format) => (
            <button
              key={format.extension}
              onClick={() => setTargetExtension(format.extension)}
              className={`rounded-md border-2 px-3 py-1.5 font-mono text-xs font-medium transition-colors
                ${
                  effectiveTarget === format.extension
                    ? "border-accent bg-accent text-surface"
                    : "border-primary-muted/40 text-ink-muted hover:border-primary-muted"
                }`}
            >
              .{format.extension}
            </button>
          ))}
        </div>

        <div className="mt-4 flex gap-2">
          <div className="flex-1">
            <label className="block font-body text-xs font-medium text-ink-muted">Data</label>
            <input
              type="date"
              value={dateValue}
              onChange={(event) => setDateValue(event.target.value)}
              className="mt-1 w-full rounded-md border-2 border-primary-muted/40 bg-surface px-2 py-1.5 font-body text-sm text-ink outline-none focus:border-primary"
            />
          </div>
          <div className="flex-1">
            <label className="block font-body text-xs font-medium text-ink-muted">Hora</label>
            <input
              type="time"
              value={timeValue}
              onChange={(event) => setTimeValue(event.target.value)}
              className="mt-1 w-full rounded-md border-2 border-primary-muted/40 bg-surface px-2 py-1.5 font-body text-sm text-ink outline-none focus:border-primary"
            />
          </div>
        </div>

        {isPastDateTime && (
          <div className="mt-3">
            <Callout tone="info">
              Esse horario ja passou. O agendamento sera executado assim que voce confirmar aqui
              (a mesma regra usada para agendamentos atrasados ao reabrir o app).
            </Callout>
          </div>
        )}
        {error && <p className="mt-3 font-body text-xs text-danger">{error}</p>}

        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" onClick={onCancel}>
            Cancelar
          </Button>
          <Button
            variant="accent"
            disabled={!effectiveTarget || !dateValue || !timeValue || isSaving}
            onClick={handleConfirm}
          >
            {isSaving ? "Agendando..." : "Agendar"}
          </Button>
        </div>
      </div>
    </div>
  );
}
