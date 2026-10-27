interface ProgressBarProps {
  active: boolean;
  /** 0-100. Quando ausente, usa a animacao indeterminada (imagem/documento). */
  percent?: number;
}

/**
 * Barra de progresso achatada (sem gradiente, sem brilho). Mostra uma
 * porcentagem real quando o motor reporta uma (midia, via evento
 * "conversion-progress"); cai para a animacao indeterminada quando nao
 * (imagem, documento) -- ainda assim da retorno visual claro de que algo
 * esta acontecendo, como pedido na especificacao.
 */
export function ProgressBar({ active, percent }: ProgressBarProps) {
  if (!active) return null;

  const isDeterminate = typeof percent === "number";

  return (
    <div className="h-1.5 w-full overflow-hidden rounded-sm bg-surface-warm">
      {isDeterminate ? (
        <div
          className="h-full rounded-sm bg-accent transition-[width] duration-200 ease-linear"
          style={{ width: `${Math.max(0, Math.min(100, percent)).toFixed(0)}%` }}
        />
      ) : (
        <div className="h-full w-1/3 animate-progress-slide rounded-sm bg-accent" />
      )}
    </div>
  );
}
