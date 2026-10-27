import type { ReactNode } from "react";
import { AlertTriangle, Info } from "lucide-react";

interface CalloutProps {
  tone?: "warning" | "info";
  children: ReactNode;
}

/**
 * Faixa de aviso reutilizavel. Nasceu do aviso de vetorizacao SVG da Fase 1
 * (que estava com o JSX duplicado direto em ConverterTab.tsx) e agora
 * tambem e usada para o aviso de fidelidade de PDF->documento e o de perda
 * de qualidade em audio/video -- um unico lugar para o estilo do aviso em
 * vez de tres blocos JSX quase identicos.
 */
export function Callout({ tone = "info", children }: CalloutProps) {
  const toneClasses =
    tone === "warning" ? "border-accent/40 bg-accent/10 text-ink" : "border-primary-muted/40 bg-surface-warm/30 text-ink";
  const Icon = tone === "warning" ? AlertTriangle : Info;
  const iconClasses = tone === "warning" ? "text-accent" : "text-primary-muted";

  return (
    <div className={`flex items-start gap-2 rounded-md border-2 px-3 py-2 ${toneClasses}`}>
      <Icon size={16} className={`mt-0.5 flex-shrink-0 ${iconClasses}`} />
      <p className="font-body text-xs">{children}</p>
    </div>
  );
}
