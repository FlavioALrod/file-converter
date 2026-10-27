import type { FormatOption } from "@/types/conversion";

interface FormatPickerProps {
  groupLabel: string;
  options: FormatOption[];
  selected: string;
  onSelect: (extension: string) => void;
}

/**
 * Um grupo de pilulas de formato para UMA categoria. ConverterTab renderiza
 * um destes por categoria presente na fila (ver secao "Converter para"),
 * em vez de um unico seletor genuino que confundiria quando a fila tem
 * arquivos de tipos diferentes -- e o que a especificacao pede com
 * "use contextual options so the interface displays only the settings
 * relevant to the selected file type".
 */
export function FormatPicker({ groupLabel, options, selected, onSelect }: FormatPickerProps) {
  return (
    <div>
      <p className="mb-1.5 font-display text-xs font-semibold uppercase tracking-wide text-ink-muted">
        {groupLabel}
      </p>
      <div className="flex flex-wrap gap-2">
        {options.map((format) => (
          <button
            key={format.extension}
            onClick={() => onSelect(format.extension)}
            className={`rounded-md border-2 px-3 py-1.5 font-mono text-xs font-medium transition-colors
              ${
                selected === format.extension
                  ? "border-accent bg-accent text-surface"
                  : "border-primary-muted/40 text-ink-muted hover:border-primary-muted"
              }`}
          >
            .{format.extension}
          </button>
        ))}
      </div>
    </div>
  );
}
