import { useState } from "react";
import { AlertTriangle, CheckCircle2, Clock, RotateCcw, Trash2, XCircle } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { cancelJob, deleteJob, formatUnixMs, localDateTimeToUnixMs, retryJob } from "@/lib/manager-api";
import type { JobStatus, JobView } from "@/types/manager";

interface JobsPanelProps {
  jobs: JobView[];
  onChanged: () => void;
}

const STATUS_LABEL: Record<JobStatus, string> = {
  pending: "Agendado",
  running: "Convertendo",
  completed: "Concluido",
  failed: "Falhou",
  cancelled: "Cancelado",
};

function StatusIcon({ status }: { status: JobStatus }) {
  switch (status) {
    case "completed":
      return <CheckCircle2 size={16} className="text-primary" />;
    case "failed":
      return <AlertTriangle size={16} className="text-danger" />;
    case "cancelled":
      return <XCircle size={16} className="text-ink-muted" />;
    case "running":
      return <span className="font-mono text-xs text-accent">...</span>;
    default:
      return <Clock size={16} className="text-primary-muted" />;
  }
}

export function JobsPanel({ jobs, onChanged }: JobsPanelProps) {
  const [busyId, setBusyId] = useState<number | null>(null);

  if (jobs.length === 0) {
    return (
      <p className="py-6 text-center font-body text-sm text-ink-muted">
        Nenhum agendamento ainda. Selecione um arquivo na lista e clique em "Agendar".
      </p>
    );
  }

  async function handleCancel(id: number) {
    setBusyId(id);
    try {
      await cancelJob(id);
      onChanged();
    } catch (err) {
      console.error("Falha ao cancelar agendamento:", err);
    } finally {
      setBusyId(null);
    }
  }

  async function handleRetry(id: number) {
    // Reagenda para daqui a 1 minuto por padrao -- simples o bastante para
    // nao precisar reabrir o dialogo completo so para tentar de novo; o
    // usuario ainda pode cancelar e criar um novo agendamento com outro
    // horario se quiser algo mais especifico.
    const inOneMinute = localDateTimeToUnixMs(
      new Date(Date.now() + 60_000).toISOString().slice(0, 10),
      new Date(Date.now() + 60_000).toTimeString().slice(0, 5),
    );
    if (inOneMinute === null) return;

    setBusyId(id);
    try {
      await retryJob(id, inOneMinute);
      onChanged();
    } catch (err) {
      console.error("Falha ao reagendar:", err);
    } finally {
      setBusyId(null);
    }
  }

  /**
   * Botao de lixeira (Fase 7): remove a ENTRADA DO HISTORICO, nunca um
   * arquivo. Disponivel para qualquer status, exceto "running" -- um job
   * que esta convertendo de verdade agora nao pode sumir da lista so por
   * um clique, a mesma regra ja usada para conversoes ativas na Aba 1. Sem
   * dialogo de confirmacao de proposito: e o mesmo nivel de acao que
   * remover um item pendente da fila na Aba 1 (um "X" direto), nao o
   * equivalente a excluir um arquivo de verdade.
   */
  async function handleDelete(id: number) {
    setBusyId(id);
    try {
      await deleteJob(id);
      onChanged();
    } catch (err) {
      console.error("Falha ao remover agendamento do historico:", err);
    } finally {
      setBusyId(null);
    }
  }

  return (
    <ul className="flex flex-col gap-1.5">
      {jobs.map((job) => (
        <li
          key={job.id}
          className="flex items-center gap-3 rounded-md border border-primary-muted/20 bg-surface px-3 py-2"
        >
          <StatusIcon status={job.status} />
          <div className="min-w-0 flex-1">
            <p className="truncate font-body text-sm text-ink">{job.fileName}</p>
            <p className="font-mono text-xs text-ink-muted">
              &rarr; .{job.targetExtension} &middot; {STATUS_LABEL[job.status]} &middot;{" "}
              {formatUnixMs(job.scheduledAtUnixMs)}
            </p>
            {job.status === "failed" && job.errorMessage && (
              <p className="mt-0.5 truncate font-body text-xs text-danger">{job.errorMessage}</p>
            )}
          </div>
          {job.status === "pending" && (
            <Button variant="ghost" disabled={busyId === job.id} onClick={() => handleCancel(job.id)}>
              Cancelar
            </Button>
          )}
          {(job.status === "failed" || job.status === "cancelled") && (
            <Button
              variant="outline"
              icon={<RotateCcw size={14} />}
              disabled={busyId === job.id}
              onClick={() => handleRetry(job.id)}
            >
              Tentar de novo
            </Button>
          )}
          {job.status !== "running" && (
            <button
              onClick={() => handleDelete(job.id)}
              disabled={busyId === job.id}
              aria-label="Remover do historico"
              className="flex-shrink-0 p-1 text-ink-muted hover:text-danger disabled:opacity-50"
            >
              <Trash2 size={15} />
            </button>
          )}
        </li>
      ))}
    </ul>
  );
}
