import { ArrowLeftRight, FolderTree, Sparkles } from "lucide-react";
import type { ReactNode } from "react";

export type TabId = "converter" | "files" | "future";

const TABS: { id: TabId; label: string; icon: ReactNode }[] = [
  { id: "converter", label: "Converter", icon: <ArrowLeftRight size={20} strokeWidth={2} /> },
  { id: "files", label: "Arquivos e Pastas", icon: <FolderTree size={20} strokeWidth={2} /> },
  { id: "future", label: "Em breve", icon: <Sparkles size={20} strokeWidth={2} /> },
];

/**
 * Nav lateral em vez do topo: evita o padrao generico de "pills" horizontais
 * e da ao app uma sensacao de ferramenta de bancada (trilho fixo, conteudo
 * passa por dentro) -- decisao de layout combinada com a identidade visual
 * industrial da paleta de referencia.
 */
export function TabRail({ active, onChange }: { active: TabId; onChange: (id: TabId) => void }) {
  return (
    <nav className="flex w-56 flex-shrink-0 flex-col gap-1 bg-primary px-3 py-6">
      <div className="mb-6 px-3">
        <p className="font-display text-base font-semibold text-primary-foreground">Conversor</p>
        <p className="font-display text-xs text-primary-foreground/70">de Arquivos</p>
      </div>

      {TABS.map((tab) => {
        const isActive = tab.id === active;
        return (
          <button
            key={tab.id}
            onClick={() => onChange(tab.id)}
            className={`flex items-center gap-3 rounded-md px-3 py-2.5 text-left font-display text-sm
              transition-colors duration-150
              ${
                isActive
                  ? "bg-surface text-ink"
                  : "text-primary-foreground/80 hover:bg-primary-muted/40 hover:text-primary-foreground"
              }`}
          >
            <span className={isActive ? "text-accent" : ""}>{tab.icon}</span>
            {tab.label}
          </button>
        );
      })}
    </nav>
  );
}
