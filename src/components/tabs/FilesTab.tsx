import { useCallback, useEffect, useState } from "react";
import { FileBrowser } from "@/components/files/FileBrowser";
import { JobsPanel } from "@/components/files/JobsPanel";
import { listJobs } from "@/lib/manager-api";
import type { JobView } from "@/types/manager";

/**
 * Aba 2: gerenciador de arquivos local + agendamento de conversoes.
 *
 * Fica de proposito como um orquestrador fino -- FileBrowser cuida de
 * pastas/arquivos, JobsPanel so exibe e age sobre agendamentos (cancelar,
 * tentar de novo). Nenhum dos dois conhece o outro diretamente; esta aba e
 * quem decide que criar um agendamento (em FileBrowser) deve atualizar a
 * lista mostrada em JobsPanel, via `refreshJobs`.
 *
 * Nao reusa nem modifica nada da Aba 1 (Converter): formatos de arquivo vem
 * do mesmo registro compartilhado (`lib/formats.ts`), mas a logica de fila
 * de conversao imediata da Aba 1 e o agendamento desta aba sao coisas
 * diferentes e ficam em componentes separados.
 */
export function FilesTab() {
  const [jobs, setJobs] = useState<JobView[]>([]);

  const refreshJobs = useCallback(() => {
    listJobs()
      .then(setJobs)
      .catch((err) => console.error("Falha ao listar agendamentos:", err));
  }, []);

  useEffect(() => {
    refreshJobs();
    // Atualiza a lista periodicamente para refletir jobs que o agendador em
    // segundo plano foi executando sozinho, sem precisar de um sistema de
    // eventos dedicado so para isto -- um intervalo simples e suficiente
    // aqui, bem mais barato que manter um listener de evento o tempo todo
    // para uma lista que muda com pouca frequencia.
    const interval = setInterval(refreshJobs, 15_000);
    return () => clearInterval(interval);
  }, [refreshJobs]);

  return (
    <div className="flex h-full flex-col gap-5">
      <div>
        <h1 className="font-display text-lg font-semibold text-ink">Arquivos e Pastas</h1>
        <p className="font-body text-sm text-ink-muted">
          Organize os arquivos importados em pastas e agende conversoes para uma data e hora
          especificas.
        </p>
      </div>

      <div className="min-h-0 flex-1">
        <FileBrowser onJobScheduled={refreshJobs} />
      </div>

      <div className="flex max-h-56 flex-col gap-2 border-t-2 border-primary-muted/20 pt-4">
        <p className="font-display text-xs font-semibold uppercase tracking-wide text-ink-muted">
          Agendamentos ({jobs.length})
        </p>
        <div className="overflow-y-auto">
          <JobsPanel jobs={jobs} onChanged={refreshJobs} />
        </div>
      </div>
    </div>
  );
}
