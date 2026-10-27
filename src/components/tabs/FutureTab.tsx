import { Sparkles } from "lucide-react";

/**
 * Aba 3 do briefing: propositalmente vazia nesta fase. Nenhuma funcionalidade
 * deve ser implementada aqui ainda -- apenas a mesma linguagem visual das
 * outras abas, para o espaco reservado nao destoar do resto do app.
 */
export function FutureTab() {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 text-center">
      <div className="flex h-14 w-14 items-center justify-center rounded-lg border-2 border-dashed border-primary-muted/50 text-primary-muted">
        <Sparkles size={24} strokeWidth={1.75} />
      </div>
      <p className="font-display text-sm font-semibold text-ink">Reservado para o futuro</p>
      <p className="max-w-xs font-body text-sm text-ink-muted">
        Esta aba ainda nao tem funcionalidade definida. O espaco foi deixado pronto para a
        proxima etapa do projeto.
      </p>
    </div>
  );
}
