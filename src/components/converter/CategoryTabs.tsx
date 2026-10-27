import { CATEGORY_LABELS } from "@/lib/formats";
import type { FileCategory } from "@/types/conversion";

interface CategoryTabsProps {
  categories: FileCategory[];
  active: FileCategory;
  onSelect: (category: FileCategory) => void;
}

/**
 * So renderizado quando ha mais de uma categoria com arquivos ainda nao
 * convertidos (ver ConverterTab) -- o caso comum, de uma categoria por vez,
 * nunca mostra isto: o seletor de formato da categoria em foco aparece
 * sozinho. Isto so existe para o caso raro de, por exemplo, ter uma imagem
 * E um video pendentes ao mesmo tempo, sem esconder um dos dois sem jeito
 * de voltar a ele.
 */
export function CategoryTabs({ categories, active, onSelect }: CategoryTabsProps) {
  if (categories.length < 2) return null;

  return (
    <div className="flex gap-1.5">
      {categories.map((category) => (
        <button
          key={category}
          onClick={() => onSelect(category)}
          className={`rounded-sm px-2 py-1 font-display text-xs font-medium transition-colors
            ${
              category === active
                ? "bg-primary text-primary-foreground"
                : "text-ink-muted hover:bg-surface-warm/50"
            }`}
        >
          {CATEGORY_LABELS[category]}
        </button>
      ))}
    </div>
  );
}
